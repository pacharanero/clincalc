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

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use clap::{Args, ValueEnum, ValueHint};

use crate::{CalculationResponse, Calculator};

/// How to render computed results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text.
    #[default]
    Text,
    /// Machine-readable JSON (the `CalculationResponse` schema).
    Json,
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
    // `--tags` lists every known tag (with calculator counts) and exits.
    if cmd.tags {
        return print_tags(cmd.format);
    }

    // No name (or legacy `list` / `ls`) means: show the catalogue (optionally filtered by --tag).
    let name = match cmd.name.as_deref() {
        None | Some("list") | Some("ls") => return print_list(cmd.format, &cmd.tag),
        Some(n) => n,
    };

    let calc = crate::get(name)
        .ok_or_else(|| anyhow!("unknown calculator: {name} (try `clincalc list`)"))?;

    // `--schema` prints the formal contract, regardless of everything else.
    if cmd.schema {
        println!("{}", serde_json::to_string_pretty(&calc.input_schema())?);
        return Ok(());
    }

    // `--license` prints the algorithm's distribution licence and evidence URL.
    if cmd.license {
        println!("{}", serde_json::to_string_pretty(&calc.license())?);
        return Ok(());
    }

    match cmd.input.as_deref() {
        // No input: print a fillable template and explain how to pass it back.
        None => {
            let schema = calc.input_schema();
            let template = calc.input_template();
            // A calculator with no inputs (today: every proprietary "unavailable"
            // stub) has nothing to fill in - printing `{}` and a "fill it in"
            // hint just hides the real content. Compute it directly so the
            // user sees the actual response (typically the proprietary
            // explanation and the open alternative).
            if template.as_object().is_some_and(serde_json::Map::is_empty) {
                let response = calc
                    .calculate(&serde_json::json!({}))
                    .map_err(|e| anyhow!("{e}"))?;
                return emit(&response, cmd.format);
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
                 clincalc calc {name} --input <file.json>\n  \
                 clincalc calc {name} --input '<json>'\n  \
                 clincalc calc {name} --input -        # read JSON from stdin\n\
                 See the full input contract with: clincalc calc {name} --schema"
            );
            Ok(())
        }
        // Input supplied: validate (via the calculator's typed deserialization)
        // and compute.
        Some(src) => {
            let input = read_input(src)?;
            let response = calc.calculate(&input).map_err(|e| anyhow!("{e}"))?;
            emit(&response, cmd.format)
        }
    }
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
    if cmd.tags {
        print_tags(cmd.format)
    } else {
        print_list(cmd.format, &cmd.tag)
    }
}

/// Run `clincalc tags`.
pub fn run_tags(cmd: TagsCommand) -> Result<()> {
    print_tags(cmd.format)
}

/// Run `clincalc version`.
pub fn run_version(cmd: VersionCommand) -> Result<()> {
    match cmd.format {
        OutputFormat::Text => println!("clincalc {}", env!("CARGO_PKG_VERSION")),
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

fn print_list(format: OutputFormat, required_tags: &[String]) -> Result<()> {
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
                        "title": c.title(),
                        "description": c.description(),
                        "license": lic.license,
                        "license_source": lic.source_url,
                        "tags": c.tags(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
        OutputFormat::Text => {
            for c in crate::all().iter().filter(|c| passes(c.as_ref())) {
                println!(
                    "{:<12}  {:<48}  [{}]",
                    c.name(),
                    c.title(),
                    c.tags().join(", ")
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
        OutputFormat::Text => {
            for (t, n) in &counts {
                println!("{:<22}  {:>3}", t, n);
            }
        }
    }
    Ok(())
}

fn emit(response: &CalculationResponse, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(response)?),
        OutputFormat::Text => println!("{}", render_text(response)),
    }
    Ok(())
}

/// Render a result as a clinician-facing text block.
fn render_text(r: &CalculationResponse) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{} = {}\n\n",
        r.calculator,
        value_to_string(&r.result)
    ));
    out.push_str(&r.interpretation);
    if !r.working.is_empty() {
        out.push_str("\n\nWorking:");
        for (k, v) in &r.working {
            out.push_str(&format!("\n  {k}: {}", value_to_string(v)));
        }
    }
    out.push_str(&format!("\n\nReference: {}", r.reference));
    out
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
    use super::oneof_alternatives_note;
    use serde_json::json;

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
}
