# Calculator catalogue

The full registry. 91 active calculators that compute a real result, plus 10 named-but-unavailable proprietary stubs (carrying the `proprietary` and `unavailable` tags). One row per calculator.

`clincalc list` prints the same data at any time; `clincalc list --tag <tag>` filters by tag; `clincalc calc <name> --license` prints the algorithm's distribution licence for any single entry.

!!! info "Two kinds of entry"
    **Active** entries compute a real score. Their algorithm is either public-domain (implemented from primary literature) or open-source (notably QRISK3 and QFracture, ported from ClinRisk's LGPL-3 source).

    **Unavailable** entries (`proprietary` + `unavailable` tags) are named on purpose. They appear in `clincalc list`, but invoking them returns a structured explanation - owner, reason, and an open alternative where one exists. See [Unavailable on principle](#unavailable-on-principle).

## Filtering by tag

Every calculator carries one or more **tags** - specialty (where it is used) and status (`proprietary`, `nhs-mandated`, `screening`, `risk`, ...). Tags drive the catalogue below, the `--tag` CLI filter, and the JSON output of `clincalc list`:

```bash
clincalc list --tag cardiology                       # everything in cardiology
clincalc list --tag primary-care --tag screening     # AND - narrows the filter
clincalc list --tag proprietary                      # the unshippable ones
clincalc tags                                        # enumerate every tag, with counts
```

The full vocabulary lives in [`src/tags.rs`](https://github.com/pacharanero/clincalc/blob/main/src/tags.rs) and is reviewable in one file. New tags are added there only after at least two calculators want one.

## Catalogue

| Name | Title | What it does | Tags |
|---|---|---|---|
| `abcd2` | ABCD2 Score (Stroke Risk after TIA) | 2-day stroke risk after a transient ischaemic attack. Note NICE NG128 advises against using ABCD2 to guide referral urgency. | `neurology`, `emergency`, `risk` |
| `abpi` | ABPI (Ankle-Brachial Pressure Index) | Ankle-Brachial Pressure Index per leg from ankle and brachial systolic pressures; screens for peripheral arterial disease and informs compression-therapy safety. | `primary-care`, `vascular` |
| `acq` | ACQ (Asthma Control Questionnaire) | Asthma control monitoring (BTS/NICE/SIGN asthma). | `respiratory`, `severity`, `proprietary`, `unavailable` |
| `alcohol_units` | Alcohol Units (UK) | Calculates UK alcohol units from drink volume and ABV, with optional weekly tally and alcohol kcal. | `primary-care`, `mental-health` |
| `alvarado` | Alvarado Score (Appendicitis) | MANTRELS score for suspected acute appendicitis (0-10). | `emergency`, `surgery`, `risk` |
| `amts` | Abbreviated Mental Test Score (AMTS) | Ten-item bedside cognitive screen (0-10); a score below 8 suggests cognitive impairment. | `primary-care`, `geriatrics`, `neurology`, `screening` |
| `anion_gap` | Anion Gap | Serum anion gap from sodium, chloride, bicarbonate, optional potassium, and optional albumin correction. | `acute-medicine`, `nephrology` |
| `apache2` | APACHE II | ICU acute physiology, age, and chronic health severity score (0-71). | `intensive-care`, `severity`, `prognostic` |
| `apgar` | Apgar Score | Scores one newborn observation set at 1, 5, 10, 15, or 20 minutes. It documents clinical status and response but does not determine initial resuscitation or diagnose asphyxia. | `paediatrics`, `perinatal`, `severity` |
| `asa_physical_status` | ASA Physical Status | ASA preoperative physical-status classification, with optional emergency suffix. | `surgery`, `severity` |
| `ascvd` | ASCVD Pooled Cohort Equations | ACC/AHA 2013 10-year ASCVD risk estimate for US adults aged 40-79. | `primary-care`, `cardiology`, `risk` |
| `asrs` | ASRS-v1.1 Six-Question Adult ADHD Screener | Scores six coded adult responses covering the past six months from the authorised form, reporting the classic and continuous methods separately; questionnaire text is not bundled. | `primary-care`, `mental-health`, `screening` |
| `audit` | AUDIT Alcohol Use Screen | Ten-item WHO alcohol-use screen (0-40); four risk zones from low risk to possible dependence. | `primary-care`, `mental-health`, `screening` |
| `auditc` | AUDIT-C Alcohol Consumption Screen | Three-item WHO AUDIT consumption subscale (0-12); positive at 4+ (men) or 3+ (women). | `primary-care`, `mental-health`, `screening` |
| `barthel` | Barthel Index | Activities of daily living score (0-100) from ten functional domains. | `geriatrics`, `severity` |
| `basdai` | BASDAI | Bath Ankylosing Spondylitis Disease Activity Index (0-10). | `rheumatology`, `severity` |
| `binet` | Binet Stage for Chronic Lymphocytic Leukaemia | Derives historical CLL stage A, B, or C from five physical-examination lymphoid areas, haemoglobin, and platelet count. It is not a treatment instruction. | `oncology`, `severity`, `prognostic` |
| `bmi` | BMI (Body Mass Index) | Body mass index from weight and height, with standard adult category. | `primary-care`, `endocrinology` |
| `bode` | BODE Index (COPD prognosis) | Multidimensional prognostic index in COPD from BMI, FEV1, mMRC dyspnoea, and six-minute walk distance; predicts ~4-year survival. | `respiratory`, `prognostic` |
| `body_fat_circumference` | Body Fat % (US Navy Circumference Method) | Estimates body fat percentage from height, waist, neck (and hip for women) using the US Navy / Hodgdon-Beckett regression equations. | `primary-care`, `endocrinology`, `screening` |
| `body_surface_area` | Body Surface Area (Mosteller) | Calculates body surface area from height and weight using the Mosteller formula. It does not infer dosing or indexing decisions, which remain protocol-specific. | `primary-care` |
| `braden` | Braden Scale (Pressure Ulcer Risk) | Predicts pressure ulcer risk across six subscales (sensory perception, moisture, activity, mobility, nutrition, friction/shear). Score 6-23; lower = higher risk. | `acute-medicine`, `geriatrics`, `screening` |
| `caprini` | Caprini VTE Risk Score | Peri-operative venous thromboembolism (VTE) risk assessment using weighted risk factors. Score 0=very low to ≥5=high. | `surgery`, `vascular`, `risk` |
| `cat` | CAT (COPD Assessment Test) | Symptom-burden / health-status measure in COPD (8 items, 0-40; GOLD/NICE NG115). | `respiratory`, `severity`, `proprietary`, `unavailable` |
| `centor` | Centor / McIsaac Score (Strep Pharyngitis) | Predicts probability of group-A streptococcal pharyngitis to guide antibiotic use and throat swab decisions. | `primary-care`, `infectious-diseases` |
| `cfs` | CFS (Clinical Frailty Scale) | 9-point judgement-based frailty grading in older adults (1 Very Fit to 9 Terminally Ill). | `geriatrics`, `severity`, `proprietary`, `unavailable` |
| `cha2ds2_va` | CHA₂DS₂-VA (2024 ESC) | Stroke risk in atrial fibrillation using the 2024 ESC sex-free update (score 0-8). | `cardiology`, `risk` |
| `cha2ds2vasc` | CHA2DS2-VASc Stroke Risk (AF) | Stroke risk in non-valvular atrial fibrillation, guiding anticoagulation (NICE NG196). | `cardiology`, `risk` |
| `chalice` | CHALICE Paediatric Head Injury Rule | Decision rule for CT head in children after head injury: any positive criterion predicts a clinically significant intracranial injury and a CT head scan is recommended (Dunning et al 2006; NICE NG232). | `paediatrics`, `emergency` |
| `charlson` | Charlson Comorbidity Index (CCI) | Predicts 10-year mortality from 19 weighted comorbidities, with optional age adjustment. | `primary-care`, `risk`, `prognostic` |
| `child_pugh` | Child-Pugh Score (Cirrhosis Severity) | Severity of chronic liver disease from bilirubin, albumin, INR, ascites, and encephalopathy; reports class A/B/C. | `hepatology`, `severity` |
| `ciwa_ar` | CIWA-Ar Alcohol Withdrawal Severity | Quantifies current withdrawal symptoms after alcohol withdrawal has been clinically identified in a reliably communicative patient. It is not diagnostic and does not prescribe medication. | `acute-medicine`, `mental-health`, `severity` |
| `ckd_risk` | KDIGO CKD risk category (eGFR x ACR heatmap) | Combines the eGFR G-stage and albuminuria A-stage into the KDIGO prognosis risk category (the green/yellow/orange/red heatmap). | `primary-care`, `nephrology`, `risk` |
| `corrected_calcium` | Albumin-corrected Calcium | Corrects total serum calcium for abnormal albumin using the Payne-style correction. | `primary-care`, `endocrinology` |
| `corrected_sodium` | Hyperglycaemia-corrected Sodium | Estimates expected serum sodium at normoglycaemia in hyperglycaemia (DKA/HHS workup), using the Katz or Hillier correction factor. | `endocrinology`, `acute-medicine` |
| `cows` | Clinical Opiate Withdrawal Scale (COWS) | Quantifies current clinician-assessed findings apparently attributable to opioid withdrawal. It does not independently diagnose withdrawal or determine medication timing or dosage. | `acute-medicine`, `mental-health`, `severity` |
| `curb65` | CURB-65 Pneumonia Severity | Severity and 30-day mortality risk in adults with community-acquired pneumonia, guiding place of care (BTS 2009 / NICE NG250). | `acute-medicine`, `respiratory`, `infectious-diseases`, `severity` |
| `das28` | DAS28 (Rheumatoid Arthritis Disease Activity) | Disease Activity Score in 28 joints for rheumatoid arthritis, from tender/swollen joint counts, an ESR or CRP marker, and patient global health. | `rheumatology`, `severity` |
| `cockcroft_gault` | Cockcroft-Gault Creatinine Clearance | Creatinine clearance (CrCl, mL/min) from age, weight, sex, and creatinine. Superseded by CKD-EPI 2021 for CKD staging; retained for renal drug dosing where guidelines cite CrCl. | `primary-care`, `nephrology` |
| `egfr` | eGFR (CKD-EPI 2021) | Estimated glomerular filtration rate from creatinine (race-free CKD-EPI 2021); reports CKD G-stage. | `primary-care`, `nephrology` |
| `ehra` | EHRA AF Symptom Classification | Classifies atrial fibrillation symptom burden (Classes 1, 2a, 2b, 3, 4) to guide rhythm-control decisions. | `cardiology` |
| `elf` | ELF (Enhanced Liver Fibrosis test) | Second-line serum biomarker test for liver fibrosis (NICE NG49). | `hepatology`, `screening`, `proprietary`, `unavailable` |
| `energy_requirement` | Energy Requirement (BMR/RMR/TDEE) | Adult basal/resting energy estimate using Mifflin-St Jeor, Harris-Benedict original/revised, Schofield, or Cunningham, with optional activity factor and kcal target adjustment. | `primary-care`, `endocrinology` |
| `epds` | Edinburgh Postnatal Depression Scale (EPDS) | Ten-item perinatal depression screen (0-30); >=10 possible, >=13 probable; item 10 flags self-harm risk. | `primary-care`, `mental-health`, `perinatal`, `screening` |
| `euroscore2` | EuroSCORE II (Cardiac Surgery Mortality) | Predicted operative mortality after cardiac surgery from 18 preoperative factors (Nashef 2012). | `cardiology`, `surgery`, `prognostic` |
| `familial_hypercholesterolaemia` | Familial Hypercholesterolaemia (Simon Broome + DLCN) | Diagnoses familial hypercholesterolaemia using both the Simon Broome (UK) and Dutch Lipid Clinic Network (DLCN) criteria. | `primary-care`, `cardiology`, `endocrinology`, `risk` |
| `fena` | FENa (Fractional Excretion of Sodium) | Differentiates pre-renal from intrinsic renal failure using urine and plasma sodium and creatinine. | `acute-medicine`, `nephrology` |
| `feverpain` | FeverPAIN Score | Five-item score guiding antibiotic prescribing in acute sore throat (validated for adults and children aged 3+). | `primary-care`, `infectious-diseases`, `respiratory` |
| `fib4` | FIB-4 Liver Fibrosis Index | Non-invasive screen for advanced liver fibrosis from age, AST, ALT, and platelets (NICE NG49). | `primary-care`, `hepatology`, `screening` |
| `findrisc` | FINDRISC (Finnish Diabetes Risk Score) | Predicts 10-year risk of type 2 diabetes using 8 lifestyle and clinical items (score 0-26). | `primary-care`, `endocrinology`, `screening` |
| `fourat` | 4AT Rapid Delirium Screening | Rapid bedside screen for delirium and cognitive impairment (four items, score 0-12). | `acute-medicine`, `geriatrics`, `neurology`, `screening` |
| `four_ts` | 4Ts Score for Heparin-Induced Thrombocytopenia | Estimates HIT pretest probability from four clinician-resolved domains using the standard days 5-10 timing variant. It is not diagnostic and does not autonomously select treatment. | `haematology`, `risk` |
| `free_water_deficit` | Free-water Deficit | Estimates the static free-water deficit in an adult with hypernatraemia using an explicit sodium target and clinician-selected total-body-water fraction. It is not a fluid prescription or correction rate. | `acute-medicine`, `nephrology` |
| `frax` | FRAX (10-year fracture risk) | 10-year probability of osteoporotic and hip fracture (NICE CG146). | `endocrinology`, `musculoskeletal`, `risk`, `proprietary`, `unavailable` |
| `gad7` | GAD-7 Anxiety Severity | Seven-item generalised anxiety severity score (0-21); a total of 10+ flags likely GAD. | `primary-care`, `mental-health`, `screening` |
| `gcs` | Glasgow Coma Scale (GCS) | Bedside score (3-15) of conscious level from eye, verbal, and motor response (Teasdale & Jennett 1974); omits the total and band when any component is not testable, per current guidance. | `neurology`, `emergency`, `acute-medicine`, `severity` |
| `glasgow_blatchford` | Glasgow-Blatchford Bleeding Score (GBS) | Pre-endoscopy risk assessment at first presentation with acute upper gastrointestinal bleeding; distinguishes NICE's score-0 early-discharge consideration from ACG's score-0-to-1 very-low-risk example without making disposition automatic. | `emergency`, `acute-medicine`, `risk` |
| `gleason` | Gleason Grade Group (ISUP/WHO) | Gleason score and ISUP/WHO Grade Group (1-5) from the primary and secondary prostate cancer patterns. | `oncology`, `urology` |
| `grace` | GRACE ACS Risk Score (in-hospital mortality) | Point-based GRACE 1.0 score (Granger 2003) estimating in-hospital mortality risk in acute coronary syndrome. | `cardiology`, `acute-medicine`, `prognostic` |
| `hasbled` | HAS-BLED Bleeding Risk (AF) | Bleeding risk in atrial fibrillation on anticoagulation, used alongside CHA2DS2-VASc (NICE NG196). | `cardiology`, `risk` |
| `heart` | HEART Score (ED Chest Pain) | 6-week MACE risk for emergency department chest pain, guiding discharge versus admission versus early invasive management (Six AJ et al. 2008). | `cardiology`, `emergency`, `risk` |
| `hinchey` | Modified Hinchey Classification (Wasvary/Kaiser) | Classifies diverticulitis anatomy using the Wasvary modified stages and Kaiser CT operationalisation; preserves fistula/obstruction as separate categories and does not infer purulent versus fecal peritonitis from CT alone. | `surgery`, `acute-medicine`, `severity` |
| `homa_ir` | HOMA-IR | Calculates the continuous original HOMA1 surrogate estimate from fasting glucose and fasting insulin. It does not apply a universal diagnostic cutoff because values depend on the assay and population. | `endocrinology`, `screening` |
| `ipss` | IPSS - International Prostate Symptom Score | Seven-item lower urinary tract symptom score (0-35) for benign prostatic hyperplasia; bands mild 0-7, moderate 8-19, severe 20-35, with an optional quality-of-life item (0-6). | `urology`, `severity` |
| `isth_overt_dic` | ISTH Overt DIC Score (2025) | Derives the current overt/late-phase DIC score from same-episode measurements in a patient with a recognised DIC-associated underlying disorder. It supports but does not establish or exclude the diagnosis. | `acute-medicine`, `haematology`, `severity` |
| `khorana` | Khorana Score for Chemotherapy-Associated VTE | Estimates short-term symptomatic VTE risk before a new chemotherapy regimen in adult ambulatory patients. The original bands and modern score-2 prophylaxis-assessment threshold are reported separately. | `oncology`, `haematology`, `risk` |
| `lanss` | LANSS (Leeds Assessment of Neuropathic Symptoms and Signs) | Screening for pain of predominantly neuropathic origin (7 items, 0-24; >=12 likely neuropathic). | `neurology`, `screening`, `proprietary`, `unavailable` |
| `ldl_cholesterol` | LDL and non-HDL Cholesterol | Calculates non-HDL cholesterol and estimates LDL cholesterol using Friedewald, Martin-Hopkins, and Sampson-NIH, with method-specific triglyceride limits. | `primary-care`, `cardiology`, `endocrinology` |
| `lrinec` | LRINEC Score (Necrotising Fasciitis Risk Indicator) | Six-variable laboratory diagnostic adjunct for severe soft-tissue infection; a score of 6 or more increases suspicion, but poor sensitivity means a low score does not exclude necrotising infection or justify delaying surgical consultation. | `emergency`, `surgery`, `infectious-diseases`, `risk` |
| `max_heart_rate` | Max Heart Rate & Training Zones | Estimates HRmax from age (Tanaka 2001) and derives aerobic training zones; uses Karvonen heart-rate reserve when resting HR is supplied. | `primary-care`, `screening` |
| `meld` | MELD Score (original, 2001) | Model for End-Stage Liver Disease: 3-month mortality risk from bilirubin, INR, and creatinine (Kamath 2001). | `hepatology`, `prognostic` |
| `meld_3` | MELD 3.0 (OPTN) | Current OPTN MELD 3.0 allocation score for liver-transplant candidates registered at age 12 or older; preserves the uncapped policy-formula score separately and does not determine official allocation. | `hepatology`, `prognostic` |
| `mmse` | MMSE (Mini-Mental State Examination) | Cognitive screening / dementia monitoring (NICE NG97). | `geriatrics`, `neurology`, `mental-health`, `screening`, `proprietary`, `unavailable` |
| `mrc_dyspnoea` | MRC Dyspnoea Scale | Grades breathlessness-related disability on the classic MRC 1-5 scale (Fletcher 1959; NICE/BTS UK usage). | `primary-care`, `respiratory` |
| `must` | MUST (Malnutrition Universal Screening Tool) | Malnutrition risk screening (NICE CG32). | `primary-care`, `screening`, `proprietary`, `unavailable` |
| `news2` | NEWS2 (National Early Warning Score 2) | NHS-mandated aggregate physiology score (RCP 2017) driving the clinical-response band. | `acute-medicine`, `nhs-mandated`, `severity` |
| `nhfs` | Nottingham Hip Fracture Score (NHFS) | Preoperative score (0-10) predicting 30-day mortality after hip fracture surgery. | `surgery`, `geriatrics`, `musculoskeletal`, `prognostic` |
| `npi` | Nottingham Prognostic Index (NPI) | Prognosis in primary operable breast cancer from invasive tumour size, lymph node stage, and histological grade; reports the prognostic group. | `oncology`, `prognostic` |
| `nyha` | NYHA Functional Classification | Classifies heart-failure functional capacity (Class I-IV) by how much physical activity provokes fatigue, palpitation, dyspnoea, or anginal pain. | `cardiology`, `severity` |
| `ohs` | Oxford Hip Score (OHS) | Patient-reported outcome after hip replacement (NHS England PROMs). | `surgery`, `musculoskeletal`, `proprietary`, `unavailable` |
| `oks` | Oxford Knee Score (OKS) | Patient-reported outcome after knee replacement (NHS England PROMs). | `surgery`, `musculoskeletal`, `proprietary`, `unavailable` |
| `one_rep_max` | One-Rep Max Estimator | Estimates 1RM from a submaximal weight and reps using Epley, Brzycki, or Lombardi. | `primary-care`, `musculoskeletal` |
| `padua` | Padua Prediction Score (VTE risk) | VTE risk in hospitalised medical inpatients, guiding thromboprophylaxis (NICE NG89). | `acute-medicine`, `vascular`, `risk` |
| `perc` | PERC Rule (PE Rule-out Criteria) | Eight-item block rule for stable adult emergency-department outpatients with suspected pulmonary embolism and clinician gestalt below 15%; PERC-negative can avoid further PE-specific testing. | `emergency`, `respiratory`, `vascular` |
| `phq9` | PHQ-9 Depression Severity | Nine-item depression severity score (0-27) with standard bands; item 9 flags self-harm risk. | `primary-care`, `mental-health`, `screening` |
| `psa_density` | Prostate-specific Antigen Density | Calculates total serum PSA divided by imaging-derived prostate volume as a continuous prostate-cancer risk modifier, without imposing a universal diagnostic or biopsy cutoff. | `urology`, `oncology` |
| `qfracture` | QFracture (10-year fracture risk) | 10-year risk of major osteoporotic and hip fracture (QFracture-2012), the open UK alternative to FRAX (NICE CG146/NG6). | `primary-care`, `endocrinology`, `risk` |
| `qrisk3` | QRISK3 (10-year cardiovascular risk) | 10-year risk of heart attack or stroke (QRISK3-2017), the UK standard for primary CVD risk assessment (NICE NG238). | `primary-care`, `cardiology`, `risk` |
| `qsofa` | qSOFA Score (Sepsis-3) | Quick bedside prompt flagging suspected-infection patients at higher risk of poor outcome (Sepsis-3). A prognostic prompt, not a diagnosis of sepsis. | `acute-medicine`, `intensive-care`, `screening` |
| `rcri` | Revised Cardiac Risk Index (RCRI) | Six-factor Lee index for in-hospital major cardiac complications after noncardiac surgery, used within stepwise adult preoperative assessment; historical event rates are shown only when the original validation population matches. | `cardiology`, `surgery`, `risk` |
| `relative_fat_mass` | Relative Fat Mass (RFM) | Sex-specific estimate of whole-body fat percentage from height and source-protocol waist circumference (Woolcott-Bergman 2018), evaluated against DXA in US adults aged 20-69; accuracy decreased with age and was lower at lower body-fat levels. It does not provide a diagnostic obesity cut-point. | `primary-care`, `endocrinology`, `screening` |
| `sofa` | SOFA Score (Sequential Organ Failure Assessment) | Grades dysfunction across six organ systems (0-24); underpins the Sepsis-3 definition (rise >= 2 from baseline). | `intensive-care`, `severity` |
| `timi` | TIMI Risk Score for UA/NSTEMI | 14-day risk of death, MI, or urgent revascularisation in unstable angina / NSTEMI (Antman et al, JAMA 2000). Not the STEMI score. | `cardiology`, `acute-medicine`, `risk` |
| `uacr` | uACR (urine albumin-to-creatinine ratio) | Urine albumin-to-creatinine ratio from a measured ratio or raw albumin/creatinine; reports the KDIGO albuminuria category (A1-A3). | `primary-care`, `nephrology` |
| `ukeld` | UKELD (UK Model for End-Stage Liver Disease) | UK liver-transplant listing score from INR, creatinine, bilirubin, and sodium (Barber 2011); 49 is the listing threshold. | `hepatology`, `prognostic` |
| `waist_to_height_ratio` | Waist-to-Height Ratio (WHtR) | Unitless central-adiposity index: waist circumference divided by height. Boundary 0.5 is the "keep your waist to less than half your height" rule. | `primary-care`, `endocrinology`, `screening` |
| `waist_to_hip_ratio` | Waist-to-Hip Ratio (WHR) | Adult waist circumference divided by hip circumference, compared with commonly WHO-attributed cut-offs (>= 0.90 men, >= 0.85 women); the WHO report cautions that cut-offs are not universal. | `primary-care`, `endocrinology`, `screening` |
| `waterlow` | Waterlow Score (Pressure Ulcer Risk) | Bedside pressure-ulcer (pressure-injury) risk assessment: summed weighted categories (10+ at risk, 15+ high, 20+ very high). | `acute-medicine`, `geriatrics`, `screening` |
| `wells_dvt` | Wells Score (DVT) | Clinical pre-test probability of deep vein thrombosis, guiding ultrasound vs D-dimer (NICE NG158). | `emergency`, `vascular` |
| `wells_pe` | Wells Score for Pulmonary Embolism | Pretest probability of pulmonary embolism, guiding D-dimer vs CTPA (NICE NG158). | `emergency`, `respiratory`, `vascular` |
| `wilks` | Wilks Score | Historical bodyweight-adjusted powerlifting score for bench press or three-lift total using the IPF-era Wilks coefficient; the IPF replaced Wilks with IPF Points in 2019. | `musculoskeletal` |

## Unavailable on principle

A handful of widely-used clinical tools are licence-locked or proprietary. They are tagged `proprietary` + `unavailable` in the table above. Invoking any of them returns a structured "unavailable" response, never a score:

```console
$ clincalc calc frax --input '{}'
frax = unavailable: proprietary

FRAX (10-year fracture risk) is not available here because it is proprietary or licence-locked. Owner: University of Sheffield (Centre for Metabolic Bone Diseases). The FRAX algorithm and its country-specific coefficients are a trade secret and have never been published, so it cannot be reimplemented from primary literature. ...
```

The point is to make the *gap* a first-class object. Where an open alternative exists in this catalogue, it is named in the response (e.g. `qfracture` for FRAX, `amts` and `fourat` for MMSE).

See [Why some calculators are unavailable](how-it-works.md#unavailable-on-principle) for the rationale.

## Wishlist (candidates for future addition)

Calculators below are clinically valuable and on the radar but not yet implemented. They include remaining entries from the 23-calculator gap recorded against [MedikQuantis](https://medikquantis.me) (Laura Piñero Roig, Barcelona, MIT) at upstream commit [`16c63c85`](https://github.com/laurapiro17/medikquantis/tree/16c63c85aee7a64417205f60cb3e66fccf19fae2), plus candidates from other sources. The complete governed queue and stable identifiers live in [`spec/calculator-roadmap.md`](https://github.com/pacharanero/clincalc/blob/main/spec/calculator-roadmap.md).

Contributions welcome. The shape of the work is documented in [How it works](how-it-works.md#embedding-clincalc-in-a-host), [`AGENTS.md`](https://github.com/pacharanero/clincalc/blob/main/AGENTS.md), and the [`spec/`](https://github.com/pacharanero/clincalc/tree/main/spec) and [`examples/`](https://github.com/pacharanero/clincalc/tree/main/examples) directories.

| Candidate | What it does | Tentative tags |
|---|---|---|
| **StatinMD** (Oxford STRATIFY) | Personalised 1/5/10-year risk of serious statin-induced muscle disorders (rhabdomyolysis / hospitalised myopathy) from 22 routinely-recorded factors (Cai et al, *Lancet Digital Health* 2026). Natural pairing with QRISK3: QRISK3 is the benefit side, StatinMD is the harm side. Licensed for **academic use** via Oxford University Innovation - covered while this project is non-commercial. ([source](https://process.innovation.ox.ac.uk/software/p/25396/stratify---stainmd-risk-calculator---academic-use/1)) | `primary-care`, `cardiology`, `risk` |
| **NIHSS** | Acute stroke severity standard. | `neurology`, `emergency`, `severity` |
| **Norton Scale** | Pressure-ulcer risk; complements the shipped Braden and Waterlow tools. | `geriatrics`, `screening` |
| **Pitt Bacteraemia** | BSI severity. | `infectious-diseases`, `severity` |
| **Modified Duke criteria** | Endocarditis. | `infectious-diseases` |
| **PASI**, **SCORAD** | Psoriasis / atopic dermatitis. | `dermatology`, `severity` |
| **ORBIT** | Bleeding risk in atrial fibrillation; blocked pending unrestricted redistribution evidence because the original article is CC BY-NC. | `cardiology`, `risk` |
| **SCORE2 / SCORE2-OP** | ESC 2021 CV risk (verify licensing). | `cardiology`, `risk` |
| **ARDSNet adult predicted body weight** | Protocol-specific adult PBW for lung-protective ventilation, reframed from the ambiguous combined BMI / BSA / ideal-body-weight proposal. | `intensive-care`, `respiratory` |
| **SAD PERSONS** | Legacy suicide-risk checklist. Poor predictive performance requires prominent safeguards against using it for disposition decisions. | `mental-health`, `risk` |
| **RCPCH Digital Growth Charts** | UK-WHO + UK90; z-score / centile / SDS; chart rendering. Needs LMS tables + RCPCH licensing terms. | `paediatrics` |
| **Skinfold body fat % (Jackson-Pollock / Durnin-Womersley)** | Caliper-derived body-fat estimate for training and body-comparison settings. | `primary-care`, `endocrinology`, `screening` |
| **Body adiposity index (BAI)** | %BF proxy from height and hip circumference; population-specific (Hispanic-origin calibration). | `primary-care`, `endocrinology`, `screening` |
| **Fat-free mass index (FFMI)** | Fat-free mass normalised to height^2; used in sports medicine and sarcopenia screening. | `primary-care`, `endocrinology`, `screening` |
| **Skeletal muscle mass index (SMI)** | Appendicular lean mass / height^2; sarcopenia definition (EWGSOP2 / FNIH). | `geriatrics`, `endocrinology`, `screening` |
| **Protein / macronutrient target** | g/day from weight or LBM + goal (e.g. 1.6-2.2 g/kg for lean-mass retention in a deficit). | `primary-care`, `endocrinology` |
| **Axial length centile charts (CREAM-Kids)** | Age-, sex-, and region-specific centile charts for axial eye length in children and adolescents (Kneepkens, Lingham, Mackey et al, *JAMA Ophthalmology* 2026; [DOI: 10.1001/jamaophthalmol.2026.2539](https://doi.org/10.1001/jamaophthalmol.2026.2539)). Reusable coefficients/data and their distribution terms must be confirmed before implementation; serial change needs a separately cited model. | `ophthalmology`, `paediatrics`, `screening` |

For every MedikQuantis parity calculator, implementation and complete Catalan/Spanish adaptation are one roadmap workstream. The independently verified English calculation may land first when review availability blocks translation, but the parity item remains partial and the non-English locale is not advertised until the review and completeness gates in [`spec/multilingual.md`](https://github.com/pacharanero/clincalc/blob/main/spec/multilingual.md) pass.
