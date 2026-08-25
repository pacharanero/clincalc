// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Waist-to-hip ratio (WHR).
//!
//! A simple, unitless index of body-fat distribution: waist circumference
//! divided by hip circumference. Annex A, Table A1 of the 2011 WHO report
//! collates commonly WHO-attributed adult cut-offs: >= 0.90 in men and >= 0.85
//! in women. The report does not establish them as universal cut-offs and notes
//! that risk relationships vary between populations.
//!
//! Reference: World Health Organization. Waist Circumference and Waist-Hip
//! Ratio: Report of a WHO Expert Consultation, Geneva, 8-11 December 2008.
//! WHO Press; 2011.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "waist_to_hip_ratio";

pub const REFERENCE: &str = "World Health Organization. Waist Circumference and Waist-Hip Ratio: Report of a WHO Expert Consultation, Geneva, 8-11 December 2008. Geneva: WHO Press; 2011. Annex A, Table A1 collates the commonly WHO-attributed >=0.90 male and >=0.85 female cut-offs; the underlying WHO 1999 metabolic-syndrome working definition used >0.90 and >0.85.";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - standard anthropometric ratio; WHO-attributed adult cut-offs",
    source_url: "https://www.who.int/publications/i/item/9789241501491",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaistToHipRatioInput {
    /// The WHO-attributed risk cut-offs are intended for adults.
    pub adult: bool,
    pub sex: Sex,
    /// WHO-protocol average waist circumference in centimetres.
    pub waist_cm: f64,
    /// WHO-protocol average hip circumference in centimetres.
    pub hip_cm: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaistToHipRatioOutcome {
    pub ratio: f64,
    pub who_attributed_cutoff: f64,
    pub at_or_above_who_attributed_cutoff: bool,
    pub interpretation: String,
}

fn threshold(sex: Sex) -> f64 {
    match sex {
        Sex::Male => 0.90,
        Sex::Female => 0.85,
    }
}

pub fn compute(input: &WaistToHipRatioInput) -> Result<WaistToHipRatioOutcome, CalcError> {
    if !input.adult {
        return Err(CalcError::InvalidInput(
            "adult must be true; the WHO-attributed risk cut-offs are intended for adults".into(),
        ));
    }
    if !(30.0..=250.0).contains(&input.waist_cm) || !input.waist_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "waist_cm must be finite and between 30 and 250".into(),
        ));
    }
    if !(30.0..=250.0).contains(&input.hip_cm) || !input.hip_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "hip_cm must be finite and between 30 and 250".into(),
        ));
    }

    let ratio = input.waist_cm / input.hip_cm;
    let cutoff = threshold(input.sex);
    let at_or_above_who_attributed_cutoff = ratio >= cutoff;
    let sex_label = match input.sex {
        Sex::Male => "male",
        Sex::Female => "female",
    };

    let boundary_note = if at_or_above_who_attributed_cutoff {
        format!(
            "at or above the commonly WHO-attributed cut-off of {cutoff:.2}, which Table A1 associates with substantially increased risk of metabolic complications"
        )
    } else {
        format!("below the commonly WHO-attributed cut-off of {cutoff:.2}")
    };

    let interpretation = format!(
        "Waist-to-hip ratio {ratio} ({sex_label} adult; {boundary_note}). The 2011 WHO report does not establish a universal cut-off and notes that risk relationships and optimal cut-offs can vary between populations. WHR is one input into cardiometabolic risk assessment, not a diagnosis on its own."
    );

    Ok(WaistToHipRatioOutcome {
        ratio,
        who_attributed_cutoff: cutoff,
        at_or_above_who_attributed_cutoff,
        interpretation,
    })
}

pub fn build_response(input: &WaistToHipRatioInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("adult".into(), json!(input.adult));
    working.insert("sex".into(), json!(input.sex));
    working.insert("waist_cm".into(), json!(input.waist_cm));
    working.insert("hip_cm".into(), json!(input.hip_cm));
    working.insert(
        "who_attributed_cutoff".into(),
        json!(o.who_attributed_cutoff),
    );
    working.insert(
        "at_or_above_who_attributed_cutoff".into(),
        json!(o.at_or_above_who_attributed_cutoff),
    );
    working.insert("ratio".into(), json!(o.ratio));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.ratio),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct WaistToHipRatio;

impl Calculator for WaistToHipRatio {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Waist-to-Hip Ratio (WHR)"
    }

    fn description(&self) -> &'static str {
        "Adult waist circumference divided by hip circumference, compared with commonly WHO-attributed sex-specific cut-offs."
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
            "title": "WaistToHipRatioInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["adult", "sex", "waist_cm", "hip_cm"],
            "properties": {
                "adult": {
                    "type": "boolean",
                    "description": "Confirm that the person is an adult; the risk cut-offs used here are derived from adult evidence.",
                    "definition": {
                        "concept": "Adult eligibility for WHO-attributed waist-to-hip ratio cut-offs",
                        "statement": "The person is an adult.",
                        "excludes": ["Children and adolescents"],
                        "source": {
                            "citation": "World Health Organization. Waist Circumference and Waist-Hip Ratio: Report of a WHO Expert Consultation. 2011.",
                            "url": "https://www.who.int/publications/i/item/9789241501491"
                        },
                        "caveats": "The report's disease-risk evidence and commonly attributed cut-offs concern adults and do not establish paediatric cut-offs.",
                        "status": "draft"
                    }
                },
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Sex (determines which commonly WHO-attributed adult cut-off is applied)"
                },
                "waist_cm": {
                    "type": "number",
                    "minimum": 30,
                    "maximum": 250,
                    "description": "Average waist circumference in cm from two WHO-protocol measurements within 1 cm of each other (repeat both if they differ by more than 1 cm), taken at the midpoint between the lower margin of the last palpable rib and the top of the iliac crest at the end of a normal expiration"
                },
                "hip_cm": {
                    "type": "number",
                    "minimum": 30,
                    "maximum": 250,
                    "description": "Average hip circumference in cm from two WHO-protocol measurements within 1 cm of each other (repeat both if they differ by more than 1 cm), taken around the widest portion of the buttocks with the tape parallel to the floor"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: WaistToHipRatioInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc(sex: Sex, waist: f64, hip: f64) -> WaistToHipRatioInput {
        WaistToHipRatioInput {
            adult: true,
            sex,
            waist_cm: waist,
            hip_cm: hip,
        }
    }

    #[test]
    fn male_just_below_threshold() {
        // Source-derived boundary vector: WHO 2011, Annex A, Table A1 (p. 27)
        // collates >= 0.90 for men. This protects C018 and C019 for H001.
        let o = compute(&calc(Sex::Male, 89.9, 100.0)).unwrap();
        assert!((o.ratio - 0.899).abs() < 0.0001, "got {:.4}", o.ratio);
        assert!(!o.at_or_above_who_attributed_cutoff);
        assert!(
            o.interpretation
                .contains("below the commonly WHO-attributed")
        );
    }

    #[test]
    fn male_at_threshold() {
        // WHO 2011, Annex A, Table A1 (p. 27) uses >= 0.90, although the
        // underlying WHO 1999 metabolic-syndrome working definition used > 0.90.
        let o = compute(&calc(Sex::Male, 90.0, 100.0)).unwrap();
        assert!((o.ratio - 0.90).abs() < 0.001, "got {:.4}", o.ratio);
        assert!(o.at_or_above_who_attributed_cutoff);
        assert!(
            o.interpretation
                .contains("at or above the commonly WHO-attributed")
        );
    }

    #[test]
    fn female_at_threshold() {
        // Source-derived boundary vector: WHO 2011, Annex A, Table A1 (p. 27)
        // collates >= 0.85 for women.
        let o = compute(&calc(Sex::Female, 85.0, 100.0)).unwrap();
        assert!((o.ratio - 0.85).abs() < 0.001, "got {:.4}", o.ratio);
        assert!(o.at_or_above_who_attributed_cutoff);
        assert!(
            o.interpretation
                .contains("at or above the commonly WHO-attributed cut-off of 0.85")
        );
    }

    #[test]
    fn female_just_below_threshold() {
        let o = compute(&calc(Sex::Female, 84.9, 100.0)).unwrap();
        assert!((o.ratio - 0.849).abs() < 0.0001, "got {:.4}", o.ratio);
        assert!(!o.at_or_above_who_attributed_cutoff);
        assert!(
            o.interpretation
                .contains("below the commonly WHO-attributed cut-off of 0.85")
        );
    }

    #[test]
    fn rejects_out_of_range_waist() {
        assert!(compute(&calc(Sex::Male, 10.0, 100.0)).is_err());
        assert!(compute(&calc(Sex::Male, 300.0, 100.0)).is_err());
    }

    #[test]
    fn rejects_out_of_range_hip() {
        assert!(compute(&calc(Sex::Male, 90.0, 10.0)).is_err());
        assert!(compute(&calc(Sex::Male, 90.0, 300.0)).is_err());
    }

    #[test]
    fn rejects_non_adult_use() {
        let mut input = calc(Sex::Male, 90.0, 100.0);
        input.adult = false;
        assert_eq!(
            compute(&input),
            Err(CalcError::InvalidInput(
                "adult must be true; the WHO-attributed risk cut-offs are intended for adults"
                    .into()
            ))
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "adult": true,
            "sex": "female",
            "waist_cm": 78.0,
            "hip_cm": 102.0
        });
        let dynamic = WaistToHipRatio.calculate(&value).unwrap();
        let typed = build_response(&calc(Sex::Female, 78.0, 102.0)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn response_preserves_inputs_threshold_and_decision() {
        let response = build_response(&calc(Sex::Male, 89.9, 100.0)).unwrap();
        assert_eq!(response.result, json!(0.899));
        assert_eq!(response.working["adult"], json!(true));
        assert_eq!(response.working["sex"], json!("male"));
        assert_eq!(response.working["waist_cm"], json!(89.9));
        assert_eq!(response.working["hip_cm"], json!(100.0));
        assert_eq!(response.working["who_attributed_cutoff"], json!(0.9));
        assert_eq!(
            response.working["at_or_above_who_attributed_cutoff"],
            json!(false)
        );
    }

    #[test]
    fn response_does_not_hide_a_below_cutoff_ratio_with_rounding() {
        let response = build_response(&calc(Sex::Male, 89.99, 100.0)).unwrap();
        let reported_ratio = response.result.as_f64().unwrap();
        assert!(reported_ratio < 0.90);
        assert_eq!(
            response.working["at_or_above_who_attributed_cutoff"],
            json!(false)
        );
        assert!(
            response
                .interpretation
                .contains(&reported_ratio.to_string())
        );
    }

    #[test]
    fn schema_documents_the_who_repeat_measurement_protocol() {
        let schema = WaistToHipRatio.input_schema();
        for field in ["waist_cm", "hip_cm"] {
            let description = schema["properties"][field]["description"].as_str().unwrap();
            assert!(description.contains("Average"));
            assert!(description.contains("two WHO-protocol measurements"));
            assert!(description.contains("within 1 cm"));
            assert!(description.contains("repeat both"));
        }
    }
}
