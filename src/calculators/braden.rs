// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Braden Scale for Predicting Pressure Ulcer Risk.
//!
//! Six subscales scored as follows:
//!  - Sensory perception: 1 (completely limited) to 4 (no impairment)
//!  - Moisture:           1 (constantly moist)   to 4 (rarely moist)
//!  - Activity:           1 (bedfast)             to 4 (walks frequently)
//!  - Mobility:           1 (completely immobile) to 4 (no limitations)
//!  - Nutrition:          1 (very poor)           to 4 (excellent)
//!  - Friction/shear:     1 (problem)             to 3 (no apparent problem)
//!
//! Total range: 6-23 (lower = higher risk).
//!
//! Risk bands (Bergstrom et al.):
//!  - ≤9:   Very high risk
//!  - 10-12: High risk
//!  - 13-14: Moderate risk
//!  - 15-18: Mild risk
//!  - ≥19:  Low / no risk

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "braden";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1097/00006199-198709000-00015",
};

pub const REFERENCE: &str = "Bergstrom N, Braden BJ, Laguzza A, Holman V. The Braden Scale for predicting \
pressure sore risk. Nurs Res. 1987;36(4):205-210. doi:10.1097/00006199-198709000-00015";

/// Sensory perception subscale (1-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensoryPerception {
    /// Completely limited (1)
    CompletelyLimited,
    /// Very limited (2)
    VeryLimited,
    /// Slightly limited (3)
    SlightlyLimited,
    /// No impairment (4)
    NoImpairment,
}

/// Moisture subscale (1-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Moisture {
    /// Constantly moist (1)
    ConstantlyMoist,
    /// Often moist (2)
    OftenMoist,
    /// Occasionally moist (3)
    OccasionallyMoist,
    /// Rarely moist (4)
    RarelyMoist,
}

/// Activity subscale (1-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activity {
    /// Bedfast (1)
    Bedfast,
    /// Chairfast (2)
    Chairfast,
    /// Walks occasionally (3)
    WalksOccasionally,
    /// Walks frequently (4)
    WalksFrequently,
}

/// Mobility subscale (1-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mobility {
    /// Completely immobile (1)
    CompletelyImmobile,
    /// Very limited (2)
    VeryLimited,
    /// Slightly limited (3)
    SlightlyLimited,
    /// No limitations (4)
    NoLimitations,
}

/// Nutrition subscale (1-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Nutrition {
    /// Very poor (1)
    VeryPoor,
    /// Probably inadequate (2)
    ProbablyInadequate,
    /// Adequate (3)
    Adequate,
    /// Excellent (4)
    Excellent,
}

/// Friction and shear subscale (1-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrictionShear {
    /// Problem (1)
    Problem,
    /// Potential problem (2)
    PotentialProblem,
    /// No apparent problem (3)
    NoApparentProblem,
}

fn sensory_score(v: SensoryPerception) -> u8 {
    match v {
        SensoryPerception::CompletelyLimited => 1,
        SensoryPerception::VeryLimited => 2,
        SensoryPerception::SlightlyLimited => 3,
        SensoryPerception::NoImpairment => 4,
    }
}

fn moisture_score(v: Moisture) -> u8 {
    match v {
        Moisture::ConstantlyMoist => 1,
        Moisture::OftenMoist => 2,
        Moisture::OccasionallyMoist => 3,
        Moisture::RarelyMoist => 4,
    }
}

fn activity_score(v: Activity) -> u8 {
    match v {
        Activity::Bedfast => 1,
        Activity::Chairfast => 2,
        Activity::WalksOccasionally => 3,
        Activity::WalksFrequently => 4,
    }
}

fn mobility_score(v: Mobility) -> u8 {
    match v {
        Mobility::CompletelyImmobile => 1,
        Mobility::VeryLimited => 2,
        Mobility::SlightlyLimited => 3,
        Mobility::NoLimitations => 4,
    }
}

fn nutrition_score(v: Nutrition) -> u8 {
    match v {
        Nutrition::VeryPoor => 1,
        Nutrition::ProbablyInadequate => 2,
        Nutrition::Adequate => 3,
        Nutrition::Excellent => 4,
    }
}

fn friction_score(v: FrictionShear) -> u8 {
    match v {
        FrictionShear::Problem => 1,
        FrictionShear::PotentialProblem => 2,
        FrictionShear::NoApparentProblem => 3,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BradenInput {
    pub sensory_perception: SensoryPerception,
    pub moisture: Moisture,
    pub activity: Activity,
    pub mobility: Mobility,
    pub nutrition: Nutrition,
    pub friction_shear: FrictionShear,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BradenOutcome {
    pub score: u8,
    pub risk_level: &'static str,
    pub interpretation: String,
}

pub fn compute(input: &BradenInput) -> Result<BradenOutcome, CalcError> {
    let score = sensory_score(input.sensory_perception)
        + moisture_score(input.moisture)
        + activity_score(input.activity)
        + mobility_score(input.mobility)
        + nutrition_score(input.nutrition)
        + friction_score(input.friction_shear);

    let risk_level = match score {
        6..=9 => "Very high risk",
        10..=12 => "High risk",
        13..=14 => "Moderate risk",
        15..=18 => "Mild risk",
        _ => "Low / no risk",
    };

    let recommendation = match score {
        6..=9 => {
            "Intensive prevention protocol: frequent repositioning (q1-2h), \
pressure-redistributing mattress, barrier creams, nutritional support."
        }
        10..=12 => {
            "High-risk prevention: repositioning q2h, pressure-relief surface, \
skin inspection at each care episode."
        }
        13..=14 => {
            "Prevention measures indicated: regular repositioning, skin inspection, \
optimise nutrition and hydration."
        }
        15..=18 => "Standard preventive care; monitor for changes in risk factors.",
        _ => "Routine skin care; reassess if condition changes.",
    };

    let interpretation = format!("Braden score {score}/23 - {risk_level}. {recommendation}");

    Ok(BradenOutcome {
        score,
        risk_level,
        interpretation,
    })
}

pub fn build_response(input: &BradenInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("braden_score".into(), json!(o.score));
    working.insert("risk_level".into(), json!(o.risk_level));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Braden;

impl Calculator for Braden {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Braden Scale (Pressure Ulcer Risk)"
    }

    fn description(&self) -> &'static str {
        "Predicts pressure ulcer risk across six subscales (sensory perception, moisture, activity, mobility, nutrition, friction/shear). Score 6-23; lower = higher risk."
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
            "title": "BradenInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["sensory_perception", "moisture", "activity", "mobility", "nutrition", "friction_shear"],
            "properties": {
                "sensory_perception": {
                    "type": "string",
                    "enum": ["completely_limited", "very_limited", "slightly_limited", "no_impairment"],
                    "description": "Ability to respond to pressure-related discomfort (1=completely_limited, 4=no_impairment)"
                },
                "moisture": {
                    "type": "string",
                    "enum": ["constantly_moist", "often_moist", "occasionally_moist", "rarely_moist"],
                    "description": "Degree to which skin is exposed to moisture (1=constantly_moist, 4=rarely_moist)"
                },
                "activity": {
                    "type": "string",
                    "enum": ["bedfast", "chairfast", "walks_occasionally", "walks_frequently"],
                    "description": "Degree of physical activity (1=bedfast, 4=walks_frequently)"
                },
                "mobility": {
                    "type": "string",
                    "enum": ["completely_immobile", "very_limited", "slightly_limited", "no_limitations"],
                    "description": "Ability to change and control body position (1=completely_immobile, 4=no_limitations)"
                },
                "nutrition": {
                    "type": "string",
                    "enum": ["very_poor", "probably_inadequate", "adequate", "excellent"],
                    "description": "Usual food intake pattern (1=very_poor, 4=excellent)"
                },
                "friction_shear": {
                    "type": "string",
                    "enum": ["problem", "potential_problem", "no_apparent_problem"],
                    "description": "Friction and shear risk (1=problem, 3=no_apparent_problem)"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: BradenInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn worst() -> BradenInput {
        BradenInput {
            sensory_perception: SensoryPerception::CompletelyLimited,
            moisture: Moisture::ConstantlyMoist,
            activity: Activity::Bedfast,
            mobility: Mobility::CompletelyImmobile,
            nutrition: Nutrition::VeryPoor,
            friction_shear: FrictionShear::Problem,
        }
    }

    fn best() -> BradenInput {
        BradenInput {
            sensory_perception: SensoryPerception::NoImpairment,
            moisture: Moisture::RarelyMoist,
            activity: Activity::WalksFrequently,
            mobility: Mobility::NoLimitations,
            nutrition: Nutrition::Excellent,
            friction_shear: FrictionShear::NoApparentProblem,
        }
    }

    #[test]
    fn minimum_score_is_6() {
        let o = compute(&worst()).unwrap();
        assert_eq!(o.score, 6);
        assert_eq!(o.risk_level, "Very high risk");
    }

    #[test]
    fn maximum_score_is_23() {
        let o = compute(&best()).unwrap();
        assert_eq!(o.score, 23);
        assert_eq!(o.risk_level, "Low / no risk");
    }

    #[test]
    fn moderate_risk_band() {
        let o = compute(&BradenInput {
            sensory_perception: SensoryPerception::SlightlyLimited,
            moisture: Moisture::OccasionallyMoist,
            activity: Activity::Chairfast,
            mobility: Mobility::VeryLimited,
            nutrition: Nutrition::ProbablyInadequate,
            friction_shear: FrictionShear::PotentialProblem,
        })
        .unwrap();
        // 3+3+2+2+2+2 = 14
        assert_eq!(o.score, 14);
        assert_eq!(o.risk_level, "Moderate risk");
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "sensory_perception": "no_impairment",
            "moisture": "rarely_moist",
            "activity": "walks_frequently",
            "mobility": "no_limitations",
            "nutrition": "excellent",
            "friction_shear": "no_apparent_problem"
        });
        let dynamic = Braden.calculate(&value).unwrap();
        let typed = build_response(&best()).unwrap();
        assert_eq!(dynamic, typed);
    }
}
