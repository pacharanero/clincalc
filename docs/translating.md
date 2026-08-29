# Contributing a translation

`clincalc` supports reviewed, complete translation bundles per calculator - never partial or machine-translated strings mixed into an otherwise-English response. This page is the workflow for adding or updating one, per [`spec/multilingual.md`](https://github.com/pacharanero/clincalc/blob/main/spec/multilingual.md), the design document that governs locale support across every surface.

If you are looking for the architecture and standards behind translation (BCP 47 locale identifiers, message design, the engine API), read `spec/multilingual.md` first. This page is the contributor-facing "how do I actually submit one" companion to it.

## Current status

The dependency-free locale foundation and CLI locale selection are implemented, but no calculator advertises a reviewed non-English locale yet. Generic complete-bundle enforcement is still tracked as ENG-001.4 in [`spec/roadmap.md`](https://github.com/pacharanero/clincalc/blob/main/spec/roadmap.md). Until that registry-wide gate lands, the first translation pull request must add calculator-specific completeness tests and must not add a locale to `supported_locales()` until every layer and review requirement below is satisfied.

## What gets translated

Three layers, together, per calculator:

1. **Metadata** - title and description.
2. **Schema prose** - property descriptions and the human-readable fields inside governed input definitions (`concept`, `statement`, `includes`, `excludes`, `caveats`).
3. **Computed prose** - interpretation text, recommendation labels, summaries, and other human-readable display values.

Machine identifiers never change: calculator names, schema property names, enum values, tags, message IDs, and `working` keys stay English ASCII in every locale. Source citations, URLs, SNOMED ECL codes, and other reference data also remain byte-for-byte unchanged. If you find yourself translating one of these, stop - it is not meant to move.

## Before you start

- **Check for an existing source.** [MedikQuantis](https://medikquantis.me) (MIT licensed, maintained by Laura Piñero Roig) already ships Catalan, Spanish, and English strings for several overlapping calculators. Reusing that source text with attribution is preferred over translating from scratch - see [Attribution](#attribution) below. Cross-check its scoring semantics against `clincalc`'s own primary-source implementation before adapting any wording; the two projects can agree on terminology while keeping independently verified scoring logic.
- **Plan translations alongside new MedikQuantis parity calculators when reviewers are available.** Prefer including all three locales in the implementation pull request, but missing translations do not block completion of an independently verified calculation. Language availability is tracked separately in the README status table, and `ca`/`es` remain unadvertised until reviewed.
- **Pick a calculator with stable clinical prose.** Review its implementation under `src/calculators/`, any calculator-specific document under `spec/calculators/`, and recent roadmap work. Most calculators do not yet have a separate calculator spec, so the absence of one is not evidence that the wording is stable.
- **Keep scoring and translation review coherent.** A single pull request is preferred when both reviewer groups are available, but a translation follow-up may proceed independently after the calculation ships. Keep the translation linked to the calculator and update its README language status only after all review gates pass.

## Workflow

1. **Implement the locale-specific companion methods and renderers** in the calculator's existing Rust module, following the message-ID and named-argument conventions in `spec/multilingual.md` ("Messages and formatting" and "Structured interpretation"). Stable message IDs (`curb65.interpretation.high`), never English source text, are the keys. A "bundle" currently means the complete set of locale-specific metadata, schema prose, and rendering functions - there is no generic bundle file or parallel translation-only directory.
2. **Translate all three layers** (metadata, schema prose, computed prose) for the locale. A calculator cannot advertise a locale with any layer incomplete - key parity must be demonstrated by tests even before the generic registry gate lands.
3. **Leave numeric and machine fields untouched.** Scores, thresholds, risk-band codes, recommendation codes, and citation text are identical across locales by construction; only display prose changes.
4. **Add calculator-specific completeness and conformance tests** for the new locale. Until ENG-001.4 supplies the generic registry gate, the test must enumerate every translated metadata field, governed schema-prose field, message ID, and rendered risk or recommendation band. Also cover exact lookup, regional fallback (for example, `es-MX` resolving to `es`), whole-response English fallback in the engine, and the CLI's unsupported-locale error path.
5. **Record provenance beside the implementation and in the PR description**: whether the wording is original or adapted, the upstream repository and file path when reused, the exact upstream commit or release, the source-English `clincalc` commit, translator name, reviewer name, review date, and applicable licence.
6. **Open the pull request** against the calculator's existing files. Do not introduce a parallel translation-only file layout - translations live in-tree beside the clinical context, source attribution, and tests they translate, per `spec/multilingual.md`.

## Attribution

- Every translation bundle records its source, translator, reviewer, review date, licence, upstream revision, and source-English `clincalc` revision - in a comment beside the locale-specific implementation until a standard metadata structure exists, and in the PR description.
- Content reused from MedikQuantis retains attribution to Laura Piñero Roig and the MIT notice, per the collaboration terms in `spec/multilingual.md`. Reusing source text does not remove the requirement for clincalc's own review of the final adapted wording.
- If you are translating from scratch, credit the translator and reviewers in both locations so the evidence remains discoverable without reconstructing pull-request history.

## Native-speaker and clinical review

A translation is not merged on linguistic accuracy alone.

- **Native-speaker review** confirms the translated text reads naturally and uses locally-appropriate clinical terminology.
- **Clinical review** confirms the translation has not silently changed a recommendation or a safety meaning - for example, softening or strengthening a threshold, or losing a caveat present in the English source. This matters most for computed prose (interpretations, recommendations) and governed schema fields (`caveats`, `excludes`).
- Both reviews are recorded (reviewer name, date) alongside the translation, not just implied by an approving PR review. A calculator does not advertise a locale until this review is on record - see ENG-001.5 in `spec/roadmap.md` for the CURB-65 precedent being established with MedikQuantis.
- Wording changes to an *existing* translation that could plausibly change a recommendation or safety meaning need the same review before merge, even for a one-line fix.

## Key parity

Locale-key parity is the required invariant: every message ID and schema-prose key present in the English representation must be present in every advertised locale, and vice versa. Generic registry enforcement is ENG-001.4 and is not implemented yet. Until it lands, each translated calculator must carry an explicit completeness test, and reviewers must compare that test with all three translatable layers rather than assuming the registry already guarantees parity.

If your PR adds a new English message ID to a calculator that already has translations, that PR must also either:

- add the corresponding key to every existing translation bundle for that calculator, or
- explicitly document in the PR why the locale is being removed from the calculator's advertised `supported_locales()` until the new key is translated and reviewed.

Never leave a calculator advertising a locale with a missing or English-placeholder key. When a locale is not advertised, the CLI rejects an explicit request for it; a direct engine call falls back to the complete English representation and reports `working.content_locale` as `en`. Advertising an incomplete locale would bypass that protection, which is why `supported_locales()` changes and completeness tests must be reviewed together.

## Stale-source workflow

Source text drifts: a calculator's English wording, thresholds, or caveats change after a translation already exists.

1. **Any PR that changes translated English source content** (title, description, schema prose, computed prose for a calculator with existing bundles) must flag every locale that bundle affects, in the PR description.
2. **Completeness tests catch missing keys**, but not *stale but still-present* keys - a translated string that still validates as present but no longer matches the current English meaning. Reviewers should treat this as part of normal review for any PR touching translated content: does an existing translation still say what the new English says?
3. **When in doubt, do not merge a silently stale translation.** Either update the translation in the same PR (with its own re-review per [Native-speaker and clinical review](#native-speaker-and-clinical-review) if the meaning changed), or open a tracking issue and remove that locale from the calculator's advertised `supported_locales()` until it is updated. The locale-specific code can remain in-tree while unadvertised, but every surface must resolve to or require English consistently. A stale but plausible-sounding translation is worse than an explicit English response.
4. **Source-text revision is recorded per translation** (see [Attribution](#attribution)) specifically so staleness can be detected by comparing revisions, not by re-reading every string on every change.

## Batched translation

This workflow is written for the current stage: proving the pattern on a handful of calculators (CURB-65 first, per `spec/multilingual.md`'s rollout). Opening translation up to batches of calculators at once is deferred until:

- complete-bundle enforcement (locale-key parity as a blocking registry test) is in place across the registry, and
- at least two more calculators of different shapes have gone through this workflow with native-speaker and clinical review recorded.

Track progress on both in [`spec/roadmap.md`](https://github.com/pacharanero/clincalc/blob/main/spec/roadmap.md) under ENG-001.
