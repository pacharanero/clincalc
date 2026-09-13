// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Jackson-Pollock 3-site skinfold body fat percentage.
//!
//! Caliper-derived body density from three sex-specific skinfold sites, then
//! converted to body fat percentage by the Siri equation. Jackson & Pollock
//! (1978) derived the men's equation in 308 men aged 18-61 and cross-validated
//! it in 95 further men against hydrostatic (underwater) weighing. Jackson,
//! Pollock & Ward (1980) derived the women's equation in 249 women aged 18-55
//! and cross-validated it in 82 further women by the same method; they cautioned
//! that care is needed above age 40. Later reanalysis reported that nearly all
//! participants were non-Hispanic white and advised against using the quadratic
//! equations when the three-site sum exceeds 120 mm.
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
//! (not muscle), measured by a trained assessor using a calibrated caliper and
//! a consistent standardised protocol:
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
//! 1993;9(5):480-491; discussion 480, 492. Nevill AM, Metsios GS, Jackson AS,
//! et al. Can we use the Jackson and Pollock equations to predict body
//! density/fat of obese individuals in the 21st century? Int J Body Compos Res.
//! 2008;6(3):114-121. PMID:20582331.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "jackson_pollock_skinfold";

pub const REFERENCE: &str = "Jackson AS, Pollock ML. Generalized equations for predicting body density of men. Br J Nutr. 1978;40(3):497-504. doi:10.1079/bjn19780152. Jackson AS, Pollock ML, Ward A. Generalized equations for predicting body density of women. Med Sci Sports Exerc. 1980;12(3):175-181. PMID:7402053. Siri WE. Body composition from fluid spaces and density: analysis of methods. 1961. Reprinted in Nutrition. 1993;9(5):480-491; discussion 480, 492. Nevill AM, Metsios GS, Jackson AS, et al. Can we use the Jackson and Pollock equations to predict body density/fat of obese individuals in the 21st century? Int J Body Compos Res. 2008;6(3):114-121. PMID:20582331.";

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
#[serde(tag = "sex", rename_all = "lowercase", deny_unknown_fields)]
pub enum JacksonPollockSkinfoldInput {
    Male {
        age_years: u32,
        thigh_mm: f64,
        chest_mm: f64,
        abdomen_mm: f64,
    },
    Female {
        age_years: u32,
        thigh_mm: f64,
        triceps_mm: f64,
        suprailiac_mm: f64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct JacksonPollockSkinfoldOutcome {
    pub sum_skinfolds_mm: f64,
    pub body_density: f64,
    pub body_fat_percent: f64,
    pub interpretation: String,
}

const SKINFOLD_RANGE_MM: std::ops::RangeInclusive<f64> = 2.0..=100.0;
const MAX_SUM_SKINFOLDS_MM: f64 = 120.0;

fn validate_skinfold(value: f64, field: &str) -> Result<(), CalcError> {
    if !SKINFOLD_RANGE_MM.contains(&value) || !value.is_finite() {
        return Err(CalcError::InvalidInput(format!(
            "{field} must be finite and between 2 and 100 mm"
        )));
    }
    Ok(())
}

pub fn compute(
    input: &JacksonPollockSkinfoldInput,
) -> Result<JacksonPollockSkinfoldOutcome, CalcError> {
    let (
        sex,
        age_years,
        sum,
        age_range,
        age_coefficient,
        constant,
        linear_coefficient,
        quadratic_coefficient,
    ) = match *input {
        JacksonPollockSkinfoldInput::Male {
            age_years,
            thigh_mm,
            chest_mm,
            abdomen_mm,
        } => {
            validate_skinfold(thigh_mm, "thigh_mm")?;
            validate_skinfold(chest_mm, "chest_mm")?;
            validate_skinfold(abdomen_mm, "abdomen_mm")?;
            (
                Sex::Male,
                age_years,
                chest_mm + abdomen_mm + thigh_mm,
                18..=61,
                0.0002574,
                1.10938,
                0.0008267,
                0.0000016,
            )
        }
        JacksonPollockSkinfoldInput::Female {
            age_years,
            thigh_mm,
            triceps_mm,
            suprailiac_mm,
        } => {
            validate_skinfold(thigh_mm, "thigh_mm")?;
            validate_skinfold(triceps_mm, "triceps_mm")?;
            validate_skinfold(suprailiac_mm, "suprailiac_mm")?;
            (
                Sex::Female,
                age_years,
                triceps_mm + suprailiac_mm + thigh_mm,
                18..=55,
                0.0001392,
                1.0994921,
                0.0009929,
                0.0000023,
            )
        }
    };

    if !age_range.contains(&age_years) {
        return Err(CalcError::InvalidInput(format!(
            "age_years must be between {} and {} - the range evaluated in the validation cohort for this sex",
            age_range.start(),
            age_range.end()
        )));
    }

    if sum > MAX_SUM_SKINFOLDS_MM {
        return Err(CalcError::InvalidInput(
            "sum of the three skinfolds must not exceed 120 mm - later validation found that the Jackson-Pollock quadratic equations underestimate body fat above this boundary".into(),
        ));
    }

    let age = age_years as f64;
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

    let sex_label = match sex {
        Sex::Male => "male",
        Sex::Female => "female",
    };

    let interpretation = format!(
        "Estimated body fat {body_fat_percent:.1}% by the Jackson-Pollock 3-site skinfold method ({sex_label}), converting predicted body density to fat percentage via the Siri equation. In cross-validation, the men's source reported a body-density standard error of 0.0077 g/mL; the women's source reported 3.7-4.0 percentage points across its equations. These model-level errors are not individual accuracy bounds. The source samples were nearly all non-Hispanic white, the women's source advised care above age 40, and later evidence found underestimation above a 120 mm three-site sum. Accuracy also depends on trained measurement technique and inter-rater consistency. This is an estimate, not a direct body-composition measurement, diagnosis, or treatment rule."
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
    match *input {
        JacksonPollockSkinfoldInput::Male {
            age_years,
            thigh_mm,
            chest_mm,
            abdomen_mm,
        } => {
            working.insert("sex".into(), json!(Sex::Male));
            working.insert("age_years".into(), json!(age_years));
            working.insert("thigh_mm".into(), json!(thigh_mm));
            working.insert("chest_mm".into(), json!(chest_mm));
            working.insert("abdomen_mm".into(), json!(abdomen_mm));
        }
        JacksonPollockSkinfoldInput::Female {
            age_years,
            thigh_mm,
            triceps_mm,
            suprailiac_mm,
        } => {
            working.insert("sex".into(), json!(Sex::Female));
            working.insert("age_years".into(), json!(age_years));
            working.insert("thigh_mm".into(), json!(thigh_mm));
            working.insert("triceps_mm".into(), json!(triceps_mm));
            working.insert("suprailiac_mm".into(), json!(suprailiac_mm));
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
        "Caliper-derived body fat estimate from three sex-specific skinfold sites via the Jackson-Pollock generalized body-density equations and Siri conversion. Restricted to the source age ranges and a three-site sum of at most 120 mm; model error is not an individual accuracy bound."
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
            "description": "Jackson-Pollock 3-site skinfold body fat estimate. Supply exactly the site measurements for the selected sex, obtained by a trained assessor with a calibrated caliper and consistent standardised technique. Restricted to the source age ranges (men 18-61, women 18-55) and a three-site sum of at most 120 mm; the women's source additionally advises care above age 40.",
            "type": "object",
            "additionalProperties": false,
            "oneOf": [
                {
                    "title": "Male equation",
                    "properties": { "sex": { "const": "male" } },
                    "required": ["sex", "age_years", "thigh_mm", "chest_mm", "abdomen_mm"],
                    "not": { "anyOf": [
                        { "required": ["triceps_mm"] },
                        { "required": ["suprailiac_mm"] }
                    ] }
                },
                {
                    "title": "Female equation",
                    "properties": { "sex": { "const": "female" } },
                    "required": ["sex", "age_years", "thigh_mm", "triceps_mm", "suprailiac_mm"],
                    "not": { "anyOf": [
                        { "required": ["chest_mm"] },
                        { "required": ["abdomen_mm"] }
                    ] }
                }
            ],
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
                    "description": "Age in completed years. Source range: 18-61 for the male equation and 18-55 for the female equation; out-of-range values for the selected sex are rejected."
                },
                "thigh_mm": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Anterior thigh skinfold in mm: vertical fold on the anterior midline, midway between the proximal border of the patella and the inguinal crease. Measure with a calibrated caliper and consistent standardised technique. The 2-100 mm per-site bounds are broad input-safety guards; the three-site sum must not exceed the later evidence-based 120 mm boundary."
                },
                "chest_mm": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Male equation only. Chest skinfold in mm: diagonal fold halfway between the anterior axillary line and the nipple."
                },
                "abdomen_mm": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Male equation only. Abdominal skinfold in mm: vertical fold 2 cm to the right of the umbilicus."
                },
                "triceps_mm": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Female equation only. Triceps skinfold in mm: vertical fold on the posterior midline of the upper arm, midway between the acromion and olecranon processes."
                },
                "suprailiac_mm": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Female equation only. Suprailiac skinfold in mm: diagonal fold immediately above the iliac crest at the anterior axillary line, following its natural angle."
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        if let Some(field) = input.as_object().and_then(|object| {
            object.keys().find(|field| {
                !matches!(
                    field.as_str(),
                    "sex"
                        | "age_years"
                        | "thigh_mm"
                        | "chest_mm"
                        | "abdomen_mm"
                        | "triceps_mm"
                        | "suprailiac_mm"
                )
            })
        }) {
            return Err(CalcError::InvalidInput(format!("unknown field `{field}`")));
        }

        let parsed: JacksonPollockSkinfoldInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn male(age: u32, chest: f64, abdomen: f64, thigh: f64) -> JacksonPollockSkinfoldInput {
        JacksonPollockSkinfoldInput::Male {
            age_years: age,
            thigh_mm: thigh,
            chest_mm: chest,
            abdomen_mm: abdomen,
        }
    }

    fn female(age: u32, triceps: f64, suprailiac: f64, thigh: f64) -> JacksonPollockSkinfoldInput {
        JacksonPollockSkinfoldInput::Female {
            age_years: age,
            thigh_mm: thigh,
            triceps_mm: triceps,
            suprailiac_mm: suprailiac,
        }
    }

    #[test]
    fn male_equation_conformance_vector() {
        // Fixed regression vector independently calculated from Jackson &
        // Pollock (1978) equation and Siri (1961), not recomputed in the test.
        let o = compute(&male(30, 15.0, 20.0, 15.0)).unwrap();
        assert!((o.sum_skinfolds_mm - 50.0).abs() < 1e-9);
        assert!((o.body_density - 1.064323).abs() < 1e-12);
        assert!((o.body_fat_percent - 15.084377580865919).abs() < 1e-9);
    }

    #[test]
    fn female_equation_conformance_vector() {
        // Fixed regression vector independently calculated from Jackson,
        // Pollock & Ward (1980) equation and Siri (1961).
        let o = compute(&female(35, 18.0, 22.0, 25.0)).unwrap();
        assert!((o.sum_skinfolds_mm - 65.0).abs() < 1e-9);
        assert!((o.body_density - 1.0397991).abs() < 1e-12);
        assert!((o.body_fat_percent - 26.05349918075524).abs() < 1e-9);
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
    fn rejects_sum_above_later_validation_boundary() {
        assert!(compute(&male(30, 40.0, 40.0, 40.0)).is_ok());
        assert!(compute(&male(30, 40.0, 40.0, 40.1)).is_err());
        assert!(compute(&female(30, 40.0, 40.0, 40.1)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "sex": "male",
            "age_years": 30,
            "thigh_mm": 15.0,
            "chest_mm": 15.0,
            "abdomen_mm": 20.0
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
    fn dynamic_surface_rejects_wrong_equation_fields() {
        let invalid = json!({
            "sex": "male",
            "age_years": 30,
            "thigh_mm": 15.0,
            "chest_mm": 15.0,
            "abdomen_mm": 20.0,
            "triceps_mm": 18.0
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
        assert!(
            response
                .interpretation
                .contains("not individual accuracy bounds")
        );
    }

    #[test]
    fn schema_documents_units_and_applicability_contract() {
        let schema = JacksonPollockSkinfold.input_schema();
        assert_eq!(schema["properties"]["thigh_mm"]["unit"], json!("mm"));
        assert_eq!(schema["properties"]["chest_mm"]["unit"], json!("mm"));
        let description = schema["description"].as_str().unwrap();
        assert!(description.contains("men 18-61, women 18-55"));
        assert_eq!(
            schema["oneOf"][0]["properties"]["sex"]["const"],
            json!("male")
        );
        let template = JacksonPollockSkinfold.input_template();
        assert!(template.get("chest_mm").is_some());
        assert!(template.get("abdomen_mm").is_some());
        assert!(template.get("triceps_mm").is_none());
    }
}
