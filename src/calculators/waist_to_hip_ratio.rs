// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Waist-to-hip ratio (WHR).
//!
//! A simple, unitless index of central (abdominal-vs-gluteal) adiposity: waist
//! circumference divided by hip circumference. Read against the WHO Expert
//! Consultation's sex-specific cut-offs for substantially increased risk of
//! metabolic complications: >= 0.90 in men, >= 0.85 in women.
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

pub const REFERENCE: &str = "World Health Organization. Waist Circumference and Waist-Hip Ratio: Report of a WHO Expert Consultation, Geneva, 8-11 December 2008. Geneva: WHO Press; 2011.";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - standard anthropometric ratio; WHO expert-consultation thresholds",
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
    pub sex: Sex,
    /// Waist circumference in centimetres (midpoint between lowest rib and iliac crest)
    pub waist_cm: f64,
    /// Hip circumference in centimetres (widest point over the buttocks)
    pub hip_cm: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaistToHipRatioOutcome {
    pub ratio: f64,
    pub interpretation: String,
}

fn threshold(sex: Sex) -> f64 {
    match sex {
        Sex::Male => 0.90,
        Sex::Female => 0.85,
    }
}

pub fn compute(input: &WaistToHipRatioInput) -> Result<WaistToHipRatioOutcome, CalcError> {
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
    let sex_label = match input.sex {
        Sex::Male => "male",
        Sex::Female => "female",
    };

    let boundary_note = if ratio >= cutoff {
        format!(
            "at or above the WHO threshold of {cutoff:.2} for substantially increased risk of metabolic complications"
        )
    } else {
        format!("below the WHO threshold of {cutoff:.2} for substantially increased risk")
    };

    let interpretation = format!(
        "Waist-to-hip ratio {ratio:.2} ({sex_label}; {boundary_note}). WHR reflects the distribution of abdominal-vs-gluteal fat and is one input into cardiometabolic risk assessment, not a diagnosis on its own."
    );

    Ok(WaistToHipRatioOutcome {
        ratio,
        interpretation,
    })
}

pub fn build_response(input: &WaistToHipRatioInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("ratio".into(), json!(round2(o.ratio)));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round2(o.ratio)),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
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
        "Unitless central-adiposity index: waist circumference divided by hip circumference, read against sex-specific WHO risk thresholds."
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
            "required": ["sex", "waist_cm", "hip_cm"],
            "properties": {
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Sex (determines which WHO risk threshold is applied)"
                },
                "waist_cm": {
                    "type": "number",
                    "minimum": 30,
                    "maximum": 250,
                    "description": "Waist circumference in cm, measured midway between the lowest rib and the iliac crest"
                },
                "hip_cm": {
                    "type": "number",
                    "minimum": 30,
                    "maximum": 250,
                    "description": "Hip circumference in cm, measured at the widest point over the buttocks"
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
            sex,
            waist_cm: waist,
            hip_cm: hip,
        }
    }

    #[test]
    fn male_below_threshold() {
        let o = compute(&calc(Sex::Male, 85.0, 100.0)).unwrap();
        assert!((o.ratio - 0.85).abs() < 0.001, "got {:.4}", o.ratio);
        assert!(o.interpretation.contains("below the WHO threshold"));
    }

    #[test]
    fn male_at_threshold() {
        let o = compute(&calc(Sex::Male, 90.0, 100.0)).unwrap();
        assert!((o.ratio - 0.90).abs() < 0.001, "got {:.4}", o.ratio);
        assert!(o.interpretation.contains("at or above the WHO threshold"));
    }

    #[test]
    fn female_at_threshold() {
        let o = compute(&calc(Sex::Female, 85.0, 100.0)).unwrap();
        assert!((o.ratio - 0.85).abs() < 0.001, "got {:.4}", o.ratio);
        assert!(
            o.interpretation
                .contains("at or above the WHO threshold of 0.85")
        );
    }

    #[test]
    fn female_below_threshold() {
        let o = compute(&calc(Sex::Female, 70.0, 100.0)).unwrap();
        assert!((o.ratio - 0.70).abs() < 0.001, "got {:.4}", o.ratio);
        assert!(o.interpretation.contains("below the WHO threshold of 0.85"));
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
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "sex": "female",
            "waist_cm": 78.0,
            "hip_cm": 102.0
        });
        let dynamic = WaistToHipRatio.calculate(&value).unwrap();
        let typed = build_response(&calc(Sex::Female, 78.0, 102.0)).unwrap();
        assert_eq!(dynamic, typed);
    }
}
