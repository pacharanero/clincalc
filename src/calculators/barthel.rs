// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Barthel Index of Activities of Daily Living.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "barthel";
pub const REFERENCE: &str = "Mahoney FI, Barthel DW. Functional evaluation: the Barthel Index. Md State Med J. 1965;14:61-65.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://pubmed.ncbi.nlm.nih.gov/14258950/",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feeding {
    Unable,
    NeedsHelp,
    Independent,
}
impl Feeding {
    fn points(self) -> u8 {
        match self {
            Feeding::Unable => 0,
            Feeding::NeedsHelp => 5,
            Feeding::Independent => 10,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bathing {
    Dependent,
    Independent,
}
impl Bathing {
    fn points(self) -> u8 {
        match self {
            Bathing::Dependent => 0,
            Bathing::Independent => 5,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grooming {
    NeedsHelp,
    Independent,
}
impl Grooming {
    fn points(self) -> u8 {
        match self {
            Grooming::NeedsHelp => 0,
            Grooming::Independent => 5,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dressing {
    Dependent,
    NeedsHelp,
    Independent,
}
impl Dressing {
    fn points(self) -> u8 {
        match self {
            Dressing::Dependent => 0,
            Dressing::NeedsHelp => 5,
            Dressing::Independent => 10,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BowelsBladder {
    Incontinent,
    OccasionalAccident,
    Continent,
}
impl BowelsBladder {
    fn points(self) -> u8 {
        match self {
            BowelsBladder::Incontinent => 0,
            BowelsBladder::OccasionalAccident => 5,
            BowelsBladder::Continent => 10,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Toilet {
    Dependent,
    NeedsHelp,
    Independent,
}
impl Toilet {
    fn points(self) -> u8 {
        match self {
            Toilet::Dependent => 0,
            Toilet::NeedsHelp => 5,
            Toilet::Independent => 10,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transfer {
    Unable,
    MajorHelp,
    MinorHelp,
    Independent,
}
impl Transfer {
    fn points(self) -> u8 {
        match self {
            Transfer::Unable => 0,
            Transfer::MajorHelp => 5,
            Transfer::MinorHelp => 10,
            Transfer::Independent => 15,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mobility {
    Immobile,
    WheelchairIndependent,
    WalksWithHelp,
    Independent,
}
impl Mobility {
    fn points(self) -> u8 {
        match self {
            Mobility::Immobile => 0,
            Mobility::WheelchairIndependent => 5,
            Mobility::WalksWithHelp => 10,
            Mobility::Independent => 15,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stairs {
    Unable,
    NeedsHelp,
    Independent,
}
impl Stairs {
    fn points(self) -> u8 {
        match self {
            Stairs::Unable => 0,
            Stairs::NeedsHelp => 5,
            Stairs::Independent => 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BarthelInput {
    pub feeding: Feeding,
    pub bathing: Bathing,
    pub grooming: Grooming,
    pub dressing: Dressing,
    pub bowels: BowelsBladder,
    pub bladder: BowelsBladder,
    pub toilet_use: Toilet,
    pub transfers: Transfer,
    pub mobility: Mobility,
    pub stairs: Stairs,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarthelOutcome {
    pub score: u8,
    pub interpretation: String,
}

pub fn compute(i: &BarthelInput) -> Result<BarthelOutcome, CalcError> {
    let score = i.feeding.points()
        + i.bathing.points()
        + i.grooming.points()
        + i.dressing.points()
        + i.bowels.points()
        + i.bladder.points()
        + i.toilet_use.points()
        + i.transfers.points()
        + i.mobility.points()
        + i.stairs.points();
    let band = if score < 20 {
        "total dependency"
    } else if score < 60 {
        "severe dependency"
    } else if score < 90 {
        "moderate dependency"
    } else if score < 100 {
        "slight dependency"
    } else {
        "independent in measured ADLs"
    };
    let interpretation = format!(
        "Barthel Index {score}/100: {band}. The index measures observed function in basic activities of daily living; it is not a diagnosis and should be interpreted in context."
    );
    Ok(BarthelOutcome {
        score,
        interpretation,
    })
}

pub fn build_response(i: &BarthelInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(i)?;
    let mut working = Map::new();
    working.insert("feeding".into(), json!(i.feeding.points()));
    working.insert("bathing".into(), json!(i.bathing.points()));
    working.insert("grooming".into(), json!(i.grooming.points()));
    working.insert("dressing".into(), json!(i.dressing.points()));
    working.insert("bowels".into(), json!(i.bowels.points()));
    working.insert("bladder".into(), json!(i.bladder.points()));
    working.insert("toilet_use".into(), json!(i.toilet_use.points()));
    working.insert("transfers".into(), json!(i.transfers.points()));
    working.insert("mobility".into(), json!(i.mobility.points()));
    working.insert("stairs".into(), json!(i.stairs.points()));
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Barthel;
impl Calculator for Barthel {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Barthel Index"
    }
    fn description(&self) -> &'static str {
        "Activities of daily living score (0-100) from ten functional domains."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({ "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "BarthelInput", "type": "object", "additionalProperties": false, "required": ["feeding", "bathing", "grooming", "dressing", "bowels", "bladder", "toilet_use", "transfers", "mobility", "stairs"], "properties": { "feeding": { "type": "string", "enum": ["unable", "needs_help", "independent"] }, "bathing": { "type": "string", "enum": ["dependent", "independent"] }, "grooming": { "type": "string", "enum": ["needs_help", "independent"] }, "dressing": { "type": "string", "enum": ["dependent", "needs_help", "independent"] }, "bowels": { "type": "string", "enum": ["incontinent", "occasional_accident", "continent"] }, "bladder": { "type": "string", "enum": ["incontinent", "occasional_accident", "continent"] }, "toilet_use": { "type": "string", "enum": ["dependent", "needs_help", "independent"] }, "transfers": { "type": "string", "enum": ["unable", "major_help", "minor_help", "independent"] }, "mobility": { "type": "string", "enum": ["immobile", "wheelchair_independent", "walks_with_help", "independent"] }, "stairs": { "type": "string", "enum": ["unable", "needs_help", "independent"] } } })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: BarthelInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fully_independent_scores_100() {
        let out = compute(&BarthelInput {
            feeding: Feeding::Independent,
            bathing: Bathing::Independent,
            grooming: Grooming::Independent,
            dressing: Dressing::Independent,
            bowels: BowelsBladder::Continent,
            bladder: BowelsBladder::Continent,
            toilet_use: Toilet::Independent,
            transfers: Transfer::Independent,
            mobility: Mobility::Independent,
            stairs: Stairs::Independent,
        })
        .unwrap();
        assert_eq!(out.score, 100);
    }
    #[test]
    fn all_dependent_scores_zero() {
        let out = compute(&BarthelInput {
            feeding: Feeding::Unable,
            bathing: Bathing::Dependent,
            grooming: Grooming::NeedsHelp,
            dressing: Dressing::Dependent,
            bowels: BowelsBladder::Incontinent,
            bladder: BowelsBladder::Incontinent,
            toilet_use: Toilet::Dependent,
            transfers: Transfer::Unable,
            mobility: Mobility::Immobile,
            stairs: Stairs::Unable,
        })
        .unwrap();
        assert_eq!(out.score, 0);
    }
}
