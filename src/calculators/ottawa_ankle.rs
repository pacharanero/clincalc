// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Ottawa Ankle Rule - decision rule for selective radiography after acute
//! ankle or midfoot injury.
//!
//! Two independent zones, each with its own imaging decision (Stiell et al.
//! JAMA 1994). Applies to patients aged 18 or over who are not intoxicated,
//! do not have multiple painful or distracting injuries, and do not have a
//! head injury or a neurological deficit that would make examination
//! unreliable.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "ottawa_ankle";

pub const REFERENCE: &str = "Stiell IG, McKnight RD, Greenberg GH, et al. Implementation of the Ottawa ankle rules. \
JAMA. 1994;271(11):827-832.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain decision rule - implemented from the primary literature",
    "https://doi.org/10.1001/jama.1994.03510110045032",
);

/// Ottawa Ankle Rule inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OttawaAnkleInput {
    /// Pain in the malleolar zone (around either malleolus).
    pub pain_in_malleolar_zone: bool,
    /// Bone tenderness along the distal 6 cm of the posterior edge of the
    /// fibula or the tip of the lateral malleolus.
    pub tenderness_posterior_edge_or_tip_lateral_malleolus: bool,
    /// Bone tenderness along the distal 6 cm of the posterior edge of the
    /// tibia or the tip of the medial malleolus.
    pub tenderness_posterior_edge_or_tip_medial_malleolus: bool,
    /// Pain in the midfoot zone.
    pub pain_in_midfoot_zone: bool,
    /// Bone tenderness at the base of the 5th metatarsal.
    pub tenderness_base_fifth_metatarsal: bool,
    /// Bone tenderness at the navicular bone.
    pub tenderness_navicular: bool,
    /// Unable to bear weight for 4 steps, both immediately after the injury
    /// and in the emergency department/clinic (limping counts as bearing
    /// weight).
    pub unable_to_bear_weight_four_steps: bool,
}

/// The computed outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OttawaAnkleOutcome {
    pub ankle_series_indicated: bool,
    pub foot_series_indicated: bool,
}

pub fn compute(input: &OttawaAnkleInput) -> OttawaAnkleOutcome {
    let ankle_series_indicated = input.pain_in_malleolar_zone
        && (input.tenderness_posterior_edge_or_tip_lateral_malleolus
            || input.tenderness_posterior_edge_or_tip_medial_malleolus
            || input.unable_to_bear_weight_four_steps);
    let foot_series_indicated = input.pain_in_midfoot_zone
        && (input.tenderness_base_fifth_metatarsal
            || input.tenderness_navicular
            || input.unable_to_bear_weight_four_steps);
    OttawaAnkleOutcome {
        ankle_series_indicated,
        foot_series_indicated,
    }
}

pub fn build_response(input: &OttawaAnkleInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input);
    let any_indicated = o.ankle_series_indicated || o.foot_series_indicated;

    let mut working = Map::new();
    working.insert(
        "ankle_series_indicated".into(),
        json!(o.ankle_series_indicated),
    );
    working.insert(
        "foot_series_indicated".into(),
        json!(o.foot_series_indicated),
    );

    let interpretation = match (o.ankle_series_indicated, o.foot_series_indicated) {
        (true, true) => "Both an ankle and a foot radiograph series are indicated: pain in each zone plus a positive finding in that zone.".to_string(),
        (true, false) => "An ankle radiograph series is indicated: pain in the malleolar zone plus bone tenderness at the posterior edge/tip of either malleolus or inability to bear weight for 4 steps.".to_string(),
        (false, true) => "A foot radiograph series is indicated: pain in the midfoot zone plus bone tenderness at the base of the 5th metatarsal or the navicular, or inability to bear weight for 4 steps.".to_string(),
        (false, false) => "Neither series is indicated by the rule. The rule is highly sensitive but not specific for clinically significant fracture; applies to patients aged 18 or over who are reliably examinable (not intoxicated, no multiple painful or distracting injuries, no head injury or neurological deficit affecting sensation) with an injury less than 10 days old.".to_string(),
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(any_indicated),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct OttawaAnkle;
impl Calculator for OttawaAnkle {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Ottawa Ankle Rule"
    }
    fn description(&self) -> &'static str {
        "Decision rule for selective ankle and/or midfoot radiography after acute ankle injury."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "OttawaAnkleInput", "type": "object", "additionalProperties": false,
            "required": [
                "pain_in_malleolar_zone", "tenderness_posterior_edge_or_tip_lateral_malleolus",
                "tenderness_posterior_edge_or_tip_medial_malleolus", "pain_in_midfoot_zone",
                "tenderness_base_fifth_metatarsal", "tenderness_navicular", "unable_to_bear_weight_four_steps"
            ],
            "properties": {
                "pain_in_malleolar_zone": { "type": "boolean", "description": "Pain in the malleolar zone" },
                "tenderness_posterior_edge_or_tip_lateral_malleolus": { "type": "boolean", "description": "Bone tenderness, distal 6cm posterior edge of fibula or tip of lateral malleolus" },
                "tenderness_posterior_edge_or_tip_medial_malleolus": { "type": "boolean", "description": "Bone tenderness, distal 6cm posterior edge of tibia or tip of medial malleolus" },
                "pain_in_midfoot_zone": { "type": "boolean", "description": "Pain in the midfoot zone" },
                "tenderness_base_fifth_metatarsal": { "type": "boolean", "description": "Bone tenderness at the base of the 5th metatarsal" },
                "tenderness_navicular": { "type": "boolean", "description": "Bone tenderness at the navicular bone" },
                "unable_to_bear_weight_four_steps": { "type": "boolean", "description": "Unable to bear weight for 4 steps, both immediately after injury and in the ED/clinic (limping still counts as weight-bearing)" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: OttawaAnkleInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> OttawaAnkleInput {
        OttawaAnkleInput {
            pain_in_malleolar_zone: false,
            tenderness_posterior_edge_or_tip_lateral_malleolus: false,
            tenderness_posterior_edge_or_tip_medial_malleolus: false,
            pain_in_midfoot_zone: false,
            tenderness_base_fifth_metatarsal: false,
            tenderness_navicular: false,
            unable_to_bear_weight_four_steps: false,
        }
    }

    #[test]
    fn no_pain_no_imaging() {
        let o = compute(&none());
        assert!(!o.ankle_series_indicated);
        assert!(!o.foot_series_indicated);
    }

    #[test]
    fn malleolar_pain_alone_is_not_enough() {
        let mut i = none();
        i.pain_in_malleolar_zone = true;
        let o = compute(&i);
        assert!(!o.ankle_series_indicated);
    }

    #[test]
    fn malleolar_pain_plus_lateral_tenderness_indicates_ankle_series() {
        let mut i = none();
        i.pain_in_malleolar_zone = true;
        i.tenderness_posterior_edge_or_tip_lateral_malleolus = true;
        let o = compute(&i);
        assert!(o.ankle_series_indicated);
        assert!(!o.foot_series_indicated);
    }

    #[test]
    fn malleolar_pain_plus_inability_to_bear_weight_indicates_ankle_series() {
        let mut i = none();
        i.pain_in_malleolar_zone = true;
        i.unable_to_bear_weight_four_steps = true;
        assert!(compute(&i).ankle_series_indicated);
    }

    #[test]
    fn midfoot_pain_plus_navicular_tenderness_indicates_foot_series() {
        let mut i = none();
        i.pain_in_midfoot_zone = true;
        i.tenderness_navicular = true;
        let o = compute(&i);
        assert!(o.foot_series_indicated);
        assert!(!o.ankle_series_indicated);
    }

    #[test]
    fn both_zones_can_be_indicated_together() {
        let mut i = none();
        i.pain_in_malleolar_zone = true;
        i.unable_to_bear_weight_four_steps = true;
        i.pain_in_midfoot_zone = true;
        i.tenderness_base_fifth_metatarsal = true;
        let o = compute(&i);
        assert!(o.ankle_series_indicated);
        assert!(o.foot_series_indicated);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let mut typed = none();
        typed.pain_in_midfoot_zone = true;
        typed.tenderness_base_fifth_metatarsal = true;
        let value = json!({
            "pain_in_malleolar_zone": false, "tenderness_posterior_edge_or_tip_lateral_malleolus": false,
            "tenderness_posterior_edge_or_tip_medial_malleolus": false, "pain_in_midfoot_zone": true,
            "tenderness_base_fifth_metatarsal": true, "tenderness_navicular": false,
            "unable_to_bear_weight_four_steps": false
        });
        let dynamic = OttawaAnkle.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(true));
    }

    #[test]
    fn rejects_missing_field() {
        assert!(
            OttawaAnkle
                .calculate(&json!({"pain_in_malleolar_zone": true}))
                .is_err()
        );
    }
}
