// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Body surface area (BSA) using the Mosteller formula.
//!
//! Formula: BSA (m2) = sqrt(height_cm x weight_kg / 3600).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "body_surface_area";
pub const REFERENCE: &str = "Mosteller RD. Simplified calculation of body-surface area. N Engl J Med. 1987;317:1098. PMID: 3657876. doi:10.1056/NEJM198710223171717.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1056/NEJM198710223171717",
};

const FORMULA: &str = "sqrt(height_cm * weight_kg / 3600)";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodySurfaceAreaInput {
    /// Height in centimetres.
    pub height_cm: f64,
    /// Weight in kilograms.
    pub weight_kg: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BodySurfaceAreaOutcome {
    pub body_surface_area_m2: f64,
    pub interpretation: String,
}

pub fn compute(input: &BodySurfaceAreaInput) -> Result<BodySurfaceAreaOutcome, CalcError> {
    if !input.height_cm.is_finite() || !(10.0..=250.0).contains(&input.height_cm) {
        return Err(CalcError::InvalidInput(
            "height_cm must be finite and between 10 and 250".into(),
        ));
    }
    if !input.weight_kg.is_finite() || !(0.5..=300.0).contains(&input.weight_kg) {
        return Err(CalcError::InvalidInput(
            "weight_kg must be finite and between 0.5 and 300".into(),
        ));
    }

    let body_surface_area_m2 = (input.height_cm * input.weight_kg / 3600.0).sqrt();
    let interpretation = format!(
        "Body surface area is {body_surface_area_m2:.2} m2 using the Mosteller formula. Use for drug dosing or indexing must follow the relevant protocol."
    );

    Ok(BodySurfaceAreaOutcome {
        body_surface_area_m2,
        interpretation,
    })
}

pub fn build_response(input: &BodySurfaceAreaInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert("height_cm".into(), json!(input.height_cm));
    working.insert("weight_kg".into(), json!(input.weight_kg));
    working.insert(
        "body_surface_area_m2_unrounded".into(),
        json!(outcome.body_surface_area_m2),
    );
    working.insert("formula".into(), json!(FORMULA));
    working.insert("unit".into(), json!("m2"));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round2(outcome.body_surface_area_m2)),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub struct BodySurfaceArea;

impl Calculator for BodySurfaceArea {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Body Surface Area (Mosteller)"
    }

    fn description(&self) -> &'static str {
        "Calculates body surface area from height and weight using the Mosteller formula."
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
            "title": "BodySurfaceAreaInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["height_cm", "weight_kg"],
            "properties": {
                "height_cm": {
                    "type": "number",
                    "minimum": 10,
                    "maximum": 250,
                    "description": "Height in centimetres",
                    "unit": "cm"
                },
                "weight_kg": {
                    "type": "number",
                    "minimum": 0.5,
                    "maximum": 300,
                    "description": "Weight in kilograms",
                    "unit": "kg"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: BodySurfaceAreaInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::SupportedLocale;

    fn input(height_cm: f64, weight_kg: f64) -> BodySurfaceAreaInput {
        BodySurfaceAreaInput {
            height_cm,
            weight_kg,
        }
    }

    #[test]
    fn computes_exact_two_square_metres() {
        let outcome = compute(&input(180.0, 80.0)).unwrap();
        assert_eq!(outcome.body_surface_area_m2, 2.0);
        assert_eq!(
            build_response(&input(180.0, 80.0)).unwrap().result,
            json!(2.0)
        );
    }

    #[test]
    fn computes_exact_one_square_metre() {
        let outcome = compute(&input(100.0, 36.0)).unwrap();
        assert_eq!(outcome.body_surface_area_m2, 1.0);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let dynamic = BodySurfaceArea
            .calculate(&json!({"height_cm": 175.0, "weight_kg": 70.0}))
            .unwrap();
        let typed = build_response(&input(175.0, 70.0)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn response_preserves_inputs_unrounded_result_formula_and_unit() {
        let response = build_response(&input(175.0, 70.0)).unwrap();
        assert_eq!(response.working["height_cm"], json!(175.0));
        assert_eq!(response.working["weight_kg"], json!(70.0));
        assert_eq!(response.working["formula"], json!(FORMULA));
        assert_eq!(response.working["unit"], json!("m2"));
        assert_eq!(
            response.working["body_surface_area_m2_unrounded"],
            json!((175.0_f64 * 70.0 / 3600.0).sqrt())
        );
        assert_eq!(response.result, json!(1.84));
        assert!(response.interpretation.contains("relevant protocol"));
    }

    #[test]
    fn compatibility_calculation_records_content_locale() {
        let response = BodySurfaceArea
            .calculate_for(
                &json!({"height_cm": 180.0, "weight_kg": 80.0}),
                SupportedLocale::En,
            )
            .unwrap();
        assert_eq!(response.working["content_locale"], json!("en"));
    }

    #[test]
    fn rejects_nonfinite_values() {
        for height_cm in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(compute(&input(height_cm, 70.0)).is_err());
        }
        for weight_kg in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(compute(&input(175.0, weight_kg)).is_err());
        }
    }

    #[test]
    fn rejects_out_of_range_values() {
        for height_cm in [9.99, 250.01] {
            assert!(compute(&input(height_cm, 70.0)).is_err());
        }
        for weight_kg in [0.49, 300.01] {
            assert!(compute(&input(175.0, weight_kg)).is_err());
        }
    }

    #[test]
    fn accepts_domain_boundaries() {
        assert!(compute(&input(10.0, 0.5)).is_ok());
        assert!(compute(&input(250.0, 300.0)).is_ok());
    }

    #[test]
    fn dynamic_api_rejects_out_of_range_and_unknown_fields() {
        assert!(
            BodySurfaceArea
                .calculate(&json!({"height_cm": 9.0, "weight_kg": 80.0}))
                .is_err()
        );
        assert!(
            BodySurfaceArea
                .calculate(&json!({"height_cm": 180.0, "weight_kg": 301.0}))
                .is_err()
        );
        assert!(
            BodySurfaceArea
                .calculate(&json!({
                    "height_cm": 180.0,
                    "weight_kg": 80.0,
                    "unexpected": true
                }))
                .is_err()
        );
    }

    #[test]
    fn schema_records_closed_shape_units_and_ranges() {
        let schema = BodySurfaceArea.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["properties"]["height_cm"]["minimum"], json!(10));
        assert_eq!(schema["properties"]["height_cm"]["maximum"], json!(250));
        assert_eq!(schema["properties"]["height_cm"]["unit"], json!("cm"));
        assert_eq!(schema["properties"]["weight_kg"]["minimum"], json!(0.5));
        assert_eq!(schema["properties"]["weight_kg"]["maximum"], json!(300));
        assert_eq!(schema["properties"]["weight_kg"]["unit"], json!("kg"));
    }
}
