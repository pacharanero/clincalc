# Clinical Calculator Roadmap

The clinical-calculator backlog, grouped strictly by completion status. Calculator categorisation by specialty / setting / status lives in [tags](../docs/calculators.md#filtering-by-tag) on each calculator - this file is purely a list of what is being built and what is queued.

**Engineering, infrastructure, GUI, distribution, and any other non-calculator work lives in [`spec/roadmap.md`](roadmap.md), not this file.** Keeping the two split means a clinician scanning the calculator backlog is not buried in build-tooling items, and an engineer scanning the build-tooling roadmap is not buried in clinical scores.

Roadmap items have stable identifiers so they can be referred to in conversation, commits, PRs, and release notes. Do not renumber existing IDs just because items are completed or removed.

Completed calculators are removed from this file rather than kept as roadmap history. The shipped catalogue is [`docs/calculators.md`](../docs/calculators.md), currently 43 active calculators that compute a real score plus 10 proprietary / unavailable stubs.

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

- [ ] **CALC-001 Albumin-corrected calcium** (Payne 1973)
- [ ] **CALC-002 Alvarado** - Acute appendicitis
- [ ] **CALC-003 Anion gap** - Na − (Cl + HCO₃)
- [ ] **CALC-004 APACHE II** - ICU severity / mortality
- [ ] **CALC-005 ASA Physical Status** - Peri-operative classification
- [ ] **CALC-006 ASCVD Pooled Cohort** - ACC/AHA 2013 CV risk (US population)
- [ ] **CALC-007 Barthel Index** - Activities of daily living
- [ ] **CALC-008 BASDAI** - Ankylosing spondylitis disease activity
- [ ] **CALC-009 Braden Scale** - Pressure-ulcer risk (acute care)
- [ ] **CALC-010 Caprini** - Peri-operative VTE risk
- [ ] **CALC-011 Centor / McIsaac** - Strep pharyngitis probability
- [ ] **CALC-012 CHA₂DS₂-VA** - 2024 ESC sex-free update of CHA₂DS₂-VASc
- [ ] **CALC-013 Charlson Comorbidity Index** - 10-year comorbidity mortality
- [ ] **CALC-014 EHRA** - AF symptom classification
- [ ] **CALC-015 FENa** - Fractional excretion of sodium
- [ ] **CALC-016 FINDRISC** - 10-year T2DM risk
- [ ] **CALC-017 GCS** - Glasgow Coma Scale
- [ ] **CALC-018 Glasgow-Blatchford** - Upper-GI bleed pre-endoscopy triage
- [ ] **CALC-019 Hinchey** - Acute diverticulitis anatomy
- [ ] **CALC-020 Hyperglycaemia-corrected sodium** (Katz / Hillier)
- [ ] **CALC-021 LRINEC** - Necrotising fasciitis
- [ ] **CALC-022 MELD 3.0** - Updated MELD
- [ ] **CALC-023 Modified Duke criteria** - Infective endocarditis
- [ ] **CALC-024 NIHSS** - Acute stroke severity
- [ ] **CALC-025 Norton Scale** - Pressure-ulcer risk (immobile)
- [ ] **CALC-026 NYHA** - Heart-failure functional class
- [ ] **CALC-027 ORBIT** - Bleeding risk in AF (DOAC era)
- [ ] **CALC-028 PASI** - Psoriasis Area and Severity Index
- [ ] **CALC-029 PERC** - PE rule-out criteria
- [ ] **CALC-030 Pitt Bacteraemia** - BSI severity
- [ ] **CALC-031 PSA density** - PSA / prostate volume
- [ ] **CALC-032 RCPCH Digital Growth Charts** - UK-WHO + UK90; z-score / centile / SDS, chart rendering. Needs LMS tables (binary-size variable) and confirmation of RCPCH licensing.
- [ ] **CALC-033 RCRI** (Lee) - Pre-op cardiac risk
- [ ] **CALC-034 SCORAD** - Atopic dermatitis severity
- [ ] **CALC-035 SCORE2 / SCORE2-OP** - ESC 2021 CV risk (verify licensing)
- [ ] **CALC-036 StatinMD** (Oxford STRATIFY) - personalised 1/5/10-year risk of serious statin-induced muscle disorders; natural pairing with QRISK3 (benefit vs harm). Academic licence via Oxford University Innovation (Cai et al, *Lancet Digital Health* 2026; [licence page](https://process.innovation.ox.ac.uk/software/p/25396/stratify---stainmd-risk-calculator---academic-use/1))
