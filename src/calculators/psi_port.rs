// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Pneumonia Severity Index (PSI) / PORT Score for community-acquired
//! pneumonia, complementing the shipped `curb65`.
//!
//! Two-step rule: Step 1 assigns Class I by clinical rule alone (age 50 or
//! under, no listed comorbidity, no abnormal vital sign or mental status,
//! not a nursing-home resident) with no labs needed. If Step 1 criteria are
//! not met, Step 2 sums points across demographics, comorbidities, physical
//! findings, and labs/imaging to assign Class II-V. Labs are only required
//! when Step 2 actually applies, matching the rule's intent of avoiding
//! unnecessary testing in clearly low-risk patients.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "psi_port";
pub const REFERENCE: &str = "Fine MJ, Auble TE, Yealy DM, et al. A prediction rule to identify low-risk patients with community-acquired pneumonia. N Engl J Med. 1997;336(4):243-250.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain decision rule - implemented from the primary literature",
    "https://doi.org/10.1056/NEJM199701233360402",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PsiPortInput {
    pub age_years: u16,
    pub female: bool,
    pub nursing_home_resident: bool,
    pub neoplastic_disease: bool,
    pub liver_disease: bool,
    pub congestive_heart_failure: bool,
    pub cerebrovascular_disease: bool,
    pub renal_disease: bool,
    pub altered_mental_status: bool,
    pub respiratory_rate_30_or_more: bool,
    pub systolic_bp_below_90: bool,
    pub temperature_below_35_or_40_or_more: bool,
    pub pulse_125_or_more: bool,
    /// All seven lab/imaging fields below are required only when Step 1
    /// does not already assign Class I; omit all seven if Step 1 applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arterial_ph_below_7_35: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bun_30_or_more_mg_dl: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sodium_below_130: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glucose_250_or_more_mg_dl: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hematocrit_below_30_percent: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pao2_below_60_mm_hg: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pleural_effusion: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskClass {
    I,
    II,
    III,
    IV,
    V,
}
impl RiskClass {
    fn slug(self) -> &'static str {
        match self {
            Self::I => "I",
            Self::II => "II",
            Self::III => "III",
            Self::IV => "IV",
            Self::V => "V",
        }
    }
    fn mortality_percent(self) -> f64 {
        match self {
            Self::I => 0.1,
            Self::II => 0.6,
            Self::III => 2.8,
            Self::IV => 8.2,
            Self::V => 29.2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PsiPortOutcome {
    pub risk_class: RiskClass,
    pub points: Option<i32>,
}

pub fn compute(input: &PsiPortInput) -> Result<PsiPortOutcome, CalcError> {
    if !(1..=120).contains(&input.age_years) {
        return Err(CalcError::InvalidInput(
            "age_years must be between 1 and 120".to_string(),
        ));
    }

    let any_comorbidity = input.neoplastic_disease
        || input.liver_disease
        || input.congestive_heart_failure
        || input.cerebrovascular_disease
        || input.renal_disease;
    let any_abnormal_exam = input.altered_mental_status
        || input.respiratory_rate_30_or_more
        || input.systolic_bp_below_90
        || input.temperature_below_35_or_40_or_more
        || input.pulse_125_or_more;

    let class_i_eligible = input.age_years <= 50
        && !input.nursing_home_resident
        && !any_comorbidity
        && !any_abnormal_exam;

    if class_i_eligible {
        return Ok(PsiPortOutcome {
            risk_class: RiskClass::I,
            points: None,
        });
    }

    let labs = (
        input.arterial_ph_below_7_35,
        input.bun_30_or_more_mg_dl,
        input.sodium_below_130,
        input.glucose_250_or_more_mg_dl,
        input.hematocrit_below_30_percent,
        input.pao2_below_60_mm_hg,
        input.pleural_effusion,
    );
    let (ph, bun, sodium, glucose, hct, pao2, effusion) = match labs {
        (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g)) => (a, b, c, d, e, f, g),
        _ => {
            return Err(CalcError::InvalidInput(
                "all seven lab/imaging fields are required because Step 1 does not assign Class I for this patient".to_string(),
            ));
        }
    };

    let age_points = if input.female {
        (input.age_years as i32 - 10).max(0)
    } else {
        input.age_years as i32
    };

    let mut points = age_points;
    if input.nursing_home_resident {
        points += 10;
    }
    if input.neoplastic_disease {
        points += 30;
    }
    if input.liver_disease {
        points += 20;
    }
    if input.congestive_heart_failure {
        points += 10;
    }
    if input.cerebrovascular_disease {
        points += 10;
    }
    if input.renal_disease {
        points += 10;
    }
    if input.altered_mental_status {
        points += 20;
    }
    if input.respiratory_rate_30_or_more {
        points += 20;
    }
    if input.systolic_bp_below_90 {
        points += 20;
    }
    if input.temperature_below_35_or_40_or_more {
        points += 15;
    }
    if input.pulse_125_or_more {
        points += 10;
    }
    if ph {
        points += 30;
    }
    if bun {
        points += 20;
    }
    if sodium {
        points += 20;
    }
    if glucose {
        points += 10;
    }
    if hct {
        points += 10;
    }
    if pao2 {
        points += 10;
    }
    if effusion {
        points += 10;
    }

    let risk_class = if points <= 70 {
        RiskClass::II
    } else if points <= 90 {
        RiskClass::III
    } else if points <= 130 {
        RiskClass::IV
    } else {
        RiskClass::V
    };

    Ok(PsiPortOutcome {
        risk_class,
        points: Some(points),
    })
}

pub fn build_response(input: &PsiPortInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "mortality_30_day_percent".into(),
        json!(o.risk_class.mortality_percent()),
    );
    if let Some(points) = o.points {
        working.insert("points".into(), json!(points));
    }

    let disposition = match o.risk_class {
        RiskClass::I | RiskClass::II => "outpatient management is usually appropriate",
        RiskClass::III => "brief inpatient observation is often appropriate",
        RiskClass::IV => "inpatient admission is indicated",
        RiskClass::V => "inpatient admission is indicated, with assessment for ICU-level care",
    };

    let interpretation = match o.points {
        Some(points) => format!(
            "PSI/PORT Class {} ({} points): estimated 30-day mortality {}% (Fine et al 1997 cohort). {}. PSI predicts mortality, not the need for vasopressors or mechanical ventilation - a low-mortality class does not exclude a need for closer monitoring if other severity markers are present.",
            o.risk_class.slug(),
            points,
            o.risk_class.mortality_percent(),
            disposition
        ),
        None => format!(
            "PSI/PORT Class I by Step 1 clinical rule (age 50 or under, no listed comorbidity, no abnormal vital sign or mental status, not a nursing-home resident) - no points calculation needed. Estimated 30-day mortality {}%. {}.",
            o.risk_class.mortality_percent(),
            disposition
        ),
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.risk_class.slug()),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct PsiPort;
impl Calculator for PsiPort {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Pneumonia Severity Index / PORT Score"
    }
    fn description(&self) -> &'static str {
        "Community-acquired-pneumonia 30-day mortality risk class (I-V), complementing the shipped CURB-65."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "PsiPortInput", "type": "object", "additionalProperties": false,
            "required": [
                "age_years", "female", "nursing_home_resident", "neoplastic_disease", "liver_disease",
                "congestive_heart_failure", "cerebrovascular_disease", "renal_disease", "altered_mental_status",
                "respiratory_rate_30_or_more", "systolic_bp_below_90", "temperature_below_35_or_40_or_more", "pulse_125_or_more"
            ],
            "properties": {
                "age_years": { "type": "integer", "minimum": 1, "maximum": 120 },
                "female": { "type": "boolean" },
                "nursing_home_resident": { "type": "boolean" },
                "neoplastic_disease": { "type": "boolean" },
                "liver_disease": { "type": "boolean" },
                "congestive_heart_failure": { "type": "boolean" },
                "cerebrovascular_disease": { "type": "boolean" },
                "renal_disease": { "type": "boolean" },
                "altered_mental_status": { "type": "boolean" },
                "respiratory_rate_30_or_more": { "type": "boolean" },
                "systolic_bp_below_90": { "type": "boolean" },
                "temperature_below_35_or_40_or_more": { "type": "boolean", "description": "Temperature below 35 degC or 40 degC or more" },
                "pulse_125_or_more": { "type": "boolean" },
                "arterial_ph_below_7_35": { "type": "boolean", "description": "Required unless Step 1 already assigns Class I" },
                "bun_30_or_more_mg_dl": { "type": "boolean", "description": "Required unless Step 1 already assigns Class I" },
                "sodium_below_130": { "type": "boolean", "description": "Required unless Step 1 already assigns Class I" },
                "glucose_250_or_more_mg_dl": { "type": "boolean", "description": "Required unless Step 1 already assigns Class I" },
                "hematocrit_below_30_percent": { "type": "boolean", "description": "Required unless Step 1 already assigns Class I" },
                "pao2_below_60_mm_hg": { "type": "boolean", "description": "Required unless Step 1 already assigns Class I" },
                "pleural_effusion": { "type": "boolean", "description": "Required unless Step 1 already assigns Class I" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: PsiPortInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class_i_eligible() -> PsiPortInput {
        PsiPortInput {
            age_years: 35,
            female: false,
            nursing_home_resident: false,
            neoplastic_disease: false,
            liver_disease: false,
            congestive_heart_failure: false,
            cerebrovascular_disease: false,
            renal_disease: false,
            altered_mental_status: false,
            respiratory_rate_30_or_more: false,
            systolic_bp_below_90: false,
            temperature_below_35_or_40_or_more: false,
            pulse_125_or_more: false,
            arterial_ph_below_7_35: None,
            bun_30_or_more_mg_dl: None,
            sodium_below_130: None,
            glucose_250_or_more_mg_dl: None,
            hematocrit_below_30_percent: None,
            pao2_below_60_mm_hg: None,
            pleural_effusion: None,
        }
    }

    #[test]
    fn young_healthy_patient_is_class_i_without_labs() {
        let o = compute(&class_i_eligible()).unwrap();
        assert_eq!(o.risk_class, RiskClass::I);
        assert_eq!(o.points, None);
    }

    #[test]
    fn worked_example_age_72_matches_metricgate_reference() {
        // age 72 male, no comorbidities, RR 24 SBP 95 temp 38.4 pulse 115 (none abnormal by
        // PSI thresholds), labs: pH 7.35 (not <7.35), BUN 32 (>=30), Na 130 (not <130),
        // glucose 260 (>=250), Hct 28 (<30), PaO2 55 (<60), no effusion.
        // Age points 72 + BUN 20 + glucose 10 + Hct 10 + PaO2 10 = 122 -> Class IV.
        let input = PsiPortInput {
            age_years: 72,
            female: false,
            nursing_home_resident: false,
            neoplastic_disease: false,
            liver_disease: false,
            congestive_heart_failure: false,
            cerebrovascular_disease: false,
            renal_disease: false,
            altered_mental_status: false,
            respiratory_rate_30_or_more: false,
            systolic_bp_below_90: false,
            temperature_below_35_or_40_or_more: false,
            pulse_125_or_more: false,
            arterial_ph_below_7_35: Some(false),
            bun_30_or_more_mg_dl: Some(true),
            sodium_below_130: Some(false),
            glucose_250_or_more_mg_dl: Some(true),
            hematocrit_below_30_percent: Some(true),
            pao2_below_60_mm_hg: Some(true),
            pleural_effusion: Some(false),
        };
        let o = compute(&input).unwrap();
        assert_eq!(o.points, Some(122));
        assert_eq!(o.risk_class, RiskClass::IV);
    }

    #[test]
    fn female_age_adjustment_subtracts_10() {
        let mut input = class_i_eligible();
        input.age_years = 55; // over 50, so Step 1 does not apply
        input.female = true;
        input.arterial_ph_below_7_35 = Some(false);
        input.bun_30_or_more_mg_dl = Some(false);
        input.sodium_below_130 = Some(false);
        input.glucose_250_or_more_mg_dl = Some(false);
        input.hematocrit_below_30_percent = Some(false);
        input.pao2_below_60_mm_hg = Some(false);
        input.pleural_effusion = Some(false);
        let o = compute(&input).unwrap();
        // age 55 - 10 = 45 points -> Class II
        assert_eq!(o.points, Some(45));
        assert_eq!(o.risk_class, RiskClass::II);
    }

    #[test]
    fn comorbidity_disqualifies_class_i_and_requires_labs() {
        let mut input = class_i_eligible();
        input.neoplastic_disease = true;
        assert!(compute(&input).is_err());
    }

    #[test]
    fn points_above_130_is_class_v() {
        let mut input = class_i_eligible();
        input.age_years = 90;
        input.neoplastic_disease = true;
        input.altered_mental_status = true;
        input.arterial_ph_below_7_35 = Some(true);
        input.bun_30_or_more_mg_dl = Some(true);
        input.sodium_below_130 = Some(false);
        input.glucose_250_or_more_mg_dl = Some(false);
        input.hematocrit_below_30_percent = Some(false);
        input.pao2_below_60_mm_hg = Some(false);
        input.pleural_effusion = Some(false);
        let o = compute(&input).unwrap();
        // 90 + 30 + 20 + 30 + 20 = 190
        assert_eq!(o.points, Some(190));
        assert_eq!(o.risk_class, RiskClass::V);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age_years": 35, "female": false, "nursing_home_resident": false, "neoplastic_disease": false,
            "liver_disease": false, "congestive_heart_failure": false, "cerebrovascular_disease": false,
            "renal_disease": false, "altered_mental_status": false, "respiratory_rate_30_or_more": false,
            "systolic_bp_below_90": false, "temperature_below_35_or_40_or_more": false, "pulse_125_or_more": false
        });
        let dynamic = PsiPort.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&class_i_eligible()).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(PsiPort.calculate(&json!({"age_years": 35})).is_err());
    }
}
