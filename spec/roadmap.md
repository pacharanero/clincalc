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

### Future

- [x] **API-001 `rest-api` feature flag** - axum 0.8 + tokio behind `--features rest-api` (on by default); `clincalc api [--port 8080] [--host 127.0.0.1]` starts the server. Endpoints: `GET /calculators`, `GET /calculators/{name}/schema`, `GET /calculators/{name}/template`, `GET /calculators/{name}/license`, `POST /calculators/{name}`.
- [x] **API-002 OpenAPI / Swagger spec** - `GET /openapi.json` returns an OpenAPI 3.1.0 document auto-generated from the registry at request time; per-calculator POST paths included so Swagger UI shows the correct input schema for each calculator.
- ~~**API-003 Auth / rate-limiting**~~ - out of scope for the crate. Use a reverse proxy (e.g. Caddy) for auth, TLS, and rate-limiting in deployments that need it.

---

## Python FFI

A `clincalc-py` crate using `pyo3` to expose the engine to Python data-science workflows. Kept as a separate crate so the core `clincalc` crate remains leaf-clean.

### Future

- [ ] **PY-001 `clincalc-py` crate** - `pyo3`-based Python bindings. `import clincalc; clincalc.calculate("egfr", {...})` returning a dict matching `CalculationResponse`.
- [ ] **PY-002 PyPI publish** - `maturin` build + publish to PyPI alongside the Rust crates.io release.
- [ ] **PY-003 Pandas-friendly helpers** - `clincalc.batch("egfr", df)` applying a calculator to every row of a DataFrame.

---

## Engine & embedding

### Future

- [ ] **ENG-001 Multilingual support** - implement `Locale` enum + `LocalizedString` per [`spec/multilingual.md`](spec/multilingual.md). Validate with one calculator (FeverPAIN) and a native speaker before opening the catalogue for batched translation.
- [ ] **ENG-002 Translation reciprocity with [MedikQuantis](https://medikquantis.me)** - their Catalan/Spanish strings for the 14 overlapping calculators are exactly what we need; agree a shared tag taxonomy and citation shape so either project can ingest the other's metadata.
- [ ] **ENG-003 `clincalc-web`** - single-file HTML calculators returning, ideally with `clincalc` compiled to WebAssembly so the browser surface shares the engine.
- [ ] **ENG-004 FHIR Observation export** - standardised exchange of results.
- [ ] **ENG-005 Unit conversion** - metric / imperial at the input boundary; today `--input` is units-explicit per field.
- [ ] **ENG-006 Printable / clipboard-friendly result formatting** - beyond the existing text block, possibly rich Markdown with citation links, PDF, or RTF.
- [ ] **ENG-007 Plugin system** - user-defined / third-party calculators loaded at runtime.
- [ ] **ENG-008 Guideline-update registry** - a mechanism to re-verify each calculator's licence and reference URL on a schedule.
- [ ] **ENG-009 High-risk-score alerts** - events embedding hosts can subscribe to, for example NEWS2 >= 7.

---

## Calculator backlog

See [`spec/calculator-roadmap.md`](spec/calculator-roadmap.md).

At time of writing: 64 active + 10 proprietary stubs shipped; 36 Future candidates queued (chiefly from MedikQuantis, plus the recently-added [StatinMD](https://www.thelancet.com/journals/landig/article/PIIS2589-7500\(26\)00047-6/fulltext)).
