// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! GCS - Glasgow Coma Scale.
//!
//! A three-component bedside score of conscious level (Teasdale & Jennett,
//! Lancet 1974): eye opening (E, 1-4), verbal response (V, 1-5), and motor
//! response (M, 1-6). The total (E + V + M) ranges 3-15 and is the universal
//! trauma, sedation, and neuro-observation score.
//!
//! One clinical subtlety: the verbal component cannot be scored in a patient
//! who is intubated, has a tracheostomy, or has a condition preventing speech
//! (e.g. dysphasia). Convention records this as "1T" (untestable) rather than
//! omitting the component, because the motor response remains the single best
//! predictor of outcome and the total must stay comparable across
//! observations. This calculator scores the three components as given; when
//! verbal response is untestable, record it as 1 and note the reason
//! separately, per standard practice - the calculator does not encode a
//! separate "T" flag.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "gcs";

/// Primary citation.
pub const REFERENCE: &str = "Teasdale G, Jennett B. Assessment of coma and impaired consciousness. \
A practical scale. Lancet. 1974;2(7872):81-84. doi:10.1016/s0140-6736(74)91639-0";

/// Distribution licence: the Glasgow Coma Scale is a published clinical
/// method, free to use in clinical practice, teaching, and research;
/// implemented here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Free to use in clinical practice, teaching, and research - implemented from the primary literature",
    source_url: "https://www.glasgowcomascale.org/",
};

/// Lowest valid eye-opening score.
pub const EYE_MIN: u8 = 1;
/// Highest valid eye-opening score.
pub const EYE_MAX: u8 = 4;
/// Lowest valid verbal-response score.
pub const VERBAL_MIN: u8 = 1;
/// Highest valid verbal-response score.
pub const VERBAL_MAX: u8 = 5;
/// Lowest valid motor-response score.
pub const MOTOR_MIN: u8 = 1;
/// Highest valid motor-response score.
pub const MOTOR_MAX: u8 = 6;

/// GCS inputs: the three component scores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcsInput {
    /// Eye-opening response, 1 (none) to 4 (spontaneous).
    pub eye_opening: u8,
    /// Verbal response, 1 (none) to 5 (oriented).
    pub verbal_response: u8,
    /// Motor response, 1 (none) to 6 (obeys commands).
    pub motor_response: u8,
}

/// Descriptor for a given eye-opening score.
fn eye_descriptor(score: u8) -> &'static str {
    match score {
        4 => "Spontaneous eye opening",
        3 => "Eye opening to speech (verbal command)",
        2 => "Eye opening to pain",
        1 => "No eye opening",
        _ => unreachable!("eye_opening is validated to 1-4 before this is called"),
    }
}

/// Descriptor for a given verbal-response score.
fn verbal_descriptor(score: u8) -> &'static str {
    match score {
        5 => "Oriented",
        4 => "Confused conversation",
        3 => "Inappropriate words",
        2 => "Incomprehensible sounds",
        1 => "No verbal response",
        _ => unreachable!("verbal_response is validated to 1-5 before this is called"),
    }
}

/// Descriptor for a given motor-response score.
fn motor_descriptor(score: u8) -> &'static str {
    match score {
        6 => "Obeys commands",
        5 => "Localises to pain",
        4 => "Normal flexion (withdrawal) to pain",
        3 => "Abnormal flexion to pain (decorticate posturing)",
        2 => "Extension to pain (decerebrate posturing)",
        1 => "No motor response",
        _ => unreachable!("motor_response is validated to 1-6 before this is called"),
    }
}

/// Severity band implied by the total score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// Total 3-8: severe brain injury.
    Severe,
    /// Total 9-12: moderate brain injury.
    Moderate,
    /// Total 13-15: mild brain injury.
    Mild,
}

impl Band {
    /// Stable slug.
    pub fn slug(self) -> &'static str {
        match self {
            Band::Severe => "severe",
            Band::Moderate => "moderate",
            Band::Mild => "mild",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcsOutcome {
    /// Total score (3-15).
    pub total_score: u8,
    pub band: Band,
    pub eye_opening: u8,
    pub verbal_response: u8,
    pub motor_response: u8,
    pub interpretation: String,
}

/// Pure scoring: validate each component and sum.
pub fn compute(input: &GcsInput) -> Result<GcsOutcome, CalcError> {
    if input.eye_opening < EYE_MIN || input.eye_opening > EYE_MAX {
        return Err(CalcError::InvalidInput(format!(
            "eye_opening must be an integer from {EYE_MIN} to {EYE_MAX}, got {}",
            input.eye_opening
        )));
    }
    if input.verbal_response < VERBAL_MIN || input.verbal_response > VERBAL_MAX {
        return Err(CalcError::InvalidInput(format!(
            "verbal_response must be an integer from {VERBAL_MIN} to {VERBAL_MAX}, got {}",
            input.verbal_response
        )));
    }
    if input.motor_response < MOTOR_MIN || input.motor_response > MOTOR_MAX {
        return Err(CalcError::InvalidInput(format!(
            "motor_response must be an integer from {MOTOR_MIN} to {MOTOR_MAX}, got {}",
            input.motor_response
        )));
    }

    let total_score = input.eye_opening + input.verbal_response + input.motor_response;

    let band = match total_score {
        3..=8 => Band::Severe,
        9..=12 => Band::Moderate,
        _ => Band::Mild,
    };

    let interpretation = match band {
        Band::Severe => format!(
            "GCS {total_score}/15 (E{} V{} M{}): severe brain injury (GCS <= 8). This threshold is \
widely used to trigger airway protection (intubation) and urgent senior/neurosurgical review.",
            input.eye_opening, input.verbal_response, input.motor_response
        ),
        Band::Moderate => format!(
            "GCS {total_score}/15 (E{} V{} M{}): moderate brain injury (GCS 9-12). Warrants close \
observation and a low threshold for escalation if the score falls.",
            input.eye_opening, input.verbal_response, input.motor_response
        ),
        Band::Mild => format!(
            "GCS {total_score}/15 (E{} V{} M{}): mild brain injury or normal conscious level (GCS \
13-15). A falling GCS, or any drop of 2 or more points, should prompt urgent reassessment regardless \
of the absolute score.",
            input.eye_opening, input.verbal_response, input.motor_response
        ),
    };

    Ok(GcsOutcome {
        total_score,
        band,
        eye_opening: input.eye_opening,
        verbal_response: input.verbal_response,
        motor_response: input.motor_response,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &GcsInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.total_score));
    working.insert("level".into(), json!(o.band.slug()));
    working.insert("eye_opening".into(), json!(o.eye_opening));
    working.insert(
        "eye_opening_descriptor".into(),
        json!(eye_descriptor(o.eye_opening)),
    );
    working.insert("verbal_response".into(), json!(o.verbal_response));
    working.insert(
        "verbal_response_descriptor".into(),
        json!(verbal_descriptor(o.verbal_response)),
    );
    working.insert("motor_response".into(), json!(o.motor_response));
    working.insert(
        "motor_response_descriptor".into(),
        json!(motor_descriptor(o.motor_response)),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.total_score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Gcs;

impl Calculator for Gcs {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Glasgow Coma Scale (GCS)"
    }

    fn description(&self) -> &'static str {
        "Universal bedside score (3-15) of conscious level from eye, verbal, and motor response \
(Teasdale & Jennett 1974); trauma, sedation, and neuro-observation."
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
            "title": "GcsInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["eye_opening", "verbal_response", "motor_response"],
            "properties": {
                "eye_opening": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 4,
                    "description": "Eye-opening response (E). 4 = spontaneous; 3 = to speech; 2 = to pain; 1 = none.",
                    "definition": {
                        "concept": "GCS eye-opening component (E)",
                        "statement": "An ordinal 1-4 score of eye opening: spontaneous (4), to speech (3), to pain (2), or none (1).",
                        "source": { "citation": "Teasdale G, Jennett B. Lancet. 1974;2(7872):81-84.", "url": "https://www.glasgowcomascale.org/" },
                        "status": "draft"
                    }
                },
                "verbal_response": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 5,
                    "description": "Verbal response (V). 5 = oriented; 4 = confused conversation; 3 = inappropriate words; 2 = incomprehensible sounds; 1 = none.",
                    "definition": {
                        "concept": "GCS verbal-response component (V)",
                        "statement": "An ordinal 1-5 score of verbal response: oriented (5), confused conversation (4), inappropriate words (3), incomprehensible sounds (2), or none (1).",
                        "caveats": "Cannot be assessed in an intubated, tracheostomised, or dysphasic patient. Convention records this as \"1T\" (untestable); pass 1 here and note the reason for untestability alongside the result.",
                        "source": { "citation": "Teasdale G, Jennett B. Lancet. 1974;2(7872):81-84.", "url": "https://www.glasgowcomascale.org/" },
                        "status": "draft"
                    }
                },
                "motor_response": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 6,
                    "description": "Motor response (M). 6 = obeys commands; 5 = localises to pain; 4 = normal flexion/withdrawal; 3 = abnormal flexion (decorticate); 2 = extension (decerebrate); 1 = none.",
                    "definition": {
                        "concept": "GCS motor-response component (M)",
                        "statement": "An ordinal 1-6 score of motor response: obeys commands (6), localises to pain (5), normal flexion/withdrawal (4), abnormal flexion/decorticate posturing (3), extension/decerebrate posturing (2), or none (1).",
                        "caveats": "Score the best response observed in any limb.",
                        "source": { "citation": "Teasdale G, Jennett B. Lancet. 1974;2(7872):81-84.", "url": "https://www.glasgowcomascale.org/" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: GcsInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(eye: u8, verbal: u8, motor: u8) -> GcsInput {
        GcsInput {
            eye_opening: eye,
            verbal_response: verbal,
            motor_response: motor,
        }
    }

    #[test]
    fn fully_alert_scores_fifteen() {
        let o = compute(&input(4, 5, 6)).unwrap();
        assert_eq!(o.total_score, 15);
        assert_eq!(o.band, Band::Mild);
    }

    #[test]
    fn unresponsive_scores_three() {
        let o = compute(&input(1, 1, 1)).unwrap();
        assert_eq!(o.total_score, 3);
        assert_eq!(o.band, Band::Severe);
    }

    #[test]
    fn severe_band_upper_boundary_is_inclusive() {
        // E1 V2 M5 = 8: still severe.
        let o = compute(&input(1, 2, 5)).unwrap();
        assert_eq!(o.total_score, 8);
        assert_eq!(o.band, Band::Severe);
    }

    #[test]
    fn moderate_band_boundaries() {
        // E2 V2 M5 = 9: lowest moderate.
        let o = compute(&input(2, 2, 5)).unwrap();
        assert_eq!(o.total_score, 9);
        assert_eq!(o.band, Band::Moderate);

        // E4 V3 M5 = 12: highest moderate.
        let o = compute(&input(4, 3, 5)).unwrap();
        assert_eq!(o.total_score, 12);
        assert_eq!(o.band, Band::Moderate);
    }

    #[test]
    fn mild_band_lower_boundary_is_inclusive() {
        // E4 V4 M5 = 13: lowest mild.
        let o = compute(&input(4, 4, 5)).unwrap();
        assert_eq!(o.total_score, 13);
        assert_eq!(o.band, Band::Mild);
    }

    #[test]
    fn rejects_out_of_range_components() {
        assert!(compute(&input(0, 5, 6)).is_err());
        assert!(compute(&input(5, 5, 6)).is_err());
        assert!(compute(&input(4, 0, 6)).is_err());
        assert!(compute(&input(4, 6, 6)).is_err());
        assert!(compute(&input(4, 5, 0)).is_err());
        assert!(compute(&input(4, 5, 7)).is_err());
    }

    #[test]
    fn build_response_carries_components_and_reference() {
        let r = build_response(&input(3, 4, 5)).unwrap();
        assert_eq!(r.calculator, "gcs");
        assert_eq!(r.result, json!(12));
        assert_eq!(r.working["level"], json!("moderate"));
        assert_eq!(r.working["eye_opening"], json!(3));
        assert_eq!(
            r.working["eye_opening_descriptor"],
            json!("Eye opening to speech (verbal command)")
        );
        assert_eq!(r.working["verbal_response"], json!(4));
        assert_eq!(r.working["motor_response"], json!(5));
        assert!(r.reference.contains("Teasdale"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "eye_opening": 2, "verbal_response": 3, "motor_response": 4 });
        let typed = input(2, 3, 4);
        let dynamic = Gcs.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(9));
    }

    #[test]
    fn dynamic_calculate_rejects_garbage() {
        assert!(Gcs.calculate(&json!({ "eye_opening": "open" })).is_err());
    }

    #[test]
    fn verbal_definition_notes_untestable_caveat() {
        let schema = Gcs.input_schema();
        let caveats = schema["properties"]["verbal_response"]["definition"]["caveats"]
            .as_str()
            .unwrap();
        assert!(caveats.contains("intubated"));
    }
}
