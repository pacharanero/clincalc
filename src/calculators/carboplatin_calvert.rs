// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Carboplatin dose by the Calvert formula (target-AUC dosing from renal
//! function).
//!
//! This is a calculation aid, not a substitute for independent verification.
//! Carboplatin dosing is routinely subject to a second-checker policy in
//! oncology pharmacy practice; the target AUC, any GFR cap, and the
//! resulting dose must be verified against the local chemotherapy protocol
//! before administration.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "carboplatin_calvert";
pub const REFERENCE: &str = "Calvert AH, Newell DR, Gumbrell LA, et al. Carboplatin dosage: prospective evaluation of a simple formula based on renal function. J Clin Oncol. 1989;7(11):1748-1756. The 125 mL/min GFR cap for AUC dosing is widely adopted regulatory/manufacturer labelling guidance (e.g. Paraplatin prescribing information) to avoid overdosing patients with supranormal measured or estimated GFR.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain formula - implemented from the primary literature",
    "https://doi.org/10.1200/JCO.1989.7.11.1748",
);

const GFR_CAP_ML_MIN: f64 = 125.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarboplatinCalvertInput {
    /// Target AUC in mg/mL x min, per the chosen regimen/protocol.
    pub target_auc_mg_ml_min: f64,
    /// Measured or estimated GFR (e.g. Cockcroft-Gault creatinine clearance
    /// or an isotopic/measured GFR), mL/min. Capped at 125 mL/min for dosing
    /// purposes per standard labelling guidance.
    pub gfr_ml_min: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarboplatinCalvertOutcome {
    pub dose_mg: f64,
    pub gfr_used_ml_min: f64,
    pub gfr_was_capped: bool,
}

pub fn compute(input: &CarboplatinCalvertInput) -> Result<CarboplatinCalvertOutcome, CalcError> {
    validate(
        input.target_auc_mg_ml_min,
        "target_auc_mg_ml_min",
        1.0,
        12.0,
    )?;
    validate(input.gfr_ml_min, "gfr_ml_min", 5.0, 250.0)?;

    let gfr_was_capped = input.gfr_ml_min > GFR_CAP_ML_MIN;
    let gfr_used_ml_min = input.gfr_ml_min.min(GFR_CAP_ML_MIN);
    let dose_mg = input.target_auc_mg_ml_min * (gfr_used_ml_min + 25.0);

    Ok(CarboplatinCalvertOutcome {
        dose_mg,
        gfr_used_ml_min,
        gfr_was_capped,
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

pub fn build_response(input: &CarboplatinCalvertInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("gfr_used_ml_min".into(), json!(round1(o.gfr_used_ml_min)));
    working.insert("gfr_was_capped".into(), json!(o.gfr_was_capped));
    working.insert("unit".into(), json!("mg"));

    let cap_note = if o.gfr_was_capped {
        format!(
            " The supplied GFR ({:.0} mL/min) exceeded the standard 125 mL/min dosing cap, so 125 mL/min was used in the calculation.",
            input.gfr_ml_min
        )
    } else {
        String::new()
    };

    let interpretation = format!(
        "Calvert formula dose: {:.0} mg (target AUC {:.1} mg/mL.min x (GFR {:.0} mL/min + 25)).{} This is a calculation aid only - verify the target AUC, the GFR used (and any local dose-capping or rounding policy), and the final dose against the chemotherapy protocol and an independent second checker before prescribing or administering.",
        o.dose_mg, input.target_auc_mg_ml_min, o.gfr_used_ml_min, cap_note
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round0(o.dose_mg)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round0(v: f64) -> f64 {
    v.round()
}
fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct CarboplatinCalvert;
impl Calculator for CarboplatinCalvert {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Carboplatin Dosing (Calvert Formula)"
    }
    fn description(&self) -> &'static str {
        "Target-AUC carboplatin dose from renal function (Calvert formula), with the standard 125 mL/min GFR dosing cap."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "CarboplatinCalvertInput", "type": "object", "additionalProperties": false,
            "required": ["target_auc_mg_ml_min", "gfr_ml_min"],
            "properties": {
                "target_auc_mg_ml_min": { "type": "number", "minimum": 1, "maximum": 12, "description": "Target AUC in mg/mL x min, per the chosen regimen/protocol" },
                "gfr_ml_min": { "type": "number", "minimum": 5, "maximum": 250, "description": "Measured or estimated GFR, mL/min; capped at 125 mL/min for dosing purposes" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: CarboplatinCalvertInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_auc5_dose() {
        let out = compute(&CarboplatinCalvertInput {
            target_auc_mg_ml_min: 5.0,
            gfr_ml_min: 80.0,
        })
        .unwrap();
        // 5 * (80 + 25) = 525
        assert_eq!(out.dose_mg, 525.0);
        assert!(!out.gfr_was_capped);
    }

    #[test]
    fn gfr_above_cap_is_capped_at_125() {
        let out = compute(&CarboplatinCalvertInput {
            target_auc_mg_ml_min: 5.0,
            gfr_ml_min: 160.0,
        })
        .unwrap();
        // 5 * (125 + 25) = 750, not 5 * (160+25) = 925
        assert_eq!(out.dose_mg, 750.0);
        assert!(out.gfr_was_capped);
        assert_eq!(out.gfr_used_ml_min, 125.0);
    }

    #[test]
    fn gfr_at_cap_boundary_not_flagged_as_capped() {
        let out = compute(&CarboplatinCalvertInput {
            target_auc_mg_ml_min: 5.0,
            gfr_ml_min: 125.0,
        })
        .unwrap();
        assert!(!out.gfr_was_capped);
    }

    #[test]
    fn rejects_out_of_range_auc() {
        assert!(
            compute(&CarboplatinCalvertInput {
                target_auc_mg_ml_min: 15.0,
                gfr_ml_min: 80.0,
            })
            .is_err()
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"target_auc_mg_ml_min": 5.0, "gfr_ml_min": 80.0});
        let typed = CarboplatinCalvertInput {
            target_auc_mg_ml_min: 5.0,
            gfr_ml_min: 80.0,
        };
        let dynamic = CarboplatinCalvert.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            CarboplatinCalvert
                .calculate(&json!({"target_auc_mg_ml_min": 5.0}))
                .is_err()
        );
    }
}
