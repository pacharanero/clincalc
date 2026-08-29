// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Estimated free-water deficit in an adult with hypernatraemia.
//!
//! The total-body-water fraction is supplied explicitly because sex, age,
//! adiposity, and hydration-state categories are imperfect proxies for body
//! composition. This is a static estimate, not a fluid prescription.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "free_water_deficit";
pub const REFERENCE: &str = "Yun G, Baek SH, Kim S. Evaluation and management of hypernatremia in adults: clinical perspectives. Korean J Intern Med. 2023;38(3):290-302. doi:10.3904/kjim.2022.346. Adrogue HJ, Madias NE. Hypernatremia. N Engl J Med. 2000;342(20):1493-1499. doi:10.1056/NEJM200005183422006.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published clinical calculation method - independently implemented from the literature",
    source_url: "https://doi.org/10.3904/kjim.2022.346",
};

const LIMITATIONS: &str = "Static estimate only, not a fluid prescription or correction rate. It excludes ongoing and insensible losses, intake, sodium and potassium losses or gains, and the fluid needed to restore extracellular volume. Correct sodium for substantial hyperglycaemia before use. Assess the cause, extracellular volume, renal function, acuity, and serial sodium response; predictive equations may be inaccurate in severe volume depletion or markedly reduced renal function.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentContext {
    AdultWithHypernatraemia,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreeWaterDeficitInput {
    pub assessment_context: AssessmentContext,
    pub weight_kg: f64,
    pub current_sodium_mmol_l: f64,
    pub target_sodium_mmol_l: f64,
    pub total_body_water_fraction: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreeWaterDeficitOutcome {
    pub estimated_total_body_water_l: f64,
    pub free_water_deficit_l: f64,
    pub interpretation: String,
}

pub fn compute(input: &FreeWaterDeficitInput) -> Result<FreeWaterDeficitOutcome, CalcError> {
    for (name, value) in [
        ("weight_kg", input.weight_kg),
        ("current_sodium_mmol_l", input.current_sodium_mmol_l),
        ("target_sodium_mmol_l", input.target_sodium_mmol_l),
        ("total_body_water_fraction", input.total_body_water_fraction),
    ] {
        if !value.is_finite() {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be a finite number"
            )));
        }
    }
    if input.weight_kg <= 0.0 {
        return Err(CalcError::InvalidInput("weight_kg must be positive".into()));
    }
    if input.current_sodium_mmol_l <= 145.0 {
        return Err(CalcError::InvalidInput(
            "current_sodium_mmol_l must be above 145 for adult hypernatraemia".into(),
        ));
    }
    if !(135.0..=145.0).contains(&input.target_sodium_mmol_l) {
        return Err(CalcError::InvalidInput(
            "target_sodium_mmol_l must be between 135 and 145".into(),
        ));
    }
    if input.target_sodium_mmol_l >= input.current_sodium_mmol_l {
        return Err(CalcError::InvalidInput(
            "target_sodium_mmol_l must be below current_sodium_mmol_l".into(),
        ));
    }
    if !(0.3..=0.7).contains(&input.total_body_water_fraction) {
        return Err(CalcError::InvalidInput(
            "total_body_water_fraction must be between 0.3 and 0.7".into(),
        ));
    }

    let estimated_total_body_water_l = input.weight_kg * input.total_body_water_fraction;
    let free_water_deficit_l = estimated_total_body_water_l
        * (input.current_sodium_mmol_l / input.target_sodium_mmol_l - 1.0);
    if !estimated_total_body_water_l.is_finite() || !free_water_deficit_l.is_finite() {
        return Err(CalcError::InvalidInput(
            "inputs produce a non-finite result".into(),
        ));
    }

    let rounded = (free_water_deficit_l * 10.0).round() / 10.0;
    let interpretation = format!(
        "Estimated free-water deficit is {rounded:.1} L using a selected total-body-water fraction of {:.2}. {LIMITATIONS}",
        input.total_body_water_fraction
    );

    Ok(FreeWaterDeficitOutcome {
        estimated_total_body_water_l,
        free_water_deficit_l,
        interpretation,
    })
}

pub fn build_response(input: &FreeWaterDeficitInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let rounded = (outcome.free_water_deficit_l * 10.0).round() / 10.0;
    let mut working = Map::new();
    working.insert("assessment_context".into(), json!(input.assessment_context));
    working.insert("weight_kg".into(), json!(input.weight_kg));
    working.insert(
        "current_sodium_mmol_l".into(),
        json!(input.current_sodium_mmol_l),
    );
    working.insert(
        "target_sodium_mmol_l".into(),
        json!(input.target_sodium_mmol_l),
    );
    working.insert(
        "total_body_water_fraction".into(),
        json!(input.total_body_water_fraction),
    );
    working.insert(
        "estimated_total_body_water_l".into(),
        json!(outcome.estimated_total_body_water_l),
    );
    working.insert(
        "free_water_deficit_l_unrounded".into(),
        json!(outcome.free_water_deficit_l),
    );
    working.insert(
        "formula".into(),
        json!("weight_kg * total_body_water_fraction * (current_sodium_mmol_l / target_sodium_mmol_l - 1)"),
    );
    working.insert("result_unit".into(), json!("L"));
    working.insert("limitations".into(), json!(LIMITATIONS));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(rounded),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn input_schema() -> Value {
    let source = json!({
        "citation": "Yun G, Baek SH, Kim S. Korean J Intern Med. 2023;38(3):290-302.",
        "url": "https://doi.org/10.3904/kjim.2022.346"
    });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "FreeWaterDeficitInput",
        "description": "Estimates the positive water balance required to move an adult's sodium from a hypernatraemic current value toward an explicit target. This is a static estimate, not a fluid prescription or correction rate. Select the total-body-water fraction from clinical assessment rather than asking the calculator to infer body composition from sex or an arbitrary age boundary.",
        "type": "object",
        "additionalProperties": false,
        "required": ["assessment_context", "weight_kg", "current_sodium_mmol_l", "target_sodium_mmol_l", "total_body_water_fraction"],
        "properties": {
            "assessment_context": {
                "type": "string",
                "const": "adult_with_hypernatraemia",
                "description": "Exact supported context: an adult with current sodium above 145 mmol/L",
                "definition": {
                    "concept": "Free-water-deficit assessment context",
                    "statement": "Use only as an estimate in an adult with hypernatraemia after clinical assessment of cause and volume status.",
                    "excludes": ["Paediatric use", "Normonatraemia or hyponatraemia", "Use as a stand-alone fluid prescription"],
                    "caveats": "Restore circulation first when haemodynamic compromise is present; the equation does not calculate that volume.",
                    "source": source,
                    "status": "draft"
                }
            },
            "weight_kg": {
                "type": "number", "exclusiveMinimum": 0, "unit": "kg",
                "description": "Current body weight in kilograms",
                "definition": {
                    "concept": "Body weight for estimated total body water",
                    "statement": "Enter current body weight in kilograms; it is multiplied by the selected total-body-water fraction.",
                    "excludes": ["Weight in pounds without conversion"],
                    "caveats": "Oedema and major fluid shifts can make current weight a poor proxy for body water.",
                    "source": source, "status": "draft"
                }
            },
            "current_sodium_mmol_l": {
                "type": "number", "exclusiveMinimum": 145, "unit": "mmol/L",
                "description": "Current sodium above 145 mmol/L; mmol/L and mEq/L are numerically equivalent for sodium",
                "definition": {
                    "concept": "Current sodium concentration",
                    "statement": "Enter the sodium concentration used for the current hypernatraemic assessment.",
                    "excludes": ["Uncorrected measured sodium when substantial hyperglycaemia requires a corrected value"],
                    "caveats": "Use serial measurements to assess actual response; this static equation does not predict ongoing change.",
                    "source": source, "status": "draft"
                }
            },
            "target_sodium_mmol_l": {
                "type": "number", "minimum": 135, "maximum": 145, "unit": "mmol/L",
                "description": "Explicit sodium target from 135 to 145 mmol/L and below the current sodium",
                "definition": {
                    "concept": "Target sodium concentration",
                    "statement": "Enter the clinician-selected physiologic sodium target used only to estimate the corresponding water balance.",
                    "excludes": ["A target equal to or above current sodium", "Use as an implied correction-rate target"],
                    "caveats": "The safe timing and rate of correction depend on acuity and clinical context and are outside this calculator.",
                    "source": source, "status": "draft"
                }
            },
            "total_body_water_fraction": {
                "type": "number", "minimum": 0.3, "maximum": 0.7,
                "description": "Clinician-selected estimated fraction of body weight that is total body water. Common heuristics include 0.60 for a typical adult man and 0.50 for a typical adult woman, with lower estimates in older, adipose, or water-depleted adults; these are estimates, not categorical truths.",
                "definition": {
                    "concept": "Estimated total-body-water fraction",
                    "statement": "Select a fraction from 0.3 to 0.7 based on sex, age, adiposity, hydration state, and the clinical context.",
                    "excludes": ["A percentage such as 50 instead of the fraction 0.50", "Automatic inference from sex or an arbitrary age threshold"],
                    "caveats": "Body composition is not measured by this calculator, and uncertainty in this fraction directly affects the estimated deficit.",
                    "source": source, "status": "draft"
                }
            }
        }
    })
}

pub struct FreeWaterDeficit;

impl Calculator for FreeWaterDeficit {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Free-water Deficit"
    }
    fn description(&self) -> &'static str {
        "Estimates free-water deficit in an adult with hypernatraemia from weight, sodium values, and an explicit total-body-water fraction."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        input_schema()
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: FreeWaterDeficitInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> FreeWaterDeficitInput {
        FreeWaterDeficitInput {
            assessment_context: AssessmentContext::AdultWithHypernatraemia,
            weight_kg: 60.0,
            current_sodium_mmol_l: 166.0,
            target_sodium_mmol_l: 140.0,
            total_body_water_fraction: 0.5,
        }
    }

    #[test]
    fn yun_worked_example_is_about_five_point_six_litres() {
        let outcome = compute(&example()).unwrap();
        assert_eq!(outcome.estimated_total_body_water_l, 30.0);
        assert!((outcome.free_water_deficit_l - 5.571428571428571).abs() < 1e-12);
        assert_eq!(build_response(&example()).unwrap().result, json!(5.6));
    }

    #[test]
    fn literature_formula_differs_from_reversed_ratio() {
        let outcome = compute(&example()).unwrap();
        let reversed_ratio = 30.0 * (1.0 - 140.0 / 166.0);
        assert!((outcome.free_water_deficit_l - reversed_ratio).abs() > 0.8);
    }

    #[test]
    fn target_and_fraction_vectors_are_exact() {
        let vectors = [
            (70.0, 0.6, 160.0, 140.0, 6.0),
            (68.0, 0.5, 168.0, 145.0, 5.393103448275863),
            (60.0, 0.4, 166.0, 140.0, 4.457142857142857),
        ];
        for (weight, fraction, current, target, expected) in vectors {
            let outcome = compute(&FreeWaterDeficitInput {
                weight_kg: weight,
                total_body_water_fraction: fraction,
                current_sodium_mmol_l: current,
                target_sodium_mmol_l: target,
                ..example()
            })
            .unwrap();
            assert!((outcome.free_water_deficit_l - expected).abs() < 1e-12);
        }
    }

    #[test]
    fn rejects_nonfinite_invalid_context_and_out_of_domain_values() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut input = example();
            input.weight_kg = value;
            assert!(compute(&input).is_err());
        }
        for (field, value) in [
            ("weight_kg", 0.0),
            ("current_sodium_mmol_l", 145.0),
            ("target_sodium_mmol_l", 134.999),
            ("target_sodium_mmol_l", 145.001),
            ("total_body_water_fraction", 0.299),
            ("total_body_water_fraction", 0.701),
        ] {
            let mut input = example();
            match field {
                "weight_kg" => input.weight_kg = value,
                "current_sodium_mmol_l" => input.current_sodium_mmol_l = value,
                "target_sodium_mmol_l" => input.target_sodium_mmol_l = value,
                _ => input.total_body_water_fraction = value,
            }
            assert!(compute(&input).is_err(), "accepted {field}={value}");
        }
        let mut invalid_context = serde_json::to_value(example()).unwrap();
        invalid_context["assessment_context"] = json!("child_with_hypernatraemia");
        assert!(FreeWaterDeficit.calculate(&invalid_context).is_err());
    }

    #[test]
    fn rejects_target_not_below_current_and_overflow() {
        let mut input = example();
        input.current_sodium_mmol_l = 145.5;
        input.target_sodium_mmol_l = 145.0;
        assert!(compute(&input).is_ok());
        input.target_sodium_mmol_l = 146.0;
        assert!(compute(&input).is_err());

        input = example();
        input.weight_kg = f64::MAX;
        input.current_sodium_mmol_l = f64::MAX;
        assert!(compute(&input).is_err());
    }

    #[test]
    fn dynamic_api_is_closed_and_matches_typed_response() {
        let value = serde_json::to_value(example()).unwrap();
        assert_eq!(
            FreeWaterDeficit.calculate(&value).unwrap(),
            build_response(&example()).unwrap()
        );
        let mut unknown = value;
        unknown["sex"] = json!("female");
        assert!(FreeWaterDeficit.calculate(&unknown).is_err());
    }

    #[test]
    fn working_and_interpretation_preserve_assumptions_and_limits() {
        let response = build_response(&example()).unwrap();
        assert_eq!(response.working["total_body_water_fraction"], json!(0.5));
        assert_eq!(
            response.working["estimated_total_body_water_l"],
            json!(30.0)
        );
        assert_eq!(response.working["result_unit"], json!("L"));
        assert!(response.interpretation.contains("not a fluid prescription"));
        assert!(
            response
                .interpretation
                .contains("ongoing and insensible losses")
        );
        assert!(response.interpretation.contains("hyperglycaemia"));
        assert!(response.interpretation.contains("serial sodium"));
    }

    #[test]
    fn schema_is_closed_required_and_defines_units_and_assumptions() {
        let schema = FreeWaterDeficit.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["required"].as_array().unwrap().len(), 5);
        assert_eq!(schema["properties"]["weight_kg"]["unit"], json!("kg"));
        assert_eq!(
            schema["properties"]["current_sodium_mmol_l"]["unit"],
            json!("mmol/L")
        );
        assert!(
            schema["description"]
                .as_str()
                .unwrap()
                .contains("not a fluid prescription")
        );
        for property in schema["properties"].as_object().unwrap().values() {
            assert!(property["definition"]["statement"].is_string());
            assert_eq!(property["definition"]["status"], json!("draft"));
        }
    }
}
