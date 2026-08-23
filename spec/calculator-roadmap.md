# Clinical Calculator Roadmap

The clinical-calculator backlog, grouped strictly by completion status. Calculator categorisation by specialty / setting / status lives in [tags](../docs/calculators.md#filtering-by-tag) on each calculator - this file is purely a list of what is being built and what is queued.

**Engineering, infrastructure, GUI, distribution, and any other non-calculator work lives in [`spec/roadmap.md`](roadmap.md), not this file.** Keeping the two split means a clinician scanning the calculator backlog is not buried in build-tooling items, and an engineer scanning the build-tooling roadmap is not buried in clinical scores.

Roadmap items have stable identifiers so they can be referred to in conversation, commits, PRs, and release notes. Do not renumber existing IDs just because items are completed or removed.

Completed calculators are removed from this file rather than kept as roadmap history. The shipped catalogue is [`docs/calculators.md`](../docs/calculators.md).

## Status legend

- `[~]` **In-progress** - actively being implemented or under review.
- `[ ]` **Planned** - committed to build; the next batch.
- Items under [Future](#future) are explicitly **under consideration** rather than committed - they get promoted to **Planned** when scheduled.

---

## Calculators

### In-progress

_None active right now._

### Planned

_Nothing currently committed to build. Promote from [Future](#future) when scheduled._

### Future

Calculators worth shipping, under consideration. Largely surfaced from sibling open-source projects (notably [MedikQuantis](https://medikquantis.me), MIT licensed). Clinical context for each lives in the [docs catalogue wishlist](../docs/calculators.md#wishlist-candidates-for-future-addition).

Anthropometric and body-composition measures beyond the shipped BMI, body-fat-circumference, WHtR, 1RM, and training-zone calculators are grouped under CALC-046..052. They range from simple tape-measure proxies (WHR, RFM, BAI) to lab/field methods for body composition (skinfolds, FFMI, SMI) and strength tools (Wilks/DOTS).

- [ ] **CALC-018 Glasgow-Blatchford** - Upper-GI bleed pre-endoscopy triage
- [ ] **CALC-019 Hinchey** - Acute diverticulitis anatomy
- [ ] **CALC-020 Hyperglycaemia-corrected sodium** (Katz / Hillier)
- [ ] **CALC-041 LDL / non-HDL cholesterol** - Friedewald, Martin-Hopkins, and Sampson-NIH LDL estimation from a lipid panel
- [ ] **CALC-021 LRINEC** - Necrotising fasciitis
- [ ] **CALC-022 MELD 3.0** - Updated MELD
- [ ] **CALC-023 Modified Duke criteria** - Infective endocarditis
- [ ] **CALC-024 NIHSS** - Acute stroke severity
- [ ] **CALC-025 Norton Scale** - Pressure-ulcer risk (immobile)
- [ ] **CALC-026 NYHA** - Heart-failure functional class
- [ ] **CALC-027 ORBIT** - Bleeding risk in AF (DOAC era)
- [ ] **CALC-028 PASI** - Psoriasis Area and Severity Index
- [ ] **CALC-030 Pitt Bacteraemia** - BSI severity
- [ ] **CALC-044 Protein / macronutrient target** - g/day from weight or LBM + goal (e.g. 1.6-2.2 g/kg for lean-mass retention in a deficit)
- [ ] **CALC-031 PSA density** - PSA / prostate volume
- [ ] **CALC-032 RCPCH Digital Growth Charts** - UK-WHO + UK90; z-score / centile / SDS, chart rendering. Needs LMS tables (binary-size variable) and confirmation of RCPCH licensing.
- [ ] **CALC-033 RCRI** (Lee) - Pre-op cardiac risk
- [ ] **CALC-034 SCORAD** - Atopic dermatitis severity
- [ ] **CALC-035 SCORE2 / SCORE2-OP** - ESC 2021 CV risk (verify licensing)
- [ ] **CALC-036 StatinMD** (Oxford STRATIFY) - personalised 1/5/10-year risk of serious statin-induced muscle disorders; natural pairing with QRISK3 (benefit vs harm). Academic licence via Oxford University Innovation (Cai et al, *Lancet Digital Health* 2026; [licence page](https://process.innovation.ox.ac.uk/software/p/25396/stratify---stainmd-risk-calculator---academic-use/1))
- [ ] **CALC-046 Wilks / DOTS** - bodyweight-adjusted strength score
- [ ] **CALC-047 Waist-to-hip ratio (WHR)** - abdominal-vs-gluteal adiposity distribution; sex-specific metabolic-risk thresholds
- [ ] **CALC-048 Skinfold body fat % (Jackson-Pollock / Durnin-Womersley)** - caliper-derived body-fat estimate for training and body-comparison settings
- [ ] **CALC-049 Body adiposity index (BAI)** - %BF proxy from height and hip circumference; population-specific (Hispanic-origin calibration)
- [ ] **CALC-050 Relative fat mass (RFM)** - simplified sex-specific height/waist equation validated against DXA
- [ ] **CALC-051 Fat-free mass index (FFMI)** - fat-free mass normalised to height^2; used in sports medicine and sarcopenia screening
- [ ] **CALC-052 Skeletal muscle mass index (SMI)** - appendicular lean mass / height^2; sarcopenia definition (EWGSOP2 / FNIH)
