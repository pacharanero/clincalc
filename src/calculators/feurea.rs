// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! FEUrea - Fractional Excretion of Urea.
//!
//! Formula: FEUrea (%) = (U_urea x P_Cr) / (P_urea x U_Cr) x 100
//!
//! Sodium-independent companion to the shipped `fena`, usable when a patient
//! is on diuretics (which raise urine sodium excretion and make FENa
//! unreliable) because loop and thiazide diuretics have much less effect on
//! urea handling.
//!
//! Interpretation (Carvounis et al 2002):
//! - FEUrea < 35%: pre-renal aetiology
//! - FEUrea >= 35% and < 50%: indeterminate
//! - FEUrea >= 50%: intrinsic renal failure (e.g. acute tubular necrosis)

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "feurea";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.1046/j.1523-1755.2002.00683.x",
);

pub const REFERENCE: &str = "Carvounis CP, Nisar S, Guro-Razuman S. Significance of the fractional excretion of urea in the differential diagnosis of acute renal failure. Kidney Int. 2002;62(6):2223-2229. doi:10.1046/j.1523-1755.2002.00683.x";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeureaInput {
    /// Urine urea (any unit, same as plasma_urea)
    pub urine_urea: f64,
    /// Plasma urea (any unit, same as urine_urea)
    pub plasma_urea: f64,
    /// Urine creatinine (any unit, same as plasma_creatinine)
    pub urine_creatinine: f64,
    /// Plasma creatinine (any unit, same as urine_creatinine)
    pub plasma_creatinine: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FeureaOutcome {
    pub feurea_percent: f64,
    pub interpretation: String,
}

pub fn compute(input: &FeureaInput) -> Result<FeureaOutcome, CalcError> {
    for (name, value) in [
        ("urine_urea", input.urine_urea),
        ("plasma_urea", input.plasma_urea),
        ("urine_creatinine", input.urine_creatinine),
        ("plasma_creatinine", input.plasma_creatinine),
    ] {
        if value <= 0.0 || !value.is_finite() {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be a finite positive number"
            )));
        }
    }

    let feurea = (input.urine_urea * input.plasma_creatinine)
        / (input.plasma_urea * input.urine_creatinine)
        * 100.0;

    let category = if feurea < 35.0 {
        "pre-renal aetiology"
    } else if feurea < 50.0 {
        "indeterminate - clinical context and trend are required"
    } else {
        "intrinsic renal failure (e.g. acute tubular necrosis)"
    };

    let interpretation = format!(
        "FEUrea {feurea:.1}% - consistent with {category}. FEUrea is less affected by loop and thiazide diuretics than FENa, so it is preferred in diuretic-treated patients; it can still be affected by severe volume depletion and some drugs."
    );

    Ok(FeureaOutcome {
        feurea_percent: feurea,
        interpretation,
    })
}

pub fn build_response(input: &FeureaInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "feurea_percent".into(),
        json!(format!("{:.1}", o.feurea_percent)),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(format!("{:.1}", o.feurea_percent)),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Feurea;

impl Calculator for Feurea {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "FEUrea (Fractional Excretion of Urea)"
    }
    fn description(&self) -> &'static str {
        "Differentiates pre-renal from intrinsic renal failure using urine and plasma urea and creatinine; more reliable than FENa in diuretic-treated patients."
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
            "title": "FeureaInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["urine_urea", "plasma_urea", "urine_creatinine", "plasma_creatinine"],
            "properties": {
                "urine_urea": { "type": "number", "exclusiveMinimum": 0, "description": "Urine urea (any unit, must match plasma_urea unit)" },
                "plasma_urea": { "type": "number", "exclusiveMinimum": 0, "description": "Plasma/serum urea (same unit as urine_urea)" },
                "urine_creatinine": { "type": "number", "exclusiveMinimum": 0, "description": "Urine creatinine (any unit, must match plasma_creatinine unit)" },
                "plasma_creatinine": { "type": "number", "exclusiveMinimum": 0, "description": "Plasma/serum creatinine (same unit as urine_creatinine)" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: FeureaInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(u_urea: f64, p_urea: f64, u_cr: f64, p_cr: f64) -> FeureaInput {
        FeureaInput {
            urine_urea: u_urea,
            plasma_urea: p_urea,
            urine_creatinine: u_cr,
            plasma_creatinine: p_cr,
        }
    }

    #[test]
    fn prerenal_low_feurea() {
        // U_urea=300, P_urea=10, U_Cr=120, P_Cr=2.0 -> (300*2)/(10*120)*100 = 50%... adjust for a clearly low case
        let o = compute(&inp(150.0, 10.0, 120.0, 2.0)).unwrap();
        assert!(
            (o.feurea_percent - 25.0).abs() < 0.01,
            "got {}",
            o.feurea_percent
        );
        assert!(o.interpretation.contains("pre-renal"));
    }

    #[test]
    fn intrinsic_high_feurea() {
        let o = compute(&inp(400.0, 10.0, 60.0, 2.0)).unwrap();
        assert!(o.feurea_percent >= 50.0, "got {}", o.feurea_percent);
        assert!(o.interpretation.contains("intrinsic"));
    }

    #[test]
    fn rejects_zero_inputs() {
        assert!(compute(&inp(0.0, 10.0, 120.0, 2.0)).is_err());
        assert!(compute(&inp(150.0, 0.0, 120.0, 2.0)).is_err());
        assert!(compute(&inp(150.0, 10.0, 0.0, 2.0)).is_err());
        assert!(compute(&inp(150.0, 10.0, 120.0, 0.0)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "urine_urea": 150.0, "plasma_urea": 10.0, "urine_creatinine": 120.0, "plasma_creatinine": 2.0
        });
        let dynamic = Feurea.calculate(&value).unwrap();
        let typed = build_response(&inp(150.0, 10.0, 120.0, 2.0)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            Feurea
                .calculate(&json!({"urine_urea": 150.0, "plasma_urea": 10.0}))
                .is_err()
        );
    }
}
