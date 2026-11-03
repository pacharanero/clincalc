// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Absolute neutrophil count (ANC) from white cell count and differential,
//! with CTCAE-style neutropenia severity grading.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "anc";
pub const REFERENCE: &str = "Standard haematology formula: ANC = WBC x (%neutrophils + %bands) / 100. Severity bands per National Cancer Institute. Common Terminology Criteria for Adverse Events (CTCAE) v5.0. 2017.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain formula and US-government-work grading criteria - implemented from standard haematology and CTCAE v5.0",
    "https://ctep.cancer.gov/protocoldevelopment/electronic_applications/docs/ctcae_v5_quick_reference_5x7.pdf",
);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AncInput {
    /// White cell count, x10^9/L (numerically equal to x10^3/uL).
    pub wbc_x10_9_l: f64,
    /// Percentage of neutrophils (segmented/mature forms) on the differential.
    pub neutrophils_percent: f64,
    /// Percentage of band forms on the differential.
    #[serde(default)]
    pub bands_percent: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AncOutcome {
    pub anc_cells_per_ul: f64,
    pub band: &'static str,
}

pub fn compute(input: &AncInput) -> Result<AncOutcome, CalcError> {
    validate(input.wbc_x10_9_l, "wbc_x10_9_l", 0.1, 100.0)?;
    validate(input.neutrophils_percent, "neutrophils_percent", 0.0, 100.0)?;
    validate(input.bands_percent, "bands_percent", 0.0, 50.0)?;
    let total_percent = input.neutrophils_percent + input.bands_percent;
    if total_percent > 100.0 {
        return Err(CalcError::InvalidInput(
            "neutrophils_percent + bands_percent cannot exceed 100".to_string(),
        ));
    }

    let anc_cells_per_ul = input.wbc_x10_9_l * 1000.0 * total_percent / 100.0;
    let band = if anc_cells_per_ul >= 1500.0 {
        "normal (ANC >=1500/uL)"
    } else if anc_cells_per_ul >= 1000.0 {
        "CTCAE grade 1 neutropenia (1000-1499/uL)"
    } else if anc_cells_per_ul >= 500.0 {
        "CTCAE grade 2 neutropenia (500-999/uL)"
    } else if anc_cells_per_ul >= 100.0 {
        "CTCAE grade 3 neutropenia (100-499/uL)"
    } else {
        "CTCAE grade 4 neutropenia (<100/uL)"
    };

    Ok(AncOutcome {
        anc_cells_per_ul,
        band,
    })
}

fn validate(value: f64, name: &str, min: f64, max: f64) -> Result<(), CalcError> {
    if !(min..=max).contains(&value) || !value.is_finite() {
        return Err(CalcError::InvalidInput(format!(
            "{name} must be finite and between {min} and {max}"
        )));
    }
    Ok(())
}

pub fn build_response(input: &AncInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("band".into(), json!(o.band));
    working.insert("unit".into(), json!("cells/uL"));

    let interpretation = format!(
        "ANC {:.0} cells/uL ({}). An ANC below 500/uL (or below 1000/uL and expected to fall below 500/uL within 48 hours) in a patient with fever defines febrile neutropenia and is a medical emergency warranting urgent empirical antibiotics per local protocol.",
        o.anc_cells_per_ul, o.band
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round0(o.anc_cells_per_ul)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round0(v: f64) -> f64 {
    v.round()
}

pub struct Anc;
impl Calculator for Anc {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Absolute Neutrophil Count (ANC)"
    }
    fn description(&self) -> &'static str {
        "Absolute neutrophil count from white cell count and differential, with CTCAE neutropenia severity grading."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "AncInput", "type": "object", "additionalProperties": false,
            "required": ["wbc_x10_9_l", "neutrophils_percent"],
            "properties": {
                "wbc_x10_9_l": { "type": "number", "minimum": 0.1, "maximum": 100, "description": "White cell count, x10^9/L" },
                "neutrophils_percent": { "type": "number", "minimum": 0, "maximum": 100, "description": "Segmented/mature neutrophil percentage on the differential" },
                "bands_percent": { "type": "number", "minimum": 0, "maximum": 50, "description": "Optional band-form percentage; defaults to 0 if the differential does not report it" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AncInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_anc() {
        let out = compute(&AncInput {
            wbc_x10_9_l: 7.5,
            neutrophils_percent: 60.0,
            bands_percent: 0.0,
        })
        .unwrap();
        assert_eq!(out.anc_cells_per_ul, 4500.0);
        assert_eq!(out.band, "normal (ANC >=1500/uL)");
    }

    #[test]
    fn severe_neutropenia_with_bands() {
        let out = compute(&AncInput {
            wbc_x10_9_l: 1.0,
            neutrophils_percent: 10.0,
            bands_percent: 5.0,
        })
        .unwrap();
        assert_eq!(out.anc_cells_per_ul, 150.0);
        assert_eq!(out.band, "CTCAE grade 3 neutropenia (100-499/uL)");
    }

    #[test]
    fn rejects_percentages_exceeding_100() {
        assert!(
            compute(&AncInput {
                wbc_x10_9_l: 7.5,
                neutrophils_percent: 80.0,
                bands_percent: 30.0,
            })
            .is_err()
        );
    }

    #[test]
    fn bands_percent_defaults_to_zero() {
        let value = json!({"wbc_x10_9_l": 7.5, "neutrophils_percent": 60.0});
        let parsed: AncInput = serde_json::from_value(value).unwrap();
        assert_eq!(parsed.bands_percent, 0.0);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"wbc_x10_9_l": 7.5, "neutrophils_percent": 60.0, "bands_percent": 0.0});
        let typed = AncInput {
            wbc_x10_9_l: 7.5,
            neutrophils_percent: 60.0,
            bands_percent: 0.0,
        };
        let dynamic = Anc.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(Anc.calculate(&json!({"wbc_x10_9_l": 7.5})).is_err());
    }
}
