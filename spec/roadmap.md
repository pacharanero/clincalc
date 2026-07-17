# Roadmap

Engineering, infrastructure, and product-level work for the `clincalc` project. **This file is the home for everything that is not a new calculator.** The clinical-calculator backlog lives in [`spec/calculator-roadmap.md`](spec/calculator-roadmap.md).

Roadmap items have stable identifiers so they can be referred to in conversation, commits, PRs, and release notes. Do not renumber existing IDs just because items are completed or removed.

Completed items are removed from this file rather than kept as a historical changelog. Use Git history and `CHANGELOG.md` for completed work.

## Status legend

- `[~]` **In-progress** - actively being worked on or dormant pending a one-off setup.
- `[ ]` **Planned** - committed to build; the next batch.
- `[ ]` **Future** - under consideration; promote to Planned when scheduled.

---

## Distribution & release

### In-progress

- [~] **REL-001 Activate releases** - the pipeline is built and validated (`dist plan` green, workflows lint-clean) but dormant until two repo secrets are set:
    - `CARGO_REGISTRY_TOKEN` - crates.io API token with publish rights on `clincalc`.
    - `HOMEBREW_TAP_TOKEN` - PAT with write access to `pacharanero/homebrew-tap`.

    Once set, `s/version++ [patch|minor|major]` cuts a release: it bumps the workspace (and excluded GUI) version, regenerates `CHANGELOG.md` (git-cliff), and lands `chore(release): vX.Y.Z` on `main`. `auto-tag.yml` then tags `vX.Y.Z` and invokes `release.yml` and `publish-crates.yml`. After the first publish, `cargo install clincalc` works without `--git`, and downstream consumers (notably [GitEHR](https://github.com/gitehr/gitehr)) can depend on `clincalc` (with `default-features = false` for the pure engine) from crates.io.

    The docs-site `install.sh` / `install.ps1` proxy scripts are wired into the Pages deploy and will become usable as soon as the first cargo-dist release assets exist.

### Planned

- [ ] **REL-002 Windows code-signing** - EV cert from Sectigo / SSL.com once procured. The cert covers `sct`, `dsc`, **and** `clincalc` in one purchase; see [`spec/gui.md`](spec/gui.md#windows-code-signing). Until then the GUI installer triggers SmartScreen on first run.

### Future

- [ ] **REL-003 `cargo binstall` metadata** - cargo-dist releases are natively binstall-discoverable; add an explicit `[package.metadata.binstall]` override only if a case needs it.
- [ ] **REL-004 deb / rpm / Scoop packaging** - only if user demand surfaces.

---

## Desktop GUI

See the design spec at [`spec/gui.md`](spec/gui.md) and the implementation guide at [`gui/README.md`](gui/README.md).

### Planned

- [ ] **GUI-001 Decide CI build cadence for the GUI** - on every push (slow, ~5 min) vs on release tag only. Probably the latter once cargo-dist lands.

### Future

- [ ] **GUI-002 Updater** - Tauri's built-in updater speaking to a manifest hosted on the docs site.
- [ ] **GUI-003 iOS / Android builds** - Tauri 2 supports them and `clincalc` is pure Rust, so this is mostly a packaging question.
- [ ] **GUI-004 Theme parity with GitEHR** - shared CSS variables or a tiny `@clincalc/ui-tokens` package, so the two apps stay visually coherent without copy-paste drift.

---

## Authoring workflow & docs

### Planned

The docs.rs front page now documents the `cli` / `mcp` feature flags, `default-features = false` leaf usage, and a minimal registry example. docs.rs is configured to build all features so the optional MCP module is visible.

### Future

- [ ] **DOC-002 `docs/translating.md`** - contribution path for translators once multilingual lands.

---



## REST API surface

A persistent HTTP server exposing every calculator as a JSON endpoint. Implemented as an optional `rest-api` feature (axum + tokio), following the same pattern as the `mcp` feature - the engine is unchanged, only a new surface is added.

---

## Python FFI

A `clincalc-py` crate using `pyo3` to expose the engine to Python data-science workflows. Kept as a separate crate so the core `clincalc` crate remains leaf-clean.

---

## Engine & embedding

### ENG-001 Multilingual support

Status: Future

Add a `Locale` enum and `LocalizedString` storage per [`spec/multilingual.md`](spec/multilingual.md) without breaking the leaf rule. Translations stay `&'static str` in-tree, keyed and looked up at compile time.

Proposed implementation steps:
1. Introduce `clincalc::Locale { En, Es, Ca, ... }` with `En` as the default fallback.
2. Add `LocalizedString { en, es, ca }` and a `.get(locale)` method; strings live in each calculator module.
3. Extend `Calculator` trait methods that emit human prose with an optional/defaulted `locale: Locale` parameter: `title`, `description`, `input_schema`, and `calculate`. Machine identifiers (`name`, `working` keys, enum variants) remain English.
4. Convert `interpretation` strings from inline `format!` to a structured interpretation object keyed by stable IDs (band, score, recommendation), with localised labels and a small templater. This is less fragile than format-string argument ordering across languages.
5. Wire `--lang` and `CALC_LANG` through `clincalc::cli::run` so all surfaces (CLI, MCP, Python, REST API) receive the same locale.
6. Validate with FeverPAIN as the first migrated calculator, then translate three calculators with a native speaker before opening the catalogue for batched translation.
7. Write `docs/translating.md` contribution path.

Open questions for comment:
- Should the REST API and Python API accept `locale` per-request, or default to a configured locale? Per-request is more flexible but complicates caching.
- Is a tiny internal Mustache-like templater acceptable, or should we depend on a battle-tested crate behind an optional feature?

### ENG-002 Translation reciprocity with MedikQuantis

Status: Future

[MedikQuantis](https://medikquantis.me) ships Catalan/Spanish/English calculator strings for several scores we also implement. The goal is to avoid duplicating native-speaker translation work.

Proposed implementation steps:
1. Align tag taxonomy with MedikQuantis so calculators are discoverable under the same specialty labels in both projects.
2. Agree a shared citation shape (PMID + URL + access date) so either project can ingest the other's metadata.
3. Write a one-way or bidirectional converter that maps their exported JSON/YAML catalogue to our `LocalizedString` blocks, preserving provenance and licence.
4. Pull their Catalan/Spanish strings for the 14 overlapping calculators (DAS28, CHA2DS2-VASc, HEART, Wells PE, GRACE, TIMI, qSOFA, SOFA, CURB-65, Child-Pugh, IPSS, HAS-BLED, Wells DVT, CKD-EPI) as the seed translations for ENG-001.

Open questions for comment:
- Do we want a formal data-sharing agreement or is MIT-to-AGPL ingestion already acceptable with attribution?
- Should this live as a one-off import script or a recurring sync job?

### ENG-003 `clincalc-web`

Status: Future

A single-file HTML/web UI for calculators. The headline is copy-paste interoperability, so the web surface should produce the same clean text summary as the CLI and let users copy results into an EHR.

Proposed implementation steps:
1. Compile the `clincalc` engine to WebAssembly (`wasm32-unknown-unknown`) with `default-features = false` so the browser surface shares the scoring logic.
2. Build a minimal HTML/CSS/JS shell that loads the WASM module, lists calculators from `clincalc::all()`, renders forms from `input_schema()`, and displays the JSON/text result.
3. Keep it single-file or very few files so it can be deployed to GitHub Pages or opened locally without a build step for end users.
4. Add a `clincalc web` CLI subcommand (behind a new `web` feature?) that serves the bundle locally, or ship it as a static asset on the docs site.

Open questions for comment:
- Should the web UI be a separate repo (`clincalc-web`) or a sub-directory here that publishes to the existing Pages site?
- Is a WASM build worth the toolchain complexity, or is duplicating logic in TypeScript acceptable for a read-only web demo?

### ENG-004 FHIR Observation export

Status: Future

Map `CalculationResponse` to FHIR `Observation` resources for interoperability with EHRs and research pipelines.

Proposed implementation steps:
1. Define a mapping from calculator result fields to FHIR Observation elements: `code` (LOINC/SNOMED where available), `valueQuantity` or `valueString`, `component` for sub-scores, `note` for interpretation and citation.
2. Add an optional `fhir` feature to `clincalc` that depends only on `serde_json` (to keep the leaf rule) and exports `CalculationResponse::to_fhir_observation(...)` returning a JSON object ready to POST to a FHIR server.
3. Do NOT add network code or OAuth to the engine; the CLI can add a `clincalc export --format fhir` command and hosts can POST the JSON themselves.
4. Start with calculators that have clear LOINC codes (BMI, eGFR) and document the mapping table.

Open questions for comment:
- Do we need full FHIR R4 validation, or is a documented JSON shape sufficient?
- Should we lobby for / contribute new LOINC codes for scores that lack them?

### ENG-005 Unit conversion

Status: Future

Accept metric and imperial measurements at the input boundary so callers do not need to convert manually. The engine stays units-explicit internally.

Proposed implementation steps:
1. Add a `Unit` enum or string alias per field type: `weight: { kg, lb, st }`, `height: { cm, ft_in }`, `creatinine: { umol/L, mg/dL }`, etc.
2. Validate and convert at the `Calculator::calculate` entrypoint before passing the typed `Input` to `compute`. Reject ambiguous or mixed-unit inputs.
3. Keep `interpretation` output in the same units as the input so the copied text matches what the clinician entered.
4. Document conversion constants and rounding rules; cite the source for any non-obvious conversions.

Open questions for comment:
- Should conversion be automatic, or gated behind an `allow_unit_conversion` flag to preserve auditability?
- Stone/pound entry is common in UK primary care - do we support composite units or require a single numeric field?

### ENG-006 Printable / clipboard-friendly result formatting

Status: Future

Improve the copy-paste output beyond the existing plain text block, with richer formats that preserve the citation.

Proposed implementation steps:
1. Define a `Formatter` trait or set of output modes in `clincalc::cli` (or a new module): `text`, `markdown`, `html`, `pdf`, `rtf`.
2. `text` remains the default for CLI stdout and MCP.
3. `markdown` adds headings, bullet working steps, and a hyperlink to the primary reference - useful for pasting into EHR free-text or notes apps.
4. `html` and `pdf` are rendered by a small template engine for the web/GUI surfaces.
5. Keep the engine returning the same structured `CalculationResponse`; formatting is a surface concern.

Open questions for comment:
- Which formats are actually used by clinicians today? Markdown + PDF seems highest value; RTF probably lowest.
- Should the formatter be an engine feature or a CLI-only concern?

### ENG-007 Plugin system

Status: Future

Allow users or third parties to load calculators at runtime without recompiling `clincalc`. Useful for trust-local calculators, research tools, or proprietary algorithms that cannot ship in the core crate.

Proposed implementation steps:
1. Define a stable plugin interface: a JSON schema, a WASM module, or a dynamic Rust trait loaded from a `.so`/`.dll`.
2. The safest option is WASM plugins: sandboxed, language-agnostic, and aligned with ENG-003. Each plugin exposes the same `Calculator` trait shape via a small ABI.
3. Add `clincalc plugin list / add / remove` to the CLI and a plugin directory (`~/.local/share/clincalc/plugins/`).
4. Validate plugins on load: schema is valid, license is present, name does not collide with core calculators (or is namespaced as `plugin:<name>`).
5. Core engine stays unchanged; the CLI and GUI load plugins at startup.

Open questions for comment:
- WASM plugins add a runtime dependency (`wasmtime` or `wasmer`). Is that acceptable behind a `plugins` feature, or do we prefer JSON-only "calculator definitions" for simple scoring rules?
- How do we handle plugin versioning and updates - semver, hash pinning, or trust-store?

### ENG-008 Guideline-update registry

Status: Future

A mechanism to re-verify each calculator's licence and reference URL on a schedule, so dead links or superseded guidelines do not silently rot.

Proposed implementation steps:
1. Add a `last_verified` date and `verification_url` to `CalculatorLicense`.
2. Provide a `clincalc audit` command (or `cargo xtask audit-references`) that HEAD-requests every `source_url`, reports 404s/redirects, and flags calculators whose `last_verified` is older than a threshold.
3. Integrate with CI as a scheduled job (monthly) that opens an issue or fails a build if references go stale.
4. Keep this out of the hot path; it is a maintenance tool, not part of scoring.

Open questions for comment:
- Should stale references fail CI or just open a tracking issue?
- Do we also diff guideline PDFs/texts, or is URL liveness + `last_verified` enough?

### ENG-009 High-risk-score alerts

Status: Future

Let embedding hosts subscribe to high-risk result events, for example NEWS2 >= 7 or CURB-65 >= 3.

Proposed implementation steps:
1. Define a small `RiskThreshold` descriptor per calculator: `{ score >= 7 }`, `{ interpretation == "high" }`, etc.
2. Add an optional `alerts` field to `CalculationResponse` listing triggered thresholds and recommended actions.
3. Expose this through `clincalc::cli::run` so MCP hosts and the GUI can subscribe without per-calculator logic.
4. Keep thresholds configurable per deployment so a host can override defaults.

Open questions for comment:
- Should thresholds live in the calculator module or in a central policy file?
- Is a simple numeric threshold enough, or do we need rule expressions (`(news2 >= 7) OR (red_score == true)`)?

---

## Collaboration & sister projects

### COLL-001 Converge CHA₂DS₂-VASc with MedikQuantis

Status: Planned

Laura Piró (author of [MedikQuantis](https://medikquantis.me)) opened [issue #3](https://github.com/pacharanero/clincalc/issues/3) proposing CHA₂DS₂-VASc as a first sister-project alignment. The goal is to make the two implementations agree on results and test vectors, then document a shared spec other projects can adopt.

Proposed implementation steps:
1. Verify point-for-point agreement with the MedikQuantis implementation:
   - Age 65-74 = 1 point, ≥75 = 2 points.
   - Female sex = 1 point.
   - Congestive heart failure, hypertension, diabetes, vascular disease = 1 point each.
   - Prior stroke / TIA / systemic arterial thromboembolism = 2 points.
   - Maximum score = 9.
2. Confirm the female-sex-only edge case is handled identically: a total score of 1 arising only from female sex is treated as low risk (no anticoagulation), matching NICE NG196. clincalc already encodes this via `non_sex_score`.
3. Add literature-vector tests pinning the exact boundary cases Laura raised (age 74 vs 75) and the annual stroke-risk table from Friberg 2012 (PMID 22246443) to `src/calculators/cha2ds2vasc.rs`. Add the risk percentages to `working` so they are visible to both projects.
4. Create `spec/calculators/cha2ds2-vasc.md` as a shared source of truth: scoring table, recommendation rules, input definitions, reference, and a test-vector table.
5. Invite Laura to review the spec and contribute her MedikQuantis test cases in whatever format is easiest for her to share.
6. Once converged, add a "sister projects" note to `docs/calculators.md` linking to MedikQuantis.

Open questions for comment:
- Should the risk table be part of the public `CalculationResponse` result or only in `working`?
- Do we align on the NICE NG196 recommendation wording, or keep locale-specific guidance separate until ENG-001 lands?

### COLL-002 Invite MedikQuantis author into the clincalc team

Status: Future

Long-term goal: bring Laura and the MedikQuantis corpus into clincalc rather than maintaining two separate calculator projects.

Proposed implementation steps:
1. After COLL-001 succeeds, propose that Laura move her calculator implementations to clincalc and use its Rust engine + her own front-end, rather than duplicating scoring logic.
2. Offer co-maintainership of clincalc and credit for the Catalan/Spanish translation seed data (ENG-002).
3. If she agrees, migrate the 14 overlapping calculators' translations into clincalc's `LocalizedString` format once ENG-001 lands.
4. Keep MedikQuantis running as a sibling consumer / demonstrator until the transition is seamless.

Open questions for comment:
- What governance / licence questions need to be resolved for MedikQuantis code/strings to move into the AGPL clincalc repo?
- Should we set up a shared chat channel or regular sync for the collaboration?

---

## Calculator backlog

See [`spec/calculator-roadmap.md`](spec/calculator-roadmap.md).

At time of writing: 64 active + 10 proprietary stubs shipped; 36 Future candidates queued (chiefly from MedikQuantis, plus the recently-added [StatinMD](https://www.thelancet.com/journals/landig/article/PIIS2589-7500\(26\)00047-6/fulltext)).
