// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Body adiposity index (BAI).
//!
//! A weight-free anthropometric estimate of whole-body fat percentage derived
//! from height and hip circumference alone. Bergman et al
//! (2011) fit the equation by regression against DXA-measured body fat in the
//! "BetaGene" cohort of Mexican-American adults (n = 1,733, ages 18-67;
//! R = 0.79 vs DXA) and externally validated it in the "TARA" cohort of
//! African-American adults (n = 223, ages 20-50; R = 0.849, bias-correction
//! factor Cb = 0.947, and Lin concordance correlation coefficient about
//! 0.804 vs DXA). A 2018 systematic review found wide individual error,
//! systematic overestimation at lower body fat and underestimation at higher
//! body fat, and did not recommend BAI for adult body-fat determination.
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

pub const REFERENCE: &str = "Equation: Bergman RN, Stefanovski D, Buchanan TA, et al. A better index of body adiposity. Obesity (Silver Spring). 2011;19(5):1083-1089. doi:10.1038/oby.2011.38. Validation limitations: Cerqueira M, Amorim P, Magalhaes F, et al. Validity of body adiposity index in predicting body fat in adults: a systematic review. Adv Nutr. 2018;9(5):617-624. doi:10.1093/advances/nmy043. PMID:30239583.";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published mathematical method - independently implemented; formulas and algorithms are not protected by US copyright",
    source_url: "https://www.copyright.gov/circs/circ31.pdf",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentContext {
    AdultLegacyAnthropometricEstimate,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyAdiposityIndexInput {
    /// Acknowledgement that BAI is a legacy estimate, not a measurement.
    pub assessment_context: AssessmentContext,
    /// Age in years; restricted to the external-validation cohort.
    pub age_years: u32,
    /// Standing height in centimetres.
    pub height_cm: f64,
    /// Hip circumference in centimetres, at the widest point of the hips/buttocks.
    pub hip_cm: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BodyAdiposityIndexOutcome {
    pub bai_estimate_percent: f64,
    pub interpretation: String,
}

pub fn compute(input: &BodyAdiposityIndexInput) -> Result<BodyAdiposityIndexOutcome, CalcError> {
    if !(20..=50).contains(&input.age_years) {
        return Err(CalcError::InvalidInput(
            "age_years must be between 20 and 50 - the age range observed in the TARA external-validation cohort".into(),
        ));
    }
    if !(148.0..=197.0).contains(&input.height_cm) || !input.height_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "height_cm must be finite and between 148 and 197 - the range observed in the TARA external-validation cohort".into(),
        ));
    }
    if !(82.0..=162.8).contains(&input.hip_cm) || !input.hip_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "hip_cm must be finite and between 82 and 162.8 - the range observed in the TARA external-validation cohort".into(),
        ));
    }

    let height_m = input.height_cm / 100.0;
    let hip_to_height_ratio = input.hip_cm / height_m.powf(1.5);
    let bai_estimate_percent = hip_to_height_ratio - 18.0;

    if !(0.0..=100.0).contains(&bai_estimate_percent) {
        return Err(CalcError::InvalidInput(
            "computed body adiposity index is outside the plausible percentage range 0-100% - check measurement inputs and model applicability".into(),
        ));
    }

    let interpretation = format!(
        "Legacy BAI estimate {bai_estimate_percent:.1} percentage points from hip circumference and height; this is not a direct body-composition measurement and does not diagnose obesity. The equation was derived in a Mexican-American family cohort enriched for gestational-diabetes risk and externally tested in 223 African-American adults. In that cohort, R=0.849 and the bias-correction factor Cb=0.947, giving a Lin concordance correlation coefficient of about 0.804; these cohort statistics do not establish individual agreement. A 2018 systematic review found wide individual error, systematic overestimation at lower body fat and underestimation at higher body fat, and did not recommend BAI for adult body-fat determination. No diagnostic BAI cut-points were established."
    );

    Ok(BodyAdiposityIndexOutcome {
        bai_estimate_percent,
        interpretation,
    })
}

pub fn build_response(input: &BodyAdiposityIndexInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let rounded_estimate = (o.bai_estimate_percent * 10.0).round() / 10.0;

    let mut working = Map::new();
    working.insert("assessment_context".into(), json!(input.assessment_context));
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
    working.insert(
        "bai_estimate_percent_unrounded".into(),
        json!(o.bai_estimate_percent),
    );
    working.insert("bai_estimate_percent".into(), json!(rounded_estimate));
    working.insert(
        "validation_summary".into(),
        json!("Later systematic review found wide individual error and did not recommend BAI for adult body-fat determination"),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(rounded_estimate),
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
        "Legacy weight-free body-fat estimate from height and hip circumference (Bergman 2011); later systematic review found wide individual error and did not recommend BAI for adult body-fat determination."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        let primary_source = json!({
            "citation": "Bergman RN et al. A better index of body adiposity. Obesity. 2011;19(5):1083-1089.",
            "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC3275633/"
        });
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "BodyAdiposityIndexInput",
            "description": "Legacy BAI anthropometric estimate for adults within the TARA external-validation cohort measurement ranges. It is not a direct body-composition measurement or diagnostic obesity classification; a 2018 systematic review found wide individual error and did not recommend BAI for adult body-fat determination.",
            "type": "object",
            "additionalProperties": false,
            "required": ["assessment_context", "age_years", "height_cm", "hip_cm"],
            "properties": {
                "assessment_context": {
                    "type": "string",
                    "const": "adult_legacy_anthropometric_estimate",
                    "description": "Acknowledgement that BAI is being used only as a legacy adult anthropometric estimate, not as a direct body-composition measurement, diagnosis, or treatment rule",
                    "definition": {
                        "concept": "BAI assessment context",
                        "statement": "Use BAI only as a legacy anthropometric estimate in an adult whose measurements are within the external-validation cohort envelope.",
                        "includes": ["Historical or comparative calculation with limitations retained alongside the result"],
                        "excludes": ["Direct body-fat measurement", "Diagnosis or classification of obesity", "Treatment or referral decisions based on BAI"],
                        "source": primary_source,
                        "snomedEcl": null,
                        "refset": null,
                        "caveats": "Cerqueira et al. 2018 found wide individual error and did not recommend BAI for adult body-fat determination.",
                        "status": "draft"
                    }
                },
                "age_years": {
                    "type": "integer",
                    "minimum": 20,
                    "maximum": 50,
                    "unit": "years",
                    "description": "Age in completed years, restricted to 20-50 - the range observed in the TARA external-validation cohort"
                },
                "height_cm": {
                    "type": "number",
                    "minimum": 148,
                    "maximum": 197,
                    "unit": "cm",
                    "description": "Standing height in centimetres, restricted to the 148-197 cm range observed in the TARA external-validation cohort"
                },
                "hip_cm": {
                    "type": "number",
                    "minimum": 82,
                    "maximum": 162.8,
                    "unit": "cm",
                    "description": "Mean of three hip-circumference measurements in centimetres, measured over nonrestrictive underwear or lightweight shorts in a horizontal plane at the maximum posterior extension of the buttocks; restricted to the 82-162.8 cm range observed in the TARA external-validation cohort",
                    "definition": {
                        "concept": "Hip circumference for BAI",
                        "statement": "Measure hip circumference using the TARA protocol used to externally evaluate BAI.",
                        "includes": ["Horizontal tape at maximum posterior extension of the buttocks", "Mean of three measurements", "Nonrestrictive underwear or lightweight shorts"],
                        "excludes": ["Waist circumference", "Single unconfirmed measurement", "Tape angled out of the horizontal plane"],
                        "source": primary_source,
                        "snomedEcl": null,
                        "refset": null,
                        "caveats": "Measurement error propagates directly into the BAI estimate.",
                        "status": "draft"
                    }
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
            assessment_context: AssessmentContext::AdultLegacyAnthropometricEstimate,
            age_years: age,
            height_cm: height,
            hip_cm: hip,
        }
    }

    #[test]
    fn equation_conformance_vector() {
        // Bergman 2011: BAI = hip_cm / (height_m ^ 1.5) - 18.
        let o = compute(&calc(35, 175.0, 100.0)).unwrap();
        assert!(
            (o.bai_estimate_percent - 25.195_939_772_483_108).abs() < 1e-9,
            "got {}",
            o.bai_estimate_percent
        );
    }

    #[test]
    fn second_equation_conformance_vector() {
        let o = compute(&calc(35, 160.0, 110.0)).unwrap();
        assert!(
            (o.bai_estimate_percent - 36.351_647_284_144_01).abs() < 1e-9,
            "got {}",
            o.bai_estimate_percent
        );
    }

    #[test]
    fn accepts_external_validation_envelope_boundaries() {
        assert!(compute(&calc(20, 175.0, 100.0)).is_ok());
        assert!(compute(&calc(50, 175.0, 100.0)).is_ok());
        assert!(compute(&calc(35, 148.0, 82.0)).is_ok());
        assert!(compute(&calc(35, 197.0, 162.8)).is_ok());
    }

    #[test]
    fn rejects_age_outside_external_validation_range() {
        assert!(compute(&calc(19, 175.0, 100.0)).is_err());
        assert!(compute(&calc(51, 175.0, 100.0)).is_err());
    }

    #[test]
    fn rejects_out_of_range_height() {
        assert!(compute(&calc(35, 147.9, 100.0)).is_err());
        assert!(compute(&calc(35, 197.1, 100.0)).is_err());
        assert!(compute(&calc(35, f64::NAN, 100.0)).is_err());
    }

    #[test]
    fn rejects_out_of_range_hip() {
        assert!(compute(&calc(35, 175.0, 81.9)).is_err());
        assert!(compute(&calc(35, 175.0, 162.9)).is_err());
        assert!(compute(&calc(35, 175.0, f64::INFINITY)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "assessment_context": "adult_legacy_anthropometric_estimate",
            "age_years": 35,
            "height_cm": 175.0,
            "hip_cm": 100.0
        });
        let dynamic = BodyAdiposityIndex.calculate(&value).unwrap();
        let typed = build_response(&calc(35, 175.0, 100.0)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn dynamic_surface_requires_context_and_rejects_unknown_fields() {
        for invalid in [
            json!({"age_years": 35, "height_cm": 175.0, "hip_cm": 100.0}),
            json!({
                "assessment_context": "direct_body_fat_measurement",
                "age_years": 35,
                "height_cm": 175.0,
                "hip_cm": 100.0
            }),
            json!({
                "assessment_context": "adult_legacy_anthropometric_estimate",
                "age_years": 35,
                "height_cm": 175.0,
                "hip_cm": 100.0,
                "unexpected": true
            }),
        ] {
            assert!(BodyAdiposityIndex.calculate(&invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn response_preserves_inputs_and_equation_working() {
        let response = build_response(&calc(35, 175.0, 100.0)).unwrap();
        assert_eq!(response.working["age_years"], json!(35));
        assert_eq!(response.working["height_cm"], json!(175.0));
        assert_eq!(response.working["hip_cm"], json!(100.0));
        assert_eq!(response.working["height_exponent"], json!(1.5));
        assert_eq!(response.working["equation_constant"], json!(-18.0));
        assert_eq!(response.result, json!(25.2));
        assert_eq!(
            response.working["bai_estimate_percent_unrounded"],
            json!(25.195_939_772_483_108)
        );
        assert_eq!(response.result, response.working["bai_estimate_percent"]);
        assert!(
            response
                .interpretation
                .contains("did not recommend BAI for adult body-fat determination")
        );
        assert!(response.interpretation.contains("about 0.804"));
    }

    #[test]
    fn schema_documents_source_cohorts_and_applicability_contract() {
        let schema = BodyAdiposityIndex.input_schema();
        let age = schema["properties"]["age_years"]["description"]
            .as_str()
            .unwrap();
        assert!(age.contains("TARA external-validation cohort"));
        assert_eq!(schema["properties"]["height_cm"]["minimum"], json!(148));
        assert_eq!(schema["properties"]["hip_cm"]["maximum"], json!(162.8));

        let hip = schema["properties"]["hip_cm"]["description"]
            .as_str()
            .unwrap();
        assert!(hip.contains("maximum posterior extension of the buttocks"));
        assert!(hip.contains("Mean of three"));
        assert!(schema["properties"]["hip_cm"]["definition"].is_object());
        assert_eq!(schema["properties"]["height_cm"]["unit"], json!("cm"));
    }
}
