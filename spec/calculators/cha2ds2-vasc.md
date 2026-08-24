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

Note the rate is **not monotonic** at the top of the range in this cohort - score 8 (10.8) is lower than score 7 (11.2). This is a genuine feature of the source data, not a transcription error, and is pinned by `friberg_2012_stroke_rate_table` in the test suite so a future refactor cannot "smooth" it into a monotonic curve by mistake.

## Inputs

Each boolean criterion is a clinician-asserted predicate with an explicit include/exclude definition (see `spec/calculator-input-definitions.md` for the governing system, and `input_schema()` in the implementation for the full text, including SNOMED CT ECL expressions). The two subtleties most likely to cause point-for-point divergence between implementations:

- **Vascular disease (V)** and **stroke/TIA/thromboembolism (S2)** are both explicitly **arterial** criteria. Venous thromboembolism (DVT or PE) does not count toward either.
- **Age** is a single numeric input from which the two mutually exclusive bands are derived, so an implementation cannot accidentally apply both the 65-74 and >=75 points to the same patient.

## Cross-project verification (COLL-001.1 / COLL-001.2)

Verified 2026-07-28, S2 scope re-verified 2026-08-24, against MedikQuantis's actual source at commit [`cf5afb9`](https://github.com/laurapiro17/medikquantis/blob/cf5afb993fb900d28542b39ae022df6932e34076/packages/calculators/src/cha2ds2vasc.ts) (`packages/calculators/src/cha2ds2vasc.ts`, MIT-licensed, public repo), not just its published description. Pinned to this commit rather than `main` so the citation stays valid if their file changes later.

- **Point values agree on the shared input semantics**: age 65-74 = 1, age >=75 = 2, female sex = 1, CHF/HTN/DM/vascular disease = 1 each, prior stroke/TIA = 2, maximum 9.
- **S2 criterion - point value agrees, semantic scope confirmed narrower than `clincalc`'s**: MedikQuantis exposes this input only as a boolean `strokeOrTia` with no validation comments or description in `cha2ds2vasc.ts` itself. Its UI copy in `apps/web/messages/{en,es}.json` (same commit) labels the field **"Prior stroke or TIA"** / **"Ictus o AIT previo"** with no other help text - no mention of systemic arterial thromboembolism in either locale, and no separate docs file describes the criterion further. This confirms MedikQuantis's `strokeOrTia` is scoped to stroke/TIA only, narrower than `clincalc`'s `stroke_tia_thromboembolism`, which explicitly also counts systemic arterial thromboembolism per the criterion's include/exclude definition. Both projects score it at 2 points regardless, so this is a semantic-scope divergence with no numeric disagreement on the cases both implementations recognise. `COLL-001.1` is complete.
- **The female-sex-only edge case is handled identically**: a woman whose only point is the sex point (no other risk factors, age <65) is treated as equivalent to a man scoring 0 - anticoagulation not recommended. Confirms `COLL-001.2`.
- **The recommendation thresholds diverge**: MedikQuantis's `cha2ds2vasc.ts` puts "score 1 (men) or score 2 (women)" in a moderate/"consider" tier, and "score >=2 (men) or >=3 (women)" in a high/"offer" tier. `clincalc` follows NICE NG196, which offers anticoagulation at a score of 2 **or above regardless of sex**, and reserves "consider" only for men scoring 1. The sex-specific MedikQuantis thresholds reflect the pre-2024 CHA2DS2-VASc approach, but its pinned source cites the 2024 ESC guideline, which instead recommends the sex-free CHA2DS2-VA scheme. Its guideline attribution therefore needs clarification before the implementations can be described as correct against their respective cited guidelines. No `clincalc` code change follows from this documentation discrepancy.

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
- Van Gelder IC, Rienstra M, Bunting KV, et al. 2024 ESC Guidelines for the management of atrial fibrillation. Eur Heart J. 2024;45(36):3314-3414. <https://doi.org/10.1093/eurheartj/ehae176>

## Open items

- Inviting the MedikQuantis author to review this spec and contribute her test cases (`COLL-001.5`) is a standing outreach action, not a code change - tracked in `spec/roadmap.md`.
- Resolve the MedikQuantis CHA2DS2-VASc recommendation-threshold citation before describing both implementations as guideline-conformant.
- A "sister projects" note in `docs/calculators.md` (`COLL-001.6`) is deferred until convergence is confirmed from both sides.
