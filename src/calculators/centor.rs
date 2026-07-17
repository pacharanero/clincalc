// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Centor / McIsaac score for streptococcal pharyngitis.
//!
//! The four Centor criteria (1981) predict group-A streptococcal pharyngitis.
//! McIsaac (1998) added an age modifier to improve specificity in adults and
//! to apply the score to children. This implementation supports both the
//! original 4-point Centor score and the 5-point McIsaac-modified version.
//!
//! McIsaac age modifier:
//!  - Age 3-14: +1
//!  - Age 15-44: 0
//!  - Age 45+: -1
//!
//! Score interpretation (McIsaac):
//!  - ≤0: No antibiotic; throat culture not needed (strep probability ~2%)
//!  - 1:  No antibiotic; throat culture not needed (~6%)
//!  - 2:  Throat culture or RADT; treat if positive (~17%)
//!  - 3:  Throat culture or RADT; empiric antibiotics reasonable (~32%)
//!  - ≥4: Empiric antibiotic treatment reasonable (~56%)

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "centor";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1001/archinte.1998.00380200109013",
};

pub const REFERENCE: &str = "McIsaac WJ, White D, Tannenbaum D, Low DE. A clinical score to reduce unnecessary antibiotic use \
in patients with sore throat. CMAJ. 1998;158(1):75-83. | Centor RM et al. Ann Intern Med. 1981;94(1):31-35. \
doi:10.7326/0003-4819-94-1-31";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CentorInput {
    /// Patient age in years (required for McIsaac age modifier)
    pub age: u8,
    /// Tonsillar exudate or swelling
    pub tonsillar_exudate: bool,
    /// Tender anterior cervical lymphadenopathy
    pub tender_anterior_cervical_nodes: bool,
    /// Absence of cough (score point for NO cough)
    pub absence_of_cough: bool,
    /// Fever > 38°C (history or measured)
    pub fever: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CentorOutcome {
    pub centor_score: i8,
    pub mcisaac_score: i8,
    pub interpretation: String,
}

pub fn compute(input: &CentorInput) -> Result<CentorOutcome, CalcError> {
    let centor_score: i8 = [
        input.tonsillar_exudate,
        input.tender_anterior_cervical_nodes,
        input.absence_of_cough,
        input.fever,
    ]
    .iter()
    .map(|&b| if b { 1i8 } else { 0i8 })
    .sum();

    let age_modifier: i8 = if input.age < 3 {
        return Err(CalcError::InvalidInput(
            "Centor/McIsaac is validated in patients aged 3+".into(),
        ));
    } else if input.age <= 14 {
        1
    } else if input.age <= 44 {
        0
    } else {
        -1
    };

    let mcisaac_score = centor_score + age_modifier;

    let recommendation = match mcisaac_score {
        s if s <= 0 => "No antibiotic; throat culture not needed (strep probability ~2%)",
        1 => "No antibiotic; throat culture not needed (strep probability ~6%)",
        2 => "Throat swab or RADT; treat only if positive (strep probability ~17%)",
        3 => "Throat swab or RADT; empiric antibiotics may be reasonable (strep probability ~32%)",
        _ => "Empiric antibiotic treatment is reasonable (strep probability ~56%)",
    };

    let interpretation = format!(
        "Centor score {centor_score}/4; McIsaac score {mcisaac_score} (age modifier {age_modifier:+}). \
{recommendation}."
    );

    Ok(CentorOutcome {
        centor_score,
        mcisaac_score,
        interpretation,
    })
}

pub fn build_response(input: &CentorInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("centor_score".into(), json!(o.centor_score));
    working.insert("mcisaac_score".into(), json!(o.mcisaac_score));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.mcisaac_score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Centor;

impl Calculator for Centor {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Centor / McIsaac Score (Strep Pharyngitis)"
    }

    fn description(&self) -> &'static str {
        "Predicts probability of group-A streptococcal pharyngitis to guide antibiotic use and throat swab decisions."
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
            "title": "CentorInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["age", "tonsillar_exudate", "tender_anterior_cervical_nodes", "absence_of_cough", "fever"],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 3,
                    "maximum": 120,
                    "description": "Patient age in years (McIsaac age modifier applied: +1 if 3-14, 0 if 15-44, -1 if 45+)"
                },
                "tonsillar_exudate": {
                    "type": "boolean",
                    "description": "Tonsillar exudate or swelling present"
                },
                "tender_anterior_cervical_nodes": {
                    "type": "boolean",
                    "description": "Tender anterior cervical lymphadenopathy"
                },
                "absence_of_cough": {
                    "type": "boolean",
                    "description": "Absence of cough (true = no cough present = score point)"
                },
                "fever": {
                    "type": "boolean",
                    "description": "History of fever or measured temperature > 38 degrees C"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: CentorInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(age: u8, exudate: bool, nodes: bool, no_cough: bool, fever: bool) -> CentorInput {
        CentorInput {
            age,
            tonsillar_exudate: exudate,
            tender_anterior_cervical_nodes: nodes,
            absence_of_cough: no_cough,
            fever,
        }
    }

    #[test]
    fn classic_high_risk_adult() {
        // 25yo, all 4 criteria: Centor 4, McIsaac +0 = 4
        let o = compute(&inp(25, true, true, true, true)).unwrap();
        assert_eq!(o.centor_score, 4);
        assert_eq!(o.mcisaac_score, 4);
        assert!(o.interpretation.contains("Empiric antibiotic"));
    }

    #[test]
    fn child_age_modifier_plus_one() {
        // 10yo, 3 criteria: Centor 3 + age +1 = McIsaac 4
        let o = compute(&inp(10, true, true, true, false)).unwrap();
        assert_eq!(o.centor_score, 3);
        assert_eq!(o.mcisaac_score, 4);
    }

    #[test]
    fn older_adult_age_modifier_minus_one() {
        // 50yo, 3 criteria: Centor 3 + age -1 = McIsaac 2
        let o = compute(&inp(50, true, true, false, true)).unwrap();
        assert_eq!(o.centor_score, 3);
        assert_eq!(o.mcisaac_score, 2);
        assert!(o.interpretation.contains("RADT"));
    }

    #[test]
    fn low_risk_no_criteria() {
        let o = compute(&inp(30, false, false, false, false)).unwrap();
        assert_eq!(o.centor_score, 0);
        assert_eq!(o.mcisaac_score, 0);
        assert!(o.interpretation.contains("No antibiotic"));
    }

    #[test]
    fn rejects_age_below_3() {
        assert!(compute(&inp(2, true, true, true, true)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age": 25,
            "tonsillar_exudate": true,
            "tender_anterior_cervical_nodes": true,
            "absence_of_cough": true,
            "fever": true
        });
        let dynamic = Centor.calculate(&value).unwrap();
        let typed = build_response(&inp(25, true, true, true, true)).unwrap();
        assert_eq!(dynamic, typed);
    }
}
