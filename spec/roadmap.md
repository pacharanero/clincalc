# Roadmap

Engineering, infrastructure, and product-level work for the `clincalc` project. **This file is the home for everything that is not a new calculator.** The clinical-calculator backlog lives in [`calculator-roadmap.md`](calculator-roadmap.md).

Roadmap items have stable identifiers so they can be referred to in conversation, commits, PRs, and release notes. Do not renumber existing IDs just because items are completed or removed.

Completed items are removed from this file rather than kept as a historical changelog. Use Git history and `CHANGELOG.md` for completed work.

## Status legend

- `[~]` **In-progress** - actively being worked on or dormant pending a one-off setup.
- `[ ]` **Planned** - committed to build; the next batch.
- `[ ]` **Future** - under consideration; promote to Planned when scheduled.

---

## Distribution & release

### Planned

- [ ] **REL-002 Windows code-signing** - EV cert from Sectigo / SSL.com once procured. The cert covers `sct`, `dsc`, **and** `clincalc` in one purchase; see [`gui.md`](gui.md#windows-code-signing). Until then the GUI installer triggers SmartScreen on first run.

### Future

- [ ] **REL-003 `cargo binstall` metadata** - cargo-dist releases are natively binstall-discoverable; add an explicit `[package.metadata.binstall]` override only if a case needs it.
- [ ] **REL-004 deb / rpm / Scoop packaging** - only if user demand surfaces.

---

## Desktop GUI

See the design spec at [`gui.md`](gui.md) and the implementation guide at [`gui/README.md`](../gui/README.md).

### Planned

- [ ] **GUI-001 Decide CI build cadence for the GUI** - on every push (slow, ~5 min) vs on release tag only. Probably the latter once cargo-dist lands.

### Future

- [ ] **GUI-002 Updater** - Tauri's built-in updater speaking to a manifest hosted on the docs site.
- [ ] **GUI-003 iOS / Android builds** - Tauri 2 supports them and `clincalc` is pure Rust, so this is mostly a packaging question.
- [ ] **GUI-004 Theme parity with GitEHR** - shared CSS variables or a tiny `@clincalc/ui-tokens` package, so the two apps stay visually coherent without copy-paste drift.

---

## Authoring workflow & docs

### Future

- [ ] **DOC-002 `docs/translating.md`** - contribution path for translators once multilingual lands.

---



## Engine & embedding

### ENG-001 Multilingual support

Status: In-progress

Adopt BCP 47 locale identifiers, RFC 4647 lookup, complete calculator translation bundles, and MessageFormat-compatible message design per [`multilingual.md`](multilingual.md), without breaking the leaf rule or existing English APIs.

- [ ] **ENG-001.4 Enforce complete translation bundles** - metadata, governed schema prose, and computed prose must reach key parity before a calculator advertises a locale. Fall back the complete calculator representation, never individual strings.
- [ ] **ENG-001.5 Review CURB-65 and two more MedikQuantis overlaps** - obtain and record native-speaker clinical review of the final adapted English, Spanish, and Catalan CURB-65 wording, then validate two calculators of different shapes before opening translation batches.
- [ ] **ENG-001.6 Wire locale through remaining surfaces** - CLI `--locale` / `CLINCALC_LOCALE` and the Python keyword-only `locale` (on `calculate`, `list_calculators`, `get_schema`, `get_template`, `batch`) are implemented; add REST query / `Accept-Language` / `Content-Language` / `Vary`, MCP session configuration, and GUI locale and text direction. All surfaces report the resolved locale.
- [x] **ENG-001.7 Write `docs/translating.md`** - contribution, attribution, native/clinical review, key-parity, and stale-source workflow before opening batched translation.

Decisions:
- Locale is per operation/request, with an immutable configured default at surface boundaries and no process-global engine state.
- The core does not implement a Mustache-like parser. Initial computed messages use explicit locale-specific Rust renderers; a standards-based renderer may later live behind an optional feature without changing message IDs or arguments.

### ENG-002 Translation reciprocity with MedikQuantis

Status: Future

[MedikQuantis](https://medikquantis.me) ships Catalan/Spanish/English calculator strings for several scores we also implement. The goal is to avoid duplicating native-speaker translation work.

- [ ] **ENG-002.1 Align tag taxonomy** with MedikQuantis so calculators are discoverable under the same specialty labels in both projects.
- [ ] **ENG-002.2 Agree a shared citation shape** (PMID + URL + access date) so either project can ingest the other's metadata.
- [ ] **ENG-002.3 Write a converter** (one-way or bidirectional) that maps their exported JSON catalogue to our structured translation bundles, preserving stable message IDs, provenance, reviewer attribution, and licence.
- [ ] **ENG-002.4 Pull Catalan/Spanish strings** for CURB-65 first, then the remaining overlapping calculators, as the reviewed seed translations for ENG-001.

Open questions for comment:
- Do we want a formal data-sharing agreement or is MIT-to-AGPL ingestion already acceptable with attribution?
- Should this live as a one-off import script or a recurring sync job?

### ENG-003 `clincalc-web`

Status: Future

A single-file HTML/web UI for calculators. The headline is copy-paste interoperability, so the web surface should produce the same clean text summary as the CLI and let users copy results into an EHR.

- [ ] **ENG-003.1 Compile the `clincalc` engine to WebAssembly** (`wasm32-unknown-unknown`) with `default-features = false` so the browser surface shares the scoring logic.
- [ ] **ENG-003.2 Build a minimal HTML/CSS/JS shell** that loads the WASM module, lists calculators from `clincalc::all()`, renders forms from `input_schema()`, and displays the JSON/text result.
- [ ] **ENG-003.3 Keep it single-file or very few files** so it can be deployed to GitHub Pages or opened locally without a build step for end users.
- [ ] **ENG-003.4 Add a `clincalc web` CLI subcommand** (behind a new `web` feature?) that serves the bundle locally, or ship it as a static asset on the docs site.

Open questions for comment:
- Should the web UI be a separate repo (`clincalc-web`) or a sub-directory here that publishes to the existing Pages site?
- Is a WASM build worth the toolchain complexity, or is duplicating logic in TypeScript acceptable for a read-only web demo?

### ENG-004 FHIR Observation export

Status: Future

Map `CalculationResponse` to FHIR `Observation` resources for interoperability with EHRs and research pipelines.

- [ ] **ENG-004.1 Define a mapping** from calculator result fields to FHIR Observation elements: `code` (LOINC/SNOMED where available), `valueQuantity` or `valueString`, `component` for sub-scores, `note` for interpretation and citation.
- [ ] **ENG-004.2 Add an optional `fhir` feature** to `clincalc` that depends only on `serde_json` (to keep the leaf rule) and exports `CalculationResponse::to_fhir_observation(...)` returning a JSON object ready to POST to a FHIR server.
- [ ] **ENG-004.3 Keep network code out of the engine**; the CLI can add a `clincalc export --format fhir` command and hosts can POST the JSON themselves.
- [ ] **ENG-004.4 Start with clear LOINC codes** (BMI, eGFR) and document the mapping table.

Open questions for comment:
- Do we need full FHIR R4 validation, or is a documented JSON shape sufficient?
- Should we lobby for / contribute new LOINC codes for scores that lack them?

### ENG-005 Unit conversion

Status: Future

Accept metric and imperial measurements at the input boundary so callers do not need to convert manually. The engine stays units-explicit internally.

- [ ] **ENG-005.1 Add a `Unit` enum or string alias** per field type: `weight: { kg, lb, st }`, `height: { cm, ft_in }`, `creatinine: { umol/L, mg/dL }`, etc.
- [ ] **ENG-005.2 Validate and convert at the `Calculator::calculate` entrypoint** before passing the typed `Input` to `compute`. Reject ambiguous or mixed-unit inputs.
- [ ] **ENG-005.3 Keep `interpretation` output in the same units** as the input so the copied text matches what the clinician entered.
- [ ] **ENG-005.4 Document conversion constants and rounding rules**; cite the source for any non-obvious conversions.

Open questions for comment:
- Should conversion be automatic, or gated behind an `allow_unit_conversion` flag to preserve auditability?
- Stone/pound entry is common in UK primary care - do we support composite units or require a single numeric field?

### ENG-006 Printable / clipboard-friendly result formatting

Status: Future

Improve the copy-paste output beyond the existing plain text block, with richer formats that preserve the citation.

- [ ] **ENG-006.1 Define a `Formatter` trait or output modes** in `clincalc::cli` (or a new module): `text`, `markdown`, `html`, `pdf`, `rtf`.
- [ ] **ENG-006.2 Keep `text` as default** for CLI stdout and MCP.
- [ ] **ENG-006.3 Add `markdown` output** with headings, bullet working steps, and a hyperlink to the primary reference - useful for pasting into EHR free-text or notes apps.
- [ ] **ENG-006.4 Add `html` and `pdf` rendering** via a small template engine for the web/GUI surfaces.
- [ ] **ENG-006.5 Keep the engine returning the same structured `CalculationResponse`**; formatting is a surface concern.

Open questions for comment:
- Which formats are actually used by clinicians today? Markdown + PDF seems highest value; RTF probably lowest.
- Should the formatter be an engine feature or a CLI-only concern?

### ENG-007 Plugin system

Status: Future

Allow users or third parties to load calculators at runtime without recompiling `clincalc`. Useful for trust-local calculators, research tools, or proprietary algorithms that cannot ship in the core crate.

- [ ] **ENG-007.1 Define a stable plugin interface**: a JSON schema, a WASM module, or a dynamic Rust trait loaded from a `.so`/`.dll`.
- [ ] **ENG-007.2 Choose WASM plugins as the default** (sandboxed, language-agnostic, and aligned with ENG-003). Each plugin exposes the same `Calculator` trait shape via a small ABI.
- [ ] **ENG-007.3 Add `clincalc plugin list / add / remove`** to the CLI and a plugin directory (`~/.local/share/clincalc/plugins/`).
- [ ] **ENG-007.4 Validate plugins on load**: schema is valid, license is present, name does not collide with core calculators (or is namespaced as `plugin:<name>`).
- [ ] **ENG-007.5 Keep the core engine unchanged**; the CLI and GUI load plugins at startup.

Open questions for comment:
- WASM plugins add a runtime dependency (`wasmtime` or `wasmer`). Is that acceptable behind a `plugins` feature, or do we prefer JSON-only "calculator definitions" for simple scoring rules?
- How do we handle plugin versioning and updates - semver, hash pinning, or trust-store?

### ENG-008 Guideline-update registry

Status: Future

A mechanism to re-verify each calculator's licence and reference URL on a schedule, so dead links or superseded guidelines do not silently rot.

- [ ] **ENG-008.1 Add `last_verified` date and `verification_url`** to `CalculatorLicense`.
- [ ] **ENG-008.2 Provide a `clincalc audit` command** (or `cargo xtask audit-references`) that HEAD-requests every `source_url`, reports 404s/redirects, and flags calculators whose `last_verified` is older than a threshold.
- [ ] **ENG-008.3 Integrate with CI as a scheduled job** (monthly) that opens an issue or fails a build if references go stale.
- [ ] **ENG-008.4 Keep this out of the hot path**; it is a maintenance tool, not part of scoring.

Open questions for comment:
- Should stale references fail CI or just open a tracking issue?
- Do we also diff guideline PDFs/texts, or is URL liveness + `last_verified` enough?

### ENG-009 High-risk-score alerts

Status: Future

Let embedding hosts subscribe to high-risk result events, for example NEWS2 >= 7 or CURB-65 >= 3.

- [ ] **ENG-009.1 Define a `RiskThreshold` descriptor** per calculator: `{ score >= 7 }`, `{ interpretation == "high" }`, etc.
- [ ] **ENG-009.2 Add an optional `alerts` field** to `CalculationResponse` listing triggered thresholds and recommended actions.
- [ ] **ENG-009.3 Expose this through `clincalc::cli::run`** so MCP hosts and the GUI can subscribe without per-calculator logic.
- [ ] **ENG-009.4 Keep thresholds configurable** per deployment so a host can override defaults.

Open questions for comment:
- Should thresholds live in the calculator module or in a central policy file?
- Is a simple numeric threshold enough, or do we need rule expressions (`(news2 >= 7) OR (red_score == true)`)?

### ENG-010 Generic centile engine

Status: Future

A generic centile-lookup engine for calculators that place a measured value on age/sex/region-specific centile curves, rather than aggregating points into a score. Driven by CALC-053 (CREAM-Kids axial length) and CALC-032 (RCPCH Digital Growth Charts), but designed once for all current and future centile references.

The AGPL-licensed RCPCH `rcpchgrowth-python` library is a useful reference implementation for engine behaviour and test design, but it does not grant blanket rights to its underlying reference datasets. Each LMS table needs an independent provenance and redistribution review; in particular, the UK growth references are subject to MRC licensing rather than the code repository's AGPL terms.

- [ ] **ENG-010.1 Implement the LMS Box-Cox transforms in pure Rust** - measurement to SDS (`z = ((x/M)^L - 1) / (L*S)`, or `log(x/M) / S` when `L == 0`) and the inverse SDS to measurement. No `statrs` or `ndarray` dependency - the strict leaf rule requires a dependency-free error-function approximation for the normal CDF (Abramowitz-Stegun, accurate to ~1e-7).
- [ ] **ENG-010.2 Implement cubic Lagrange and linear interpolation** of L, M, S independently, matching Tim Cole's LMSGrowth algorithm. Let each reference declare its interpolation strategy (cubic default, linear for WHO per `anthro`/`anthroplus`).
- [ ] **ENG-010.3 Define a `CentileReference` trait** - each reference dataset supplies its LMS table for a given `(age, sex, parameter)`. The engine handles lookup, interpolation, and the LMS math; the reference handles only data and coverage policy (age range, prematurity correction, overlap disjunctions).
- [ ] **ENG-010.4 Store approved reference data as JSON** - flat arrays of `{decimal_age, l, m, s}` embedded via `include_str!` at build time, with source, version, and redistribution terms recorded beside each dataset. No runtime file I/O or CSV/XLSX parser belongs in the engine.
- [ ] **ENG-010.5 Build licensed conformance vectors** - derive vectors from primary publications or fixtures whose code and underlying data terms permit redistribution. Do not copy restricted UK reference tables or treat every RCPCH fixture as automatically reusable.
- [ ] **ENG-010.6 Build CALC-053 (CREAM-Kids axial length)** only after confirming that reusable coefficients or tables are available and recording their terms. Test against values directly reported by the publication; leave serial-change risk out unless separately specified and validated.
- [ ] **ENG-010.7 Build CALC-032 (RCPCH Digital Growth Charts)** on the same engine one reference at a time. Verify the terms for WHO, CDC, UK-WHO, and every other dataset independently; UK reference inclusion remains blocked until the required MRC permission is established.

Key lessons from the RCPCH library (avoid these pitfalls):
- Code licensing and reference-data licensing are separate gates; passing the AGPL code review does not make MRC-controlled tables redistributable.
- Interpolation method is per-reference, not universal (WHO uses linear, UK-WHO uses cubic).
- Daily LMS data eliminates interpolation in practice - the engine should support dense tables natively.
- The +/-3 SD WHO tail correction is directionally asymmetric - applied for measurement to SDS but NOT for SDS to measurement.
- Prematurity correction is reference-specific - UK-WHO corrects for all gestations lifelong; CDC and WHO stop correcting at 2y.
- Overlap ages where two sub-references meet are a recurring bug source - the reference trait must handle disjunctions, not the engine.

---

## Collaboration & sister projects

### COLL-001 Converge CHA₂DS₂-VASc with MedikQuantis

Status: In-progress

- [x] **COLL-001.1 Verify point-for-point agreement** with the MedikQuantis implementation:
   - Age 65-74 = 1 point, ≥75 = 2 points.
   - Female sex = 1 point.
   - Congestive heart failure, hypertension, diabetes, vascular disease = 1 point each.
   - Prior stroke / TIA = 2 points; confirm whether MedikQuantis also includes systemic arterial thromboembolism.
   - Maximum score = 9.

   Confirmed 2026-07-28 (point values) and 2026-08-24 (S2 scope) against MedikQuantis's actual `cha2ds2vasc.ts` source and its `en`/`es` UI copy (not just its description, pinned at commit `cf5afb9`). Point values agree for the shared input semantics. MedikQuantis's `strokeOrTia` field is labelled "Prior stroke or TIA" / "Ictus o AIT previo" with no other description, confirming it does not extend to systemic arterial thromboembolism the way `clincalc`'s `stroke_tia_thromboembolism` does - a semantic-scope divergence, not a scoring disagreement. See "Cross-project verification" in `spec/calculators/cha2ds2-vasc.md`, which also documents a recommendation-threshold divergence and a citation mismatch in the MedikQuantis source (both still open).
- [x] **COLL-001.2 Confirm the female-sex-only edge case** is handled identically: a total score of 1 arising only from female sex is treated as low risk (no anticoagulation), matching NICE NG196. clincalc already encodes this via `non_sex_score`. Confirmed identical in MedikQuantis's source 2026-07-28.
- [x] **COLL-001.3 Add literature-vector tests** pinning the exact boundary cases Laura raised (age 74 vs 75) and the untreated-cohort stroke event rates per 100 patient-years from Friberg 2012 (PMID 22246443) to `src/calculators/cha2ds2vasc.rs`. Add the rate and its reference to `working` so they are visible to both projects.
- [x] **COLL-001.4 Create `spec/calculators/cha2ds2-vasc.md`** as a shared source of truth: scoring table, recommendation rules, input definitions, reference, and a test-vector table.
- [ ] **COLL-001.5 Invite Laura to review the spec** and contribute her MedikQuantis test cases in whatever format is easiest for her to share.
- [ ] **COLL-001.6 Surface the collaboration** by adding a "sister projects" note to `docs/calculators.md` linking to MedikQuantis once converged.

Open questions for comment:
- Should the event-rate table be part of the public `CalculationResponse` result or only in `working`?
- Do we align on the NICE NG196 recommendation wording, or keep locale-specific guidance separate until ENG-001 lands?

### COLL-002 Invite MedikQuantis author into the clincalc team

Status: Future

- [ ] **COLL-002.1 Propose deeper collaboration** after COLL-001 succeeds: Laura moves her calculator implementations to clincalc and uses its Rust engine + her own front-end, rather than duplicating scoring logic.
- [ ] **COLL-002.2 Offer co-maintainership** of clincalc and credit for the Catalan/Spanish translation seed data (ENG-002).
- [ ] **COLL-002.3 Migrate overlapping translations** into clincalc's structured translation bundles once ENG-001 establishes the CURB-65 pattern.
- [ ] **COLL-002.4 Keep MedikQuantis running** as a sibling consumer / demonstrator until the transition is seamless.

Open questions for comment:
- What governance / licence questions need to be resolved for MedikQuantis code/strings to move into the AGPL clincalc repo?
- Should we set up a shared chat channel or regular sync for the collaboration?

---

## Calculator backlog

See [`calculator-roadmap.md`](calculator-roadmap.md).

The live calculator counts and remaining candidates are maintained in [`docs/calculators.md`](../docs/calculators.md) and [`spec/calculator-roadmap.md`](calculator-roadmap.md), respectively.
