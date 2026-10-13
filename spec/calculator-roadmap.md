# Clinical Calculator Roadmap

The clinical-calculator backlog, grouped strictly by completion status. Calculator categorisation by specialty / setting / status lives in [tags](../docs/calculators.md#filtering-by-tag) on each calculator - this file is purely a list of what is being built and what is queued.

**Engineering, infrastructure, GUI, distribution, and any other non-calculator work lives in [`spec/roadmap.md`](roadmap.md), not this file.** Keeping the two split means a clinician scanning the calculator backlog is not buried in build-tooling items, and an engineer scanning the build-tooling roadmap is not buried in clinical scores.

Roadmap items have stable identifiers so they can be referred to in conversation, commits, PRs, and release notes. Do not renumber existing IDs just because items are completed or removed.

Completed calculators are removed from this file rather than kept as roadmap history. The shipped catalogue is [`docs/calculators.md`](../docs/calculators.md).

MedikQuantis parity items follow the normal calculator completion rule: an independently implemented calculation is complete when it passes the primary-source, licence, closed-schema, clinical-safety, and testing gates. Translation availability is tracked independently in the README calculator-status table and does not block calculator completion. A locale is advertised only after its complete attributed bundle passes the review gates in [`multilingual.md`](multilingual.md) and [`docs/translating.md`](../docs/translating.md). Upstream MedikQuantis logic and tests are useful cross-project evidence, not a substitute for clincalc's primary-source verification.

## Status legend

- `[~]` **In-progress** - actively being implemented or under review.
- `[ ]` **Planned** - committed to build; the next batch.
- Items under [Future](#future) are explicitly **under consideration** rather than committed - they get promoted to **Planned** when scheduled.

---

## Calculators

### In-progress

_Nothing currently in progress._

### Planned

_Nothing currently planned._

### Future

Calculators worth shipping, under consideration. Largely surfaced from sibling open-source projects (notably [MedikQuantis](https://medikquantis.me), MIT licensed). Clinical context for each lives in the [docs catalogue wishlist](../docs/calculators.md#wishlist-candidates-for-future-addition).

#### MedikQuantis parity gaps

This is the remaining calculation gap against MedikQuantis's live 65-calculator registry at upstream commit [`16c63c85`](https://github.com/laurapiro17/medikquantis/tree/16c63c85aee7a64417205f60cb3e66fccf19fae2), reviewed 2026-08-29. clincalc ships 61 equivalent calculations and lacks the 4 below. MedikQuantis's README still reports 49 calculators, so its registry is the authoritative inventory. Review this snapshot when upstream changes; add new stable `CALC-*` items rather than silently letting parity drift.

For every item below, completion means independent implementation from the primary source; closed schema, licence evidence, and literature-vector tests under the normal calculator contract; and explicit comparison with the pinned MedikQuantis behavior with intentional clinical or guideline differences documented. Machine identifiers and numeric behavior remain locale-neutral. Reviewed translations are tracked independently in the README and do not block completion.

The upstream combined BMI / BSA / ideal-body-weight item (`CALC-056`) is completed through the existing standalone BMI and Mosteller BSA calculators plus `CALC-066`, the named NIH-NHLBI ARDSNet adult predicted-body-weight protocol. This intentionally does not reproduce the upstream generic Devine/adjusted-weight outputs: the upstream panel applies adult BMI to paediatric-sized inputs, treats a dosing scalar as a personal ideal weight, and applies an uncited adjusted-weight formula whenever actual weight exceeds rounded Devine weight. Adjusted weight is indication- and protocol-specific, so no universal value is emitted.

`CALC-027` ORBIT is completed as an independent implementation of the factual five-factor method and observed rates in O'Brien et al. The source article is CC BY-NC 4.0, so clincalc does not reproduce its prose, tables, figures, or presentation. The legal provenance is the expression-method distinction in [WIPO Copyright Treaty Article 2](https://www.wipo.int/wipolex/en/text/295166): copyright protection extends to expression, not ideas, procedures, methods of operation, or mathematical concepts as such. Unlike the pinned upstream calculator, which accepts pre-resolved anaemia and renal booleans and labels category incidence as annual risk percent, clincalc requires raw sex-branch haemoglobin, haematocrit, and eGFR values and reports score-specific and category-level observed bleeds per 100 patient-years, explicitly not a personalised annual probability. Its broad numeric upper bounds are clincalc input-safety guards against unit and data-entry errors, not source scoring thresholds.

`CALC-026` NYHA is completed from the NCI Thesaurus Class I-IV definitions, which NCI distributes under CC BY 4.0. clincalc requires explicit attestation of defined or presumed cardiac disease and a mutually exclusive functional profile, then returns the ordinal and Roman class without the pinned upstream calculator's unsupported treatment recommendations. The response records the selected profile and NCI concept identifier and warns that NYHA is subjective, particularly across Classes II and III: Raphael et al. reported only 54% interobserver concordance.

- [ ] **CALC-025 Norton Scale** - Pressure-ulcer risk; blocked pending unrestricted redistribution permission or legal review because the Centre for Policy on Ageing claims ownership of the scale
- [ ] **CALC-034 SCORAD** - Atopic dermatitis severity; blocked pending explicit unrestricted permission or legal review because [ePROVIDE identifies SCORAD as all rights reserved](https://eprovide.mapi-trust.org/instruments/scoring-in-atopic-dermatitis) and the [Eczema Foundation legal notice](https://www.pierrefabreeczemafoundation.org/en/legal-notice-po-scorad) reserves adaptation, translation, and software-integration rights
- [ ] **CALC-035 SCORE2 / SCORE2-OP** - ESC 2021 cardiovascular risk; blocked pending explicit unrestricted software-redistribution terms or legal review
- [ ] **CALC-061 SAD PERSONS** - Blocked on clinical-safety grounds: poor predictive performance makes a scored implementation liable to misuse for discharge, observation, or referral despite warnings

#### QResearch / ClinRisk candidates

Further QResearch-derived algorithms from the same ClinRisk open-source lineage as the shipped `qrisk3`, `qfracture`, and `qdiabetes` ports. ClinRisk also mirrors source for `qKidney`, `qStroke`, `qThrombosis`, `qbleed`, and `qStatin` at [nhsland/clinrisk-modules](https://github.com/nhsland/clinrisk-modules) under the same README-stated LGPL/AGPL licensing rationale; none of those has had its individual `COPYING` file verified yet, so none has a `CALC-*` identifier until that licensing scan happens.

- [ ] **CALC-069 QCancer** - Sex-specific absolute risk across roughly ten cancer sites (blood, breast, cervical, colorectal, gastro-oesophageal, lung, ovarian, pancreatic, renal tract, uterine for women; an analogous male-specific set) from routinely-collected primary-care risk factors and new symptoms, informing NICE NG12 suspected-cancer referral thresholds. ClinRisk released the source under AGPL-3.0 at [nhsland/clinrisk-modules](https://github.com/nhsland/clinrisk-modules/tree/master/qCancer), same faithful-reimplementation rationale as QRISK3/QDiabetes. Two blockers before implementation: (1) the openly-released coefficients are QCancer-2013 vintage (source files dated 2012-13); qcancer.org documents a 2017 recalibration and Hippisley-Cox has published further recalibration work since (CanPredict, 2023/2025), and no newer open coefficient release has been confirmed - do not present 2013 coefficients as the current algorithm without disclosing the vintage explicitly, and do not implement at all until a current open release is confirmed or the vintage disclosure is accepted as sufficient; (2) QCancer is not one score but roughly ten independent per-site logistic sub-models per sex, so a single-calculator-returning-a-vector-of-risks vs. several separate `CALC-*` entries decision is needed before implementation starts.

#### Other candidates

Shipped anthropometric and body-composition measures include BMI, body-fat circumference, Jackson-Pollock skinfolds, BAI, WHtR, WHR, RFM, FFMI, SMI, 1RM, training zones, and the Wilks score. Remaining non-MedikQuantis candidates include other lab or field methods for body composition and the alternative DOTS strength score.

- [ ] **CALC-036 StatinMD** (Oxford STRATIFY) - personalised 1/5/10-year risk of serious statin-induced muscle disorders; natural pairing with QRISK3 (benefit vs harm). Academic licence via Oxford University Innovation (Cai et al, *Lancet Digital Health* 2026; [licence page](https://process.innovation.ox.ac.uk/software/p/25396/stratify---stainmd-risk-calculator---academic-use/1))
- [ ] **CALC-032 RCPCH Digital Growth Charts** - UK-WHO + UK90; z-score / centile / SDS, chart rendering. Needs LMS tables (binary-size variable) and confirmation of RCPCH licensing.
- [ ] **CALC-044 Energy-restricted protein / macronutrient target** - protein or broader macronutrient targets from weight or lean body mass and a defined goal, including lean-mass retention during a caloric deficit. The shipped `protein_target` calculator deliberately does not complete this candidate: Morton et al. excluded energy-restricted participants and supports only a total-body-weight protein amount for healthy adults undertaking resistance training.
- [ ] **CALC-046 DOTS** - alternative bodyweight-adjusted strength score to the shipped Wilks calculator; needs DOTS's polynomial coefficients confirmed against a citable primary source or official federation technical documentation, not just secondary calculator sites, before implementation
- [ ] **CALC-053 Axial length centile charts (CREAM-Kids)** - age-, sex-, and region-specific centile charts for axial eye length in children and adolescents, from the CREAM-Kids Consortium (Kneepkens, Lingham, Mackey et al, *JAMA Ophthalmology* 2026). Reusable coefficients or tables and their distribution terms must be confirmed before implementation. Do not infer a serial rate-of-change risk model unless a primary source specifies and validates one. Shared centile-engine work is tracked only under ENG-010 in [`roadmap.md`](roadmap.md#eng-010-generic-centile-engine).
