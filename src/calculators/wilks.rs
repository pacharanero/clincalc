// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Wilks coefficient - bodyweight-adjusted powerlifting score.
//!
//! Scales a lifted weight (a single lift or a competition total) by a
//! sex-specific quintic-polynomial function of bodyweight, so that lifters of
//! different bodyweights and sexes can be compared on one scale. Used by the
//! International Powerlifting Federation from 1994 to 2019, and still widely
//! used alongside its successors (DOTS, IPF GL Points).
//!
//! `coefficient = 500 / (a + b*x + c*x^2 + d*x^3 + e*x^4 + f*x^5)`, where `x`
//! is bodyweight in kilograms. `score = coefficient * lifted_kg`.
//!
//! The published coefficient tables cover bodyweights from 40 kg up to 205 kg
//! (men) and 150 kg (women); outside that range the quintic is not validated
//! and can behave non-monotonically, so this calculator rejects bodyweights
//! outside the published range rather than extrapolate.
//!
//! Coefficients as tabulated by Robert Wilks (Australia) and reproduced at
//! <https://www.europowerlifting.org/fileadmin/data/wilks_formula/Wilksformula_01.pdf>;
//! independently verified here against the published lookup tables at
//! bodyweights 69.3 kg (men) and 100.0 kg (men and women).
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

pub const REFERENCE: &str = "Vanderburgh PM, Batterham AM. Validation of the Wilks powerlifting formula. Med Sci Sports Exerc. 1999;31(12):1869-1875. doi:10.1097/00005768-199912000-00027. Coefficients as tabulated by Robert Wilks (Australia); used by the International Powerlifting Federation 1994-2019.";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain formula - historically the International Powerlifting Federation's official scoring coefficient",
    source_url: "https://doi.org/10.1097/00005768-199912000-00027",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WilksInput {
    pub sex: Sex,
    /// Bodyweight in kilograms.
    pub bodyweight_kg: f64,
    /// Weight lifted in kilograms - a single lift or a summed competition total.
    pub lifted_kg: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WilksOutcome {
    pub coefficient: f64,
    pub score: f64,
    pub interpretation: String,
}

fn bodyweight_range(sex: Sex) -> (f64, f64) {
    match sex {
        Sex::Male => (40.0, 205.0),
        Sex::Female => (40.0, 150.0),
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
    500.0 / (a + b * x + c * x.powi(2) + d * x.powi(3) + e * x.powi(4) + f * x.powi(5))
}

fn round4(v: f64) -> f64 {
    (v * 10_000.0).round() / 10_000.0
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub fn compute(input: &WilksInput) -> Result<WilksOutcome, CalcError> {
    let (min_bw, max_bw) = bodyweight_range(input.sex);
    if !input.bodyweight_kg.is_finite() || !(min_bw..=max_bw).contains(&input.bodyweight_kg) {
        return Err(CalcError::InvalidInput(format!(
            "bodyweight_kg must be finite and between {min_bw} and {max_bw} for this sex - the published coefficient tables are not validated outside that range"
        )));
    }
    if !input.lifted_kg.is_finite() || input.lifted_kg <= 0.0 || input.lifted_kg > 1500.0 {
        return Err(CalcError::InvalidInput(
            "lifted_kg must be finite, positive, and no more than 1500".into(),
        ));
    }

    let coefficient = round4(raw_coefficient(input.sex, input.bodyweight_kg));
    let score = round2(coefficient * input.lifted_kg);
    let sex_label = match input.sex {
        Sex::Male => "male",
        Sex::Female => "female",
    };

    let interpretation = format!(
        "Wilks coefficient {coefficient:.4} at {:.1} kg bodyweight ({sex_label}), applied to {:.1} kg lifted gives a Wilks score of {score:.2}. The score allows comparison of lifters of different bodyweights and sexes on one scale; higher is a relatively stronger performance. Superseded as the IPF's official formula by DOTS (2019) and IPF GL Points (2020), but still widely reported.",
        input.bodyweight_kg, input.lifted_kg
    );

    Ok(WilksOutcome {
        coefficient,
        score,
        interpretation,
    })
}

pub fn build_response(input: &WilksInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("sex".into(), json!(input.sex));
    working.insert("bodyweight_kg".into(), json!(input.bodyweight_kg));
    working.insert("lifted_kg".into(), json!(input.lifted_kg));
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
        "Bodyweight- and sex-adjusted powerlifting score from a lifted weight or competition total, using the 1994-2019 IPF-era Wilks coefficient."
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
            "required": ["sex", "bodyweight_kg", "lifted_kg"],
            "properties": {
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Sex (determines which coefficient polynomial is applied)"
                },
                "bodyweight_kg": {
                    "type": "number",
                    "minimum": 40,
                    "maximum": 205,
                    "description": "Bodyweight in kg. The published coefficient tables are validated for 40-205 kg (men) and 40-150 kg (women); values are rejected outside the range for the given sex."
                },
                "lifted_kg": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "maximum": 1500,
                    "description": "Weight lifted in kg - a single lift or a summed competition total"
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
    fn accepts_bodyweight_boundaries() {
        assert!(compute(&calc(Sex::Male, 40.0, 100.0)).is_ok());
        assert!(compute(&calc(Sex::Male, 205.0, 100.0)).is_ok());
        assert!(compute(&calc(Sex::Female, 40.0, 100.0)).is_ok());
        assert!(compute(&calc(Sex::Female, 150.0, 100.0)).is_ok());
    }

    #[test]
    fn rejects_bodyweight_outside_published_range() {
        assert!(compute(&calc(Sex::Male, 39.9, 100.0)).is_err());
        assert!(compute(&calc(Sex::Male, 205.1, 100.0)).is_err());
        assert!(compute(&calc(Sex::Female, 150.1, 100.0)).is_err());
    }

    #[test]
    fn rejects_non_positive_lifted_weight() {
        assert!(compute(&calc(Sex::Male, 100.0, 0.0)).is_err());
        assert!(compute(&calc(Sex::Male, 100.0, -10.0)).is_err());
    }

    #[test]
    fn rejects_excessive_lifted_weight() {
        assert!(compute(&calc(Sex::Male, 100.0, 1500.1)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "sex": "female",
            "bodyweight_kg": 63.0,
            "lifted_kg": 250.0
        });
        let dynamic = Wilks.calculate(&value).unwrap();
        let typed = build_response(&calc(Sex::Female, 63.0, 250.0)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn response_preserves_inputs_and_coefficient() {
        let response = build_response(&calc(Sex::Male, 100.0, 500.0)).unwrap();
        assert_eq!(response.working["sex"], json!("male"));
        assert_eq!(response.working["bodyweight_kg"], json!(100.0));
        assert_eq!(response.working["lifted_kg"], json!(500.0));
        assert_eq!(response.working["coefficient"], json!(0.6086));
        assert_eq!(response.result, json!(304.3));
    }
}
