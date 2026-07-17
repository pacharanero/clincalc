// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! ASA Physical Status classification.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "asa_physical_status";
pub const REFERENCE: &str = "American Society of Anesthesiologists. ASA Physical Status Classification System. Last amended Dec 13, 2020.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public professional classification - implemented from ASA published definitions",
    source_url: "https://www.asahq.org/standards-and-practice-parameters/statement-on-asa-physical-status-classification-system",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AsaClass {
    Asa1,
    Asa2,
    Asa3,
    Asa4,
    Asa5,
    Asa6,
}
impl AsaClass {
    fn number(self) -> u8 {
        match self {
            AsaClass::Asa1 => 1,
            AsaClass::Asa2 => 2,
            AsaClass::Asa3 => 3,
            AsaClass::Asa4 => 4,
            AsaClass::Asa5 => 5,
            AsaClass::Asa6 => 6,
        }
    }
    fn definition(self) -> &'static str {
        match self {
            AsaClass::Asa1 => "A normal healthy patient",
            AsaClass::Asa2 => "A patient with mild systemic disease",
            AsaClass::Asa3 => "A patient with severe systemic disease",
            AsaClass::Asa4 => {
                "A patient with severe systemic disease that is a constant threat to life"
            }
            AsaClass::Asa5 => {
                "A moribund patient who is not expected to survive without the operation"
            }
            AsaClass::Asa6 => {
                "A declared brain-dead patient whose organs are being removed for donor purposes"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AsaPhysicalStatusInput {
    pub asa_class: AsaClass,
    #[serde(default)]
    pub emergency: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsaPhysicalStatusOutcome {
    pub label: String,
    pub interpretation: String,
}

pub fn compute(input: &AsaPhysicalStatusInput) -> Result<AsaPhysicalStatusOutcome, CalcError> {
    let suffix = if input.emergency { "E" } else { "" };
    let label = format!("ASA {}{suffix}", input.asa_class.number());
    let emergency_note = if input.emergency {
        " The E suffix denotes an emergency operation."
    } else {
        ""
    };
    let interpretation = format!(
        "{label}: {}.{emergency_note} ASA Physical Status describes preoperative physical status; it is not by itself an operative risk score.",
        input.asa_class.definition()
    );
    Ok(AsaPhysicalStatusOutcome {
        label,
        interpretation,
    })
}

pub fn build_response(input: &AsaPhysicalStatusInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("asa_class".into(), json!(input.asa_class.number()));
    working.insert("emergency_suffix".into(), json!(input.emergency));
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.label),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct AsaPhysicalStatus;
impl Calculator for AsaPhysicalStatus {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "ASA Physical Status"
    }
    fn description(&self) -> &'static str {
        "ASA preoperative physical-status classification, with optional emergency suffix."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({ "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "AsaPhysicalStatusInput", "type": "object", "additionalProperties": false, "required": ["asa_class"], "properties": { "asa_class": { "type": "string", "enum": ["asa1", "asa2", "asa3", "asa4", "asa5", "asa6"], "description": "ASA class 1-6 using ASA definitions" }, "emergency": { "type": "boolean", "description": "Append E suffix for emergency operation" } } })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AsaPhysicalStatusInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn emergency_suffix_is_added() {
        let out = compute(&AsaPhysicalStatusInput {
            asa_class: AsaClass::Asa3,
            emergency: true,
        })
        .unwrap();
        assert_eq!(out.label, "ASA 3E");
    }
}
