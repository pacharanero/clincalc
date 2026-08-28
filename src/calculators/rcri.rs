// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Revised Cardiac Risk Index (RCRI, or Lee index).
//!
//! Lee et al. prospectively derived and validated this six-factor index in
//! 4,315 patients aged 50 years or older undergoing nonemergency major
//! noncardiac surgery with an expected stay of at least two days at one tertiary
//! hospital. Each factor scores one point: high-risk
//! surgery, ischaemic heart disease, congestive heart failure, cerebrovascular
//! disease, preoperative insulin treatment, and serum creatinine above
//! 2.0 mg/dL (176.8 umol/L).
//!
//! The original validation-cohort event rates are retained as historical
//! provenance, not presented as current patient-specific probabilities. The
//! cohort and outcome definition differ from many modern perioperative
//! populations and MACE definitions. The 2024 AHA/ACC multisociety guideline
//! supports validated risk tools such as RCRI within a stepwise assessment;
//! traditionally, RCRI >1 identifies elevated perioperative risk.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "rcri";

/// Primary derivation and current perioperative guideline.
pub const REFERENCE: &str = "Lee TH, Marcantonio ER, Mangione CM, et al. Derivation and prospective validation of a simple index for prediction of cardiac risk of major noncardiac surgery. Circulation. 1999;100(10):1043-1049. doi:10.1161/01.CIR.100.10.1043. Thompson A, Fleischmann KE, Smilowitz NR, et al. 2024 AHA/ACC/ACS/ASNC/HRS/SCA/SCCT/SCMR/SVM Guideline for Perioperative Cardiovascular Management for Noncardiac Surgery. Circulation. 2024;150(19):e351-e442. doi:10.1161/CIR.0000000000001285.";

/// Distribution licence: independently implemented from the published method.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published clinical scoring method - independently implemented from the primary literature",
    source_url: "https://doi.org/10.1161/01.CIR.100.10.1043",
};

/// The primary paper's creatinine threshold: >2.0 mg/dL, converted using
/// 1 mg/dL = 88.4 umol/L.
pub const CREATININE_THRESHOLD_UMOL_L: f64 = 176.8;

/// RCRI inputs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RcriInput {
    /// Confirm an adult patient is being assessed before noncardiac surgery.
    pub adult_noncardiac_surgery_candidate: bool,
    /// Whether the patient and operation match the original validation population.
    pub original_validation_population_matches: bool,
    /// Intraperitoneal, intrathoracic, or suprainguinal vascular surgery.
    pub high_risk_surgery: bool,
    /// Ischaemic heart disease as defined by Lee et al.
    pub ischemic_heart_disease: bool,
    /// Congestive heart failure as defined by Lee et al.
    pub congestive_heart_failure: bool,
    /// Prior stroke or transient ischaemic attack.
    pub cerebrovascular_disease: bool,
    /// Diabetes treated with insulin before surgery.
    pub preoperative_insulin_treatment: bool,
    /// Preoperative serum creatinine, umol/L.
    pub creatinine_umol_l: f64,
}

/// Risk class defined in the original RCRI publication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginalRiskClass {
    /// No predictors.
    I,
    /// One predictor.
    II,
    /// Two predictors.
    III,
    /// Three or more predictors.
    IV,
}

impl OriginalRiskClass {
    fn from_score(score: u8) -> Self {
        match score {
            0 => Self::I,
            1 => Self::II,
            2 => Self::III,
            _ => Self::IV,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::I => "I",
            Self::II => "II",
            Self::III => "III",
            Self::IV => "IV",
        }
    }

    fn validation_rate_percent(self) -> f64 {
        match self {
            Self::I => 0.4,
            Self::II => 0.9,
            Self::III => 6.6,
            Self::IV => 11.0,
        }
    }
}

/// Computed RCRI outcome with every component retained.
#[derive(Debug, Clone, PartialEq)]
pub struct RcriOutcome {
    pub high_risk_surgery_points: u8,
    pub ischemic_heart_disease_points: u8,
    pub congestive_heart_failure_points: u8,
    pub cerebrovascular_disease_points: u8,
    pub preoperative_insulin_treatment_points: u8,
    pub creatinine_points: u8,
    /// Total score, 0-6.
    pub score: u8,
    pub original_risk_class: OriginalRiskClass,
    /// Observed rate in the original 1,422-patient validation cohort, reported
    /// only when the caller confirms that the source population matches.
    pub original_validation_rate_percent: Option<f64>,
    /// Traditional guideline threshold: RCRI >1.
    pub elevated_risk_by_traditional_threshold: bool,
    pub interpretation: String,
}

/// Pure scoring.
pub fn compute(input: &RcriInput) -> Result<RcriOutcome, CalcError> {
    if !input.adult_noncardiac_surgery_candidate {
        return Err(CalcError::InvalidInput(
            "RCRI is intended for preoperative assessment of adults undergoing noncardiac surgery"
                .into(),
        ));
    }
    if !input.creatinine_umol_l.is_finite() || input.creatinine_umol_l <= 0.0 {
        return Err(CalcError::InvalidInput(
            "creatinine_umol_l must be finite and positive".into(),
        ));
    }

    let high_risk_surgery_points = u8::from(input.high_risk_surgery);
    let ischemic_heart_disease_points = u8::from(input.ischemic_heart_disease);
    let congestive_heart_failure_points = u8::from(input.congestive_heart_failure);
    let cerebrovascular_disease_points = u8::from(input.cerebrovascular_disease);
    let preoperative_insulin_treatment_points = u8::from(input.preoperative_insulin_treatment);
    let creatinine_points = u8::from(input.creatinine_umol_l > CREATININE_THRESHOLD_UMOL_L);

    let score = high_risk_surgery_points
        + ischemic_heart_disease_points
        + congestive_heart_failure_points
        + cerebrovascular_disease_points
        + preoperative_insulin_treatment_points
        + creatinine_points;
    let original_risk_class = OriginalRiskClass::from_score(score);
    let original_validation_rate_percent = input
        .original_validation_population_matches
        .then(|| original_risk_class.validation_rate_percent());
    let elevated_risk_by_traditional_threshold = score > 1;
    let threshold_text = if elevated_risk_by_traditional_threshold {
        "meets"
    } else {
        "does not meet"
    };

    let population_text = match original_validation_rate_percent {
        Some(rate) => format!(
            "The original validation cohort observed a {rate:.1}% in-hospital rate of its major cardiac complication composite (myocardial infarction, pulmonary oedema, ventricular fibrillation or primary cardiac arrest, or complete heart block) in this class; this is a historical cohort rate, not an individualized current probability or a contemporary MACE estimate."
        ),
        None => "The original validation population criteria are not met, so its historical in-hospital cardiac-complication rate is not reported.".into(),
    };
    let interpretation = format!(
        "RCRI score {score} of 6 (original class {}). {population_text} This score {threshold_text} the traditional RCRI >1 elevated-risk threshold. Use RCRI only within a stepwise perioperative assessment that also considers urgency, unstable cardiac conditions, functional capacity, procedure risk, and risk modifiers; the score alone must not determine testing or delay necessary surgery.",
        original_risk_class.label(),
    );

    Ok(RcriOutcome {
        high_risk_surgery_points,
        ischemic_heart_disease_points,
        congestive_heart_failure_points,
        cerebrovascular_disease_points,
        preoperative_insulin_treatment_points,
        creatinine_points,
        score,
        original_risk_class,
        original_validation_rate_percent,
        elevated_risk_by_traditional_threshold,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &RcriInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();

    working.insert(
        "adult_noncardiac_surgery_candidate".into(),
        json!(input.adult_noncardiac_surgery_candidate),
    );
    working.insert(
        "original_validation_population_matches".into(),
        json!(input.original_validation_population_matches),
    );
    working.insert("high_risk_surgery".into(), json!(input.high_risk_surgery));
    working.insert(
        "high_risk_surgery_points".into(),
        json!(o.high_risk_surgery_points),
    );
    working.insert(
        "ischemic_heart_disease".into(),
        json!(input.ischemic_heart_disease),
    );
    working.insert(
        "ischemic_heart_disease_points".into(),
        json!(o.ischemic_heart_disease_points),
    );
    working.insert(
        "congestive_heart_failure".into(),
        json!(input.congestive_heart_failure),
    );
    working.insert(
        "congestive_heart_failure_points".into(),
        json!(o.congestive_heart_failure_points),
    );
    working.insert(
        "cerebrovascular_disease".into(),
        json!(input.cerebrovascular_disease),
    );
    working.insert(
        "cerebrovascular_disease_points".into(),
        json!(o.cerebrovascular_disease_points),
    );
    working.insert(
        "preoperative_insulin_treatment".into(),
        json!(input.preoperative_insulin_treatment),
    );
    working.insert(
        "preoperative_insulin_treatment_points".into(),
        json!(o.preoperative_insulin_treatment_points),
    );
    working.insert("creatinine_umol_l".into(), json!(input.creatinine_umol_l));
    working.insert("creatinine_points".into(), json!(o.creatinine_points));
    working.insert("total_score".into(), json!(o.score));
    working.insert(
        "original_risk_class".into(),
        json!(o.original_risk_class.label()),
    );
    working.insert(
        "original_validation_outcome".into(),
        json!("myocardial infarction, pulmonary oedema, ventricular fibrillation or primary cardiac arrest, or complete heart block"),
    );
    working.insert(
        "original_validation_time_horizon".into(),
        json!("index surgical admission"),
    );
    if let Some(rate) = o.original_validation_rate_percent {
        working.insert(
            "original_validation_cohort_major_cardiac_complication_rate_percent".into(),
            json!(rate),
        );
    }
    working.insert(
        "elevated_risk_by_traditional_rcri_threshold".into(),
        json!(o.elevated_risk_by_traditional_threshold),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Dynamic calculator implementation.
pub struct Rcri;

impl Calculator for Rcri {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Revised Cardiac Risk Index (RCRI)"
    }

    fn description(&self) -> &'static str {
        "Six-factor Lee index for major cardiac complications after noncardiac surgery, used within stepwise adult preoperative assessment."
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
            "title": "RcriInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "adult_noncardiac_surgery_candidate", "original_validation_population_matches",
                "high_risk_surgery",
                "ischemic_heart_disease", "congestive_heart_failure",
                "cerebrovascular_disease", "preoperative_insulin_treatment",
                "creatinine_umol_l"
            ],
            "properties": {
                "adult_noncardiac_surgery_candidate": {
                    "type": "boolean",
                    "description": "Confirm preoperative assessment of an adult candidate for noncardiac surgery",
                    "definition": {
                        "concept": "RCRI intended context",
                        "statement": "Use as one component of stepwise cardiovascular risk assessment before noncardiac surgery in an adult.",
                        "excludes": ["Cardiac surgery", "Paediatric use", "Using RCRI alone to decide testing or to delay necessary surgery"],
                        "caveats": "Current guidelines place validated tools within a broader stepwise assessment. Complete original_validation_population_matches separately so historical event rates are reported only for patients matching the source cohort.",
                        "source": { "citation": "Lee TH et al. Circulation. 1999;100(10):1043-1049; Thompson A et al. Circulation. 2024;150(19):e351-e442.", "url": "https://doi.org/10.1161/CIR.0000000000001285" },
                        "status": "draft"
                    }
                },
                "original_validation_population_matches": {
                    "type": "boolean",
                    "description": "Whether the patient and operation match Lee et al.'s original validation population",
                    "definition": {
                        "concept": "Original RCRI validation population",
                        "statement": "True only for a patient aged 50 years or older undergoing nonemergency major noncardiac surgery with an expected hospital stay of at least two days.",
                        "includes": ["Age >=50 years", "Nonemergency major noncardiac surgery", "Expected hospital stay >=2 days"],
                        "excludes": ["Age under 50 years", "Emergency surgery", "Minor or ambulatory surgery", "Expected hospital stay under 2 days"],
                        "caveats": "The study was conducted at one tertiary hospital. When false, the RCRI score is still returned for guideline-supported adult perioperative assessment, but the original cohort's event rate is omitted.",
                        "source": { "citation": "Lee TH et al. Circulation. 1999;100(10):1043-1049.", "url": "https://doi.org/10.1161/01.CIR.100.10.1043" },
                        "status": "draft"
                    }
                },
                "high_risk_surgery": {
                    "type": "boolean",
                    "description": "High-risk surgery: intraperitoneal, intrathoracic, or suprainguinal vascular (1 point)",
                    "definition": {
                        "concept": "RCRI high-risk surgery",
                        "statement": "The planned operation is intraperitoneal, intrathoracic, or suprainguinal vascular surgery.",
                        "includes": ["Intraperitoneal surgery", "Intrathoracic surgery", "Suprainguinal vascular surgery"],
                        "excludes": ["A procedure is not high-risk for RCRI merely because it is labelled complex or urgent", "Infrainguinal vascular surgery does not satisfy the published suprainguinal criterion"],
                        "source": { "citation": "Lee TH et al. Circulation. 1999;100(10):1043-1049.", "url": "https://doi.org/10.1161/01.CIR.100.10.1043" },
                        "status": "draft"
                    }
                },
                "ischemic_heart_disease": {
                    "type": "boolean",
                    "description": "History of ischaemic heart disease (1 point)",
                    "definition": {
                        "concept": "RCRI ischaemic heart disease",
                        "statement": "History of myocardial infarction or positive exercise test, current chest pain considered due to myocardial ischaemia, nitrate therapy, or pathological Q waves on ECG.",
                        "includes": ["Prior myocardial infarction", "Prior positive exercise test", "Current chest pain considered ischaemic", "Nitrate therapy", "Pathological Q waves on ECG"],
                        "excludes": ["Prior coronary revascularisation alone does not score unless at least one published ischaemic-heart-disease criterion is also present"],
                        "source": { "citation": "Lee TH et al. Circulation. 1999;100(10):1043-1049.", "url": "https://doi.org/10.1161/01.CIR.100.10.1043" },
                        "status": "draft"
                    }
                },
                "congestive_heart_failure": {
                    "type": "boolean",
                    "description": "History of congestive heart failure (1 point)",
                    "definition": {
                        "concept": "RCRI congestive heart failure",
                        "statement": "Any documented history of congestive heart failure, pulmonary oedema, or paroxysmal nocturnal dyspnoea; bilateral rales or S3 gallop; or pulmonary vascular redistribution on chest radiograph.",
                        "includes": ["Documented history of congestive heart failure", "History of pulmonary oedema", "Paroxysmal nocturnal dyspnoea", "Bilateral rales", "S3 gallop", "Pulmonary vascular redistribution on chest radiograph"],
                        "excludes": ["Isolated peripheral oedema or dyspnoea without documented congestive heart failure or another published criterion"],
                        "source": { "citation": "Lee TH et al. Circulation. 1999;100(10):1043-1049.", "url": "https://doi.org/10.1161/01.CIR.100.10.1043" },
                        "status": "draft"
                    }
                },
                "cerebrovascular_disease": {
                    "type": "boolean",
                    "description": "History of stroke or transient ischaemic attack (1 point)",
                    "definition": {
                        "concept": "RCRI cerebrovascular disease",
                        "statement": "History of stroke or transient ischaemic attack.",
                        "includes": ["Prior stroke", "Prior transient ischaemic attack"],
                        "excludes": ["Dizziness, syncope, or other neurological symptoms without a diagnosed stroke or TIA"],
                        "source": { "citation": "Lee TH et al. Circulation. 1999;100(10):1043-1049.", "url": "https://doi.org/10.1161/01.CIR.100.10.1043" },
                        "status": "draft"
                    }
                },
                "preoperative_insulin_treatment": {
                    "type": "boolean",
                    "description": "Diabetes treated with insulin before surgery (1 point)",
                    "definition": {
                        "concept": "RCRI preoperative insulin treatment",
                        "statement": "The patient receives insulin treatment for diabetes before surgery.",
                        "includes": ["Established insulin therapy for diabetes before surgery"],
                        "excludes": ["Diabetes managed without insulin", "A one-off perioperative correction dose in a patient not previously treated with insulin"],
                        "source": { "citation": "Lee TH et al. Circulation. 1999;100(10):1043-1049.", "url": "https://doi.org/10.1161/01.CIR.100.10.1043" },
                        "status": "draft"
                    }
                },
                "creatinine_umol_l": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Preoperative serum creatinine, umol/L (>176.8 scores 1 point)",
                    "definition": {
                        "concept": "RCRI preoperative creatinine",
                        "statement": "Preoperative serum creatinine above 2.0 mg/dL scores one point; using 1 mg/dL = 88.4 umol/L, the SI threshold is strictly above 176.8 umol/L.",
                        "excludes": ["A creatinine reported in mg/dL must not be passed as umol/L", "Exactly 176.8 umol/L does not satisfy the primary paper's strictly greater-than threshold"],
                        "source": { "citation": "Lee TH et al. Circulation. 1999;100(10):1043-1049.", "url": "https://doi.org/10.1161/01.CIR.100.10.1043" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: RcriInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_predictors() -> RcriInput {
        RcriInput {
            adult_noncardiac_surgery_candidate: true,
            original_validation_population_matches: true,
            high_risk_surgery: false,
            ischemic_heart_disease: false,
            congestive_heart_failure: false,
            cerebrovascular_disease: false,
            preoperative_insulin_treatment: false,
            creatinine_umol_l: 80.0,
        }
    }

    #[test]
    fn lee_1999_no_predictors_is_class_i_with_validation_rate() {
        let outcome = compute(&no_predictors()).unwrap();
        assert_eq!(outcome.score, 0);
        assert_eq!(outcome.original_risk_class, OriginalRiskClass::I);
        assert_eq!(outcome.original_validation_rate_percent, Some(0.4));
        assert!(!outcome.elevated_risk_by_traditional_threshold);
    }

    #[test]
    fn each_published_predictor_scores_one_point() {
        for set_predictor in [
            |input: &mut RcriInput| input.high_risk_surgery = true,
            |input: &mut RcriInput| input.ischemic_heart_disease = true,
            |input: &mut RcriInput| input.congestive_heart_failure = true,
            |input: &mut RcriInput| input.cerebrovascular_disease = true,
            |input: &mut RcriInput| input.preoperative_insulin_treatment = true,
            |input: &mut RcriInput| input.creatinine_umol_l = 176.9,
        ] {
            let mut input = no_predictors();
            set_predictor(&mut input);
            assert_eq!(compute(&input).unwrap().score, 1);
        }
    }

    #[test]
    fn creatinine_boundary_matches_strictly_greater_than_two_mg_dl() {
        let mut input = no_predictors();
        input.creatinine_umol_l = 176.8;
        assert_eq!(compute(&input).unwrap().creatinine_points, 0);

        input.creatinine_umol_l = 176.800_001;
        assert_eq!(compute(&input).unwrap().creatinine_points, 1);
    }

    #[test]
    fn lee_1999_classes_and_validation_rates_match_publication() {
        assert_eq!(OriginalRiskClass::from_score(0), OriginalRiskClass::I);
        assert_eq!(OriginalRiskClass::from_score(1), OriginalRiskClass::II);
        assert_eq!(OriginalRiskClass::from_score(2), OriginalRiskClass::III);
        assert_eq!(OriginalRiskClass::from_score(3), OriginalRiskClass::IV);
        assert_eq!(OriginalRiskClass::from_score(6), OriginalRiskClass::IV);
        assert_eq!(OriginalRiskClass::I.validation_rate_percent(), 0.4);
        assert_eq!(OriginalRiskClass::II.validation_rate_percent(), 0.9);
        assert_eq!(OriginalRiskClass::III.validation_rate_percent(), 6.6);
        assert_eq!(OriginalRiskClass::IV.validation_rate_percent(), 11.0);
    }

    #[test]
    fn score_two_crosses_traditional_elevated_risk_threshold() {
        let mut input = no_predictors();
        input.ischemic_heart_disease = true;
        assert!(
            !compute(&input)
                .unwrap()
                .elevated_risk_by_traditional_threshold
        );

        input.cerebrovascular_disease = true;
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.score, 2);
        assert!(outcome.elevated_risk_by_traditional_threshold);
        assert!(outcome.interpretation.contains("meets"));
    }

    #[test]
    fn all_predictors_score_six_and_class_iv() {
        let input = RcriInput {
            adult_noncardiac_surgery_candidate: true,
            original_validation_population_matches: true,
            high_risk_surgery: true,
            ischemic_heart_disease: true,
            congestive_heart_failure: true,
            cerebrovascular_disease: true,
            preoperative_insulin_treatment: true,
            creatinine_umol_l: 200.0,
        };
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.score, 6);
        assert_eq!(outcome.original_risk_class, OriginalRiskClass::IV);
    }

    #[test]
    fn rejects_use_outside_adult_noncardiac_surgery() {
        let mut input = no_predictors();
        input.adult_noncardiac_surgery_candidate = false;
        assert!(
            compute(&input)
                .unwrap_err()
                .to_string()
                .contains("noncardiac surgery")
        );
    }

    #[test]
    fn rejects_nonpositive_or_nonfinite_creatinine() {
        for creatinine in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let mut input = no_predictors();
            input.creatinine_umol_l = creatinine;
            assert!(compute(&input).is_err());
        }
    }

    #[test]
    fn response_preserves_inputs_points_provenance_and_threshold() {
        let mut input = no_predictors();
        input.high_risk_surgery = true;
        input.creatinine_umol_l = 200.0;
        let response = build_response(&input).unwrap();

        assert_eq!(response.calculator, NAME);
        assert_eq!(response.result, json!(2));
        assert_eq!(response.working["high_risk_surgery"], json!(true));
        assert_eq!(response.working["high_risk_surgery_points"], json!(1));
        assert_eq!(response.working["creatinine_umol_l"], json!(200.0));
        assert_eq!(response.working["creatinine_points"], json!(1));
        assert_eq!(response.working["original_risk_class"], json!("III"));
        assert_eq!(
            response.working["original_validation_cohort_major_cardiac_complication_rate_percent"],
            json!(6.6)
        );
        assert_eq!(
            response.working["elevated_risk_by_traditional_rcri_threshold"],
            json!(true)
        );
        assert!(response.reference.contains("Lee TH"));
        assert!(response.reference.contains("2024 AHA/ACC"));
    }

    #[test]
    fn interpretation_labels_historical_rate_and_limits_use() {
        let interpretation = compute(&no_predictors()).unwrap().interpretation;
        assert!(interpretation.contains("historical cohort rate"));
        assert!(interpretation.contains("not an individualized current probability"));
        assert!(interpretation.contains("complete heart block"));
        assert!(interpretation.contains("contemporary MACE estimate"));
        assert!(interpretation.contains("score alone must not"));
    }

    #[test]
    fn omits_historical_rate_outside_original_validation_population() {
        let mut input = no_predictors();
        input.original_validation_population_matches = false;
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.original_validation_rate_percent, None);
        assert!(outcome.interpretation.contains("rate is not reported"));

        let response = build_response(&input).unwrap();
        assert!(
            !response
                .working
                .contains_key("original_validation_cohort_major_cardiac_complication_rate_percent")
        );
    }

    #[test]
    fn dynamic_calculation_matches_typed_contract() {
        let input = no_predictors();
        let dynamic = Rcri
            .calculate(&serde_json::to_value(input).unwrap())
            .unwrap();
        assert_eq!(dynamic, build_response(&input).unwrap());
    }

    #[test]
    fn dynamic_calculation_rejects_unknown_fields() {
        let mut value = serde_json::to_value(no_predictors()).unwrap();
        value["unexpected"] = json!(true);
        assert!(Rcri.calculate(&value).is_err());
    }

    #[test]
    fn schema_requires_all_inputs_and_defines_common_traps() {
        let schema = Rcri.input_schema();
        assert_eq!(schema["required"].as_array().unwrap().len(), 8);
        assert!(schema["additionalProperties"] == false);
        assert!(
            schema["properties"]["high_risk_surgery"]["definition"]["excludes"][1]
                .as_str()
                .unwrap()
                .contains("Infrainguinal")
        );
        assert!(
            schema["properties"]["ischemic_heart_disease"]["definition"]["excludes"][0]
                .as_str()
                .unwrap()
                .contains("revascularisation alone")
        );
        assert!(
            schema["properties"]["creatinine_umol_l"]["definition"]["statement"]
                .as_str()
                .unwrap()
                .contains("strictly above 176.8")
        );
    }
}
