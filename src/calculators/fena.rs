// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! FENa - Fractional Excretion of Sodium.
//!
//! Formula: FENa (%) = (U_Na × P_Cr) / (P_Na × U_Cr) × 100
//!
//! Because sodium appears in both numerator and denominator (same unit), and
//! creatinine likewise, the units cancel as long as each pair is consistent.
//! The recommended approach is to express both sodium values in mEq/L (or
//! mmol/L - numerically identical) and both creatinine values in the same
//! unit (mg/dL or µmol/L).
//!
//! Interpretation:
//! - FENa < 1%: pre-renal aetiology (or contrast nephropathy, myoglobinuria)
//! - FENa 1–2%: indeterminate
//! - FENa > 2%: intrinsic renal failure (ATN, etc.)
//!
//! Caveat: FENa is unreliable in patients on diuretics, CKD, non-oliguric ATN,
//! or early obstruction. FEUrea may be more useful after diuretic use.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "fena";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1001/jama.1976.03270060029020",
};

pub const REFERENCE: &str = "Espinel CH. The FENa test. Use in the differential diagnosis of acute renal failure. \
JAMA. 1976;236(6):579-581. doi:10.1001/jama.1976.03270060029020";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FenaInput {
    /// Urine sodium (mEq/L or mmol/L)
    pub urine_sodium: f64,
    /// Plasma sodium (mEq/L or mmol/L - same unit as urine_sodium)
    pub plasma_sodium: f64,
    /// Urine creatinine (any unit, but same as plasma_creatinine)
    pub urine_creatinine: f64,
    /// Plasma creatinine (any unit, but same as urine_creatinine)
    pub plasma_creatinine: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FenaOutcome {
    pub fena_percent: f64,
    pub interpretation: String,
}

pub fn compute(input: &FenaInput) -> Result<FenaOutcome, CalcError> {
    if input.urine_sodium <= 0.0 || !input.urine_sodium.is_finite() {
        return Err(CalcError::InvalidInput(
            "urine_sodium must be positive".into(),
        ));
    }
    if input.plasma_sodium <= 0.0 || !input.plasma_sodium.is_finite() {
        return Err(CalcError::InvalidInput(
            "plasma_sodium must be positive".into(),
        ));
    }
    if input.urine_creatinine <= 0.0 || !input.urine_creatinine.is_finite() {
        return Err(CalcError::InvalidInput(
            "urine_creatinine must be positive".into(),
        ));
    }
    if input.plasma_creatinine <= 0.0 || !input.plasma_creatinine.is_finite() {
        return Err(CalcError::InvalidInput(
            "plasma_creatinine must be positive".into(),
        ));
    }

    let fena = (input.urine_sodium * input.plasma_creatinine)
        / (input.plasma_sodium * input.urine_creatinine)
        * 100.0;

    let category = if fena < 1.0 {
        "pre-renal aetiology (or conditions causing low FENa with intrinsic disease: contrast nephropathy, myoglobinuria, early obstruction)"
    } else if fena <= 2.0 {
        "indeterminate - clinical context and trend are required"
    } else {
        "intrinsic renal failure (e.g. acute tubular necrosis)"
    };

    let interpretation = format!(
        "FENa {:.2}% - consistent with {}. Note: FENa is unreliable after diuretics, in CKD, \
non-oliguric ATN, or early obstruction. Consider FEUrea in patients on diuretics.",
        fena, category
    );

    Ok(FenaOutcome {
        fena_percent: fena,
        interpretation,
    })
}

pub fn build_response(input: &FenaInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert(
        "fena_percent".into(),
        json!(format!("{:.2}", o.fena_percent)),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(format!("{:.2}", o.fena_percent)),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Fena;

impl Calculator for Fena {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "FENa (Fractional Excretion of Sodium)"
    }

    fn description(&self) -> &'static str {
        "Differentiates pre-renal from intrinsic renal failure using urine and plasma sodium and creatinine."
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
            "title": "FenaInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["urine_sodium", "plasma_sodium", "urine_creatinine", "plasma_creatinine"],
            "properties": {
                "urine_sodium": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Urine sodium (mEq/L or mmol/L)"
                },
                "plasma_sodium": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Plasma/serum sodium (same unit as urine_sodium)"
                },
                "urine_creatinine": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Urine creatinine (any unit, must match plasma_creatinine unit)"
                },
                "plasma_creatinine": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Plasma/serum creatinine (same unit as urine_creatinine)"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: FenaInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(u_na: f64, p_na: f64, u_cr: f64, p_cr: f64) -> FenaInput {
        FenaInput {
            urine_sodium: u_na,
            plasma_sodium: p_na,
            urine_creatinine: u_cr,
            plasma_creatinine: p_cr,
        }
    }

    #[test]
    fn prerenal_low_fena() {
        // Classic pre-renal: low urine Na, high urine Cr concentration.
        // U_Na=20, P_Na=140, U_Cr=120, P_Cr=2.0 -> FENa = (20*2)/(140*120)*100 = 0.238%
        let o = compute(&inp(20.0, 140.0, 120.0, 2.0)).unwrap();
        assert!(
            (o.fena_percent - 0.238).abs() < 0.001,
            "got {}",
            o.fena_percent
        );
        assert!(o.interpretation.contains("pre-renal"));
    }

    #[test]
    fn intrinsic_high_fena() {
        // ATN: high urine Na, lower urine Cr concentration.
        // U_Na=80, P_Na=140, U_Cr=30, P_Cr=2.0 -> FENa = (80*2)/(140*30)*100 = 3.81%
        let o = compute(&inp(80.0, 140.0, 30.0, 2.0)).unwrap();
        assert!(
            (o.fena_percent - 3.810).abs() < 0.01,
            "got {}",
            o.fena_percent
        );
        assert!(o.interpretation.contains("intrinsic"));
    }

    #[test]
    fn indeterminate_range() {
        // FENa exactly 1.5%: U_Na=42, P_Na=140, U_Cr=40, P_Cr=2.0
        // (42*2)/(140*40)*100 = 84/5600*100 = 1.5%
        let o = compute(&inp(42.0, 140.0, 40.0, 2.0)).unwrap();
        assert!(
            (o.fena_percent - 1.5).abs() < 0.001,
            "got {}",
            o.fena_percent
        );
        assert!(o.interpretation.contains("indeterminate"));
    }

    #[test]
    fn rejects_zero_inputs() {
        assert!(compute(&inp(0.0, 140.0, 40.0, 2.0)).is_err());
        assert!(compute(&inp(20.0, 0.0, 40.0, 2.0)).is_err());
        assert!(compute(&inp(20.0, 140.0, 0.0, 2.0)).is_err());
        assert!(compute(&inp(20.0, 140.0, 40.0, 0.0)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "urine_sodium": 20.0,
            "plasma_sodium": 140.0,
            "urine_creatinine": 120.0,
            "plasma_creatinine": 2.0
        });
        let dynamic = Fena.calculate(&value).unwrap();
        let typed = build_response(&inp(20.0, 140.0, 120.0, 2.0)).unwrap();
        assert_eq!(dynamic, typed);
    }
}
