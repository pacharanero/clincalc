// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Homeostasis Model Assessment of Insulin Resistance (HOMA-IR).
//!
//! This implements the original HOMA1 surrogate estimate from fasting plasma
//! glucose and fasting insulin. HOMA-IR is reported as a continuous value; the
//! original method does not provide a universal diagnostic cut-off.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "homa_ir";
pub const REFERENCE: &str = "Matthews DR, Hosker JP, Rudenski AS, Naylor BA, Treacher DF, Turner RC. Homeostasis model assessment: insulin resistance and beta-cell function from fasting plasma glucose and insulin concentrations in man. Diabetologia. 1985;28:412-419. PMID: 3899825. doi:10.1007/BF00280883";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1007/BF00280883",
};

const MGDL_PER_MMOLL_GLUCOSE: f64 = 18.0;
const MIN_GLUCOSE_MMOLL: f64 = 1.7;
const MAX_GLUCOSE_MMOLL: f64 = 27.8;
const MIN_GLUCOSE_MGDL: f64 = 30.0;
const MAX_GLUCOSE_MGDL: f64 = 500.0;
const MIN_INSULIN_MIU_L: f64 = 0.5;
const MAX_INSULIN_MIU_L: f64 = 300.0;

const LIMITATIONS: &str = "HOMA-IR is the original HOMA1 surrogate estimate and has limited precision. It is not a universal diagnostic cut-off: values depend on insulin assay and population. Do not use it in people receiving exogenous insulin unless a validated protocol specifically supports that use.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlucoseUnit {
    #[serde(rename = "mmol/L")]
    MmolL,
    #[serde(rename = "mg/dL")]
    MgDl,
}

impl GlucoseUnit {
    fn label(self) -> &'static str {
        match self {
            Self::MmolL => "mmol/L",
            Self::MgDl => "mg/dL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HomaIrInput {
    pub fasting_glucose: f64,
    pub glucose_unit: GlucoseUnit,
    pub fasting_insulin_miu_l: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HomaIrOutcome {
    /// Unrounded HOMA-IR value.
    pub homa_ir: f64,
    pub fasting_glucose_mmol_l: f64,
    pub interpretation: String,
}

pub fn compute(input: &HomaIrInput) -> Result<HomaIrOutcome, CalcError> {
    if !input.fasting_glucose.is_finite() {
        return Err(CalcError::InvalidInput(
            "fasting_glucose must be a finite number".into(),
        ));
    }
    if !input.fasting_insulin_miu_l.is_finite() {
        return Err(CalcError::InvalidInput(
            "fasting_insulin_miu_l must be a finite number".into(),
        ));
    }

    let fasting_glucose_mmol_l = match input.glucose_unit {
        GlucoseUnit::MmolL => {
            if !(MIN_GLUCOSE_MMOLL..=MAX_GLUCOSE_MMOLL).contains(&input.fasting_glucose) {
                return Err(CalcError::InvalidInput(format!(
                    "fasting_glucose must be between {MIN_GLUCOSE_MMOLL} and {MAX_GLUCOSE_MMOLL} mmol/L"
                )));
            }
            input.fasting_glucose
        }
        GlucoseUnit::MgDl => {
            if !(MIN_GLUCOSE_MGDL..=MAX_GLUCOSE_MGDL).contains(&input.fasting_glucose) {
                return Err(CalcError::InvalidInput(format!(
                    "fasting_glucose must be between {MIN_GLUCOSE_MGDL} and {MAX_GLUCOSE_MGDL} mg/dL"
                )));
            }
            input.fasting_glucose / MGDL_PER_MMOLL_GLUCOSE
        }
    };

    if !(MIN_INSULIN_MIU_L..=MAX_INSULIN_MIU_L).contains(&input.fasting_insulin_miu_l) {
        return Err(CalcError::InvalidInput(format!(
            "fasting_insulin_miu_l must be between {MIN_INSULIN_MIU_L} and {MAX_INSULIN_MIU_L} mIU/L"
        )));
    }

    let homa_ir = input.fasting_insulin_miu_l * fasting_glucose_mmol_l / 22.5;
    let rounded = (homa_ir * 100.0).round() / 100.0;
    let interpretation = format!(
        "HOMA-IR is {rounded:.2}, reported as the continuous original HOMA1 surrogate estimate. {LIMITATIONS}"
    );

    Ok(HomaIrOutcome {
        homa_ir,
        fasting_glucose_mmol_l,
        interpretation,
    })
}

pub fn build_response(input: &HomaIrInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let rounded = (outcome.homa_ir * 100.0).round() / 100.0;
    let mut working = Map::new();
    working.insert("fasting_glucose_input".into(), json!(input.fasting_glucose));
    working.insert(
        "glucose_input_unit".into(),
        json!(input.glucose_unit.label()),
    );
    working.insert(
        "fasting_glucose_mmol_l".into(),
        json!(outcome.fasting_glucose_mmol_l),
    );
    working.insert(
        "fasting_insulin_miu_l".into(),
        json!(input.fasting_insulin_miu_l),
    );
    working.insert("homa_ir_unrounded".into(), json!(outcome.homa_ir));
    working.insert(
        "formula".into(),
        json!("fasting_insulin_miu_l * fasting_glucose_mmol_l / 22.5"),
    );
    working.insert("limitations".into(), json!(LIMITATIONS));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(rounded),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct HomaIr;

impl Calculator for HomaIr {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "HOMA-IR"
    }

    fn description(&self) -> &'static str {
        "Calculates the continuous original HOMA1 surrogate estimate from fasting glucose and fasting insulin."
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
            "title": "HomaIrInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["fasting_glucose", "glucose_unit", "fasting_insulin_miu_l"],
            "properties": {
                "fasting_glucose": {
                    "type": "number",
                    "description": "Fasting plasma glucose from the same fasting sample as insulin. Enter 1.7-27.8 mmol/L or 30-500 mg/dL according to glucose_unit; values are accepted only as a broad analytic range, not as a diagnostic domain."
                },
                "glucose_unit": {
                    "type": "string",
                    "enum": ["mmol/L", "mg/dL"],
                    "description": "Unit of fasting_glucose. mg/dL is converted using exactly 18 mg/dL per mmol/L; do not enter mg/dL while selecting mmol/L or vice versa."
                },
                "fasting_insulin_miu_l": {
                    "type": "number",
                    "minimum": 0.5,
                    "maximum": 300,
                    "description": "Fasting insulin from the same fasting sample as glucose, in mIU/L, numerically equivalent to microU/mL. The 0.5-300 mIU/L bounds are a broad analytic range, not diagnostic thresholds. Do not use in people receiving exogenous insulin unless supported by a validated protocol."
                }
            },
            "allOf": [
                {
                    "if": {
                        "properties": { "glucose_unit": { "const": "mmol/L" } },
                        "required": ["glucose_unit"]
                    },
                    "then": {
                        "properties": { "fasting_glucose": { "minimum": 1.7, "maximum": 27.8 } }
                    }
                },
                {
                    "if": {
                        "properties": { "glucose_unit": { "const": "mg/dL" } },
                        "required": ["glucose_unit"]
                    },
                    "then": {
                        "properties": { "fasting_glucose": { "minimum": 30, "maximum": 500 } }
                    }
                }
            ]
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: HomaIrInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(glucose: f64, unit: GlucoseUnit, insulin: f64) -> HomaIrInput {
        HomaIrInput {
            fasting_glucose: glucose,
            glucose_unit: unit,
            fasting_insulin_miu_l: insulin,
        }
    }

    #[test]
    fn matthews_normalisation_vector_equals_one() {
        let outcome = compute(&input(4.5, GlucoseUnit::MmolL, 5.0)).unwrap();
        assert_eq!(outcome.homa_ir, 1.0);
        assert_eq!(outcome.fasting_glucose_mmol_l, 4.5);
    }

    #[test]
    fn mgdl_vector_is_equivalent() {
        let mmol = compute(&input(4.5, GlucoseUnit::MmolL, 5.0)).unwrap();
        let mgdl = compute(&input(81.0, GlucoseUnit::MgDl, 5.0)).unwrap();
        assert_eq!(mgdl.homa_ir, mmol.homa_ir);
        assert_eq!(mgdl.fasting_glucose_mmol_l, 4.5);
    }

    #[test]
    fn result_is_rounded_but_working_preserves_unrounded_value() {
        let response = build_response(&input(5.1, GlucoseUnit::MmolL, 7.3)).unwrap();
        assert_eq!(response.result, json!(1.65));
        assert_eq!(
            response.working["homa_ir_unrounded"],
            json!(7.3 * 5.1 / 22.5)
        );
    }

    #[test]
    fn dynamic_api_matches_typed_api() {
        let typed = input(81.0, GlucoseUnit::MgDl, 5.0);
        let dynamic = json!({
            "fasting_glucose": 81.0,
            "glucose_unit": "mg/dL",
            "fasting_insulin_miu_l": 5.0
        });
        assert_eq!(
            HomaIr.calculate(&dynamic).unwrap(),
            build_response(&typed).unwrap()
        );
    }

    #[test]
    fn rejects_nonfinite_values() {
        assert!(compute(&input(f64::NAN, GlucoseUnit::MmolL, 5.0)).is_err());
        assert!(compute(&input(4.5, GlucoseUnit::MmolL, f64::INFINITY)).is_err());
    }

    #[test]
    fn rejects_out_of_range_values() {
        assert!(compute(&input(1.69, GlucoseUnit::MmolL, 5.0)).is_err());
        assert!(compute(&input(27.81, GlucoseUnit::MmolL, 5.0)).is_err());
        assert!(compute(&input(29.9, GlucoseUnit::MgDl, 5.0)).is_err());
        assert!(compute(&input(500.1, GlucoseUnit::MgDl, 5.0)).is_err());
        assert!(compute(&input(4.5, GlucoseUnit::MmolL, 0.49)).is_err());
        assert!(compute(&input(4.5, GlucoseUnit::MmolL, 300.1)).is_err());
    }

    #[test]
    fn rejects_unknown_fields() {
        let value = json!({
            "fasting_glucose": 4.5,
            "glucose_unit": "mmol/L",
            "fasting_insulin_miu_l": 5.0,
            "fasting": true
        });
        assert!(HomaIr.calculate(&value).is_err());
    }

    #[test]
    fn schema_is_closed_and_describes_fasting_units() {
        let schema = HomaIr.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(
            schema["properties"]["glucose_unit"]["enum"],
            json!(["mmol/L", "mg/dL"])
        );
        assert!(
            schema["properties"]["fasting_glucose"]["description"]
                .as_str()
                .unwrap()
                .contains("fasting sample")
        );
        assert!(
            schema["properties"]["fasting_insulin_miu_l"]["description"]
                .as_str()
                .unwrap()
                .contains("microU/mL")
        );
    }
}
