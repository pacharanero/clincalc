// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rockall score for acute upper gastrointestinal bleeding.
//!
//! Two related scores from the same five-variable system (Rockall et al,
//! 1996): the **pre-endoscopy** (clinical) score from age, shock, and
//! comorbidity alone (0-7), and the **full** score, which adds the
//! post-endoscopy diagnosis and any major stigmata of recent haemorrhage
//! (0-11). Supplying `diagnosis` and `major_stigmata_of_recent_haemorrhage`
//! computes the full score; omitting both reports the pre-endoscopy score
//! only, matching how the score is actually used in practice - once at first
//! assessment, before endoscopy, and again afterwards.
//!
//! This pairs with the shipped [`crate::calculators::glasgow_blatchford`]
//! score, which is pre-endoscopy-only and does not require an endoscopic
//! diagnosis or comorbidity assessment; Rockall is the complementary
//! instrument that extends into the post-endoscopy risk-stratification step
//! Glasgow-Blatchford does not cover.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "rockall";

/// Primary citation and the validation study behind the commonly-cited risk
/// bands used in [`compute`]'s interpretation.
pub const REFERENCE: &str = "Rockall TA, Logan RF, Devlin HB, Northfield TC. Risk assessment after acute upper \
gastrointestinal haemorrhage. Gut. 1996;38(3):316-321. doi:10.1136/gut.38.3.316. Risk bands per: Vreeburg EM, \
Terwee CB, Snel P, et al. Validation of the Rockall risk scoring system in upper gastrointestinal bleeding. Gut. \
1999;44(3):331-335. doi:10.1136/gut.44.3.331.";

/// Distribution licence: independently implemented from the published method.
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Published clinical scoring method - independently implemented from the primary literature",
    "https://doi.org/10.1136/gut.38.3.316",
);

/// Comorbidity category, in the three published tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Comorbidity {
    /// Nil major comorbidity.
    NilMajor,
    /// Cardiac failure, ischaemic heart disease, or any other major comorbidity.
    CardiacFailureIschaemicHeartDiseaseOrOtherMajor,
    /// Renal failure, liver failure, or disseminated malignancy.
    RenalFailureLiverFailureOrDisseminatedMalignancy,
}

impl Comorbidity {
    fn points(self) -> u8 {
        match self {
            Self::NilMajor => 0,
            Self::CardiacFailureIschaemicHeartDiseaseOrOtherMajor => 2,
            Self::RenalFailureLiverFailureOrDisseminatedMalignancy => 3,
        }
    }
}

/// Endoscopic diagnosis, in the three published tiers. Only supplied once
/// endoscopy has been performed; selects the full score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Diagnosis {
    /// Mallory-Weiss tear, or no lesion identified and no stigmata of recent haemorrhage.
    MalloryWeissOrNoLesionIdentified,
    /// All other diagnoses (e.g. peptic ulcer, oesophagitis, varices).
    AllOtherDiagnoses,
    /// Malignancy of the upper gastrointestinal tract.
    UpperGiMalignancy,
}

impl Diagnosis {
    fn points(self) -> u8 {
        match self {
            Self::MalloryWeissOrNoLesionIdentified => 0,
            Self::AllOtherDiagnoses => 1,
            Self::UpperGiMalignancy => 2,
        }
    }
}

/// Which score was computed, depending on whether endoscopic findings were supplied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreKind {
    /// Age + shock + comorbidity only (0-7), available before endoscopy.
    PreEndoscopy,
    /// Pre-endoscopy score plus diagnosis and major stigmata (0-11).
    Full,
}

/// Inputs for the Rockall score. `diagnosis` and
/// `major_stigmata_of_recent_haemorrhage` are endoscopic findings: supply
/// both to compute the full score, or omit both for the pre-endoscopy score.
/// Supplying only one is rejected as ambiguous.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RockallInput {
    /// Age in years.
    pub age: u8,
    /// Systolic blood pressure, mmHg, at presentation.
    pub systolic_bp_mm_hg: f64,
    /// Heart rate, beats per minute, at presentation.
    pub heart_rate_bpm: f64,
    pub comorbidity: Comorbidity,
    /// Endoscopic diagnosis. Supply alongside `major_stigmata_of_recent_haemorrhage`
    /// to compute the full post-endoscopy score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnosis: Option<Diagnosis>,
    /// Major stigmata of recent haemorrhage at endoscopy (blood in the upper GI
    /// tract, an adherent clot, or a visible or spurting vessel); `false` for
    /// none or a dark spot only. Supply alongside `diagnosis` to compute the
    /// full post-endoscopy score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub major_stigmata_of_recent_haemorrhage: Option<bool>,
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RockallOutcome {
    pub age_points: u8,
    pub shock_points: u8,
    pub comorbidity_points: u8,
    /// `None` when only the pre-endoscopy score was computed.
    pub diagnosis_points: Option<u8>,
    /// `None` when only the pre-endoscopy score was computed.
    pub stigmata_points: Option<u8>,
    /// The pre-endoscopy (clinical) subtotal, 0-7.
    pub pre_endoscopy_score: u8,
    /// The full score (0-11) when endoscopic findings were supplied, otherwise
    /// equal to `pre_endoscopy_score`.
    pub total_score: u8,
    pub score_kind: ScoreKind,
    pub interpretation: String,
}

fn age_points(age: u8) -> u8 {
    if age >= 80 {
        2
    } else if age >= 60 {
        1
    } else {
        0
    }
}

fn shock_points(systolic_bp_mm_hg: f64, heart_rate_bpm: f64) -> u8 {
    if systolic_bp_mm_hg < 100.0 {
        2
    } else if heart_rate_bpm > 100.0 {
        1
    } else {
        0
    }
}

fn render_interpretation(score_kind: ScoreKind, total_score: u8) -> String {
    match score_kind {
        ScoreKind::PreEndoscopy => {
            let band = if total_score == 0 {
                "Rockall's own cohort found a pre-endoscopy score of 0 identified a low-risk group \
(around 15% of all presentations) potentially suitable for early discharge or outpatient management, \
subject to clinical judgement and local pathway"
            } else {
                "a non-zero pre-endoscopy score does not by itself determine admission or discharge; \
continue assessment and proceed to endoscopy"
            };
            format!(
                "Rockall pre-endoscopy score {total_score} of 7 (age + shock + comorbidity only; \
endoscopic diagnosis and stigmata not yet supplied). {band}."
            )
        }
        ScoreKind::Full => {
            let band = match total_score {
                0..=2 => "low risk",
                3..=5 => "intermediate risk",
                _ => "high risk",
            };
            format!(
                "Rockall full score {total_score} of 11 (post-endoscopy): {band} of rebleeding and \
mortality, per the Vreeburg et al validation bands (0-2 low, 3-5 intermediate, 6 or more high). The \
score informs risk stratification alongside clinical judgement; it does not by itself determine \
disposition or treatment."
            )
        }
    }
}

/// Pure scoring: the pre-endoscopy score always; the full score additionally
/// when both `diagnosis` and `major_stigmata_of_recent_haemorrhage` are supplied.
pub fn compute(input: &RockallInput) -> Result<RockallOutcome, CalcError> {
    if !input.systolic_bp_mm_hg.is_finite() || !input.heart_rate_bpm.is_finite() {
        return Err(CalcError::InvalidInput(
            "systolic_bp_mm_hg and heart_rate_bpm must be finite numbers".into(),
        ));
    }
    if input.systolic_bp_mm_hg <= 0.0 || input.heart_rate_bpm <= 0.0 {
        return Err(CalcError::InvalidInput(
            "systolic_bp_mm_hg and heart_rate_bpm must be positive".into(),
        ));
    }
    if input.diagnosis.is_some() != input.major_stigmata_of_recent_haemorrhage.is_some() {
        return Err(CalcError::InvalidInput(
            "supply both diagnosis and major_stigmata_of_recent_haemorrhage together for the full \
score, or neither for the pre-endoscopy score"
                .into(),
        ));
    }

    let age_points = age_points(input.age);
    let shock_points = shock_points(input.systolic_bp_mm_hg, input.heart_rate_bpm);
    let comorbidity_points = input.comorbidity.points();
    let pre_endoscopy_score = age_points + shock_points + comorbidity_points;

    let (diagnosis_points, stigmata_points, total_score, score_kind) =
        match (input.diagnosis, input.major_stigmata_of_recent_haemorrhage) {
            (Some(diagnosis), Some(major_srh)) => {
                let diagnosis_points = diagnosis.points();
                let stigmata_points = if major_srh { 2 } else { 0 };
                (
                    Some(diagnosis_points),
                    Some(stigmata_points),
                    pre_endoscopy_score + diagnosis_points + stigmata_points,
                    ScoreKind::Full,
                )
            }
            _ => (None, None, pre_endoscopy_score, ScoreKind::PreEndoscopy),
        };

    Ok(RockallOutcome {
        age_points,
        shock_points,
        comorbidity_points,
        diagnosis_points,
        stigmata_points,
        pre_endoscopy_score,
        total_score,
        score_kind,
        interpretation: render_interpretation(score_kind, total_score),
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &RockallInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("age_points".into(), json!(o.age_points));
    working.insert("shock_points".into(), json!(o.shock_points));
    working.insert("comorbidity_points".into(), json!(o.comorbidity_points));
    if let Some(diagnosis_points) = o.diagnosis_points {
        working.insert("diagnosis_points".into(), json!(diagnosis_points));
    }
    if let Some(stigmata_points) = o.stigmata_points {
        working.insert("stigmata_points".into(), json!(stigmata_points));
    }
    working.insert("pre_endoscopy_score".into(), json!(o.pre_endoscopy_score));
    working.insert("score_kind".into(), json!(o.score_kind));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.total_score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Rockall;

impl Calculator for Rockall {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Rockall Score (upper GI bleeding)"
    }

    fn description(&self) -> &'static str {
        "Pre-endoscopy and full (post-endoscopy) risk of rebleeding and mortality after acute upper \
gastrointestinal haemorrhage (Rockall 1996), complementing the pre-endoscopy-only Glasgow-Blatchford score."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "RockallInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["age", "systolic_bp_mm_hg", "heart_rate_bpm", "comorbidity"],
            "properties": {
                "age": { "type": "integer", "minimum": 0, "maximum": 130, "description": "Age in years" },
                "systolic_bp_mm_hg": { "type": "number", "exclusiveMinimum": 0, "description": "Systolic blood pressure, mmHg, at presentation" },
                "heart_rate_bpm": { "type": "number", "exclusiveMinimum": 0, "description": "Heart rate, beats per minute, at presentation" },
                "comorbidity": {
                    "type": "string",
                    "enum": [
                        "nil_major",
                        "cardiac_failure_ischaemic_heart_disease_or_other_major",
                        "renal_failure_liver_failure_or_disseminated_malignancy"
                    ],
                    "description": "Comorbidity category in the three published tiers"
                },
                "diagnosis": {
                    "type": "string",
                    "enum": ["mallory_weiss_or_no_lesion_identified", "all_other_diagnoses", "upper_gi_malignancy"],
                    "description": "Endoscopic diagnosis; supply with major_stigmata_of_recent_haemorrhage for the full post-endoscopy score"
                },
                "major_stigmata_of_recent_haemorrhage": {
                    "type": "boolean",
                    "description": "Blood in the upper GI tract, an adherent clot, or a visible/spurting vessel at endoscopy; supply with diagnosis for the full post-endoscopy score"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: RockallInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(age: u8, systolic_bp_mm_hg: f64, heart_rate_bpm: f64) -> RockallInput {
        RockallInput {
            age,
            systolic_bp_mm_hg,
            heart_rate_bpm,
            comorbidity: Comorbidity::NilMajor,
            diagnosis: None,
            major_stigmata_of_recent_haemorrhage: None,
        }
    }

    // --- Per-criterion point values, verified against the original table ---

    #[test]
    fn age_bands() {
        assert_eq!(age_points(30), 0);
        assert_eq!(age_points(59), 0);
        assert_eq!(age_points(60), 1);
        assert_eq!(age_points(79), 1);
        assert_eq!(age_points(80), 2);
        assert_eq!(age_points(95), 2);
    }

    #[test]
    fn shock_bands() {
        assert_eq!(shock_points(120.0, 80.0), 0, "no shock");
        assert_eq!(
            shock_points(110.0, 110.0),
            1,
            "tachycardia without hypotension"
        );
        assert_eq!(
            shock_points(95.0, 80.0),
            2,
            "hypotension, regardless of pulse"
        );
        assert_eq!(
            shock_points(95.0, 120.0),
            2,
            "hypotension takes precedence over tachycardia"
        );
    }

    #[test]
    fn comorbidity_points_match_table() {
        assert_eq!(Comorbidity::NilMajor.points(), 0);
        assert_eq!(
            Comorbidity::CardiacFailureIschaemicHeartDiseaseOrOtherMajor.points(),
            2
        );
        assert_eq!(
            Comorbidity::RenalFailureLiverFailureOrDisseminatedMalignancy.points(),
            3
        );
    }

    #[test]
    fn diagnosis_points_match_table() {
        assert_eq!(Diagnosis::MalloryWeissOrNoLesionIdentified.points(), 0);
        assert_eq!(Diagnosis::AllOtherDiagnoses.points(), 1);
        assert_eq!(Diagnosis::UpperGiMalignancy.points(), 2);
    }

    // --- compute() ---

    #[test]
    fn pre_endoscopy_score_with_no_risk_factors_is_zero() {
        let i = base(40, 120.0, 80.0);
        let o = compute(&i).unwrap();
        assert_eq!(o.total_score, 0);
        assert_eq!(o.pre_endoscopy_score, 0);
        assert_eq!(o.score_kind, ScoreKind::PreEndoscopy);
    }

    /// 85yo, hypotensive, renal failure: age 2 + shock 2 + comorbidity 3 = 7 (max pre-endoscopy).
    #[test]
    fn pre_endoscopy_max_score_is_seven() {
        let mut i = base(85, 90.0, 110.0);
        i.comorbidity = Comorbidity::RenalFailureLiverFailureOrDisseminatedMalignancy;
        let o = compute(&i).unwrap();
        assert_eq!(o.pre_endoscopy_score, 7);
        assert_eq!(o.total_score, 7);
    }

    /// Same patient, now with malignancy and major stigmata: 7 + 2 + 2 = 11 (max full score).
    #[test]
    fn full_max_score_is_eleven() {
        let mut i = base(85, 90.0, 110.0);
        i.comorbidity = Comorbidity::RenalFailureLiverFailureOrDisseminatedMalignancy;
        i.diagnosis = Some(Diagnosis::UpperGiMalignancy);
        i.major_stigmata_of_recent_haemorrhage = Some(true);
        let o = compute(&i).unwrap();
        assert_eq!(o.pre_endoscopy_score, 7);
        assert_eq!(o.total_score, 11);
        assert_eq!(o.score_kind, ScoreKind::Full);
        assert_eq!(o.diagnosis_points, Some(2));
        assert_eq!(o.stigmata_points, Some(2));
    }

    #[test]
    fn full_score_with_mallory_weiss_and_no_stigmata_adds_nothing() {
        let mut i = base(40, 120.0, 80.0);
        i.diagnosis = Some(Diagnosis::MalloryWeissOrNoLesionIdentified);
        i.major_stigmata_of_recent_haemorrhage = Some(false);
        let o = compute(&i).unwrap();
        assert_eq!(o.total_score, 0);
        assert_eq!(o.score_kind, ScoreKind::Full);
    }

    #[test]
    fn rejects_diagnosis_without_stigmata() {
        let mut i = base(40, 120.0, 80.0);
        i.diagnosis = Some(Diagnosis::AllOtherDiagnoses);
        assert!(compute(&i).is_err());
    }

    #[test]
    fn rejects_stigmata_without_diagnosis() {
        let mut i = base(40, 120.0, 80.0);
        i.major_stigmata_of_recent_haemorrhage = Some(true);
        assert!(compute(&i).is_err());
    }

    #[test]
    fn rejects_non_positive_vitals() {
        assert!(compute(&base(40, 0.0, 80.0)).is_err());
        assert!(compute(&base(40, 120.0, 0.0)).is_err());
        assert!(compute(&base(40, -10.0, 80.0)).is_err());
    }

    #[test]
    fn interpretation_bands_match_vreeburg_cutoffs() {
        let mut i = base(40, 120.0, 80.0);
        i.diagnosis = Some(Diagnosis::MalloryWeissOrNoLesionIdentified);
        i.major_stigmata_of_recent_haemorrhage = Some(false);
        let low = compute(&i).unwrap();
        assert!(low.interpretation.contains("low risk"));

        i.comorbidity = Comorbidity::CardiacFailureIschaemicHeartDiseaseOrOtherMajor;
        i.diagnosis = Some(Diagnosis::AllOtherDiagnoses);
        let intermediate = compute(&i).unwrap(); // 0+0+2 + 1+0 = 3
        assert_eq!(intermediate.total_score, 3);
        assert!(intermediate.interpretation.contains("intermediate risk"));

        i.comorbidity = Comorbidity::RenalFailureLiverFailureOrDisseminatedMalignancy;
        i.diagnosis = Some(Diagnosis::UpperGiMalignancy);
        i.major_stigmata_of_recent_haemorrhage = Some(true);
        let high = compute(&i).unwrap(); // 0+0+3 + 2+2 = 7
        assert_eq!(high.total_score, 7);
        assert!(high.interpretation.contains("high risk"));
    }

    #[test]
    fn dynamic_calculate_matches_typed_build_response() {
        let calc = Rockall;
        let input = json!({
            "age": 65, "systolic_bp_mm_hg": 95.0, "heart_rate_bpm": 105.0,
            "comorbidity": "cardiac_failure_ischaemic_heart_disease_or_other_major"
        });
        let dynamic = calc.calculate(&input).unwrap();
        let typed = {
            let mut i = base(65, 95.0, 105.0);
            i.comorbidity = Comorbidity::CardiacFailureIschaemicHeartDiseaseOrOtherMajor;
            i
        };
        let typed_response = build_response(&typed).unwrap();
        assert_eq!(dynamic.result, typed_response.result);
    }

    #[test]
    fn schema_closed_and_lists_required_fields() {
        let calc = Rockall;
        let schema = calc.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        let required = schema["required"].as_array().unwrap();
        for field in ["age", "systolic_bp_mm_hg", "heart_rate_bpm", "comorbidity"] {
            assert!(required.iter().any(|v| v == field));
        }
    }
}
