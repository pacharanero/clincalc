<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# CHA2DS2-VASc: shared source of truth

Stroke risk in non-valvular atrial fibrillation, guiding anticoagulation decisions. This document exists per roadmap item `COLL-001` as a converged, shared reference between `clincalc` and [MedikQuantis](https://medikquantis.me), so both projects can verify their implementations point-for-point against the same table rather than against each other's code. Implementation: [`src/calculators/cha2ds2vasc.rs`](../../src/calculators/cha2ds2vasc.rs).

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

Newer schemes (CHA2DS2-VA) drop the sex criterion entirely; `clincalc` still follows the original CHA2DS2-VASc definition pending a possible future `CHA2DS2-VA` calculator.

## Annual stroke risk by score

Friberg 2012 (the Swedish Atrial Fibrillation cohort study, 182,678 patients) is a larger validation cohort than the original 2010 derivation study and is the reference `clincalc` uses for the annual ischaemic stroke risk shown alongside the score (`annual_stroke_risk_percent` in `working`):

| Score | Annual stroke risk (%) |
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

Note the risk is **not monotonic** at the top of the range in this cohort - score 8 (10.8%) is lower than score 7 (11.2%). This is a genuine feature of the source data, not a transcription error, and is pinned by `friberg_2012_stroke_risk_table` in the test suite so a future refactor cannot "smooth" it into a monotonic curve by mistake.

## Inputs

Each boolean criterion is a clinician-asserted predicate with an explicit include/exclude definition (see `spec/calculator-input-definitions.md` for the governing system, and `input_schema()` in the implementation for the full text, including SNOMED CT ECL expressions). The two subtleties most likely to cause point-for-point divergence between implementations:

- **Vascular disease (V)** and **stroke/TIA/thromboembolism (S2)** are both explicitly **arterial** criteria. Venous thromboembolism (DVT or PE) does not count toward either.
- **Age** is a single numeric input from which the two mutually exclusive bands are derived, so an implementation cannot accidentally apply both the 65-74 and >=75 points to the same patient.

## Cross-project verification (COLL-001.1 / COLL-001.2)

Verified 2026-07-28 against MedikQuantis's actual source, [`packages/calculators/src/cha2ds2vasc.ts`](https://github.com/laurapiro17/medikquantis/blob/main/packages/calculators/src/cha2ds2vasc.ts) (MIT-licensed, public repo), not just its published description:

- **Raw scoring is identical**: age 65-74 = 1, age >=75 = 2, female sex = 1, CHF/HTN/DM/vascular disease = 1 each, stroke/TIA/thromboembolism = 2, maximum 9. Confirms `COLL-001.1`.
- **The female-sex-only edge case is handled identically**: a woman whose only point is the sex point (no other risk factors, age <65) is treated as equivalent to a man scoring 0 - anticoagulation not recommended. Confirms `COLL-001.2`.
- **The recommendation *thresholds* genuinely diverge, and this is not a bug in either project**: MedikQuantis follows ESC guidance, under which "consider" extends to a score of 2 in women (their code puts "score 1 (men) or score 2 (women)" in the moderate/"consider" tier, and "score >=2 (men) or >=3 (women)" in the high/"offer" tier). `clincalc` follows NICE NG196, which offers anticoagulation at a score of 2 **or above regardless of sex**, and reserves "consider" only for men scoring 1 - a woman scoring 2 (one risk factor plus the sex point) is "offer", not "consider", under NICE NG196. Both implementations are correct against their respective cited guidelines; this is the open question already on record in `spec/roadmap.md` ("Do we align on the NICE NG196 recommendation wording, or keep locale-specific guidance separate until ENG-001 lands?"), now confirmed as a real rather than hypothetical divergence. No code change follows from this alone - resolving it means picking a guideline-per-locale story, which is `ENG-001` territory.

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
