// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Ottawa Knee Rule - decision rule for selective radiography after acute
//! knee injury (Stiell et al. Ann Emerg Med 1995).
//!
//! Validated for adults aged 18 or over; not recommended for paediatric use
//! (lower sensitivity than the Pittsburgh Knee Rule in children). Excludes
//! injuries more than 7 days old, isolated skin injuries, and patients with
//! an altered level of consciousness or paraplegia.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "ottawa_knee";

pub const REFERENCE: &str = "Stiell IG, Greenberg GH, Wells GA, et al. Derivation of a decision rule for the use of \
radiography in acute knee injuries. Ann Emerg Med. 1995;26(4):405-413.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain decision rule - implemented from the primary literature",
    "https://doi.org/10.1016/S0196-0644(95)70106-0",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OttawaKneeInput {
    pub age_55_or_over: bool,
    /// Isolated tenderness of the patella, i.e. no bone tenderness of the
    /// knee other than the patella.
    pub isolated_patellar_tenderness: bool,
    pub tenderness_at_fibular_head: bool,
    pub unable_to_flex_knee_90_degrees: bool,
    /// Unable to bear weight for 4 steps, both immediately after the injury
    /// and in the emergency department/clinic, regardless of limping.
    pub unable_to_bear_weight_four_steps: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OttawaKneeOutcome {
    pub xray_indicated: bool,
}

pub fn compute(input: &OttawaKneeInput) -> OttawaKneeOutcome {
    OttawaKneeOutcome {
        xray_indicated: input.age_55_or_over
            || input.isolated_patellar_tenderness
            || input.tenderness_at_fibular_head
            || input.unable_to_flex_knee_90_degrees
            || input.unable_to_bear_weight_four_steps,
    }
}

pub fn build_response(input: &OttawaKneeInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input);
    let mut working = Map::new();
    working.insert("age_55_or_over".into(), json!(input.age_55_or_over));
    working.insert(
        "isolated_patellar_tenderness".into(),
        json!(input.isolated_patellar_tenderness),
    );
    working.insert(
        "tenderness_at_fibular_head".into(),
        json!(input.tenderness_at_fibular_head),
    );
    working.insert(
        "unable_to_flex_knee_90_degrees".into(),
        json!(input.unable_to_flex_knee_90_degrees),
    );
    working.insert(
        "unable_to_bear_weight_four_steps".into(),
        json!(input.unable_to_bear_weight_four_steps),
    );

    let interpretation = if o.xray_indicated {
        "At least one criterion is present: a knee radiograph series is indicated. The rule is highly sensitive (close to 100%) but not specific for clinically significant fracture, so a positive result produces many negative radiographs.".to_string()
    } else {
        "No criterion is present: a knee radiograph series is not indicated by the rule. Validated for patients aged 18 or over with an acute knee injury less than 7 days old who are reliably examinable; not validated for children.".to_string()
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.xray_indicated),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct OttawaKnee;
impl Calculator for OttawaKnee {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Ottawa Knee Rule"
    }
    fn description(&self) -> &'static str {
        "Decision rule for selective knee radiography after acute knee injury in adults."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "OttawaKneeInput", "type": "object", "additionalProperties": false,
            "required": [
                "age_55_or_over", "isolated_patellar_tenderness", "tenderness_at_fibular_head",
                "unable_to_flex_knee_90_degrees", "unable_to_bear_weight_four_steps"
            ],
            "properties": {
                "age_55_or_over": { "type": "boolean", "description": "Age 55 years or over" },
                "isolated_patellar_tenderness": { "type": "boolean", "description": "Isolated tenderness of the patella (no bone tenderness of the knee other than the patella)" },
                "tenderness_at_fibular_head": { "type": "boolean", "description": "Tenderness at the head of the fibula" },
                "unable_to_flex_knee_90_degrees": { "type": "boolean", "description": "Unable to flex the knee to 90 degrees" },
                "unable_to_bear_weight_four_steps": { "type": "boolean", "description": "Unable to bear weight for 4 steps, both immediately after the injury and in the ED/clinic, regardless of limping" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: OttawaKneeInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> OttawaKneeInput {
        OttawaKneeInput {
            age_55_or_over: false,
            isolated_patellar_tenderness: false,
            tenderness_at_fibular_head: false,
            unable_to_flex_knee_90_degrees: false,
            unable_to_bear_weight_four_steps: false,
        }
    }

    #[test]
    fn no_criteria_no_xray() {
        assert!(!compute(&none()).xray_indicated);
    }

    #[test]
    fn any_single_criterion_indicates_xray() {
        let mut i = none();
        i.age_55_or_over = true;
        assert!(compute(&i).xray_indicated);

        let mut i = none();
        i.isolated_patellar_tenderness = true;
        assert!(compute(&i).xray_indicated);

        let mut i = none();
        i.tenderness_at_fibular_head = true;
        assert!(compute(&i).xray_indicated);

        let mut i = none();
        i.unable_to_flex_knee_90_degrees = true;
        assert!(compute(&i).xray_indicated);

        let mut i = none();
        i.unable_to_bear_weight_four_steps = true;
        assert!(compute(&i).xray_indicated);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let mut typed = none();
        typed.age_55_or_over = true;
        let value = json!({
            "age_55_or_over": true, "isolated_patellar_tenderness": false, "tenderness_at_fibular_head": false,
            "unable_to_flex_knee_90_degrees": false, "unable_to_bear_weight_four_steps": false
        });
        let dynamic = OttawaKnee.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(true));
    }

    #[test]
    fn rejects_missing_field() {
        assert!(
            OttawaKnee
                .calculate(&json!({"age_55_or_over": true}))
                .is_err()
        );
    }
}
