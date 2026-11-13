// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! NEXUS Criteria - five low-risk criteria for selective cervical-spine
//! imaging after blunt trauma (Hoffman et al. N Engl J Med 2000).
//!
//! Imaging can be safely deferred only when all five findings below are
//! ABSENT. Each input here is phrased as the finding being present (true),
//! matching the convention used elsewhere in this registry; cervical-spine
//! imaging is considered unnecessary only when every one of the five is
//! false.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "nexus";

pub const REFERENCE: &str = "Hoffman JR, Mower WR, Wolfson AB, Todd KH, Zucker MI. Validity of a set of clinical \
criteria to rule out injury to the cervical spine in patients with blunt trauma. National Emergency \
X-Radiography Utilization Study Group. N Engl J Med. 2000;343(2):94-99.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain decision rule - implemented from the primary literature",
    "https://doi.org/10.1056/NEJM200007133430203",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NexusInput {
    pub midline_cervical_tenderness: bool,
    pub focal_neurologic_deficit: bool,
    /// Not a normal level of alertness: GCS below 15, disorientation to
    /// person/time/place/events, inability to remember 3 objects at 5
    /// minutes, or a delayed or inappropriate response to external stimuli.
    pub abnormal_level_of_alertness: bool,
    pub evidence_of_intoxication: bool,
    /// A clinically apparent, painful injury that might distract the patient
    /// from the pain of a cervical-spine injury.
    pub painful_distracting_injury: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NexusOutcome {
    pub low_probability_of_injury: bool,
}

pub fn compute(input: &NexusInput) -> NexusOutcome {
    let any_present = input.midline_cervical_tenderness
        || input.focal_neurologic_deficit
        || input.abnormal_level_of_alertness
        || input.evidence_of_intoxication
        || input.painful_distracting_injury;
    NexusOutcome {
        low_probability_of_injury: !any_present,
    }
}

pub fn build_response(input: &NexusInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input);
    let mut working = Map::new();
    working.insert(
        "midline_cervical_tenderness".into(),
        json!(input.midline_cervical_tenderness),
    );
    working.insert(
        "focal_neurologic_deficit".into(),
        json!(input.focal_neurologic_deficit),
    );
    working.insert(
        "abnormal_level_of_alertness".into(),
        json!(input.abnormal_level_of_alertness),
    );
    working.insert(
        "evidence_of_intoxication".into(),
        json!(input.evidence_of_intoxication),
    );
    working.insert(
        "painful_distracting_injury".into(),
        json!(input.painful_distracting_injury),
    );
    working.insert(
        "imaging_required".into(),
        json!(!o.low_probability_of_injury),
    );

    let interpretation = if o.low_probability_of_injury {
        "All five NEXUS criteria are absent: low probability of cervical-spine injury, and cervical-spine imaging can be deferred. In the validation study this classified patients with 99.6% sensitivity and 99.9% negative predictive value for cervical-spine injury, at the cost of low specificity (12.9%)."
    } else {
        "At least one NEXUS criterion is present: the low-probability rule is not met, and cervical-spine imaging is indicated."
    }
    .to_string();

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.low_probability_of_injury),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Nexus;
impl Calculator for Nexus {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "NEXUS Criteria (Cervical-Spine Imaging)"
    }
    fn description(&self) -> &'static str {
        "Five low-risk criteria to determine whether cervical-spine imaging can be deferred after blunt trauma."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "NexusInput", "type": "object", "additionalProperties": false,
            "required": [
                "midline_cervical_tenderness", "focal_neurologic_deficit", "abnormal_level_of_alertness",
                "evidence_of_intoxication", "painful_distracting_injury"
            ],
            "properties": {
                "midline_cervical_tenderness": { "type": "boolean", "description": "Tenderness at the posterior midline of the cervical spine" },
                "focal_neurologic_deficit": { "type": "boolean", "description": "Any focal neurologic deficit" },
                "abnormal_level_of_alertness": { "type": "boolean", "description": "Not a normal level of alertness (GCS <15, disorientation, inability to remember 3 objects at 5 minutes, or a delayed/inappropriate response to stimuli)" },
                "evidence_of_intoxication": { "type": "boolean", "description": "Evidence of intoxication" },
                "painful_distracting_injury": { "type": "boolean", "description": "A clinically apparent, painful injury that could distract the patient from cervical-spine pain" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: NexusInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> NexusInput {
        NexusInput {
            midline_cervical_tenderness: false,
            focal_neurologic_deficit: false,
            abnormal_level_of_alertness: false,
            evidence_of_intoxication: false,
            painful_distracting_injury: false,
        }
    }

    #[test]
    fn all_absent_is_low_probability() {
        assert!(compute(&none()).low_probability_of_injury);
    }

    #[test]
    fn any_single_criterion_requires_imaging() {
        let mut i = none();
        i.midline_cervical_tenderness = true;
        assert!(!compute(&i).low_probability_of_injury);

        let mut i = none();
        i.painful_distracting_injury = true;
        assert!(!compute(&i).low_probability_of_injury);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let mut typed = none();
        typed.evidence_of_intoxication = true;
        let value = json!({
            "midline_cervical_tenderness": false, "focal_neurologic_deficit": false,
            "abnormal_level_of_alertness": false, "evidence_of_intoxication": true,
            "painful_distracting_injury": false
        });
        let dynamic = Nexus.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(false));
    }

    #[test]
    fn rejects_missing_field() {
        assert!(
            Nexus
                .calculate(&json!({"midline_cervical_tenderness": true}))
                .is_err()
        );
    }
}
