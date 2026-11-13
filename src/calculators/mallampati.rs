// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Mallampati Classification - a four-class ordinal oropharyngeal-view
//! classification predicting difficult laryngoscopy.
//!
//! Mallampati's original 1985 description had three classes; Samsoon and
//! Young (1987) added the fourth class ("only the hard palate visible") used
//! throughout modern practice, so this implements the four-class "modified
//! Mallampati" form.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "mallampati";

pub const REFERENCE: &str = "Mallampati SR, Gatt SP, Gugino LD, et al. A clinical sign to predict difficult \
tracheal intubation: a prospective study. Can Anaesth Soc J. 1985;32(4):429-434. Class IV added by \
Samsoon GL, Young JR. Difficult tracheal intubation: a retrospective study. Anaesthesia. \
1987;42(5):487-490.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.1007/BF03011357",
);

pub const MIN_CLASS: u8 = 1;
pub const MAX_CLASS: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MallampatiInput {
    /// The Mallampati class, an integer from 1 to 4.
    pub class: u8,
}

fn descriptor(class: u8) -> &'static str {
    match class {
        1 => "Soft palate, uvula, fauces, and pillars fully visible",
        2 => "Soft palate, major part of uvula, and fauces visible",
        3 => "Soft palate and base of uvula visible",
        4 => "Only the hard palate visible",
        _ => unreachable!("class is validated to 1-4 before this is called"),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MallampatiOutcome {
    pub class: u8,
    pub descriptor: &'static str,
    pub predicts_difficult_laryngoscopy: bool,
    pub interpretation: String,
}

pub fn compute(input: &MallampatiInput) -> Result<MallampatiOutcome, CalcError> {
    if !(MIN_CLASS..=MAX_CLASS).contains(&input.class) {
        return Err(CalcError::InvalidInput(format!(
            "class must be an integer from {MIN_CLASS} to {MAX_CLASS}, got {}",
            input.class
        )));
    }
    let descriptor = descriptor(input.class);
    let predicts_difficult_laryngoscopy = input.class >= 3;
    let interpretation = format!(
        "Mallampati class {} of 4: {}. Classes III and IV are associated with a higher incidence of difficult laryngoscopy and intubation; this is a single bedside sign with limited standalone sensitivity and should be combined with other airway-assessment findings rather than used alone.",
        input.class, descriptor
    );
    Ok(MallampatiOutcome {
        class: input.class,
        descriptor,
        predicts_difficult_laryngoscopy,
        interpretation,
    })
}

pub fn build_response(input: &MallampatiInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("class".into(), json!(o.class));
    working.insert("descriptor".into(), json!(o.descriptor));
    working.insert(
        "predicts_difficult_laryngoscopy".into(),
        json!(o.predicts_difficult_laryngoscopy),
    );
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.class),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Mallampati;
impl Calculator for Mallampati {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Mallampati Classification"
    }
    fn description(&self) -> &'static str {
        "Four-class ordinal oropharyngeal-view classification predicting difficult laryngoscopy."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "MallampatiInput", "type": "object",
            "additionalProperties": false, "required": ["class"],
            "properties": {
                "class": {
                    "type": "integer", "minimum": 1, "maximum": 4,
                    "description": "Mallampati class (1-4), assessed with the patient seated, mouth open maximally, tongue protruded without phonation. 1 = soft palate, uvula, fauces, and pillars fully visible; 2 = soft palate, major part of uvula, and fauces visible; 3 = soft palate and base of uvula visible; 4 = only the hard palate visible."
                }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: MallampatiInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(class: u8) -> MallampatiInput {
        MallampatiInput { class }
    }

    #[test]
    fn class_1_description() {
        let o = compute(&input(1)).unwrap();
        assert!(o.descriptor.contains("fully visible"));
        assert!(!o.predicts_difficult_laryngoscopy);
    }

    #[test]
    fn class_3_and_4_predict_difficulty() {
        assert!(compute(&input(3)).unwrap().predicts_difficult_laryngoscopy);
        assert!(compute(&input(4)).unwrap().predicts_difficult_laryngoscopy);
    }

    #[test]
    fn class_2_does_not_predict_difficulty() {
        assert!(!compute(&input(2)).unwrap().predicts_difficult_laryngoscopy);
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(compute(&input(0)).is_err());
        assert!(compute(&input(5)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "class": 3 });
        let dynamic = Mallampati.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&input(3)).unwrap());
        assert_eq!(dynamic.result, json!(3));
    }

    #[test]
    fn dynamic_rejects_out_of_range() {
        assert!(Mallampati.calculate(&json!({ "class": 0 })).is_err());
    }
}
