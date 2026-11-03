// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! CAM-ICU - Confusion Assessment Method for the ICU, a four-feature
//! delirium-detection algorithm for mechanically ventilated and other ICU
//! patients who cannot reliably speak.
//!
//! Unlike most calculators in this registry, CAM-ICU is not a summed score:
//! delirium is present when Feature 1 AND Feature 2 AND (Feature 3 OR
//! Feature 4) are all positive.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "cam_icu";
pub const REFERENCE: &str = "Ely EW, Inouye SK, Bernard GR, et al. Delirium in mechanically ventilated patients: validity and reliability of the confusion assessment method for the intensive care unit (CAM-ICU). JAMA. 2001;286(21):2703-2710.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Published for clinical and research use by its developers (Vanderbilt University); implemented from the primary literature. Official training/reference materials are separately copyrighted by Vanderbilt and are not reproduced here.",
    "https://doi.org/10.1001/jama.286.21.2703",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CamIcuInput {
    /// Feature 1: acute change or fluctuating course of mental status from
    /// baseline over the preceding 24 hours.
    pub acute_or_fluctuating_course: bool,
    /// Feature 2: inattention (e.g. errors on a letter-vigilance or picture
    /// test).
    pub inattention: bool,
    /// Feature 3: disorganised thinking (e.g. errors on yes/no questions or
    /// a simple command).
    pub disorganized_thinking: bool,
    /// Feature 4: altered level of consciousness, i.e. RASS other than 0
    /// (alert and calm). If RASS is -4 or -5, CAM-ICU is not assessable -
    /// do not use this calculator at that sedation depth.
    pub altered_consciousness: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CamIcuOutcome {
    pub delirium_present: bool,
}

pub fn compute(input: &CamIcuInput) -> CamIcuOutcome {
    let delirium_present = input.acute_or_fluctuating_course
        && input.inattention
        && (input.disorganized_thinking || input.altered_consciousness);
    CamIcuOutcome { delirium_present }
}

pub fn build_response(input: &CamIcuInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input);
    let mut working = Map::new();
    working.insert(
        "feature_1_positive".into(),
        json!(input.acute_or_fluctuating_course),
    );
    working.insert("feature_2_positive".into(), json!(input.inattention));
    working.insert(
        "feature_3_positive".into(),
        json!(input.disorganized_thinking),
    );
    working.insert(
        "feature_4_positive".into(),
        json!(input.altered_consciousness),
    );

    let interpretation = if o.delirium_present {
        "CAM-ICU positive: delirium present (Feature 1 and Feature 2 and at least one of Feature 3/4). This confirms the clinical picture of delirium; it does not identify the cause - review and treat reversible contributors (e.g. pain, hypoxia, sepsis, drug withdrawal, metabolic derangement, medications) alongside non-pharmacological delirium-prevention measures.".to_string()
    } else {
        "CAM-ICU negative: delirium not detected on this assessment. CAM-ICU should be reassessed regularly (e.g. each shift), as delirium can fluctuate; a negative assessment does not exclude delirium at another time, and this tool is not assessable at RASS -4/-5 (unarousable/deep sedation).".to_string()
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(if o.delirium_present {
            "positive"
        } else {
            "negative"
        }),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct CamIcu;
impl Calculator for CamIcu {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "CAM-ICU"
    }
    fn description(&self) -> &'static str {
        "Four-feature Confusion Assessment Method for the ICU: delirium is present when Feature 1 and Feature 2 and (Feature 3 or Feature 4) are all positive."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "CamIcuInput", "type": "object", "additionalProperties": false,
            "required": ["acute_or_fluctuating_course", "inattention", "disorganized_thinking", "altered_consciousness"],
            "properties": {
                "acute_or_fluctuating_course": { "type": "boolean", "description": "Feature 1: acute change or fluctuating course of mental status from baseline over the preceding 24 hours" },
                "inattention": { "type": "boolean", "description": "Feature 2: inattention, e.g. errors on a letter-vigilance or picture test" },
                "disorganized_thinking": { "type": "boolean", "description": "Feature 3: disorganised thinking, e.g. errors on yes/no questions or a simple command" },
                "altered_consciousness": { "type": "boolean", "description": "Feature 4: altered level of consciousness (RASS other than 0). Not assessable at RASS -4 or -5." }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: CamIcuInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_four_positive_is_delirium() {
        let o = compute(&CamIcuInput {
            acute_or_fluctuating_course: true,
            inattention: true,
            disorganized_thinking: true,
            altered_consciousness: true,
        });
        assert!(o.delirium_present);
    }

    #[test]
    fn feature_1_2_and_only_3_is_still_delirium() {
        let o = compute(&CamIcuInput {
            acute_or_fluctuating_course: true,
            inattention: true,
            disorganized_thinking: true,
            altered_consciousness: false,
        });
        assert!(o.delirium_present);
    }

    #[test]
    fn feature_1_2_and_only_4_is_still_delirium() {
        let o = compute(&CamIcuInput {
            acute_or_fluctuating_course: true,
            inattention: true,
            disorganized_thinking: false,
            altered_consciousness: true,
        });
        assert!(o.delirium_present);
    }

    #[test]
    fn missing_feature_1_is_negative() {
        let o = compute(&CamIcuInput {
            acute_or_fluctuating_course: false,
            inattention: true,
            disorganized_thinking: true,
            altered_consciousness: true,
        });
        assert!(!o.delirium_present);
    }

    #[test]
    fn missing_feature_2_is_negative() {
        let o = compute(&CamIcuInput {
            acute_or_fluctuating_course: true,
            inattention: false,
            disorganized_thinking: true,
            altered_consciousness: true,
        });
        assert!(!o.delirium_present);
    }

    #[test]
    fn missing_both_3_and_4_is_negative() {
        let o = compute(&CamIcuInput {
            acute_or_fluctuating_course: true,
            inattention: true,
            disorganized_thinking: false,
            altered_consciousness: false,
        });
        assert!(!o.delirium_present);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "acute_or_fluctuating_course": true, "inattention": true,
            "disorganized_thinking": true, "altered_consciousness": false
        });
        let typed = CamIcuInput {
            acute_or_fluctuating_course: true,
            inattention: true,
            disorganized_thinking: true,
            altered_consciousness: false,
        };
        let dynamic = CamIcu.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            CamIcu
                .calculate(&json!({"acute_or_fluctuating_course": true}))
                .is_err()
        );
    }
}
