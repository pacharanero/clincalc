// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Richmond Agitation-Sedation Scale (RASS) - a ten-point ordinal scale of
//! sedation/agitation for mechanically ventilated and other ICU patients.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "rass";
pub const REFERENCE: &str = "Sessler CN, Gosnell MS, Grap MJ, et al. The Richmond Agitation-Sedation Scale: validity and reliability in adult intensive care unit patients. Am J Respir Crit Care Med. 2002;166(10):1338-1344.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Published for clinical and research use by its developers (Virginia Commonwealth University / Richmond VA Medical Center); implemented from the primary literature",
    "https://doi.org/10.1164/rccm.2107138",
);

pub const MIN_LEVEL: i8 = -5;
pub const MAX_LEVEL: i8 = 4;

fn descriptor(level: i8) -> &'static str {
    match level {
        4 => "Combative: overtly combative or violent; immediate danger to staff",
        3 => {
            "Very agitated: pulls or removes tube(s) or catheter(s), or is aggressive toward staff"
        }
        2 => "Agitated: frequent non-purposeful movement, fights the ventilator",
        1 => "Restless: anxious or apprehensive, but movements not aggressive or vigorous",
        0 => "Alert and calm",
        -1 => {
            "Drowsy: not fully alert, but has sustained (>10 seconds) awakening, with eye contact, to voice"
        }
        -2 => "Light sedation: briefly (<10 seconds) awakens, with eye contact, to voice",
        -3 => "Moderate sedation: any movement (but no eye contact) to voice",
        -4 => "Deep sedation: no response to voice, but any movement to physical stimulation",
        -5 => "Unarousable: no response to voice or physical stimulation",
        _ => unreachable!("level is validated to -5..=4 before this is called"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RassInput {
    pub level: i8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RassOutcome {
    pub level: i8,
    pub descriptor: &'static str,
}

pub fn compute(input: &RassInput) -> Result<RassOutcome, CalcError> {
    if !(MIN_LEVEL..=MAX_LEVEL).contains(&input.level) {
        return Err(CalcError::InvalidInput(format!(
            "level must be an integer from {MIN_LEVEL} to {MAX_LEVEL}, got {}",
            input.level
        )));
    }
    Ok(RassOutcome {
        level: input.level,
        descriptor: descriptor(input.level),
    })
}

pub fn build_response(input: &RassInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("descriptor".into(), json!(o.descriptor));

    let interpretation = format!(
        "RASS {:+}: {}. Assessed by observation, then verbal stimulus, then physical stimulus if needed, in that order - score the earliest response obtained. A positive RASS (agitation) and a RASS below the target sedation level both warrant review of analgesia, sedation, and reversible causes (e.g. delirium, pain, hypoxia) rather than sedative titration alone.",
        o.level, o.descriptor
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.level),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Rass;
impl Calculator for Rass {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Richmond Agitation-Sedation Scale (RASS)"
    }
    fn description(&self) -> &'static str {
        "Ten-point ordinal sedation/agitation scale for ICU patients, from +4 (combative) to -5 (unarousable)."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "RassInput", "type": "object", "additionalProperties": false,
            "required": ["level"],
            "properties": {
                "level": { "type": "integer", "minimum": -5, "maximum": 4 }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: RassInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_and_calm() {
        let out = compute(&RassInput { level: 0 }).unwrap();
        assert_eq!(out.descriptor, "Alert and calm");
    }

    #[test]
    fn combative_top_of_scale() {
        let out = compute(&RassInput { level: 4 }).unwrap();
        assert!(out.descriptor.starts_with("Combative"));
    }

    #[test]
    fn unarousable_bottom_of_scale() {
        let out = compute(&RassInput { level: -5 }).unwrap();
        assert!(out.descriptor.starts_with("Unarousable"));
    }

    #[test]
    fn rejects_out_of_range_level() {
        assert!(compute(&RassInput { level: 5 }).is_err());
        assert!(compute(&RassInput { level: -6 }).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"level": -2});
        let typed = RassInput { level: -2 };
        let dynamic = Rass.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(Rass.calculate(&json!({})).is_err());
    }
}
