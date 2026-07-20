// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Waist-to-height ratio (WHtR).
//!
//! A simple, unitless index of central adiposity: waist circumference divided by
//! height, using the same units for both. The 0.5 boundary is widely quoted as
//! "keep your waist to less than half your height" (NICE public-health guidance
//! and Ashwell et al.).
//!
//! Reference: Ashwell M, Gibson S. Waist-to-height ratio as an indicator of
//! 'early health risk': simpler and more predictive than using a 'matrix' based
//! on BMI and waist circumference. BMJ Open. 2016;6(3):e010159.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "waist_to_height_ratio";
pub const REFERENCE: &str = "Ashwell M, Gibson S. Waist-to-height ratio as an indicator of 'early health risk': simpler and more predictive than using a 'matrix' based on BMI and waist circumference. BMJ Open. 2016;6(3):e010159. doi:10.1136/bmjopen-2015-010159.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - standard anthropometric ratio",
    source_url: "https://doi.org/10.1136/bmjopen-2015-010159",
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaistToHeightRatioInput {
    /// Waist circumference in centimetres
    pub waist_cm: f64,
    /// Height in centimetres
    pub height_cm: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaistToHeightRatioOutcome {
    pub ratio: f64,
    pub interpretation: String,
}

pub fn compute(input: &WaistToHeightRatioInput) -> Result<WaistToHeightRatioOutcome, CalcError> {
    if !(30.0..=250.0).contains(&input.waist_cm) || !input.waist_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "waist_cm must be finite and between 30 and 250".into(),
        ));
    }
    if !(50.0..=250.0).contains(&input.height_cm) || !input.height_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "height_cm must be finite and between 50 and 250".into(),
        ));
    }
    if input.waist_cm > input.height_cm {
        return Err(CalcError::InvalidInput(
            "waist_cm cannot exceed height_cm - check measurements".into(),
        ));
    }

    let ratio = input.waist_cm / input.height_cm;

    let category = if ratio < 0.4 {
        "low central adiposity"
    } else if ratio < 0.5 {
        "acceptable / increased risk"
    } else {
        "elevated central adiposity"
    };

    let boundary_note = if ratio < 0.5 {
        "below the 0.5 'keep your waist to less than half your height' threshold"
    } else {
        "at or above the 0.5 'keep your waist to less than half your height' threshold"
    };

    let interpretation = format!(
        "Waist-to-height ratio {:.2} ({category}; {boundary_note}). WHtR reflects central adiposity and is more informative than BMI alone for metabolic risk.",
        ratio
    );

    Ok(WaistToHeightRatioOutcome {
        ratio,
        interpretation,
    })
}

pub fn build_response(input: &WaistToHeightRatioInput) -> Result<CalculationResponse, CalcError> {
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

pub struct WaistToHeightRatio;

impl Calculator for WaistToHeightRatio {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Waist-to-Height Ratio (WHtR)"
    }

    fn description(&self) -> &'static str {
        "Unitless central-adiposity index: waist circumference divided by height."
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
            "title": "WaistToHeightRatioInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["waist_cm", "height_cm"],
            "properties": {
                "waist_cm": {
                    "type": "number",
                    "minimum": 30,
                    "maximum": 250,
                    "description": "Waist circumference in centimetres (use the same units as height)"
                },
                "height_cm": {
                    "type": "number",
                    "minimum": 50,
                    "maximum": 250,
                    "description": "Height in centimetres (use the same units as waist)"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: WaistToHeightRatioInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc(waist: f64, height: f64) -> WaistToHeightRatioInput {
        WaistToHeightRatioInput {
            waist_cm: waist,
            height_cm: height,
        }
    }

    #[test]
    fn healthy_boundary() {
        let o = compute(&calc(80.0, 170.0)).unwrap();
        assert!((o.ratio - 0.4706).abs() < 0.001, "got {:.4}", o.ratio);
        assert!(o.interpretation.contains("below the 0.5"));
    }

    #[test]
    fn at_threshold() {
        let o = compute(&calc(85.0, 170.0)).unwrap();
        assert!((o.ratio - 0.5).abs() < 0.001, "got {:.4}", o.ratio);
        assert!(o.interpretation.contains("elevated central adiposity"));
    }

    #[test]
    fn rejects_waist_exceeds_height() {
        assert!(compute(&calc(160.0, 150.0)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "waist_cm": 88.0,
            "height_cm": 178.0
        });
        let dynamic = WaistToHeightRatio.calculate(&value).unwrap();
        let typed = build_response(&calc(88.0, 178.0)).unwrap();
        assert_eq!(dynamic, typed);
    }
}
