// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Canadian C-Spine Rule - three-step decision rule for cervical-spine
//! radiography after blunt trauma in alert (GCS 15), haemodynamically stable
//! patients (Stiell et al. JAMA 2001).
//!
//! The three steps are: (1) any high-risk factor mandates radiography; (2)
//! absent any high-risk factor, the presence of at least one low-risk factor
//! allows safe assessment of active range of motion, otherwise radiography
//! is required; (3) a patient with a low-risk factor who can actively rotate
//! their neck 45 degrees left and right does not require radiography.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "canadian_cspine";

pub const REFERENCE: &str = "Stiell IG, Wells GA, Vandemheen KL, et al. The Canadian C-Spine Rule for radiography in alert and stable trauma \
patients. JAMA. 2001;286(15):1841-1848.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain decision rule - implemented from the primary literature",
    "https://doi.org/10.1001/jama.286.15.1841",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanadianCspineInput {
    // High-risk factors (any one mandates radiography).
    pub age_65_or_over: bool,
    /// Fall from an elevation of 3 feet / 5 stairs or more; axial load to the
    /// head (e.g. diving); high-speed (>100 km/h) motor vehicle collision,
    /// rollover, or ejection; a collision involving a motorised recreational
    /// vehicle; or a bicycle collision.
    pub dangerous_mechanism: bool,
    pub paresthesias_in_extremities: bool,
    // Low-risk factors (at least one allows safe range-of-motion assessment).
    /// A rear-end motor vehicle collision, excluding being pushed into
    /// oncoming traffic, hit by a bus or large truck, rollover, or hit by a
    /// high-speed vehicle.
    pub simple_rear_end_mvc: bool,
    pub sitting_position_in_ed: bool,
    pub ambulatory_at_any_time_since_injury: bool,
    /// Delayed (not immediate) onset of neck pain.
    pub delayed_onset_of_neck_pain: bool,
    pub absence_of_midline_cspine_tenderness: bool,
    // Step 3, only assessed when no high-risk factor is present and at
    // least one low-risk factor is present.
    #[serde(default)]
    pub able_to_actively_rotate_neck_45_degrees: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CspineOutcome {
    HighRisk,
    NoLowRiskFactor,
    LowRiskUnableToRotate,
    LowRiskAbleToRotate,
    RotationNotAssessed,
}

impl CspineOutcome {
    fn radiography_recommended(self) -> bool {
        !matches!(self, CspineOutcome::LowRiskAbleToRotate)
    }

    fn slug(self) -> &'static str {
        match self {
            CspineOutcome::HighRisk => "high_risk",
            CspineOutcome::NoLowRiskFactor => "no_low_risk_factor",
            CspineOutcome::LowRiskUnableToRotate => "low_risk_unable_to_rotate",
            CspineOutcome::LowRiskAbleToRotate => "low_risk_able_to_rotate",
            CspineOutcome::RotationNotAssessed => "rotation_not_assessed",
        }
    }
}

pub fn compute(input: &CanadianCspineInput) -> Result<CspineOutcome, CalcError> {
    let high_risk =
        input.age_65_or_over || input.dangerous_mechanism || input.paresthesias_in_extremities;
    if high_risk {
        return Ok(CspineOutcome::HighRisk);
    }

    let low_risk = input.simple_rear_end_mvc
        || input.sitting_position_in_ed
        || input.ambulatory_at_any_time_since_injury
        || input.delayed_onset_of_neck_pain
        || input.absence_of_midline_cspine_tenderness;
    if !low_risk {
        return Ok(CspineOutcome::NoLowRiskFactor);
    }

    match input.able_to_actively_rotate_neck_45_degrees {
        Some(true) => Ok(CspineOutcome::LowRiskAbleToRotate),
        Some(false) => Ok(CspineOutcome::LowRiskUnableToRotate),
        None => Ok(CspineOutcome::RotationNotAssessed),
    }
}

pub fn build_response(input: &CanadianCspineInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("outcome".into(), json!(o.slug()));

    let interpretation = match o {
        CspineOutcome::HighRisk => "At least one high-risk factor is present (age >=65, dangerous mechanism, or paresthesias in extremities): radiography is mandated without proceeding to range-of-motion assessment.".to_string(),
        CspineOutcome::NoLowRiskFactor => "No high-risk factor, but also none of the five low-risk factors: range of motion cannot be safely assessed, so radiography is recommended.".to_string(),
        CspineOutcome::LowRiskUnableToRotate => "No high-risk factor and at least one low-risk factor is present, but the patient is unable to actively rotate the neck 45 degrees left and right: radiography is recommended.".to_string(),
        CspineOutcome::LowRiskAbleToRotate => "No high-risk factor, at least one low-risk factor is present, and the patient can actively rotate the neck 45 degrees left and right: radiography is not required by the rule.".to_string(),
        CspineOutcome::RotationNotAssessed => "No high-risk factor and at least one low-risk factor is present, so active range-of-motion assessment is the deciding step - supply able_to_actively_rotate_neck_45_degrees to complete the rule.".to_string(),
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.radiography_recommended()),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct CanadianCspine;
impl Calculator for CanadianCspine {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Canadian C-Spine Rule"
    }
    fn description(&self) -> &'static str {
        "Three-step decision rule for cervical-spine radiography in alert, stable blunt trauma patients."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "CanadianCspineInput", "type": "object", "additionalProperties": false,
            "required": [
                "age_65_or_over", "dangerous_mechanism", "paresthesias_in_extremities",
                "simple_rear_end_mvc", "sitting_position_in_ed", "ambulatory_at_any_time_since_injury",
                "delayed_onset_of_neck_pain", "absence_of_midline_cspine_tenderness"
            ],
            "properties": {
                "age_65_or_over": { "type": "boolean", "description": "High-risk: age 65 or over" },
                "dangerous_mechanism": { "type": "boolean", "description": "High-risk: fall >=3 feet/5 stairs, axial load to head, high-speed (>100km/h) MVC/rollover/ejection, motorised recreational vehicle, or bicycle collision" },
                "paresthesias_in_extremities": { "type": "boolean", "description": "High-risk: paresthesias in the extremities" },
                "simple_rear_end_mvc": { "type": "boolean", "description": "Low-risk: simple rear-end motor vehicle collision (excludes pushed into oncoming traffic, hit by bus/large truck, rollover, or hit by high-speed vehicle)" },
                "sitting_position_in_ed": { "type": "boolean", "description": "Low-risk: sitting position in the emergency department" },
                "ambulatory_at_any_time_since_injury": { "type": "boolean", "description": "Low-risk: ambulatory at any time since the injury" },
                "delayed_onset_of_neck_pain": { "type": "boolean", "description": "Low-risk: delayed (not immediate) onset of neck pain" },
                "absence_of_midline_cspine_tenderness": { "type": "boolean", "description": "Low-risk: absence of midline cervical-spine tenderness" },
                "able_to_actively_rotate_neck_45_degrees": { "type": "boolean", "description": "Step 3, only assessed when no high-risk factor and at least one low-risk factor is present: able to actively rotate the neck 45 degrees left and right, regardless of pain" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: CanadianCspineInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> CanadianCspineInput {
        CanadianCspineInput {
            age_65_or_over: false,
            dangerous_mechanism: false,
            paresthesias_in_extremities: false,
            simple_rear_end_mvc: false,
            sitting_position_in_ed: false,
            ambulatory_at_any_time_since_injury: false,
            delayed_onset_of_neck_pain: false,
            absence_of_midline_cspine_tenderness: false,
            able_to_actively_rotate_neck_45_degrees: None,
        }
    }

    #[test]
    fn high_risk_mandates_radiography_without_rotation() {
        let mut i = none();
        i.age_65_or_over = true;
        let o = compute(&i).unwrap();
        assert_eq!(o, CspineOutcome::HighRisk);
        assert!(o.radiography_recommended());
    }

    #[test]
    fn no_risk_factors_at_all_requires_radiography() {
        let o = compute(&none()).unwrap();
        assert_eq!(o, CspineOutcome::NoLowRiskFactor);
        assert!(o.radiography_recommended());
    }

    #[test]
    fn low_risk_but_rotation_not_supplied() {
        let mut i = none();
        i.ambulatory_at_any_time_since_injury = true;
        let o = compute(&i).unwrap();
        assert_eq!(o, CspineOutcome::RotationNotAssessed);
    }

    #[test]
    fn low_risk_and_able_to_rotate_means_no_radiography() {
        let mut i = none();
        i.ambulatory_at_any_time_since_injury = true;
        i.able_to_actively_rotate_neck_45_degrees = Some(true);
        let o = compute(&i).unwrap();
        assert_eq!(o, CspineOutcome::LowRiskAbleToRotate);
        assert!(!o.radiography_recommended());
    }

    #[test]
    fn low_risk_and_unable_to_rotate_requires_radiography() {
        let mut i = none();
        i.absence_of_midline_cspine_tenderness = true;
        i.able_to_actively_rotate_neck_45_degrees = Some(false);
        let o = compute(&i).unwrap();
        assert_eq!(o, CspineOutcome::LowRiskUnableToRotate);
        assert!(o.radiography_recommended());
    }

    #[test]
    fn high_risk_takes_precedence_over_low_risk_and_rotation() {
        let mut i = none();
        i.dangerous_mechanism = true;
        i.ambulatory_at_any_time_since_injury = true;
        i.able_to_actively_rotate_neck_45_degrees = Some(true);
        assert_eq!(compute(&i).unwrap(), CspineOutcome::HighRisk);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let mut typed = none();
        typed.sitting_position_in_ed = true;
        typed.able_to_actively_rotate_neck_45_degrees = Some(true);
        let value = json!({
            "age_65_or_over": false, "dangerous_mechanism": false, "paresthesias_in_extremities": false,
            "simple_rear_end_mvc": false, "sitting_position_in_ed": true, "ambulatory_at_any_time_since_injury": false,
            "delayed_onset_of_neck_pain": false, "absence_of_midline_cspine_tenderness": false,
            "able_to_actively_rotate_neck_45_degrees": true
        });
        let dynamic = CanadianCspine.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(false));
    }

    #[test]
    fn rejects_missing_field() {
        assert!(
            CanadianCspine
                .calculate(&json!({"age_65_or_over": true}))
                .is_err()
        );
    }
}
