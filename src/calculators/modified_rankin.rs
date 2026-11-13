// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Modified Rankin Scale - a seven-point (0-6) global disability scale after
//! stroke.
//!
//! Developed from Rankin's original 5-grade scale (1957) and modified to its
//! current 0-5 form by van Swieten et al. (1988); grade 6 (death) is a
//! near-universal addition used in stroke trials and is included here as
//! the standard modern 0-6 form.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "modified_rankin";

pub const REFERENCE: &str = "Rankin J. Cerebral vascular accidents in patients over the age of 60. Scott Med J. \
1957;2(5):200-215 (original 5-level scale); van Swieten JC, Koudstaal PJ, Visser MC, Schouten HJ, \
van Gijn J. Interobserver agreement for the assessment of handicap in stroke patients. Stroke. \
1988;19(5):604-607 (current 0-6 modified version).";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.1161/01.STR.19.5.604",
);

pub const MIN_GRADE: u8 = 0;
pub const MAX_GRADE: u8 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModifiedRankinInput {
    /// The modified Rankin grade, an integer from 0 (no symptoms) to 6 (dead).
    pub grade: u8,
}

fn descriptor(grade: u8) -> &'static str {
    match grade {
        0 => "No symptoms at all",
        1 => {
            "No significant disability despite symptoms; able to carry out all usual duties and activities"
        }
        2 => {
            "Slight disability; unable to carry out all previous activities, but able to look after own affairs without assistance"
        }
        3 => "Moderate disability; requiring some help, but able to walk without assistance",
        4 => {
            "Moderately severe disability; unable to walk without assistance and unable to attend to own bodily needs without assistance"
        }
        5 => {
            "Severe disability; bedridden, incontinent, and requiring constant nursing care and attention"
        }
        6 => "Dead",
        _ => unreachable!("grade is validated to 0-6 before this is called"),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModifiedRankinOutcome {
    pub grade: u8,
    pub descriptor: &'static str,
    pub interpretation: String,
}

pub fn compute(input: &ModifiedRankinInput) -> Result<ModifiedRankinOutcome, CalcError> {
    if input.grade > MAX_GRADE {
        return Err(CalcError::InvalidInput(format!(
            "grade must be an integer from {MIN_GRADE} to {MAX_GRADE}, got {}",
            input.grade
        )));
    }
    let descriptor = descriptor(input.grade);
    let interpretation = format!(
        "Modified Rankin Scale grade {} of 6: {}. A grade of 0-2 is conventionally treated as a good functional outcome and 3-6 as poor in stroke trials, though some analyses treat the scale ordinally rather than dichotomising it.",
        input.grade, descriptor
    );
    Ok(ModifiedRankinOutcome {
        grade: input.grade,
        descriptor,
        interpretation,
    })
}

pub fn build_response(input: &ModifiedRankinInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("grade".into(), json!(o.grade));
    working.insert("descriptor".into(), json!(o.descriptor));
    working.insert("good_functional_outcome".into(), json!(o.grade <= 2));
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.grade),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct ModifiedRankin;
impl Calculator for ModifiedRankin {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Modified Rankin Scale"
    }
    fn description(&self) -> &'static str {
        "Seven-point (0-6) global disability scale used to grade functional outcome after stroke."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "ModifiedRankinInput", "type": "object",
            "additionalProperties": false, "required": ["grade"],
            "properties": {
                "grade": {
                    "type": "integer", "minimum": 0, "maximum": 6,
                    "description": "Modified Rankin Scale grade (0-6). 0 = no symptoms; 1 = no significant disability despite symptoms; 2 = slight disability, independent; 3 = moderate disability, needs some help but walks unassisted; 4 = moderately severe disability, cannot walk or attend to bodily needs unassisted; 5 = severe disability, bedridden and needs constant nursing care; 6 = dead."
                }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: ModifiedRankinInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(grade: u8) -> ModifiedRankinInput {
        ModifiedRankinInput { grade }
    }

    #[test]
    fn grade_0_is_no_symptoms() {
        let o = compute(&input(0)).unwrap();
        assert_eq!(o.descriptor, "No symptoms at all");
    }

    #[test]
    fn grade_6_is_dead() {
        let o = compute(&input(6)).unwrap();
        assert_eq!(o.descriptor, "Dead");
    }

    #[test]
    fn good_outcome_threshold_is_two_or_below() {
        let r2 = build_response(&input(2)).unwrap();
        assert_eq!(r2.working["good_functional_outcome"], json!(true));
        let r3 = build_response(&input(3)).unwrap();
        assert_eq!(r3.working["good_functional_outcome"], json!(false));
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(compute(&input(7)).is_err());
        assert!(compute(&input(255)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "grade": 4 });
        let dynamic = ModifiedRankin.calculate(&value).unwrap();
        let typed = build_response(&input(4)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn dynamic_rejects_out_of_range() {
        assert!(ModifiedRankin.calculate(&json!({ "grade": 9 })).is_err());
    }
}
