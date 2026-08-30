// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! NYHA (New York Heart Association) Functional Classification.
//!
//! A four-class ordinal classification of symptom-limited functional capacity
//! in patients with known heart disease, from class I (no limitation) to
//! class IV (symptomatic at rest). It describes functional capacity, not
//! underlying structural disease, and is distinct from the ACC/AHA structural
//! stages A-D that heart-failure guidelines use alongside it.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "nyha";

/// Primary citation.
pub const REFERENCE: &str = "The Criteria Committee of the New York Heart Association. Nomenclature and \
Criteria for Diagnosis of Diseases of the Heart and Great Vessels. 9th ed. Boston, Mass: Little, \
Brown & Co; 1994:253-256.";

/// Distribution licence: the classification is reproduced from the American
/// Heart Association's public patient-education summary of the New York
/// Heart Association criteria.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public professional/patient classification - reproduced from the American Heart \
Association's summary of the New York Heart Association criteria",
    source_url: "https://www.heart.org/en/health-topics/heart-failure/what-is-heart-failure/classes-of-heart-failure",
};

/// Lowest valid NYHA class.
pub const MIN_CLASS: u8 = 1;

/// Highest valid NYHA class.
pub const MAX_CLASS: u8 = 4;

/// Input: a single NYHA functional class (1-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NyhaInput {
    /// The NYHA functional class, an integer from 1 (least limited) to 4 (most limited).
    pub class: u8,
}

/// The roman-numeral label for a given class.
fn numeral(class: u8) -> &'static str {
    match class {
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        _ => unreachable!("class is validated to 1-4 before this is called"),
    }
}

/// The descriptor for a given class.
fn descriptor(class: u8) -> &'static str {
    match class {
        1 => {
            "No limitation of physical activity. Ordinary physical activity does not cause undue \
fatigue, palpitation, or dyspnoea (shortness of breath)."
        }
        2 => {
            "Slight limitation of physical activity. Comfortable at rest, but ordinary physical \
activity results in fatigue, palpitation, or dyspnoea."
        }
        3 => {
            "Marked limitation of physical activity. Comfortable at rest, but less than ordinary \
physical activity causes fatigue, palpitation, or dyspnoea."
        }
        4 => {
            "Unable to carry out any physical activity without discomfort. Symptoms of cardiac \
insufficiency are present at rest. If any physical activity is undertaken, discomfort increases."
        }
        _ => unreachable!("class is validated to 1-4 before this is called"),
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct NyhaOutcome {
    /// The NYHA class (echoed back, 1-4).
    pub class: u8,
    /// The class's roman-numeral label, e.g. "II".
    pub label: &'static str,
    /// The class's functional-capacity descriptor.
    pub descriptor: &'static str,
    /// Clinical interpretation.
    pub interpretation: String,
}

/// Pure scoring: validate the class and attach its label and descriptor.
pub fn compute(input: &NyhaInput) -> Result<NyhaOutcome, CalcError> {
    if input.class < MIN_CLASS || input.class > MAX_CLASS {
        return Err(CalcError::InvalidInput(format!(
            "class must be an integer from {MIN_CLASS} to {MAX_CLASS}, got {}",
            input.class
        )));
    }

    let label = numeral(input.class);
    let descriptor = descriptor(input.class);
    let interpretation = format!(
        "NYHA class {label}: {descriptor} NYHA class describes symptom-limited functional \
capacity in a patient with known heart disease; it is not itself a diagnosis of heart failure and \
commonly changes with treatment. It is distinct from, and used alongside, the ACC/AHA structural \
stages A-D."
    );

    Ok(NyhaOutcome {
        class: input.class,
        label,
        descriptor,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &NyhaInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("class".into(), json!(o.class));
    working.insert("label".into(), json!(o.label));
    working.insert("descriptor".into(), json!(o.descriptor));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(format!("NYHA {}", o.label)),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Nyha;

impl Calculator for Nyha {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "NYHA Functional Classification"
    }

    fn description(&self) -> &'static str {
        "Classifies symptom-limited functional capacity in known heart disease on the NYHA I-IV scale."
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
            "title": "NyhaInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["class"],
            "properties": {
                "class": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 4,
                    "description": "NYHA functional class (1-4). 1 = no limitation of physical activity; 2 = slight limitation, comfortable at rest; 3 = marked limitation, comfortable at rest; 4 = unable to carry out any physical activity without discomfort, symptomatic at rest.",
                    "definition": {
                        "concept": "NYHA functional class",
                        "statement": "A single ordinal class (1-4) describing how much physical activity a patient with known heart disease can tolerate before symptoms of fatigue, palpitation, or dyspnoea occur, from no limitation (1) to symptomatic at rest (4).",
                        "caveats": "This classifies functional capacity, not structural disease; assign it in a patient with an established diagnosis of heart disease. It is subjective, clinician-assessed, and often changes with treatment - it is not the ACC/AHA structural stage (A-D), which is assessed separately.",
                        "source": {
                            "citation": "The Criteria Committee of the New York Heart Association. 9th ed. Boston, Mass: Little, Brown & Co; 1994:253-256.",
                            "url": "https://www.heart.org/en/health-topics/heart-failure/what-is-heart-failure/classes-of-heart-failure"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: NyhaInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(class: u8) -> NyhaInput {
        NyhaInput { class }
    }

    #[test]
    fn class_1_interpretation() {
        let o = compute(&input(1)).unwrap();
        assert_eq!(o.class, 1);
        assert_eq!(o.label, "I");
        assert!(o.descriptor.contains("No limitation of physical activity"));
    }

    #[test]
    fn class_2_interpretation() {
        let o = compute(&input(2)).unwrap();
        assert_eq!(o.label, "II");
        assert!(o.descriptor.contains("Slight limitation"));
    }

    #[test]
    fn class_3_interpretation() {
        let o = compute(&input(3)).unwrap();
        assert_eq!(o.label, "III");
        assert!(o.descriptor.contains("Marked limitation"));
    }

    #[test]
    fn class_4_interpretation() {
        let o = compute(&input(4)).unwrap();
        assert_eq!(o.label, "IV");
        assert!(
            o.descriptor
                .contains("Unable to carry out any physical activity")
        );
        assert!(o.descriptor.contains("at rest"));
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(compute(&input(0)).is_err());
        assert!(compute(&input(5)).is_err());
        assert!(compute(&input(255)).is_err());
    }

    #[test]
    fn result_is_nyha_prefixed_roman_numeral() {
        let r = build_response(&input(3)).unwrap();
        assert_eq!(r.result, json!("NYHA III"));
        assert_eq!(r.calculator, NAME);
        assert_eq!(r.working["class"], json!(3));
        assert_eq!(r.working["label"], json!("III"));
    }

    #[test]
    fn interpretation_distinguishes_from_acc_aha_stage() {
        let o = compute(&input(2)).unwrap();
        assert!(o.interpretation.contains("ACC/AHA structural"));
        assert!(
            o.interpretation
                .contains("not itself a diagnosis of heart failure")
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "class": 4 });
        let dynamic = Nyha.calculate(&value).unwrap();
        let typed = build_response(&input(4)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn dynamic_rejects_out_of_range_and_unknown_fields() {
        assert!(Nyha.calculate(&json!({ "class": 0 })).is_err());
        assert!(Nyha.calculate(&json!({ "class": 9 })).is_err());
        assert!(
            Nyha.calculate(&json!({ "class": 2, "stage": "C" }))
                .is_err()
        );
    }

    #[test]
    fn schema_constrains_class_and_documents_caveats() {
        let schema = Nyha.input_schema();
        let class = &schema["properties"]["class"];
        assert_eq!(class["minimum"], json!(1));
        assert_eq!(class["maximum"], json!(4));
        let def = &class["definition"];
        assert!(def["caveats"].as_str().unwrap().contains("ACC/AHA"));
        assert!(
            def["caveats"]
                .as_str()
                .unwrap()
                .contains("established diagnosis")
        );
    }

    #[test]
    fn license_has_evidence_url() {
        assert!(LICENSE.source_url.starts_with("https://"));
        assert!(!LICENSE.license.is_empty());
    }
}
