# Roadmap

Engineering, infrastructure, and product-level work for the `clincalc` project. **This file is the home for everything that is not a new calculator.** The clinical-calculator backlog lives in [`spec/calculator-roadmap.md`](spec/calculator-roadmap.md).

## Status legend

- `[~]` **In-progress** - actively being worked on or dormant pending a one-off setup.
- `[ ]` **Planned** - committed to build; the next batch.
- `[ ]` **Future** - under consideration; promote to Planned when scheduled.
- `[x]` **Completed** - (these items are periodically removed from the file).

---

## Distribution & release

### In-progress

- [~] **Activate releases** - the pipeline is built and validated (`dist plan` green, workflows lint-clean) but dormant until two repo secrets are set:
    - `CARGO_REGISTRY_TOKEN` - crates.io API token with publish rights on `clincalc`.
    - `HOMEBREW_TAP_TOKEN` - PAT with write access to `pacharanero/homebrew-tap`.

    Once set, `s/version++ [patch|minor|major]` cuts a release: it bumps the workspace (and excluded GUI) version, regenerates `CHANGELOG.md` (git-cliff), and lands `chore(release): vX.Y.Z` on `main`. `auto-tag.yml` then tags `vX.Y.Z` and invokes `release.yml` and `publish-crates.yml`. After the first publish, `cargo install clincalc` works without `--git`, and downstream consumers (notably [GitEHR](https://github.com/gitehr/gitehr)) can depend on `clincalc` (with `default-features = false` for the pure engine) from crates.io.

    The docs-site `install.sh` / `install.ps1` proxy scripts are wired into the Pages deploy and will become usable as soon as the first cargo-dist release assets exist.

### Planned

- [ ] **Windows code-signing** - EV cert from Sectigo / SSL.com once procured. The cert covers `sct`, `dsc`, **and** `clincalc` in one purchase; see [`spec/gui.md`](spec/gui.md#windows-code-signing). Until then the GUI installer triggers SmartScreen on first run.

### Future

- [ ] **`cargo binstall`** - cargo-dist releases are natively binstall-discoverable; add an explicit `[package.metadata.binstall]` override only if a case needs it.
- [ ] **deb / rpm / Scoop** packaging - only if user demand surfaces.

---

## Desktop GUI

See the design spec at [`spec/gui.md`](spec/gui.md) and the implementation guide at [`gui/README.md`](gui/README.md).

### Planned

- [ ] **CHA₂DS₂-VASc UI** - first non-trivial calculator (enums, age band); templates the next class of widget.
- [ ] **QRISK3 UI** - the politically-motivated one (still missing from EMIS and SystmOne); 22 mixed-type inputs.
- [ ] **Decide CI build cadence for the GUI** - on every push (slow, ~5 min) vs on release tag only. Probably the latter once cargo-dist lands.

### Future

- [ ] **Updater** - Tauri's built-in updater speaking to a manifest hosted on the docs site.
- [ ] **iOS / Android builds** - Tauri 2 supports them and `clincalc` is pure Rust, so this is mostly a packaging question.
- [ ] **Theme parity with GitEHR** - shared CSS variables or a tiny `@clincalc/ui-tokens` package, so the two apps stay visually coherent without copy-paste drift.

---

## Authoring workflow & docs

### Planned

- [ ] **Retire `.claude/skills/build-calculator/`** in favour of `spec/` + `examples/` + `AGENTS.md` as the authoring entry point. Skill is Claude-specific; new authoring path should work in any agent.

The docs.rs front page now documents the `cli` / `mcp` feature flags, `default-features = false` leaf usage, and a minimal registry example. docs.rs is configured to build all features so the optional MCP module is visible.

### Future

- [ ] **`docs/translating.md`** - contribution path for translators once multilingual lands.
- [ ] **API reference for `clincalc`** linked from the docs site (docs.rs handles this automatically once published; just need a link from the Zensical nav).

---

## Engine & embedding

### Completed

- [x] **Reference MCP server** in this repo via `clincalc mcp`, gated behind the optional `mcp` Cargo feature. See [`spec/mcp.md`](spec/mcp.md) and [`docs/mcp.md`](../docs/mcp.md).

### Future

- [ ] **Multilingual support** - implement `Locale` enum + `LocalizedString` per [`spec/multilingual.md`](spec/multilingual.md). Validate with one calculator (FeverPAIN) and a native speaker before opening the catalogue for batched translation.
- [ ] **Translation reciprocity with [MedikQuantis](https://medikquantis.me)** - their Catalan/Spanish strings for the 14 overlapping calculators are exactly what we need; agree a shared tag taxonomy and citation shape so either project can ingest the other's metadata.
- [ ] **`clincalc-web`** (single-file HTML calculators) returning, ideally with `clincalc` compiled to WebAssembly so the browser surface shares the engine.
- [ ] **FHIR Observation export** for standardised exchange of results.
- [ ] **Unit conversion** (metric ↔ imperial) at the input boundary; today `--input` is units-explicit per field.
- [ ] **Printable / clipboard-friendly result formatting** beyond the existing text block (rich Markdown with citation links? PDF? RTF?).
- [ ] **Plugin system** for user-defined / third-party calculators loaded at runtime.
- [ ] **Guideline-update registry** - a mechanism to re-verify each calculator's licence and reference URL on a schedule.
- [ ] **High-risk-score alerts** as events embedding hosts can subscribe to (e.g. NEWS2 ≥ 7).

---

## Calculator backlog

See [`spec/calculator-roadmap.md`](spec/calculator-roadmap.md).

At time of writing: 42 active + 10 proprietary stubs shipped; 36+ Future candidates queued (chiefly from MedikQuantis, plus the recently-added [StatinMD](https://www.thelancet.com/journals/landig/article/PIIS2589-7500\(26\)00047-6/fulltext)).
