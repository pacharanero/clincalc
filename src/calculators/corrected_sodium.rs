// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Hyperglycaemia-corrected sodium (Katz / Hillier).
//!
//! Hyperglycaemia draws water osmotically into the extracellular space,
//! diluting measured sodium. This estimates what sodium would read once
//! glucose normalises - useful when working up DKA/HHS to see whether a low
//! measured sodium reflects true hyponatraemia or is an expected artefact of
//! the hyperglycaemia.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "corrected_sodium";
pub const REFERENCE: &str = "Katz MA. Hyperglycemia-induced hyponatremia--calculation of expected serum sodium depression. N Engl J Med. 1973;289(16):843-844. doi:10.1056/NEJM197310182891607; Hillier TA, Abbott RD, Barrett EJ. Hyponatremia: evaluating the correction factor for hyperglycemia. Am J Med. 1999;106(4):399-403. doi:10.1016/s0002-9343(99)00055-8";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1056/NEJM197310182891607",
};

/// mg/dL per mmol/L for glucose (molar mass 180.16 g/mol).
pub const MGDL_PER_MMOL_GLUCOSE: f64 = 18.016;

/// The reference glucose concentration (mg/dL) both correction factors are
/// anchored to - normoglycaemia, at which no correction applies.
pub const NORMAL_GLUCOSE_MGDL: f64 = 100.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlucoseUnit {
    #[serde(rename = "mmol/L")]
    MmolL,
    #[serde(rename = "mg/dL")]
    MgDl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrectionMethod {
    #[serde(rename = "katz")]
    Katz,
    #[serde(rename = "hillier")]
    Hillier,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorrectedSodiumInput {
    pub sodium: f64,
    pub glucose: f64,
    pub glucose_unit: GlucoseUnit,
    pub method: CorrectionMethod,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CorrectedSodiumOutcome {
    pub corrected_sodium: f64,
    pub glucose_mgdl: f64,
    pub method: CorrectionMethod,
    pub interpretation: String,
}

pub fn compute(input: &CorrectedSodiumInput) -> Result<CorrectedSodiumOutcome, CalcError> {
    if !input.sodium.is_finite() || !(80.0..=200.0).contains(&input.sodium) {
        return Err(CalcError::InvalidInput(
            "sodium must be a finite number between 80 and 200 mmol/L".into(),
        ));
    }
    if !input.glucose.is_finite() || input.glucose <= 0.0 {
        return Err(CalcError::InvalidInput(
            "glucose must be a positive finite number".into(),
        ));
    }

    let glucose_mgdl = match input.glucose_unit {
        GlucoseUnit::MgDl => input.glucose,
        GlucoseUnit::MmolL => input.glucose * MGDL_PER_MMOL_GLUCOSE,
    };
    if !(18.0..=2500.0).contains(&glucose_mgdl) {
        return Err(CalcError::InvalidInput(
            "glucose must be between 18 and 2500 mg/dL (or equivalent)".into(),
        ));
    }

    let factor_per_100mgdl = match input.method {
        CorrectionMethod::Katz => 1.6,
        CorrectionMethod::Hillier => 2.4,
    };
    let corrected_sodium =
        input.sodium + factor_per_100mgdl * (glucose_mgdl - NORMAL_GLUCOSE_MGDL) / 100.0;

    let method_name = match input.method {
        CorrectionMethod::Katz => "Katz (1.6 mmol/L per 100 mg/dL glucose above 100 mg/dL)",
        CorrectionMethod::Hillier => "Hillier (2.4 mmol/L per 100 mg/dL glucose above 100 mg/dL)",
    };
    let interpretation = format!(
        "Corrected sodium is {:.1} mmol/L using the {method_name} correction. This estimates \
         what sodium would read once glucose normalises; a corrected value still low suggests \
         true (non-hyperglycaemic) hyponatraemia. Hillier's factor is empirically more accurate \
         at markedly elevated glucose (Katz tends to underestimate the true rise above roughly \
         400 mg/dL / 22 mmol/L).",
        corrected_sodium
    );

    Ok(CorrectedSodiumOutcome {
        corrected_sodium,
        glucose_mgdl,
        method: input.method,
        interpretation,
    })
}

pub fn build_response(input: &CorrectedSodiumInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("glucose_mgdl".into(), json!(o.glucose_mgdl));
    working.insert(
        "method".into(),
        json!(match o.method {
            CorrectionMethod::Katz => "katz",
            CorrectionMethod::Hillier => "hillier",
        }),
    );
    working.insert(
        "factor_mmol_per_100mgdl".into(),
        json!(match o.method {
            CorrectionMethod::Katz => 1.6,
            CorrectionMethod::Hillier => 2.4,
        }),
    );
    working.insert(
        "formula".into(),
        json!("sodium + factor * (glucose_mgdl - 100) / 100"),
    );
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!((o.corrected_sodium * 10.0).round() / 10.0),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct CorrectedSodium;

impl Calculator for CorrectedSodium {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Hyperglycaemia-corrected Sodium"
    }
    fn description(&self) -> &'static str {
        "Estimates expected serum sodium at normoglycaemia in hyperglycaemia (DKA/HHS workup), using the Katz or Hillier correction factor."
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
            "title": "CorrectedSodiumInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["sodium", "glucose", "glucose_unit", "method"],
            "properties": {
                "sodium": { "type": "number", "minimum": 80, "maximum": 200, "description": "Measured serum sodium, mmol/L" },
                "glucose": { "type": "number", "exclusiveMinimum": 0, "description": "Serum glucose" },
                "glucose_unit": { "type": "string", "enum": ["mmol/L", "mg/dL"] },
                "method": { "type": "string", "enum": ["katz", "hillier"], "description": "Correction factor: Katz (1.6, 1973) or Hillier (2.4, 1999)" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: CorrectedSodiumInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn katz_matches_worked_example_mgdl() {
        // Na 130, glucose 600 mg/dL: 130 + 1.6 * (600-100)/100 = 130 + 8 = 138.
        let out = compute(&CorrectedSodiumInput {
            sodium: 130.0,
            glucose: 600.0,
            glucose_unit: GlucoseUnit::MgDl,
            method: CorrectionMethod::Katz,
        })
        .unwrap();
        assert_eq!((out.corrected_sodium * 10.0).round() / 10.0, 138.0);
    }

    #[test]
    fn hillier_matches_worked_example_mgdl() {
        // Na 130, glucose 600 mg/dL: 130 + 2.4 * (600-100)/100 = 130 + 12 = 142.
        let out = compute(&CorrectedSodiumInput {
            sodium: 130.0,
            glucose: 600.0,
            glucose_unit: GlucoseUnit::MgDl,
            method: CorrectionMethod::Hillier,
        })
        .unwrap();
        assert_eq!((out.corrected_sodium * 10.0).round() / 10.0, 142.0);
    }

    #[test]
    fn mmol_l_glucose_matches_equivalent_mgdl() {
        // 33.3 mmol/L glucose ~= 600 mg/dL.
        let mmol = compute(&CorrectedSodiumInput {
            sodium: 130.0,
            glucose: 600.0 / MGDL_PER_MMOL_GLUCOSE,
            glucose_unit: GlucoseUnit::MmolL,
            method: CorrectionMethod::Katz,
        })
        .unwrap();
        let mgdl = compute(&CorrectedSodiumInput {
            sodium: 130.0,
            glucose: 600.0,
            glucose_unit: GlucoseUnit::MgDl,
            method: CorrectionMethod::Katz,
        })
        .unwrap();
        assert!((mmol.corrected_sodium - mgdl.corrected_sodium).abs() < 0.01);
    }

    #[test]
    fn normal_glucose_applies_no_correction() {
        let out = compute(&CorrectedSodiumInput {
            sodium: 140.0,
            glucose: 100.0,
            glucose_unit: GlucoseUnit::MgDl,
            method: CorrectionMethod::Katz,
        })
        .unwrap();
        assert_eq!((out.corrected_sodium * 10.0).round() / 10.0, 140.0);
    }

    #[test]
    fn rejects_out_of_range_sodium() {
        let err = compute(&CorrectedSodiumInput {
            sodium: 400.0,
            glucose: 150.0,
            glucose_unit: GlucoseUnit::MgDl,
            method: CorrectionMethod::Katz,
        })
        .unwrap_err();
        assert!(matches!(err, CalcError::InvalidInput(_)));
    }

    #[test]
    fn rejects_non_positive_glucose() {
        let err = compute(&CorrectedSodiumInput {
            sodium: 135.0,
            glucose: 0.0,
            glucose_unit: GlucoseUnit::MgDl,
            method: CorrectionMethod::Katz,
        })
        .unwrap_err();
        assert!(matches!(err, CalcError::InvalidInput(_)));
    }
}
