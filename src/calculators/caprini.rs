// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Caprini VTE Risk Assessment Model.
//!
//! Weighted risk-factor scoring for peri-operative venous thromboembolism (VTE).
//! Based on Caprini (2005) with subsequent validation (Bahl et al. 2010).
//!
//! Risk bands:
//!  - Score 0:   Very low  (~0.5% VTE risk without prophylaxis)
//!  - Score 1-2: Low       (~1.5%)
//!  - Score 3-4: Moderate  (~3%)
//!  - Score ≥5:  High      (~6% or more)

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "caprini";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1016/j.dmon.2005.02.003",
};

pub const REFERENCE: &str = "Caprini JA. Thrombosis risk assessment as a guide to quality patient care. \
Dis Mon. 2005;51(2-3):70-78. doi:10.1016/j.dmon.2005.02.003 | \
Bahl V et al. A validation study of a retrospective venous thromboembolism risk scoring method. \
Ann Surg. 2010;251(2):344-350. doi:10.1097/SLA.0b013e3181b7fca6";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapriniInput {
    // --- 1-point risk factors ---
    /// Age 41-60 years
    pub age_41_60: bool,
    /// Minor surgery planned
    pub minor_surgery: bool,
    /// BMI > 25 kg/m²
    pub bmi_over_25: bool,
    /// Swollen legs (current)
    pub swollen_legs: bool,
    /// Varicose veins
    pub varicose_veins: bool,
    /// Pregnancy or postpartum (within 1 month)
    pub pregnancy_or_postpartum: bool,
    /// History of unexplained or recurrent spontaneous abortion
    pub history_of_miscarriage: bool,
    /// Oral contraceptives or hormone replacement therapy
    pub oral_contraceptives_or_hrt: bool,
    /// Sepsis within 1 month
    pub sepsis_within_1_month: bool,
    /// Serious lung disease including pneumonia within 1 month
    pub serious_lung_disease: bool,
    /// Abnormal pulmonary function (COPD)
    pub abnormal_pulmonary_function: bool,
    /// Acute myocardial infarction
    pub acute_mi: bool,
    /// Congestive heart failure within 1 month
    pub congestive_heart_failure: bool,
    /// History of inflammatory bowel disease
    pub inflammatory_bowel_disease: bool,
    /// Patient confined to bed (medical patient currently at bed rest)
    pub medical_patient_at_bed_rest: bool,

    // --- 2-point risk factors ---
    /// Age 61-74 years
    pub age_61_74: bool,
    /// Arthroscopic surgery
    pub arthroscopic_surgery: bool,
    /// Malignancy (present or previous)
    pub malignancy: bool,
    /// Major surgery > 45 minutes
    pub major_surgery_over_45_min: bool,
    /// Laparoscopic surgery > 45 minutes
    pub laparoscopic_surgery_over_45_min: bool,
    /// Patient confined to bed > 72 hours
    pub confined_to_bed_over_72h: bool,
    /// Immobilising plaster cast
    pub immobilising_cast: bool,
    /// Central venous access
    pub central_venous_access: bool,

    // --- 3-point risk factors ---
    /// Age ≥ 75 years
    pub age_75_plus: bool,
    /// History of DVT or PE
    pub history_dvt_pe: bool,
    /// Family history of VTE
    pub family_history_vte: bool,
    /// Factor V Leiden mutation
    pub factor_v_leiden: bool,
    /// Prothrombin 20210A mutation
    pub prothrombin_20210a: bool,
    /// Lupus anticoagulant
    pub lupus_anticoagulant: bool,
    /// Elevated anticardiolipin antibodies
    pub anticardiolipin_antibodies: bool,
    /// Elevated serum homocysteine
    pub elevated_homocysteine: bool,
    /// Heparin-induced thrombocytopenia (HIT)
    pub heparin_induced_thrombocytopenia: bool,
    /// Other congenital or acquired thrombophilia
    pub other_thrombophilia: bool,

    // --- 5-point risk factors ---
    /// Stroke within 1 month
    pub stroke_within_1_month: bool,
    /// Elective major lower extremity arthroplasty
    pub elective_arthroplasty: bool,
    /// Hip, pelvis, or leg fracture within 1 month
    pub hip_pelvis_leg_fracture: bool,
    /// Acute spinal cord injury / paralysis within 1 month
    pub acute_spinal_cord_injury: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapriniOutcome {
    pub score: u8,
    pub risk_level: &'static str,
    pub interpretation: String,
}

pub fn compute(input: &CapriniInput) -> Result<CapriniOutcome, CalcError> {
    let mut score: u8 = 0;

    // 1-point
    if input.age_41_60 {
        score += 1;
    }
    if input.minor_surgery {
        score += 1;
    }
    if input.bmi_over_25 {
        score += 1;
    }
    if input.swollen_legs {
        score += 1;
    }
    if input.varicose_veins {
        score += 1;
    }
    if input.pregnancy_or_postpartum {
        score += 1;
    }
    if input.history_of_miscarriage {
        score += 1;
    }
    if input.oral_contraceptives_or_hrt {
        score += 1;
    }
    if input.sepsis_within_1_month {
        score += 1;
    }
    if input.serious_lung_disease {
        score += 1;
    }
    if input.abnormal_pulmonary_function {
        score += 1;
    }
    if input.acute_mi {
        score += 1;
    }
    if input.congestive_heart_failure {
        score += 1;
    }
    if input.inflammatory_bowel_disease {
        score += 1;
    }
    if input.medical_patient_at_bed_rest {
        score += 1;
    }

    // 2-point
    if input.age_61_74 {
        score += 2;
    }
    if input.arthroscopic_surgery {
        score += 2;
    }
    if input.malignancy {
        score += 2;
    }
    if input.major_surgery_over_45_min {
        score += 2;
    }
    if input.laparoscopic_surgery_over_45_min {
        score += 2;
    }
    if input.confined_to_bed_over_72h {
        score += 2;
    }
    if input.immobilising_cast {
        score += 2;
    }
    if input.central_venous_access {
        score += 2;
    }

    // 3-point
    if input.age_75_plus {
        score += 3;
    }
    if input.history_dvt_pe {
        score += 3;
    }
    if input.family_history_vte {
        score += 3;
    }
    if input.factor_v_leiden {
        score += 3;
    }
    if input.prothrombin_20210a {
        score += 3;
    }
    if input.lupus_anticoagulant {
        score += 3;
    }
    if input.anticardiolipin_antibodies {
        score += 3;
    }
    if input.elevated_homocysteine {
        score += 3;
    }
    if input.heparin_induced_thrombocytopenia {
        score += 3;
    }
    if input.other_thrombophilia {
        score += 3;
    }

    // 5-point
    if input.stroke_within_1_month {
        score += 5;
    }
    if input.elective_arthroplasty {
        score += 5;
    }
    if input.hip_pelvis_leg_fracture {
        score += 5;
    }
    if input.acute_spinal_cord_injury {
        score += 5;
    }

    let (risk_level, recommendation) = match score {
        0 => (
            "Very low",
            "Early ambulation; mechanical prophylaxis not routinely required.",
        ),
        1..=2 => (
            "Low",
            "Mechanical prophylaxis (intermittent pneumatic compression). \
Pharmacological prophylaxis generally not indicated.",
        ),
        3..=4 => (
            "Moderate",
            "Mechanical prophylaxis; consider pharmacological prophylaxis \
(LMWH or UFH) after weighing bleeding risk.",
        ),
        _ => (
            "High",
            "Combined mechanical and pharmacological prophylaxis (LMWH or UFH). \
Extended prophylaxis post-discharge may be indicated.",
        ),
    };

    let interpretation = format!(
        "Caprini VTE score {score} - {risk_level} risk. {recommendation} \
(Prophylaxis decisions must also account for individual bleeding risk and institutional protocols.)"
    );

    Ok(CapriniOutcome {
        score,
        risk_level,
        interpretation,
    })
}

pub fn build_response(input: &CapriniInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("caprini_score".into(), json!(o.score));
    working.insert("risk_level".into(), json!(o.risk_level));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Caprini;

impl Calculator for Caprini {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Caprini VTE Risk Score"
    }

    fn description(&self) -> &'static str {
        "Peri-operative venous thromboembolism (VTE) risk assessment using weighted risk factors. Score 0=very low to ≥5=high."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "CapriniInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age_41_60", "minor_surgery", "bmi_over_25", "swollen_legs",
                "varicose_veins", "pregnancy_or_postpartum", "history_of_miscarriage",
                "oral_contraceptives_or_hrt", "sepsis_within_1_month", "serious_lung_disease",
                "abnormal_pulmonary_function", "acute_mi", "congestive_heart_failure",
                "inflammatory_bowel_disease", "medical_patient_at_bed_rest",
                "age_61_74", "arthroscopic_surgery", "malignancy",
                "major_surgery_over_45_min", "laparoscopic_surgery_over_45_min",
                "confined_to_bed_over_72h", "immobilising_cast", "central_venous_access",
                "age_75_plus", "history_dvt_pe", "family_history_vte",
                "factor_v_leiden", "prothrombin_20210a", "lupus_anticoagulant",
                "anticardiolipin_antibodies", "elevated_homocysteine",
                "heparin_induced_thrombocytopenia", "other_thrombophilia",
                "stroke_within_1_month", "elective_arthroplasty",
                "hip_pelvis_leg_fracture", "acute_spinal_cord_injury"
            ],
            "properties": {
                "age_41_60": { "type": "boolean", "description": "Age 41-60 years (1 pt)" },
                "minor_surgery": { "type": "boolean", "description": "Minor surgery planned (1 pt)" },
                "bmi_over_25": { "type": "boolean", "description": "BMI > 25 kg/m² (1 pt)" },
                "swollen_legs": { "type": "boolean", "description": "Swollen legs (current) (1 pt)" },
                "varicose_veins": { "type": "boolean", "description": "Varicose veins (1 pt)" },
                "pregnancy_or_postpartum": { "type": "boolean", "description": "Pregnancy or postpartum within 1 month (1 pt)" },
                "history_of_miscarriage": { "type": "boolean", "description": "History of unexplained or recurrent spontaneous abortion (1 pt)" },
                "oral_contraceptives_or_hrt": { "type": "boolean", "description": "Oral contraceptives or HRT (1 pt)" },
                "sepsis_within_1_month": { "type": "boolean", "description": "Sepsis within 1 month (1 pt)" },
                "serious_lung_disease": { "type": "boolean", "description": "Serious lung disease (including pneumonia) within 1 month (1 pt)" },
                "abnormal_pulmonary_function": { "type": "boolean", "description": "Abnormal pulmonary function / COPD (1 pt)" },
                "acute_mi": { "type": "boolean", "description": "Acute myocardial infarction (1 pt)" },
                "congestive_heart_failure": { "type": "boolean", "description": "Congestive heart failure within 1 month (1 pt)" },
                "inflammatory_bowel_disease": { "type": "boolean", "description": "History of inflammatory bowel disease (1 pt)" },
                "medical_patient_at_bed_rest": { "type": "boolean", "description": "Medical patient currently at bed rest (1 pt)" },
                "age_61_74": { "type": "boolean", "description": "Age 61-74 years (2 pts)" },
                "arthroscopic_surgery": { "type": "boolean", "description": "Arthroscopic surgery (2 pts)" },
                "malignancy": { "type": "boolean", "description": "Malignancy (present or previous) (2 pts)" },
                "major_surgery_over_45_min": { "type": "boolean", "description": "Major open surgery > 45 minutes (2 pts)" },
                "laparoscopic_surgery_over_45_min": { "type": "boolean", "description": "Laparoscopic surgery > 45 minutes (2 pts)" },
                "confined_to_bed_over_72h": { "type": "boolean", "description": "Confined to bed > 72 hours (2 pts)" },
                "immobilising_cast": { "type": "boolean", "description": "Immobilising plaster cast (2 pts)" },
                "central_venous_access": { "type": "boolean", "description": "Central venous access (2 pts)" },
                "age_75_plus": { "type": "boolean", "description": "Age ≥ 75 years (3 pts)" },
                "history_dvt_pe": { "type": "boolean", "description": "Personal history of DVT or PE (3 pts)" },
                "family_history_vte": { "type": "boolean", "description": "Family history of VTE (3 pts)" },
                "factor_v_leiden": { "type": "boolean", "description": "Factor V Leiden mutation (3 pts)" },
                "prothrombin_20210a": { "type": "boolean", "description": "Prothrombin 20210A mutation (3 pts)" },
                "lupus_anticoagulant": { "type": "boolean", "description": "Lupus anticoagulant (3 pts)" },
                "anticardiolipin_antibodies": { "type": "boolean", "description": "Elevated anticardiolipin antibodies (3 pts)" },
                "elevated_homocysteine": { "type": "boolean", "description": "Elevated serum homocysteine (3 pts)" },
                "heparin_induced_thrombocytopenia": { "type": "boolean", "description": "Heparin-induced thrombocytopenia (HIT) (3 pts)" },
                "other_thrombophilia": { "type": "boolean", "description": "Other congenital or acquired thrombophilia (3 pts)" },
                "stroke_within_1_month": { "type": "boolean", "description": "Stroke within 1 month (5 pts)" },
                "elective_arthroplasty": { "type": "boolean", "description": "Elective major lower extremity arthroplasty (5 pts)" },
                "hip_pelvis_leg_fracture": { "type": "boolean", "description": "Hip, pelvis, or leg fracture within 1 month (5 pts)" },
                "acute_spinal_cord_injury": { "type": "boolean", "description": "Acute spinal cord injury / paralysis within 1 month (5 pts)" }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: CapriniInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_false() -> CapriniInput {
        CapriniInput {
            age_41_60: false,
            minor_surgery: false,
            bmi_over_25: false,
            swollen_legs: false,
            varicose_veins: false,
            pregnancy_or_postpartum: false,
            history_of_miscarriage: false,
            oral_contraceptives_or_hrt: false,
            sepsis_within_1_month: false,
            serious_lung_disease: false,
            abnormal_pulmonary_function: false,
            acute_mi: false,
            congestive_heart_failure: false,
            inflammatory_bowel_disease: false,
            medical_patient_at_bed_rest: false,
            age_61_74: false,
            arthroscopic_surgery: false,
            malignancy: false,
            major_surgery_over_45_min: false,
            laparoscopic_surgery_over_45_min: false,
            confined_to_bed_over_72h: false,
            immobilising_cast: false,
            central_venous_access: false,
            age_75_plus: false,
            history_dvt_pe: false,
            family_history_vte: false,
            factor_v_leiden: false,
            prothrombin_20210a: false,
            lupus_anticoagulant: false,
            anticardiolipin_antibodies: false,
            elevated_homocysteine: false,
            heparin_induced_thrombocytopenia: false,
            other_thrombophilia: false,
            stroke_within_1_month: false,
            elective_arthroplasty: false,
            hip_pelvis_leg_fracture: false,
            acute_spinal_cord_injury: false,
        }
    }

    #[test]
    fn zero_score_very_low() {
        let o = compute(&all_false()).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.risk_level, "Very low");
    }

    #[test]
    fn one_point_factors_accumulate() {
        // age_41_60 + bmi_over_25 = 2 pts -> Low risk
        let o = compute(&CapriniInput {
            age_41_60: true,
            bmi_over_25: true,
            ..all_false()
        })
        .unwrap();
        assert_eq!(o.score, 2);
        assert_eq!(o.risk_level, "Low");
    }

    #[test]
    fn high_risk_arthroplasty() {
        // Elective arthroplasty alone = 5 pts -> High risk
        let o = compute(&CapriniInput {
            elective_arthroplasty: true,
            ..all_false()
        })
        .unwrap();
        assert_eq!(o.score, 5);
        assert_eq!(o.risk_level, "High");
    }

    #[test]
    fn moderate_risk_band() {
        // age_61_74 (2) + major_surgery (2) = 4 -> Moderate
        let o = compute(&CapriniInput {
            age_61_74: true,
            major_surgery_over_45_min: true,
            ..all_false()
        })
        .unwrap();
        assert_eq!(o.score, 4);
        assert_eq!(o.risk_level, "Moderate");
    }

    #[test]
    fn dvt_history_three_points() {
        let o = compute(&CapriniInput {
            history_dvt_pe: true,
            ..all_false()
        })
        .unwrap();
        assert_eq!(o.score, 3);
        assert_eq!(o.risk_level, "Moderate");
    }
}
