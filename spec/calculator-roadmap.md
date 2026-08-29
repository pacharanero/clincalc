# Clinical Calculator Roadmap

The clinical-calculator backlog, grouped strictly by completion status. Calculator categorisation by specialty / setting / status lives in [tags](../docs/calculators.md#filtering-by-tag) on each calculator - this file is purely a list of what is being built and what is queued.

**Engineering, infrastructure, GUI, distribution, and any other non-calculator work lives in [`spec/roadmap.md`](roadmap.md), not this file.** Keeping the two split means a clinician scanning the calculator backlog is not buried in build-tooling items, and an engineer scanning the build-tooling roadmap is not buried in clinical scores.

Roadmap items have stable identifiers so they can be referred to in conversation, commits, PRs, and release notes. Do not renumber existing IDs just because items are completed or removed.

Completed calculators are removed from this file rather than kept as roadmap history. The shipped catalogue is [`docs/calculators.md`](../docs/calculators.md).

MedikQuantis parity items have a stricter completion rule: the calculation and its reviewed Catalan (`ca`) and Spanish (`es`) bundles are one adoption workstream. Prefer implementing all three locales in the calculator's initial pull request. If translation review is not available, the independently verified English calculation may ship first, but the roadmap item remains `[~]` until complete attributed `ca`/`es` bundles pass the review gates in [`multilingual.md`](multilingual.md) and [`docs/translating.md`](../docs/translating.md). Upstream MedikQuantis logic and tests are useful cross-project evidence, not a substitute for clincalc's primary-source verification.

## Status legend

- `[~]` **In-progress** - actively being implemented or under review.
- `[ ]` **Planned** - committed to build; the next batch.
- Items under [Future](#future) are explicitly **under consideration** rather than committed - they get promoted to **Planned** when scheduled.

---

## Calculators

### In-progress

- [~] **CALC-054 Apgar score** - English calculation shipped. Reviewed Catalan and Spanish bundles remain outstanding. Unlike the pinned MedikQuantis surface, clincalc records assessment timing and resuscitation context, limits descriptive bands to their supported context, and never turns the total into a resuscitation instruction.
- [~] **CALC-055 Mosteller body surface area** - English calculation shipped. Reviewed Catalan and Spanish bundles remain outstanding. Unlike the pinned MedikQuantis surface, clincalc reports the continuous BSA without inferring protocol-specific drug doses or indexing decisions.
- [~] **CALC-058 HOMA-IR** - English calculation shipped. Reviewed Catalan and Spanish bundles remain outstanding. Unlike the pinned MedikQuantis surface, clincalc does not apply universal interpretation thresholds because HOMA-IR varies by population and insulin assay.
- [~] **CALC-063 4Ts score for HIT** - English calculation shipped. Reviewed Catalan and Spanish bundles remain outstanding. Unlike the pinned MedikQuantis surface, clincalc derives points from required semantic categories, identifies the standard days 5-10 variant, and does not reduce current ASH guidance to generic band recommendations.
- [~] **CALC-064 Khorana score** - English calculation shipped. Reviewed Catalan and Spanish bundles remain outstanding. Unlike the pinned MedikQuantis surface, clincalc derives points from raw measurements and reports the original risk band separately from the modern score-2 thromboprophylaxis-assessment threshold.
- [~] **CALC-065 Binet staging** - English calculation shipped. Reviewed Catalan and Spanish bundles remain outstanding. Unlike the pinned MedikQuantis surface, clincalc derives the canonical stage from examination and blood-count findings rather than accepting a caller-selected stage and mapping it to an invented number.

### Planned

_Nothing currently committed to build. Promote from [Future](#future) when scheduled._

### Future

Calculators worth shipping, under consideration. Largely surfaced from sibling open-source projects (notably [MedikQuantis](https://medikquantis.me), MIT licensed). Clinical context for each lives in the [docs catalogue wishlist](../docs/calculators.md#wishlist-candidates-for-future-addition).

#### MedikQuantis parity gaps

This is the complete calculation gap against MedikQuantis's live 65-calculator registry at upstream commit [`16c63c85`](https://github.com/laurapiro17/medikquantis/tree/16c63c85aee7a64417205f60cb3e66fccf19fae2), reviewed 2026-08-29. clincalc already ships 42 equivalent calculations and lacks the 23 below. MedikQuantis's README still reports 49 calculators, so its registry is the authoritative inventory. Review this snapshot when upstream changes; add new stable `CALC-*` items rather than silently letting parity drift.

For every item below, completion means: independent implementation from the primary source; closed schema, licence evidence, and literature-vector tests under the normal calculator contract; explicit comparison with the pinned MedikQuantis behavior and documentation of intentional clinical or guideline differences; and complete attributed `en`/`ca`/`es` prose with recorded review. Machine identifiers and numeric behavior remain locale-neutral.

- [ ] **CALC-022 MELD 3.0** - Updated MELD
- [ ] **CALC-023 Modified Duke criteria** - Infective endocarditis
- [ ] **CALC-024 NIHSS** - Acute stroke severity
- [ ] **CALC-025 Norton Scale** - Pressure-ulcer risk (immobile)
- [ ] **CALC-026 NYHA** - Heart-failure functional class
- [ ] **CALC-027 ORBIT** - Bleeding risk in AF (DOAC era)
- [ ] **CALC-028 PASI** - Psoriasis Area and Severity Index
- [ ] **CALC-030 Pitt Bacteraemia** - BSI severity
- [ ] **CALC-031 PSA density** - PSA / prostate volume
- [ ] **CALC-034 SCORAD** - Atopic dermatitis severity
- [ ] **CALC-035 SCORE2 / SCORE2-OP** - ESC 2021 CV risk (verify licensing)
- [ ] **CALC-056 Combined BMI / BSA / ideal body weight** - Match MedikQuantis's combined anthropometric calculation while reusing shared clincalc primitives rather than duplicating BMI or BSA logic
- [~] **CALC-057 Free-water deficit** - English implementation shipped using an explicit clinician-selected total-body-water fraction rather than inferring body composition from sex or an arbitrary age boundary; reviewed Catalan and Spanish bundles remain outstanding
- [ ] **CALC-059 CIWA-Ar** - Alcohol-withdrawal severity; verify instrument reproduction terms and ensure scoring does not become an autonomous medication protocol
- [ ] **CALC-060 COWS** - Clinical Opiate Withdrawal Scale; verify instrument reproduction terms and preserve clinician-assessment context
- [ ] **CALC-061 SAD PERSONS** - Legacy suicide-risk checklist; require an evidence and safety review because poor predictive performance means it must not determine discharge, observation, or referral
- [~] **CALC-062 ISTH overt DIC score** - English implementation shipped using the 2025 ISTH update, measured inputs, D-dimer multiples of assay ULN, and an explicit DIC-associated etiology prerequisite; reviewed Catalan and Spanish bundles remain outstanding

#### Other candidates

Shipped anthropometric and body-composition measures include BMI, body-fat circumference, WHtR, WHR, RFM, 1RM, training zones, and the Wilks score. Remaining non-MedikQuantis candidates range from tape-measure proxies (BAI) to lab or field methods for body composition (skinfolds, FFMI, SMI) and the alternative DOTS strength score.

- [ ] **CALC-036 StatinMD** (Oxford STRATIFY) - personalised 1/5/10-year risk of serious statin-induced muscle disorders; natural pairing with QRISK3 (benefit vs harm). Academic licence via Oxford University Innovation (Cai et al, *Lancet Digital Health* 2026; [licence page](https://process.innovation.ox.ac.uk/software/p/25396/stratify---stainmd-risk-calculator---academic-use/1))
- [ ] **CALC-032 RCPCH Digital Growth Charts** - UK-WHO + UK90; z-score / centile / SDS, chart rendering. Needs LMS tables (binary-size variable) and confirmation of RCPCH licensing.
- [ ] **CALC-044 Protein / macronutrient target** - g/day from weight or LBM + goal (e.g. 1.6-2.2 g/kg for lean-mass retention in a deficit)
- [ ] **CALC-046 DOTS** - alternative bodyweight-adjusted strength score to the shipped Wilks calculator; needs DOTS's polynomial coefficients confirmed against a citable primary source or official federation technical documentation, not just secondary calculator sites, before implementation
- [ ] **CALC-048 Skinfold body fat % (Jackson-Pollock / Durnin-Womersley)** - caliper-derived body-fat estimate for training and body-comparison settings
- [ ] **CALC-049 Body adiposity index (BAI)** - %BF proxy from height and hip circumference; population-specific (Hispanic-origin calibration)
- [ ] **CALC-051 Fat-free mass index (FFMI)** - fat-free mass normalised to height^2; used in sports medicine and sarcopenia screening
- [ ] **CALC-052 Skeletal muscle mass index (SMI)** - appendicular lean mass / height^2; sarcopenia definition (EWGSOP2 / FNIH)
- [ ] **CALC-053 Axial length centile charts (CREAM-Kids)** - age-, sex-, and region-specific centile charts for axial eye length in children and adolescents, from the CREAM-Kids Consortium (Kneepkens, Lingham, Mackey et al, *JAMA Ophthalmology* 2026). Reusable coefficients or tables and their distribution terms must be confirmed before implementation. Do not infer a serial rate-of-change risk model unless a primary source specifies and validates one. Shared centile-engine work is tracked only under ENG-010 in [`roadmap.md`](roadmap.md#eng-010-generic-centile-engine).
