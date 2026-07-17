// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Charlson Comorbidity Index (CCI).
//!
//! Nineteen weighted comorbidities predicting 10-year mortality.
//! Charlson et al. (1987) original weights. Optional age adjustment
//! adds 1 point per decade above 50 (Charlson 1994 update).
//!
//! Weights:
//!  1 point: MI, CHF, PVD, CVD/TIA, dementia, COPD, connective tissue disease,
//!           peptic ulcer, mild liver disease, diabetes (uncomplicated)
//!  2 points: diabetes with end-organ damage, hemiplegia/paraplegia,
//!            CKD moderate-severe, solid tumour (non-metastatic),
//!            leukaemia, lymphoma
//!  3 points: moderate/severe liver disease
//!  6 points: metastatic solid tumour, AIDS/HIV

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "charlson";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1016/0021-9681(87)90171-8",
};

pub const REFERENCE: &str = "Charlson ME, Pompei P, Ales KL, MacKenzie CR. A new method of classifying \
prognostic comorbidity in longitudinal studies: development and validation. J Chronic Dis. \
1987;40(5):373-383. doi:10.1016/0021-9681(87)90171-8";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharlsonInput {
    /// Age in years (used for optional age adjustment: +1 per decade over 50)
    pub age: u8,
    /// Include the age adjustment (+1 per decade over age 50)
    pub include_age_adjustment: bool,

    // --- 1-point conditions ---
    pub myocardial_infarction: bool,
    pub congestive_heart_failure: bool,
    pub peripheral_vascular_disease: bool,
    /// Cerebrovascular disease or TIA
    pub cerebrovascular_disease: bool,
    pub dementia: bool,
    /// Chronic obstructive pulmonary disease
    pub copd: bool,
    /// Connective tissue / rheumatologic disease
    pub connective_tissue_disease: bool,
    pub peptic_ulcer_disease: bool,
    /// Mild liver disease (cirrhosis without portal hypertension, chronic hepatitis)
    pub mild_liver_disease: bool,
    /// Diabetes mellitus without end-organ damage
    pub diabetes_uncomplicated: bool,

    // --- 2-point conditions ---
    /// Diabetes with end-organ damage (retinopathy, neuropathy, nephropathy)
    pub diabetes_complicated: bool,
    /// Hemiplegia or paraplegia
    pub hemiplegia_paraplegia: bool,
    /// Renal disease, moderate or severe (CKD stage 3+, dialysis, transplant)
    pub renal_disease_moderate_severe: bool,
    /// Solid tumour, non-metastatic (treated within 5 years counts)
    pub solid_tumour_non_metastatic: bool,
    pub leukaemia: bool,
    pub lymphoma: bool,

    // --- 3-point condition ---
    /// Moderate or severe liver disease (portal hypertension, varices, encephalopathy)
    pub liver_disease_moderate_severe: bool,

    // --- 6-point conditions ---
    pub metastatic_solid_tumour: bool,
    /// AIDS (not merely HIV-positive)
    pub aids: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharlsonOutcome {
    pub score: u8,
    pub age_adjustment: u8,
    pub total: u8,
    pub interpretation: String,
}

pub fn compute(input: &CharlsonInput) -> Result<CharlsonOutcome, CalcError> {
    let mut score: u8 = 0;

    // 1-point
    if input.myocardial_infarction {
        score += 1;
    }
    if input.congestive_heart_failure {
        score += 1;
    }
    if input.peripheral_vascular_disease {
        score += 1;
    }
    if input.cerebrovascular_disease {
        score += 1;
    }
    if input.dementia {
        score += 1;
    }
    if input.copd {
        score += 1;
    }
    if input.connective_tissue_disease {
        score += 1;
    }
    if input.peptic_ulcer_disease {
        score += 1;
    }
    if input.mild_liver_disease && !input.liver_disease_moderate_severe {
        score += 1;
    }
    if input.diabetes_uncomplicated && !input.diabetes_complicated {
        score += 1;
    }

    // 2-point
    if input.diabetes_complicated {
        score += 2;
    }
    if input.hemiplegia_paraplegia {
        score += 2;
    }
    if input.renal_disease_moderate_severe {
        score += 2;
    }
    if input.solid_tumour_non_metastatic && !input.metastatic_solid_tumour {
        score += 2;
    }
    if input.leukaemia {
        score += 2;
    }
    if input.lymphoma {
        score += 2;
    }

    // 3-point
    if input.liver_disease_moderate_severe {
        score += 3;
    }

    // 6-point
    if input.metastatic_solid_tumour {
        score += 6;
    }
    if input.aids {
        score += 6;
    }

    // Age adjustment: +1 per decade over 50
    let age_adjustment = if input.include_age_adjustment && input.age > 50 {
        (input.age - 51) / 10 + 1
    } else {
        0
    };

    let total = score + age_adjustment;

    let ten_year_survival = match total {
        0 => "~98%",
        1..=2 => "~89%",
        3..=4 => "~77%",
        _ => "~21%",
    };

    let adjustment_note = if input.include_age_adjustment && age_adjustment > 0 {
        format!(" (comorbidity score {score} + age adjustment +{age_adjustment})")
    } else {
        String::new()
    };

    let interpretation = format!(
        "Charlson Comorbidity Index {total}{adjustment_note}. \
Estimated 10-year survival: {ten_year_survival} (original Charlson 1987 cohort). \
CCI is a summary measure; absolute survival estimates should be interpreted with caution \
in contemporary populations."
    );

    Ok(CharlsonOutcome {
        score,
        age_adjustment,
        total,
        interpretation,
    })
}

pub fn build_response(input: &CharlsonInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("comorbidity_score".into(), json!(o.score));
    working.insert("age_adjustment".into(), json!(o.age_adjustment));
    working.insert("cci_total".into(), json!(o.total));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.total),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Charlson;

impl Calculator for Charlson {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Charlson Comorbidity Index (CCI)"
    }

    fn description(&self) -> &'static str {
        "Predicts 10-year mortality from 19 weighted comorbidities, with optional age adjustment."
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
            "title": "CharlsonInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age", "include_age_adjustment",
                "myocardial_infarction", "congestive_heart_failure",
                "peripheral_vascular_disease", "cerebrovascular_disease",
                "dementia", "copd", "connective_tissue_disease",
                "peptic_ulcer_disease", "mild_liver_disease", "diabetes_uncomplicated",
                "diabetes_complicated", "hemiplegia_paraplegia",
                "renal_disease_moderate_severe", "solid_tumour_non_metastatic",
                "leukaemia", "lymphoma", "liver_disease_moderate_severe",
                "metastatic_solid_tumour", "aids"
            ],
            "properties": {
                "age": { "type": "integer", "minimum": 18, "maximum": 120 },
                "include_age_adjustment": {
                    "type": "boolean",
                    "description": "Add +1 per decade over age 50 (Charlson 1994 update)"
                },
                "myocardial_infarction": { "type": "boolean", "description": "History of MI (1 pt)" },
                "congestive_heart_failure": { "type": "boolean", "description": "CHF (1 pt)" },
                "peripheral_vascular_disease": { "type": "boolean", "description": "PVD or claudication (1 pt)" },
                "cerebrovascular_disease": { "type": "boolean", "description": "Stroke or TIA (1 pt)" },
                "dementia": { "type": "boolean", "description": "Dementia (1 pt)" },
                "copd": { "type": "boolean", "description": "COPD (1 pt)" },
                "connective_tissue_disease": { "type": "boolean", "description": "Connective tissue / rheumatologic disease (1 pt)" },
                "peptic_ulcer_disease": { "type": "boolean", "description": "Peptic ulcer disease (1 pt)" },
                "mild_liver_disease": { "type": "boolean", "description": "Mild liver disease (chronic hepatitis, cirrhosis without portal hypertension) (1 pt; superseded by moderate/severe)" },
                "diabetes_uncomplicated": { "type": "boolean", "description": "Diabetes without end-organ damage (1 pt; superseded by complicated)" },
                "diabetes_complicated": { "type": "boolean", "description": "Diabetes with end-organ damage (retinopathy, neuropathy, nephropathy) (2 pts)" },
                "hemiplegia_paraplegia": { "type": "boolean", "description": "Hemiplegia or paraplegia (2 pts)" },
                "renal_disease_moderate_severe": { "type": "boolean", "description": "Moderate/severe CKD, dialysis, or transplant (2 pts)" },
                "solid_tumour_non_metastatic": { "type": "boolean", "description": "Solid tumour (non-metastatic, treated within 5 years) (2 pts; superseded by metastatic)" },
                "leukaemia": { "type": "boolean", "description": "Leukaemia (2 pts)" },
                "lymphoma": { "type": "boolean", "description": "Lymphoma (2 pts)" },
                "liver_disease_moderate_severe": { "type": "boolean", "description": "Moderate/severe liver disease (portal hypertension, varices, encephalopathy) (3 pts)" },
                "metastatic_solid_tumour": { "type": "boolean", "description": "Metastatic solid tumour (6 pts)" },
                "aids": { "type": "boolean", "description": "AIDS (not HIV+ alone) (6 pts)" }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: CharlsonInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy(age: u8) -> CharlsonInput {
        CharlsonInput {
            age,
            include_age_adjustment: false,
            myocardial_infarction: false,
            congestive_heart_failure: false,
            peripheral_vascular_disease: false,
            cerebrovascular_disease: false,
            dementia: false,
            copd: false,
            connective_tissue_disease: false,
            peptic_ulcer_disease: false,
            mild_liver_disease: false,
            diabetes_uncomplicated: false,
            diabetes_complicated: false,
            hemiplegia_paraplegia: false,
            renal_disease_moderate_severe: false,
            solid_tumour_non_metastatic: false,
            leukaemia: false,
            lymphoma: false,
            liver_disease_moderate_severe: false,
            metastatic_solid_tumour: false,
            aids: false,
        }
    }

    #[test]
    fn zero_score_young_healthy() {
        let o = compute(&healthy(45)).unwrap();
        assert_eq!(o.total, 0);
        assert!(o.interpretation.contains("98%"));
    }

    #[test]
    fn age_adjustment_adds_correctly() {
        // Age 60 -> decade over 50 = 1 adjustment point
        let o = compute(&CharlsonInput {
            include_age_adjustment: true,
            ..healthy(60)
        })
        .unwrap();
        assert_eq!(o.age_adjustment, 1);
        assert_eq!(o.total, 1);

        // Age 70 -> 2 adjustment points
        let o2 = compute(&CharlsonInput {
            include_age_adjustment: true,
            ..healthy(70)
        })
        .unwrap();
        assert_eq!(o2.age_adjustment, 2);
    }

    #[test]
    fn severe_liver_supersedes_mild() {
        // mild_liver_disease=true but liver_disease_moderate_severe=true -> only 3 pts, not 4
        let o = compute(&CharlsonInput {
            mild_liver_disease: true,
            liver_disease_moderate_severe: true,
            ..healthy(50)
        })
        .unwrap();
        assert_eq!(o.score, 3);
    }

    #[test]
    fn metastatic_supersedes_non_metastatic() {
        let o = compute(&CharlsonInput {
            solid_tumour_non_metastatic: true,
            metastatic_solid_tumour: true,
            ..healthy(50)
        })
        .unwrap();
        assert_eq!(o.score, 6);
    }

    #[test]
    fn aids_scores_six() {
        let o = compute(&CharlsonInput {
            aids: true,
            ..healthy(40)
        })
        .unwrap();
        assert_eq!(o.score, 6);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let base = healthy(50);
        let value = json!({
            "age": 50,
            "include_age_adjustment": false,
            "myocardial_infarction": false,
            "congestive_heart_failure": false,
            "peripheral_vascular_disease": false,
            "cerebrovascular_disease": false,
            "dementia": false,
            "copd": false,
            "connective_tissue_disease": false,
            "peptic_ulcer_disease": false,
            "mild_liver_disease": false,
            "diabetes_uncomplicated": false,
            "diabetes_complicated": false,
            "hemiplegia_paraplegia": false,
            "renal_disease_moderate_severe": false,
            "solid_tumour_non_metastatic": false,
            "leukaemia": false,
            "lymphoma": false,
            "liver_disease_moderate_severe": false,
            "metastatic_solid_tumour": false,
            "aids": false
        });
        let dynamic = Charlson.calculate(&value).unwrap();
        let typed = build_response(&base).unwrap();
        assert_eq!(dynamic, typed);
    }
}
