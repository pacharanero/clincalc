# Clinical Calculator Roadmap

The clinical-calculator backlog, grouped strictly by completion status. Calculator categorisation by specialty / setting / status now lives in [tags](../docs/calculators.md#filtering-by-tag) on each calculator - this file is purely a list of what is done, what is being built, and what is queued.

**Engineering, infrastructure, GUI, distribution, and any other non-calculator work lives in [`spec/roadmap.md`](roadmap.md), not this file.** Keeping the two split means a clinician scanning the calculator backlog is not buried in build-tooling items, and an engineer scanning the build-tooling roadmap is not buried in clinical scores.

## Status legend

- `[x]` **Completed** - shipped in `clincalc::all()`; returns a real score against literature vectors.
- `[x]` (stub) - **Completed as a proprietary / unavailable stub**. Listed, schemable, and embeddable, but invoking it returns a structured explanation (owner, reason, open alternative) rather than a score because the algorithm or content is licence-locked. Named on purpose.
- `[~]` **In-progress** - actively being implemented or under review.
- `[ ]` **Planned** - committed to build; the next batch.
- Items under [Future](#future) are explicitly **under consideration** rather than committed - they get promoted to **Planned** when scheduled.

---

## Calculators

### Completed

Active calculators (42) that compute a real score:

- [x] **4AT** - Delirium screening at bedside
- [x] **ABCD2** - Short-term stroke risk after TIA
- [x] **ABPI** - Ankle-brachial pressure index; PAD screen and compression-therapy safety
- [x] **AMTS** - Abbreviated Mental Test Score
- [x] **ASRS-v1.1** - Adult ADHD screener (WHO)
- [x] **AUDIT** - Alcohol misuse screening (10-item)
- [x] **AUDIT-C** - Brief alcohol screen (3-item)
- [x] **BODE Index** - COPD mortality prognostication
- [x] **CHA₂DS₂-VASc** - Stroke risk in AF
- [x] **CHALICE** - CT-head decision in paediatric head injury
- [x] **Child-Pugh** - Cirrhosis severity
- [x] **CURB-65** - Pneumonia severity / admission decision
- [x] **DAS28** - Rheumatoid arthritis disease activity
- [x] **eGFR (CKD-EPI 2021)** - Race-free eGFR; CKD G-stage
- [x] **eGFR + uACR heatmap** (`ckd_risk`) - KDIGO CKD risk category
- [x] **EPDS** - Edinburgh Postnatal Depression Scale
- [x] **EuroSCORE II** - Operative mortality after cardiac surgery
- [x] **FeverPAIN** - Antibiotic prescribing in acute sore throat
- [x] **FIB-4** - Liver fibrosis screening
- [x] **GAD-7** - Anxiety severity
- [x] **Gleason Grade Groups** - Prostate cancer aggressiveness
- [x] **GRACE** - In-hospital mortality in ACS
- [x] **HAS-BLED** - Bleeding risk on anticoagulation
- [x] **HEART** - ED chest-pain risk stratification
- [x] **IPSS** - International Prostate Symptom Score
- [x] **MELD** (original 2001) - End-stage liver disease
- [x] **MRC Dyspnoea** - Breathlessness grading
- [x] **NEWS2** - Acute illness severity / sepsis trigger (NHS-mandated)
- [x] **Nottingham Hip Fracture Score** - 30-day mortality post hip fracture
- [x] **Nottingham Prognostic Index** - Breast cancer prognosis
- [x] **Padua Prediction Score** - VTE risk in medical inpatients
- [x] **PHQ-9** - Depression severity & monitoring
- [x] **qSOFA** - Rapid sepsis identification
- [x] **QFracture** - 10-year fracture risk (UK-native; open alternative to FRAX)
- [x] **QRISK3** - 10-year cardiovascular risk
- [x] **SOFA** - Organ dysfunction in ICU / Sepsis-3
- [x] **TIMI** - UA/NSTEMI risk
- [x] **uACR** - Urine albumin-to-creatinine ratio + KDIGO A-stage
- [x] **UKELD** - UK transplant listing threshold
- [x] **Waterlow** - Pressure-ulcer risk
- [x] **Wells DVT** - Pre-test probability of DVT
- [x] **Wells PE** - Pre-test probability of PE

Proprietary / unavailable stubs (10) - listed and computable, but the response explains why the score is not shipped and points at an open alternative where one exists:

- [x] (stub) **ACQ** - Asthma Control Questionnaire (copyright Juniper)
- [x] (stub) **CAT** - COPD Assessment Test (copyright GSK)
- [x] (stub) **CFS** - Clinical Frailty Scale (copyright Rockwood et al.)
- [x] (stub) **ELF** - Enhanced Liver Fibrosis test (Siemens Healthineers)
- [x] (stub) **FRAX** - 10-year fracture risk (trade-secret algorithm)
- [x] (stub) **LANSS** - Neuropathic pain scale (copyright)
- [x] (stub) **MMSE** - Mini-Mental State Examination (copyright PAR Inc.)
- [x] (stub) **MUST** - Malnutrition Universal Screening Tool (BAPEN)
- [x] (stub) **OHS** - Oxford Hip Score (Isis Innovation licence)
- [x] (stub) **OKS** - Oxford Knee Score (Isis Innovation licence)

### In-progress

_None active right now._

### Planned

_Nothing currently committed to build. Promote from [Future](#future) when scheduled._

### Future

Calculators worth shipping, under consideration. Largely surfaced from sibling open-source projects (notably [MedikQuantis](https://medikquantis.me), MIT licensed). Clinical context for each lives in the [docs catalogue wishlist](../docs/calculators.md#wishlist-candidates-for-future-addition).

- [ ] **Albumin-corrected calcium** (Payne 1973)
- [ ] **Alvarado** - Acute appendicitis
- [ ] **Anion gap** - Na − (Cl + HCO₃)
- [ ] **APACHE II** - ICU severity / mortality
- [ ] **ASA Physical Status** - Peri-operative classification
- [ ] **ASCVD Pooled Cohort** - ACC/AHA 2013 CV risk (US population)
- [ ] **Barthel Index** - Activities of daily living
- [ ] **BASDAI** - Ankylosing spondylitis disease activity
- [ ] **Braden Scale** - Pressure-ulcer risk (acute care)
- [ ] **Caprini** - Peri-operative VTE risk
- [ ] **Centor / McIsaac** - Strep pharyngitis probability
- [ ] **CHA₂DS₂-VA** - 2024 ESC sex-free update of CHA₂DS₂-VASc
- [ ] **Charlson Comorbidity Index** - 10-year comorbidity mortality
- [ ] **EHRA** - AF symptom classification
- [ ] **FENa** - Fractional excretion of sodium
- [ ] **FINDRISC** - 10-year T2DM risk
- [ ] **GCS** - Glasgow Coma Scale
- [ ] **Glasgow-Blatchford** - Upper-GI bleed pre-endoscopy triage
- [ ] **Harris-Benedict** - Basal metabolic rate / energy needs
- [ ] **Hinchey** - Acute diverticulitis anatomy
- [ ] **Hyperglycaemia-corrected sodium** (Katz / Hillier)
- [ ] **LRINEC** - Necrotising fasciitis
- [ ] **MELD 3.0** - Updated MELD
- [ ] **Modified Duke criteria** - Infective endocarditis
- [ ] **NIHSS** - Acute stroke severity
- [ ] **Norton Scale** - Pressure-ulcer risk (immobile)
- [ ] **NYHA** - Heart-failure functional class
- [ ] **ORBIT** - Bleeding risk in AF (DOAC era)
- [ ] **PASI** - Psoriasis Area and Severity Index
- [ ] **PERC** - PE rule-out criteria
- [ ] **Pitt Bacteraemia** - BSI severity
- [ ] **PSA density** - PSA / prostate volume
- [ ] **RCPCH Digital Growth Charts** - UK-WHO + UK90; z-score / centile / SDS, chart rendering. Needs LMS tables (binary-size variable) and confirmation of RCPCH licensing.
- [ ] **RCRI** (Lee) - Pre-op cardiac risk
- [ ] **SCORAD** - Atopic dermatitis severity
- [ ] **SCORE2 / SCORE2-OP** - ESC 2021 CV risk (verify licensing)
- [ ] **StatinMD** (Oxford STRATIFY) - personalised 1/5/10-year risk of serious statin-induced muscle disorders; natural pairing with QRISK3 (benefit vs harm). Academic licence via Oxford University Innovation (Cai et al, *Lancet Digital Health* 2026; [licence page](https://process.innovation.ox.ac.uk/software/p/25396/stratify---stainmd-risk-calculator---academic-use/1))
