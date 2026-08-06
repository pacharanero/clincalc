<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# CHA2DS2-VASc: shared source of truth

Stroke risk in non-valvular atrial fibrillation, guiding anticoagulation decisions. This document exists per roadmap item `COLL-001` as a shared reference for converging `clincalc` and [MedikQuantis](https://medikquantis.me), so both projects can verify their implementations point-for-point against the same table rather than against each other's code. Convergence is in progress, not complete - see "Open items" below for what's still outstanding, including external review by MedikQuantis's author. Implementation: [`src/calculators/cha2ds2vasc.rs`](../../src/calculators/cha2ds2vasc.rs).

## Scoring table

| Criterion | Points |
|---|---|
| **C** - Congestive heart failure / LV dysfunction | 1 |
| **H** - Hypertension | 1 |
| **A2** - Age >=75 | 2 |
| **D** - Diabetes mellitus | 1 |
| **S2** - Prior stroke, TIA, or systemic arterial thromboembolism | 2 |
| **V** - Vascular disease (prior MI, peripheral arterial disease, aortic plaque) | 1 |
| **A** - Age 65-74 | 1 |
| **Sc** - Sex category (female) | 1 |

Maximum score: **9**. Age contributes to exactly one band (0, 1, or 2 points) - the 65-74 and >=75 bands are mutually exclusive, never additive.

## Recommendation bands (NICE NG196)

- **Score 0** (score 0 in men, or 1 in women arising only from the sex point): anticoagulation **not recommended** - low risk.
- **Score 1 in men** (excluding the sex-only case above): **consider** anticoagulation, weighing bleeding risk and patient preference.
- **Score >=2**, or any score >=1 not covered above: **offer** anticoagulation, taking bleeding risk into account.

### The female-sex-only edge case

A total score of 1 arising *only* from the female-sex point (no other risk factors, age <65) is treated as **low risk** - anticoagulation is not recommended. Female sex is an age-dependent risk *modifier*, not an independent indication; scoring it as a stand-alone indication for anticoagulation is a common implementation bug. `clincalc` encodes this via `non_sex_score` (the score excluding the sex point): when `non_sex_score == 0`, the recommendation is always "not recommended" regardless of the sex point. See `female_sex_only_is_low_risk` in the test suite.

The 2024 ESC AF guidelines moved to CHA2DS2-VA, which drops the sex criterion entirely; `clincalc` already implements this as a separate calculator (`cha2ds2_va`, [`src/calculators/cha2ds2_va.rs`](../../src/calculators/cha2ds2_va.rs)) alongside the original CHA2DS2-VASc here, rather than replacing one with the other.

## Stroke event rate by score (Friberg 2012)

Friberg 2012 (the Swedish Atrial Fibrillation cohort study) is a larger validation cohort than the original 2010 derivation study. Table 3 of that paper reports the observed ischaemic stroke **event rate per 100 patient-years** among the roughly 90,490 patients in the cohort who remained off warfarin - a person-time incidence rate in a specific untreated subgroup, not a one-year cumulative risk prediction for an individual patient or for the full 182,678-patient cohort. `clincalc` exposes it as an epidemiological reference point alongside the score (`friberg_2012_stroke_rate_per_100_patient_years` in `working`, with the citation itself in `friberg_2012_stroke_rate_reference`):

| Score | Stroke rate (per 100 patient-years, untreated) |
|---|---|
| 0 | 0.2 |
| 1 | 0.6 |
| 2 | 2.2 |
| 3 | 3.2 |
| 4 | 4.8 |
| 5 | 7.2 |
| 6 | 9.7 |
| 7 | 11.2 |
| 8 | 10.8 |
| 9 | 12.2 |

Note the rate is **not monotonic** at the top of the range in this cohort - score 8 (10.8) is lower than score 7 (11.2). This is a genuine feature of the source data, not a transcription error, and is pinned by `friberg_2012_stroke_risk_table` in the test suite so a future refactor cannot "smooth" it into a monotonic curve by mistake.

## Inputs

Each boolean criterion is a clinician-asserted predicate with an explicit include/exclude definition (see `spec/calculator-input-definitions.md` for the governing system, and `input_schema()` in the implementation for the full text, including SNOMED CT ECL expressions). The two subtleties most likely to cause point-for-point divergence between implementations:

- **Vascular disease (V)** and **stroke/TIA/thromboembolism (S2)** are both explicitly **arterial** criteria. Venous thromboembolism (DVT or PE) does not count toward either.
- **Age** is a single numeric input from which the two mutually exclusive bands are derived, so an implementation cannot accidentally apply both the 65-74 and >=75 points to the same patient.

## Cross-project verification (COLL-001.1 / COLL-001.2)

Verified 2026-07-28 against MedikQuantis's actual source at commit [`cf5afb9`](https://github.com/laurapiro17/medikquantis/blob/cf5afb993fb900d28542b39ae022df6932e34076/packages/calculators/src/cha2ds2vasc.ts) (`packages/calculators/src/cha2ds2vasc.ts`, MIT-licensed, public repo), not just its published description. Pinned to this commit rather than `main` so the citation stays valid if their file changes later.

- **Point values agree** on every criterion: age 65-74 = 1, age >=75 = 2, female sex = 1, CHF/HTN/DM/vascular disease = 1 each, stroke/TIA/thromboembolism = 2, maximum 9. Confirms the arithmetic half of `COLL-001.1`.
- **S2 criterion - point value confirmed, full semantic scope not yet verified**: MedikQuantis exposes and labels this input only as `strokeOrTia`. Both projects agree it is worth 2 points, but whether MedikQuantis's field also captures systemic arterial thromboembolism (as `clincalc`'s `stroke_tia_thromboembolism` explicitly does) hasn't been confirmed from their source alone - that needs either a look at their input validation/UI copy or confirmation from Laura directly. Treat `COLL-001.1` as agreed on point values, still open on this one criterion's exact inclusion boundary.
- **The female-sex-only edge case is handled identically**: a woman whose only point is the sex point (no other risk factors, age <65) is treated as equivalent to a man scoring 0 - anticoagulation not recommended. Confirms `COLL-001.2`.
- **The recommendation *thresholds* genuinely diverge, and this is not a bug in either project**: MedikQuantis's `cha2ds2vasc.ts` (a sex-specific scheme, distinct from their separate `cha2ds2va.ts` which implements the newer sex-free ESC 2024 score) puts "score 1 (men) or score 2 (women)" in a moderate/"consider" tier, and "score >=2 (men) or >=3 (women)" in a high/"offer" tier - so "consider" extends to a score of 2 in women. `clincalc` follows NICE NG196, which offers anticoagulation at a score of 2 **or above regardless of sex**, and reserves "consider" only for men scoring 1 - a woman scoring 2 (one risk factor plus the sex point) is "offer", not "consider", under NICE NG196. Both implementations are correct against their respective cited guidelines; this is the open question already on record in `spec/roadmap.md` ("Do we align on the NICE NG196 recommendation wording, or keep locale-specific guidance separate until ENG-001 lands?"), now confirmed as a real rather than hypothetical divergence. No code change follows from this alone - resolving it means picking a guideline-per-locale story, which is `ENG-001` territory.

## Test-vector table

Vectors below are implemented as unit tests in [`src/calculators/cha2ds2vasc.rs`](../../src/calculators/cha2ds2vasc.rs); this table is a portable, language-agnostic summary for cross-checking against MedikQuantis or any other implementation.

| Case | Age | Sex | C | H | D | S2 | V | Score | Recommendation |
|---|---|---|---|---|---|---|---|---|---|
| Male, no factors | 60 | M | | | | | | 0 | Not recommended |
| Female, no factors (sex-only) | 60 | F | | | | | | 1 | Not recommended |
| Male, age 65-74 only | 70 | M | | | | | | 1 | Consider |
| Male, age boundary (below) | 74 | M | | | | | | 1 | Consider |
| Male, age boundary (at) | 75 | M | | | | | | 2 | Offer |
| Male, vascular disease only | 60 | M | | | | | X | 1 | Consider |
| Female, age 65-74, HTN, DM | 70 | F | | X | X | | | 4 | Offer |
| Male, age >=75, prior stroke | 80 | M | | | | X | | 4 | Offer |
| Maximum score | 80 | F | X | X | X | X | X | 9 | Offer |

## References

- Lip GYH, Nieuwlaat R, Pisters R, et al. Refining clinical risk stratification for predicting stroke and thromboembolism in atrial fibrillation using a novel risk factor-based approach: the Euro Heart Survey on Atrial Fibrillation. Chest. 2010;137(2):263-272. <https://doi.org/10.1378/chest.09-1584>
- Friberg L, Rosenqvist M, Lip GYH. Evaluation of risk stratification schemes for ischaemic stroke and bleeding in 182,678 patients with atrial fibrillation: the Swedish Atrial Fibrillation cohort study. Eur Heart J. 2012;33(12):1500-1510. PMID 22246443.
- NICE NG196: Atrial fibrillation - diagnosis and management.

## Open items

- Inviting the MedikQuantis author to review this spec and contribute her test cases (`COLL-001.5`) is a standing outreach action, not a code change - tracked in `spec/roadmap.md`.
- A "sister projects" note in `docs/calculators.md` (`COLL-001.6`) is deferred until convergence is confirmed from both sides.
