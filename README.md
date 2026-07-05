# calc - open clinical calculators

Open, auditable clinical calculators driven by a single Rust engine. One scoring core (`clincalc`) powers every surface - the `calc` command line, single-file web tools, a native desktop GUI, and, in host applications, an MCP server for LLMs and EHR integration - so a result is identical wherever it is produced.

This is a standalone project: the engine, the CLI, the web tools, and the docs focus purely on calculators, and the `calc` CLI installs and runs on its own with no EHR. It is reusable by anyone with no knowledge of GitEHR; GitEHR is one downstream consumer (it depends on these crates, not the other way around).

## Why

Clinicians need clinical digital tools to provide good care, but the incentives to build them into EHRs are weak and the compliance barriers are high. The result is a patchwork of calculators scattered across the web, often behind paywalls or implemented inconsistently. This project makes them **open source, free to use, evidence-based, and auditable** - each cites primary literature, is tested against published vectors, and records the licence it is distributed under.

### Soft interoperability

"Soft" interoperability is copy-and-paste interop: it lets clinicians use the tools they want without being constrained by their EHR. Copy-and-paste is derided as a kludge, but it is what clinicians actually use, so every calculator produces a clean, editable text summary as a first-class output - while also dispatching structured results when embedded in a host.

## Install and use the `calc` CLI

```bash
cargo install --git https://github.com/pacharanero/calc clincalc
```

There are no per-calculator flags. Every calculator is driven the same way - ask for a template, fill it in, pass it back:

```bash
calc list                       # list calculators (add --format json for licences)
calc <name>                     # print a fillable input TEMPLATE (JSON)
calc <name> --schema            # the JSON Schema (full input contract)
calc <name> --license           # the algorithm's distribution licence + evidence URL
calc <name> --input -           # compute, reading JSON from stdin
calc <name> --input data.json   # ...or from a file
calc <name> --input '{...}'     # ...or inline
```

```console
$ calc curb65 --input '{"confusion":false,"urea_mmol_l":8,"respiratory_rate":32,"systolic_bp":85,"diastolic_bp":55,"age":72}'
curb65 = 4
High severity ... consider hospital admission and assessment for intensive care.
```

The template printed by `calc <name>` has the same shape as the input it expects, so it is a clean round-trip. Output, schema, and template are JSON on stdout; hints go to stderr.

## The library

The full UK-focused 50-tool roadmap (`spec/calculator-roadmap.md`) is implemented across five tiers, from QRISK3, PHQ-9, GAD-7, eGFR and FIB-4 through NEWS2, CURB-65, the Wells scores, CHA2DS2-VASc and HAS-BLED, to DAS28, SOFA, MELD, CHALICE and Gleason. Run `calc list` for the current set.

### Proprietary tools are named, not hidden

A handful of tools cannot be shipped because they are proprietary or licence-locked (FRAX, MMSE, ELF, ACQ, the Oxford Hip/Knee Scores, CAT, MUST, CFS, LANSS). Rather than omit them silently, each is registered as a calculator that returns a structured explanation - the owner, why it cannot be shipped, open alternatives (often one shipped here), and how to advocate for open clinical tools:

```console
$ calc frax --input '{}'
frax = unavailable: proprietary
FRAX ... is not available because it is proprietary or licence-locked. Owner:
University of Sheffield ... Open alternatives: qfracture ...
```

## Architecture: one core, many surfaces

The dependency arrows all point **into** the core, which never depends on anything above it:

- **`clincalc`** - one crate, two surfaces. With `default-features = false` it is the pure scoring engine and result schema: a strict leaf depending only on `serde` and `serde_json`, never on an async runtime or any host - which makes the calculators detachable and embeddable.
- The default **`cli` feature** adds the `calc` binary and the reusable `clincalc::cli` module (`run` + `CalcCommand`). A host CLI such as GitEHR's `gitehr calc` subcommand calls this same module, so nothing is reimplemented.
- **`calc-web`** - single-file HTML calculators with a shared context-detection bridge.

Adding a calculator to `clincalc::all()` surfaces it everywhere - CLI, MCP, web - with no per-surface code.

### Input definitions

Several inputs are clinician-asserted predicates whose TRUE/FALSE conditions are easy to get subtly wrong (for example, "vascular disease" in CHA2DS2-VASc is arterial and excludes venous thromboembolism). Each such input carries a machine-readable definition - includes, excludes, a cited source, and a draft SNOMED ECL - that travels in the schema to every surface. See `spec/calculator-input-definitions.md`.

## Embedding in a host (for example, GitEHR)

Any application can embed these crates. GitEHR ([gitehr/gitehr](https://github.com/gitehr/gitehr)) is one consumer: its CLI forwards `gitehr calc` to `clincalc::cli::run`, and its MCP server exposes each calculator from `clincalc::all()` as a `calc_<name>` tool whose input schema is the calculator's own JSON Schema. The calculators are the engine; a host wires them into its own surfaces.

## Develop

```bash
cargo test                                      # all calculators
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
```

CI enforces all three. Adding a calculator: implement it in `clincalc` (typed input, pure `compute`, `build_response`, a `Calculator` impl with `input_schema()` and `license()`, and literature-vector tests), register it in `all()`, and that is the only Rust work - the CLI and MCP surfaces pick it up automatically. See `spec/calculators.md` and `skills/build-calculator/`.

## Licensing

- `clincalc`: AGPL-3.0-or-later. This work is deliberately not available for subsumption into proprietary EHRs; if that service needs to exist, it can be offered as a hosted Calc-API.
- Clinical algorithms are implemented from primary literature (most scores are public-domain methods); QRISK3 and QFracture are ported from ClinRisk's LGPL-3.0 source and carry the required disclaimer. Each calculator records its own distribution licence via `calc <name> --license`.
- Clinical content (source references) under CC-BY-SA-4.0.

## Roadmap

See [`ROADMAP.md`](ROADMAP.md) for engineering, infrastructure, distribution, GUI, and product-level work, and [`spec/calculator-roadmap.md`](spec/calculator-roadmap.md) for the clinical-calculator backlog.
