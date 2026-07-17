// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Alvarado score (MANTRELS) for suspected acute appendicitis.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "alvarado";
pub const REFERENCE: &str = "Alvarado A. A practical score for the early diagnosis of acute appendicitis. Ann Emerg Med. 1986;15(5):557-564. doi:10.1016/S0196-0644(86)80993-3";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1016/S0196-0644(86)80993-3",
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlvaradoInput {
    pub migration_to_rlq: bool,
    pub anorexia: bool,
    pub nausea_or_vomiting: bool,
    pub rlq_tenderness: bool,
    pub rebound_tenderness: bool,
    pub fever: bool,
    pub leukocytosis: bool,
    pub left_shift: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbabilityBand {
    Unlikely,
    Possible,
    Probable,
    VeryProbable,
}
impl ProbabilityBand {
    fn slug(self) -> &'static str {
        match self {
            ProbabilityBand::Unlikely => "unlikely",
            ProbabilityBand::Possible => "possible",
            ProbabilityBand::Probable => "probable",
            ProbabilityBand::VeryProbable => "very_probable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlvaradoOutcome {
    pub score: u8,
    pub band: ProbabilityBand,
    pub interpretation: String,
}

pub fn compute(input: &AlvaradoInput) -> Result<AlvaradoOutcome, CalcError> {
    let score = u8::from(input.migration_to_rlq)
        + u8::from(input.anorexia)
        + u8::from(input.nausea_or_vomiting)
        + 2 * u8::from(input.rlq_tenderness)
        + u8::from(input.rebound_tenderness)
        + u8::from(input.fever)
        + 2 * u8::from(input.leukocytosis)
        + u8::from(input.left_shift);
    let band = match score {
        0..=4 => ProbabilityBand::Unlikely,
        5..=6 => ProbabilityBand::Possible,
        7..=8 => ProbabilityBand::Probable,
        _ => ProbabilityBand::VeryProbable,
    };
    let interpretation = match band {
        ProbabilityBand::Unlikely => format!(
            "Alvarado score {score}/10: appendicitis is less likely. Use clinical judgement and safety-netting; the score does not exclude appendicitis."
        ),
        ProbabilityBand::Possible => format!(
            "Alvarado score {score}/10: compatible with appendicitis. Observation, repeat examination, imaging, or surgical review may be appropriate depending on context."
        ),
        ProbabilityBand::Probable => format!(
            "Alvarado score {score}/10: probable appendicitis. Surgical assessment and/or imaging is usually warranted."
        ),
        ProbabilityBand::VeryProbable => format!(
            "Alvarado score {score}/10: very probable appendicitis in the original scoring scheme. Arrange urgent surgical assessment."
        ),
    };
    Ok(AlvaradoOutcome {
        score,
        band,
        interpretation,
    })
}

pub fn build_response(input: &AlvaradoInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "migration_to_rlq".into(),
        json!(u8::from(input.migration_to_rlq)),
    );
    working.insert("anorexia".into(), json!(u8::from(input.anorexia)));
    working.insert(
        "nausea_or_vomiting".into(),
        json!(u8::from(input.nausea_or_vomiting)),
    );
    working.insert(
        "rlq_tenderness".into(),
        json!(2 * u8::from(input.rlq_tenderness)),
    );
    working.insert(
        "rebound_tenderness".into(),
        json!(u8::from(input.rebound_tenderness)),
    );
    working.insert("fever".into(), json!(u8::from(input.fever)));
    working.insert(
        "leukocytosis".into(),
        json!(2 * u8::from(input.leukocytosis)),
    );
    working.insert("left_shift".into(), json!(u8::from(input.left_shift)));
    working.insert("probability_band".into(), json!(o.band.slug()));
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Alvarado;
impl Calculator for Alvarado {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Alvarado Score (Appendicitis)"
    }
    fn description(&self) -> &'static str {
        "MANTRELS score for suspected acute appendicitis (0-10)."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        let bool_prop = |desc: &str| json!({ "type": "boolean", "description": desc });
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "AlvaradoInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["migration_to_rlq", "anorexia", "nausea_or_vomiting", "rlq_tenderness", "rebound_tenderness", "fever", "leukocytosis", "left_shift"],
            "properties": {
                "migration_to_rlq": bool_prop("Migration of pain to the right lower quadrant (1 point)"),
                "anorexia": bool_prop("Anorexia / acetone (1 point)"),
                "nausea_or_vomiting": bool_prop("Nausea and/or vomiting (1 point)"),
                "rlq_tenderness": bool_prop("Tenderness in the right lower quadrant (2 points)"),
                "rebound_tenderness": bool_prop("Rebound pain/tenderness (1 point)"),
                "fever": bool_prop("Elevated temperature (1 point)"),
                "leukocytosis": bool_prop("Leukocytosis (2 points)"),
                "left_shift": bool_prop("Left shift of neutrophil maturation (1 point)")
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AlvaradoInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_criteria_score_ten() {
        let out = compute(&AlvaradoInput {
            migration_to_rlq: true,
            anorexia: true,
            nausea_or_vomiting: true,
            rlq_tenderness: true,
            rebound_tenderness: true,
            fever: true,
            leukocytosis: true,
            left_shift: true,
        })
        .unwrap();
        assert_eq!(out.score, 10);
        assert_eq!(out.band, ProbabilityBand::VeryProbable);
    }
    #[test]
    fn rlq_and_leukocytosis_are_two_points_each() {
        let out = compute(&AlvaradoInput {
            migration_to_rlq: false,
            anorexia: false,
            nausea_or_vomiting: false,
            rlq_tenderness: true,
            rebound_tenderness: false,
            fever: false,
            leukocytosis: true,
            left_shift: false,
        })
        .unwrap();
        assert_eq!(out.score, 4);
    }
}
