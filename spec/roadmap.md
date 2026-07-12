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

## CLI & interaction

Making the terminal surface friendlier for a human at a prompt - the engine and MCP surfaces are strong; this is the ergonomics gap.

### Future

- [ ] **CLI-001 Common-name aliases** - resolve `clincalc tdee` / `clincalc bmr` to `energy_requirement` (and `egfr`, etc.); show aliases in `list`; fuzzy "did you mean ..." on unknown names. User feedback: `tdee` reads better than `energy_requirement`.
- [ ] **CLI-002 Dynamic result labelling for `energy_requirement`** - headline the result as **TDEE** when an `activity_factor` is supplied, **BMR/RMR** when not (keep both in the Working block). One calculator, correct name per mode.
- [ ] **CLI-003 Named activity presets** - `--activity sedentary|light|moderate|very-active|extra-active` mapping to the standard multipliers instead of raw `activity_factor` numbers; echo the factor used.
- [ ] **CLI-004 Human flag inputs + interactive mode** - accept `--sex male --age 48 ...` alongside `--input <json>` for common calculators; a guided `--interactive` walks and validates the schema (JSON stays the machine / MCP path).
- [ ] **CLI-005 Reusable subject profile** - pull shared demographics / analytes from `~/.config/clincalc/profile.json` or a GitEHR record (`--from-record <path>`), so recurring self- or same-patient calcs don't re-enter demographics. Pairs with ENG-004 (FHIR Observation export).
- [ ] **CLI-006 Goal-driven energy targets** - `--goal lose|maintain|gain --rate 0.5kg/week` derives the kcal adjustment (~7700 kcal/kg) and time-to-`--target-weight`, instead of a hand-computed `calorie_adjustment_kcal_day`.
- [ ] **CLI-007 Body-fat to lean-mass convenience** - accept `body_fat_pct` + `weight_kg` and derive LBM internally for Cunningham (and a future Katch-McArdle).

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

At time of writing: 43 active + 10 proprietary stubs shipped; 46 Future candidates queued (chiefly from MedikQuantis, plus the recently-added [StatinMD](https://www.thelancet.com/journals/landig/article/PIIS2589-7500\(26\)00047-6/fulltext)).
