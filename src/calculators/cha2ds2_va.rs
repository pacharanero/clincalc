// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! CHA₂DS₂-VA score - 2024 ESC sex-free update of CHA₂DS₂-VASc.
//!
//! The 2024 ESC AF Guidelines removed female sex as a risk modifier.
//! The score now ranges 0-8 and is applied equally to all patients.
//!
//! C  Congestive heart failure (1 point)
//! H  Hypertension (1 point)
//! A₂ Age ≥ 75 years (2 points)
//! D  Diabetes mellitus (1 point)
//! S₂ Prior stroke, TIA, or thromboembolism (2 points)
//! V  Vascular disease (MI, peripheral artery disease, or aortic plaque) (1 point)
//! A  Age 65-74 years (1 point)
//!
//! Age ≥ 75 and age 65-74 are mutually exclusive; age ≥ 75 takes priority.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "cha2ds2_va";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1093/eurheartj/ehae176",
};

pub const REFERENCE: &str = "Van Gelder IC, Rienstra M, Bunting KV, et al. 2024 ESC Guidelines for the management of atrial \
fibrillation. Eur Heart J. 2024;45(36):3314-3414. doi:10.1093/eurheartj/ehae176";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cha2ds2VaInput {
    /// Patient age in years
    pub age: u8,
    /// Congestive heart failure or LVEF < 40%
    pub congestive_heart_failure: bool,
    /// Hypertension (treated or untreated, resting BP > 140/90 on two occasions)
    pub hypertension: bool,
    /// Diabetes mellitus
    pub diabetes: bool,
    /// Prior stroke, TIA, or thromboembolism
    pub stroke_tia_thromboembolism: bool,
    /// Vascular disease: MI, peripheral artery disease, or aortic plaque
    pub vascular_disease: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cha2ds2VaOutcome {
    pub score: u8,
    pub interpretation: String,
}

pub fn compute(input: &Cha2ds2VaInput) -> Result<Cha2ds2VaOutcome, CalcError> {
    let mut score: u8 = 0;

    if input.congestive_heart_failure {
        score += 1;
    }
    if input.hypertension {
        score += 1;
    }
    if input.age >= 75 {
        score += 2;
    } else if input.age >= 65 {
        score += 1;
    }
    if input.diabetes {
        score += 1;
    }
    if input.stroke_tia_thromboembolism {
        score += 2;
    }
    if input.vascular_disease {
        score += 1;
    }

    let recommendation = match score {
        0 => "Low risk. Anticoagulation is not recommended.",
        1 => "Low-to-moderate risk. Anticoagulation may be considered; reassess at next review.",
        _ => "Moderate-to-high risk. Oral anticoagulation is recommended (if no contraindication).",
    };

    let interpretation = format!(
        "CHA\u{2082}DS\u{2082}-VA score {score}/8. {recommendation} \
(2024 ESC Guidelines: sex is no longer a risk-modifier in this updated score.)"
    );

    Ok(Cha2ds2VaOutcome {
        score,
        interpretation,
    })
}

pub fn build_response(input: &Cha2ds2VaInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("cha2ds2_va_score".into(), json!(o.score));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Cha2ds2Va;

impl Calculator for Cha2ds2Va {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "CHA\u{2082}DS\u{2082}-VA (2024 ESC)"
    }

    fn description(&self) -> &'static str {
        "Stroke risk in atrial fibrillation using the 2024 ESC sex-free update (CHA\u{2082}DS\u{2082}-VA, score 0-8)."
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
            "title": "Cha2ds2VaInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["age", "congestive_heart_failure", "hypertension", "diabetes",
                         "stroke_tia_thromboembolism", "vascular_disease"],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 120,
                    "description": "Patient age in years (65-74 scores 1, ≥75 scores 2)"
                },
                "congestive_heart_failure": {
                    "type": "boolean",
                    "description": "Congestive heart failure or LVEF < 40% (1 point)"
                },
                "hypertension": {
                    "type": "boolean",
                    "description": "Hypertension (treated or resting BP > 140/90 on two occasions) (1 point)"
                },
                "diabetes": {
                    "type": "boolean",
                    "description": "Diabetes mellitus (1 point)"
                },
                "stroke_tia_thromboembolism": {
                    "type": "boolean",
                    "description": "Prior stroke, TIA, or systemic thromboembolism (2 points)"
                },
                "vascular_disease": {
                    "type": "boolean",
                    "description": "Vascular disease: prior MI, peripheral artery disease, or aortic plaque (1 point)"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: Cha2ds2VaInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(age: u8, chf: bool, htn: bool, dm: bool, stroke: bool, vasc: bool) -> Cha2ds2VaInput {
        Cha2ds2VaInput {
            age,
            congestive_heart_failure: chf,
            hypertension: htn,
            diabetes: dm,
            stroke_tia_thromboembolism: stroke,
            vascular_disease: vasc,
        }
    }

    #[test]
    fn zero_score_young_no_risk() {
        let o = compute(&inp(50, false, false, false, false, false)).unwrap();
        assert_eq!(o.score, 0);
        assert!(o.interpretation.contains("not recommended"));
    }

    #[test]
    fn age_65_74_scores_one() {
        let o = compute(&inp(70, false, false, false, false, false)).unwrap();
        assert_eq!(o.score, 1);
    }

    #[test]
    fn age_75_scores_two() {
        let o = compute(&inp(75, false, false, false, false, false)).unwrap();
        assert_eq!(o.score, 2);
    }

    #[test]
    fn maximum_score() {
        // Age ≥75 (2) + CHF (1) + HTN (1) + DM (1) + stroke (2) + vasc (1) = 8
        let o = compute(&inp(80, true, true, true, true, true)).unwrap();
        assert_eq!(o.score, 8);
    }

    #[test]
    fn stroke_history_scores_two() {
        let o = compute(&inp(60, false, false, false, true, false)).unwrap();
        assert_eq!(o.score, 2);
        assert!(o.interpretation.contains("recommended"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age": 70,
            "congestive_heart_failure": true,
            "hypertension": true,
            "diabetes": false,
            "stroke_tia_thromboembolism": false,
            "vascular_disease": false
        });
        let dynamic = Cha2ds2Va.calculate(&value).unwrap();
        let typed = build_response(&inp(70, true, true, false, false, false)).unwrap();
        assert_eq!(dynamic, typed);
    }
}
