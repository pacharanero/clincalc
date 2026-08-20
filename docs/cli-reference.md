# CLI reference

Every command and flag the `clincalc` binary supports, in one place. There are no per-calculator flags - the same registry-backed surface drives every active calculator and proprietary/unavailable stub.

```text
clincalc [COMMAND]
```

## Commands

| Invocation | What it does |
|---|---|
| `clincalc` | Print the calculator catalogue. Bare invocation is intentionally useful. |
| `clincalc list` / `clincalc ls` | Print the calculator catalogue, optionally filtered by tag. |
| `clincalc tags` | Print every tag with the number of calculators carrying it. |
| `clincalc calc <name>` | Print a fillable **input template** (JSON to stdout, hint to stderr). |
| `clincalc calc <name> --schema` | Print the calculator's full **JSON Schema** input contract. |
| `clincalc calc <name> --license` | Print the algorithm's distribution licence and evidence URL. |
| `clincalc calc <name> --input <src>` | **Compute** a result. `<src>` is `-` (stdin), a file path, or an inline JSON string. |
| `clincalc version` | Print version information; add `--format json` for a structured object. |
| `clincalc completions install` | Install shell completions for the current user. |
| `clincalc mcp` | Start the local stdio MCP server when compiled with `--features mcp`. |
| `clincalc api` | Start the REST API when compiled with the default-enabled `rest-api` feature. |

`clincalc <name>` remains supported as shorthand for `clincalc calc <name>`, so existing scripts continue to work. Common aliases such as `bmr`, `rmr`, `ree`, and `tdee` resolve to `energy_requirement`; `ckd-epi` / `ckdepi` resolve to `egfr`. `clincalc list` shows aliases, and unknown calculator names include a small "did you mean" hint when there is a close match. Computing always requires an explicit `--input`, so template mode never blocks waiting on stdin.

## Options

### `--locale <BCP47>`

Select human-readable calculator content with a BCP 47 locale tag. The explicit flag takes precedence over `CLINCALC_LOCALE`; English is the default. Regional tags use RFC 4647 lookup when a reviewed language bundle is available.

The locale architecture is implemented, but calculators currently advertise English only. Spanish and Catalan CURB-65 adaptations remain withheld until the final recommendation wording has recorded native-speaker clinical review.

When a reviewed bundle is available, the locale applies to metadata, schema descriptions, template hints, computed interpretation, and the human text output labels. Input field names, JSON numbers, result values, risk codes, recommendation codes, units, and citations remain stable. A calculation reports the resolved bundle in `working.content_locale`. A calculator that does not advertise the requested locale exits with its supported locale list rather than mixing languages; catalogue entries without that translation remain in English and expose their `supported_locales` in JSON output.

Locale negotiation is currently implemented for catalogue and calculator commands. `clincalc api` and `clincalc mcp` reject the global `--locale` option until their protocol-specific negotiation described in the multilingual roadmap is implemented.

### `--input <JSON|FILE|->`

Source of the input JSON. The argument is resolved in this order:

1. `-` reads the whole of standard input.
2. An existing file path is read from disk.
3. Anything else is treated as an inline JSON string.

```bash
clincalc calc news2 --input -                    # stdin
clincalc calc news2 --input examples/news2.json  # file
clincalc calc news2 --input '{"respiratory_rate":21, ...}'  # inline
```

Invalid JSON is rejected with a clear message and a non-zero exit; the reminder points you at `clincalc calc <name>` to see the expected shape.

### `--activity <sedentary|light|moderate|very-active|extra-active>`

Convenience input for `energy_requirement` and its aliases (`bmr`, `rmr`, `ree`, `tdee`). The preset injects the standard activity factor before calculation: sedentary 1.2, light 1.375, moderate 1.55, very-active 1.725, extra-active 1.9. Do not combine it with an explicit `activity_factor` in the JSON input.

```bash
clincalc calc tdee --activity moderate --input '{"equation":"mifflin_st_jeor","sex":"male","age":30,"weight_kg":70,"height_cm":175}'
```

The selected preset is echoed in the Working block as `activity_preset`; the numeric factor remains visible as `activity_factor`.

Energy target helpers can derive `calorie_adjustment_kcal_day` from a goal and rate using roughly 7700 kcal/kg. For Cunningham, `--body-fat-pct` can derive `lean_body_mass_kg` from `weight_kg`.

```bash
clincalc calc tdee --equation mifflin_st_jeor --sex male --age 30 --weight-kg 70 --height-cm 175 --activity moderate --goal lose --rate 0.5 --target-weight 65
clincalc calc bmr --equation cunningham --weight-kg 80 --body-fat-pct 25
```

These helpers echo their derived values in the Working block (`energy_goal`, `weight_change_rate_kg_week`, `estimated_weeks_to_target`, `body_fat_pct`, `derived_lean_body_mass_kg`).

### Human field flags and `--interactive`

For quick hand use, common scalar input fields can be supplied as flags instead of writing JSON: `--equation`, `--sex`, `--age`, `--weight-kg`, `--height-cm`, `--lean-body-mass-kg`, `--creatinine`, `--creatinine-unit`, and `--calorie-adjustment-kcal-day`. These flags build the same JSON object that `--input` would have supplied, so calculator validation and output remain unchanged.

```bash
clincalc calc tdee --equation mifflin_st_jeor --sex male --age 30 --weight-kg 70 --height-cm 175 --activity moderate
clincalc calc egfr --age 60 --sex female --creatinine 80 --creatinine-unit umol/L
```

You can combine these flags with `--input` to add missing fields, but a flag cannot overwrite a field that is already present in the JSON. For guided entry, use `--interactive`; prompts are written to stderr and the result remains on stdout.

```bash
clincalc calc egfr --interactive
```

### `--profile` and `--from-record <FILE>`

`--profile` fills missing fields from `~/.config/clincalc/profile.json` (or `$XDG_CONFIG_HOME/clincalc/profile.json`). `--from-record <FILE>` does the same from a specified JSON file. Only keys present in the selected calculator's schema are copied; matching keys can live at the top level, under `subject`, or under `profile`.

```json
{
  "subject": {
    "age": 60,
    "sex": "female",
    "weight_kg": 70,
    "height_cm": 165
  }
}
```

Explicit `--input` values and human field flags win; profile/record data only fills gaps.

```bash
clincalc calc egfr --from-record patient.json --creatinine 80 --creatinine-unit umol/L
```

### `--schema`

Print the calculator's JSON Schema to stdout. This is the formal input contract - field names, types, ranges, enumerations - and is the same schema served to LLMs by the MCP surface when `clincalc` is embedded in a host.

```bash
clincalc calc gad7 --schema
```

### `--license`

Print the algorithm's distribution licence (an SPDX identifier where one applies, otherwise a short description) plus a reverifiable URL, as a small JSON object:

```bash
clincalc calc qrisk3 --license
```

This is separate from the licence of the original `clincalc` code (AGPL-3.0-or-later) - it records the basis on which the **clinical algorithm itself** is being shipped. Third-party-derived modules retain their own licences, including LGPL-3.0-or-later for QRISK3 and QFracture.

### `--format <text|json>`

Output format for computed results, `list`, `tags`, and `version`.

- `text` (default) - a clinician-facing block: result, interpretation, working, reference. Designed for the clipboard. Calculators may provide a more helpful headline label than their machine name, for example `BMR/RMR`, `TDEE`, or `Target intake` for `energy_requirement`.
- `json` - the `CalculationResponse` structure as machine-readable JSON. The same shape every surface (CLI, MCP, GUI) produces.

```bash
clincalc list --format json
clincalc tags --format json
clincalc version --format json
clincalc calc feverpain --input examples/feverpain.json --format json
```

### `--help`, `--version`, `-V`, `-v`, `-version`

`--help` describes commands and flags. `clincalc version` is the documented version command and supports `--format json`; the conventional quick-check flags `--version` / `-V` also work, and lone `-v` / `-version` are accepted as helpful aliases.

## MCP server

`clincalc mcp` starts the local stdio Model Context Protocol server when the binary is compiled with the optional `mcp` feature:

```bash
cargo install clincalc --features mcp
clincalc mcp
```

The MCP server exposes every calculator from `clincalc::all()` as a tool named `clincalc_<name>`. Each tool's input schema is the calculator's own JSON Schema, and calls return the same structured `CalculationResponse` as `clincalc calc <name> --input ... --format json`.

A binary compiled without the `mcp` feature reserves the command and prints a targeted reinstall hint instead of treating `mcp` as an unknown calculator. See [MCP server](mcp.md) for host configuration and safety notes.

## REST API

`clincalc api` starts the default-enabled HTTP surface on `127.0.0.1:8080`. Override the bind address with `--host` and `--port`:

```bash
clincalc api --host 127.0.0.1 --port 8080
```

The API exposes `GET /calculators`, calculator schema/template/licence routes, `POST /calculators/{name}`, and `GET /openapi.json`. It currently serves canonical English content; locale query/header negotiation remains tracked in the multilingual roadmap.

## Shell completions

Install completions for your current shell:

```bash
clincalc completions install
```

For package managers or custom locations, generate or write a specific shell's completion script:

```bash
clincalc completions bash
clincalc completions --dir ~/.local/share/bash-completion/completions bash
clincalc completions --dir ~/.zfunc zsh
clincalc completions --dir ~/.config/fish/completions fish
clincalc completions --dir ~/.config/powershell/completions powershell
```

The installer detects `$SHELL`, creates the completion directory, writes the correctly named file, and prints any one-time shell configuration still needed.

## The `CalculationResponse` shape

Every computed result, regardless of calculator, has the same JSON shape:

```json
{
  "calculator": "feverpain",
  "result": 3,
  "interpretation": "A score of 3 is associated with 34–40% isolation of streptococcus. ...",
  "working": {
    "score": 3,
    "level": "delayed",
    "...": "..."
  },
  "reference": "Little P, Stuart B, Hobbs FDR, et al. Lancet Infect Dis. 2014. ..."
}
```

- `calculator` - the machine name (matches the CLI subcommand and the MCP tool name).
- `result` - the primary computed value. A number for most scores, a short string for categorical results.
- `interpretation` - the clinician-facing summary line(s).
- `working` - every intermediate value the score depends on, so the result is auditable without re-running.
- `working.content_locale` - present on locale-aware calculations; the canonical BCP 47 bundle used for human-readable content.
- `reference` - the primary citation.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Success. (Includes proprietary-stub responses, which are valid `CalculationResponse` objects - they are *unavailable*, not errors.) |
| `1` | Anything else: unknown calculator, invalid JSON, schema mismatch, range violation. |

## Conventions

- Template / schema / compute output is on **stdout** as pure JSON or pure text - safe to redirect or pipe.
- Reminders and usage hints go to **stderr** so they never corrupt a stream.
- The CLI never reads stdin unless you ask for it (`--input -`).
- All output is deterministic - no timestamps, no random ids - so it diffs cleanly in tests and audits.

## Embedding in a host CLI

The same library that drives the `clincalc` binary is reusable as `clincalc::cli::run`, so a host CLI repeats nothing:

```rust
#[derive(clap::Subcommand)]
enum Commands {
    // ...
    /// Clinical calculators
    Calc(clincalc::cli::CalcCommand),
}

// dispatch:
Commands::Calc(cmd) => clincalc::cli::run(cmd)?,
```

GitEHR's `gitehr calc` subcommand is implemented exactly this way. See [How it works](how-it-works.md) for the wider architecture.
