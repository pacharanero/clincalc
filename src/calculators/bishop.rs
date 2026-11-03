// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bishop Score - five-parameter cervical-exam score predicting the
//! likelihood of successful labour induction.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "bishop";
pub const REFERENCE: &str =
    "Bishop EH. Pelvic scoring for elective induction. Obstet Gynecol. 1964;24:266-268.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://pubmed.ncbi.nlm.nih.gov/14199536/",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dilation {
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "1_2_cm")]
    OneToTwoCm,
    #[serde(rename = "3_4_cm")]
    ThreeToFourCm,
    #[serde(rename = "5_plus_cm")]
    FivePlusCm,
}
impl Dilation {
    fn points(self) -> u8 {
        match self {
            Self::Closed => 0,
            Self::OneToTwoCm => 1,
            Self::ThreeToFourCm => 2,
            Self::FivePlusCm => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effacement {
    #[serde(rename = "0_30_percent")]
    ZeroToThirtyPercent,
    #[serde(rename = "40_50_percent")]
    FortyToFiftyPercent,
    #[serde(rename = "60_70_percent")]
    SixtyToSeventyPercent,
    #[serde(rename = "80_plus_percent")]
    EightyPlusPercent,
}
impl Effacement {
    fn points(self) -> u8 {
        match self {
            Self::ZeroToThirtyPercent => 0,
            Self::FortyToFiftyPercent => 1,
            Self::SixtyToSeventyPercent => 2,
            Self::EightyPlusPercent => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Station {
    #[serde(rename = "minus_3")]
    MinusThree,
    #[serde(rename = "minus_2")]
    MinusTwo,
    #[serde(rename = "minus_1_or_0")]
    MinusOneOrZero,
    #[serde(rename = "plus_1_or_plus_2")]
    PlusOneOrPlusTwo,
}
impl Station {
    fn points(self) -> u8 {
        match self {
            Self::MinusThree => 0,
            Self::MinusTwo => 1,
            Self::MinusOneOrZero => 2,
            Self::PlusOneOrPlusTwo => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Consistency {
    Firm,
    Medium,
    Soft,
}
impl Consistency {
    fn points(self) -> u8 {
        match self {
            Self::Firm => 0,
            Self::Medium => 1,
            Self::Soft => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Position {
    Posterior,
    MidPosition,
    Anterior,
}
impl Position {
    fn points(self) -> u8 {
        match self {
            Self::Posterior => 0,
            Self::MidPosition => 1,
            Self::Anterior => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BishopInput {
    pub dilation: Dilation,
    pub effacement: Effacement,
    pub station: Station,
    pub consistency: Consistency,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BishopOutcome {
    pub score: u8,
    pub favourable: bool,
}

pub fn compute(input: &BishopInput) -> BishopOutcome {
    let score = input.dilation.points()
        + input.effacement.points()
        + input.station.points()
        + input.consistency.points()
        + input.position.points();
    BishopOutcome {
        score,
        favourable: score >= 8,
    }
}

pub fn build_response(input: &BishopInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input);
    let mut working = Map::new();
    working.insert("favourable".into(), json!(o.favourable));

    let interpretation = if o.favourable {
        format!(
            "Bishop score {}/13: favourable cervix. A score of 8 or more is associated with a likelihood of successful induction similar to spontaneous labour.",
            o.score
        )
    } else if o.score >= 7 {
        format!(
            "Bishop score {}/13: intermediate. Scores of 6-7 sit between the favourable and unfavourable bands; local protocol and clinical context guide the induction method.",
            o.score
        )
    } else {
        format!(
            "Bishop score {}/13: unfavourable cervix. A low score is associated with a higher chance of failed induction and may prompt cervical ripening before oxytocin.",
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

pub struct Bishop;
impl Calculator for Bishop {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Bishop Score"
    }
    fn description(&self) -> &'static str {
        "Five-parameter cervical-exam score predicting the likelihood of successful labour induction."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "BishopInput", "type": "object", "additionalProperties": false,
            "required": ["dilation", "effacement", "station", "consistency", "position"],
            "properties": {
                "dilation": { "type": "string", "enum": ["closed", "1_2_cm", "3_4_cm", "5_plus_cm"] },
                "effacement": { "type": "string", "enum": ["0_30_percent", "40_50_percent", "60_70_percent", "80_plus_percent"] },
                "station": { "type": "string", "enum": ["minus_3", "minus_2", "minus_1_or_0", "plus_1_or_plus_2"], "description": "Fetal station relative to the ischial spines" },
                "consistency": { "type": "string", "enum": ["Firm", "Medium", "Soft"] },
                "position": { "type": "string", "enum": ["Posterior", "MidPosition", "Anterior"] }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: BishopInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimum_score_all_unfavourable() {
        let o = compute(&BishopInput {
            dilation: Dilation::Closed,
            effacement: Effacement::ZeroToThirtyPercent,
            station: Station::MinusThree,
            consistency: Consistency::Firm,
            position: Position::Posterior,
        });
        assert_eq!(o.score, 0);
        assert!(!o.favourable);
    }

    #[test]
    fn maximum_score_all_favourable() {
        let o = compute(&BishopInput {
            dilation: Dilation::FivePlusCm,
            effacement: Effacement::EightyPlusPercent,
            station: Station::PlusOneOrPlusTwo,
            consistency: Consistency::Soft,
            position: Position::Anterior,
        });
        assert_eq!(o.score, 13);
        assert!(o.favourable);
    }

    #[test]
    fn eight_is_favourable_threshold() {
        let o = compute(&BishopInput {
            dilation: Dilation::ThreeToFourCm,
            effacement: Effacement::SixtyToSeventyPercent,
            station: Station::MinusOneOrZero,
            consistency: Consistency::Soft,
            position: Position::Anterior,
        });
        assert_eq!(o.score, 2 + 2 + 2 + 2 + 2);
        assert!(o.favourable);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "dilation": "closed", "effacement": "0_30_percent", "station": "minus_3",
            "consistency": "Firm", "position": "Posterior"
        });
        let typed = BishopInput {
            dilation: Dilation::Closed,
            effacement: Effacement::ZeroToThirtyPercent,
            station: Station::MinusThree,
            consistency: Consistency::Firm,
            position: Position::Posterior,
        };
        let dynamic = Bishop.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            Bishop
                .calculate(&json!({"dilation": "closed", "effacement": "0_30_percent"}))
                .is_err()
        );
    }
}
