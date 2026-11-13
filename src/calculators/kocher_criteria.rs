// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Kocher Criteria - four-criteria clinical prediction rule distinguishing
//! septic arthritis from transient synovitis of the hip in children
//! (Kocher, Zurakowski, Kasser. J Bone Joint Surg Am. 1999).
//!
//! The predicted probability of septic arthritis by number of criteria met
//! is taken from the original derivation study's simplified algorithm
//! (Table V); the subsequent 2004 validation study found materially lower
//! probabilities for the same criterion counts in a new population, which
//! is noted in the interpretation rather than substituted silently, since
//! clinicians using this rule should be aware that predictive performance
//! varies between populations.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "kocher_criteria";

pub const REFERENCE: &str = "Kocher MS, Zurakowski D, Kasser JR. Differentiating between septic arthritis and \
transient synovitis of the hip in children: an evidence-based clinical prediction algorithm. J Bone Joint \
Surg Am. 1999;81(12):1662-1670.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.2106/00004623-199912000-00002",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KocherCriteriaInput {
    pub non_weight_bearing: bool,
    /// History of fever of 38.5C (101.3F) or higher.
    pub history_of_fever_over_38_5c: bool,
    pub esr_over_40_mm_hr: bool,
    pub wbc_over_12000_per_mm3: bool,
}

/// Predicted probability of septic arthritis by number of criteria met
/// (0-4), from the original 1999 derivation study (Table V).
fn predicted_probability_percent(criteria_met: u8) -> f64 {
    match criteria_met {
        0 => 0.2,
        1 => 3.0,
        2 => 40.0,
        3 => 93.1,
        4 => 99.6,
        _ => unreachable!("criteria_met is at most 4"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KocherCriteriaOutcome {
    pub criteria_met: u8,
    pub predicted_probability_percent: f64,
}

pub fn compute(input: &KocherCriteriaInput) -> KocherCriteriaOutcome {
    let criteria_met = u8::from(input.non_weight_bearing)
        + u8::from(input.history_of_fever_over_38_5c)
        + u8::from(input.esr_over_40_mm_hr)
        + u8::from(input.wbc_over_12000_per_mm3);
    KocherCriteriaOutcome {
        criteria_met,
        predicted_probability_percent: predicted_probability_percent(criteria_met),
    }
}

pub fn build_response(input: &KocherCriteriaInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input);
    let mut working = Map::new();
    working.insert("criteria_met".into(), json!(o.criteria_met));
    working.insert(
        "predicted_probability_percent".into(),
        json!(o.predicted_probability_percent),
    );

    let interpretation = format!(
        "{}/4 criteria met (non-weight-bearing, fever >=38.5C, ESR >40 mm/hr, WBC >12,000/mm3): predicted probability of septic arthritis {:.1}% per the original 1999 derivation study. A 2004 validation study in a new population found lower probabilities for the same criterion counts (2%, 9.5%, 35%, 72.8%, and 93% for 0-4 criteria respectively), so treat this as one input alongside clinical judgement and, where available, joint aspiration rather than a standalone rule-in/rule-out threshold.",
        o.criteria_met, o.predicted_probability_percent
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.criteria_met),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct KocherCriteria;
impl Calculator for KocherCriteria {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Kocher Criteria (Paediatric Septic Arthritis)"
    }
    fn description(&self) -> &'static str {
        "Four-criteria clinical prediction rule distinguishing septic arthritis from transient synovitis of the hip in children."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "KocherCriteriaInput", "type": "object",
            "additionalProperties": false,
            "required": ["non_weight_bearing", "history_of_fever_over_38_5c", "esr_over_40_mm_hr", "wbc_over_12000_per_mm3"],
            "properties": {
                "non_weight_bearing": { "type": "boolean", "description": "Unable or unwilling to bear weight on the affected side" },
                "history_of_fever_over_38_5c": { "type": "boolean", "description": "History of fever of 38.5C (101.3F) or higher" },
                "esr_over_40_mm_hr": { "type": "boolean", "description": "Erythrocyte sedimentation rate over 40 mm/hr" },
                "wbc_over_12000_per_mm3": { "type": "boolean", "description": "Serum white cell count over 12,000/mm3 (12.0 x 10^9/L)" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: KocherCriteriaInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> KocherCriteriaInput {
        KocherCriteriaInput {
            non_weight_bearing: false,
            history_of_fever_over_38_5c: false,
            esr_over_40_mm_hr: false,
            wbc_over_12000_per_mm3: false,
        }
    }

    #[test]
    fn zero_criteria_is_lowest_probability() {
        let o = compute(&none());
        assert_eq!(o.criteria_met, 0);
        assert_eq!(o.predicted_probability_percent, 0.2);
    }

    #[test]
    fn two_criteria_is_forty_percent() {
        let mut i = none();
        i.non_weight_bearing = true;
        i.esr_over_40_mm_hr = true;
        let o = compute(&i);
        assert_eq!(o.criteria_met, 2);
        assert_eq!(o.predicted_probability_percent, 40.0);
    }

    #[test]
    fn four_criteria_is_highest_probability() {
        let i = KocherCriteriaInput {
            non_weight_bearing: true,
            history_of_fever_over_38_5c: true,
            esr_over_40_mm_hr: true,
            wbc_over_12000_per_mm3: true,
        };
        let o = compute(&i);
        assert_eq!(o.criteria_met, 4);
        assert_eq!(o.predicted_probability_percent, 99.6);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let mut typed = none();
        typed.history_of_fever_over_38_5c = true;
        typed.wbc_over_12000_per_mm3 = true;
        let value = json!({
            "non_weight_bearing": false, "history_of_fever_over_38_5c": true,
            "esr_over_40_mm_hr": false, "wbc_over_12000_per_mm3": true
        });
        let dynamic = KocherCriteria.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(2));
    }

    #[test]
    fn rejects_missing_field() {
        assert!(
            KocherCriteria
                .calculate(&json!({"non_weight_bearing": true}))
                .is_err()
        );
    }
}
