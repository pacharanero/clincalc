<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Multilingual readiness

## Why this is in scope

`clincalc` today emits English-only strings: titles, descriptions, interpretations, references, and the human-readable hints inside the working map. Other open calculator projects (notably [MedikQuantis](https://medikquantis.me) - Catalan, Spanish, English by native speakers) demonstrate that a credible clinical-calculator project in 2026 is multilingual from day one.

This document is design-only. No code in this repo speaks any language but English today. The goal is to **freeze a design that does not paint us into a corner**, so we can adopt languages incrementally without breaking the engine or any host.

## Constraints (non-negotiable)

- **The leaf rule still holds.** No new always-on dependencies in `clincalc` (the engine stays `serde` + `serde_json`). No I/O, no global state, no clock.
- **Machine-stable identifiers stay English.** Calculator `name()` (the CLI subcommand and MCP tool name), the keys inside the `working` map, the keys in `input_schema()`, and every enum variant are stable English slugs. They are part of the contract, not human prose.
- **The result schema does not gain a `lang` field.** A response is a response; the locale used to produce it is the responsibility of the caller / host.
- **Translations are versioned with the source.** A translation that drifts behind the English source is worse than no translation, so every translatable string carries a stable key and translations live in-tree with their literature citation context.

## What is translatable

Three layers, treated separately:

1. **Calculator metadata** - `title()`, `description()`. Short, stable. Easy.
2. **Computed prose** - the `interpretation` field of `CalculationResponse`, and any human-readable strings that the calculator emits into `working` (banding labels, recommendations). Variable, depends on inputs. Hardest.
3. **Schema descriptions** - the `description` on each input property in `input_schema()`. Drives the CLI placeholder hints. Mid-effort.

The `reference` field (primary citation) is **not** translated - a literature citation is the same in every language.

## Design

### Locale type

```rust
pub enum Locale {
    En,    // default
    Es,    // Spanish
    Ca,    // Catalan
    // ...further locales added on demand
}
```

`En` is always the source language and the fallback. Any string missing in another locale falls back to `En` rather than failing. Locales are an enum (not a free-form string) so the compiler enumerates them and missing-locale handling is a `match` exhaustiveness problem.

### Storage

Every translatable string is keyed and looked up at compile time. The natural shape:

```rust
pub struct LocalizedString {
    pub en: &'static str,
    pub es: Option<&'static str>,
    pub ca: Option<&'static str>,
}

impl LocalizedString {
    pub fn get(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::En => self.en,
            Locale::Es => self.es.unwrap_or(self.en),
            Locale::Ca => self.ca.unwrap_or(self.en),
        }
    }
}
```

The strings live in the calculator's own module, alongside the literature they were translated from. They are `&'static str`, so the binary stays self-contained and there is no runtime translation file to ship.

### Trait shape

The current `Calculator` methods (`title`, `description`, `interpretation` strings produced inside `calculate`) all return `&'static str`. The minimal change adds a locale to the methods that produce human prose:

```rust
pub trait Calculator {
    fn name(&self) -> &'static str;                          // unchanged - stable identifier
    fn title(&self, locale: Locale) -> &'static str;         // localised
    fn description(&self, locale: Locale) -> &'static str;   // localised
    fn reference(&self) -> &'static str;                     // unchanged
    fn license(&self) -> CalculatorLicense;                  // unchanged
    fn input_schema(&self, locale: Locale) -> Value;         // schema strings localised
    fn calculate(&self, input: &Value, locale: Locale)
        -> Result<CalculationResponse, CalcError>;
}
```

A default `Locale::En` keeps every existing call site working while host code is migrated.

### Computed prose (the hard bit)

A calculator's `interpretation` string typically interpolates the computed values: `"NEWS2 7 (high). Emergency response: ..."`. Today these are built with `format!` inline. The two paths:

1. **Format strings per locale.** Each interpretation has a `LocalizedString` template with positional arguments. Cheap but fragile - argument order has to match across translations.
2. **Structured interpretation.** `interpretation` becomes a struct keyed by stable IDs (band, score, recommendation) with localised labels and a small Mustache-style templater. Larger upfront cost; less drift over time.

Recommendation: **option 2** when translations land, with option 1 acceptable as an interim. The `working` map already gives the structured surface a host can render in its own way; adding band labels there is a small step.

### CLI surface

```bash
calc --lang es feverpain --input examples/feverpain.json
calc --lang ca list
```

`--lang` defaults to `en`. Invalid locales fail loudly at the CLI boundary. The locale is also accepted via `CALC_LANG=es` for environments where flags are awkward.

### MCP surface

The host passes a locale to `clincalc::cli::run`, set from the LLM's session preference or from a tool-call argument. No engine change beyond the new trait shape.

## What we are NOT designing today

- **Right-to-left scripts** (Arabic, Hebrew). Possible later but adds layout concerns no current consumer needs.
- **Pluralisation rules** (`one apple` / `2 apples`). Mostly handled by emitting numbers separately from labels in `working`.
- **Inflected agreement** (Spanish gender, Catalan number). Where this matters, the structured-interpretation path supports it via stable label IDs whose translations choose the right form. Format-string templates do not.
- **Locale-specific units.** Out of scope. `--input` is units-explicit (`creatinine_unit: "mg/dL"|"umol/L"`); the locale never decides the unit.

## Rollout plan

When this is picked up:

1. Add `Locale` to `clincalc` (zero-cost while only `En` exists).
2. Migrate `Calculator` trait to take `Locale` (default to `En` everywhere, no behaviour change).
3. Pick **one** calculator (FeverPAIN: small, NICE-cited) as the migration pattern.
4. Translate three calculators to validate the design with a native speaker.
5. Document the contribution path for translators in `docs/translating.md`.
6. Open the catalogue for translation in batches.

Until step 1 lands, contributors continue to write English-only calculators - the future locale parameter will gain a default, not a constraint.

## Collaboration with MedikQuantis

[MedikQuantis](https://medikquantis.me) (Laura Piró, Barcelona; MIT licensed; Catalan, Spanish, English) already ships a multilingual calculator suite. Possible directions:

- **Shared taxonomy** - agree on tag names so a calculator in one project is discoverable under the same specialty in the other.
- **Translation reciprocity** - their Catalan/Spanish strings for the calculators we both ship (DAS28, CHA2DS2-VASc, HEART, Wells PE, GRACE, TIMI, qSOFA, SOFA, CURB-65, Child-Pugh, IPSS, HAS-BLED, Wells DVT, CKD-EPI) are exactly the strings we would need.
- **Citation alignment** - their schema exposes PMID per score; ours exposes a citation string. A shared `references` shape would let either project ingest the other's metadata.

This file documents the design that makes that practical.
