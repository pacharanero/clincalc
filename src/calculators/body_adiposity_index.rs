// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Body adiposity index (BAI).
//!
//! A weight-free, unitless anthropometric estimator of whole-body fat
//! percentage derived from height and hip circumference alone. Bergman et al
//! (2011) fit the equation by regression against DXA-measured body fat in the
//! "BetaGene" cohort of Mexican-American adults (n = 1,733, ages 18-67;
//! R = 0.79 vs DXA) and externally validated it in the "TARA" cohort of
//! African-American adults (n = 223, ages 20-50; R = 0.85, concordance
//! correlation coefficient 0.95 vs DXA). The source study states that BAI's
//! utility "has not yet been confirmed" in Caucasian or other ethnic groups,
//! or in children, and it does not establish diagnostic BAI cut-points.
//!
//! BAI = (hip circumference in cm / (height in m)^1.5) - 18
//!
//! Reference: Bergman RN, Stefanovski D, Buchanan TA, et al. A better index
//! of body adiposity. Obesity (Silver Spring). 2011;19(5):1083-1089.
//! doi:10.1038/oby.2011.38.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "body_adiposity_index";

pub const REFERENCE: &str = "Bergman RN, Stefanovski D, Buchanan TA, et al. A better index of body adiposity. Obesity (Silver Spring). 2011;19(5):1083-1089. doi:10.1038/oby.2011.38.";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1038/oby.2011.38",
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyAdiposityIndexInput {
    /// Age in years; restricted to the range confirmed by both cohorts.
    pub age_years: u32,
    /// Standing height in centimetres.
    pub height_cm: f64,
    /// Hip circumference in centimetres, at the widest point of the hips/buttocks.
    pub hip_cm: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BodyAdiposityIndexOutcome {
    pub bai_percent: f64,
    pub interpretation: String,
}

pub fn compute(input: &BodyAdiposityIndexInput) -> Result<BodyAdiposityIndexOutcome, CalcError> {
    if !(20..=50).contains(&input.age_years) {
        return Err(CalcError::InvalidInput(
            "age_years must be between 20 and 50 - the range with confirmed DXA agreement across both the BetaGene derivation cohort (ages 18-67) and the TARA external-validation cohort (ages 20-50)".into(),
        ));
    }
    if !(50.0..=250.0).contains(&input.height_cm) || !input.height_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "height_cm must be finite and between 50 and 250".into(),
        ));
    }
    if !(30.0..=250.0).contains(&input.hip_cm) || !input.hip_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "hip_cm must be finite and between 30 and 250".into(),
        ));
    }

    let height_m = input.height_cm / 100.0;
    let hip_to_height_ratio = input.hip_cm / height_m.powf(1.5);
    let bai_percent = hip_to_height_ratio - 18.0;

    if !(0.0..=100.0).contains(&bai_percent) {
        return Err(CalcError::InvalidInput(
            "computed body adiposity index is outside the plausible percentage range 0-100% - check measurement inputs and model applicability".into(),
        ));
    }

    let interpretation = format!(
        "Estimated whole-body fat {bai_percent:.1}% by the Bergman body adiposity index (BAI) equation (hip circumference / height^1.5 - 18), from height and hip circumference alone, without a weight term. The equation was fit by regression against DXA in the BetaGene cohort of Mexican-American adults (n=1,733, ages 18-67; R=0.79 vs DXA) and externally validated in the TARA cohort of African-American adults (n=223, ages 20-50; R=0.85, concordance correlation coefficient 0.95 vs DXA). The primary source states BAI's utility has not yet been confirmed in Caucasian or other ethnic groups, or in children, and it does not establish diagnostic BAI cut-points; this estimate does not diagnose obesity."
    );

    Ok(BodyAdiposityIndexOutcome {
        bai_percent,
        interpretation,
    })
}

pub fn build_response(input: &BodyAdiposityIndexInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("age_years".into(), json!(input.age_years));
    working.insert("height_cm".into(), json!(input.height_cm));
    working.insert("hip_cm".into(), json!(input.hip_cm));
    let height_m = input.height_cm / 100.0;
    working.insert("height_m".into(), json!(height_m));
    working.insert(
        "hip_to_height_ratio".into(),
        json!(input.hip_cm / height_m.powf(1.5)),
    );
    working.insert("height_exponent".into(), json!(1.5));
    working.insert("equation_constant".into(), json!(-18.0));
    working.insert("bai_percent".into(), json!(o.bai_percent));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.bai_percent),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct BodyAdiposityIndex;

impl Calculator for BodyAdiposityIndex {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Body Adiposity Index (BAI)"
    }

    fn description(&self) -> &'static str {
        "Weight-free estimate of whole-body fat percentage from height and hip circumference (Bergman 2011), fit and validated against DXA in Mexican-American and African-American adults aged 20-50; not confirmed in other ethnic groups or in children, and not a diagnostic obesity classification."
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
            "title": "BodyAdiposityIndexInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["age_years", "height_cm", "hip_cm"],
            "properties": {
                "age_years": {
                    "type": "integer",
                    "minimum": 20,
                    "maximum": 50,
                    "description": "Age in years, restricted to 20-50 - the range with confirmed DXA agreement across both the BetaGene derivation cohort (ages 18-67) and the TARA external-validation cohort (ages 20-50)"
                },
                "height_cm": {
                    "type": "number",
                    "minimum": 50,
                    "maximum": 250,
                    "description": "Standing height in centimetres"
                },
                "hip_cm": {
                    "type": "number",
                    "minimum": 30,
                    "maximum": 250,
                    "description": "Hip circumference in centimetres, measured at the widest point of the hips/buttocks"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: BodyAdiposityIndexInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc(age: u32, height: f64, hip: f64) -> BodyAdiposityIndexInput {
        BodyAdiposityIndexInput {
            age_years: age,
            height_cm: height,
            hip_cm: hip,
        }
    }

    #[test]
    fn source_equation_vector() {
        // Bergman 2011: BAI = hip_cm / (height_m ^ 1.5) - 18.
        let o = compute(&calc(35, 175.0, 100.0)).unwrap();
        assert!(
            (o.bai_percent - 25.195_939_772_483_108).abs() < 1e-9,
            "got {}",
            o.bai_percent
        );
    }

    #[test]
    fn second_source_equation_vector() {
        let o = compute(&calc(35, 160.0, 110.0)).unwrap();
        assert!(
            (o.bai_percent - 36.351_647_284_144_01).abs() < 1e-9,
            "got {}",
            o.bai_percent
        );
    }

    #[test]
    fn accepts_confirmed_age_boundaries() {
        assert!(compute(&calc(20, 175.0, 100.0)).is_ok());
        assert!(compute(&calc(50, 175.0, 100.0)).is_ok());
    }

    #[test]
    fn rejects_age_outside_confirmed_range() {
        assert!(compute(&calc(19, 175.0, 100.0)).is_err());
        assert!(compute(&calc(51, 175.0, 100.0)).is_err());
    }

    #[test]
    fn rejects_out_of_range_height() {
        assert!(compute(&calc(35, 10.0, 100.0)).is_err());
        assert!(compute(&calc(35, 300.0, 100.0)).is_err());
    }

    #[test]
    fn rejects_out_of_range_hip() {
        assert!(compute(&calc(35, 175.0, 10.0)).is_err());
        assert!(compute(&calc(35, 175.0, 300.0)).is_err());
    }

    #[test]
    fn rejects_measurement_combination_producing_negative_percentage() {
        assert_eq!(
            compute(&calc(35, 250.0, 30.0)),
            Err(CalcError::InvalidInput(
                "computed body adiposity index is outside the plausible percentage range 0-100% - check measurement inputs and model applicability".into()
            ))
        );
    }

    #[test]
    fn rejects_measurement_combination_producing_implausibly_high_percentage() {
        assert!(compute(&calc(35, 100.0, 150.0)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age_years": 35,
            "height_cm": 175.0,
            "hip_cm": 100.0
        });
        let dynamic = BodyAdiposityIndex.calculate(&value).unwrap();
        let typed = build_response(&calc(35, 175.0, 100.0)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn response_preserves_inputs_and_equation_working() {
        let response = build_response(&calc(35, 175.0, 100.0)).unwrap();
        assert_eq!(response.working["age_years"], json!(35));
        assert_eq!(response.working["height_cm"], json!(175.0));
        assert_eq!(response.working["hip_cm"], json!(100.0));
        assert_eq!(response.working["height_exponent"], json!(1.5));
        assert_eq!(response.working["equation_constant"], json!(-18.0));
        assert_eq!(response.result, response.working["bai_percent"]);
        assert!(
            response
                .interpretation
                .contains("does not establish diagnostic BAI cut-points")
        );
    }

    #[test]
    fn schema_documents_source_cohorts_and_applicability_contract() {
        let schema = BodyAdiposityIndex.input_schema();
        let age = schema["properties"]["age_years"]["description"]
            .as_str()
            .unwrap();
        assert!(age.contains("BetaGene derivation cohort (ages 18-67)"));
        assert!(age.contains("TARA external-validation cohort (ages 20-50)"));

        let hip = schema["properties"]["hip_cm"]["description"]
            .as_str()
            .unwrap();
        assert!(hip.contains("widest point of the hips/buttocks"));
    }
}
