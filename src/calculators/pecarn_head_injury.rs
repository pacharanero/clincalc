// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! PECARN paediatric head injury/trauma algorithm - age-stratified decision
//! rule (under 2 years vs 2-18 years) for selective CT imaging after minor
//! blunt head trauma.
//!
//! Applies to children with GCS 14-15 presenting within 24 hours of blunt
//! head trauma; a GCS below 14, penetrating trauma, or a known brain tumour
//! or neurological disorder complicating assessment are outside the
//! validated population.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "pecarn_head_injury";
pub const REFERENCE: &str = "Kuppermann N, Holmes JF, Dayan PS, et al. Identification of children at very low risk of clinically-important brain injuries after head trauma: a prospective cohort study. Lancet. 2009;374(9696):1160-1170.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain decision rule - implemented from the primary literature",
    "https://doi.org/10.1016/S0140-6736(09)61558-0",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    High,
    Intermediate,
    Low,
}
impl RiskLevel {
    fn slug(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Intermediate => "intermediate",
            Self::Low => "low",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PecarnHeadInjuryInput {
    pub age_under_2: bool,
    // High-risk tier, shared concept across both age bands.
    pub gcs_14_or_less: bool,
    pub altered_mental_status: bool,
    /// Required (Some) when age_under_2 is true; must be omitted otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub palpable_skull_fracture: Option<bool>,
    /// Required (Some) when age_under_2 is false; must be omitted otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signs_of_basilar_skull_fracture: Option<bool>,
    // Intermediate-risk tier, under-2 branch only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub non_frontal_scalp_hematoma: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loc_5_seconds_or_more: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abnormal_behavior_per_parent: Option<bool>,
    // Intermediate-risk tier, 2-and-over branch only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_of_vomiting: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loc_any_duration: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severe_headache: Option<bool>,
    /// Pedestrian/cyclist without helmet struck by a motorised vehicle, a
    /// fall (>3ft/1m under 2; >5ft/2m aged 2+), or head struck by a
    /// high-impact object. Shared concept across both age bands.
    pub severe_mechanism: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PecarnHeadInjuryOutcome {
    pub risk_level: RiskLevel,
}

pub fn compute(input: &PecarnHeadInjuryInput) -> Result<PecarnHeadInjuryOutcome, CalcError> {
    if input.age_under_2 {
        if input.palpable_skull_fracture.is_none() {
            return Err(CalcError::InvalidInput(
                "palpable_skull_fracture is required when age_under_2 is true".to_string(),
            ));
        }
        if input.signs_of_basilar_skull_fracture.is_some() {
            return Err(CalcError::InvalidInput(
                "signs_of_basilar_skull_fracture must be omitted when age_under_2 is true"
                    .to_string(),
            ));
        }
        if input.non_frontal_scalp_hematoma.is_none()
            || input.loc_5_seconds_or_more.is_none()
            || input.abnormal_behavior_per_parent.is_none()
        {
            return Err(CalcError::InvalidInput(
                "non_frontal_scalp_hematoma, loc_5_seconds_or_more, and abnormal_behavior_per_parent are all required when age_under_2 is true".to_string(),
            ));
        }
        if input.history_of_vomiting.is_some()
            || input.loc_any_duration.is_some()
            || input.severe_headache.is_some()
        {
            return Err(CalcError::InvalidInput(
                "history_of_vomiting, loc_any_duration, and severe_headache must be omitted when age_under_2 is true".to_string(),
            ));
        }

        let high_risk = input.gcs_14_or_less
            || input.altered_mental_status
            || input.palpable_skull_fracture.unwrap();
        let intermediate_risk = input.non_frontal_scalp_hematoma.unwrap()
            || input.loc_5_seconds_or_more.unwrap()
            || input.severe_mechanism
            || input.abnormal_behavior_per_parent.unwrap();

        let risk_level = if high_risk {
            RiskLevel::High
        } else if intermediate_risk {
            RiskLevel::Intermediate
        } else {
            RiskLevel::Low
        };
        Ok(PecarnHeadInjuryOutcome { risk_level })
    } else {
        if input.signs_of_basilar_skull_fracture.is_none() {
            return Err(CalcError::InvalidInput(
                "signs_of_basilar_skull_fracture is required when age_under_2 is false".to_string(),
            ));
        }
        if input.palpable_skull_fracture.is_some() {
            return Err(CalcError::InvalidInput(
                "palpable_skull_fracture must be omitted when age_under_2 is false".to_string(),
            ));
        }
        if input.history_of_vomiting.is_none()
            || input.loc_any_duration.is_none()
            || input.severe_headache.is_none()
        {
            return Err(CalcError::InvalidInput(
                "history_of_vomiting, loc_any_duration, and severe_headache are all required when age_under_2 is false".to_string(),
            ));
        }
        if input.non_frontal_scalp_hematoma.is_some()
            || input.loc_5_seconds_or_more.is_some()
            || input.abnormal_behavior_per_parent.is_some()
        {
            return Err(CalcError::InvalidInput(
                "non_frontal_scalp_hematoma, loc_5_seconds_or_more, and abnormal_behavior_per_parent must be omitted when age_under_2 is false".to_string(),
            ));
        }

        let high_risk = input.gcs_14_or_less
            || input.altered_mental_status
            || input.signs_of_basilar_skull_fracture.unwrap();
        let intermediate_risk = input.history_of_vomiting.unwrap()
            || input.loc_any_duration.unwrap()
            || input.severe_mechanism
            || input.severe_headache.unwrap();

        let risk_level = if high_risk {
            RiskLevel::High
        } else if intermediate_risk {
            RiskLevel::Intermediate
        } else {
            RiskLevel::Low
        };
        Ok(PecarnHeadInjuryOutcome { risk_level })
    }
}

pub fn build_response(input: &PecarnHeadInjuryInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("risk_level".into(), json!(o.risk_level.slug()));
    working.insert(
        "age_band".into(),
        json!(if input.age_under_2 {
            "under_2"
        } else {
            "2_to_18"
        }),
    );

    let interpretation = match o.risk_level {
        RiskLevel::High => "High risk: CT recommended. Risk of clinically-important traumatic brain injury (ciTBI) was approximately 4.4% (under 2) / 4.3% (2 and older) in this tier in the derivation/validation cohorts.".to_string(),
        RiskLevel::Intermediate => "Intermediate risk: consider CT or observation (clinical judgement, shared decision-making, and local factors such as multiple findings or worsening symptoms should guide the choice). Risk of ciTBI in this tier was approximately 0.9%; isolated vomiting alone (no other risk factor) carries a substantially lower risk (~0.2%) and favours observation over imaging.".to_string(),
        RiskLevel::Low => "Low risk: CT not recommended by this rule. Applies only to children with GCS 14-15 presenting within 24 hours of blunt head trauma; excludes trivial mechanisms, penetrating trauma, known brain tumour, or a pre-existing neurological disorder complicating assessment.".to_string(),
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.risk_level.slug()),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct PecarnHeadInjury;
impl Calculator for PecarnHeadInjury {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "PECARN Paediatric Head Injury/Trauma Algorithm"
    }
    fn description(&self) -> &'static str {
        "Age-stratified (under 2 / 2-18 years) decision rule for selective CT imaging after minor paediatric blunt head trauma."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "PecarnHeadInjuryInput", "type": "object", "additionalProperties": false,
            "required": ["age_under_2", "gcs_14_or_less", "altered_mental_status", "severe_mechanism"],
            "properties": {
                "age_under_2": { "type": "boolean" },
                "gcs_14_or_less": { "type": "boolean" },
                "altered_mental_status": { "type": "boolean", "description": "Agitation, somnolence, repetitive questioning, or slow response to verbal communication" },
                "palpable_skull_fracture": { "type": "boolean", "description": "Required when age_under_2 is true; omit otherwise" },
                "signs_of_basilar_skull_fracture": { "type": "boolean", "description": "Required when age_under_2 is false; omit otherwise" },
                "non_frontal_scalp_hematoma": { "type": "boolean", "description": "Required when age_under_2 is true; omit otherwise" },
                "loc_5_seconds_or_more": { "type": "boolean", "description": "Required when age_under_2 is true; omit otherwise" },
                "abnormal_behavior_per_parent": { "type": "boolean", "description": "Required when age_under_2 is true; omit otherwise" },
                "history_of_vomiting": { "type": "boolean", "description": "Required when age_under_2 is false; omit otherwise" },
                "loc_any_duration": { "type": "boolean", "description": "Required when age_under_2 is false; omit otherwise" },
                "severe_headache": { "type": "boolean", "description": "Required when age_under_2 is false; omit otherwise" },
                "severe_mechanism": { "type": "boolean", "description": "Pedestrian/cyclist without helmet struck by a motorised vehicle, a fall (>3ft/1m if under 2; >5ft/2m if 2 or older), or head struck by a high-impact object" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: PecarnHeadInjuryInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn under_2_low_risk() -> PecarnHeadInjuryInput {
        PecarnHeadInjuryInput {
            age_under_2: true,
            gcs_14_or_less: false,
            altered_mental_status: false,
            palpable_skull_fracture: Some(false),
            signs_of_basilar_skull_fracture: None,
            non_frontal_scalp_hematoma: Some(false),
            loc_5_seconds_or_more: Some(false),
            abnormal_behavior_per_parent: Some(false),
            history_of_vomiting: None,
            loc_any_duration: None,
            severe_headache: None,
            severe_mechanism: false,
        }
    }

    fn two_and_over_low_risk() -> PecarnHeadInjuryInput {
        PecarnHeadInjuryInput {
            age_under_2: false,
            gcs_14_or_less: false,
            altered_mental_status: false,
            palpable_skull_fracture: None,
            signs_of_basilar_skull_fracture: Some(false),
            non_frontal_scalp_hematoma: None,
            loc_5_seconds_or_more: None,
            abnormal_behavior_per_parent: None,
            history_of_vomiting: Some(false),
            loc_any_duration: Some(false),
            severe_headache: Some(false),
            severe_mechanism: false,
        }
    }

    #[test]
    fn under_2_low_risk_no_ct() {
        let o = compute(&under_2_low_risk()).unwrap();
        assert_eq!(o.risk_level, RiskLevel::Low);
    }

    #[test]
    fn under_2_high_risk_from_palpable_fracture() {
        let mut input = under_2_low_risk();
        input.palpable_skull_fracture = Some(true);
        let o = compute(&input).unwrap();
        assert_eq!(o.risk_level, RiskLevel::High);
    }

    #[test]
    fn under_2_intermediate_from_scalp_hematoma() {
        let mut input = under_2_low_risk();
        input.non_frontal_scalp_hematoma = Some(true);
        let o = compute(&input).unwrap();
        assert_eq!(o.risk_level, RiskLevel::Intermediate);
    }

    #[test]
    fn two_and_over_low_risk_no_ct() {
        let o = compute(&two_and_over_low_risk()).unwrap();
        assert_eq!(o.risk_level, RiskLevel::Low);
    }

    #[test]
    fn two_and_over_high_risk_from_basilar_fracture() {
        let mut input = two_and_over_low_risk();
        input.signs_of_basilar_skull_fracture = Some(true);
        let o = compute(&input).unwrap();
        assert_eq!(o.risk_level, RiskLevel::High);
    }

    #[test]
    fn two_and_over_intermediate_from_vomiting() {
        let mut input = two_and_over_low_risk();
        input.history_of_vomiting = Some(true);
        let o = compute(&input).unwrap();
        assert_eq!(o.risk_level, RiskLevel::Intermediate);
    }

    #[test]
    fn rejects_under_2_fields_when_age_2_and_over() {
        let mut input = two_and_over_low_risk();
        input.non_frontal_scalp_hematoma = Some(true);
        assert!(compute(&input).is_err());
    }

    #[test]
    fn rejects_missing_branch_specific_field() {
        let mut input = under_2_low_risk();
        input.palpable_skull_fracture = None;
        assert!(compute(&input).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let typed = under_2_low_risk();
        let value = json!({
            "age_under_2": true, "gcs_14_or_less": false, "altered_mental_status": false,
            "palpable_skull_fracture": false, "non_frontal_scalp_hematoma": false,
            "loc_5_seconds_or_more": false, "abnormal_behavior_per_parent": false, "severe_mechanism": false
        });
        let dynamic = PecarnHeadInjury.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            PecarnHeadInjury
                .calculate(&json!({"age_under_2": true}))
                .is_err()
        );
    }
}
