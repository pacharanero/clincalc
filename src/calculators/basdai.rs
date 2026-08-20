// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! BASDAI - Bath Ankylosing Spondylitis Disease Activity Index.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "basdai";
pub const REFERENCE: &str = "Garrett S, Jenkinson T, Kennedy LG, Whitelock H, Gaisford P, Calin A. A new approach to defining disease status in ankylosing spondylitis: the Bath Ankylosing Spondylitis Disease Activity Index. J Rheumatol. 1994;21(12):2286-2291.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published clinical index - implemented from the primary literature",
    source_url: "https://pubmed.ncbi.nlm.nih.gov/7699630/",
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasdaiInput {
    pub fatigue: f64,
    pub spinal_pain: f64,
    pub peripheral_joint_pain_swelling: f64,
    pub enthesitis: f64,
    pub morning_stiffness_severity: f64,
    pub morning_stiffness_duration: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BasdaiOutcome {
    pub score: f64,
    pub interpretation: String,
}

pub fn compute(i: &BasdaiInput) -> Result<BasdaiOutcome, CalcError> {
    for (name, value) in [
        ("fatigue", i.fatigue),
        ("spinal_pain", i.spinal_pain),
        (
            "peripheral_joint_pain_swelling",
            i.peripheral_joint_pain_swelling,
        ),
        ("enthesitis", i.enthesitis),
        ("morning_stiffness_severity", i.morning_stiffness_severity),
        ("morning_stiffness_duration", i.morning_stiffness_duration),
    ] {
        if !(0.0..=10.0).contains(&value) || !value.is_finite() {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be a finite 0-10 score"
            )));
        }
    }
    let stiffness = (i.morning_stiffness_severity + i.morning_stiffness_duration) / 2.0;
    let score =
        (i.fatigue + i.spinal_pain + i.peripheral_joint_pain_swelling + i.enthesitis + stiffness)
            / 5.0;
    let interpretation = if score >= 4.0 {
        format!(
            "BASDAI {score:.1}/10: active disease by the commonly used >=4 threshold. Interpret alongside function, inflammation markers, imaging, and treatment context."
        )
    } else {
        format!(
            "BASDAI {score:.1}/10: below the commonly used >=4 active-disease threshold. Interpret alongside function, inflammation markers, imaging, and treatment context."
        )
    };
    Ok(BasdaiOutcome {
        score,
        interpretation,
    })
}

pub fn build_response(i: &BasdaiInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(i)?;
    let mut working = Map::new();
    working.insert(
        "morning_stiffness_component".into(),
        json!(round1(
            (i.morning_stiffness_severity + i.morning_stiffness_duration) / 2.0
        )),
    );
    working.insert("active_disease_threshold".into(), json!(4.0));
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.score)),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}
fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct Basdai;
impl Calculator for Basdai {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "BASDAI"
    }
    fn description(&self) -> &'static str {
        "Bath Ankylosing Spondylitis Disease Activity Index (0-10)."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        let q = |desc: &str| json!({ "type": "number", "minimum": 0, "maximum": 10, "description": desc });
        json!({ "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "BasdaiInput", "type": "object", "additionalProperties": false, "required": ["fatigue", "spinal_pain", "peripheral_joint_pain_swelling", "enthesitis", "morning_stiffness_severity", "morning_stiffness_duration"], "properties": { "fatigue": q("Fatigue/tiredness 0-10"), "spinal_pain": q("Neck/back/hip pain 0-10"), "peripheral_joint_pain_swelling": q("Peripheral joint pain/swelling 0-10"), "enthesitis": q("Areas tender to touch/pressure 0-10"), "morning_stiffness_severity": q("Morning stiffness severity 0-10"), "morning_stiffness_duration": q("Morning stiffness duration 0-10") } })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: BasdaiInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn computes_stiffness_average_then_total() {
        let out = compute(&BasdaiInput {
            fatigue: 4.0,
            spinal_pain: 5.0,
            peripheral_joint_pain_swelling: 6.0,
            enthesitis: 7.0,
            morning_stiffness_severity: 8.0,
            morning_stiffness_duration: 6.0,
        })
        .unwrap();
        assert_eq!(round1(out.score), 5.8);
    }
    #[test]
    fn rejects_out_of_range() {
        assert!(
            compute(&BasdaiInput {
                fatigue: 11.0,
                spinal_pain: 0.0,
                peripheral_joint_pain_swelling: 0.0,
                enthesitis: 0.0,
                morning_stiffness_severity: 0.0,
                morning_stiffness_duration: 0.0
            })
            .is_err()
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let typed = BasdaiInput {
            fatigue: 4.0,
            spinal_pain: 5.0,
            peripheral_joint_pain_swelling: 6.0,
            enthesitis: 7.0,
            morning_stiffness_severity: 8.0,
            morning_stiffness_duration: 6.0,
        };
        let value = json!({
            "fatigue": 4.0, "spinal_pain": 5.0, "peripheral_joint_pain_swelling": 6.0,
            "enthesitis": 7.0, "morning_stiffness_severity": 8.0, "morning_stiffness_duration": 6.0
        });
        let dynamic = Basdai.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }
}
