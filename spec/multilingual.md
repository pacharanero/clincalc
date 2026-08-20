<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Multilingual architecture

## Why this is in scope

`clincalc` emits human prose in calculator titles, descriptions, input schemas, interpretations, summaries, validation errors, and some `working` values. A credible clinical-calculator project must make the language of that content explicit and support reviewed translations without changing the scoring engine or allowing surfaces to drift.

[MedikQuantis](https://medikquantis.me) demonstrates this with clinician-reviewed Catalan, Spanish, and English content. Its overlapping calculators give `clincalc` a practical source of reviewed translations and a collaborator with whom terminology can be aligned.

## Constraints

- **The leaf rule still holds.** With `default-features = false`, the engine depends only on `serde` and `serde_json`. It does not acquire an internationalisation runtime, locale database, I/O, global state, or a clock.
- **Scoring remains locale-neutral.** Locale never changes thresholds, clinical precision, numeric JSON values, input interpretation, or units.
- **Machine identifiers remain stable.** Calculator names, schema property names, enum values, tags, message IDs, and `working` keys are English ASCII slugs and are never translated.
- **References remain unchanged.** A primary literature citation is the same in every language.
- **Translations are versioned with the calculator.** They live in-tree beside their clinical context, source attribution, and tests.
- **No mixed-language clinical response.** A calculator advertises a locale only when its metadata, schema prose, and computed prose form a complete reviewed bundle. Otherwise the complete representation falls back to English.
- **No process-global locale.** Locale is explicit per operation, with immutable defaults available at surface boundaries for convenience.

## Standards

### Locale identifiers

Public surfaces use [BCP 47](https://www.rfc-editor.org/info/bcp47/) language tags, defined by RFC 5646 and RFC 4647. Examples include `en`, `en-GB`, `es`, `es-MX`, `ca`, and `sr-Latn`.

- Tags use hyphens, not underscores.
- Comparisons are ASCII case-insensitive.
- Emitted tags use conventional casing: lower-case language, title-case script, upper-case region.
- RFC 4647 lookup selects one supported translation bundle by progressively truncating a requested tag. For example, `es-MX` can resolve to `es` when no Mexican Spanish bundle exists.
- The leaf crate does not claim full BCP 47 validity or Unicode CLDR canonicalisation. Those operations require versioned IANA or CLDR data and belong in a surface or optional internationalisation layer.

The implementation distinguishes two concepts:

1. **Requested locale** - a BCP 47 string supplied by a caller or protocol, such as `es-MX`.
2. **Supported locale** - the canonical tag of a complete bundle compiled into a calculator, such as `es`.

`SupportedLocale` is a `#[non_exhaustive]` Rust enum because the compiled set is finite and benefits from exhaustive handling inside the crate. It is not used as the public wire syntax: its serialised form is the canonical BCP 47 tag.

### HTTP language negotiation

The REST surface follows RFC 9110:

- Explicit `?locale=<tag>` takes precedence.
- `Accept-Language` is used when no explicit locale is supplied.
- A configured server default is used when the request expresses no preference.
- English is the final application fallback.
- Localised responses return `Content-Language` with the locale actually used.
- Responses negotiated through `Accept-Language` return `Vary: Accept-Language` when cacheable.
- An explicitly requested unsupported locale fails clearly rather than silently changing language.

The OpenAPI document remains canonical English. Localised calculator schemas are obtained from the calculator schema endpoint rather than generating a different OpenAPI contract for every locale.

### Messages and formatting

Message design follows the concepts standardised by [Unicode MessageFormat](https://www.unicode.org/reports/tr35/tr35-messageFormat.html):

- Stable semantic message IDs, not English source text as keys.
- Named variables, never positional arguments.
- Numbers remain numbers until human presentation.
- Complete sentences are translated; translated fragments are not concatenated.
- Message variables and their types are validated consistently across locale bundles.
- Right-to-left text, plural selection, grammatical gender, and locale-specific number formatting are not required in the first translation, but the data model must not preclude them.

The leaf crate does not implement a general template parser. Initial calculators use explicit locale-specific Rust rendering functions with compile-time `format!` strings and named captures. This lets real translations establish the required message model without creating a proprietary Mustache-like language. A mature MessageFormat/CLDR renderer can later live behind an optional feature or in a companion layer without changing stable message IDs and arguments.

### Numbers and units

- JSON inputs and structured results use locale-independent numbers, never strings such as `"1,23"`.
- Clinical rounding is specified by the calculator, not by the locale.
- Locale can eventually change decimal separators, grouping, digit shapes, spacing, and display labels only at the human-rendering boundary.
- Units remain explicit and are never inferred from language or region.
- If machine-interoperable unit codes are introduced, prefer UCUM; CLDR display names are presentation, not unit semantics.

## Translatable content

Three calculator layers are translated together before a locale is advertised:

1. **Metadata** - title and description.
2. **Schema prose** - property descriptions and the human-readable fields inside governed input definitions (`concept`, `statement`, `includes`, `excludes`, and `caveats`). Property names, enum values, source citations, URLs, SNOMED ECL, and status values remain stable.
3. **Computed prose** - interpretation, recommendation labels, summaries, and any human-readable display values.

CLI chrome, REST errors, MCP server instructions, Python error messages, and GUI controls are surface translations. They use the same resolved locale but are not part of a calculator's clinical translation bundle.

## Engine API

Rust has no default function parameters. Adding a locale parameter to every existing trait method would break every calculator, downstream implementation, and caller. Locale support therefore enters through additive companion methods while the existing methods remain English compatibility APIs:

```rust
pub trait Calculator {
    fn name(&self) -> &'static str;

    fn title(&self) -> &'static str;
    fn title_for(&self, locale: SupportedLocale) -> &'static str;

    fn description(&self) -> &'static str;
    fn description_for(&self, locale: SupportedLocale) -> &'static str;

    fn input_schema(&self) -> Value;
    fn input_schema_for(&self, locale: SupportedLocale) -> Value;

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError>;
    fn calculate_for(
        &self,
        input: &Value,
        locale: SupportedLocale,
    ) -> Result<CalculationResponse, CalcError>;

    fn supported_locales(&self) -> &'static [SupportedLocale];
}
```

Default companion methods delegate to the existing English methods. A calculator overrides them only after adding a complete reviewed bundle. The typed `compute()` API remains locale-neutral.

The English compatibility method `calculate()` retains its existing response contract. Every `calculate_for()` response reports the canonical bundle actually used in `working.content_locale`; an English-only calculator therefore reports `en` even when a direct engine caller requests another compiled locale. Surface boundaries must still validate explicit requests according to their protocol: the CLI rejects a locale the selected calculator does not advertise, while future HTTP localisation will also return `Content-Language`. Silent fallback without reporting the resolved locale is not acceptable because persisted clinical prose needs language provenance.

## Structured interpretation

Several current calculators mix display prose into typed outcomes, string-valued results, or the `working` map. Translation must not change values that consumers may treat as machine codes.

Each migrated calculator separates:

- Stable clinical facts: score, thresholds met, risk-band code, recommendation code, rates, and quantities.
- Stable message identity: for example `curb65.interpretation.high`.
- Named message arguments: for example `score`, `mortality_percent`, and `icu_assessment`.
- Localised display prose rendered from those facts.

Existing English fields remain compatible during incremental migration. New stable code fields are added before any existing display value is localised or retired.

## Surface policy

### CLI

```bash
clincalc --locale es curb65 --input examples/curb65.json
clincalc --locale ca list
```

Precedence is `--locale`, then `CLINCALC_LOCALE`, then `en`. Invalid or explicitly unsupported locale requests fail with the supported locales. The CLI resolves the requested BCP 47 tag before calling the engine.

### Python

Locale is an optional keyword-only argument on every prose-producing operation:

```python
clincalc.calculate("curb65", inputs, locale="es")
clincalc.list_calculators(locale="ca")
clincalc.get_schema("curb65", locale="es")
clincalc.batch("curb65", frame, locale="ca")
```

There is no module-global `set_locale()`. A future immutable client object may hold a default locale for repeated calls while still allowing per-call overrides.

### REST API

REST uses the explicit-query/header/configured-default precedence described above. Locale is resolved independently for each request, making concurrent multi-user use deterministic and safe.

### MCP

Locale is configured for the MCP server/session and can be overridden by host context. It is not inserted into each calculator's input schema, because locale is representation metadata rather than a clinical input and calculator schemas reject additional properties.

### GUI and web

The host resolves one locale for calculator content and UI chrome. Stable result and recommendation codes drive logic and styling; translated labels are never used as control-flow values. HTML declares the resolved BCP 47 language and sets text direction when right-to-left locales are introduced.

## Rollout

1. Add the dependency-free BCP 47 locale foundation: `SupportedLocale`, canonical tags, RFC 4647 lookup, and English fallback.
2. Add locale-aware `Calculator` companion methods with English defaults, preserving all existing APIs and results.
3. Define stable message IDs, named argument conventions, and translation completeness tests from a real calculator.
4. Migrate **CURB-65** first. It is compact but exercises metadata, governed schema descriptions, units, risk bands, variable interpretation text, and ICU guidance. MedikQuantis already has clinician-reviewed English, Spanish, and Catalan CURB-65 content with CI-enforced key parity under the MIT licence.
5. Attribute imported MedikQuantis translation content to Laura Piñero Roig and retain the MIT notice as required. Review terminology differences against `clincalc`'s primary-source implementation rather than copying scoring logic.
6. Add two more overlapping calculators of different shapes and have native speakers review all three bundles.
7. Propagate locale through CLI, Python, REST, MCP, and GUI once the engine pattern is proven.
8. Write `docs/translating.md` with the contribution, attribution, review, and stale-translation workflow before opening translation batches.

## Translation quality gates

- Locale-key parity is a blocking registry test.
- A calculator cannot advertise a locale with any missing clinical message or schema description.
- Every translation records its source, translator/reviewer attribution, and source-text revision.
- Numeric results and stable machine fields are identical across locales.
- Tests cover exact locale lookup, regional fallback, unsupported requests, whole-response English fallback, and every translated risk band.
- Translation changes receive clinical review where wording could change a recommendation or safety meaning.

## Collaboration with MedikQuantis

[MedikQuantis](https://medikquantis.me) is an MIT-licensed multilingual calculator suite maintained by Laura Piñero Roig. It has Catalan, Spanish, and English message catalogues with a CI parity check, and many calculators overlap with `clincalc`.

The collaboration path is:

- Reuse reviewed translations with attribution while retaining `clincalc`'s independently verified scoring implementation and citations.
- Align stable clinical concepts, recommendation codes, tags, and citations where appropriate.
- Exchange test vectors and terminology review rather than reverse-engineering either scoring implementation.
- Invite upstream review whenever imported wording is adapted to `clincalc`'s schema or guideline context.

This design keeps one scoring engine authoritative while making translated clinical prose explicit, complete, attributable, and reproducible across every surface.
