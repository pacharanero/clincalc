// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Body fat percentage from circumference measurements - US Navy method.
//!
//! Hodgdon & Beckett (1984) regression equations developed for the US Navy.
//! All measurements in centimetres.
//!
//! Men:   %BF = 495 / (1.0324 - 0.19077 × log10(waist - neck) + 0.15456 × log10(height)) - 450
//! Women: %BF = 495 / (1.29579 - 0.35004 × log10(waist + hip - neck) + 0.22100 × log10(height)) - 450
//!
//! Waist is measured at the navel (men) or narrowest point (women).
//! Neck is measured just below the larynx.
//! Hip is measured at the widest point (women only).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "body_fat_circumference";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://apps.dtic.mil/sti/citations/ADA148757",
};

pub const REFERENCE: &str = "Hodgdon JA, Beckett MB. Prediction of percent body fat for US Navy men and women \
from body circumference and height. Report No. 84-29. Naval Health Research Center; 1984.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyFatCircumferenceInput {
    pub sex: Sex,
    /// Height in centimetres
    pub height_cm: f64,
    /// Waist circumference in cm (at navel for men; narrowest point for women)
    pub waist_cm: f64,
    /// Neck circumference in cm (just below the larynx)
    pub neck_cm: f64,
    /// Hip circumference in cm (widest point; required for women, ignored for men)
    pub hip_cm: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BodyFatCircumferenceOutcome {
    pub body_fat_percent: f64,
    pub interpretation: String,
}

fn classify_body_fat(sex: Sex, bf: f64) -> &'static str {
    match sex {
        Sex::Male => {
            if bf < 6.0 {
                "essential fat (below healthy minimum)"
            } else if bf < 14.0 {
                "athletic"
            } else if bf < 18.0 {
                "fitness"
            } else if bf < 25.0 {
                "average"
            } else {
                "obese range"
            }
        }
        Sex::Female => {
            if bf < 14.0 {
                "essential fat (below healthy minimum)"
            } else if bf < 21.0 {
                "athletic"
            } else if bf < 25.0 {
                "fitness"
            } else if bf < 32.0 {
                "average"
            } else {
                "obese range"
            }
        }
    }
}

pub fn compute(
    input: &BodyFatCircumferenceInput,
) -> Result<BodyFatCircumferenceOutcome, CalcError> {
    if input.height_cm <= 0.0 || !input.height_cm.is_finite() {
        return Err(CalcError::InvalidInput("height_cm must be positive".into()));
    }
    if input.waist_cm <= 0.0 || !input.waist_cm.is_finite() {
        return Err(CalcError::InvalidInput("waist_cm must be positive".into()));
    }
    if input.neck_cm <= 0.0 || !input.neck_cm.is_finite() {
        return Err(CalcError::InvalidInput("neck_cm must be positive".into()));
    }
    if input.waist_cm <= input.neck_cm {
        return Err(CalcError::InvalidInput(
            "waist_cm must be greater than neck_cm".into(),
        ));
    }

    let bf = match input.sex {
        Sex::Male => {
            let log_diff = (input.waist_cm - input.neck_cm).log10();
            let log_height = input.height_cm.log10();
            495.0 / (1.0324 - 0.19077 * log_diff + 0.15456 * log_height) - 450.0
        }
        Sex::Female => {
            let hip = input.hip_cm.ok_or_else(|| {
                CalcError::InvalidInput("hip_cm is required for female sex".into())
            })?;
            if hip <= 0.0 || !hip.is_finite() {
                return Err(CalcError::InvalidInput("hip_cm must be positive".into()));
            }
            if (input.waist_cm + hip) <= input.neck_cm {
                return Err(CalcError::InvalidInput(
                    "waist_cm + hip_cm must be greater than neck_cm".into(),
                ));
            }
            let log_sum = (input.waist_cm + hip - input.neck_cm).log10();
            let log_height = input.height_cm.log10();
            495.0 / (1.29579 - 0.35004 * log_sum + 0.22100 * log_height) - 450.0
        }
    };

    if !bf.is_finite() || bf < 0.0 {
        return Err(CalcError::InvalidInput(
            "computed body fat percentage is invalid - check measurement inputs".into(),
        ));
    }

    let category = classify_body_fat(input.sex, bf);
    let interpretation = format!(
        "Estimated body fat {:.1}% ({category}) by the US Navy circumference method \
(Hodgdon & Beckett 1984). This method has a standard error of ~3-4% and should not \
replace clinical assessment or direct body composition measurement.",
        bf
    );

    Ok(BodyFatCircumferenceOutcome {
        body_fat_percent: bf,
        interpretation,
    })
}

pub fn build_response(input: &BodyFatCircumferenceInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert(
        "body_fat_percent".into(),
        json!(format!("{:.1}", o.body_fat_percent)),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(format!("{:.1}", o.body_fat_percent)),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct BodyFatCircumference;

impl Calculator for BodyFatCircumference {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Body Fat % (US Navy Circumference Method)"
    }

    fn description(&self) -> &'static str {
        "Estimates body fat percentage from height, waist, neck (and hip for women) using the US Navy / Hodgdon-Beckett regression equations."
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
            "title": "BodyFatCircumferenceInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["sex", "height_cm", "waist_cm", "neck_cm"],
            "properties": {
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Sex (determines which regression equation is used; hip measurement required for female)"
                },
                "height_cm": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Standing height in centimetres"
                },
                "waist_cm": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Waist circumference in cm: at navel level (men) or narrowest point (women)"
                },
                "neck_cm": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Neck circumference in cm, measured just below the larynx"
                },
                "hip_cm": {
                    "type": ["number", "null"],
                    "exclusiveMinimum": 0,
                    "description": "Hip circumference in cm at widest point (required for female, ignored for male)"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: BodyFatCircumferenceInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn male(height: f64, waist: f64, neck: f64) -> BodyFatCircumferenceInput {
        BodyFatCircumferenceInput {
            sex: Sex::Male,
            height_cm: height,
            waist_cm: waist,
            neck_cm: neck,
            hip_cm: None,
        }
    }

    fn female(height: f64, waist: f64, neck: f64, hip: f64) -> BodyFatCircumferenceInput {
        BodyFatCircumferenceInput {
            sex: Sex::Female,
            height_cm: height,
            waist_cm: waist,
            neck_cm: neck,
            hip_cm: Some(hip),
        }
    }

    #[test]
    fn male_typical() {
        // 178 cm, waist 85 cm, neck 38 cm -> ~17-18% BF
        let o = compute(&male(178.0, 85.0, 38.0)).unwrap();
        assert!(
            o.body_fat_percent > 14.0 && o.body_fat_percent < 22.0,
            "got {:.1}%",
            o.body_fat_percent
        );
    }

    #[test]
    fn female_typical() {
        // 165 cm, waist 72 cm, neck 34 cm, hip 96 cm -> ~25-30% BF
        let o = compute(&female(165.0, 72.0, 34.0, 96.0)).unwrap();
        assert!(
            o.body_fat_percent > 20.0 && o.body_fat_percent < 35.0,
            "got {:.1}%",
            o.body_fat_percent
        );
    }

    #[test]
    fn female_requires_hip() {
        let input = BodyFatCircumferenceInput {
            sex: Sex::Female,
            height_cm: 165.0,
            waist_cm: 72.0,
            neck_cm: 34.0,
            hip_cm: None,
        };
        assert!(compute(&input).is_err());
    }

    #[test]
    fn rejects_waist_le_neck() {
        assert!(compute(&male(178.0, 38.0, 38.0)).is_err());
        assert!(compute(&male(178.0, 35.0, 38.0)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "sex": "male",
            "height_cm": 178.0,
            "waist_cm": 85.0,
            "neck_cm": 38.0,
            "hip_cm": null
        });
        let dynamic = BodyFatCircumference.calculate(&value).unwrap();
        let typed = build_response(&male(178.0, 85.0, 38.0)).unwrap();
        assert_eq!(dynamic, typed);
    }
}
