// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! GCS - Glasgow Coma Scale.
//!
//! A three-component bedside score of conscious level: eye opening (E),
//! verbal response (V), and motor response (M). Teasdale & Jennett (Lancet
//! 1974) introduced the three-component framework with a 3-14 range (the
//! original motor scale had five levels, not distinguishing normal from
//! abnormal flexion). The distinction between normal and abnormal
//! (decorticate) flexion was added in 1976-77, giving the six-level motor
//! scale and the familiar 3-15 total in use today. The current standardised
//! terminology, stimuli ("to sound" / "to pressure" rather than the older "to
//! speech" / "to pain"), and structured assessment approach are set out in
//! Teasdale et al, Nursing Times 2014.
//!
//! Copyright of the Glasgow Coma Scale is held by the University of Glasgow
//! and Sir Graham Teasdale; it is free to use for clinical care, teaching,
//! and research, with no licence required, subject to acknowledgement
//! (<https://www.glasgowcomascale.org/permissions/>).
//!
//! ## Clinical safety: "not testable" (NT)
//!
//! A component can be genuinely unassessable - most commonly verbal response
//! in an intubated or tracheostomised patient, but also eye opening (eyes
//! swollen shut, periorbital trauma) or motor response (neuromuscular
//! blockade, paralysis). The official guidance is explicit: record this as
//! "NT", never as a score of 1, and do not report a total or severity band
//! when any component is untestable, because a component recorded as if
//! absent understates the true conscious level and can mislead colleagues
//! (<https://www.glasgowcomascale.org/faq/>). This calculator models
//! testability as part of each component's value, so an untestable component
//! cannot be silently scored, and the total/band are only computed when all
//! three components are assessable.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "gcs";

/// Primary citations: the original scale, the current structured-approach
/// guidance (standardised terminology and the NT convention), and the
/// rationale for reporting components alongside (not only) the total.
pub const REFERENCE: &str = "Teasdale G, Jennett B. Assessment of coma and impaired consciousness. \
A practical scale. Lancet. 1974;2(7872):81-84. doi:10.1016/s0140-6736(74)91639-0 | Teasdale G, Allen D, \
Brennan P, McElhinney E, Mackinnon L. Forty years on: updating the Glasgow Coma Scale. Nursing Times. \
2014;110(42):12-16. | Teasdale G, Maas AIR, Lecky F, Manley G, Stocchetti N, Murray GD. The Glasgow Coma \
Scale at 40 years: standing the test of time. Lancet Neurol. 2014;13(8):844-854. \
doi:10.1016/S1474-4422(14)70120-6";

/// Distribution licence: the Glasgow Coma Scale is copyright the University
/// of Glasgow and Sir Graham Teasdale, free to use for clinical care,
/// teaching, and research with no licence required, subject to
/// acknowledgement; implemented here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Free to use for clinical care, teaching, and research, no licence required - copyright \
University of Glasgow and Sir Graham Teasdale, acknowledgement requested",
    source_url: "https://www.glasgowcomascale.org/permissions/",
};

/// Shared source citation for the per-component schema `definition` blocks.
const COMPONENT_SOURCE_CITATION: &str = "Teasdale G, Jennett B. Lancet. 1974;2(7872):81-84. | Teasdale \
G et al. Nursing Times. 2014;110(42):12-16.";

/// Eye-opening response (E). `NotTestable` scores no points and excludes the
/// total/band - see the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EyeOpening {
    /// Spontaneous eye opening.
    Spontaneous,
    /// Eye opening to sound (current standardised stimulus; formerly "to speech").
    ToSound,
    /// Eye opening to pressure (current standardised stimulus; formerly "to pain").
    ToPressure,
    /// No eye opening.
    NoResponse,
    /// Cannot be assessed, e.g. eyes swollen shut or periorbital trauma.
    NotTestable,
}

impl EyeOpening {
    fn score(self) -> Option<u8> {
        match self {
            EyeOpening::Spontaneous => Some(4),
            EyeOpening::ToSound => Some(3),
            EyeOpening::ToPressure => Some(2),
            EyeOpening::NoResponse => Some(1),
            EyeOpening::NotTestable => None,
        }
    }

    fn descriptor(self) -> &'static str {
        match self {
            EyeOpening::Spontaneous => "Spontaneous eye opening",
            EyeOpening::ToSound => "Eye opening to sound",
            EyeOpening::ToPressure => "Eye opening to pressure",
            EyeOpening::NoResponse => "No eye opening",
            EyeOpening::NotTestable => "Not testable (e.g. eyes swollen shut, periorbital trauma)",
        }
    }
}

/// Verbal response (V). `NotTestable` scores no points and excludes the
/// total/band - see the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerbalResponse {
    /// Orientated to person, place, and time.
    Orientated,
    /// Confused conversation, able to answer questions.
    Confused,
    /// Inappropriate words, exclamations, or random words - not a conversational exchange.
    Words,
    /// Incomprehensible sounds - moaning, groaning, no recognisable words.
    Sounds,
    /// No verbal response.
    NoResponse,
    /// Cannot be assessed, e.g. intubated, tracheostomised, or dysphasic.
    NotTestable,
}

impl VerbalResponse {
    fn score(self) -> Option<u8> {
        match self {
            VerbalResponse::Orientated => Some(5),
            VerbalResponse::Confused => Some(4),
            VerbalResponse::Words => Some(3),
            VerbalResponse::Sounds => Some(2),
            VerbalResponse::NoResponse => Some(1),
            VerbalResponse::NotTestable => None,
        }
    }

    fn descriptor(self) -> &'static str {
        match self {
            VerbalResponse::Orientated => "Orientated",
            VerbalResponse::Confused => "Confused conversation",
            VerbalResponse::Words => "Words (inappropriate, not a conversational exchange)",
            VerbalResponse::Sounds => "Sounds (incomprehensible)",
            VerbalResponse::NoResponse => "No verbal response",
            VerbalResponse::NotTestable => {
                "Not testable (e.g. intubated, tracheostomised, dysphasic)"
            }
        }
    }
}

/// Motor response (M). `NotTestable` scores no points and excludes the
/// total/band - see the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotorResponse {
    /// Obeys commands.
    ObeyCommands,
    /// Localises to a pressure stimulus.
    Localising,
    /// Normal flexion (withdrawal) to a pressure stimulus.
    NormalFlexion,
    /// Abnormal flexion to a pressure stimulus (decorticate posturing).
    AbnormalFlexion,
    /// Extension to a pressure stimulus (decerebrate posturing).
    Extension,
    /// No motor response.
    NoResponse,
    /// Cannot be assessed, e.g. neuromuscular blockade or paralysis.
    NotTestable,
}

impl MotorResponse {
    fn score(self) -> Option<u8> {
        match self {
            MotorResponse::ObeyCommands => Some(6),
            MotorResponse::Localising => Some(5),
            MotorResponse::NormalFlexion => Some(4),
            MotorResponse::AbnormalFlexion => Some(3),
            MotorResponse::Extension => Some(2),
            MotorResponse::NoResponse => Some(1),
            MotorResponse::NotTestable => None,
        }
    }

    fn descriptor(self) -> &'static str {
        match self {
            MotorResponse::ObeyCommands => "Obeys commands",
            MotorResponse::Localising => "Localises to pressure",
            MotorResponse::NormalFlexion => "Normal flexion (withdrawal) to pressure",
            MotorResponse::AbnormalFlexion => {
                "Abnormal flexion to pressure (decorticate posturing)"
            }
            MotorResponse::Extension => "Extension to pressure (decerebrate posturing)",
            MotorResponse::NoResponse => "No motor response",
            MotorResponse::NotTestable => "Not testable (e.g. neuromuscular blockade, paralysis)",
        }
    }
}

/// Severity band implied by the total score. Only produced when all three
/// components are testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// Total 3-8: severe.
    Severe,
    /// Total 9-12: moderate.
    Moderate,
    /// Total 13-15: mild, or a normal conscious level.
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

/// GCS inputs: the three component responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GcsInput {
    /// Eye-opening response (E).
    pub eye_opening: EyeOpening,
    /// Verbal response (V).
    pub verbal_response: VerbalResponse,
    /// Motor response (M).
    pub motor_response: MotorResponse,
}

/// The computed outcome. `total_score` and `band` are only `Some` when all
/// three components are testable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcsOutcome {
    pub eye_opening: EyeOpening,
    pub eye_opening_score: Option<u8>,
    pub verbal_response: VerbalResponse,
    pub verbal_response_score: Option<u8>,
    pub motor_response: MotorResponse,
    pub motor_response_score: Option<u8>,
    pub total_score: Option<u8>,
    pub band: Option<Band>,
    pub interpretation: String,
}

/// `"3"`, `"NT"`, etc, for the interpretation's component summary.
fn score_label(score: Option<u8>) -> String {
    match score {
        Some(s) => s.to_string(),
        None => "NT".to_string(),
    }
}

/// Compose the interpretation. Reports the total/band only when all three
/// components are testable; otherwise names which are not and explains why
/// no total is given (see the module documentation).
fn interpretation_text(
    eye_score: Option<u8>,
    verbal_score: Option<u8>,
    motor_score: Option<u8>,
    total_and_band: Option<(u8, Band)>,
) -> String {
    let components = format!(
        "E{} V{} M{}",
        score_label(eye_score),
        score_label(verbal_score),
        score_label(motor_score)
    );

    match total_and_band {
        Some((total, band)) => {
            let band_text = match band {
                Band::Severe => {
                    "severe (GCS 3-8) - the band associated with the greatest impairment of conscious level"
                }
                Band::Moderate => "moderate (GCS 9-12)",
                Band::Mild => "mild, or a normal conscious level (GCS 13-15)",
            };
            format!(
                "GCS {total}/15 ({components}): {band_text}. Report and track the individual eye, \
verbal, and motor components alongside the total - a change in one component can be clinically \
significant even if the total is unchanged (Teasdale et al, Lancet Neurol. 2014;13(8):844-854)."
            )
        }
        None => {
            let mut not_testable = Vec::new();
            if eye_score.is_none() {
                not_testable.push("eye opening");
            }
            if verbal_score.is_none() {
                not_testable.push("verbal response");
            }
            if motor_score.is_none() {
                not_testable.push("motor response");
            }
            format!(
                "{components}: {} not testable, so no total score or severity band is reported - a \
component recorded as if absent (a score of 1) would falsely lower the apparent conscious level and \
could mislead colleagues. Assess, communicate, and make decisions using the components that can be \
scored (glasgowcomascale.org FAQ; Teasdale et al, Nursing Times. 2014;110(42):12-16).",
                not_testable.join(", ")
            )
        }
    }
}

/// Pure scoring: score each component and, only if all three are testable,
/// sum to a total and band.
pub fn compute(input: &GcsInput) -> Result<GcsOutcome, CalcError> {
    let eye_score = input.eye_opening.score();
    let verbal_score = input.verbal_response.score();
    let motor_score = input.motor_response.score();

    let total_and_band = match (eye_score, verbal_score, motor_score) {
        (Some(e), Some(v), Some(m)) => {
            let total = e + v + m;
            let band = match total {
                3..=8 => Band::Severe,
                9..=12 => Band::Moderate,
                _ => Band::Mild,
            };
            Some((total, band))
        }
        _ => None,
    };

    let interpretation = interpretation_text(eye_score, verbal_score, motor_score, total_and_band);

    Ok(GcsOutcome {
        eye_opening: input.eye_opening,
        eye_opening_score: eye_score,
        verbal_response: input.verbal_response,
        verbal_response_score: verbal_score,
        motor_response: input.motor_response,
        motor_response_score: motor_score,
        total_score: total_and_band.map(|(t, _)| t),
        band: total_and_band.map(|(_, b)| b),
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &GcsInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("eye_opening".into(), json!(o.eye_opening));
    working.insert(
        "eye_opening_descriptor".into(),
        json!(o.eye_opening.descriptor()),
    );
    if let Some(score) = o.eye_opening_score {
        working.insert("eye_opening_score".into(), json!(score));
    }
    working.insert("verbal_response".into(), json!(o.verbal_response));
    working.insert(
        "verbal_response_descriptor".into(),
        json!(o.verbal_response.descriptor()),
    );
    if let Some(score) = o.verbal_response_score {
        working.insert("verbal_response_score".into(), json!(score));
    }
    working.insert("motor_response".into(), json!(o.motor_response));
    working.insert(
        "motor_response_descriptor".into(),
        json!(o.motor_response.descriptor()),
    );
    if let Some(score) = o.motor_response_score {
        working.insert("motor_response_score".into(), json!(score));
    }
    if let Some(total) = o.total_score {
        working.insert("total_score".into(), json!(total));
    }
    if let Some(band) = o.band {
        working.insert("level".into(), json!(band.slug()));
    }

    let result = match o.total_score {
        Some(total) => json!(total),
        None => json!("not_testable"),
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result,
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
        "Bedside score (3-15) of conscious level from eye, verbal, and motor response (Teasdale & \
Jennett 1974); omits the total and band when any component is not testable, per current guidance."
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
                    "type": "string",
                    "enum": ["spontaneous", "to_sound", "to_pressure", "no_response", "not_testable"],
                    "description": "Eye-opening response (E). spontaneous (4); to_sound (3); to_pressure (2); no_response (1); not_testable (NT) if eyes cannot be assessed - never substitute no_response.",
                    "definition": {
                        "concept": "GCS eye-opening component (E)",
                        "statement": "An ordinal score of eye opening: spontaneous (4), to sound (3), to pressure (2), or none (1). 'To sound' and 'to pressure' are the current standardised stimuli (previously described as 'to speech' and 'to pain').",
                        "caveats": "If eye opening genuinely cannot be assessed (e.g. eyes swollen shut, periorbital trauma), record not_testable. Do not record no_response as a substitute - that misrepresents an unassessable finding as the worst possible response and falsely lowers the apparent conscious level.",
                        "source": { "citation": COMPONENT_SOURCE_CITATION, "url": "https://www.glasgowcomascale.org/gcs-aid/" },
                        "status": "draft"
                    }
                },
                "verbal_response": {
                    "type": "string",
                    "enum": ["orientated", "confused", "words", "sounds", "no_response", "not_testable"],
                    "description": "Verbal response (V). orientated (5); confused (4); words (3, inappropriate); sounds (2, incomprehensible); no_response (1); not_testable (NT) - e.g. intubated, tracheostomised, dysphasic - never substitute no_response.",
                    "definition": {
                        "concept": "GCS verbal-response component (V)",
                        "statement": "An ordinal score of verbal response: orientated (5), confused (4), words (3), sounds (2), or none (1).",
                        "caveats": "Cannot be assessed in an intubated, tracheostomised, or dysphasic patient. Record not_testable, not a score of 1: a score of 1 falsely lowers the apparent conscious level, and the total should not be reported when this happens.",
                        "source": { "citation": COMPONENT_SOURCE_CITATION, "url": "https://www.glasgowcomascale.org/faq/" },
                        "status": "draft"
                    }
                },
                "motor_response": {
                    "type": "string",
                    "enum": ["obey_commands", "localising", "normal_flexion", "abnormal_flexion", "extension", "no_response", "not_testable"],
                    "description": "Motor response (M). obey_commands (6); localising (5); normal_flexion (4); abnormal_flexion (3, decorticate); extension (2, decerebrate); no_response (1); not_testable (NT) - e.g. neuromuscular blockade, paralysis - never substitute no_response.",
                    "definition": {
                        "concept": "GCS motor-response component (M)",
                        "statement": "An ordinal score of the best motor response observed in any limb: obeys commands (6), localises to pressure (5), normal flexion/withdrawal (4), abnormal flexion/decorticate posturing (3), extension/decerebrate posturing (2), or none (1). The six-level form (normal vs abnormal flexion distinguished) postdates the original 1974 publication, which had five motor levels and a maximum total of 14; the distinction was added in 1976-77, giving today's six-level, 15-point scale.",
                        "caveats": "If motor response cannot be assessed (e.g. neuromuscular blockade, bilateral limb paralysis or injury), record not_testable, not a score of 1.",
                        "source": { "citation": COMPONENT_SOURCE_CITATION, "url": "https://www.glasgowcomascale.org/gcs-aid/" },
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

    fn input(eye: EyeOpening, verbal: VerbalResponse, motor: MotorResponse) -> GcsInput {
        GcsInput {
            eye_opening: eye,
            verbal_response: verbal,
            motor_response: motor,
        }
    }

    #[test]
    fn fully_alert_scores_fifteen() {
        let o = compute(&input(
            EyeOpening::Spontaneous,
            VerbalResponse::Orientated,
            MotorResponse::ObeyCommands,
        ))
        .unwrap();
        assert_eq!(o.total_score, Some(15));
        assert_eq!(o.band, Some(Band::Mild));
    }

    #[test]
    fn unresponsive_scores_three() {
        let o = compute(&input(
            EyeOpening::NoResponse,
            VerbalResponse::NoResponse,
            MotorResponse::NoResponse,
        ))
        .unwrap();
        assert_eq!(o.total_score, Some(3));
        assert_eq!(o.band, Some(Band::Severe));
    }

    #[test]
    fn severe_band_upper_boundary_is_inclusive() {
        // E1 V2 M5 = 8: still severe.
        let o = compute(&input(
            EyeOpening::NoResponse,
            VerbalResponse::Sounds,
            MotorResponse::Localising,
        ))
        .unwrap();
        assert_eq!(o.total_score, Some(8));
        assert_eq!(o.band, Some(Band::Severe));
    }

    #[test]
    fn moderate_band_boundaries() {
        // E2 V2 M5 = 9: lowest moderate.
        let o = compute(&input(
            EyeOpening::ToPressure,
            VerbalResponse::Sounds,
            MotorResponse::Localising,
        ))
        .unwrap();
        assert_eq!(o.total_score, Some(9));
        assert_eq!(o.band, Some(Band::Moderate));

        // E4 V3 M5 = 12: highest moderate.
        let o = compute(&input(
            EyeOpening::Spontaneous,
            VerbalResponse::Words,
            MotorResponse::Localising,
        ))
        .unwrap();
        assert_eq!(o.total_score, Some(12));
        assert_eq!(o.band, Some(Band::Moderate));
    }

    #[test]
    fn mild_band_lower_boundary_is_inclusive() {
        // E4 V4 M5 = 13: lowest mild.
        let o = compute(&input(
            EyeOpening::Spontaneous,
            VerbalResponse::Confused,
            MotorResponse::Localising,
        ))
        .unwrap();
        assert_eq!(o.total_score, Some(13));
        assert_eq!(o.band, Some(Band::Mild));
    }

    #[test]
    fn verbal_not_testable_omits_total_and_band() {
        let o = compute(&input(
            EyeOpening::Spontaneous,
            VerbalResponse::NotTestable,
            MotorResponse::ObeyCommands,
        ))
        .unwrap();
        assert_eq!(o.total_score, None);
        assert_eq!(o.band, None);
        assert_eq!(o.eye_opening_score, Some(4));
        assert_eq!(o.verbal_response_score, None);
        assert_eq!(o.motor_response_score, Some(6));
        assert!(o.interpretation.contains("verbal response"));
        assert!(o.interpretation.contains("not testable"));
        assert!(!o.interpretation.contains("GCS 3-8"));
        assert!(!o.interpretation.contains("GCS 9-12"));
        assert!(!o.interpretation.contains("GCS 13-15"));
    }

    #[test]
    fn eye_not_testable_omits_total_and_band() {
        let o = compute(&input(
            EyeOpening::NotTestable,
            VerbalResponse::Orientated,
            MotorResponse::ObeyCommands,
        ))
        .unwrap();
        assert_eq!(o.total_score, None);
        assert_eq!(o.band, None);
        assert!(o.interpretation.contains("eye opening"));
    }

    #[test]
    fn motor_not_testable_omits_total_and_band() {
        let o = compute(&input(
            EyeOpening::Spontaneous,
            VerbalResponse::Orientated,
            MotorResponse::NotTestable,
        ))
        .unwrap();
        assert_eq!(o.total_score, None);
        assert_eq!(o.band, None);
        assert!(o.interpretation.contains("motor response"));
    }

    #[test]
    fn multiple_not_testable_lists_all_of_them() {
        let o = compute(&input(
            EyeOpening::NotTestable,
            VerbalResponse::NotTestable,
            MotorResponse::ObeyCommands,
        ))
        .unwrap();
        assert!(o.interpretation.contains("eye opening, verbal response"));
    }

    #[test]
    fn build_response_result_is_not_testable_string_when_nt() {
        let r = build_response(&input(
            EyeOpening::Spontaneous,
            VerbalResponse::NotTestable,
            MotorResponse::ObeyCommands,
        ))
        .unwrap();
        assert_eq!(r.result, json!("not_testable"));
        assert!(!r.working.contains_key("total_score"));
        assert!(!r.working.contains_key("level"));
        assert!(!r.working.contains_key("verbal_response_score"));
        assert_eq!(r.working["eye_opening_score"], json!(4));
    }

    #[test]
    fn build_response_carries_components_and_reference() {
        let r = build_response(&input(
            EyeOpening::ToSound,
            VerbalResponse::Confused,
            MotorResponse::Localising,
        ))
        .unwrap();
        assert_eq!(r.calculator, "gcs");
        assert_eq!(r.result, json!(12));
        assert_eq!(r.working["level"], json!("moderate"));
        assert_eq!(r.working["eye_opening"], json!("to_sound"));
        assert_eq!(
            r.working["eye_opening_descriptor"],
            json!("Eye opening to sound")
        );
        assert_eq!(r.working["verbal_response_score"], json!(4));
        assert_eq!(r.working["motor_response_score"], json!(5));
        assert!(r.reference.contains("Teasdale"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "eye_opening": "to_pressure",
            "verbal_response": "words",
            "motor_response": "normal_flexion"
        });
        let typed = input(
            EyeOpening::ToPressure,
            VerbalResponse::Words,
            MotorResponse::NormalFlexion,
        );
        let dynamic = Gcs.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(9));
    }

    #[test]
    fn dynamic_calculate_handles_not_testable() {
        let value = json!({
            "eye_opening": "spontaneous",
            "verbal_response": "not_testable",
            "motor_response": "obey_commands"
        });
        let dynamic = Gcs.calculate(&value).unwrap();
        assert_eq!(dynamic.result, json!("not_testable"));
    }

    #[test]
    fn dynamic_calculate_rejects_garbage() {
        assert!(
            Gcs.calculate(&json!({ "eye_opening": "wide_open" }))
                .is_err()
        );
    }

    #[test]
    fn dynamic_calculate_rejects_unknown_fields() {
        let value = json!({
            "eye_opening": "spontaneous",
            "verbal_response": "orientated",
            "motor_response": "obey_commands",
            "pupil_size_mm": 3
        });
        assert!(Gcs.calculate(&value).is_err());
    }

    #[test]
    fn schema_uses_current_terminology_and_not_testable_option() {
        let schema = Gcs.input_schema();
        let eye_enum: Vec<&str> = schema["properties"]["eye_opening"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(eye_enum.contains(&"to_sound"));
        assert!(eye_enum.contains(&"to_pressure"));
        assert!(eye_enum.contains(&"not_testable"));
        assert!(!eye_enum.contains(&"to_speech"));
        assert!(!eye_enum.contains(&"to_pain"));

        let verbal_caveats = schema["properties"]["verbal_response"]["definition"]["caveats"]
            .as_str()
            .unwrap();
        assert!(verbal_caveats.contains("not a score of 1"));
    }
}
