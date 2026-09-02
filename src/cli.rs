// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # clincalc::cli
//!
//! The command-line surface for the open clinical calculators. This module is
//! the single source of CLI behaviour: the standalone `clincalc` binary
//! (`src/main.rs`) and any host CLI that embeds it (e.g. GitEHR's `gitehr calc`)
//! both drive [`CalcCommand`](crate::cli::CalcCommand) + [`run`](crate::cli::run), so there is nothing to re-implement
//! when embedding it. It is compiled only with the `cli` feature (on by default).
//!
//! ## One regular surface for every calculator
//!
//! There are no per-calculator flags. Every calculator is driven through the
//! same registry-backed shape, so adding a calculator to the registry gives it a
//! working CLI for free, and a human or an LLM learns the interface once:
//!
//! ```text
//! clincalc list                       # list available calculators
//! clincalc ls                         # same as list
//! clincalc calc <name>                # print a fillable INPUT TEMPLATE
//! clincalc calc <name> --schema       # print the JSON Schema (the full contract)
//! clincalc calc <name> --input -      # compute, reading JSON from stdin
//! clincalc <name> --input data.json   # shorthand for `clincalc calc <name>`
//! clincalc tags                       # list all tags
//! ```
//!
//! The template printed by `clincalc calc <name>` has the same shape as the input
//! `clincalc calc <name> --input` expects: fill in the placeholder values and pass it
//! back. Computing always requires an explicit `--input`, so a bare invocation
//! never blocks reading stdin. `clincalc <name>` remains supported as shorthand.
//!
//! To embed in a host CLI (e.g. gitehr):
//!
//! ```ignore
//! #[derive(clap::Subcommand)]
//! enum Commands {
//!     // ...
//!     /// Clinical calculators
//!     Calc(clincalc::cli::CalcCommand),
//! }
//! // ...
//! Commands::Calc(cmd) => clincalc::cli::run(cmd)?,
//! ```

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use clap::{Args, ValueEnum, ValueHint};

use crate::{COMPILED_LOCALES, CalculationResponse, Calculator, SupportedLocale, lookup_locale};

const CALCULATOR_ALIASES: &[(&str, &str)] = &[
    ("bmr", "energy_requirement"),
    ("ckd-epi", "egfr"),
    ("ckd_epi", "egfr"),
    ("ckdepi", "egfr"),
    ("ree", "energy_requirement"),
    ("rmr", "energy_requirement"),
    ("tdee", "energy_requirement"),
];

/// Named physical activity presets for energy-requirement calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ActivityPreset {
    /// Sedentary or little exercise: activity factor 1.2.
    Sedentary,
    /// Light exercise: activity factor 1.375.
    Light,
    /// Moderate exercise: activity factor 1.55.
    Moderate,
    /// Very active: activity factor 1.725.
    VeryActive,
    /// Extra active: activity factor 1.9.
    ExtraActive,
}

impl ActivityPreset {
    fn factor(self) -> f64 {
        match self {
            ActivityPreset::Sedentary => 1.2,
            ActivityPreset::Light => 1.375,
            ActivityPreset::Moderate => 1.55,
            ActivityPreset::VeryActive => 1.725,
            ActivityPreset::ExtraActive => 1.9,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            ActivityPreset::Sedentary => "sedentary",
            ActivityPreset::Light => "light",
            ActivityPreset::Moderate => "moderate",
            ActivityPreset::VeryActive => "very-active",
            ActivityPreset::ExtraActive => "extra-active",
        }
    }
}

/// Goal-directed energy target adjustment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum EnergyGoal {
    /// No calorie adjustment.
    Maintain,
    /// Negative calorie adjustment for weight loss.
    Lose,
    /// Positive calorie adjustment for weight gain.
    Gain,
}

impl EnergyGoal {
    fn slug(self) -> &'static str {
        match self {
            EnergyGoal::Maintain => "maintain",
            EnergyGoal::Lose => "lose",
            EnergyGoal::Gain => "gain",
        }
    }
}

/// How to render computed results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text.
    #[default]
    Text,
    /// Machine-readable JSON (the `CalculationResponse` schema).
    Json,
    /// Markdown with headings, bullet working steps, and a linked reference -
    /// for pasting into EHR free-text fields or notes apps.
    Markdown,
}

/// The `clincalc` command surface. Reused unchanged by host CLIs such as `gitehr calc`.
///
/// A single shape covers discovery, schema, and compute for every calculator;
/// the calculator is selected by `name` and looked up in the `clincalc`
/// registry at runtime.
#[derive(Debug, Args)]
pub struct CalcCommand {
    /// Calculator machine name (e.g. `feverpain`). Omit, or use `list`, to see
    /// all available calculators.
    pub name: Option<String>,

    /// Compute a result from this JSON input: `-` for stdin, a file path, or an
    /// inline JSON string. Without it, a fillable input template is printed.
    #[arg(long, value_name = "JSON|FILE|-", value_hint = ValueHint::AnyPath)]
    pub input: Option<String>,

    /// Print the calculator's JSON Schema (the full input contract) instead of a
    /// template.
    #[arg(long)]
    pub schema: bool,

    /// Print the calculator's distribution licence and the URL evidencing it.
    #[arg(long)]
    pub license: bool,

    /// Convenience activity preset for `energy_requirement` (and aliases such as `tdee`).
    /// Injects the corresponding `activity_factor` into JSON input.
    #[arg(long, value_enum, value_name = "PRESET")]
    pub activity: Option<ActivityPreset>,

    /// Energy target goal for `energy_requirement`: derive calorie adjustment from a weight-change rate.
    #[arg(long, value_enum, value_name = "lose|maintain|gain")]
    pub goal: Option<EnergyGoal>,

    /// Weight change rate in kg/week for `--goal lose|gain`.
    #[arg(long, value_name = "KG_PER_WEEK")]
    pub rate: Option<f64>,

    /// Optional target weight for estimating time to target with `--goal lose|gain --rate`.
    #[arg(long = "target-weight", alias = "target-weight-kg", value_name = "KG")]
    pub target_weight_kg: Option<f64>,

    /// Derive `lean_body_mass_kg` from `weight_kg` and body fat percentage, mainly for Cunningham.
    #[arg(long, value_name = "PERCENT")]
    pub body_fat_pct: Option<f64>,

    /// Prompt for required fields from the calculator's schema, writing prompts to stderr.
    #[arg(long)]
    pub interactive: bool,

    /// Fill missing fields from ~/.config/clincalc/profile.json.
    #[arg(long)]
    pub profile: bool,

    /// Fill missing fields from a JSON record/profile file.
    #[arg(long, value_name = "FILE", value_hint = ValueHint::AnyPath)]
    pub from_record: Option<String>,

    /// Human-friendly field input for calculators with an `equation` field.
    #[arg(long, value_name = "VALUE")]
    pub equation: Option<String>,

    /// Human-friendly field input for calculators with a `sex` field.
    #[arg(long, value_name = "male|female")]
    pub sex: Option<String>,

    /// Human-friendly field input for calculators with an `age` field.
    #[arg(long, value_name = "YEARS")]
    pub age: Option<u64>,

    /// Human-friendly field input for calculators with a `weight_kg` field.
    #[arg(long, value_name = "KG")]
    pub weight_kg: Option<f64>,

    /// Human-friendly field input for calculators with a `height_cm` field.
    #[arg(long, value_name = "CM")]
    pub height_cm: Option<f64>,

    /// Human-friendly field input for calculators with a `lean_body_mass_kg` field.
    #[arg(long, value_name = "KG")]
    pub lean_body_mass_kg: Option<f64>,

    /// Human-friendly field input for calculators with a `creatinine` field.
    #[arg(long, value_name = "VALUE")]
    pub creatinine: Option<f64>,

    /// Human-friendly field input for calculators with a `creatinine_unit` field.
    #[arg(long, value_name = "mg/dL|umol/L")]
    pub creatinine_unit: Option<String>,

    /// Human-friendly field input for calculators with a `calorie_adjustment_kcal_day` field.
    #[arg(long, value_name = "KCAL_PER_DAY")]
    pub calorie_adjustment_kcal_day: Option<f64>,

    /// Restrict `list` output to calculators that carry this tag (e.g.
    /// `cardiology`, `proprietary`, `nhs-mandated`). Repeat the flag to
    /// require ALL of several tags. With no calculator name, this filters the
    /// catalogue; with a name, it has no effect (the calculator is selected
    /// by name).
    #[arg(long, value_name = "TAG")]
    pub tag: Vec<String>,

    /// Instead of listing calculators, list every tag in the registry (with
    /// the number of calculators that carry it). Implies `list` mode.
    #[arg(long)]
    pub tags: bool,

    /// Output format for computed results and `list`.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
}

/// Arguments for `clincalc list` / `clincalc ls`.
#[derive(Debug, Args)]
pub struct ListCommand {
    /// Restrict output to calculators that carry this tag. Repeat to require ALL tags.
    #[arg(long, value_name = "TAG")]
    pub tag: Vec<String>,

    /// List every tag in the registry instead of calculators. Prefer `clincalc tags`; this is kept for compatibility.
    #[arg(long)]
    pub tags: bool,

    /// Output format for the listing.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
}

/// Arguments for `clincalc tags`.
#[derive(Debug, Args)]
pub struct TagsCommand {
    /// Output format for the tag listing.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
}

/// Arguments for `clincalc version`.
#[derive(Debug, Args)]
pub struct VersionCommand {
    /// Output format for version information.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
}

/// Dispatch a parsed [`CalcCommand`].
pub fn run(cmd: CalcCommand) -> Result<()> {
    run_with_locale(cmd, resolve_cli_locale(None)?)
}

/// Dispatch a parsed [`CalcCommand`] with an already resolved locale.
pub fn run_with_locale(cmd: CalcCommand, locale: SupportedLocale) -> Result<()> {
    // `--tags` lists every known tag (with calculator counts) and exits.
    if cmd.tags {
        return print_tags(cmd.format);
    }

    // No name (or legacy `list` / `ls`) means: show the catalogue (optionally filtered by --tag).
    let name = match cmd.name.as_deref() {
        None | Some("list") | Some("ls") => return print_list(cmd.format, &cmd.tag, locale),
        Some(n) => n,
    };

    let canonical_name = canonical_calculator_name(name);
    let calc =
        crate::get(canonical_name).ok_or_else(|| anyhow!(unknown_calculator_message(name)))?;
    ensure_calculator_locale(calc.as_ref(), locale)?;

    if has_energy_only_flags(&cmd) && calc.name() != "energy_requirement" {
        return Err(anyhow!(
            "--activity, --goal, --rate, --target-weight, and --body-fat-pct are only supported for energy_requirement (aliases: bmr, ree, rmr, tdee)"
        ));
    }

    // `--schema` prints the formal contract, regardless of everything else.
    if cmd.schema {
        println!(
            "{}",
            serde_json::to_string_pretty(&calc.input_schema_for(locale))?
        );
        return Ok(());
    }

    // `--license` prints the algorithm's distribution licence and evidence URL.
    if cmd.license {
        println!("{}", serde_json::to_string_pretty(&calc.license())?);
        return Ok(());
    }

    if cmd.interactive && cmd.input.is_some() {
        return Err(anyhow!(
            "--interactive cannot be combined with --input; use one input source"
        ));
    }

    match cmd.input.as_deref() {
        // No input: print a fillable template and explain how to pass it back,
        // unless human flags or interactive mode provide an input object.
        None if !has_input_intent(&cmd) => {
            let schema = calc.input_schema_for(locale);
            let template = calc.input_template_for(locale);
            // A calculator with no inputs (today: every proprietary "unavailable"
            // stub) has nothing to fill in - printing `{}` and a "fill it in"
            // hint just hides the real content. Compute it directly so the
            // user sees the actual response (typically the proprietary
            // explanation and the open alternative).
            if template.as_object().is_some_and(serde_json::Map::is_empty) {
                let response = calc
                    .calculate_for(&serde_json::json!({}), locale)
                    .map_err(|e| anyhow!("{e}"))?;
                return emit(&response, cmd.format, locale, calc.license().source_url);
            }
            println!("{}", serde_json::to_string_pretty(&template)?);
            // If the schema has `oneOf` alternatives, the template shows only
            // the first - flag the others so they're discoverable without
            // having to read the full schema.
            if let Some(note) = oneof_alternatives_note(&schema) {
                eprintln!("\n{note}");
            }
            eprintln!(
                "\nReplace each placeholder with a value, then compute with one of:\n  \
                 clincalc calc {} --input <file.json>\n  \
                 clincalc calc {} --input '<json>'\n  \
                 clincalc calc {} --input -        # read JSON from stdin\n\
                 See the full input contract with: clincalc calc {} --schema",
                calc.name(),
                calc.name(),
                calc.name(),
                calc.name()
            );
            Ok(())
        }
        // Input supplied or built from human-friendly flags: validate (via the
        // calculator's typed deserialization) and compute.
        maybe_src => {
            let schema = calc.input_schema_for(locale);
            let mut input = match maybe_src {
                Some(src) => read_input(src)?,
                None if cmd.interactive => read_interactive_input(&schema)?,
                None => serde_json::Value::Object(serde_json::Map::new()),
            };
            apply_human_field_flags(&mut input, &cmd)?;
            apply_profile_sources(&mut input, &cmd, &schema)?;
            let mut cli_working = serde_json::Map::new();
            apply_body_fat_derivation(&mut input, &cmd, &mut cli_working)?;
            apply_goal_adjustment(&mut input, &cmd, &mut cli_working)?;
            apply_activity_preset(&mut input, cmd.activity, &mut cli_working)?;
            let mut response = calc
                .calculate_for(&input, locale)
                .map_err(|e| anyhow!("{e}"))?;
            for (key, value) in cli_working {
                response.working.insert(key, value);
            }
            emit(&response, cmd.format, locale, calc.license().source_url)
        }
    }
}

/// Resolve the CLI locale with `explicit > CLINCALC_LOCALE > en` precedence.
pub fn resolve_cli_locale(explicit: Option<&str>) -> Result<SupportedLocale> {
    let environment = std::env::var("CLINCALC_LOCALE").ok();
    resolve_requested_locale(explicit.or(environment.as_deref()).unwrap_or("en"))
}

fn resolve_requested_locale(requested: &str) -> Result<SupportedLocale> {
    lookup_locale(requested, COMPILED_LOCALES).ok_or_else(|| {
        anyhow!(
            "unsupported locale `{requested}`; available locales: {}",
            locale_list(COMPILED_LOCALES)
        )
    })
}

fn ensure_calculator_locale(calc: &dyn Calculator, locale: SupportedLocale) -> Result<()> {
    if calc.supported_locales().contains(&locale) {
        Ok(())
    } else {
        Err(anyhow!(
            "calculator `{}` is not available in locale `{locale}`; supported locales: {}",
            calc.name(),
            locale_list(calc.supported_locales())
        ))
    }
}

fn locale_list(locales: &[SupportedLocale]) -> String {
    locales
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Resolve an `--input` argument to a JSON value.
///
/// `-` reads stdin; an existing file path is read from disk; anything else is
/// treated as an inline JSON string. A leading `~` is expanded before checking
/// for a file, so `--input=~/score.json` behaves the same as shell-expanded
/// `--input ~/score.json`.
fn read_input(src: &str) -> Result<serde_json::Value> {
    let raw = if src == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else if let Some(path) = existing_input_path(src) {
        std::fs::read_to_string(path)?
    } else {
        src.to_string()
    };

    serde_json::from_str(&raw).map_err(|e| {
        anyhow!("invalid JSON input: {e}\nSee the expected shape with: clincalc calc <name>")
    })
}

fn has_input_intent(cmd: &CalcCommand) -> bool {
    cmd.interactive
        || cmd.profile
        || cmd.from_record.is_some()
        || has_energy_only_flags(cmd)
        || has_human_field_flags(cmd)
}

fn has_energy_only_flags(cmd: &CalcCommand) -> bool {
    cmd.activity.is_some()
        || cmd.goal.is_some()
        || cmd.rate.is_some()
        || cmd.target_weight_kg.is_some()
        || cmd.body_fat_pct.is_some()
}

fn has_human_field_flags(cmd: &CalcCommand) -> bool {
    cmd.equation.is_some()
        || cmd.sex.is_some()
        || cmd.age.is_some()
        || cmd.weight_kg.is_some()
        || cmd.height_cm.is_some()
        || cmd.lean_body_mass_kg.is_some()
        || cmd.creatinine.is_some()
        || cmd.creatinine_unit.is_some()
        || cmd.calorie_adjustment_kcal_day.is_some()
}

fn apply_human_field_flags(input: &mut serde_json::Value, cmd: &CalcCommand) -> Result<()> {
    if !has_human_field_flags(cmd) {
        return Ok(());
    }
    let Some(input) = input.as_object_mut() else {
        return Err(anyhow!("human field flags require JSON object input"));
    };

    insert_string_field(input, "equation", cmd.equation.as_deref())?;
    insert_string_field(input, "sex", cmd.sex.as_deref())?;
    insert_u64_field(input, "age", cmd.age)?;
    insert_f64_field(input, "weight_kg", cmd.weight_kg)?;
    insert_f64_field(input, "height_cm", cmd.height_cm)?;
    insert_f64_field(input, "lean_body_mass_kg", cmd.lean_body_mass_kg)?;
    insert_f64_field(input, "creatinine", cmd.creatinine)?;
    insert_string_field(input, "creatinine_unit", cmd.creatinine_unit.as_deref())?;
    insert_f64_field(
        input,
        "calorie_adjustment_kcal_day",
        cmd.calorie_adjustment_kcal_day,
    )?;
    Ok(())
}

fn insert_string_field(
    input: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<&str>,
) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    insert_field(input, key, serde_json::json!(value))
}

fn insert_u64_field(
    input: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<u64>,
) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    insert_field(input, key, serde_json::json!(value))
}

fn insert_f64_field(
    input: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<f64>,
) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    insert_field(input, key, serde_json::json!(value))
}

fn insert_field(
    input: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: serde_json::Value,
) -> Result<()> {
    if input.contains_key(key) {
        return Err(anyhow!(
            "--{flag} cannot be combined with `{key}` already present in JSON input",
            flag = key.replace('_', "-")
        ));
    }
    input.insert(key.to_string(), value);
    Ok(())
}

fn read_interactive_input(schema: &serde_json::Value) -> Result<serde_json::Value> {
    let mut input = serde_json::Map::new();
    loop {
        let required = required_fields(schema, &input);
        let mut prompted = false;
        for field in required {
            if !input.contains_key(&field) {
                let value = prompt_for_field(schema, &field)?;
                input.insert(field, value);
                prompted = true;
            }
        }
        if !prompted {
            break;
        }
    }
    Ok(serde_json::Value::Object(input))
}

fn required_fields(
    schema: &serde_json::Value,
    input: &serde_json::Map<String, serde_json::Value>,
) -> Vec<String> {
    let mut fields = Vec::new();
    append_required(schema.get("required"), &mut fields);
    if let Some(all_of) = schema.get("allOf").and_then(serde_json::Value::as_array) {
        for item in all_of {
            if item
                .get("if")
                .is_some_and(|condition| condition_matches(condition, input))
            {
                append_required(
                    item.get("then").and_then(|then| then.get("required")),
                    &mut fields,
                );
            }
        }
    }
    fields
}

fn append_required(required: Option<&serde_json::Value>, fields: &mut Vec<String>) {
    if let Some(required) = required.and_then(serde_json::Value::as_array) {
        for field in required.iter().filter_map(serde_json::Value::as_str) {
            if !fields.iter().any(|existing| existing == field) {
                fields.push(field.to_string());
            }
        }
    }
}

fn condition_matches(
    condition: &serde_json::Value,
    input: &serde_json::Map<String, serde_json::Value>,
) -> bool {
    let Some(properties) = condition
        .get("properties")
        .and_then(serde_json::Value::as_object)
    else {
        return false;
    };
    properties.iter().all(|(key, expected)| {
        expected
            .get("const")
            .is_some_and(|expected| input.get(key) == Some(expected))
    })
}

fn prompt_for_field(schema: &serde_json::Value, field: &str) -> Result<serde_json::Value> {
    let property = schema
        .get("properties")
        .and_then(|properties| properties.get(field));
    let mut prompt = field.to_string();
    if let Some(options) = property
        .and_then(|p| p.get("enum"))
        .and_then(serde_json::Value::as_array)
    {
        let options: Vec<_> = options
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect();
        if !options.is_empty() {
            prompt.push_str(&format!(" ({})", options.join("|")));
        }
    }
    prompt.push_str(": ");

    let mut stderr = std::io::stderr();
    stderr.write_all(prompt.as_bytes())?;
    stderr.flush()?;

    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let raw = line.trim();
    if raw.is_empty() {
        return Err(anyhow!("{field} is required"));
    }
    parse_interactive_value(field, raw, property)
}

fn parse_interactive_value(
    field: &str,
    raw: &str,
    property: Option<&serde_json::Value>,
) -> Result<serde_json::Value> {
    if let Some(options) = property
        .and_then(|p| p.get("enum"))
        .and_then(serde_json::Value::as_array)
    {
        let allowed = options.iter().filter_map(serde_json::Value::as_str);
        if !allowed.clone().any(|option| option == raw) {
            let values: Vec<_> = allowed.collect();
            return Err(anyhow!(
                "invalid value for {field}: {raw} (expected one of: {})",
                values.join(", ")
            ));
        }
        return Ok(serde_json::json!(raw));
    }

    match property
        .and_then(|p| p.get("type"))
        .and_then(serde_json::Value::as_str)
    {
        Some("integer") => raw
            .parse::<u64>()
            .map(serde_json::Value::from)
            .map_err(|e| anyhow!("invalid integer for {field}: {e}")),
        Some("number") => raw
            .parse::<f64>()
            .map(serde_json::Value::from)
            .map_err(|e| anyhow!("invalid number for {field}: {e}")),
        Some("boolean") => raw
            .parse::<bool>()
            .map(serde_json::Value::from)
            .map_err(|e| anyhow!("invalid boolean for {field}: {e}")),
        _ => Ok(serde_json::json!(raw)),
    }
}

fn apply_profile_sources(
    input: &mut serde_json::Value,
    cmd: &CalcCommand,
    schema: &serde_json::Value,
) -> Result<()> {
    if !cmd.profile && cmd.from_record.is_none() {
        return Ok(());
    }
    let Some(target) = input.as_object_mut() else {
        return Err(anyhow!("profile/record input requires JSON object input"));
    };

    if cmd.profile {
        let path = default_profile_path()?;
        let profile = read_json_file(&path)?;
        merge_profile_fields(target, schema, &profile);
    }
    if let Some(path) = &cmd.from_record {
        let record = read_json_file(&tilde_path(path))?;
        merge_profile_fields(target, schema, &record);
    }
    Ok(())
}

fn default_profile_path() -> Result<PathBuf> {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(config_home).join("clincalc/profile.json"));
    }
    let home = home_dir().ok_or_else(|| anyhow!("could not determine home directory"))?;
    Ok(home.join(".config/clincalc/profile.json"))
}

fn read_json_file(path: &Path) -> Result<serde_json::Value> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| anyhow!("reading profile/record {}: {e}", path.display()))?;
    serde_json::from_str(&raw)
        .map_err(|e| anyhow!("invalid JSON in profile/record {}: {e}", path.display()))
}

fn merge_profile_fields(
    target: &mut serde_json::Map<String, serde_json::Value>,
    schema: &serde_json::Value,
    source: &serde_json::Value,
) {
    merge_profile_object(target, schema, source);
    if let Some(subject) = source.get("subject") {
        merge_profile_object(target, schema, subject);
    }
    if let Some(profile) = source.get("profile") {
        merge_profile_object(target, schema, profile);
    }
}

fn merge_profile_object(
    target: &mut serde_json::Map<String, serde_json::Value>,
    schema: &serde_json::Value,
    source: &serde_json::Value,
) {
    let Some(source) = source.as_object() else {
        return;
    };
    let Some(properties) = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
    else {
        return;
    };
    for key in properties.keys() {
        if !target.contains_key(key)
            && let Some(value) = source.get(key)
        {
            target.insert(key.clone(), value.clone());
        }
    }
}

fn apply_body_fat_derivation(
    input: &mut serde_json::Value,
    cmd: &CalcCommand,
    cli_working: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<()> {
    let Some(body_fat_pct) = cmd.body_fat_pct else {
        return Ok(());
    };
    if !(0.0..=80.0).contains(&body_fat_pct) || !body_fat_pct.is_finite() {
        return Err(anyhow!(
            "--body-fat-pct must be a finite percentage between 0 and 80"
        ));
    }
    let Some(input) = input.as_object_mut() else {
        return Err(anyhow!("--body-fat-pct requires JSON object input"));
    };
    if input.contains_key("lean_body_mass_kg") {
        return Err(anyhow!(
            "--body-fat-pct cannot be combined with lean_body_mass_kg already present in input"
        ));
    }
    let weight_kg = input
        .get("weight_kg")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("--body-fat-pct requires weight_kg, for example --weight-kg 80"))?;
    let lean_body_mass_kg = weight_kg * (1.0 - body_fat_pct / 100.0);
    input.insert(
        "lean_body_mass_kg".into(),
        serde_json::json!(lean_body_mass_kg),
    );
    cli_working.insert("body_fat_pct".into(), serde_json::json!(body_fat_pct));
    cli_working.insert(
        "derived_lean_body_mass_kg".into(),
        serde_json::json!(lean_body_mass_kg),
    );
    Ok(())
}

fn apply_goal_adjustment(
    input: &mut serde_json::Value,
    cmd: &CalcCommand,
    cli_working: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<()> {
    if cmd.goal.is_none() && cmd.rate.is_none() && cmd.target_weight_kg.is_none() {
        return Ok(());
    }
    let goal = cmd
        .goal
        .ok_or_else(|| anyhow!("--rate and --target-weight require --goal lose|maintain|gain"))?;
    let Some(input) = input.as_object_mut() else {
        return Err(anyhow!("--goal requires JSON object input"));
    };
    if input.contains_key("calorie_adjustment_kcal_day") {
        return Err(anyhow!(
            "--goal cannot be combined with calorie_adjustment_kcal_day already present in input"
        ));
    }

    let adjustment = match goal {
        EnergyGoal::Maintain => {
            if cmd.rate.is_some() || cmd.target_weight_kg.is_some() {
                return Err(anyhow!(
                    "--goal maintain cannot be combined with --rate or --target-weight"
                ));
            }
            0.0
        }
        EnergyGoal::Lose | EnergyGoal::Gain => {
            let rate = cmd
                .rate
                .ok_or_else(|| anyhow!("--goal lose|gain requires --rate <kg/week>"))?;
            if !(0.0..=2.0).contains(&rate) || rate == 0.0 || !rate.is_finite() {
                return Err(anyhow!(
                    "--rate must be a finite number above 0 and at most 2 kg/week"
                ));
            }
            cli_working.insert("weight_change_rate_kg_week".into(), serde_json::json!(rate));
            if let Some(target_weight_kg) = cmd.target_weight_kg {
                let current_weight_kg = input
                    .get("weight_kg")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or_else(|| {
                        anyhow!("--target-weight requires weight_kg, for example --weight-kg 80")
                    })?;
                if !(20.0..=500.0).contains(&target_weight_kg) || !target_weight_kg.is_finite() {
                    return Err(anyhow!(
                        "--target-weight must be a finite number between 20 and 500 kg"
                    ));
                }
                let kg_to_target = (target_weight_kg - current_weight_kg).abs();
                cli_working.insert(
                    "target_weight_kg".into(),
                    serde_json::json!(target_weight_kg),
                );
                cli_working.insert(
                    "estimated_weeks_to_target".into(),
                    serde_json::json!(kg_to_target / rate),
                );
            }
            let daily_delta = rate * 7700.0 / 7.0;
            if matches!(goal, EnergyGoal::Lose) {
                -daily_delta
            } else {
                daily_delta
            }
        }
    };

    input.insert(
        "calorie_adjustment_kcal_day".into(),
        serde_json::json!(adjustment),
    );
    cli_working.insert("energy_goal".into(), serde_json::json!(goal.slug()));
    Ok(())
}

fn apply_activity_preset(
    input: &mut serde_json::Value,
    activity: Option<ActivityPreset>,
    cli_working: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<()> {
    let Some(activity) = activity else {
        return Ok(());
    };
    let Some(input) = input.as_object_mut() else {
        return Err(anyhow!("--activity requires JSON object input"));
    };
    if input.contains_key("activity_factor") {
        return Err(anyhow!(
            "--activity cannot be combined with an explicit activity_factor in --input"
        ));
    }
    input.insert(
        "activity_factor".into(),
        serde_json::json!(activity.factor()),
    );
    cli_working.insert("activity_preset".into(), serde_json::json!(activity.slug()));
    Ok(())
}

fn existing_input_path(src: &str) -> Option<PathBuf> {
    let path = tilde_path(src);
    path.is_file().then_some(path)
}

fn tilde_path(src: &str) -> PathBuf {
    if src == "~" {
        if let Some(home) = home_dir() {
            return home;
        }
    } else if let Some(rest) = src.strip_prefix("~/")
        && let Some(home) = home_dir()
    {
        return home.join(rest);
    }
    Path::new(src).to_path_buf()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// Run `clincalc list` / `clincalc ls`.
pub fn run_list(cmd: ListCommand) -> Result<()> {
    run_list_with_locale(cmd, resolve_cli_locale(None)?)
}

/// Run `clincalc list` / `clincalc ls` with an already resolved locale.
pub fn run_list_with_locale(cmd: ListCommand, locale: SupportedLocale) -> Result<()> {
    if cmd.tags {
        print_tags(cmd.format)
    } else {
        print_list(cmd.format, &cmd.tag, locale)
    }
}

/// Run `clincalc tags`.
pub fn run_tags(cmd: TagsCommand) -> Result<()> {
    print_tags(cmd.format)
}

/// Run `clincalc version`.
pub fn run_version(cmd: VersionCommand) -> Result<()> {
    match cmd.format {
        OutputFormat::Text | OutputFormat::Markdown => {
            println!("clincalc {}", env!("CARGO_PKG_VERSION"));
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "name": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
            }))?
        ),
    }
    Ok(())
}

fn canonical_calculator_name(name: &str) -> &str {
    CALCULATOR_ALIASES
        .iter()
        .find_map(|(alias, canonical)| (*alias == name).then_some(*canonical))
        .unwrap_or(name)
}

fn aliases_for(name: &str) -> Vec<&'static str> {
    CALCULATOR_ALIASES
        .iter()
        .filter_map(|(alias, canonical)| (*canonical == name).then_some(*alias))
        .collect()
}

fn unknown_calculator_message(name: &str) -> String {
    let mut message = format!("unknown calculator: {name}");
    if let Some(suggestion) = closest_calculator_name(name) {
        message.push_str(". Did you mean `");
        message.push_str(suggestion.name);
        message.push('`');
        if let Some(canonical) = suggestion.alias_for {
            message.push_str(" (alias for `");
            message.push_str(canonical);
            message.push_str("`)");
        }
        message.push('?');
    }
    message.push_str(" (try `clincalc list`)");
    message
}

struct NameSuggestion {
    name: &'static str,
    alias_for: Option<&'static str>,
}

fn closest_calculator_name(name: &str) -> Option<NameSuggestion> {
    let mut candidates: Vec<NameSuggestion> = crate::all()
        .into_iter()
        .map(|c| NameSuggestion {
            name: c.name(),
            alias_for: None,
        })
        .collect();
    candidates.extend(
        CALCULATOR_ALIASES
            .iter()
            .map(|(alias, canonical)| NameSuggestion {
                name: alias,
                alias_for: Some(canonical),
            }),
    );

    candidates
        .into_iter()
        .map(|candidate| {
            let distance = levenshtein(name, candidate.name);
            (distance, candidate)
        })
        .filter(|(distance, candidate)| *distance <= suggestion_threshold(name, candidate.name))
        .min_by_key(|(distance, candidate)| (*distance, candidate.name.len()))
        .map(|(_, candidate)| candidate)
}

fn suggestion_threshold(a: &str, b: &str) -> usize {
    let max_len = a.len().max(b.len());
    if max_len <= 4 { 1 } else { 3.min(max_len / 3) }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a = a.as_bytes();
    let b = b.as_bytes();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0; b.len() + 1];

    for (i, &a_byte) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &b_byte) in b.iter().enumerate() {
            let substitution = prev[j] + usize::from(a_byte != b_byte);
            let insertion = curr[j] + 1;
            let deletion = prev[j + 1] + 1;
            curr[j + 1] = substitution.min(insertion).min(deletion);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b.len()]
}

fn print_list(
    format: OutputFormat,
    required_tags: &[String],
    locale: SupportedLocale,
) -> Result<()> {
    // A calculator passes the filter only if it carries every requested tag
    // (AND semantics, so the filter narrows as more --tag flags are added).
    let passes = |c: &dyn Calculator| -> bool {
        if required_tags.is_empty() {
            return true;
        }
        let tags = c.tags();
        required_tags.iter().all(|t| tags.contains(&t.as_str()))
    };

    match format {
        OutputFormat::Json => {
            let items: Vec<_> = crate::all()
                .iter()
                .filter(|c| passes(c.as_ref()))
                .map(|c| {
                    let lic = c.license();
                    serde_json::json!({
                        "name": c.name(),
                        "title": c.title_for(locale),
                        "description": c.description_for(locale),
                        "supported_locales": c.supported_locales(),
                        "license": lic.license,
                        "license_source": lic.source_url,
                        "tags": c.tags(),
                        "aliases": aliases_for(c.name()),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
        OutputFormat::Text | OutputFormat::Markdown => {
            for c in crate::all().iter().filter(|c| passes(c.as_ref())) {
                let aliases = aliases_for(c.name());
                let alias_note = if aliases.is_empty() {
                    String::new()
                } else {
                    format!("  aliases: {}", aliases.join(", "))
                };
                println!(
                    "{:<20}  {:<48}  [{}]{}",
                    c.name(),
                    c.title_for(locale),
                    c.tags().join(", "),
                    alias_note
                );
            }
        }
    }
    Ok(())
}

/// `clincalc list --tags`: enumerate every tag in the registry with a count.
fn print_tags(format: OutputFormat) -> Result<()> {
    use std::collections::BTreeMap;

    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for c in crate::all() {
        for t in c.tags() {
            *counts.entry(*t).or_insert(0) += 1;
        }
    }

    match format {
        OutputFormat::Json => {
            let items: Vec<_> = counts
                .iter()
                .map(|(t, n)| serde_json::json!({ "tag": t, "count": n }))
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
        OutputFormat::Text | OutputFormat::Markdown => {
            for (t, n) in &counts {
                println!("{:<22}  {:>3}", t, n);
            }
        }
    }
    Ok(())
}

fn emit(
    response: &CalculationResponse,
    format: OutputFormat,
    locale: SupportedLocale,
    source_url: &str,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(response)?),
        OutputFormat::Text => println!("{}", render_text(response, locale)),
        OutputFormat::Markdown => println!("{}", render_markdown(response, locale, source_url)),
    }
    Ok(())
}

/// Render a result as a clinician-facing text block.
fn render_text(r: &CalculationResponse, locale: SupportedLocale) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{} = {}{}\n\n",
        result_label(r),
        value_to_string(&r.result),
        result_unit(r)
    ));
    out.push_str(&r.interpretation);
    if !r.working.is_empty() {
        out.push_str(match locale {
            SupportedLocale::En => "\n\nWorking:",
            SupportedLocale::Es => "\n\nDesglose:",
            SupportedLocale::Ca => "\n\nDesglossament:",
        });
        for (k, v) in &r.working {
            if k == "result_label" {
                continue;
            }
            out.push_str(&format!("\n  {k}: {}", value_to_string(v)));
        }
    }
    let reference_label = match locale {
        SupportedLocale::En => "Reference",
        SupportedLocale::Es => "Referencia",
        SupportedLocale::Ca => "Referència",
    };
    out.push_str(&format!("\n\n{reference_label}: {}", r.reference));
    out
}

/// Render a result as a Markdown block, for pasting into EHR free-text fields
/// or notes apps that render Markdown.
fn render_markdown(r: &CalculationResponse, locale: SupportedLocale, source_url: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "## {} = {}{}\n\n",
        result_label(r),
        value_to_string(&r.result),
        result_unit(r)
    ));
    out.push_str(&r.interpretation);
    out.push('\n');
    if !r.working.is_empty() {
        let working_heading = match locale {
            SupportedLocale::En => "Working",
            SupportedLocale::Es => "Desglose",
            SupportedLocale::Ca => "Desglossament",
        };
        out.push_str(&format!("\n### {working_heading}\n\n"));
        for (k, v) in &r.working {
            if k == "result_label" {
                continue;
            }
            out.push_str(&format!("- **{k}:** {}\n", value_to_string(v)));
        }
    }
    let reference_label = match locale {
        SupportedLocale::En => "Reference",
        SupportedLocale::Es => "Referencia",
        SupportedLocale::Ca => "Referència",
    };
    out.push_str(&format!(
        "\n**{reference_label}:** [{}]({source_url})",
        r.reference
    ));
    out
}

fn result_label(r: &CalculationResponse) -> &str {
    r.working
        .get("result_label")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&r.calculator)
}

fn result_unit(r: &CalculationResponse) -> String {
    r.working
        .get("unit")
        .and_then(serde_json::Value::as_str)
        .map(|unit| format!(" {unit}"))
        .unwrap_or_default()
}

fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// If the schema declares top-level `oneOf` alternative input shapes (each
/// with its own `required` array), build a one-paragraph note listing them.
///
/// The template printed by `clincalc calc <name>` shows only the first alternative;
/// this note tells the reader what else is permitted, so they don't have to
/// read the full schema to discover the other shapes.
fn oneof_alternatives_note(schema: &serde_json::Value) -> Option<String> {
    let alts = schema.get("oneOf")?.as_array()?;
    let groups: Vec<Vec<String>> = alts
        .iter()
        .filter_map(|alt| {
            alt.get("required").and_then(|r| r.as_array()).map(|r| {
                r.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
        })
        .filter(|g: &Vec<String>| !g.is_empty())
        .collect();

    if groups.len() < 2 {
        return None;
    }

    let mut out =
        String::from("This calculator accepts more than one input shape. Pick exactly one:\n");
    for (i, g) in groups.iter().enumerate() {
        let marker = if i == 0 { "shown above" } else { "alternative" };
        out.push_str(&format!("  {}: {}    ({marker})\n", i + 1, g.join(" + ")));
    }
    Some(out.trim_end().to_string())
}

#[cfg(test)]
mod tests {
    use super::{closest_calculator_name, levenshtein, oneof_alternatives_note, render_markdown};
    use crate::{CalculationResponse, SupportedLocale};
    use serde_json::json;

    #[test]
    fn aliases_and_fuzzy_suggestions_work() {
        let tdee = closest_calculator_name("tdde").unwrap();
        assert_eq!(tdee.name, "tdee");
        assert_eq!(tdee.alias_for, Some("energy_requirement"));

        let feverpain = closest_calculator_name("fevrpain").unwrap();
        assert_eq!(feverpain.name, "feverpain");
        assert_eq!(feverpain.alias_for, None);

        assert_eq!(levenshtein("fevrpain", "feverpain"), 1);
    }

    #[test]
    fn oneof_note_lists_each_alternative() {
        let schema = json!({
            "type": "object",
            "oneOf": [
                { "required": ["acr", "acr_unit"] },
                { "required": ["albumin", "creatinine"] }
            ]
        });
        let note = oneof_alternatives_note(&schema).unwrap();
        assert!(note.contains("acr + acr_unit"));
        assert!(note.contains("albumin + creatinine"));
        assert!(note.contains("shown above"));
    }

    #[test]
    fn no_oneof_yields_no_note() {
        assert!(oneof_alternatives_note(&json!({"type": "object"})).is_none());
    }

    #[test]
    fn markdown_render_has_heading_working_bullets_and_linked_reference() {
        let mut working = serde_json::Map::new();
        working.insert("result_label".to_string(), json!("CURB-65"));
        working.insert("confusion".to_string(), json!(false));
        let response = CalculationResponse {
            calculator: "curb65".to_string(),
            result: json!(2),
            interpretation: "Moderate severity.".to_string(),
            working,
            reference: "Lim WS, et al. Thorax. 2003;58(5):377-382.".to_string(),
        };

        let out = render_markdown(
            &response,
            SupportedLocale::En,
            "https://doi.org/10.1136/thorax.58.5.377",
        );

        assert!(out.starts_with("## CURB-65 = 2\n\n"));
        assert!(out.contains("Moderate severity."));
        assert!(out.contains("### Working\n\n"));
        assert!(out.contains("- **confusion:** false"));
        assert!(!out.contains("- **result_label:**"));
        assert!(out.contains(
            "**Reference:** [Lim WS, et al. Thorax. 2003;58(5):377-382.](https://doi.org/10.1136/thorax.58.5.377)"
        ));
    }
}
