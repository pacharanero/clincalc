# CLI reference

Every command and flag the `clincalc` binary supports, in one place. There are no per-calculator flags - the same surface drives all 43 active calculators and 10 proprietary/unavailable stubs.

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

`clincalc <name>` remains supported as shorthand for `clincalc calc <name>`, so existing scripts continue to work. Computing always requires an explicit `--input`, so template mode never blocks waiting on stdin.

## Options

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

This is separate from the **code** licence (AGPL-3.0-or-later for the whole project) - it records the basis on which the **clinical algorithm itself** is being shipped.

### `--format <text|json>`

Output format for computed results, `list`, `tags`, and `version`.

- `text` (default) - a clinician-facing block: result, interpretation, working, reference. Designed for the clipboard.
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
