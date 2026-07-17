// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! FINDRISC - Finnish Diabetes Risk Score.
//!
//! Eight-item questionnaire predicting 10-year risk of type 2 diabetes.
//! Developed and validated by Lindstrom & Tuomilehto (2003).
//!
//! Items and points:
//!  1. Age: <45=0, 45-54=2, 55-64=3, ≥65=4
//!  2. BMI: <25=0, 25-30=1, >30=3
//!  3. Waist circumference (sex-specific):
//!     Men:   <94 cm=0, 94-102=3, >102=4
//!     Women: <80 cm=0, 80-88=3, >88=4
//!  4. Physical activity ≥30 min/day: yes=0, no=2
//!  5. Daily fruit/vegetables/berries: yes=0, no=1
//!  6. Antihypertensive medication: no=0, yes=2
//!  7. History of high blood glucose: no=0, yes=5
//!  8. Family history of diabetes: none=0, 2nd-degree only=3, 1st-degree or both=5
//!
//! Score 0-26, five risk bands.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "findrisc";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.2337/diacare.26.3.725",
};

pub const REFERENCE: &str = "Lindstrom J, Tuomilehto J. The diabetes risk score: a practical tool to predict \
type 2 diabetes risk. Diabetes Care. 2003;26(3):725-731. doi:10.2337/diacare.26.3.725";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FamilyHistory {
    /// No family history (0 points)
    None,
    /// Second-degree relative only (grandparent, aunt/uncle, cousin) (3 points)
    SecondDegree,
    /// First-degree relative (parent, sibling, child) or both first and second degree (5 points)
    FirstDegree,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindriscInput {
    pub sex: Sex,
    pub age: u8,
    /// BMI (kg/m²)
    pub bmi: f64,
    /// Waist circumference in cm
    pub waist_cm: f64,
    /// Physically active ≥30 minutes per day (moderate activity)
    pub physically_active: bool,
    /// Eats vegetables, fruit, or berries every day
    pub daily_vegetables_fruit: bool,
    /// Currently taking antihypertensive medication
    pub antihypertensive_medication: bool,
    /// History of high blood glucose (incl. during pregnancy)
    pub high_blood_glucose_history: bool,
    pub family_history: FamilyHistory,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FindriscOutcome {
    pub score: u8,
    pub risk_band: &'static str,
    pub ten_year_risk: &'static str,
    pub interpretation: String,
}

pub fn compute(input: &FindriscInput) -> Result<FindriscOutcome, CalcError> {
    if input.bmi <= 0.0 || !input.bmi.is_finite() {
        return Err(CalcError::InvalidInput("bmi must be positive".into()));
    }
    if input.waist_cm <= 0.0 || !input.waist_cm.is_finite() {
        return Err(CalcError::InvalidInput("waist_cm must be positive".into()));
    }

    let age_pts: u8 = if input.age < 45 {
        0
    } else if input.age < 55 {
        2
    } else if input.age < 65 {
        3
    } else {
        4
    };

    let bmi_pts: u8 = if input.bmi < 25.0 {
        0
    } else if input.bmi <= 30.0 {
        1
    } else {
        3
    };

    let waist_pts: u8 = match input.sex {
        Sex::Male => {
            if input.waist_cm < 94.0 {
                0
            } else if input.waist_cm <= 102.0 {
                3
            } else {
                4
            }
        }
        Sex::Female => {
            if input.waist_cm < 80.0 {
                0
            } else if input.waist_cm <= 88.0 {
                3
            } else {
                4
            }
        }
    };

    let activity_pts: u8 = if input.physically_active { 0 } else { 2 };
    let diet_pts: u8 = if input.daily_vegetables_fruit { 0 } else { 1 };
    let antihyp_pts: u8 = if input.antihypertensive_medication {
        2
    } else {
        0
    };
    let glucose_pts: u8 = if input.high_blood_glucose_history {
        5
    } else {
        0
    };
    let family_pts: u8 = match input.family_history {
        FamilyHistory::None => 0,
        FamilyHistory::SecondDegree => 3,
        FamilyHistory::FirstDegree => 5,
    };

    let score = age_pts
        + bmi_pts
        + waist_pts
        + activity_pts
        + diet_pts
        + antihyp_pts
        + glucose_pts
        + family_pts;

    let (risk_band, ten_year_risk) = match score {
        0..=7 => ("Low", "approximately 1 in 100"),
        8..=11 => ("Slightly elevated", "approximately 1 in 25"),
        12..=14 => ("Moderate", "approximately 1 in 6"),
        15..=20 => ("High", "approximately 1 in 3"),
        _ => ("Very high", "approximately 1 in 2"),
    };

    let interpretation = format!(
        "FINDRISC score {score}/26 - {risk_band} risk of developing type 2 diabetes \
within 10 years ({ten_year_risk}). Lifestyle interventions (weight loss, increased \
physical activity, dietary change) can substantially reduce risk in moderate and above categories."
    );

    Ok(FindriscOutcome {
        score,
        risk_band,
        ten_year_risk,
        interpretation,
    })
}

pub fn build_response(input: &FindriscInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("findrisc_score".into(), json!(o.score));
    working.insert("risk_band".into(), json!(o.risk_band));
    working.insert("ten_year_risk".into(), json!(o.ten_year_risk));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Findrisc;

impl Calculator for Findrisc {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "FINDRISC (Finnish Diabetes Risk Score)"
    }

    fn description(&self) -> &'static str {
        "Predicts 10-year risk of type 2 diabetes using 8 lifestyle and clinical items (score 0-26)."
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
            "title": "FindriscInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["sex", "age", "bmi", "waist_cm", "physically_active",
                         "daily_vegetables_fruit", "antihypertensive_medication",
                         "high_blood_glucose_history", "family_history"],
            "properties": {
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Sex (used for sex-specific waist circumference thresholds)"
                },
                "age": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 120,
                    "description": "Age in years (<45=0, 45-54=2, 55-64=3, ≥65=4 points)"
                },
                "bmi": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Body mass index in kg/m² (<25=0, 25-30=1, >30=3 points)"
                },
                "waist_cm": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Waist circumference in cm (sex-specific thresholds apply)"
                },
                "physically_active": {
                    "type": "boolean",
                    "description": "At least 30 minutes of moderate physical activity daily (true=0, false=2 points)"
                },
                "daily_vegetables_fruit": {
                    "type": "boolean",
                    "description": "Eats vegetables, fruit, or berries every day (true=0, false=1 point)"
                },
                "antihypertensive_medication": {
                    "type": "boolean",
                    "description": "Currently taking antihypertensive medication (false=0, true=2 points)"
                },
                "high_blood_glucose_history": {
                    "type": "boolean",
                    "description": "History of high blood glucose (including during pregnancy) (false=0, true=5 points)"
                },
                "family_history": {
                    "type": "string",
                    "enum": ["none", "second_degree", "first_degree"],
                    "description": "Family history of diabetes: none=0, second_degree=3, first_degree=5 points"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: FindriscInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> FindriscInput {
        FindriscInput {
            sex: Sex::Male,
            age: 40,
            bmi: 22.0,
            waist_cm: 85.0,
            physically_active: true,
            daily_vegetables_fruit: true,
            antihypertensive_medication: false,
            high_blood_glucose_history: false,
            family_history: FamilyHistory::None,
        }
    }

    #[test]
    fn low_risk_young_healthy() {
        let o = compute(&base()).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.risk_band, "Low");
    }

    #[test]
    fn high_risk_scenario() {
        // age≥65 (4) + bmi>30 (3) + waist>102 male (4) + inactive (2) + no veg (1) + antihyp (2) + glucose hx (5) = 21
        let o = compute(&FindriscInput {
            sex: Sex::Male,
            age: 67,
            bmi: 32.0,
            waist_cm: 110.0,
            physically_active: false,
            daily_vegetables_fruit: false,
            antihypertensive_medication: true,
            high_blood_glucose_history: true,
            family_history: FamilyHistory::None,
        })
        .unwrap();
        assert_eq!(o.score, 21);
        assert_eq!(o.risk_band, "Very high");
    }

    #[test]
    fn female_waist_thresholds() {
        // Female waist 82 cm -> 3 pts (80-88 band)
        let o = compute(&FindriscInput {
            sex: Sex::Female,
            waist_cm: 82.0,
            ..base()
        })
        .unwrap();
        assert_eq!(o.score, 3);
    }

    #[test]
    fn family_history_first_degree() {
        let o = compute(&FindriscInput {
            family_history: FamilyHistory::FirstDegree,
            ..base()
        })
        .unwrap();
        assert_eq!(o.score, 5);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let b = base();
        let value = json!({
            "sex": "male",
            "age": 40,
            "bmi": 22.0,
            "waist_cm": 85.0,
            "physically_active": true,
            "daily_vegetables_fruit": true,
            "antihypertensive_medication": false,
            "high_blood_glucose_history": false,
            "family_history": "none"
        });
        let dynamic = Findrisc.calculate(&value).unwrap();
        let typed = build_response(&b).unwrap();
        assert_eq!(dynamic, typed);
    }
}
