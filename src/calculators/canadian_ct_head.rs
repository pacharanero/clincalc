// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Canadian CT Head Rule - decision rule for selective CT imaging after
//! minor head injury in alert adults (GCS 13-15).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "canadian_ct_head";
pub const REFERENCE: &str = "Stiell IG, Wells GA, Vandemheen K, et al. The Canadian CT Head Rule for patients with minor head injury. Lancet. 2001;357(9266):1391-1396.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain decision rule - implemented from the primary literature",
    "https://doi.org/10.1016/S0140-6736(00)04561-X",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanadianCtHeadInput {
    // High-risk criteria (for neurological intervention).
    pub gcs_below_15_at_2_hours: bool,
    pub suspected_open_or_depressed_skull_fracture: bool,
    pub sign_of_basal_skull_fracture: bool,
    pub vomiting_2_or_more_episodes: bool,
    pub age_65_or_over: bool,
    // Medium-risk criteria (for brain-injury detection on CT).
    pub amnesia_before_impact_30_minutes_or_more: bool,
    pub dangerous_mechanism: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    None,
    Medium,
    High,
}
impl RiskLevel {
    fn slug(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanadianCtHeadOutcome {
    pub risk_level: RiskLevel,
    pub ct_recommended: bool,
}

pub fn compute(input: &CanadianCtHeadInput) -> CanadianCtHeadOutcome {
    let high_risk = input.gcs_below_15_at_2_hours
        || input.suspected_open_or_depressed_skull_fracture
        || input.sign_of_basal_skull_fracture
        || input.vomiting_2_or_more_episodes
        || input.age_65_or_over;
    let medium_risk = input.amnesia_before_impact_30_minutes_or_more || input.dangerous_mechanism;

    let risk_level = if high_risk {
        RiskLevel::High
    } else if medium_risk {
        RiskLevel::Medium
    } else {
        RiskLevel::None
    };

    CanadianCtHeadOutcome {
        risk_level,
        ct_recommended: high_risk || medium_risk,
    }
}

pub fn build_response(input: &CanadianCtHeadInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input);
    let mut working = Map::new();
    working.insert("risk_level".into(), json!(o.risk_level.slug()));

    let interpretation = match o.risk_level {
        RiskLevel::High => "High risk: at least one high-risk criterion present (GCS <15 at 2 hours, suspected open/depressed skull fracture, sign of basal skull fracture, 2 or more vomiting episodes, or age 65+). CT head recommended; high-risk criteria predict the need for neurological intervention.".to_string(),
        RiskLevel::Medium => "Medium risk: no high-risk criterion, but a medium-risk criterion is present (amnesia before impact of 30 minutes or more, or a dangerous mechanism). CT head recommended; medium-risk criteria predict clinically important brain injury on CT.".to_string(),
        RiskLevel::None => "No high- or medium-risk criteria present. The rule does not recommend CT on this basis; applies only to alert adults (GCS 13-15) with loss of consciousness, definite amnesia, or witnessed disorientation, assessed within 24 hours of injury, and excludes patients under 16, GCS <13, obvious penetrating injury, acute focal neurological deficit, unstable vital signs, a seizure before assessment, a known bleeding disorder or anticoagulation, or a return visit for the same injury.".to_string(),
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.ct_recommended),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct CanadianCtHead;
impl Calculator for CanadianCtHead {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Canadian CT Head Rule"
    }
    fn description(&self) -> &'static str {
        "Decision rule for selective CT imaging after minor head injury in alert adults (GCS 13-15)."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "CanadianCtHeadInput", "type": "object", "additionalProperties": false,
            "required": [
                "gcs_below_15_at_2_hours", "suspected_open_or_depressed_skull_fracture", "sign_of_basal_skull_fracture",
                "vomiting_2_or_more_episodes", "age_65_or_over", "amnesia_before_impact_30_minutes_or_more", "dangerous_mechanism"
            ],
            "properties": {
                "gcs_below_15_at_2_hours": { "type": "boolean", "description": "High-risk: GCS below 15 when assessed at 2 hours after injury" },
                "suspected_open_or_depressed_skull_fracture": { "type": "boolean", "description": "High-risk" },
                "sign_of_basal_skull_fracture": { "type": "boolean", "description": "High-risk: haemotympanum, raccoon eyes, CSF oto/rhinorrhoea, or Battle's sign" },
                "vomiting_2_or_more_episodes": { "type": "boolean", "description": "High-risk" },
                "age_65_or_over": { "type": "boolean", "description": "High-risk" },
                "amnesia_before_impact_30_minutes_or_more": { "type": "boolean", "description": "Medium-risk" },
                "dangerous_mechanism": { "type": "boolean", "description": "Medium-risk: pedestrian struck by vehicle, occupant ejected from vehicle, or a fall from height of 3 feet / 5 stairs or more" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: CanadianCtHeadInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_negative() -> CanadianCtHeadInput {
        CanadianCtHeadInput {
            gcs_below_15_at_2_hours: false,
            suspected_open_or_depressed_skull_fracture: false,
            sign_of_basal_skull_fracture: false,
            vomiting_2_or_more_episodes: false,
            age_65_or_over: false,
            amnesia_before_impact_30_minutes_or_more: false,
            dangerous_mechanism: false,
        }
    }

    #[test]
    fn no_criteria_no_ct() {
        let o = compute(&all_negative());
        assert_eq!(o.risk_level, RiskLevel::None);
        assert!(!o.ct_recommended);
    }

    #[test]
    fn any_high_risk_criterion_recommends_ct() {
        let mut input = all_negative();
        input.age_65_or_over = true;
        let o = compute(&input);
        assert_eq!(o.risk_level, RiskLevel::High);
        assert!(o.ct_recommended);
    }

    #[test]
    fn medium_risk_alone_recommends_ct_but_not_high() {
        let mut input = all_negative();
        input.dangerous_mechanism = true;
        let o = compute(&input);
        assert_eq!(o.risk_level, RiskLevel::Medium);
        assert!(o.ct_recommended);
    }

    #[test]
    fn high_risk_takes_precedence_over_medium() {
        let mut input = all_negative();
        input.age_65_or_over = true;
        input.dangerous_mechanism = true;
        let o = compute(&input);
        assert_eq!(o.risk_level, RiskLevel::High);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let mut typed = all_negative();
        typed.vomiting_2_or_more_episodes = true;
        let value = json!({
            "gcs_below_15_at_2_hours": false, "suspected_open_or_depressed_skull_fracture": false,
            "sign_of_basal_skull_fracture": false, "vomiting_2_or_more_episodes": true, "age_65_or_over": false,
            "amnesia_before_impact_30_minutes_or_more": false, "dangerous_mechanism": false
        });
        let dynamic = CanadianCtHead.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            CanadianCtHead
                .calculate(&json!({"gcs_below_15_at_2_hours": false}))
                .is_err()
        );
    }
}
