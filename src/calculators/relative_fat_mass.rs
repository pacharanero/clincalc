// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Relative fat mass (RFM).
//!
//! A sex-specific, unitless anthropometric estimator of whole-body fat
//! percentage derived from height and waist circumference alone (no weight
//! term). Woolcott & Bergman (2018) selected it from 365 candidate indices
//! using NHANES 1999-2004 (n = 12,581) for derivation and NHANES 2005-2006
//! (n = 3,456, ages 20-69) for validation against DXA-measured body fat, and
//! reported it outperforming BMI at predicting DXA body fat percentage and at
//! flagging DXA-defined obesity in that validation cohort.
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

/// The validation cohort's own quintile-derived comparison cutpoints for
/// DXA-defined obesity, used in the source study only to benchmark RFM
/// against BMI - not an established clinical diagnostic threshold.
const STUDY_COMPARISON_CUTOFF_MALE: f64 = 22.8;
const STUDY_COMPARISON_CUTOFF_FEMALE: f64 = 33.9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelativeFatMassInput {
    pub sex: Sex,
    /// Age in years; the validation cohort was 20-69.
    pub age_years: u32,
    /// Standing height in centimetres.
    pub height_cm: f64,
    /// Waist circumference in centimetres.
    pub waist_cm: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelativeFatMassOutcome {
    pub rfm_percent: f64,
    pub study_comparison_cutoff: f64,
    pub at_or_above_study_comparison_cutoff: bool,
    pub interpretation: String,
}

fn study_comparison_cutoff(sex: Sex) -> f64 {
    match sex {
        Sex::Male => STUDY_COMPARISON_CUTOFF_MALE,
        Sex::Female => STUDY_COMPARISON_CUTOFF_FEMALE,
    }
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub fn compute(input: &RelativeFatMassInput) -> Result<RelativeFatMassOutcome, CalcError> {
    if !(20..=69).contains(&input.age_years) {
        return Err(CalcError::InvalidInput(
            "age_years must be between 20 and 69 - the range the RFM equation was derived and validated against".into(),
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
    if input.waist_cm > input.height_cm {
        return Err(CalcError::InvalidInput(
            "waist_cm cannot exceed height_cm - check measurements".into(),
        ));
    }

    let ratio = input.height_cm / input.waist_cm;
    let base = match input.sex {
        Sex::Male => 64.0,
        Sex::Female => 76.0,
    };
    let rfm_percent = round1(base - 20.0 * ratio);

    if !rfm_percent.is_finite() {
        return Err(CalcError::InvalidInput(
            "computed relative fat mass is invalid - check measurement inputs".into(),
        ));
    }

    let cutoff = study_comparison_cutoff(input.sex);
    let at_or_above_study_comparison_cutoff = rfm_percent >= cutoff;
    let sex_label = match input.sex {
        Sex::Male => "male",
        Sex::Female => "female",
    };
    let boundary_note = if at_or_above_study_comparison_cutoff {
        format!(
            "at or above the source study's own quintile-derived comparison cutpoint of {cutoff}% for {sex_label}s"
        )
    } else {
        format!(
            "below the source study's own quintile-derived comparison cutpoint of {cutoff}% for {sex_label}s"
        )
    };

    let interpretation = format!(
        "Estimated whole-body fat {rfm_percent:.1}% by the Woolcott-Bergman relative fat mass (RFM) equation ({sex_label}, {boundary_note}). RFM was derived and validated in adults aged 20-69 and better predicted DXA-measured body fat than BMI in that cohort, but the comparison cutpoint was arbitrarily chosen for that study and is not an established clinical diagnostic threshold. RFM is one input into adiposity assessment, not a diagnosis on its own."
    );

    Ok(RelativeFatMassOutcome {
        rfm_percent,
        study_comparison_cutoff: cutoff,
        at_or_above_study_comparison_cutoff,
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
        "study_comparison_cutoff".into(),
        json!(o.study_comparison_cutoff),
    );
    working.insert(
        "at_or_above_study_comparison_cutoff".into(),
        json!(o.at_or_above_study_comparison_cutoff),
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
        "Sex-specific estimate of whole-body fat percentage from height and waist circumference alone (Woolcott-Bergman 2018), validated against DXA."
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
                    "description": "Sex (determines which RFM equation and comparison cutpoint is used)"
                },
                "age_years": {
                    "type": "integer",
                    "minimum": 20,
                    "maximum": 69,
                    "description": "Age in years; the RFM equation was derived and validated in adults aged 20-69"
                },
                "height_cm": {
                    "type": "number",
                    "minimum": 50,
                    "maximum": 250,
                    "description": "Standing height in centimetres"
                },
                "waist_cm": {
                    "type": "number",
                    "minimum": 30,
                    "maximum": 250,
                    "description": "Waist circumference in centimetres"
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
    fn male_formula_vector() {
        // 175 cm / 90 cm waist -> 64 - 20*(175/90) = 64 - 38.888... = 25.1
        let o = compute(&calc(Sex::Male, 40, 175.0, 90.0)).unwrap();
        assert!((o.rfm_percent - 25.1).abs() < 0.05, "got {}", o.rfm_percent);
        assert!(o.at_or_above_study_comparison_cutoff);
    }

    #[test]
    fn female_formula_vector() {
        // 165 cm / 80 cm waist -> 76 - 20*(165/80) = 76 - 41.25 = 34.75 -> 34.8
        let o = compute(&calc(Sex::Female, 40, 165.0, 80.0)).unwrap();
        assert!((o.rfm_percent - 34.8).abs() < 0.05, "got {}", o.rfm_percent);
        assert!(o.at_or_above_study_comparison_cutoff);
    }

    #[test]
    fn male_below_study_comparison_cutoff() {
        // 180 cm / 75 cm waist -> 64 - 20*(180/75) = 64 - 48 = 16.0
        let o = compute(&calc(Sex::Male, 30, 180.0, 75.0)).unwrap();
        assert!((o.rfm_percent - 16.0).abs() < 0.05, "got {}", o.rfm_percent);
        assert!(!o.at_or_above_study_comparison_cutoff);
        assert!(
            o.interpretation
                .contains("below the source study's own quintile-derived comparison cutpoint")
        );
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
    fn rejects_waist_greater_than_height() {
        assert!(compute(&calc(Sex::Male, 40, 100.0, 150.0)).is_err());
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
    fn response_preserves_inputs_and_cutoff() {
        let response = build_response(&calc(Sex::Male, 40, 175.0, 90.0)).unwrap();
        assert_eq!(response.working["sex"], json!("male"));
        assert_eq!(response.working["age_years"], json!(40));
        assert_eq!(response.working["height_cm"], json!(175.0));
        assert_eq!(response.working["waist_cm"], json!(90.0));
        assert_eq!(response.working["study_comparison_cutoff"], json!(22.8));
        assert_eq!(
            response.working["at_or_above_study_comparison_cutoff"],
            json!(true)
        );
    }
}
