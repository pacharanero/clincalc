<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Calculators: architecture

## Goal

Provide a comprehensive, open-source library of clinical calculators with **one canonical scoring engine** driving every surface: the `calc` command line, an MCP server (in any embedding host), a desktop GUI, and standalone single-file web tools. Calculations are evidence-based, auditable, and - when run inside a host that records them - travel with both inputs and result as immutable provenance.

A clinical-calculator suite driven by one engine, shippable in many shapes, is something a monolithic "Big EHR" platform structurally cannot produce. That advantage is the architecture this spec is built to capture.

## Project shape

`calc` is a **standalone project**. It ships as one crate, `clincalc`, with two surfaces selected by a Cargo feature:

- the **engine** (`default-features = false`) - the pure scoring engine and result schema; a leaf depending only on `serde` + `serde_json`.
- the **`cli` feature** (on by default) - the `calc` binary plus the reusable `clincalc::cli` module (`CalcCommand` + `run`), embeddable by host CLIs.

`calc-web` (single-file HTML calculators) is on the roadmap but deprioritised. A Tauri desktop GUI is the next major surface.

GitEHR (<https://github.com/gitehr/gitehr>) is a **downstream consumer** - its CLI forwards `gitehr calc` to `clincalc::cli::run`, and its MCP server exposes each calculator from `clincalc::all()` as a `calc_<name>` tool whose input schema is the calculator's own JSON Schema. Anyone else can embed `calc` the same way.

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
   │ calc CLI│   │ host MCP   │  │ host GUI │  │ standalone │   │  calc-web      │
   │ (default)│  │ (e.g.      │  │ (e.g.    │  │ desktop    │   │  single-file   │
   │  `calc`  │  │  gitehr)   │  │  Tauri)  │  │ (planned,  │   │  HTML + bridge │
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
calc/                                 # repo root = Cargo workspace root
├── Cargo.toml                        # package `clincalc`; [[bin]] `calc`; `cli` default feature
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
│   └── main.rs                       #   thin standalone `calc` binary (`cli` feature)
├── tests/                            # integration tests (completions; gated on `cli`)
├── calc-web/                         # single-file HTML calculators (deprioritised)
├── docs/                             # Zensical docs site (deployed to GH Pages)
├── examples/                         # ready-to-pipe JSON inputs used in the docs
├── spec/                             # this file plus roadmap and input-definitions
└── .claude/skills/build-calculator/  # authoring skill (may be retired)
```

### `clincalc` - the leaf engine

The single source of truth. Pure, deterministic scoring with no clock, no I/O, and no global state; a host that needs a timestamp stamps it when recording. With `default-features = false` it depends only on `serde` and `serde_json` - never on any host application and never on an async runtime (the CLI's `clap` / `anyhow` are optional, behind the default `cli` feature). That leaf discipline is what makes the calculators detachable, embeddable, and trivially auditable.

Every calculator implements the `Calculator` trait and also exposes a strongly-typed `Input`/`compute` pair plus a `build_response` adapter. The crate-level registry (`all()` / `get(name)`) is the one list every surface enumerates, so adding a calculator surfaces it everywhere.

### The `cli` feature - the CLI surface (default)

All CLI behaviour lives in the library (`CalcCommand` + `run()`), so there is nothing to re-implement when embedding it. It ships two ways:

1. The standalone `calc` binary - `cargo install --git https://github.com/pacharanero/calc clincalc` installs a small, dependency-light tool (tree: `anyhow`, `serde`/`serde_json`, `clap` - no async runtime, no host).
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
- **GUI** - a Tauri app (host or standalone) calls the `clincalc` engine natively over a Tauri command, rather than reimplementing logic in the webview. The next planned `calc` surface is a standalone Tauri desktop GUI whose headline is prominent copy-paste.
- **Standalone calc app** - because `clincalc` is pure Rust it cross-compiles to iOS/Android. A standalone Tauri app gives byte-identical results to every other surface.

### Distribution and decoupling

The leaf discipline (nothing in `clincalc` depends on a host or on an async runtime) is what enables both of these without trade-off:

- **Install just the calculators**: `cargo install --git https://github.com/pacharanero/calc clincalc` (and, once published, `cargo install clincalc` from crates.io). Cargo builds only `clincalc` + `clap` + `serde` - no host. The installed binary name is `calc` (set by `[[bin]] name`), independent of the package name.
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

`input_schema()` is the key LLM affordance: it powers `calc <name> --schema`, the fillable `calc <name>` template (derived from it via `input_template()`), MCP tool definitions, and any agent that wants to discover the required inputs without parsing prose. Each calculator additionally exposes a typed `compute()` for ergonomic, compile-time-checked use from Rust.

---

## CLI design (LLM-friendly)

There are **no per-calculator flags**. Flags do not scale past the simplest scores (QRISK3 has ~20 mixed-type, enumerated, unit-bearing inputs) and would force a hand-written, drift-prone clap struct per calculator. Instead every calculator is driven through one regular, registry-backed surface - so a human or an LLM learns it once, and adding a calculator to `clincalc` gives it a working CLI for free:

```bash
calc list                       # list calculators (text or JSON via --format)
calc <name>                     # print a fillable INPUT TEMPLATE (JSON on stdout)
calc <name> --schema            # print the JSON Schema (the full input contract)
calc <name> --license           # the algorithm's distribution licence
calc <name> --input -           # compute, reading JSON from stdin
calc <name> --input data.json   # compute, reading JSON from a file
calc <name> --input '{...}'     # compute, reading an inline JSON string
calc <name> --input ... --format json   # CalculationResponse as JSON on stdout
```

The template printed by `calc <name>` has the same shape as the input that `calc <name> --input` expects: each key carries a placeholder describing the expected value, derived from the schema so it can never drift from the contract.

Conventions: the template/schema/compute outputs are pure JSON on **stdout**; usage hints go to **stderr** so they never corrupt a piped stream. Computing always requires an explicit `--input`, so a bare `calc <name>` is pure discovery and never blocks reading stdin. Invalid input is rejected by the calculator's own typed deserialization with a clear message and a non-zero exit. This mirrors the MCP surface exactly: there an LLM receives each calculator's `input_schema()` as the tool's `inputSchema` and passes back a JSON object - the same "here is the schema, give me the JSON" contract.

User-facing CLI documentation lives in [`docs/cli-reference.md`](../docs/cli-reference.md) and the [Walkthrough](../docs/walkthrough.md); committed example inputs (used by both) live in [`examples/`](../examples).

---

## Web frontend (`calc-web`) - deprioritised

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

1. Implement it in `clincalc`: a typed `Input`, a pure `compute()`, a `build_response()` adapter, a `Calculator` impl with `input_schema()` and `license()` (the distribution licence plus an evidence URL), and unit tests against known vectors. Register it in `all()`. This is the **only** Rust work needed - the CLI (`calc <name>`, template, `--schema`, `--license`, `--input`) and the MCP tool are both driven generically from the registry, so there is no per-calculator CLI or MCP code to write.
2. (Optional) add a row to [`docs/calculators.md`](../docs/calculators.md) so it appears in the published catalogue.
3. (When `calc-web` returns) create `calc-web/calculators/<name>.html` with its JS logic validated against the `clincalc` vectors.

See `.claude/skills/build-calculator/` for the detailed authoring workflow. This skill may be retired in favour of `spec/` + `examples/` + `AGENTS.md`.

---

## Calculator library roadmap

UK-focused build priority (52 tools), ordered by clinical volume and patient-safety impact. As of this writing, 42 calculators are active and 10 are intentional "named but unavailable" stubs (licence-locked or proprietary). The full table with per-tool descriptions lives in [`spec/calculator-roadmap.md`](calculator-roadmap.md); the deployed catalogue is [`docs/calculators.md`](../docs/calculators.md).

### Tier 1 - High-volume primary care / NHS-mandated

QRISK3 (NICE NG238), PHQ-9 (NG222), GAD-7 (CG113), AUDIT / AUDIT-C (CG115), eGFR CKD-EPI (NG203), MUST (CG32; stub), FRAX (CG146; stub) / QFracture (CG146), FIB-4 (NG49).

### Tier 2 - Acute / emergency

NEWS2 (NG51; RCP/NHSE mandated), CURB-65 (NG138; BTS), Wells DVT / Wells PE (NG158), GRACE (NG185, CG94), CHA2DS2-VASc (NG196), HAS-BLED (NG196, NG158), ABCD2 (NG128), 4AT (CG103), qSOFA (NG51), CHALICE (NG232).

### Tier 3 - Common chronic disease management

MRC Dyspnoea (NG115), CAT (NG115; stub), ACQ (stub), IPSS (CG97), DAS28 (NG100), uACR (NG203, NG28), KDIGO CKD risk (NG203), EPDS (CG192; SIGN 169), Clinical Frailty Scale (NG56; stub), MMSE (NG97; stub).

### Tier 4 - Secondary care / specialist

SOFA (NG51), EuroSCORE II (TA163, TA245), HEART (NG185), TIMI (CG94), Padua (NG89), ELF (NG49; stub), Child-Pugh / MELD / UKELD (NG50), Nottingham Hip Fracture Score (CG124; NHFD).

### Tier 5 - Functional / PROMs / niche but guideline-endorsed

AMTS (CG124), Waterlow (CG179), Oxford Hip / Knee Score (NHSE PROMs; stubs), BODE (NG115), LANSS (CG173; stub), ABPI (NG19, CG168), Gleason Grade Groups (NG131), Nottingham Prognostic Index (NG101), ASRS-v1.1.

### RCPCH Digital Growth Charts (special case, not yet built)

UK-WHO (0-4y, WHO 2006) and UK90 (4-20y) reference data, gestational-age correction for prematurity, z-score/centile/SDS calculation. Requires LMS reference tables (the binary-size variable noted above) and confirmation of RCPCH licensing terms for distribution.

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
- The licence is surfaced for evidencing via `calc <name> --license` and in `calc list --format json` (`license`, `license_source`). When a host records calculator results, the licence should travel with the recorded result as provenance.

Most scores are pure published methods (algorithms are generally not subject to copyright), implemented from the primary literature and citing the publication as their source. Some instruments carry an explicit grant: PHQ-9 and GAD-7 are public domain (Pfizer, 2010); the ASRS is copyright WHO / NYU / Harvard and free to use with citation. Where terms are proprietary or unclear (e.g. FRAX, MMSE, MUST, CAT, ACQ, ELF, CFS, LANSS, OHS, OKS), the calculator is listed as a stub that returns an `unavailable` response, names the owner, and points at an open alternative where one exists - the gap is a first-class object, not silently hidden.

---

## Open questions

- Unit conversion support (metric/imperial)?
- Printable reports for results in the GUI?
- FHIR Observation export for standardised exchange?
- User-defined / third-party calculators via a plugin system?

## Future enhancements

Calculator plugins; fetching guideline updates from a registry; multi-step decision trees beyond simple scores; trending results over time; high-risk-score alerts.

---

This specification establishes `calc` as a comprehensive clinical decision support library with auditable, version-controlled-friendly calculation results, driven by a single engine that is equally at home at the command line, in an LLM's toolset, embedded in a host EHR, or as a standalone app.
