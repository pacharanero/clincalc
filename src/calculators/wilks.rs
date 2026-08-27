// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Wilks coefficient - bodyweight-adjusted powerlifting score.
//!
//! Scales a bench press or three-lift competition total by a sex-specific
//! quintic-polynomial function of bodyweight. The International Powerlifting
//! Federation (IPF) used Wilks before replacing it with IPF Points in 2019;
//! IPF GL Points replaced IPF Points in 2020.
//!
//! `coefficient = 500 / (a + b*x + c*x^2 + d*x^3 + e*x^4 + f*x^5)`, where `x`
//! is bodyweight in kilograms. `score = coefficient * lifted_kg`.
//!
//! The published coefficient tables start at 40 kg. The historical coefficient
//! remains constant above 205 kg for men and 150 kg for women, so this
//! calculator applies those ceilings rather than extrapolating the polynomial.
//!
//! Coefficients as tabulated by Robert Wilks (Australia) and reproduced at
//! <https://www.europowerlifting.org/fileadmin/data/wilks_formula/Wilksformula_01.pdf>;
//! verified here against the published lookup tables at bodyweights 69.3 kg
//! (men) and 100.0 kg (men and women). The IPF adopted coefficients tabulated
//! to four decimal places, so the rounded coefficient is used to calculate the
//! score.
//!
//! Reference: Vanderburgh PM, Batterham AM. Validation of the Wilks
//! powerlifting formula. Med Sci Sports Exerc. 1999;31(12):1869-1875.
//! doi:10.1097/00005768-199912000-00027

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "wilks";

pub const REFERENCE: &str = "Wilks R. Wilks Formula coefficient table. European Powerlifting Federation. https://www.europowerlifting.org/fileadmin/data/wilks_formula/Wilksformula_01.pdf. Vanderburgh PM, Batterham AM. Validation of the Wilks powerlifting formula. Med Sci Sports Exerc. 1999;31(12):1869-1875. doi:10.1097/00005768-199912000-00027. Marksteiner J. IPF Points - Proposed Replacement for Wilks Coefficients. International Powerlifting Federation, 2018. https://www.powerlifting.sport/fileadmin/ipf/data/ipf-formula/IPF_Points_Proposal.pdf.";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain mathematical formula - coefficients published by the European Powerlifting Federation",
    source_url: "https://www.europowerlifting.org/fileadmin/data/wilks_formula/Wilksformula_01.pdf",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiftType {
    BenchPress,
    ThreeLiftTotal,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WilksInput {
    pub sex: Sex,
    /// Validated powerlifting application of the coefficient.
    pub lift_type: LiftType,
    /// Bodyweight in kilograms.
    pub bodyweight_kg: f64,
    /// Bench press or summed squat, bench press, and deadlift total in kilograms.
    pub lifted_kg: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WilksOutcome {
    pub coefficient_bodyweight_kg: f64,
    pub coefficient_was_capped: bool,
    pub coefficient: f64,
    pub score: f64,
    pub interpretation: String,
}

fn maximum_coefficient_bodyweight(sex: Sex) -> f64 {
    match sex {
        Sex::Male => 205.0,
        Sex::Female => 150.0,
    }
}

fn raw_coefficient(sex: Sex, bodyweight_kg: f64) -> f64 {
    let x = bodyweight_kg;
    let (a, b, c, d, e, f): (f64, f64, f64, f64, f64, f64) = match sex {
        Sex::Male => (
            -216.0475144,
            16.2606339,
            -0.002388645,
            -0.00113732,
            7.01863e-6,
            -1.291e-8,
        ),
        Sex::Female => (
            594.31747775582,
            -27.23842536447,
            0.82112226871,
            -0.00930733913,
            4.731582e-5,
            -9.054e-8,
        ),
    };
    let denominator = ((((f * x + e) * x + d) * x + c) * x + b) * x + a;
    500.0 / denominator
}

fn round4(v: f64) -> f64 {
    (v * 10_000.0).round() / 10_000.0
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub fn compute(input: &WilksInput) -> Result<WilksOutcome, CalcError> {
    if !input.bodyweight_kg.is_finite() || input.bodyweight_kg < 40.0 {
        return Err(CalcError::InvalidInput(
            "bodyweight_kg must be finite and at least 40 - the published coefficient table does not cover lower bodyweights".into(),
        ));
    }
    if !input.lifted_kg.is_finite() || input.lifted_kg <= 0.0 {
        return Err(CalcError::InvalidInput(
            "lifted_kg must be finite and positive".into(),
        ));
    }

    let maximum_bodyweight = maximum_coefficient_bodyweight(input.sex);
    let coefficient_bodyweight_kg = input.bodyweight_kg.min(maximum_bodyweight);
    let coefficient_was_capped = input.bodyweight_kg > maximum_bodyweight;
    let coefficient = round4(raw_coefficient(input.sex, coefficient_bodyweight_kg));
    let score = round2(coefficient * input.lifted_kg);
    if !score.is_finite() {
        return Err(CalcError::InvalidInput(
            "lifted_kg is too large to produce a finite score".into(),
        ));
    }
    let sex_label = match input.sex {
        Sex::Male => "male",
        Sex::Female => "female",
    };
    let lift_label = match input.lift_type {
        LiftType::BenchPress => "bench press",
        LiftType::ThreeLiftTotal => "three-lift total",
    };
    let cap_note = if coefficient_was_capped {
        format!(
            " The source-table coefficient is capped at {maximum_bodyweight:.1} kg for this category."
        )
    } else {
        String::new()
    };

    let interpretation = format!(
        "Wilks coefficient {coefficient:.4} for a {sex_label} lifter at {:.2} kg bodyweight, applied to a {:.1} kg {lift_label}, gives a Wilks score of {score:.2}. Higher scores represent stronger performances relative to bodyweight within the same source category. Vanderburgh and Batterham validated Wilks for bench press and three-lift total, the two applications supported here.{cap_note} The IPF replaced Wilks with IPF Points in 2019 and then IPF GL Points in 2020.",
        input.bodyweight_kg, input.lifted_kg,
    );

    Ok(WilksOutcome {
        coefficient_bodyweight_kg,
        coefficient_was_capped,
        coefficient,
        score,
        interpretation,
    })
}

pub fn build_response(input: &WilksInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("sex".into(), json!(input.sex));
    working.insert("lift_type".into(), json!(input.lift_type));
    working.insert("bodyweight_kg".into(), json!(input.bodyweight_kg));
    working.insert("lifted_kg".into(), json!(input.lifted_kg));
    working.insert(
        "coefficient_bodyweight_kg".into(),
        json!(o.coefficient_bodyweight_kg),
    );
    working.insert(
        "coefficient_was_capped".into(),
        json!(o.coefficient_was_capped),
    );
    working.insert("coefficient".into(), json!(o.coefficient));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Wilks;

impl Calculator for Wilks {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Wilks Score"
    }

    fn description(&self) -> &'static str {
        "Historical bodyweight-adjusted powerlifting score for bench press or three-lift total using the IPF-era Wilks coefficient."
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
            "title": "WilksInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["sex", "lift_type", "bodyweight_kg", "lifted_kg"],
            "properties": {
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Competition category from the source table (determines which coefficient polynomial is applied)"
                },
                "lift_type": {
                    "type": "string",
                    "enum": ["bench_press", "three_lift_total"],
                    "description": "Validated application: bench press, or the sum of the best squat, bench press, and deadlift"
                },
                "bodyweight_kg": {
                    "type": "number",
                    "minimum": 40,
                    "description": "Bodyweight in kg. The published table starts at 40 kg; coefficients are capped above 205 kg for men and 150 kg for women."
                },
                "lifted_kg": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Bench press or summed three-lift competition total in kg"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: WilksInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc(sex: Sex, bodyweight_kg: f64, lifted_kg: f64) -> WilksInput {
        WilksInput {
            sex,
            lift_type: LiftType::ThreeLiftTotal,
            bodyweight_kg,
            lifted_kg,
        }
    }

    #[test]
    fn men_coefficient_at_100kg_matches_published_table() {
        // Source-derived vector: europowerlifting.org Wilks table for men, BWT
        // 100.0 = 0.6086.
        let o = compute(&calc(Sex::Male, 100.0, 500.0)).unwrap();
        assert!(
            (o.coefficient - 0.6086).abs() < 0.0001,
            "got {:.4}",
            o.coefficient
        );
    }

    #[test]
    fn men_coefficient_at_69_3kg_matches_published_example() {
        // The source PDF's own worked example: 69.3 kg -> 0.7552.
        let o = compute(&calc(Sex::Male, 69.3, 200.0)).unwrap();
        assert!(
            (o.coefficient - 0.7552).abs() < 0.0001,
            "got {:.4}",
            o.coefficient
        );
    }

    #[test]
    fn women_coefficient_at_100kg_matches_published_table() {
        // Source-derived vector: europowerlifting.org Wilks table for women,
        // BWT 100.0 = 0.8326.
        let o = compute(&calc(Sex::Female, 100.0, 300.0)).unwrap();
        assert!(
            (o.coefficient - 0.8326).abs() < 0.0001,
            "got {:.4}",
            o.coefficient
        );
    }

    #[test]
    fn score_is_coefficient_times_lifted_weight() {
        let o = compute(&calc(Sex::Male, 100.0, 500.0)).unwrap();
        assert!((o.score - o.coefficient * 500.0).abs() < 0.01);
    }

    #[test]
    fn accepts_published_bodyweight_boundaries() {
        assert!(compute(&calc(Sex::Male, 40.0, 100.0)).is_ok());
        assert!(compute(&calc(Sex::Male, 205.0, 100.0)).is_ok());
        assert!(compute(&calc(Sex::Female, 40.0, 100.0)).is_ok());
        assert!(compute(&calc(Sex::Female, 150.0, 100.0)).is_ok());
    }

    #[test]
    fn rejects_bodyweight_below_published_table() {
        assert!(compute(&calc(Sex::Male, 39.9, 100.0)).is_err());
        assert!(compute(&calc(Sex::Female, 39.9, 100.0)).is_err());
    }

    #[test]
    fn caps_coefficients_above_source_table_ceiling() {
        // Marksteiner's IPF replacement proposal states that Wilks coefficients
        // are constant above 205 kg for men and 150 kg for women.
        let man = compute(&calc(Sex::Male, 250.0, 500.0)).unwrap();
        assert_eq!(man.coefficient_bodyweight_kg, 205.0);
        assert!(man.coefficient_was_capped);
        assert_eq!(man.coefficient, 0.5317);

        let woman = compute(&calc(Sex::Female, 200.0, 300.0)).unwrap();
        assert_eq!(woman.coefficient_bodyweight_kg, 150.0);
        assert!(woman.coefficient_was_capped);
        assert_eq!(woman.coefficient, 0.7695);
    }

    #[test]
    fn does_not_report_capping_at_the_ceiling() {
        let o = compute(&calc(Sex::Male, 205.0, 500.0)).unwrap();
        assert!(!o.coefficient_was_capped);
    }

    #[test]
    fn rejects_non_positive_lifted_weight() {
        assert!(compute(&calc(Sex::Male, 100.0, 0.0)).is_err());
        assert!(compute(&calc(Sex::Male, 100.0, -10.0)).is_err());
    }

    #[test]
    fn does_not_apply_an_unsupported_lifted_weight_ceiling() {
        assert!(compute(&calc(Sex::Male, 100.0, 1500.1)).is_ok());
    }

    #[test]
    fn rejects_lifted_weight_that_overflows_the_score() {
        assert!(compute(&calc(Sex::Female, 40.0, f64::MAX)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "sex": "female",
            "lift_type": "bench_press",
            "bodyweight_kg": 63.0,
            "lifted_kg": 250.0
        });
        let dynamic = Wilks.calculate(&value).unwrap();
        let typed = build_response(&WilksInput {
            sex: Sex::Female,
            lift_type: LiftType::BenchPress,
            bodyweight_kg: 63.0,
            lifted_kg: 250.0,
        })
        .unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn rejects_unvalidated_individual_lift_types() {
        let value = json!({
            "sex": "male",
            "lift_type": "deadlift",
            "bodyweight_kg": 100.0,
            "lifted_kg": 300.0
        });
        assert!(Wilks.calculate(&value).is_err());
    }

    #[test]
    fn response_preserves_inputs_and_coefficient() {
        let response = build_response(&calc(Sex::Male, 100.0, 500.0)).unwrap();
        assert_eq!(response.working["sex"], json!("male"));
        assert_eq!(response.working["lift_type"], json!("three_lift_total"));
        assert_eq!(response.working["bodyweight_kg"], json!(100.0));
        assert_eq!(response.working["lifted_kg"], json!(500.0));
        assert_eq!(response.working["coefficient_bodyweight_kg"], json!(100.0));
        assert_eq!(response.working["coefficient_was_capped"], json!(false));
        assert_eq!(response.working["coefficient"], json!(0.6086));
        assert_eq!(response.result, json!(304.3));
    }

    #[test]
    fn schema_exposes_supported_lifts_and_cap_behavior() {
        let schema = Wilks.input_schema();
        assert_eq!(
            schema["properties"]["lift_type"]["enum"],
            json!(["bench_press", "three_lift_total"])
        );
        assert_eq!(schema["properties"]["bodyweight_kg"]["minimum"], json!(40));
        assert!(
            schema["properties"]["bodyweight_kg"]
                .get("maximum")
                .is_none()
        );
        assert!(schema["properties"]["lifted_kg"].get("maximum").is_none());
    }
}
