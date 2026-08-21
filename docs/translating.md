# Contributing a translation

`clincalc` supports reviewed, complete translation bundles per calculator - never partial or machine-translated strings mixed into an otherwise-English response. This page is the workflow for adding or updating one, per [`spec/multilingual.md`](https://github.com/pacharanero/clincalc/blob/main/spec/multilingual.md), the design document that governs locale support across every surface.

If you are looking for the architecture and standards behind translation (BCP 47 locale identifiers, message design, the engine API), read `spec/multilingual.md` first. This page is the contributor-facing "how do I actually submit one" companion to it.

## What gets translated

Three layers, together, per calculator:

1. **Metadata** - title and description.
2. **Schema prose** - property descriptions and the human-readable fields inside governed input definitions (`concept`, `statement`, `includes`, `excludes`, `caveats`).
3. **Computed prose** - interpretation text, recommendation labels, summaries, and other human-readable display values.

Machine identifiers never change: calculator names, schema property names, enum values, tags, message IDs, `working` keys, source citations, URLs, and SNOMED ECL codes stay English ASCII in every locale. If you find yourself translating one of these, stop - it is not meant to move.

## Before you start

- **Check for an existing source.** [MedikQuantis](https://medikquantis.me) (MIT licensed, maintained by Laura Piñero Roig) already ships Catalan, Spanish, and English strings for several overlapping calculators. Reusing that source text with attribution is preferred over translating from scratch - see [Attribution](#attribution) below. Cross-check its scoring semantics against `clincalc`'s own primary-source implementation before adapting any wording; the two projects can agree on terminology while keeping independently verified scoring logic.
- **Pick a calculator with a stable spec.** A calculator whose `spec/calculators/<name>.md` is still actively changing is a poor translation target - you will end up re-translating.
- **One calculator, one pull request.** Keeps review scoped and makes attribution unambiguous.

## Workflow

1. **Add the locale's message bundle** beside the calculator's existing Rust source, following the message-ID and named-argument conventions in `spec/multilingual.md` ("Messages and formatting" and "Structured interpretation"). Stable message IDs (`curb65.interpretation.high`), never English source text, are the keys.
2. **Translate all three layers** (metadata, schema prose, computed prose) for the locale. A calculator cannot advertise a locale with any layer incomplete - key parity across locales is a blocking registry test, not a style preference.
3. **Leave numeric and machine fields untouched.** Scores, thresholds, risk-band codes, recommendation codes, and citation text are identical across locales by construction; only display prose changes.
4. **Add or extend conformance tests** for the new locale: exact lookup, regional fallback (e.g. `es-MX` resolving to `es`), and the unsupported-locale error path if this is the calculator's first non-English bundle.
5. **Record provenance in the PR description**: source (original translation or independent), translator/reviewer name, and the source-text revision the translation was made against.
6. **Open the pull request** against the calculator's existing files. Do not introduce a parallel translation-only file layout - translations live in-tree beside the clinical context, source attribution, and tests they translate, per `spec/multilingual.md`.

## Attribution

- Every translation bundle records its source, its translator/reviewer, and the source-text revision it was translated against - in the code (as a comment or metadata field near the bundle) and in the PR description.
- Content reused from MedikQuantis retains attribution to Laura Piñero Roig and the MIT notice, per the collaboration terms in `spec/multilingual.md`. Reusing source text does not remove the requirement for clincalc's own review of the final adapted wording.
- If you are translating from scratch, credit yourself (or whoever did the work) in the PR - that attribution is preserved in Git history and, where relevant, in `CHANGELOG.md`.

## Native-speaker and clinical review

A translation is not merged on linguistic accuracy alone.

- **Native-speaker review** confirms the translated text reads naturally and uses locally-appropriate clinical terminology.
- **Clinical review** confirms the translation has not silently changed a recommendation or a safety meaning - for example, softening or strengthening a threshold, or losing a caveat present in the English source. This matters most for computed prose (interpretations, recommendations) and governed schema fields (`caveats`, `excludes`).
- Both reviews are recorded (reviewer name, date) alongside the translation, not just implied by an approving PR review. A calculator does not advertise a locale until this review is on record - see COLL-001 in `spec/roadmap.md` for the CURB-65 precedent being established with MedikQuantis.
- Wording changes to an *existing* translation that could plausibly change a recommendation or safety meaning need the same review before merge, even for a one-line fix.

## Key parity

Locale-key parity is enforced by a registry test, not manual checking: every message ID and schema-prose key present in the English bundle must be present in every advertised locale, and vice versa. If your PR adds a new English message ID to a calculator that already has translations, that PR must also either:

- add the corresponding key to every existing translation bundle for that calculator, or
- explicitly document in the PR why the calculator's advertised locale set is being reduced (a bundle dropped back to English fallback until the new key is translated).

Never leave a calculator advertising a locale with a missing key - the engine falls back to the complete English representation for the whole response rather than mixing languages, so an incomplete bundle simply loses locale support silently at review time and confusingly at run time.

## Stale-source workflow

Source text drifts: a calculator's English wording, thresholds, or caveats change after a translation already exists.

1. **Any PR that changes translated English source content** (title, description, schema prose, computed prose for a calculator with existing bundles) must flag every locale that bundle affects, in the PR description.
2. **The registry test catches missing keys**, but not *stale but still-present* keys - a translated string that still validates as present but no longer matches the current English meaning. Reviewers should treat this as part of normal review for any PR touching translated content: does an existing translation still say what the new English says?
3. **When in doubt, do not merge a silently stale translation.** Either update the translation in the same PR (with its own re-review per [Native-speaker and clinical review](#native-speaker-and-clinical-review) if the meaning changed), or open a tracking issue and revert that locale's bundle to explicit English fallback until it is updated. A stale but plausible-sounding translation is worse than a clearly-marked English fallback.
4. **Source-text revision is recorded per translation** (see [Attribution](#attribution)) specifically so staleness can be detected by comparing revisions, not by re-reading every string on every change.

## Batched translation

This workflow is written for the current stage: proving the pattern on a handful of calculators (CURB-65 first, per `spec/multilingual.md`'s rollout). Opening translation up to batches of calculators at once is deferred until:

- complete-bundle enforcement (locale-key parity as a blocking registry test) is in place across the registry, and
- at least two more calculators of different shapes have gone through this workflow with native-speaker and clinical review recorded.

Track progress on both in [`spec/roadmap.md`](https://github.com/pacharanero/clincalc/blob/main/spec/roadmap.md) under ENG-001.
