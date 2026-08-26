// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Relative fat mass (RFM).
//!
//! A sex-specific, unitless anthropometric estimator of whole-body fat
//! percentage derived from height and waist circumference alone (no weight
//! term). Woolcott & Bergman (2018) selected it from 365 candidate indices
//! using NHANES 1999-2004 (n = 12,581, ages 20-85) for derivation and NHANES
//! 2005-2006 (n = 3,456, ages 20-69) for validation against DXA-measured body
//! fat. The source study did not establish diagnostic RFM cut-points.
//!
//! Men:   RFM = 64 - (20 x height / waist)
//! Women: RFM = 76 - (20 x height / waist)
//!
//! Height and waist circumference use the same units, so cm cancels cleanly.
//!
//! Reference: Woolcott OO, Bergman RN. Relative fat mass (RFM) as a new
//! estimator of whole-body fat percentage - A cross-sectional study in
//! American adult individuals. Sci Rep. 2018;8(1):10980.
//! doi:10.1038/s41598-018-29362-1.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "relative_fat_mass";

pub const REFERENCE: &str = "Woolcott OO, Bergman RN. Relative fat mass (RFM) as a new estimator of whole-body fat percentage - A cross-sectional study in American adult individuals. Sci Rep. 2018;8(1):10980. doi:10.1038/s41598-018-29362-1.";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1038/s41598-018-29362-1",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelativeFatMassInput {
    /// Source-model coefficient category: male = 0, female = 1.
    pub sex: Sex,
    /// Age in years; restricted to the validation cohort's range.
    pub age_years: u32,
    /// Standing height in centimetres, measured with a stadiometer.
    pub height_cm: f64,
    /// Waist circumference in centimetres at the uppermost lateral border of
    /// the right ilium, standing, at the end of expiration.
    pub waist_cm: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelativeFatMassOutcome {
    pub rfm_percent: f64,
    pub interpretation: String,
}

pub fn compute(input: &RelativeFatMassInput) -> Result<RelativeFatMassOutcome, CalcError> {
    if !(20..=69).contains(&input.age_years) {
        return Err(CalcError::InvalidInput(
            "age_years must be between 20 and 69 - the range evaluated in the validation cohort"
                .into(),
        ));
    }
    if !(50.0..=250.0).contains(&input.height_cm) || !input.height_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "height_cm must be finite and between 50 and 250".into(),
        ));
    }
    if !(30.0..=250.0).contains(&input.waist_cm) || !input.waist_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "waist_cm must be finite and between 30 and 250".into(),
        ));
    }
    let ratio = input.height_cm / input.waist_cm;
    let sex_indicator = match input.sex {
        Sex::Male => 0.0,
        Sex::Female => 1.0,
    };
    let rfm_percent = 64.0 - 20.0 * ratio + 12.0 * sex_indicator;

    if !(0.0..=100.0).contains(&rfm_percent) {
        return Err(CalcError::InvalidInput(
            "computed relative fat mass is outside the percentage range 0-100% - check measurement inputs and model applicability".into(),
        ));
    }

    let sex_label = match input.sex {
        Sex::Male => "male",
        Sex::Female => "female",
    };

    let interpretation = format!(
        "Estimated whole-body fat {rfm_percent:.1}% by the Woolcott-Bergman relative fat mass (RFM) equation ({sex_label}). The equation was developed in US adults aged 20-85 and evaluated in a separate nationally representative US cohort aged 20-69 against DXA; accuracy decreased with age and was lower at lower body-fat levels. Ethnicity-specific evidence was limited to Mexican-American, European-American, and African-American adults, and the source cautions against extrapolation to other ethnic groups, children, athletes, people with specific diseases, or non-US populations. The study did not establish diagnostic RFM cut-points; this estimate does not diagnose obesity."
    );

    Ok(RelativeFatMassOutcome {
        rfm_percent,
        interpretation,
    })
}

pub fn build_response(input: &RelativeFatMassInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("sex".into(), json!(input.sex));
    working.insert("age_years".into(), json!(input.age_years));
    working.insert("height_cm".into(), json!(input.height_cm));
    working.insert("waist_cm".into(), json!(input.waist_cm));
    working.insert(
        "height_to_waist_ratio".into(),
        json!(input.height_cm / input.waist_cm),
    );
    working.insert("equation_constant".into(), json!(64.0));
    working.insert("ratio_coefficient".into(), json!(-20.0));
    working.insert("sex_coefficient".into(), json!(12.0));
    working.insert(
        "sex_indicator".into(),
        json!(match input.sex {
            Sex::Male => 0,
            Sex::Female => 1,
        }),
    );
    working.insert("rfm_percent".into(), json!(o.rfm_percent));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.rfm_percent),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct RelativeFatMass;

impl Calculator for RelativeFatMass {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Relative Fat Mass (RFM)"
    }

    fn description(&self) -> &'static str {
        "Sex-specific estimate of whole-body fat percentage from height and waist circumference (Woolcott-Bergman 2018), evaluated against DXA in US adults aged 20-69; not a diagnostic obesity classification."
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
            "title": "RelativeFatMassInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["sex", "age_years", "height_cm", "waist_cm"],
            "properties": {
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Source-model coefficient category: male = 0 and female = 1. The study did not derive a coefficient for another or unknown category."
                },
                "age_years": {
                    "type": "integer",
                    "minimum": 20,
                    "maximum": 69,
                    "description": "Age in years. The equation was developed at ages 20-85; this calculator is restricted to ages 20-69 evaluated in the separate validation cohort."
                },
                "height_cm": {
                    "type": "number",
                    "minimum": 50,
                    "maximum": 250,
                    "description": "Standing height in centimetres, measured with an electronic stadiometer in the source study"
                },
                "waist_cm": {
                    "type": "number",
                    "minimum": 30,
                    "maximum": 250,
                    "description": "Waist circumference in centimetres, measured on the unclothed standing participant in a horizontal plane at the uppermost lateral border of the right ilium, at the end of expiration, and recorded to the nearest 0.1 cm, as in the source study"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: RelativeFatMassInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc(sex: Sex, age: u32, height: f64, waist: f64) -> RelativeFatMassInput {
        RelativeFatMassInput {
            sex,
            age_years: age,
            height_cm: height,
            waist_cm: waist,
        }
    }

    #[test]
    fn male_source_equation_vector() {
        // Woolcott-Bergman 2018, equation 2: 64 - 20*(height/waist).
        let o = compute(&calc(Sex::Male, 40, 175.0, 90.0)).unwrap();
        assert!(
            (o.rfm_percent - 25.111_111_111_111_114).abs() < 1e-12,
            "got {}",
            o.rfm_percent
        );
    }

    #[test]
    fn female_source_equation_vector() {
        // Woolcott-Bergman 2018, equation 1: 76 - 20*(height/waist).
        let o = compute(&calc(Sex::Female, 40, 165.0, 80.0)).unwrap();
        assert!(
            (o.rfm_percent - 34.75).abs() < 1e-12,
            "got {}",
            o.rfm_percent
        );
    }

    #[test]
    fn accepts_validation_age_boundaries() {
        assert!(compute(&calc(Sex::Male, 20, 175.0, 90.0)).is_ok());
        assert!(compute(&calc(Sex::Male, 69, 175.0, 90.0)).is_ok());
    }

    #[test]
    fn rejects_age_outside_validated_range() {
        assert!(compute(&calc(Sex::Male, 19, 175.0, 90.0)).is_err());
        assert!(compute(&calc(Sex::Male, 70, 175.0, 90.0)).is_err());
    }

    #[test]
    fn rejects_out_of_range_height() {
        assert!(compute(&calc(Sex::Male, 40, 10.0, 90.0)).is_err());
        assert!(compute(&calc(Sex::Male, 40, 300.0, 90.0)).is_err());
    }

    #[test]
    fn rejects_out_of_range_waist() {
        assert!(compute(&calc(Sex::Male, 40, 175.0, 10.0)).is_err());
        assert!(compute(&calc(Sex::Male, 40, 175.0, 300.0)).is_err());
    }

    #[test]
    fn accepts_waist_greater_than_height_when_estimate_is_physical() {
        let o = compute(&calc(Sex::Male, 40, 100.0, 150.0)).unwrap();
        assert!((o.rfm_percent - 50.666_666_666_666_67).abs() < 1e-12);
    }

    #[test]
    fn rejects_measurement_combination_producing_negative_percentage() {
        assert_eq!(
            compute(&calc(Sex::Male, 40, 200.0, 50.0)),
            Err(CalcError::InvalidInput(
                "computed relative fat mass is outside the percentage range 0-100% - check measurement inputs and model applicability".into()
            ))
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "sex": "female",
            "age_years": 35,
            "height_cm": 165.0,
            "waist_cm": 80.0
        });
        let dynamic = RelativeFatMass.calculate(&value).unwrap();
        let typed = build_response(&calc(Sex::Female, 35, 165.0, 80.0)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn response_preserves_inputs_and_equation_working() {
        let response = build_response(&calc(Sex::Male, 40, 175.0, 90.0)).unwrap();
        assert_eq!(response.working["sex"], json!("male"));
        assert_eq!(response.working["age_years"], json!(40));
        assert_eq!(response.working["height_cm"], json!(175.0));
        assert_eq!(response.working["waist_cm"], json!(90.0));
        assert_eq!(response.working["equation_constant"], json!(64.0));
        assert_eq!(response.working["ratio_coefficient"], json!(-20.0));
        assert_eq!(response.working["sex_coefficient"], json!(12.0));
        assert_eq!(response.working["sex_indicator"], json!(0));
        assert!(response.working.get("study_comparison_cutoff").is_none());
        assert!(
            response
                .working
                .get("at_or_above_study_comparison_cutoff")
                .is_none()
        );
        assert!(
            response
                .interpretation
                .contains("did not establish diagnostic RFM cut-points")
        );
        assert_eq!(response.result, response.working["rfm_percent"]);
    }

    #[test]
    fn schema_documents_source_measurement_and_applicability_contract() {
        let schema = RelativeFatMass.input_schema();
        let waist = schema["properties"]["waist_cm"]["description"]
            .as_str()
            .unwrap();
        assert!(waist.contains("uppermost lateral border of the right ilium"));
        assert!(waist.contains("end of expiration"));
        assert!(waist.contains("nearest 0.1 cm"));

        let height = schema["properties"]["height_cm"]["description"]
            .as_str()
            .unwrap();
        assert!(height.contains("stadiometer"));

        let age = schema["properties"]["age_years"]["description"]
            .as_str()
            .unwrap();
        assert!(age.contains("developed at ages 20-85"));
        assert!(age.contains("restricted to ages 20-69"));

        let sex = schema["properties"]["sex"]["description"].as_str().unwrap();
        assert!(sex.contains("male = 0 and female = 1"));
        assert!(sex.contains("another or unknown category"));
    }
}
