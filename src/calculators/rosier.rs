// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! ROSIER Scale - Recognition of Stroke in the Emergency Room (Nor et al.
//! Lancet Neurol 2005).
//!
//! Seven items: two clinical-history items that reduce the probability of
//! stroke (loss of consciousness/syncope, seizure activity) and five new,
//! acute-onset neurological signs that increase it. Total score ranges from
//! -2 to +5; a score above 0 indicates stroke is likely. Hypoglycaemia
//! should be excluded/corrected before applying the scale, per the
//! instrument's implementation guidance, but that glucose check is not part
//! of the seven scored items in the primary publication and so is not a
//! scored input here.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "rosier";

pub const REFERENCE: &str = "Nor AM, Davis J, Sen B, et al. The Recognition of Stroke in the Emergency Room \
(ROSIER) scale: development and validation of a stroke recognition instrument. Lancet Neurol. \
2005;4(11):727-734.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.1016/S1474-4422(05)70201-5",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RosierInput {
    pub loss_of_consciousness_or_syncope: bool,
    pub seizure_activity: bool,
    /// New, acute onset (or on awakening from sleep).
    pub new_asymmetric_facial_weakness: bool,
    pub new_asymmetric_arm_weakness: bool,
    pub new_asymmetric_leg_weakness: bool,
    pub new_speech_disturbance: bool,
    pub new_visual_field_defect: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RosierOutcome {
    pub score: i8,
    pub stroke_likely: bool,
}

pub fn compute(input: &RosierInput) -> RosierOutcome {
    let score = -i8::from(input.loss_of_consciousness_or_syncope)
        - i8::from(input.seizure_activity)
        + i8::from(input.new_asymmetric_facial_weakness)
        + i8::from(input.new_asymmetric_arm_weakness)
        + i8::from(input.new_asymmetric_leg_weakness)
        + i8::from(input.new_speech_disturbance)
        + i8::from(input.new_visual_field_defect);
    RosierOutcome {
        score,
        stroke_likely: score > 0,
    }
}

pub fn build_response(input: &RosierInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input);
    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("stroke_likely".into(), json!(o.stroke_likely));

    let interpretation = if o.stroke_likely {
        format!(
            "ROSIER score {} (range -2 to +5, >0 = stroke likely): stroke is possible. Urgent brain imaging and stroke-team assessment are recommended. Check and correct hypoglycaemia before relying on this assessment, since hypoglycaemia is a stroke mimic.",
            o.score
        )
    } else {
        format!(
            "ROSIER score {} (range -2 to +5, >0 = stroke likely): stroke is unlikely but not completely excluded. Consider stroke mimics (seizure, syncope, hypoglycaemia, sepsis) and continue clinical judgement if suspicion remains.",
            o.score
        )
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Rosier;
impl Calculator for Rosier {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "ROSIER Scale (Stroke Recognition)"
    }
    fn description(&self) -> &'static str {
        "Seven-item emergency-room stroke-recognition instrument distinguishing acute stroke from stroke mimics."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "RosierInput", "type": "object",
            "additionalProperties": false,
            "required": [
                "loss_of_consciousness_or_syncope", "seizure_activity", "new_asymmetric_facial_weakness",
                "new_asymmetric_arm_weakness", "new_asymmetric_leg_weakness", "new_speech_disturbance",
                "new_visual_field_defect"
            ],
            "properties": {
                "loss_of_consciousness_or_syncope": { "type": "boolean", "description": "Loss of consciousness or syncope at onset (-1 point)" },
                "seizure_activity": { "type": "boolean", "description": "Seizure activity at onset (-1 point)" },
                "new_asymmetric_facial_weakness": { "type": "boolean", "description": "New, acute-onset (or on awakening) asymmetric facial weakness (+1 point)" },
                "new_asymmetric_arm_weakness": { "type": "boolean", "description": "New, acute-onset (or on awakening) asymmetric arm weakness (+1 point)" },
                "new_asymmetric_leg_weakness": { "type": "boolean", "description": "New, acute-onset (or on awakening) asymmetric leg weakness (+1 point)" },
                "new_speech_disturbance": { "type": "boolean", "description": "New, acute-onset (or on awakening) speech disturbance (+1 point)" },
                "new_visual_field_defect": { "type": "boolean", "description": "New, acute-onset (or on awakening) visual field defect (+1 point)" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: RosierInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> RosierInput {
        RosierInput {
            loss_of_consciousness_or_syncope: false,
            seizure_activity: false,
            new_asymmetric_facial_weakness: false,
            new_asymmetric_arm_weakness: false,
            new_asymmetric_leg_weakness: false,
            new_speech_disturbance: false,
            new_visual_field_defect: false,
        }
    }

    #[test]
    fn all_absent_scores_zero_and_not_likely() {
        let o = compute(&none());
        assert_eq!(o.score, 0);
        assert!(!o.stroke_likely);
    }

    #[test]
    fn minimum_score_is_minus_two() {
        let mut i = none();
        i.loss_of_consciousness_or_syncope = true;
        i.seizure_activity = true;
        let o = compute(&i);
        assert_eq!(o.score, -2);
        assert!(!o.stroke_likely);
    }

    #[test]
    fn maximum_score_is_five() {
        let i = RosierInput {
            loss_of_consciousness_or_syncope: false,
            seizure_activity: false,
            new_asymmetric_facial_weakness: true,
            new_asymmetric_arm_weakness: true,
            new_asymmetric_leg_weakness: true,
            new_speech_disturbance: true,
            new_visual_field_defect: true,
        };
        let o = compute(&i);
        assert_eq!(o.score, 5);
        assert!(o.stroke_likely);
    }

    #[test]
    fn single_positive_sign_crosses_the_zero_threshold() {
        let mut i = none();
        i.new_asymmetric_arm_weakness = true;
        let o = compute(&i);
        assert_eq!(o.score, 1);
        assert!(o.stroke_likely);
    }

    #[test]
    fn score_of_zero_is_not_likely_stroke() {
        let mut i = none();
        i.loss_of_consciousness_or_syncope = true;
        i.new_asymmetric_arm_weakness = true;
        let o = compute(&i);
        assert_eq!(o.score, 0);
        assert!(!o.stroke_likely);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let mut typed = none();
        typed.new_speech_disturbance = true;
        typed.new_visual_field_defect = true;
        let value = json!({
            "loss_of_consciousness_or_syncope": false, "seizure_activity": false,
            "new_asymmetric_facial_weakness": false, "new_asymmetric_arm_weakness": false,
            "new_asymmetric_leg_weakness": false, "new_speech_disturbance": true, "new_visual_field_defect": true
        });
        let dynamic = Rosier.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(2));
    }

    #[test]
    fn rejects_missing_field() {
        assert!(
            Rosier
                .calculate(&json!({"seizure_activity": true}))
                .is_err()
        );
    }
}
