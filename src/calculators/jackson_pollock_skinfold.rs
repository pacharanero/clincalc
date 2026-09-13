// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Jackson-Pollock 3-site skinfold body fat percentage.
//!
//! Caliper-derived body density from three sex-specific skinfold sites, then
//! converted to body fat percentage by the Siri equation. Jackson & Pollock
//! (1978) derived the men's equation from 308 and 95 adult men aged 18-61,
//! validated against hydrostatic (underwater) weighing. Jackson, Pollock &
//! Ward (1980) derived the women's equation from 249 women aged 18-55 by the
//! same hydrostatic method, and cautioned that accuracy needs care above age
//! 40. Both equations report a standard error of estimate of roughly 3-4%
//! body fat and were developed in largely Caucasian North American samples;
//! they are not universally validated across all body types, and are known to
//! progressively underestimate body fat in obese individuals.
//!
//! Men (chest, abdomen, thigh):
//!   Body density = 1.10938 - 0.0008267 x S + 0.0000016 x S^2 - 0.0002574 x age
//!
//! Women (triceps, suprailiac, thigh):
//!   Body density = 1.0994921 - 0.0009929 x S + 0.0000023 x S^2 - 0.0001392 x age
//!
//! where S is the sum of the three skinfolds in millimetres and age is in
//! years. Body fat percentage then follows the Siri (1961) two-compartment
//! equation: %BF = (495 / body density) - 450.
//!
//! Each skinfold is a vertical or diagonal pinch of skin and subcutaneous fat
//! (not muscle), read to the nearest 0.5 mm on a calibrated caliper roughly
//! one second after the jaws close, always on the right side of the body:
//! - Chest: diagonal fold, halfway between the anterior axillary line and the
//!   nipple.
//! - Abdomen: vertical fold, 2 cm to the right of the umbilicus.
//! - Thigh: vertical fold on the anterior midline, midway between the
//!   proximal border of the patella and the inguinal crease.
//! - Triceps: vertical fold on the posterior midline of the upper arm,
//!   midway between the acromion and olecranon processes, arm relaxed.
//! - Suprailiac: diagonal fold immediately above the iliac crest, in line
//!   with its natural angle, at the anterior axillary line.
//!
//! References: Jackson AS, Pollock ML. Generalized equations for predicting
//! body density of men. Br J Nutr. 1978;40(3):497-504.
//! doi:10.1079/bjn19780152. Jackson AS, Pollock ML, Ward A. Generalized
//! equations for predicting body density of women. Med Sci Sports Exerc.
//! 1980;12(3):175-181. Siri WE. Body composition from fluid spaces and
//! density: analysis of methods. 1961. Reprinted in Nutrition.
//! 1993;9(5):480-491; discussion 480, 492.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "jackson_pollock_skinfold";

pub const REFERENCE: &str = "Jackson AS, Pollock ML. Generalized equations for predicting body density of men. Br J Nutr. 1978;40(3):497-504. doi:10.1079/bjn19780152. Jackson AS, Pollock ML, Ward A. Generalized equations for predicting body density of women. Med Sci Sports Exerc. 1980;12(3):175-181. Siri WE. Body composition from fluid spaces and density: analysis of methods. 1961. Reprinted in Nutrition. 1993;9(5):480-491; discussion 480, 492.";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published mathematical method - independently implemented; formulas and algorithms are not protected by US copyright",
    source_url: "https://www.copyright.gov/circs/circ31.pdf",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JacksonPollockSkinfoldInput {
    pub sex: Sex,
    /// Age in years; restricted to each equation's validation cohort range.
    pub age_years: u32,
    /// Anterior thigh skinfold in mm, midway between patella and inguinal
    /// crease. Required for both sexes.
    pub thigh_mm: f64,
    /// Chest skinfold in mm, diagonal, halfway between the anterior axillary
    /// line and the nipple. Required for male; not used for female.
    pub chest_mm: Option<f64>,
    /// Abdominal skinfold in mm, vertical, 2 cm right of the umbilicus.
    /// Required for male; not used for female.
    pub abdomen_mm: Option<f64>,
    /// Triceps skinfold in mm, vertical, midway between acromion and
    /// olecranon. Required for female; not used for male.
    pub triceps_mm: Option<f64>,
    /// Suprailiac skinfold in mm, diagonal, immediately above the iliac
    /// crest. Required for female; not used for male.
    pub suprailiac_mm: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JacksonPollockSkinfoldOutcome {
    pub sum_skinfolds_mm: f64,
    pub body_density: f64,
    pub body_fat_percent: f64,
    pub interpretation: String,
}

const SKINFOLD_RANGE_MM: std::ops::RangeInclusive<f64> = 2.0..=100.0;

fn require_skinfold(value: Option<f64>, field: &str) -> Result<f64, CalcError> {
    let v = value.ok_or_else(|| CalcError::InvalidInput(format!("{field} is required")))?;
    if !SKINFOLD_RANGE_MM.contains(&v) || !v.is_finite() {
        return Err(CalcError::InvalidInput(format!(
            "{field} must be finite and between 2 and 100 mm"
        )));
    }
    Ok(v)
}

pub fn compute(
    input: &JacksonPollockSkinfoldInput,
) -> Result<JacksonPollockSkinfoldOutcome, CalcError> {
    if !SKINFOLD_RANGE_MM.contains(&input.thigh_mm) || !input.thigh_mm.is_finite() {
        return Err(CalcError::InvalidInput(
            "thigh_mm must be finite and between 2 and 100 mm".into(),
        ));
    }

    let (sum, age_range, age_coefficient, constant, linear_coefficient, quadratic_coefficient) =
        match input.sex {
            Sex::Male => {
                let chest = require_skinfold(input.chest_mm, "chest_mm")?;
                let abdomen = require_skinfold(input.abdomen_mm, "abdomen_mm")?;
                (
                    chest + abdomen + input.thigh_mm,
                    18..=61,
                    0.0002574,
                    1.10938,
                    0.0008267,
                    0.0000016,
                )
            }
            Sex::Female => {
                let triceps = require_skinfold(input.triceps_mm, "triceps_mm")?;
                let suprailiac = require_skinfold(input.suprailiac_mm, "suprailiac_mm")?;
                (
                    triceps + suprailiac + input.thigh_mm,
                    18..=55,
                    0.0001392,
                    1.0994921,
                    0.0009929,
                    0.0000023,
                )
            }
        };

    if !age_range.contains(&input.age_years) {
        return Err(CalcError::InvalidInput(format!(
            "age_years must be between {} and {} - the range evaluated in the validation cohort for this sex",
            age_range.start(),
            age_range.end()
        )));
    }

    let age = input.age_years as f64;
    let body_density = constant - linear_coefficient * sum + quadratic_coefficient * sum * sum
        - age_coefficient * age;

    if !body_density.is_finite() || body_density <= 0.0 {
        return Err(CalcError::InvalidInput(
            "computed body density is invalid - check inputs".into(),
        ));
    }

    let body_fat_percent = 495.0 / body_density - 450.0;

    if !body_fat_percent.is_finite() || !(0.0..=70.0).contains(&body_fat_percent) {
        return Err(CalcError::InvalidInput(
            "computed body fat percentage is outside a physiologically plausible range - check measurement inputs".into(),
        ));
    }

    let sex_label = match input.sex {
        Sex::Male => "male",
        Sex::Female => "female",
    };

    let interpretation = format!(
        "Estimated body fat {body_fat_percent:.1}% by the Jackson-Pollock 3-site skinfold method ({sex_label}), converting predicted body density to fat percentage via the Siri equation. The source regression equations report a standard error of roughly 3-4% body fat against hydrostatic weighing in largely Caucasian North American adults, and Jackson, Pollock & Ward advised particular care applying the women's equation above age 40. These equations are known to progressively underestimate body fat in obese individuals and are not universally validated across all body types or ethnicities; accuracy also depends on measurement technique and inter-rater consistency."
    );

    Ok(JacksonPollockSkinfoldOutcome {
        sum_skinfolds_mm: sum,
        body_density,
        body_fat_percent,
        interpretation,
    })
}

pub fn build_response(
    input: &JacksonPollockSkinfoldInput,
) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let rounded_bf = round1(o.body_fat_percent);

    let mut working = Map::new();
    working.insert("sex".into(), json!(input.sex));
    working.insert("age_years".into(), json!(input.age_years));
    working.insert("thigh_mm".into(), json!(input.thigh_mm));
    match input.sex {
        Sex::Male => {
            working.insert("chest_mm".into(), json!(input.chest_mm));
            working.insert("abdomen_mm".into(), json!(input.abdomen_mm));
        }
        Sex::Female => {
            working.insert("triceps_mm".into(), json!(input.triceps_mm));
            working.insert("suprailiac_mm".into(), json!(input.suprailiac_mm));
        }
    }
    working.insert("sum_skinfolds_mm".into(), json!(o.sum_skinfolds_mm));
    working.insert("body_density".into(), json!(o.body_density));
    working.insert(
        "body_fat_percent_unrounded".into(),
        json!(o.body_fat_percent),
    );
    working.insert("body_fat_percent".into(), json!(rounded_bf));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(rounded_bf),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct JacksonPollockSkinfold;

impl Calculator for JacksonPollockSkinfold {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Jackson-Pollock 3-Site Skinfold Body Fat %"
    }

    fn description(&self) -> &'static str {
        "Caliper-derived body fat percentage from three sex-specific skinfold sites (chest, abdomen, thigh for men; triceps, suprailiac, thigh for women) via the Jackson-Pollock generalized body density equations and the Siri conversion. Standard error ~3-4% against hydrostatic weighing; not universally validated across all body types."
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
            "title": "JacksonPollockSkinfoldInput",
            "description": "Jackson-Pollock 3-site skinfold body fat percentage. Men supply chest_mm and abdomen_mm; women supply triceps_mm and suprailiac_mm; both supply thigh_mm. Age is restricted to each equation's validation cohort (men 18-61, women 18-55); the women's equation additionally warrants care above age 40 per the source study.",
            "type": "object",
            "additionalProperties": false,
            "required": ["sex", "age_years", "thigh_mm"],
            "properties": {
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Sex, which selects the equation, required skinfold sites, and validated age range"
                },
                "age_years": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 61,
                    "description": "Age in years. Validated range is 18-61 for the male equation and 18-55 for the female equation; out-of-range values for the selected sex are rejected."
                },
                "thigh_mm": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Anterior thigh skinfold in mm: vertical fold on the anterior midline, midway between the proximal border of the patella and the inguinal crease. Required for both sexes."
                },
                "chest_mm": {
                    "type": ["number", "null"],
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Chest skinfold in mm: diagonal fold halfway between the anterior axillary line and the nipple. Required for male, ignored for female."
                },
                "abdomen_mm": {
                    "type": ["number", "null"],
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Abdominal skinfold in mm: vertical fold 2 cm to the right of the umbilicus. Required for male, ignored for female."
                },
                "triceps_mm": {
                    "type": ["number", "null"],
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Triceps skinfold in mm: vertical fold on the posterior midline of the upper arm, midway between the acromion and olecranon processes. Required for female, ignored for male."
                },
                "suprailiac_mm": {
                    "type": ["number", "null"],
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Suprailiac skinfold in mm: diagonal fold immediately above the iliac crest, following its natural angle. Required for female, ignored for male."
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: JacksonPollockSkinfoldInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn male(age: u32, chest: f64, abdomen: f64, thigh: f64) -> JacksonPollockSkinfoldInput {
        JacksonPollockSkinfoldInput {
            sex: Sex::Male,
            age_years: age,
            thigh_mm: thigh,
            chest_mm: Some(chest),
            abdomen_mm: Some(abdomen),
            triceps_mm: None,
            suprailiac_mm: None,
        }
    }

    fn female(age: u32, triceps: f64, suprailiac: f64, thigh: f64) -> JacksonPollockSkinfoldInput {
        JacksonPollockSkinfoldInput {
            sex: Sex::Female,
            age_years: age,
            thigh_mm: thigh,
            chest_mm: None,
            abdomen_mm: None,
            triceps_mm: Some(triceps),
            suprailiac_mm: Some(suprailiac),
        }
    }

    #[test]
    fn male_equation_conformance_vector() {
        // Sum = 15 + 20 + 15 = 50 mm, age 30.
        let sum: f64 = 50.0;
        let age: f64 = 30.0;
        let expected_density = 1.10938 - 0.0008267 * sum + 0.0000016 * sum * sum - 0.0002574 * age;
        let expected_bf = 495.0 / expected_density - 450.0;

        let o = compute(&male(30, 15.0, 20.0, 15.0)).unwrap();
        assert!((o.sum_skinfolds_mm - sum).abs() < 1e-9);
        assert!(
            (o.body_density - expected_density).abs() < 1e-12,
            "got {}",
            o.body_density
        );
        assert!(
            (o.body_fat_percent - expected_bf).abs() < 1e-9,
            "got {} vs {}",
            o.body_fat_percent,
            expected_bf
        );
    }

    #[test]
    fn female_equation_conformance_vector() {
        // Sum = 18 + 22 + 25 = 65 mm, age 35.
        let sum: f64 = 65.0;
        let age: f64 = 35.0;
        let expected_density =
            1.0994921 - 0.0009929 * sum + 0.0000023 * sum * sum - 0.0001392 * age;
        let expected_bf = 495.0 / expected_density - 450.0;

        let o = compute(&female(35, 18.0, 22.0, 25.0)).unwrap();
        assert!((o.sum_skinfolds_mm - sum).abs() < 1e-9);
        assert!(
            (o.body_density - expected_density).abs() < 1e-12,
            "got {}",
            o.body_density
        );
        assert!(
            (o.body_fat_percent - expected_bf).abs() < 1e-9,
            "got {} vs {}",
            o.body_fat_percent,
            expected_bf
        );
    }

    #[test]
    fn male_requires_chest_and_abdomen() {
        let mut input = male(30, 15.0, 20.0, 15.0);
        input.chest_mm = None;
        assert!(compute(&input).is_err());

        let mut input = male(30, 15.0, 20.0, 15.0);
        input.abdomen_mm = None;
        assert!(compute(&input).is_err());
    }

    #[test]
    fn female_requires_triceps_and_suprailiac() {
        let mut input = female(30, 15.0, 20.0, 15.0);
        input.triceps_mm = None;
        assert!(compute(&input).is_err());

        let mut input = female(30, 15.0, 20.0, 15.0);
        input.suprailiac_mm = None;
        assert!(compute(&input).is_err());
    }

    #[test]
    fn accepts_validation_age_boundaries() {
        assert!(compute(&male(18, 15.0, 20.0, 15.0)).is_ok());
        assert!(compute(&male(61, 15.0, 20.0, 15.0)).is_ok());
        assert!(compute(&female(18, 15.0, 20.0, 15.0)).is_ok());
        assert!(compute(&female(55, 15.0, 20.0, 15.0)).is_ok());
    }

    #[test]
    fn rejects_age_outside_validated_range_per_sex() {
        assert!(compute(&male(17, 15.0, 20.0, 15.0)).is_err());
        assert!(compute(&male(62, 15.0, 20.0, 15.0)).is_err());
        assert!(compute(&female(17, 15.0, 20.0, 15.0)).is_err());
        assert!(compute(&female(56, 15.0, 20.0, 15.0)).is_err());
    }

    #[test]
    fn rejects_out_of_range_skinfolds() {
        assert!(compute(&male(30, 1.9, 20.0, 15.0)).is_err());
        assert!(compute(&male(30, 100.1, 20.0, 15.0)).is_err());
        assert!(compute(&male(30, 15.0, 20.0, f64::NAN)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "sex": "male",
            "age_years": 30,
            "thigh_mm": 15.0,
            "chest_mm": 15.0,
            "abdomen_mm": 20.0,
            "triceps_mm": null,
            "suprailiac_mm": null
        });
        let dynamic = JacksonPollockSkinfold.calculate(&value).unwrap();
        let typed = build_response(&male(30, 15.0, 20.0, 15.0)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn dynamic_surface_rejects_unknown_fields() {
        let invalid = json!({
            "sex": "male",
            "age_years": 30,
            "thigh_mm": 15.0,
            "chest_mm": 15.0,
            "abdomen_mm": 20.0,
            "unexpected": true
        });
        assert!(JacksonPollockSkinfold.calculate(&invalid).is_err());
    }

    #[test]
    fn response_preserves_inputs_and_equation_working() {
        let response = build_response(&male(30, 15.0, 20.0, 15.0)).unwrap();
        assert_eq!(response.working["sex"], json!("male"));
        assert_eq!(response.working["age_years"], json!(30));
        assert_eq!(response.working["thigh_mm"], json!(15.0));
        assert_eq!(response.working["chest_mm"], json!(15.0));
        assert_eq!(response.working["abdomen_mm"], json!(20.0));
        assert!(response.working.get("triceps_mm").is_none());
        assert!(response.working.get("suprailiac_mm").is_none());
        assert_eq!(response.working["sum_skinfolds_mm"], json!(50.0));
        assert_eq!(response.result, response.working["body_fat_percent"]);
        assert!(response.interpretation.contains("Siri equation"));
        assert!(response.interpretation.contains("3-4% body fat"));
    }

    #[test]
    fn schema_documents_units_and_applicability_contract() {
        let schema = JacksonPollockSkinfold.input_schema();
        assert_eq!(schema["properties"]["thigh_mm"]["unit"], json!("mm"));
        assert_eq!(schema["properties"]["chest_mm"]["unit"], json!("mm"));
        let description = schema["description"].as_str().unwrap();
        assert!(description.contains("men 18-61, women 18-55"));
    }
}
