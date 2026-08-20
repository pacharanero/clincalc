<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Calculators: architecture

## Goal

Provide a comprehensive, open-source library of clinical calculators with **one canonical scoring engine** driving every surface: the `clincalc` command line, an MCP server (in any embedding host), a desktop GUI, and standalone single-file web tools. Calculations are evidence-based, auditable, and - when run inside a host that records them - travel with both inputs and result as immutable provenance.

A clinical-calculator suite driven by one engine, shippable in many shapes, is something a monolithic "Big EHR" platform structurally cannot produce. That advantage is the architecture this spec is built to capture.

## Project shape

`clincalc` is a **standalone project**. It ships as one crate, `clincalc`, with two surfaces selected by a Cargo feature:

- the **engine** (`default-features = false`) - the pure scoring engine and result schema; a leaf depending only on `serde` + `serde_json`.
- the **`cli` feature** (on by default) - the `clincalc` binary plus the reusable `clincalc::cli` module (`CalcCommand` + `run`), embeddable by host CLIs.

`clincalc-web` (single-file HTML calculators) is on the roadmap but deprioritised. A Tauri desktop GUI is the next major surface.

GitEHR (<https://github.com/gitehr/gitehr>) is a **downstream consumer** - its CLI forwards `gitehr calc` to `clincalc::cli::run`, and its MCP server exposes each calculator from `clincalc::all()` as a `clincalc_<name>` tool whose input schema is the calculator's own JSON Schema. Anyone else can embed `clincalc` the same way.

## Philosophy

### Open and free

- **Open source** - anyone can view, use, modify, and share the code (AGPL-3.0-or-later; clinical content under CC-BY-SA-4.0).
- **Free to use** - no paywalls, no licences, no restrictions.
- **Auditable** - scoring logic is pure and trivially readable; every calculator cites primary literature and is tested against known vectors.

### Soft interoperability

'Soft' interoperability is copy-and-paste interop. It empowers clinicians to use the tools they want without being constrained by their EHR, and lets them exercise their own judgement about whether to reach for a given calculator. Copy-and-paste is a common clinician workaround for the deficiencies of EHRs and is often derided as a kludge, but until real interoperability arrives we should embrace and optimise for the tools clinicians actually use. Every calculator therefore produces a clean, editable text summary for the clipboard as a first-class output, in addition to structured dispatch when embedded.

---

## Architecture: one core, many surfaces

The defining decision is a single scoring engine reused everywhere, so a result produced at the command line, in the browser, in a GUI, or via MCP is identical by construction. The dependency arrows all point **into** the core; the core never depends on anything above it.

```
                         ┌───────────────────────────┐
                         │   clincalc (leaf crate)   │
                         │  scoring logic + schema    │
                         │  deps: serde, serde_json   │
                         │  NO host, NO async runtime │
                         └─────────────┬──────────────┘
                                       │ (every arrow points in)
        ┌───────────────┬──────────────┼──────────────┬──────────────────┐
        │               │              │              │                  │
   ┌─────────┐   ┌────────────┐  ┌──────────┐  ┌────────────┐   ┌───────────────┐
   │clincalc│   │ host MCP   │  │ host GUI │  │ standalone │   │  clincalc-web      │
   │ (default)│  │ (e.g.      │  │ (e.g.    │  │ desktop    │   │  single-file   │
   │  CLI    │  │  gitehr)   │  │  Tauri)  │  │ (planned,  │   │  HTML + bridge │
   │          │  │            │  │          │  │  Tauri 2)  │   │  (deferred)    │
   └────┬─────┘  └────────────┘  └──────────┘  └────────────┘   └───────────────┘
        │ reused verbatim
   ┌────┴───────────┐
   │ gitehr calc    │
   │ (subcommand)   │
   └────────────────┘
```

### Workspace layout (as built)

```
clincalc/                             # repo root = Cargo workspace root
├── Cargo.toml                        # package `clincalc`; [[bin]] `clincalc`; `cli` default feature
├── cliff.toml                        # git-cliff changelog config
├── src/                              # the single `clincalc` crate
│   ├── lib.rs                        #   engine registry: all() / get(name)
│   ├── response.rs                   #   CalculationResponse schema
│   ├── calculator.rs                 #   Calculator trait + CalcError
│   ├── license.rs                    #   CalculatorLicense type
│   ├── template.rs                   #   schema → fillable template
│   ├── proprietary.rs                #   shared "unavailable" stub helper
│   ├── calculators/                  #   one file per calculator (~52)
│   ├── cli.rs                        #   CalcCommand + run()  (behind `cli` feature)
│   └── main.rs                       #   thin standalone `clincalc` binary (`cli` feature)
├── tests/                            # integration tests (completions; gated on `cli`)
├── clincalc-web/                         # single-file HTML calculators (deprioritised)
├── docs/                             # Zensical docs site (deployed to GH Pages)
├── examples/                         # ready-to-pipe JSON inputs used in the docs
└── spec/                             # this file plus roadmap and input-definitions
```

### `clincalc` - the leaf engine

The single source of truth. Pure, deterministic scoring with no clock, no I/O, and no global state; a host that needs a timestamp stamps it when recording. With `default-features = false` it depends only on `serde` and `serde_json` - never on any host application and never on an async runtime (the CLI's `clap` / `anyhow` are optional, behind the default `cli` feature). That leaf discipline is what makes the calculators detachable, embeddable, and trivially auditable.

Every calculator implements the `Calculator` trait and also exposes a strongly-typed `Input`/`compute` pair plus a `build_response` adapter. The crate-level registry (`all()` / `get(name)`) is the one list every surface enumerates, so adding a calculator surfaces it everywhere.

### The `cli` feature - the CLI surface (default)

All CLI behaviour lives in the library (`CalcCommand` + `run()`), so there is nothing to re-implement when embedding it. It ships two ways:

1. The standalone `clincalc` binary - `cargo install --git https://github.com/pacharanero/clincalc clincalc` installs a small, dependency-light tool (tree: `anyhow`, `serde`/`serde_json`, `clap` - no async runtime, no host).
2. A host CLI subcommand - the host's CLI depends on `clincalc` (with the `cli` feature) and forwards to `clincalc::cli::run`, repeating nothing:

```rust
// host's CLI (e.g. gitehr/cli/src/main.rs)
#[derive(clap::Subcommand)]
enum Commands {
    // ...existing commands
    /// Clinical calculators
    Calc(clincalc::cli::CalcCommand),
}
// dispatch:
Commands::Calc(cmd) => clincalc::cli::run(cmd)?,
```

### MCP, GUI, and the standalone app

- **MCP** - a host's MCP server exposes each calculator from `clincalc::all()` as a tool. The tool's input schema is `Calculator::input_schema()` and the tool body calls `Calculator::calculate(value)`. This is the most LLM-native surface: typed schemas handed directly to the model rather than scraped from help text.
- **GUI** - a Tauri app (host or standalone) calls the `clincalc` engine natively over a Tauri command, rather than reimplementing logic in the webview. The next planned `clincalc` surface is a standalone Tauri desktop GUI whose headline is prominent copy-paste.
- **Standalone clincalc app** - because `clincalc` is pure Rust it cross-compiles to iOS/Android. A standalone Tauri app gives byte-identical results to every other surface.

### Distribution and decoupling

The leaf discipline (nothing in `clincalc` depends on a host or on an async runtime) is what enables both of these without trade-off:

- **Install just the calculators**: `cargo install --git https://github.com/pacharanero/clincalc clincalc` (and, once published, `cargo install clincalc` from crates.io). Cargo builds only `clincalc` + `clap` + `serde` - no host. The installed binary name is `clincalc`, matching the package name.
- **Embed in any host**: a host path-, git-, or version-depends on `clincalc` (with `default-features = false` for just the engine, or the `cli` feature to reuse the command surface). There is no fork to maintain.

The one rule that keeps this true: the `clincalc` engine must stay a leaf.

### Binary-size note

Adding the calculators to a host binary costs almost nothing, because most hosts already link `clap`, `serde`, and `serde_json` - so the simple score-based calculators add no new dependencies, only a few KB of code and string data each. The only thing that moves the needle is calculators embedding large reference datasets (growth charts, risk-equation coefficient tables); for those, prefer loading tables from an embedded asset rather than baking everything into the binary's read-only data.

---

## Result schema: `CalculationResponse`

The Rust struct and the JSON object dispatched by every surface are the same shape, so results cross surfaces unchanged.

```rust
pub struct CalculationResponse {
    pub calculator: String,         // machine name, e.g. "feverpain"
    pub result: serde_json::Value,  // primary computed value (number or short string)
    pub interpretation: String,     // human-readable clinical interpretation
    pub working: serde_json::Map<String, serde_json::Value>, // step-by-step breakdown
    pub reference: String,          // primary citation / guideline
}
```

```json
{
  "calculator": "feverpain",
  "result": 3,
  "interpretation": "A score of 3 is associated with 34–40% isolation of streptococcus. A delayed prescribing strategy is appropriate after discussion with the patient.",
  "working": {
    "score": 3,
    "level": "delayed",
    "prescribing_recommendation": "Delayed antibiotic prescribing",
    "streptococcus_rate": "34–40%"
  },
  "reference": "Little P, Stuart B, Hobbs FDR, et al. Lancet Infect Dis. 2014. ..."
}
```

The shape is timestamp-free and id-free; a recording host adds those when it journals the result, so the same response can be deterministically tested and snapshotted.

---

## The `Calculator` trait

```rust
pub trait Calculator {
    fn name(&self) -> &'static str;          // stable machine name / subcommand / MCP tool name
    fn title(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn reference(&self) -> &'static str;
    fn license(&self) -> CalculatorLicense;  // algorithm distribution licence + evidence URL
    fn input_schema(&self) -> serde_json::Value;
    fn input_template(&self) -> serde_json::Value;
    fn calculate(&self, input: &serde_json::Value)
        -> Result<CalculationResponse, CalcError>;
}
```

`license()` is a **required** method (see Licensing): every calculator must declare the terms its algorithm/content is distributed under, with a URL evidencing them, so the basis for shipping it is always on record.

`input_schema()` is the key LLM affordance: it powers `clincalc calc <name> --schema`, the fillable `clincalc calc <name>` template (derived from it via `input_template()`), MCP tool definitions, and any agent that wants to discover the required inputs without parsing prose. Each calculator additionally exposes a typed `compute()` for ergonomic, compile-time-checked use from Rust.

---

## CLI design (LLM-friendly)

There are **no per-calculator flags**. Flags do not scale past the simplest scores (QRISK3 has ~20 mixed-type, enumerated, unit-bearing inputs) and would force a hand-written, drift-prone clap struct per calculator. Instead every calculator is driven through one regular, registry-backed surface - so a human or an LLM learns it once, and adding a calculator to `clincalc` gives it a working CLI for free:

```bash
clincalc list                       # list calculators (text or JSON via --format)
clincalc ls                         # alias for list
clincalc tags                       # list tags with counts
clincalc calc <name>                # print a fillable INPUT TEMPLATE (JSON on stdout)
clincalc calc <name> --schema       # print the JSON Schema (the full input contract)
clincalc calc <name> --license      # the algorithm's distribution licence
clincalc calc <name> --input -      # compute, reading JSON from stdin
clincalc calc <name> --input data.json   # compute, reading JSON from a file
clincalc calc <name> --input '{...}'     # compute, reading an inline JSON string
clincalc calc <name> --input ... --format json   # CalculationResponse as JSON on stdout
```

The template printed by `clincalc calc <name>` has the same shape as the input that `clincalc calc <name> --input` expects: each key carries a placeholder describing the expected value, derived from the schema so it can never drift from the contract. `clincalc <name>` remains supported as shorthand.

Conventions: the template/schema/compute outputs are pure JSON on **stdout**; usage hints go to **stderr** so they never corrupt a piped stream. Computing always requires an explicit `--input`, so a bare `clincalc calc <name>` is pure discovery and never blocks reading stdin. Invalid input is rejected by the calculator's own typed deserialization with a clear message and a non-zero exit. This mirrors the MCP surface exactly: there an LLM receives each calculator's `input_schema()` as the tool's `inputSchema` and passes back a JSON object - the same "here is the schema, give me the JSON" contract.

User-facing CLI documentation lives in [`docs/cli-reference.md`](../docs/cli-reference.md) and the [Walkthrough](../docs/walkthrough.md); committed example inputs (used by both) live in [`examples/`](../examples).

---

## Web frontend (`clincalc-web`) - deprioritised

The browser tools are single, self-contained HTML files with a shared context-detection bridge. The end-state is the same `clincalc` compiled to WebAssembly so the browser surface shares the engine; until then the inline JS logic must be validated against the `clincalc` test vectors. Not actively worked on - documented for completeness and for when it returns to the roadmap.

### Result Card UI conventions (when it returns)

Every web calculator renders a result card, in this order: (1) score summary and interpretation, (2) a collapsible per-item breakdown, (3) an editable clipboard preview textarea (the copy button reads the textarea's value so clinician edits are preserved), (4) action buttons appropriate to the host context (Tauri save / iframe send / standalone copy).

---

## Host integration

When a calculator runs inside an embedding host, dispatch stops being a bridge round-trip and becomes a direct call into the host's journal/state code.

GitEHR is the worked example: results are recorded as immutable, timestamped journal entries with structured YAML frontmatter (calculator type, version, inputs, result, citation) followed by a human-readable Markdown body, with the calculator's distribution licence travelling alongside as provenance.

Any host that records results should do something similar; the engine itself stays out of it.

---

## Authoring a new calculator

1. Implement it in `clincalc`: a typed `Input`, a pure `compute()`, a `build_response()` adapter, a `Calculator` impl with `input_schema()` and `license()` (the distribution licence plus an evidence URL), and unit tests against known vectors. Register it in `all()`. This is the **only** Rust work needed - the CLI (`clincalc calc <name>`, template, `--schema`, `--license`, `--input`) and the MCP tool are both driven generically from the registry, so there is no per-calculator CLI or MCP code to write.
2. (Optional) add a row to [`docs/calculators.md`](../docs/calculators.md) so it appears in the published catalogue.
3. (When `clincalc-web` returns) create `clincalc-web/calculators/<name>.html` with its JS logic validated against the `clincalc` vectors.

The authoring workflow is intentionally repository-native: start with `AGENTS.md`, this spec, `spec/calculator-input-definitions.md`, and the committed JSON files in `examples/`. That path works for any coding agent and keeps calculator authoring independent of Claude-specific skills.

---

## Calculator library roadmap

Open calculator work is tracked by stable ID in [`spec/calculator-roadmap.md`](calculator-roadmap.md). Completed calculators are removed from the roadmap and listed in the deployed catalogue at [`docs/calculators.md`](../docs/calculators.md), currently 68 active calculators plus 10 intentional "named but unavailable" stubs (licence-locked or proprietary).

### RCPCH Digital Growth Charts

Tracked as `CALC-032` in [`spec/calculator-roadmap.md`](calculator-roadmap.md). The special-case concerns are UK-WHO (0-4y, WHO 2006) and UK90 (4-20y) reference data, gestational-age correction for prematurity, z-score/centile/SDS calculation, LMS reference tables (the binary-size variable noted above), and confirmation of RCPCH licensing terms for distribution.

---

## Clinical validation

Each calculator must include: a primary peer-reviewed citation; evidence of clinical utility; test cases with known inputs/outputs from the literature (encoded as unit tests in `clincalc`); documented limitations and contraindications; and a process for incorporating guideline changes.

---

## Licensing

- `clincalc`: AGPL-3.0-or-later. Deliberately not available for subsumption into proprietary EHRs; if that service needs to exist, it can be offered as a hosted Calc-API.
- Clinical algorithms: implement from primary literature; most scores are public-domain methods. Do not copy proprietary implementations (e.g. MDCalc). QRISK3 and QFracture are ported from ClinRisk's LGPL-3.0 source and carry the required disclaimer.
- RCPCH growth charts: confirm licensing terms with RCPCH before distribution.
- All calculators cite original publications and validation studies.

### Per-calculator distribution licence (required)

Distinct from the **code** licence (AGPL-3.0), every calculator must record the terms under which its **clinical algorithm or content** is distributed, plus a URL evidencing those terms, so the basis for shipping each calculator is on record and can be re-verified at any time. This is enforced in code, not by convention:

- The `Calculator` trait requires `fn license(&self) -> CalculatorLicense`, where `CalculatorLicense { license, source_url }` carries the terms (an SPDX id where one applies, otherwise a short description such as "Public domain - no permission required") and a reverifiable URL. A calculator that omits it does not compile.
- A registry test (`every_calculator_records_its_license`) asserts every registered calculator has a non-empty licence and an `http(s)` source URL, so a new calculator cannot ship without recording its basis.
- The licence is surfaced for evidencing via `clincalc calc <name> --license` and in `clincalc list --format json` (`license`, `license_source`). When a host records calculator results, the licence should travel with the recorded result as provenance.

Most scores are pure published methods (algorithms are generally not subject to copyright), implemented from the primary literature and citing the publication as their source. Some instruments carry an explicit grant: PHQ-9 and GAD-7 are public domain (Pfizer, 2010), while the ASRS-v1.1 six-question screener permits clinical, non-clinical, and commercial electronic use with attribution and no other modification. The ASRS adapter scores six coded responses without reproducing the questionnaire, and the separately licensed 18-question checklist remains excluded. Where terms are proprietary, restricted, or unclear (e.g. FRAX, MMSE, MUST, CAT, ACQ, ELF, CFS, LANSS, OHS, OKS), the calculator is listed as a stub that returns an `unavailable` response, names the owner, and points at an open alternative where one exists - the gap is a first-class object, not silently hidden.

---

## Open questions

Open engine-surface questions are tracked by stable ID in [`spec/roadmap.md`](roadmap.md): `ENG-004` for FHIR Observation export, `ENG-005` for unit conversion, `ENG-006` for richer printable / clipboard-friendly output, `ENG-007` for plugins, `ENG-008` for guideline-update registry work, and `ENG-009` for high-risk-score alerts.

---

This specification establishes `clincalc` as a comprehensive clinical decision support library with auditable, version-controlled-friendly calculation results, driven by a single engine that is equally at home at the command line, in an LLM's toolset, embedded in a host EHR, or as a standalone app.
