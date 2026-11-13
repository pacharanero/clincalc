// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Canadian Syncope Risk Score - 30-day risk of a serious adverse event
//! after emergency department assessment of syncope (Thiruganasambandamoorthy
//! et al. CMAJ 2016).
//!
//! Nine weighted items across clinical evaluation, investigations, and the
//! ED's own working diagnosis sum to a score from -3 to 11. The ED diagnosis
//! items (vasovagal vs cardiac syncope) are mutually exclusive by
//! definition, since they represent alternative working diagnoses for the
//! same presentation.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "canadian_syncope_risk";

pub const REFERENCE: &str = "Thiruganasambandamoorthy V, Kwong K, Wells GA, et al. Development of the Canadian \
Syncope Risk Score to predict serious adverse events after emergency department assessment of syncope. \
CMAJ. 2016;188(12):E289-E298.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.1503/cmaj.151469",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanadianSyncopeRiskInput {
    /// Triggered by being in a warm, crowded place, prolonged standing,
    /// fear, emotion, or pain. Scores -1.
    pub predisposition_to_vasovagal_symptoms: bool,
    /// Coronary or valvular heart disease, cardiomyopathy, congestive heart
    /// failure, or non-sinus rhythm. Scores +1.
    pub history_of_heart_disease: bool,
    /// Any systolic BP reading under 90 or over 180 mmHg from triage until
    /// disposition from the ED. Scores +2.
    pub systolic_bp_under_90_or_over_180: bool,
    /// Troponin above the local 99th-percentile upper reference limit.
    /// Scores +2.
    pub elevated_troponin: bool,
    /// QRS axis under -30 degrees or over 100 degrees. Scores +1.
    pub abnormal_qrs_axis: bool,
    /// QRS duration over 130 ms. Scores +1.
    pub qrs_duration_over_130ms: bool,
    /// Corrected QT interval over 480 ms. Scores +2.
    pub qtc_over_480ms: bool,
    /// ED diagnosis of vasovagal syncope. Scores -2. Mutually exclusive
    /// with `ed_diagnosis_cardiac_syncope`.
    pub ed_diagnosis_vasovagal_syncope: bool,
    /// ED diagnosis of cardiac syncope. Scores +2. Mutually exclusive with
    /// `ed_diagnosis_vasovagal_syncope`.
    pub ed_diagnosis_cardiac_syncope: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncopeRiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl SyncopeRiskLevel {
    fn from_score(score: i32) -> Self {
        match score {
            s if s <= -2 => Self::VeryLow,
            -1..=0 => Self::Low,
            1..=3 => Self::Medium,
            4..=5 => Self::High,
            _ => Self::VeryHigh,
        }
    }
    fn slug(self) -> &'static str {
        match self {
            Self::VeryLow => "very_low",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::VeryHigh => "very_high",
        }
    }
    /// 30-day serious-outcome rate, as reported alongside the derivation
    /// cohort in point-of-care summaries of this score.
    fn serious_outcome_rate(self) -> &'static str {
        match self {
            Self::VeryLow => "0.2%",
            Self::Low => "0.7%",
            Self::Medium => "8%",
            Self::High => "19%",
            Self::VeryHigh => "51%",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanadianSyncopeRiskOutcome {
    pub score: i32,
    pub risk_level: SyncopeRiskLevel,
}

pub fn compute(input: &CanadianSyncopeRiskInput) -> Result<CanadianSyncopeRiskOutcome, CalcError> {
    if input.ed_diagnosis_vasovagal_syncope && input.ed_diagnosis_cardiac_syncope {
        return Err(CalcError::InvalidInput(
            "ed_diagnosis_vasovagal_syncope and ed_diagnosis_cardiac_syncope are mutually exclusive working diagnoses".into(),
        ));
    }

    let score = -i32::from(input.predisposition_to_vasovagal_symptoms)
        + i32::from(input.history_of_heart_disease)
        + 2 * i32::from(input.systolic_bp_under_90_or_over_180)
        + 2 * i32::from(input.elevated_troponin)
        + i32::from(input.abnormal_qrs_axis)
        + i32::from(input.qrs_duration_over_130ms)
        + 2 * i32::from(input.qtc_over_480ms)
        - 2 * i32::from(input.ed_diagnosis_vasovagal_syncope)
        + 2 * i32::from(input.ed_diagnosis_cardiac_syncope);

    Ok(CanadianSyncopeRiskOutcome {
        score,
        risk_level: SyncopeRiskLevel::from_score(score),
    })
}

pub fn build_response(input: &CanadianSyncopeRiskInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("risk_level".into(), json!(o.risk_level.slug()));
    working.insert(
        "thirty_day_serious_outcome_rate".into(),
        json!(o.risk_level.serious_outcome_rate()),
    );

    let interpretation = format!(
        "Canadian Syncope Risk Score {} (range -3 to 11), risk level {} (30-day serious-outcome rate {} in the derivation/validation cohorts). Serious outcomes include death, arrhythmia, structural heart disease, other cardiac events, pulmonary embolism, and significant haemorrhage.",
        o.score,
        o.risk_level.slug(),
        o.risk_level.serious_outcome_rate()
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct CanadianSyncopeRisk;
impl Calculator for CanadianSyncopeRisk {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Canadian Syncope Risk Score"
    }
    fn description(&self) -> &'static str {
        "30-day risk of a serious adverse event after emergency department assessment of syncope."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "CanadianSyncopeRiskInput", "type": "object",
            "additionalProperties": false,
            "required": [
                "predisposition_to_vasovagal_symptoms", "history_of_heart_disease", "systolic_bp_under_90_or_over_180",
                "elevated_troponin", "abnormal_qrs_axis", "qrs_duration_over_130ms", "qtc_over_480ms",
                "ed_diagnosis_vasovagal_syncope", "ed_diagnosis_cardiac_syncope"
            ],
            "properties": {
                "predisposition_to_vasovagal_symptoms": { "type": "boolean", "description": "Triggered by a warm/crowded place, prolonged standing, fear, emotion, or pain (-1 point)" },
                "history_of_heart_disease": { "type": "boolean", "description": "Coronary/valvular heart disease, cardiomyopathy, CHF, or non-sinus rhythm (+1 point)" },
                "systolic_bp_under_90_or_over_180": { "type": "boolean", "description": "Any systolic BP reading <90 or >180 mmHg from triage to ED disposition (+2 points)" },
                "elevated_troponin": { "type": "boolean", "description": "Troponin above the local 99th-percentile upper reference limit (+2 points)" },
                "abnormal_qrs_axis": { "type": "boolean", "description": "QRS axis <-30 or >100 degrees (+1 point)" },
                "qrs_duration_over_130ms": { "type": "boolean", "description": "QRS duration >130 ms (+1 point)" },
                "qtc_over_480ms": { "type": "boolean", "description": "Corrected QT interval >480 ms (+2 points)" },
                "ed_diagnosis_vasovagal_syncope": { "type": "boolean", "description": "ED working diagnosis of vasovagal syncope (-2 points); mutually exclusive with ed_diagnosis_cardiac_syncope" },
                "ed_diagnosis_cardiac_syncope": { "type": "boolean", "description": "ED working diagnosis of cardiac syncope (+2 points); mutually exclusive with ed_diagnosis_vasovagal_syncope" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: CanadianSyncopeRiskInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> CanadianSyncopeRiskInput {
        CanadianSyncopeRiskInput {
            predisposition_to_vasovagal_symptoms: false,
            history_of_heart_disease: false,
            systolic_bp_under_90_or_over_180: false,
            elevated_troponin: false,
            abnormal_qrs_axis: false,
            qrs_duration_over_130ms: false,
            qtc_over_480ms: false,
            ed_diagnosis_vasovagal_syncope: false,
            ed_diagnosis_cardiac_syncope: false,
        }
    }

    #[test]
    fn all_absent_scores_zero_and_is_low_risk() {
        let o = compute(&none()).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.risk_level, SyncopeRiskLevel::Low);
    }

    #[test]
    fn vasovagal_predisposition_and_diagnosis_score_very_low() {
        let mut i = none();
        i.predisposition_to_vasovagal_symptoms = true;
        i.ed_diagnosis_vasovagal_syncope = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, -3);
        assert_eq!(o.risk_level, SyncopeRiskLevel::VeryLow);
    }

    #[test]
    fn cardiac_diagnosis_plus_troponin_and_qtc_scores_very_high() {
        let mut i = none();
        i.ed_diagnosis_cardiac_syncope = true;
        i.elevated_troponin = true;
        i.qtc_over_480ms = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 6);
        assert_eq!(o.risk_level, SyncopeRiskLevel::VeryHigh);
    }

    #[test]
    fn mutually_exclusive_diagnoses_are_rejected() {
        let mut i = none();
        i.ed_diagnosis_vasovagal_syncope = true;
        i.ed_diagnosis_cardiac_syncope = true;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn worked_example_from_aafp_point_of_care_guide() {
        // -1 (predisposition) + 2 (QTc >480) - 2 (vasovagal diagnosis) = -1.
        let mut i = none();
        i.predisposition_to_vasovagal_symptoms = true;
        i.qtc_over_480ms = true;
        i.ed_diagnosis_vasovagal_syncope = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, -1);
        assert_eq!(o.risk_level, SyncopeRiskLevel::Low);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let mut typed = none();
        typed.history_of_heart_disease = true;
        typed.abnormal_qrs_axis = true;
        let value = json!({
            "predisposition_to_vasovagal_symptoms": false, "history_of_heart_disease": true,
            "systolic_bp_under_90_or_over_180": false, "elevated_troponin": false, "abnormal_qrs_axis": true,
            "qrs_duration_over_130ms": false, "qtc_over_480ms": false, "ed_diagnosis_vasovagal_syncope": false,
            "ed_diagnosis_cardiac_syncope": false
        });
        let dynamic = CanadianSyncopeRisk.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(2));
    }

    #[test]
    fn rejects_missing_field() {
        assert!(
            CanadianSyncopeRisk
                .calculate(&json!({"history_of_heart_disease": true}))
                .is_err()
        );
    }
}
