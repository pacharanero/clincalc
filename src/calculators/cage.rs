// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! CAGE Questionnaire - four-question alcohol-misuse screen.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "cage";
pub const REFERENCE: &str =
    "Ewing JA. Detecting alcoholism: the CAGE questionnaire. JAMA. 1984;252(14):1905-1907.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain instrument - implemented from the primary literature",
    "https://doi.org/10.1001/jama.1984.03350140051025",
);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CageInput {
    /// Ever felt you should Cut down on your drinking?
    pub cut_down: bool,
    /// Have people Annoyed you by criticising your drinking?
    pub annoyed: bool,
    /// Have you ever felt bad or Guilty about your drinking?
    pub guilty: bool,
    /// Eye-opener: ever had a drink first thing in the morning to steady your
    /// nerves or get rid of a hangover?
    pub eye_opener: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CageOutcome {
    pub score: u8,
    pub likely_problem_drinking: bool,
}

pub fn compute(input: &CageInput) -> CageOutcome {
    let score = [
        input.cut_down,
        input.annoyed,
        input.guilty,
        input.eye_opener,
    ]
    .iter()
    .filter(|v| **v)
    .count() as u8;
    CageOutcome {
        score,
        likely_problem_drinking: score >= 2,
    }
}

pub fn build_response(input: &CageInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input);
    let mut working = Map::new();
    working.insert(
        "likely_problem_drinking".into(),
        json!(o.likely_problem_drinking),
    );

    let interpretation = if o.likely_problem_drinking {
        format!(
            "CAGE {}/4. A score of 2 or more is a positive screen, commonly cited at roughly 70-80% sensitivity and specificity for alcohol misuse/dependence; a positive screen warrants a fuller history, not a diagnosis.",
            o.score
        )
    } else {
        format!(
            "CAGE {}/4. Below the usual positive-screen threshold of 2; a low score does not exclude at-risk drinking, particularly in patients screened only once or who may under-report.",
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

pub struct Cage;
impl Calculator for Cage {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "CAGE Questionnaire"
    }
    fn description(&self) -> &'static str {
        "Four-question alcohol-misuse screen; a score of 2 or more out of 4 is a positive screen."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "CageInput", "type": "object", "additionalProperties": false,
            "required": ["cut_down", "annoyed", "guilty", "eye_opener"],
            "properties": {
                "cut_down": { "type": "boolean", "description": "Ever felt you should cut down on your drinking?" },
                "annoyed": { "type": "boolean", "description": "Have people annoyed you by criticising your drinking?" },
                "guilty": { "type": "boolean", "description": "Have you ever felt bad or guilty about your drinking?" },
                "eye_opener": { "type": "boolean", "description": "Ever had a drink first thing in the morning (eye-opener) to steady your nerves or get rid of a hangover?" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: CageInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_negative_scores_zero() {
        let o = compute(&CageInput {
            cut_down: false,
            annoyed: false,
            guilty: false,
            eye_opener: false,
        });
        assert_eq!(o.score, 0);
        assert!(!o.likely_problem_drinking);
    }

    #[test]
    fn two_positive_crosses_threshold() {
        let o = compute(&CageInput {
            cut_down: true,
            annoyed: true,
            guilty: false,
            eye_opener: false,
        });
        assert_eq!(o.score, 2);
        assert!(o.likely_problem_drinking);
    }

    #[test]
    fn all_positive_scores_four() {
        let o = compute(&CageInput {
            cut_down: true,
            annoyed: true,
            guilty: true,
            eye_opener: true,
        });
        assert_eq!(o.score, 4);
        assert!(o.likely_problem_drinking);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value =
            json!({"cut_down": true, "annoyed": false, "guilty": true, "eye_opener": false});
        let typed = CageInput {
            cut_down: true,
            annoyed: false,
            guilty: true,
            eye_opener: false,
        };
        let dynamic = Cage.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            Cage.calculate(&json!({"cut_down": true, "annoyed": false, "guilty": true}))
                .is_err()
        );
    }
}
