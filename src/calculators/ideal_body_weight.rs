// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Ideal body weight (Devine formula), commonly needed for weight-based drug
//! dosing. Distinct from the shipped ARDSNet predicted-body-weight protocol,
//! which uses a different formula for a different purpose (ventilator
//! tidal-volume calculation, not general dosing).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "ideal_body_weight";
pub const REFERENCE: &str =
    "Devine BJ. Gentamicin therapy. Drug Intell Clin Pharm. 1974;8:650-655.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain formula - implemented from the primary literature",
    "https://doi.org/10.1177/106002807400801104",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdealBodyWeightInput {
    pub sex: Sex,
    pub height_cm: f64,
    /// Actual body weight; if supplied, adjusted body weight is also
    /// reported for patients whose actual weight exceeds 130% of IBW, using
    /// the commonly cited AdjBW = IBW + 0.4 x (ABW - IBW).
    #[serde(default)]
    pub actual_weight_kg: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IdealBodyWeightOutcome {
    pub ibw_kg: f64,
    pub adjusted_body_weight_kg: Option<f64>,
}

const CM_PER_INCH: f64 = 2.54;

pub fn compute(input: &IdealBodyWeightInput) -> Result<IdealBodyWeightOutcome, CalcError> {
    validate(input.height_cm, "height_cm", 152.4, 230.0)?;
    if let Some(w) = input.actual_weight_kg {
        validate(w, "actual_weight_kg", 10.0, 400.0)?;
    }

    let height_inches = input.height_cm / CM_PER_INCH;
    let base = match input.sex {
        Sex::Male => 50.0,
        Sex::Female => 45.5,
    };
    let ibw_kg = base + 2.3 * (height_inches - 60.0);

    let adjusted_body_weight_kg = input.actual_weight_kg.and_then(|abw| {
        if abw > 1.3 * ibw_kg {
            Some(ibw_kg + 0.4 * (abw - ibw_kg))
        } else {
            None
        }
    });

    Ok(IdealBodyWeightOutcome {
        ibw_kg,
        adjusted_body_weight_kg,
    })
}

fn validate(value: f64, name: &str, min: f64, max: f64) -> Result<(), CalcError> {
    if !(min..=max).contains(&value) || !value.is_finite() {
        return Err(CalcError::InvalidInput(format!(
            "{name} must be finite and between {min} and {max}"
        )));
    }
    Ok(())
}

pub fn build_response(input: &IdealBodyWeightInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("unit".into(), json!("kg"));
    if let Some(adj) = o.adjusted_body_weight_kg {
        working.insert("adjusted_body_weight_kg".into(), json!(round1(adj)));
    }

    let interpretation = match o.adjusted_body_weight_kg {
        Some(adj) => format!(
            "Ideal body weight {:.1} kg (Devine formula). Actual weight exceeds 130% of IBW, so an adjusted body weight of {:.1} kg (IBW + 0.4 x (actual - IBW)) is also reported - whether to dose on actual, ideal, or adjusted weight is drug- and guideline-specific; check the relevant dosing reference.",
            o.ibw_kg, adj
        ),
        None => format!(
            "Ideal body weight {:.1} kg (Devine formula). The formula was derived for adults at least 60 inches (152.4 cm) tall and is not valid below that height.",
            o.ibw_kg
        ),
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.ibw_kg)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct IdealBodyWeight;
impl Calculator for IdealBodyWeight {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Ideal Body Weight (Devine Formula)"
    }
    fn description(&self) -> &'static str {
        "Devine formula ideal body weight from sex and height, with optional adjusted body weight for obese patients."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "IdealBodyWeightInput", "type": "object", "additionalProperties": false,
            "required": ["sex", "height_cm"],
            "properties": {
                "sex": { "type": "string", "enum": ["Male", "Female"] },
                "height_cm": { "type": "number", "minimum": 152.4, "maximum": 230, "description": "The Devine formula is defined for adults at least 152.4 cm (60 in) tall" },
                "actual_weight_kg": { "type": "number", "minimum": 10, "maximum": 400, "description": "Optional; if supplied, adjusted body weight is reported when actual weight exceeds 130% of IBW" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: IdealBodyWeightInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn male_at_60_inches_matches_base_value() {
        let out = compute(&IdealBodyWeightInput {
            sex: Sex::Male,
            height_cm: 152.4,
            actual_weight_kg: None,
        })
        .unwrap();
        assert!((out.ibw_kg - 50.0).abs() < 1e-9);
    }

    #[test]
    fn female_at_65_inches() {
        // 65 in = 165.1 cm; IBW = 45.5 + 2.3*5 = 57.0 kg
        let out = compute(&IdealBodyWeightInput {
            sex: Sex::Female,
            height_cm: 165.1,
            actual_weight_kg: None,
        })
        .unwrap();
        assert!((out.ibw_kg - 57.0).abs() < 0.01);
    }

    #[test]
    fn adjusted_weight_reported_when_obese() {
        let out = compute(&IdealBodyWeightInput {
            sex: Sex::Male,
            height_cm: 152.4,
            actual_weight_kg: Some(90.0),
        })
        .unwrap();
        // IBW 50, ABW 90 > 1.3*50=65, so AdjBW = 50 + 0.4*(90-50) = 66.0
        assert_eq!(out.adjusted_body_weight_kg, Some(66.0));
    }

    #[test]
    fn adjusted_weight_absent_when_not_obese() {
        let out = compute(&IdealBodyWeightInput {
            sex: Sex::Male,
            height_cm: 152.4,
            actual_weight_kg: Some(55.0),
        })
        .unwrap();
        assert_eq!(out.adjusted_body_weight_kg, None);
    }

    #[test]
    fn rejects_height_below_devine_range() {
        assert!(
            compute(&IdealBodyWeightInput {
                sex: Sex::Male,
                height_cm: 100.0,
                actual_weight_kg: None,
            })
            .is_err()
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"sex": "Male", "height_cm": 152.4});
        let typed = IdealBodyWeightInput {
            sex: Sex::Male,
            height_cm: 152.4,
            actual_weight_kg: None,
        };
        let dynamic = IdealBodyWeight.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(IdealBodyWeight.calculate(&json!({"sex": "Male"})).is_err());
    }
}
