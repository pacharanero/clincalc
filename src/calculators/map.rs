// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Mean arterial pressure (MAP) from systolic and diastolic blood pressure.

use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "map";
pub const REFERENCE: &str = "Standard cardiovascular physiology formula (MAP = DBP + 1/3 x pulse pressure); see e.g. DeMers D, Wachs D. Physiology, Mean Arterial Pressure. StatPearls. 2023. The >=65 mmHg resuscitation target is per Evans L, Rhodes A, Alhazzani W, et al. Surviving Sepsis Campaign: International Guidelines for Management of Sepsis and Septic Shock 2021. Crit Care Med. 2021;49(11):e1063-e1143.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain formula and guideline threshold - implemented from standard physiology and the cited guideline",
    "https://doi.org/10.1097/CCM.0000000000005337",
);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapInput {
    pub systolic_bp_mm_hg: f64,
    pub diastolic_bp_mm_hg: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapOutcome {
    pub map_mm_hg: f64,
    pub pulse_pressure_mm_hg: f64,
    pub meets_sepsis_resuscitation_target: bool,
}

pub fn compute(input: &MapInput) -> Result<MapOutcome, CalcError> {
    validate(input.systolic_bp_mm_hg, "systolic_bp_mm_hg", 40.0, 300.0)?;
    validate(input.diastolic_bp_mm_hg, "diastolic_bp_mm_hg", 20.0, 200.0)?;
    if input.diastolic_bp_mm_hg >= input.systolic_bp_mm_hg {
        return Err(CalcError::InvalidInput(
            "diastolic_bp_mm_hg must be lower than systolic_bp_mm_hg".to_string(),
        ));
    }

    let pulse_pressure = input.systolic_bp_mm_hg - input.diastolic_bp_mm_hg;
    let map = input.diastolic_bp_mm_hg + pulse_pressure / 3.0;
    Ok(MapOutcome {
        map_mm_hg: map,
        pulse_pressure_mm_hg: pulse_pressure,
        meets_sepsis_resuscitation_target: map >= 65.0,
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

pub fn build_response(input: &MapInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = JsonMap::new();
    working.insert(
        "pulse_pressure_mm_hg".into(),
        json!(round1(o.pulse_pressure_mm_hg)),
    );
    working.insert(
        "meets_sepsis_resuscitation_target".into(),
        json!(o.meets_sepsis_resuscitation_target),
    );
    working.insert("unit".into(), json!("mmHg"));

    let interpretation = if o.map_mm_hg >= 65.0 {
        format!(
            "MAP {:.0} mmHg. Meets the commonly cited >=65 mmHg sepsis-resuscitation target (Surviving Sepsis Campaign 2021); this is a resuscitation threshold, not a universal definition of normal perfusion pressure for every patient.",
            o.map_mm_hg
        )
    } else {
        format!(
            "MAP {:.0} mmHg. Below the commonly cited >=65 mmHg sepsis-resuscitation target; correlate with the clinical context, as some patients (e.g. chronic hypertension) may need a higher individualised target.",
            o.map_mm_hg
        )
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.map_mm_hg)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct Map;
impl Calculator for Map {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Mean Arterial Pressure (MAP)"
    }
    fn description(&self) -> &'static str {
        "Mean arterial pressure from systolic and diastolic blood pressure, with the >=65 mmHg sepsis-resuscitation target."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "MapInput", "type": "object", "additionalProperties": false,
            "required": ["systolic_bp_mm_hg", "diastolic_bp_mm_hg"],
            "properties": {
                "systolic_bp_mm_hg": { "type": "number", "minimum": 40, "maximum": 300 },
                "diastolic_bp_mm_hg": { "type": "number", "minimum": 20, "maximum": 200, "description": "Must be lower than systolic_bp_mm_hg" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: MapInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_bp_120_80() {
        let out = compute(&MapInput {
            systolic_bp_mm_hg: 120.0,
            diastolic_bp_mm_hg: 80.0,
        })
        .unwrap();
        assert_eq!(out.map_mm_hg, 93.333_333_333_333_33);
        assert_eq!(out.pulse_pressure_mm_hg, 40.0);
        assert!(out.meets_sepsis_resuscitation_target);
    }

    #[test]
    fn hypotensive_below_sepsis_target() {
        let out = compute(&MapInput {
            systolic_bp_mm_hg: 80.0,
            diastolic_bp_mm_hg: 50.0,
        })
        .unwrap();
        assert_eq!(out.map_mm_hg, 60.0);
        assert!(!out.meets_sepsis_resuscitation_target);
    }

    #[test]
    fn rejects_diastolic_not_below_systolic() {
        assert!(
            compute(&MapInput {
                systolic_bp_mm_hg: 80.0,
                diastolic_bp_mm_hg: 80.0,
            })
            .is_err()
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"systolic_bp_mm_hg": 120.0, "diastolic_bp_mm_hg": 80.0});
        let typed = MapInput {
            systolic_bp_mm_hg: 120.0,
            diastolic_bp_mm_hg: 80.0,
        };
        let dynamic = Map.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(Map.calculate(&json!({"systolic_bp_mm_hg": 120.0})).is_err());
    }
}
