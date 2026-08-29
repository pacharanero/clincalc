# clincalc - open clinical calculators

Open source, community-auditable clinical calculators. One Rust crate is the scoring engine; every surface - CLI, Python package, MCP tool, REST API, desktop GUI, web UI, and embeddable Rust library - is driven from that single source of truth.

A score is computed once, in the core library, and the result is identical wherever it appears. Every calculator cites primary literature, is tested against published vectors, and records the licence it is distributed under.

## Why

Clinicians need clinical digital tools to provide good care, but the incentives to build them into EHRs are weak and the compliance barriers are high. The result is a patchwork of calculators scattered across the web, often behind paywalls or implemented inconsistently. This project makes them **open source, free to use, evidence-based, and auditable** - each cites primary literature, is tested against published vectors, and records the licence it is distributed under.

### 'Soft interoperability'

'Soft interoperability' is a phrase coined by Marcus Baw (@pacharanero) to describe the everyday copy-and-paste interop which is used by clincians as a substitute for 'proper' interoperability: it lets clinicians use the tools they want without being constrained by their EHR. Copy-and-paste is derided as a kludge, but it is what clinicians actually use, so every calculator produces a clean, editable text summary as a first-class output - while also dispatching structured results when embedded in a host.

## Install and use the `clincalc` CLI

Install the latest prebuilt release through the docs-site installer proxy:

```bash
curl -LsSf https://pacharanero.github.io/clincalc/install.sh | sh
```

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://pacharanero.github.io/clincalc/install.ps1 | iex"
```

Or install the CLI with Cargo:

```bash
cargo install clincalc
```

There are no per-calculator flags. Every calculator is driven the same way - ask for a template, fill it in, pass it back:

```bash
clincalc list                       # list calculators (alias: ls)
clincalc tags                       # list tags with counts
clincalc calc <name>                # print a fillable input TEMPLATE (JSON)
clincalc calc <name> --schema       # the JSON Schema (full input contract)
clincalc calc <name> --license      # the algorithm's distribution licence + evidence URL
clincalc calc <name> --input -      # compute, reading JSON from stdin
clincalc calc <name> --input data.json   # ...or from a file
clincalc calc <name> --input '{...}'     # ...or inline
```

```console
$ clincalc calc curb65 --input '{"confusion":false,"urea_mmol_l":8,"respiratory_rate":32,"systolic_bp":85,"diastolic_bp":55,"age":72}'
curb65 = 4
High severity ... consider hospital admission and assessment for intensive care.
```

The template printed by `clincalc calc <name>` has the same shape as the input it expects, so it is a clean round-trip. `clincalc <name>` remains supported as shorthand for `clincalc calc <name>`. Schemas and templates are JSON; computed results are readable text by default and JSON with `--format json`. Output goes to stdout and hints go to stderr.

### Localised output

Human-readable calculator content accepts BCP 47 locale tags through `--locale` or `CLINCALC_LOCALE`. The locale architecture is in place, but calculators currently advertise English only. Spanish and Catalan CURB-65 adaptations remain withheld until the final recommendation wording has recorded native-speaker clinical review. Requesting an unreviewed locale fails clearly instead of returning mixed-language or unreviewed clinical prose.

## MCP server

`clincalc` can also run as a local stdio MCP server when installed with the optional `mcp` feature:

```bash
cargo install clincalc --features mcp
clincalc mcp
```

The MCP server exposes every calculator as a typed tool named `clincalc_<name>` using the calculator's own JSON Schema. See [`docs/mcp.md`](docs/mcp.md) for host configuration and safety notes.

## Python package

The Python package requires CPython 3.9 or later. Install it from [PyPI](https://pypi.org/project/clincalc/):

```bash
python -m pip install clincalc
```

Then run a calculation. Inputs and outputs are ordinary Python dictionaries:

```python
import clincalc

result = clincalc.calculate("bmi", {"weight_kg": 70, "height_cm": 175})
assert result["result"] == 22.9
print(result["calculator"], result["result"])
# bmi 22.9
```

For batch calculation over pandas DataFrames, install the optional extra:

```bash
python -m pip install "clincalc[pandas]"
```

```python
import pandas as pd

results = clincalc.batch("bmi", pd.DataFrame({"weight_kg": [70], "height_cm": [175]}))
```

See [`docs/python.md`](docs/python.md) for the full API and pandas helpers.

## The library

The registry includes calculators from QRISK3, PHQ-9, GAD-7, eGFR and FIB-4 through NEWS2, CURB-65, the Wells scores, CHA2DS2-VASc, DAS28, SOFA, MELD, CHALICE and Gleason. Run `clincalc list` for the authoritative current set.

### Calculator status

This table tracks every functioning calculation. `✓` means the language is available as a complete reviewed bundle or the implementation has passed clincalc's primary-source clinical verification and testing gates; `-` means that language is not yet available. English is the source language. The named-but-unavailable proprietary entries are listed separately below because they do not calculate a result.

| Calculator | English | Catalan | Spanish | Clinically verified |
|---|:---:|:---:|:---:|:---:|
| `abcd2` | ✓ | - | - | ✓ |
| `abpi` | ✓ | - | - | ✓ |
| `alcohol_units` | ✓ | - | - | ✓ |
| `alvarado` | ✓ | - | - | ✓ |
| `amts` | ✓ | - | - | ✓ |
| `anion_gap` | ✓ | - | - | ✓ |
| `apache2` | ✓ | - | - | ✓ |
| `apgar` | ✓ | - | - | ✓ |
| `asa_physical_status` | ✓ | - | - | ✓ |
| `ascvd` | ✓ | - | - | ✓ |
| `asrs` | ✓ | - | - | ✓ |
| `audit` | ✓ | - | - | ✓ |
| `auditc` | ✓ | - | - | ✓ |
| `barthel` | ✓ | - | - | ✓ |
| `basdai` | ✓ | - | - | ✓ |
| `binet` | ✓ | - | - | ✓ |
| `bmi` | ✓ | - | - | ✓ |
| `bode` | ✓ | - | - | ✓ |
| `body_fat_circumference` | ✓ | - | - | ✓ |
| `body_surface_area` | ✓ | - | - | ✓ |
| `braden` | ✓ | - | - | ✓ |
| `caprini` | ✓ | - | - | ✓ |
| `centor` | ✓ | - | - | ✓ |
| `cha2ds2_va` | ✓ | - | - | ✓ |
| `cha2ds2vasc` | ✓ | - | - | ✓ |
| `chalice` | ✓ | - | - | ✓ |
| `charlson` | ✓ | - | - | ✓ |
| `child_pugh` | ✓ | - | - | ✓ |
| `ciwa_ar` | ✓ | - | - | ✓ |
| `ckd_risk` | ✓ | - | - | ✓ |
| `cockcroft_gault` | ✓ | - | - | ✓ |
| `corrected_calcium` | ✓ | - | - | ✓ |
| `corrected_sodium` | ✓ | - | - | ✓ |
| `cows` | ✓ | - | - | ✓ |
| `curb65` | ✓ | - | - | ✓ |
| `das28` | ✓ | - | - | ✓ |
| `egfr` | ✓ | - | - | ✓ |
| `ehra` | ✓ | - | - | ✓ |
| `energy_requirement` | ✓ | - | - | ✓ |
| `epds` | ✓ | - | - | ✓ |
| `euroscore2` | ✓ | - | - | ✓ |
| `familial_hypercholesterolaemia` | ✓ | - | - | ✓ |
| `fena` | ✓ | - | - | ✓ |
| `feverpain` | ✓ | - | - | ✓ |
| `fib4` | ✓ | - | - | ✓ |
| `findrisc` | ✓ | - | - | ✓ |
| `four_ts` | ✓ | - | - | ✓ |
| `fourat` | ✓ | - | - | ✓ |
| `free_water_deficit` | ✓ | - | - | ✓ |
| `gad7` | ✓ | - | - | ✓ |
| `gcs` | ✓ | - | - | ✓ |
| `glasgow_blatchford` | ✓ | - | - | ✓ |
| `gleason` | ✓ | - | - | ✓ |
| `grace` | ✓ | - | - | ✓ |
| `hasbled` | ✓ | - | - | ✓ |
| `heart` | ✓ | - | - | ✓ |
| `hinchey` | ✓ | - | - | ✓ |
| `homa_ir` | ✓ | - | - | ✓ |
| `ipss` | ✓ | - | - | ✓ |
| `isth_overt_dic` | ✓ | - | - | ✓ |
| `khorana` | ✓ | - | - | ✓ |
| `ldl_cholesterol` | ✓ | - | - | ✓ |
| `lrinec` | ✓ | - | - | ✓ |
| `max_heart_rate` | ✓ | - | - | ✓ |
| `meld` | ✓ | - | - | ✓ |
| `meld_3` | ✓ | - | - | ✓ |
| `mrc_dyspnoea` | ✓ | - | - | ✓ |
| `news2` | ✓ | - | - | ✓ |
| `nhfs` | ✓ | - | - | ✓ |
| `nihss` | ✓ | - | - | ✓ |
| `npi` | ✓ | - | - | ✓ |
| `one_rep_max` | ✓ | - | - | ✓ |
| `padua` | ✓ | - | - | ✓ |
| `pasi` | ✓ | - | - | ✓ |
| `perc` | ✓ | - | - | ✓ |
| `phq9` | ✓ | - | - | ✓ |
| `pitt_bacteraemia` | ✓ | - | - | ✓ |
| `psa_density` | ✓ | - | - | ✓ |
| `qfracture` | ✓ | - | - | ✓ |
| `qrisk3` | ✓ | - | - | ✓ |
| `qsofa` | ✓ | - | - | ✓ |
| `rcri` | ✓ | - | - | ✓ |
| `relative_fat_mass` | ✓ | - | - | ✓ |
| `sofa` | ✓ | - | - | ✓ |
| `timi` | ✓ | - | - | ✓ |
| `uacr` | ✓ | - | - | ✓ |
| `ukeld` | ✓ | - | - | ✓ |
| `waist_to_height_ratio` | ✓ | - | - | ✓ |
| `waist_to_hip_ratio` | ✓ | - | - | ✓ |
| `waterlow` | ✓ | - | - | ✓ |
| `wells_dvt` | ✓ | - | - | ✓ |
| `wells_pe` | ✓ | - | - | ✓ |
| `wilks` | ✓ | - | - | ✓ |

### Proprietary tools are named, not hidden

A handful of tools cannot be shipped because they are proprietary or licence-locked (FRAX, MMSE, ELF, ACQ, the Oxford Hip/Knee Scores, CAT, MUST, CFS, LANSS). Rather than omit them silently, each is registered as a calculator that returns a structured explanation - the owner, why it cannot be shipped, open alternatives (often one shipped here), and how to advocate for open clinical tools:

```console
$ clincalc calc frax --input '{}'
frax = unavailable: proprietary
FRAX ... is not available because it is proprietary or licence-locked. Owner:
University of Sheffield ... Open alternatives: qfracture ...
```

## Architecture: one core, many surfaces

The dependency arrows all point **into** the core, which never depends on anything above it. All surfaces are additive - none requires a rewrite of the others.

| Surface | Status | Notes |
|---|---|---|
| **Rust library** | Shipped | `clincalc = { default-features = false }` - pure leaf, only `serde` + `serde_json`. Embeddable in any Rust program. |
| **Human CLI** | Shipped | `clincalc calc`, `clincalc list`, readable output with copy-paste text summary. |
| **Programmatic / stdio CLI** | Shipped | `--format json`, JSON stdin input - usable by any process that can exec a subprocess. |
| **MCP server** | Shipped | `clincalc mcp` (optional `mcp` feature) - each calculator as a typed MCP tool for LLM hosts. |
| **Desktop GUI** | Planned | Tauri desktop app; see `spec/gui.md`. Calls the Rust engine natively - no HTTP round-trip. |
| **REST API** | Shipped | Default-enabled `rest-api` feature - `clincalc api` starts an axum server with `GET /calculators`, per-calculator `POST`, and `GET /openapi.json`. |
| **Web UI** | Planned | `clincalc-web` single-file HTML; see `spec/roadmap.md`. |
| **Python FFI** | Shipped | `pip install clincalc`; `clincalc.calculate("egfr", {...})` and `clincalc.batch("egfr", df)`. Implemented in `python/` as a separate `pyo3` crate so the core stays leaf-clean. |

The concrete Rust structure:

- **`clincalc`** (this crate) - the scoring engine. With `default-features = false` it is a strict leaf: only `serde` and `serde_json`, no async runtime, no host dependency.
- The default **`cli` feature** adds the `clincalc` binary and the reusable `clincalc::cli` module. A host CLI (e.g. GitEHR's `gitehr calc`) calls this module directly.
- The optional **`mcp` feature** adds `clincalc mcp`, a local stdio MCP server. Each calculator in `clincalc::all()` becomes a typed MCP tool automatically.
- The default-enabled **`rest-api` feature** follows the same feature-gated pattern as `mcp`: `clincalc api` starts an axum server with an auto-generated OpenAPI spec.
- The **`clincalc` Python package** (`python/`) follows the same optional-dependency pattern, giving data-science workflows access to every calculator via PyPI without touching the Rust crate's dependency graph.

Adding a calculator to `clincalc::all()` surfaces it everywhere - CLI, MCP, REST API, GUI, web - with no per-surface code.

### Input definitions

Several inputs are clinician-asserted predicates whose TRUE/FALSE conditions are easy to get subtly wrong (for example, "vascular disease" in CHA2DS2-VASc is arterial and excludes venous thromboembolism). Each such input carries a machine-readable definition - includes, excludes, a cited source, and a draft SNOMED ECL - that travels in the schema to every surface. See `spec/calculator-input-definitions.md`.

## Embedding in a host (for example, GitEHR)

Any application can embed these crates. GitEHR ([gitehr/gitehr](https://github.com/gitehr/gitehr)) is one consumer: its CLI forwards `gitehr calc` to `clincalc::cli::run`, and a host MCP server can expose each calculator from `clincalc::all()` as a `clincalc_<name>` tool whose input schema is the calculator's own JSON Schema. The calculators are the engine; a host wires them into its own surfaces.

## Develop

```bash
cargo test                                      # all calculators
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
```

CI enforces all three. Adding a calculator: implement it in `clincalc` (typed input, pure `compute`, `build_response`, a `Calculator` impl with `input_schema()` and `license()`, and literature-vector tests), register it in `all()`, and that is the only Rust work - the CLI and MCP surfaces pick it up automatically. See `AGENTS.md`, `spec/calculators.md`, `spec/calculator-input-definitions.md`, and the examples in `examples/`.

## Licensing

- Original `clincalc` code: AGPL-3.0-or-later. This work is deliberately not available for subsumption into proprietary EHRs; if that service needs to exist, it can be offered as a hosted Calc-API.
- Clinical algorithms are implemented from primary literature (most scores are public-domain methods). The QRISK3 and QFracture modules are derivative ports of ClinRisk's LGPL-3.0-or-later source, retain that licence, and carry ClinRisk's required disclaimer with every score. Each calculator records its own distribution licence via `clincalc calc <name> --license`.
- Clinical content (source references) under CC-BY-SA-4.0.
- The ASRS-v1.1 six-question scorer is distributed with the rights holders' required attribution and accepts coded adult responses covering the past six months from the [authorised form](https://license.tov.med.nyu.edu/product/asrs6Qscreener). It reports the classic dichotomous and alternative continuous methods separately. Current source and releases from `0.3.0` onward do not distribute questionnaire text or the separately licensed 18-question checklist; legacy `0.2.2` source artifacts did include the checklist and should not be used.

## Roadmap

See [`spec/roadmap.md`](spec/roadmap.md) for engineering, infrastructure, distribution, GUI, and product-level work, and [`spec/calculator-roadmap.md`](spec/calculator-roadmap.md) for the clinical-calculator backlog.
