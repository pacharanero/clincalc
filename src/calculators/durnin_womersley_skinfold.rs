// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Durnin-Womersley 4-site skinfold body fat percentage.
//!
//! Caliper-derived body density from four skinfold sites (biceps, triceps,
//! subscapular, supra-iliac), age-banded by sex, then converted to body fat
//! percentage by the Siri equation. Durnin & Womersley (1974) derived the
//! equations in 209 men and 272 women aged 16-72, recruited in Glasgow by
//! non-random sampling with a preponderance of moderately sedentary,
//! middle-class subjects (students, business and professional people and
//! their spouses, plus volunteers from an obesity clinic, health clubs, a
//! ballet company, and sports organisations).
//!
//! Body density = c - m x log10(S), where S is the sum of the four skinfolds
//! in millimetres and c, m are age-and-sex-specific constants (Table 5, "All
//! four skinfolds" row):
//!
//! Men:
//!   17-19  c=1.1620  m=0.0630
//!   20-29  c=1.1631  m=0.0632
//!   30-39  c=1.1422  m=0.0544
//!   40-49  c=1.1620  m=0.0700
//!   50-72  c=1.1715  m=0.0779
//!
//! Women:
//!   16-19  c=1.1549  m=0.0678
//!   20-29  c=1.1599  m=0.0717
//!   30-39  c=1.1423  m=0.0632
//!   40-49  c=1.1333  m=0.0612
//!   50-68  c=1.1339  m=0.0645
//!
//! Constants transcribed directly from the scanned primary source (Table 5,
//! pages 86-87), not from a secondary calculator site. Note the youngest
//! female band is 16-19, not 17-19 as the men's is - several widely-copied
//! secondary tables homogenise this to "17-19" for both sexes, which Table 1
//! of the primary paper (subject age-group ranges, 24-100 subjects per band)
//! contradicts.
//!
//! Body fat percentage then follows the Siri (1961) two-compartment equation,
//! the same conversion already used by `jackson_pollock_skinfold`:
//! %BF = (495 / body density) - 450.
//!
//! Each skinfold is a vertical or diagonal pinch of skin and subcutaneous fat
//! (not muscle), measured by a trained assessor using a calibrated caliper
//! and a consistent standardised protocol:
//! - Biceps: vertical fold over the mid-point of biceps brachii, at the same
//!   level as the triceps site, arm relaxed.
//! - Triceps: vertical fold on the posterior midline of the upper arm,
//!   midway between the acromion and olecranon processes, arm relaxed.
//! - Subscapular: diagonal fold just below the inferior angle of the
//!   scapula, at about 45 degrees to the vertical (the primary source's own
//!   protocol description).
//! - Supra-iliac: diagonal fold immediately above the iliac crest, in the
//!   mid-axillary line (the primary source's own protocol description).
//!
//! The source reported that, across its lookup tables, the predicted fat
//! content was within +/-3.5% of body-weight as fat for two-thirds of women
//! and +/-5% for two-thirds of men - a model-level error, not an individual
//! accuracy bound. A later four-compartment-model cross-validation found the
//! equation under-estimated body fat with a mean bias around -2%,
//! particularly in larger individuals.
//!
//! References: Durnin JVGA, Womersley J. Body fat assessed from total body
//! density and its estimation from skinfold thickness: measurements on 481
//! men and women aged from 16 to 72 years. Br J Nutr. 1974;32(1):77-97.
//! doi:10.1079/BJN19740060. Siri WE. Body composition from fluid spaces and
//! density: analysis of methods. 1961. Reprinted in Nutrition.
//! 1993;9(5):480-491; discussion 480, 492. Peterson MJ, Czerwinski SA,
//! Siervogel RM. Development and validation of skinfold-thickness prediction
//! equations with a 4-compartment model. Am J Clin Nutr. 2003;77(5):1186-1191.
//! doi:10.1093/ajcn/77.5.1186.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "durnin_womersley_skinfold";

pub const REFERENCE: &str = "Durnin JVGA, Womersley J. Body fat assessed from total body density and its estimation from skinfold thickness: measurements on 481 men and women aged from 16 to 72 years. Br J Nutr. 1974;32(1):77-97. doi:10.1079/BJN19740060. Siri WE. Body composition from fluid spaces and density: analysis of methods. 1961. Reprinted in Nutrition. 1993;9(5):480-491; discussion 480, 492. Peterson MJ, Czerwinski SA, Siervogel RM. Development and validation of skinfold-thickness prediction equations with a 4-compartment model. Am J Clin Nutr. 2003;77(5):1186-1191. doi:10.1093/ajcn/77.5.1186.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Published mathematical method - independently implemented; formulas and algorithms are not protected by US copyright",
    "https://www.copyright.gov/circs/circ31.pdf",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

/// The same four sites apply to both sexes; only the age-band lookup below
/// differs, so (unlike `jackson_pollock_skinfold`'s per-sex field sets) one
/// flat shape covers both.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DurninWomersleySkinfoldInput {
    pub sex: Sex,
    pub age_years: u32,
    pub biceps_mm: f64,
    pub triceps_mm: f64,
    pub subscapular_mm: f64,
    pub suprailiac_mm: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DurninWomersleySkinfoldOutcome {
    pub age_band: &'static str,
    pub sum_skinfolds_mm: f64,
    pub body_density: f64,
    pub body_fat_percent: f64,
    pub interpretation: String,
}

const SKINFOLD_RANGE_MM: std::ops::RangeInclusive<f64> = 2.0..=100.0;

fn validate_skinfold(value: f64, field: &str) -> Result<(), CalcError> {
    if !SKINFOLD_RANGE_MM.contains(&value) || !value.is_finite() {
        return Err(CalcError::InvalidInput(format!(
            "{field} must be finite and between 2 and 100 mm"
        )));
    }
    Ok(())
}

/// Table 5 ("All four skinfolds" row) age bands and `(c, m)` constants,
/// transcribed from the scanned primary source. Returns the matched band
/// label alongside the constants so the response can record which equation
/// was used.
fn coefficients(sex: Sex, age_years: u32) -> Result<(&'static str, f64, f64), CalcError> {
    let bands: &[(std::ops::RangeInclusive<u32>, &str, f64, f64)] = match sex {
        Sex::Male => &[
            (17..=19, "17-19", 1.1620, 0.0630),
            (20..=29, "20-29", 1.1631, 0.0632),
            (30..=39, "30-39", 1.1422, 0.0544),
            (40..=49, "40-49", 1.1620, 0.0700),
            (50..=72, "50-72", 1.1715, 0.0779),
        ],
        Sex::Female => &[
            (16..=19, "16-19", 1.1549, 0.0678),
            (20..=29, "20-29", 1.1599, 0.0717),
            (30..=39, "30-39", 1.1423, 0.0632),
            (40..=49, "40-49", 1.1333, 0.0612),
            (50..=68, "50-68", 1.1339, 0.0645),
        ],
    };

    bands
        .iter()
        .find(|(range, ..)| range.contains(&age_years))
        .map(|(_, label, c, m)| (*label, *c, *m))
        .ok_or_else(|| {
            let (lo, hi) = match sex {
                Sex::Male => (17, 72),
                Sex::Female => (16, 68),
            };
            CalcError::InvalidInput(format!(
                "age_years must be between {lo} and {hi} for this sex - the range evaluated in the validation cohort"
            ))
        })
}

pub fn compute(
    input: &DurninWomersleySkinfoldInput,
) -> Result<DurninWomersleySkinfoldOutcome, CalcError> {
    validate_skinfold(input.biceps_mm, "biceps_mm")?;
    validate_skinfold(input.triceps_mm, "triceps_mm")?;
    validate_skinfold(input.subscapular_mm, "subscapular_mm")?;
    validate_skinfold(input.suprailiac_mm, "suprailiac_mm")?;

    let (age_band, c, m) = coefficients(input.sex, input.age_years)?;

    let sum = input.biceps_mm + input.triceps_mm + input.subscapular_mm + input.suprailiac_mm;
    let body_density = c - m * sum.log10();

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
        "Estimated body fat {body_fat_percent:.1}% by the Durnin-Womersley 4-site skinfold method ({sex_label}, age band {age_band}), converting predicted body density to fat percentage via the Siri equation. In the original source, predicted fat content was within +/-3.5 percentage points of body-weight as fat for two-thirds of women and +/-5 for two-thirds of men - a model-level error, not an individual accuracy bound. The source cohort was a non-random, moderately sedentary, middle-class British sample recruited in Glasgow. A later four-compartment-model cross-validation found the equation tends to under-estimate body fat, especially in larger individuals. Accuracy also depends on trained measurement technique and inter-rater consistency. This is an estimate, not a direct body-composition measurement, diagnosis, or treatment rule."
    );

    Ok(DurninWomersleySkinfoldOutcome {
        age_band,
        sum_skinfolds_mm: sum,
        body_density,
        body_fat_percent,
        interpretation,
    })
}

pub fn build_response(
    input: &DurninWomersleySkinfoldInput,
) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let rounded_bf = round1(o.body_fat_percent);

    let mut working = Map::new();
    working.insert("sex".into(), json!(input.sex));
    working.insert("age_years".into(), json!(input.age_years));
    working.insert("age_band".into(), json!(o.age_band));
    working.insert("biceps_mm".into(), json!(input.biceps_mm));
    working.insert("triceps_mm".into(), json!(input.triceps_mm));
    working.insert("subscapular_mm".into(), json!(input.subscapular_mm));
    working.insert("suprailiac_mm".into(), json!(input.suprailiac_mm));
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

pub struct DurninWomersleySkinfold;

impl Calculator for DurninWomersleySkinfold {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Durnin-Womersley 4-Site Skinfold Body Fat %"
    }

    fn description(&self) -> &'static str {
        "Caliper-derived body fat estimate from four skinfold sites (biceps, triceps, subscapular, supra-iliac) via age-and-sex-banded Durnin-Womersley body-density equations and Siri conversion. Restricted to the source cohort's age ranges (men 17-72, women 16-68); model error is not an individual accuracy bound, and later validation found a tendency to under-estimate body fat."
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
            "title": "DurninWomersleySkinfoldInput",
            "description": "Durnin-Womersley 4-site skinfold body fat estimate. All four sites apply to both sexes; obtain them with a calibrated caliper and consistent standardised technique. Restricted to the source cohort's validated age ranges: 17-72 for men, 16-68 for women - the youngest band differs by sex.",
            "type": "object",
            "additionalProperties": false,
            "required": ["sex", "age_years", "biceps_mm", "triceps_mm", "subscapular_mm", "suprailiac_mm"],
            "properties": {
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Sex, which selects the age-banded equation and the validated age range (17-72 for men, 16-68 for women)"
                },
                "age_years": {
                    "type": "integer",
                    "minimum": 16,
                    "maximum": 72,
                    "description": "Age in completed years. Source range: 17-72 for men, 16-68 for women; out-of-range values for the selected sex are rejected."
                },
                "biceps_mm": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Biceps skinfold in mm: vertical fold over the mid-point of biceps brachii, at the same level as the triceps site, arm relaxed. Measure with a calibrated caliper and consistent standardised technique. The 2-100 mm bound is a broad input-safety guard, not a source threshold."
                },
                "triceps_mm": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Triceps skinfold in mm: vertical fold on the posterior midline of the upper arm, midway between the acromion and olecranon processes, arm relaxed."
                },
                "subscapular_mm": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Subscapular skinfold in mm: diagonal fold just below the inferior angle of the scapula, at about 45 degrees to the vertical (the source's own protocol description)."
                },
                "suprailiac_mm": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 100,
                    "unit": "mm",
                    "description": "Supra-iliac skinfold in mm: diagonal fold immediately above the iliac crest, in the mid-axillary line (the source's own protocol description)."
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: DurninWomersleySkinfoldInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn male(
        age: u32,
        biceps: f64,
        triceps: f64,
        subscapular: f64,
        suprailiac: f64,
    ) -> DurninWomersleySkinfoldInput {
        DurninWomersleySkinfoldInput {
            sex: Sex::Male,
            age_years: age,
            biceps_mm: biceps,
            triceps_mm: triceps,
            subscapular_mm: subscapular,
            suprailiac_mm: suprailiac,
        }
    }

    fn female(
        age: u32,
        biceps: f64,
        triceps: f64,
        subscapular: f64,
        suprailiac: f64,
    ) -> DurninWomersleySkinfoldInput {
        DurninWomersleySkinfoldInput {
            sex: Sex::Female,
            age_years: age,
            biceps_mm: biceps,
            triceps_mm: triceps,
            subscapular_mm: subscapular,
            suprailiac_mm: suprailiac,
        }
    }

    #[test]
    fn male_20_29_conformance_vector() {
        // Independently calculated from Durnin & Womersley (1974) Table 5
        // (20-29 men: c=1.1631, m=0.0632) and Siri (1961), not recomputed in
        // the test.
        let o = compute(&male(25, 5.0, 10.0, 15.0, 10.0)).unwrap();
        assert_eq!(o.age_band, "20-29");
        assert!((o.sum_skinfolds_mm - 40.0).abs() < 1e-9);
        assert!((o.body_density - 1.0618498085480728).abs() < 1e-12);
        assert!((o.body_fat_percent - 16.167621838008756).abs() < 1e-9);
    }

    #[test]
    fn female_30_39_conformance_vector() {
        // Independently calculated from Durnin & Womersley (1974) Table 5
        // (30-39 women: c=1.1423, m=0.0632) and Siri (1961).
        let o = compute(&female(35, 8.0, 18.0, 20.0, 15.0)).unwrap();
        assert_eq!(o.age_band, "30-39");
        assert!((o.sum_skinfolds_mm - 61.0).abs() < 1e-9);
        assert!((o.body_density - 1.0294671544273195).abs() < 1e-12);
        assert!((o.body_fat_percent - 30.831270693004967).abs() < 1e-9);
    }

    #[test]
    fn accepts_validation_age_boundaries_per_sex() {
        assert!(compute(&male(17, 10.0, 10.0, 10.0, 10.0)).is_ok());
        assert!(compute(&male(72, 10.0, 10.0, 10.0, 10.0)).is_ok());
        assert!(compute(&female(16, 10.0, 10.0, 10.0, 10.0)).is_ok());
        assert!(compute(&female(68, 10.0, 10.0, 10.0, 10.0)).is_ok());
    }

    #[test]
    fn rejects_age_outside_validated_range_per_sex() {
        assert!(compute(&male(16, 10.0, 10.0, 10.0, 10.0)).is_err());
        assert!(compute(&male(73, 10.0, 10.0, 10.0, 10.0)).is_err());
        assert!(compute(&female(15, 10.0, 10.0, 10.0, 10.0)).is_err());
        assert!(compute(&female(69, 10.0, 10.0, 10.0, 10.0)).is_err());
    }

    #[test]
    fn women_youngest_band_is_sixteen_not_seventeen() {
        // The primary source's Table 1 gives women's youngest cohort as
        // 16-19, unlike men's 17-19 - several secondary calculator sites
        // homogenise this to "17-19" for both sexes, which is not what the
        // primary source's own subject age-grouping shows.
        assert!(compute(&female(16, 10.0, 10.0, 10.0, 10.0)).is_ok());
        assert!(compute(&male(16, 10.0, 10.0, 10.0, 10.0)).is_err());
    }

    #[test]
    fn age_band_boundaries_select_the_correct_equation() {
        assert_eq!(
            compute(&male(19, 10.0, 10.0, 10.0, 10.0)).unwrap().age_band,
            "17-19"
        );
        assert_eq!(
            compute(&male(20, 10.0, 10.0, 10.0, 10.0)).unwrap().age_band,
            "20-29"
        );
        assert_eq!(
            compute(&male(49, 10.0, 10.0, 10.0, 10.0)).unwrap().age_band,
            "40-49"
        );
        assert_eq!(
            compute(&male(50, 10.0, 10.0, 10.0, 10.0)).unwrap().age_band,
            "50-72"
        );
    }

    #[test]
    fn rejects_out_of_range_skinfolds() {
        assert!(compute(&male(25, 1.9, 10.0, 10.0, 10.0)).is_err());
        assert!(compute(&male(25, 100.1, 10.0, 10.0, 10.0)).is_err());
        assert!(compute(&male(25, 10.0, 10.0, 10.0, f64::NAN)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "sex": "male",
            "age_years": 25,
            "biceps_mm": 5.0,
            "triceps_mm": 10.0,
            "subscapular_mm": 15.0,
            "suprailiac_mm": 10.0
        });
        let dynamic = DurninWomersleySkinfold.calculate(&value).unwrap();
        let typed = build_response(&male(25, 5.0, 10.0, 15.0, 10.0)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn dynamic_surface_rejects_unknown_fields() {
        let invalid = json!({
            "sex": "male",
            "age_years": 25,
            "biceps_mm": 5.0,
            "triceps_mm": 10.0,
            "subscapular_mm": 15.0,
            "suprailiac_mm": 10.0,
            "unexpected": true
        });
        assert!(DurninWomersleySkinfold.calculate(&invalid).is_err());
    }

    #[test]
    fn response_preserves_inputs_and_equation_working() {
        let response = build_response(&male(25, 5.0, 10.0, 15.0, 10.0)).unwrap();
        assert_eq!(response.working["sex"], json!("male"));
        assert_eq!(response.working["age_years"], json!(25));
        assert_eq!(response.working["age_band"], json!("20-29"));
        assert_eq!(response.working["biceps_mm"], json!(5.0));
        assert_eq!(response.working["triceps_mm"], json!(10.0));
        assert_eq!(response.working["subscapular_mm"], json!(15.0));
        assert_eq!(response.working["suprailiac_mm"], json!(10.0));
        assert_eq!(response.working["sum_skinfolds_mm"], json!(40.0));
        assert_eq!(response.result, response.working["body_fat_percent"]);
        assert!(response.interpretation.contains("Siri equation"));
        assert!(
            response
                .interpretation
                .contains("not an individual accuracy bound")
        );
    }

    #[test]
    fn schema_documents_units_and_applicability_contract() {
        let schema = DurninWomersleySkinfold.input_schema();
        assert_eq!(schema["properties"]["biceps_mm"]["unit"], json!("mm"));
        assert_eq!(schema["properties"]["subscapular_mm"]["unit"], json!("mm"));
        let description = schema["description"].as_str().unwrap();
        assert!(description.contains("17-72 for men, 16-68 for women"));
        let template = DurninWomersleySkinfold.input_template();
        assert!(template.get("biceps_mm").is_some());
        assert!(template.get("subscapular_mm").is_some());
    }
}
