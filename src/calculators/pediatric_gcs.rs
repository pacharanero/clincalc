// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Paediatric (preverbal) Glasgow Coma Scale.
//!
//! A modification of the adult GCS for infants and young children who are
//! not yet reliably verbal: eye opening uses the same four-level scale as
//! the adult GCS, while verbal and motor responses use age-appropriate
//! descriptors (Reilly et al, 1988). Scoring weights (4/5/6) and the 3-15
//! range are unchanged from the adult scale, so the same severity bands
//! apply.
//!
//! As with the shipped adult `gcs`, a component that genuinely cannot be
//! assessed is recorded as not-testable (NT) rather than scored as the
//! worst response, and the total/band are withheld unless all three
//! components are testable - see `gcs.rs` for the full clinical-safety
//! rationale, which applies identically here.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "pediatric_gcs";
pub const REFERENCE: &str = "Reilly PL, Simpson DA, Sprod R, Thomas L. Assessing the conscious level in infants and young children: a paediatric version of the Glasgow Coma Scale. Childs Nerv Syst. 1988;4(1):30-33. Adapted from Teasdale G, Jennett B. Assessment of coma and impaired consciousness. A practical scale. Lancet. 1974;2(7872):81-84.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain adaptation - implemented from the primary literature; the base Glasgow Coma Scale is free to use for clinical care, teaching, and research (University of Glasgow / Sir Graham Teasdale), acknowledgement requested",
    "https://doi.org/10.1007/BF00256556",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EyeOpening {
    Spontaneous,
    ToSound,
    ToPressure,
    NoResponse,
    NotTestable,
}
impl EyeOpening {
    fn score(self) -> Option<u8> {
        match self {
            Self::Spontaneous => Some(4),
            Self::ToSound => Some(3),
            Self::ToPressure => Some(2),
            Self::NoResponse => Some(1),
            Self::NotTestable => None,
        }
    }
    fn descriptor(self) -> &'static str {
        match self {
            Self::Spontaneous => "Spontaneous eye opening",
            Self::ToSound => "Eye opening to sound",
            Self::ToPressure => "Eye opening to pressure",
            Self::NoResponse => "No eye opening",
            Self::NotTestable => "Not testable (e.g. eyes swollen shut, periorbital trauma)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerbalResponse {
    /// Coos, babbles, or uses age-appropriate words/phrases; smiles, fixes and follows.
    AppropriateWordsOrSmiles,
    /// Cries but is consolable.
    IrritableCriesConsolable,
    /// Persistently irritable, inconsolable crying.
    InconsolableCrying,
    /// Grunts or moans to pain, inconsistently.
    GruntsOrMoans,
    NoResponse,
    NotTestable,
}
impl VerbalResponse {
    fn score(self) -> Option<u8> {
        match self {
            Self::AppropriateWordsOrSmiles => Some(5),
            Self::IrritableCriesConsolable => Some(4),
            Self::InconsolableCrying => Some(3),
            Self::GruntsOrMoans => Some(2),
            Self::NoResponse => Some(1),
            Self::NotTestable => None,
        }
    }
    fn descriptor(self) -> &'static str {
        match self {
            Self::AppropriateWordsOrSmiles => {
                "Coos/babbles or age-appropriate words/phrases; smiles, fixes and follows"
            }
            Self::IrritableCriesConsolable => "Irritable cries but consolable",
            Self::InconsolableCrying => "Persistently irritable, inconsolable crying",
            Self::GruntsOrMoans => "Grunts or moans to pain (inconsistent)",
            Self::NoResponse => "No verbal response",
            Self::NotTestable => "Not testable (e.g. intubated, sedated)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotorResponse {
    NormalSpontaneousMovements,
    WithdrawsToTouch,
    WithdrawsToPain,
    AbnormalFlexion,
    Extension,
    NoResponse,
    NotTestable,
}
impl MotorResponse {
    fn score(self) -> Option<u8> {
        match self {
            Self::NormalSpontaneousMovements => Some(6),
            Self::WithdrawsToTouch => Some(5),
            Self::WithdrawsToPain => Some(4),
            Self::AbnormalFlexion => Some(3),
            Self::Extension => Some(2),
            Self::NoResponse => Some(1),
            Self::NotTestable => None,
        }
    }
    fn descriptor(self) -> &'static str {
        match self {
            Self::NormalSpontaneousMovements => "Normal spontaneous movements",
            Self::WithdrawsToTouch => "Withdraws to touch",
            Self::WithdrawsToPain => "Withdraws to pain",
            Self::AbnormalFlexion => "Abnormal flexion to pain (decorticate posturing)",
            Self::Extension => "Extension to pain (decerebrate posturing)",
            Self::NoResponse => "No motor response",
            Self::NotTestable => "Not testable (e.g. neuromuscular blockade, paralysis)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Severe,
    Moderate,
    Mild,
}
impl Band {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Severe => "severe",
            Self::Moderate => "moderate",
            Self::Mild => "mild",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PediatricGcsInput {
    pub eye_opening: EyeOpening,
    pub verbal_response: VerbalResponse,
    pub motor_response: MotorResponse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PediatricGcsOutcome {
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

fn score_label(score: Option<u8>) -> String {
    match score {
        Some(s) => s.to_string(),
        None => "NT".to_string(),
    }
}

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
                Band::Severe => "severe (3-8)",
                Band::Moderate => "moderate (9-12)",
                Band::Mild => "mild, or a normal conscious level (13-15)",
            };
            format!(
                "Paediatric GCS {total}/15 ({components}): {band_text}. Report and track the individual components alongside the total, and use age-appropriate expectations for a preverbal child - a change in one component can be clinically significant even if the total is unchanged."
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
                "{components}: {} not testable, so no total score or severity band is reported - a component recorded as if absent would falsely lower the apparent conscious level. Assess, communicate, and make decisions using the components that can be scored.",
                not_testable.join(", ")
            )
        }
    }
}

pub fn compute(input: &PediatricGcsInput) -> Result<PediatricGcsOutcome, CalcError> {
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

    Ok(PediatricGcsOutcome {
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

pub fn build_response(input: &PediatricGcsInput) -> Result<CalculationResponse, CalcError> {
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

pub struct PediatricGcs;
impl Calculator for PediatricGcs {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Paediatric (Preverbal) Glasgow Coma Scale"
    }
    fn description(&self) -> &'static str {
        "Modified GCS for infants and young children who are not yet reliably verbal; omits the total and band when any component is not testable."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "PediatricGcsInput", "type": "object", "additionalProperties": false,
            "required": ["eye_opening", "verbal_response", "motor_response"],
            "properties": {
                "eye_opening": { "type": "string", "enum": ["spontaneous", "to_sound", "to_pressure", "no_response", "not_testable"] },
                "verbal_response": { "type": "string", "enum": ["appropriate_words_or_smiles", "irritable_cries_consolable", "inconsolable_crying", "grunts_or_moans", "no_response", "not_testable"], "description": "appropriate_words_or_smiles (5): coos/babbles, age-appropriate words/phrases, or smiles/fixes/follows" },
                "motor_response": { "type": "string", "enum": ["normal_spontaneous_movements", "withdraws_to_touch", "withdraws_to_pain", "abnormal_flexion", "extension", "no_response", "not_testable"] }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: PediatricGcsInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(eye: EyeOpening, verbal: VerbalResponse, motor: MotorResponse) -> PediatricGcsInput {
        PediatricGcsInput {
            eye_opening: eye,
            verbal_response: verbal,
            motor_response: motor,
        }
    }

    #[test]
    fn fully_alert_infant_scores_fifteen() {
        let o = compute(&input(
            EyeOpening::Spontaneous,
            VerbalResponse::AppropriateWordsOrSmiles,
            MotorResponse::NormalSpontaneousMovements,
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
    fn verbal_not_testable_omits_total_and_band() {
        let o = compute(&input(
            EyeOpening::Spontaneous,
            VerbalResponse::NotTestable,
            MotorResponse::NormalSpontaneousMovements,
        ))
        .unwrap();
        assert_eq!(o.total_score, None);
        assert_eq!(o.band, None);
        assert!(o.interpretation.contains("verbal response"));
        assert!(o.interpretation.contains("not testable"));
    }

    #[test]
    fn build_response_result_is_not_testable_string_when_nt() {
        let r = build_response(&input(
            EyeOpening::Spontaneous,
            VerbalResponse::NotTestable,
            MotorResponse::NormalSpontaneousMovements,
        ))
        .unwrap();
        assert_eq!(r.result, json!("not_testable"));
        assert!(!r.working.contains_key("total_score"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "eye_opening": "to_pressure",
            "verbal_response": "grunts_or_moans",
            "motor_response": "withdraws_to_pain"
        });
        let typed = input(
            EyeOpening::ToPressure,
            VerbalResponse::GruntsOrMoans,
            MotorResponse::WithdrawsToPain,
        );
        let dynamic = PediatricGcs.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            PediatricGcs
                .calculate(&json!({"eye_opening": "spontaneous"}))
                .is_err()
        );
    }
}
