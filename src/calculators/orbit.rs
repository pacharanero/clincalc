// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! ORBIT major-bleeding risk score for anticoagulated adults with atrial fibrillation.
//!
//! This is an independent implementation of the published five-factor method.
//! It does not reproduce the article's prose, tables, figures, or presentation.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "orbit";
pub const REFERENCE: &str = "O'Brien EC, Simon DN, Thomas LE, et al. The ORBIT bleeding score: a simple bedside score to assess bleeding risk in atrial fibrillation. Eur Heart J. 2015;36(46):3258-3264. doi:10.1093/eurheartj/ehv476. PMID:26424865.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "No third-party licence required - factual method and observed rates independently encoded under the expression-method distinction in WIPO Copyright Treaty Article 2; article expression is not redistributed",
    source_url: "https://www.wipo.int/wipolex/en/text/295166",
};

const LIMITATIONS: &str = "Use only for major-bleeding risk stratification in an adult with electrocardiographically confirmed atrial fibrillation who is receiving oral anticoagulation. The score does not determine whether anticoagulation should be started, stopped, or withheld. Address modifiable bleeding risks and consider stroke risk, treatment indication, patient preferences, and clinical judgement. Published rates are observed derivation-cohort incidence rates, not personalised annual probabilities. Discrimination was modest, the US outpatient derivation cohort was predominantly warfarin-treated, and external validation used a selected clinical-trial population.";
const MAX_AGE_YEARS: u16 = 120;
const MAX_HAEMOGLOBIN_G_L: f64 = 250.0;
const MAX_EGFR_ML_MIN_1_73_M2: f64 = 200.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentContext {
    AdultWithElectrocardiographicallyConfirmedAtrialFibrillationReceivingOralAnticoagulation,
}

/// Historical sex branch used by the source haemoglobin and haematocrit thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

impl Sex {
    fn slug(self) -> &'static str {
        match self {
            Self::Male => "male",
            Self::Female => "female",
        }
    }

    fn haemoglobin_threshold_g_l(self) -> f64 {
        match self {
            Self::Male => 130.0,
            Self::Female => 120.0,
        }
    }

    fn haematocrit_threshold_percent(self) -> f64 {
        match self {
            Self::Male => 40.0,
            Self::Female => 36.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrbitInput {
    pub assessment_context: AssessmentContext,
    pub age_years: u16,
    /// Historical sex branch used only for source-defined laboratory thresholds.
    pub sex: Sex,
    /// Current haemoglobin in g/L. May be omitted only when the anaemia component is otherwise established.
    pub haemoglobin_g_l: Option<f64>,
    /// Current haematocrit as a percentage. May be omitted only when the anaemia component is otherwise established.
    pub haematocrit_percent: Option<f64>,
    pub history_of_anaemia: bool,
    pub bleeding_history: bool,
    pub egfr_ml_min_1_73_m2: f64,
    pub antiplatelet_treatment: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskBand {
    Low,
    Medium,
    High,
}

impl RiskBand {
    fn from_score(score: u8) -> Self {
        match score {
            0..=2 => Self::Low,
            3 => Self::Medium,
            _ => Self::High,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    fn observed_rate_per_100_patient_years(self) -> f64 {
        match self {
            Self::Low => 2.4,
            Self::Medium => 4.7,
            Self::High => 8.1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObservedRate {
    pub per_100_patient_years: f64,
    pub confidence_interval_low: f64,
    pub confidence_interval_high: f64,
}

const SCORE_RATES: [ObservedRate; 8] = [
    ObservedRate {
        per_100_patient_years: 1.7,
        confidence_interval_low: 1.2,
        confidence_interval_high: 2.4,
    },
    ObservedRate {
        per_100_patient_years: 2.3,
        confidence_interval_low: 1.9,
        confidence_interval_high: 2.9,
    },
    ObservedRate {
        per_100_patient_years: 2.9,
        confidence_interval_low: 2.3,
        confidence_interval_high: 3.5,
    },
    ObservedRate {
        per_100_patient_years: 4.7,
        confidence_interval_low: 4.0,
        confidence_interval_high: 5.6,
    },
    ObservedRate {
        per_100_patient_years: 6.8,
        confidence_interval_low: 5.8,
        confidence_interval_high: 8.1,
    },
    ObservedRate {
        per_100_patient_years: 9.0,
        confidence_interval_low: 7.2,
        confidence_interval_high: 11.2,
    },
    ObservedRate {
        per_100_patient_years: 12.3,
        confidence_interval_low: 9.0,
        confidence_interval_high: 16.7,
    },
    ObservedRate {
        per_100_patient_years: 14.9,
        confidence_interval_low: 8.9,
        confidence_interval_high: 25.3,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrbitPoints {
    pub older_age: u8,
    pub reduced_haemoglobin_haematocrit_or_anaemia: u8,
    pub bleeding_history: u8,
    pub insufficient_kidney_function: u8,
    pub antiplatelet_treatment: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrbitOutcome {
    pub score: u8,
    pub points: OrbitPoints,
    pub anaemia_component_present: bool,
    pub risk_band: RiskBand,
    pub score_observed_rate: ObservedRate,
    pub category_observed_rate_per_100_patient_years: f64,
    pub interpretation: String,
}

pub fn compute(input: &OrbitInput) -> Result<OrbitOutcome, CalcError> {
    if !(18..=MAX_AGE_YEARS).contains(&input.age_years) {
        return Err(CalcError::InvalidInput(format!(
            "age_years must be between 18 and {MAX_AGE_YEARS} for the adult assessment context"
        )));
    }
    if !input.egfr_ml_min_1_73_m2.is_finite()
        || !(0.0..=MAX_EGFR_ML_MIN_1_73_M2).contains(&input.egfr_ml_min_1_73_m2)
    {
        return Err(CalcError::InvalidInput(format!(
            "egfr_ml_min_1_73_m2 must be finite and between 0 and {MAX_EGFR_ML_MIN_1_73_M2}"
        )));
    }
    for (name, value, maximum) in [
        (
            "haemoglobin_g_l",
            input.haemoglobin_g_l,
            MAX_HAEMOGLOBIN_G_L,
        ),
        ("haematocrit_percent", input.haematocrit_percent, 100.0),
    ] {
        if let Some(value) = value
            && (!value.is_finite() || value <= 0.0 || value > maximum)
        {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be finite, greater than 0, and no greater than {maximum}"
            )));
        }
    }
    let reduced_haemoglobin = input
        .haemoglobin_g_l
        .is_some_and(|value| value < input.sex.haemoglobin_threshold_g_l());
    let reduced_haematocrit = input
        .haematocrit_percent
        .is_some_and(|value| value < input.sex.haematocrit_threshold_percent());
    let anaemia_component_present =
        input.history_of_anaemia || reduced_haemoglobin || reduced_haematocrit;
    if !anaemia_component_present
        && (input.haemoglobin_g_l.is_none() || input.haematocrit_percent.is_none())
    {
        return Err(CalcError::InvalidInput(
            "both haemoglobin_g_l and haematocrit_percent are required unless anaemia history or a supplied low measurement already establishes the anaemia component; missing data must not be scored as normal"
                .into(),
        ));
    }

    let points = OrbitPoints {
        older_age: u8::from(input.age_years >= 75),
        reduced_haemoglobin_haematocrit_or_anaemia: if anaemia_component_present { 2 } else { 0 },
        bleeding_history: if input.bleeding_history { 2 } else { 0 },
        insufficient_kidney_function: u8::from(input.egfr_ml_min_1_73_m2 < 60.0),
        antiplatelet_treatment: u8::from(input.antiplatelet_treatment),
    };
    let score = points.older_age
        + points.reduced_haemoglobin_haematocrit_or_anaemia
        + points.bleeding_history
        + points.insufficient_kidney_function
        + points.antiplatelet_treatment;
    let risk_band = RiskBand::from_score(score);
    let score_observed_rate = SCORE_RATES[usize::from(score)];
    let category_observed_rate_per_100_patient_years =
        risk_band.observed_rate_per_100_patient_years();
    let interpretation = format!(
        "ORBIT score {score}/7: {} major-bleeding risk category. In ORBIT-AF, score {score} had {:.1} major bleeds per 100 patient-years (95% CI {:.1}-{:.1}); the {} category had {:.1} per 100 patient-years. These are observed cohort incidence rates, not a personalised annual probability. {LIMITATIONS}",
        risk_band.slug(),
        score_observed_rate.per_100_patient_years,
        score_observed_rate.confidence_interval_low,
        score_observed_rate.confidence_interval_high,
        risk_band.slug(),
        category_observed_rate_per_100_patient_years,
    );

    Ok(OrbitOutcome {
        score,
        points,
        anaemia_component_present,
        risk_band,
        score_observed_rate,
        category_observed_rate_per_100_patient_years,
        interpretation,
    })
}

pub fn build_response(input: &OrbitInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert("assessment_context".into(), json!(input.assessment_context));
    working.insert("age_years".into(), json!(input.age_years));
    working.insert("sex_threshold_branch".into(), json!(input.sex.slug()));
    working.insert("haemoglobin_g_l".into(), json!(input.haemoglobin_g_l));
    working.insert(
        "haemoglobin_threshold_g_l".into(),
        json!(input.sex.haemoglobin_threshold_g_l()),
    );
    working.insert(
        "haematocrit_percent".into(),
        json!(input.haematocrit_percent),
    );
    working.insert(
        "haematocrit_threshold_percent".into(),
        json!(input.sex.haematocrit_threshold_percent()),
    );
    working.insert("history_of_anaemia".into(), json!(input.history_of_anaemia));
    working.insert(
        "anaemia_component_present".into(),
        json!(outcome.anaemia_component_present),
    );
    working.insert("older_age_points".into(), json!(outcome.points.older_age));
    working.insert(
        "anaemia_component_points".into(),
        json!(outcome.points.reduced_haemoglobin_haematocrit_or_anaemia),
    );
    working.insert("bleeding_history".into(), json!(input.bleeding_history));
    working.insert(
        "bleeding_history_points".into(),
        json!(outcome.points.bleeding_history),
    );
    working.insert(
        "egfr_ml_min_1_73_m2".into(),
        json!(input.egfr_ml_min_1_73_m2),
    );
    working.insert(
        "insufficient_kidney_function_points".into(),
        json!(outcome.points.insufficient_kidney_function),
    );
    working.insert(
        "antiplatelet_treatment".into(),
        json!(input.antiplatelet_treatment),
    );
    working.insert(
        "antiplatelet_treatment_points".into(),
        json!(outcome.points.antiplatelet_treatment),
    );
    working.insert("total_score".into(), json!(outcome.score));
    working.insert("risk_band".into(), json!(outcome.risk_band.slug()));
    working.insert(
        "score_observed_bleeds_per_100_patient_years".into(),
        json!(outcome.score_observed_rate.per_100_patient_years),
    );
    working.insert(
        "score_observed_rate_95_ci".into(),
        json!([
            outcome.score_observed_rate.confidence_interval_low,
            outcome.score_observed_rate.confidence_interval_high
        ]),
    );
    working.insert(
        "category_observed_bleeds_per_100_patient_years".into(),
        json!(outcome.category_observed_rate_per_100_patient_years),
    );
    working.insert(
        "major_bleeding_endpoint".into(),
        json!("ISTH major bleeding: fatal bleeding; symptomatic critical-area or critical-organ bleeding; haemoglobin fall >=20 g/L; or transfusion of >=2 units of whole blood or red cells"),
    );
    working.insert("limitations".into(), json!(LIMITATIONS));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.score),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn input_schema() -> Value {
    let source = json!({
        "citation": "O'Brien EC, Simon DN, Thomas LE, et al. Eur Heart J. 2015;36(46):3258-3264. doi:10.1093/eurheartj/ehv476. PMID:26424865.",
        "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC4670965/"
    });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "OrbitInput",
        "description": "ORBIT major-bleeding risk score for an adult with electrocardiographically confirmed atrial fibrillation who is receiving oral anticoagulation. Raw laboratory values are required to resolve the source thresholds; missing data must not be treated as normal. The result is risk stratification, not a directive to start, stop, or withhold anticoagulation. Published rates are observed events per 100 patient-years, not personalised annual probabilities.",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "assessment_context", "age_years", "sex", "history_of_anaemia",
            "bleeding_history", "egfr_ml_min_1_73_m2", "antiplatelet_treatment"
        ],
        "allOf": [{
            "if": {
                "properties": { "history_of_anaemia": { "const": false } },
                "required": ["history_of_anaemia"]
            },
            "then": {
                "anyOf": [
                    {
                        "required": ["haemoglobin_g_l", "haematocrit_percent"],
                        "properties": {
                            "haemoglobin_g_l": { "type": "number" },
                            "haematocrit_percent": { "type": "number" }
                        }
                    },
                    {
                        "required": ["sex", "haemoglobin_g_l"],
                        "properties": {
                            "sex": { "const": "male" },
                            "haemoglobin_g_l": { "type": "number", "exclusiveMaximum": 130 }
                        }
                    },
                    {
                        "required": ["sex", "haemoglobin_g_l"],
                        "properties": {
                            "sex": { "const": "female" },
                            "haemoglobin_g_l": { "type": "number", "exclusiveMaximum": 120 }
                        }
                    },
                    {
                        "required": ["sex", "haematocrit_percent"],
                        "properties": {
                            "sex": { "const": "male" },
                            "haematocrit_percent": { "type": "number", "exclusiveMaximum": 40 }
                        }
                    },
                    {
                        "required": ["sex", "haematocrit_percent"],
                        "properties": {
                            "sex": { "const": "female" },
                            "haematocrit_percent": { "type": "number", "exclusiveMaximum": 36 }
                        }
                    }
                ]
            }
        }],
        "properties": {
            "assessment_context": {
                "type": "string",
                "const": "adult_with_electrocardiographically_confirmed_atrial_fibrillation_receiving_oral_anticoagulation",
                "description": "Attestation of the source population: adult, electrocardiographically confirmed atrial fibrillation, and current oral anticoagulation",
                "definition": {
                    "concept": "ORBIT assessment population",
                    "statement": "Use only for major-bleeding risk stratification in an adult with electrocardiographically confirmed atrial fibrillation who is currently receiving oral anticoagulation.",
                    "includes": ["Adult aged 18 years or older", "Electrocardiographically confirmed atrial fibrillation", "Current oral anticoagulant treatment"],
                    "excludes": ["Paediatric use", "No confirmed atrial fibrillation", "Not receiving oral anticoagulation", "Use as an autonomous anticoagulation treatment decision"],
                    "source": source, "snomedEcl": null, "refset": null,
                    "caveats": "The derivation cohort was a US outpatient registry and was predominantly warfarin-treated. External validation used a selected trial population.",
                    "status": "draft"
                }
            },
            "age_years": {
                "type": "integer", "minimum": 18, "maximum": MAX_AGE_YEARS, "unit": "years",
                "description": "Age in completed years; age 75 or older scores 1 point. The broad upper bound is a clincalc input-safety guard, not an ORBIT scoring threshold"
            },
            "sex": {
                "type": "string", "enum": ["male", "female"],
                "description": "Historical source sex branch used only for haemoglobin and haematocrit thresholds",
                "definition": {
                    "concept": "ORBIT historical sex-specific laboratory threshold branch",
                    "statement": "Select the male or female branch used by the source method for haemoglobin and haematocrit thresholds.",
                    "includes": ["male: haemoglobin <130 g/L or haematocrit <40%", "female: haemoglobin <120 g/L or haematocrit <36%"],
                    "excludes": ["Automatic inference from name, appearance, gender identity, or unstated data"],
                    "source": source, "snomedEcl": null, "refset": null,
                    "caveats": "These labels reproduce historical coefficient branches, not a modern definition of sex or gender. Verify the branch clinically.",
                    "status": "draft"
                }
            },
            "haemoglobin_g_l": {
                "type": ["number", "null"], "exclusiveMinimum": 0, "maximum": MAX_HAEMOGLOBIN_G_L, "unit": "g/L",
                "description": "Current haemoglobin in g/L; may be omitted only when anaemia history or a supplied low haematocrit already establishes the anaemia component. The broad upper bound is a clincalc input-safety guard, not an ORBIT scoring threshold",
                "definition": {
                    "concept": "Reduced haemoglobin for ORBIT",
                    "statement": "Reduced haemoglobin is below 130 g/L in the male branch or below 120 g/L in the female branch; each boundary itself is normal for scoring.",
                    "includes": ["male branch: <130 g/L", "female branch: <120 g/L"],
                    "excludes": ["g/dL entered without multiplying by 10", "The exact boundary values 130 g/L and 120 g/L do not score"],
                    "source": source, "snomedEcl": null, "refset": null,
                    "caveats": "The source article displays mg/dL in places; the clinically coherent source thresholds are 13 g/dL and 12 g/dL, represented here as 130 g/L and 120 g/L.",
                    "status": "draft"
                }
            },
            "haematocrit_percent": {
                "type": ["number", "null"], "exclusiveMinimum": 0, "maximum": 100, "unit": "%",
                "description": "Current haematocrit percentage; may be omitted only when anaemia history or a supplied low haemoglobin already establishes the anaemia component",
                "definition": {
                    "concept": "Reduced haematocrit for ORBIT",
                    "statement": "Reduced haematocrit is below 40% in the male branch or below 36% in the female branch; each boundary itself is normal for scoring.",
                    "includes": ["male branch: <40%", "female branch: <36%"],
                    "excludes": ["Fractional values such as 0.40 entered without conversion to 40%", "The exact boundary values 40% and 36% do not score"],
                    "source": source, "snomedEcl": null, "refset": null,
                    "caveats": "If no anaemia history is present, both laboratory values are required unless one supplied value is already below its sex-branch threshold and therefore conclusively establishes the combined component. An unmeasured unresolved component must not be treated as normal.",
                    "status": "draft"
                }
            },
            "history_of_anaemia": {
                "type": "boolean",
                "description": "Documented history of anaemia; combines with reduced haemoglobin or haematocrit into one 2-point component",
                "definition": {
                    "concept": "History of anaemia for ORBIT",
                    "statement": "A documented history of anaemia satisfies the combined two-point anaemia component independently of current laboratory values.",
                    "includes": ["Documented anaemia history"],
                    "excludes": ["Unknown history treated as absent", "Adding separate points for low haemoglobin, low haematocrit, and anaemia history"],
                    "source": source, "snomedEcl": null, "refset": null,
                    "caveats": "Any one or more of reduced haemoglobin, reduced haematocrit, or anaemia history contributes exactly 2 points total.",
                    "status": "draft"
                }
            },
            "bleeding_history": {
                "type": "boolean",
                "description": "Prior gastrointestinal bleeding, intracranial bleeding, or haemorrhagic stroke documented at baseline; 2 points",
                "definition": {
                    "concept": "ORBIT bleeding history",
                    "statement": "Documented history of gastrointestinal bleeding, intracranial bleeding, or haemorrhagic stroke.",
                    "includes": ["Prior gastrointestinal bleeding", "Prior intracranial bleeding", "Prior haemorrhagic stroke"],
                    "excludes": ["Unknown history treated as absent", "Minor bruising without a qualifying bleeding history"],
                    "source": source, "snomedEcl": null, "refset": null,
                    "caveats": "This source definition is narrower than an unspecified history of any bleeding.",
                    "status": "draft"
                }
            },
            "egfr_ml_min_1_73_m2": {
                "type": "number", "minimum": 0, "maximum": MAX_EGFR_ML_MIN_1_73_M2, "unit": "mL/min/1.73 m2",
                "description": "Current estimated glomerular filtration rate; values below 60 score 1 point. The broad upper bound is a clincalc input-safety guard, not an ORBIT scoring threshold",
                "definition": {
                    "concept": "Insufficient kidney function for ORBIT",
                    "statement": "Estimated glomerular filtration rate below 60 mL/min/1.73 m2 scores one point; exactly 60 does not score.",
                    "includes": ["eGFR <60 mL/min/1.73 m2"],
                    "excludes": ["Serum creatinine entered as though it were eGFR", "eGFR exactly 60 mL/min/1.73 m2"],
                    "source": source, "snomedEcl": null, "refset": null,
                    "caveats": "The source article displays an incorrect mass unit in places. This input is eGFR in mL/min/1.73 m2 and does not calculate eGFR from creatinine.",
                    "status": "draft"
                }
            },
            "antiplatelet_treatment": {
                "type": "boolean",
                "description": "Current concomitant antiplatelet treatment; 1 point",
                "definition": {
                    "concept": "Concomitant antiplatelet treatment for ORBIT",
                    "statement": "Current treatment with an antiplatelet agent in addition to oral anticoagulation.",
                    "includes": ["Aspirin", "Clopidogrel", "Prasugrel", "Ticagrelor", "Fixed-dose aspirin/dipyridamole"],
                    "excludes": ["The oral anticoagulant itself", "Historical antiplatelet use that is no longer current"],
                    "source": source, "snomedEcl": null, "refset": null,
                    "caveats": "Review whether concomitant antiplatelet treatment remains indicated; this calculator does not direct medication changes.",
                    "status": "draft"
                }
            }
        }
    })
}

pub struct Orbit;

impl Calculator for Orbit {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "ORBIT Bleeding Risk Score"
    }

    fn description(&self) -> &'static str {
        "Estimates major-bleeding risk in anticoagulated adults with atrial fibrillation using the five-factor ORBIT score."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        input_schema()
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: OrbitInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> OrbitInput {
        OrbitInput {
            assessment_context: AssessmentContext::AdultWithElectrocardiographicallyConfirmedAtrialFibrillationReceivingOralAnticoagulation,
            age_years: 50,
            sex: Sex::Male,
            haemoglobin_g_l: Some(130.0),
            haematocrit_percent: Some(40.0),
            history_of_anaemia: false,
            bleeding_history: false,
            egfr_ml_min_1_73_m2: 60.0,
            antiplatelet_treatment: false,
        }
    }

    #[test]
    fn primary_source_component_boundaries_are_exact() {
        let mut input = baseline();
        assert_eq!(compute(&input).unwrap().score, 0);
        input.age_years = 75;
        assert_eq!(compute(&input).unwrap().points.older_age, 1);
        input.age_years = 74;
        input.egfr_ml_min_1_73_m2 = 59.999;
        assert_eq!(
            compute(&input).unwrap().points.insufficient_kidney_function,
            1
        );
        input.egfr_ml_min_1_73_m2 = 60.0;
        assert_eq!(
            compute(&input).unwrap().points.insufficient_kidney_function,
            0
        );
    }

    #[test]
    fn sex_specific_laboratory_boundaries_are_strict() {
        for (sex, haemoglobin, haematocrit) in
            [(Sex::Male, 130.0, 40.0), (Sex::Female, 120.0, 36.0)]
        {
            let mut input = baseline();
            input.sex = sex;
            input.haemoglobin_g_l = Some(haemoglobin);
            input.haematocrit_percent = Some(haematocrit);
            assert!(!compute(&input).unwrap().anaemia_component_present);
            input.haemoglobin_g_l = Some(haemoglobin - 0.1);
            assert!(compute(&input).unwrap().anaemia_component_present);
            input.haemoglobin_g_l = Some(haemoglobin);
            input.haematocrit_percent = Some(haematocrit - 0.1);
            assert!(compute(&input).unwrap().anaemia_component_present);
        }
    }

    #[test]
    fn anaemia_conditions_combine_into_one_two_point_component() {
        let mut input = baseline();
        input.history_of_anaemia = true;
        input.haemoglobin_g_l = Some(100.0);
        input.haematocrit_percent = Some(30.0);
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.points.reduced_haemoglobin_haematocrit_or_anaemia, 2);
        assert_eq!(outcome.score, 2);
    }

    #[test]
    fn all_five_components_score_seven() {
        let mut input = baseline();
        input.age_years = 80;
        input.history_of_anaemia = true;
        input.bleeding_history = true;
        input.egfr_ml_min_1_73_m2 = 30.0;
        input.antiplatelet_treatment = true;
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.score, 7);
        assert_eq!(outcome.risk_band, RiskBand::High);
    }

    #[test]
    fn primary_source_score_rates_and_categories_are_exact() {
        let expected = [
            (1.7, 1.2, 2.4),
            (2.3, 1.9, 2.9),
            (2.9, 2.3, 3.5),
            (4.7, 4.0, 5.6),
            (6.8, 5.8, 8.1),
            (9.0, 7.2, 11.2),
            (12.3, 9.0, 16.7),
            (14.9, 8.9, 25.3),
        ];
        for (score, expected_rate) in expected.into_iter().enumerate() {
            assert_eq!(
                SCORE_RATES[score],
                ObservedRate {
                    per_100_patient_years: expected_rate.0,
                    confidence_interval_low: expected_rate.1,
                    confidence_interval_high: expected_rate.2,
                }
            );
        }
        assert_eq!(RiskBand::from_score(2), RiskBand::Low);
        assert_eq!(RiskBand::from_score(3), RiskBand::Medium);
        assert_eq!(RiskBand::from_score(4), RiskBand::High);
    }

    #[test]
    fn dynamic_surface_reaches_every_published_score_and_rate() {
        let vectors: [(u8, bool, bool, bool, bool, bool); 8] = [
            (0, false, false, false, false, false),
            (1, true, false, false, false, false),
            (2, false, true, false, false, false),
            (3, true, true, false, false, false),
            (4, false, true, true, false, false),
            (5, true, true, true, false, false),
            (6, false, true, true, true, true),
            (7, true, true, true, true, true),
        ];

        for (score, older_age, anaemia, bleeding, kidney, antiplatelet) in vectors {
            let mut input = baseline();
            input.age_years = if older_age { 75 } else { 74 };
            input.history_of_anaemia = anaemia;
            input.bleeding_history = bleeding;
            input.egfr_ml_min_1_73_m2 = if kidney { 59.0 } else { 60.0 };
            input.antiplatelet_treatment = antiplatelet;

            let response = Orbit
                .calculate(&serde_json::to_value(input).unwrap())
                .unwrap();
            assert_eq!(response.result, json!(score));
            assert_eq!(
                response.working["score_observed_bleeds_per_100_patient_years"],
                json!(SCORE_RATES[usize::from(score)].per_100_patient_years)
            );
        }
    }

    #[test]
    fn missing_laboratory_data_is_not_silently_normal() {
        let mut input = baseline();
        input.haemoglobin_g_l = None;
        assert!(compute(&input).is_err());
        input.haematocrit_percent = Some(35.0);
        assert_eq!(compute(&input).unwrap().score, 2);

        input = baseline();
        input.haematocrit_percent = None;
        input.haemoglobin_g_l = Some(129.0);
        assert_eq!(compute(&input).unwrap().score, 2);

        input.haemoglobin_g_l = Some(130.0);
        assert!(compute(&input).is_err());
        input.history_of_anaemia = true;
        assert_eq!(compute(&input).unwrap().score, 2);
    }

    #[test]
    fn dynamic_laboratory_resolution_matrix_matches_source_or_rule() {
        let base = serde_json::to_value(baseline()).unwrap();
        let cases = [
            (Sex::Male, Some(130.0), Some(40.0), false, Some(0)),
            (Sex::Male, Some(129.9), None, false, Some(2)),
            (Sex::Male, Some(130.0), None, false, None),
            (Sex::Male, None, Some(39.9), false, Some(2)),
            (Sex::Male, None, Some(40.0), false, None),
            (Sex::Female, Some(120.0), Some(36.0), false, Some(0)),
            (Sex::Female, Some(119.9), None, false, Some(2)),
            (Sex::Female, Some(120.0), None, false, None),
            (Sex::Female, None, Some(35.9), false, Some(2)),
            (Sex::Female, None, Some(36.0), false, None),
            (Sex::Female, None, None, false, None),
            (Sex::Female, None, None, true, Some(2)),
        ];

        for (sex, haemoglobin, haematocrit, history, expected_points) in cases {
            let mut input = base.clone();
            input["sex"] = json!(sex);
            input["haemoglobin_g_l"] = json!(haemoglobin);
            input["haematocrit_percent"] = json!(haematocrit);
            input["history_of_anaemia"] = json!(history);
            let result = Orbit.calculate(&input);
            assert_eq!(result.is_ok(), expected_points.is_some(), "input: {input}");
            if let Some(points) = expected_points {
                assert_eq!(
                    result.unwrap().working["anaemia_component_points"],
                    json!(points)
                );
            }
        }

        let mut missing_but_resolved = base.clone();
        missing_but_resolved["haemoglobin_g_l"] = json!(129.9);
        missing_but_resolved
            .as_object_mut()
            .unwrap()
            .remove("haematocrit_percent");
        assert!(Orbit.calculate(&missing_but_resolved).is_ok());

        let mut missing_and_unresolved = base;
        missing_and_unresolved
            .as_object_mut()
            .unwrap()
            .remove("haemoglobin_g_l");
        missing_and_unresolved["haematocrit_percent"] = Value::Null;
        assert!(Orbit.calculate(&missing_and_unresolved).is_err());
    }

    #[test]
    fn rejects_invalid_measurements_and_non_adult_age() {
        let mut input = baseline();
        input.age_years = 17;
        assert!(compute(&input).is_err());
        input.age_years = MAX_AGE_YEARS + 1;
        assert!(compute(&input).is_err());
        input.age_years = 18;
        for value in [f64::NAN, f64::INFINITY, -1.0, MAX_EGFR_ML_MIN_1_73_M2 + 0.1] {
            input.egfr_ml_min_1_73_m2 = value;
            assert!(compute(&input).is_err());
        }
        input.egfr_ml_min_1_73_m2 = 60.0;
        input.haemoglobin_g_l = Some(MAX_HAEMOGLOBIN_G_L + 0.1);
        assert!(compute(&input).is_err());
        input.haemoglobin_g_l = Some(130.0);
        input.haematocrit_percent = Some(100.1);
        assert!(compute(&input).is_err());
    }

    #[test]
    fn response_labels_incidence_and_preserves_safety_limits() {
        let mut input = baseline();
        input.age_years = 78;
        input.history_of_anaemia = true;
        input.egfr_ml_min_1_73_m2 = 45.0;
        let response = build_response(&input).unwrap();
        assert_eq!(response.result, json!(4));
        assert_eq!(response.working["risk_band"], json!("high"));
        assert_eq!(
            response.working["score_observed_bleeds_per_100_patient_years"],
            json!(6.8)
        );
        assert!(
            response
                .interpretation
                .contains("not a personalised annual probability")
        );
        assert!(
            response
                .interpretation
                .contains("does not determine whether")
        );
        assert!(
            response
                .interpretation
                .contains("predominantly warfarin-treated")
        );
    }

    #[test]
    fn dynamic_surface_is_closed_and_matches_typed_response() {
        let input = baseline();
        let value = serde_json::to_value(input).unwrap();
        assert_eq!(
            Orbit.calculate(&value).unwrap(),
            build_response(&input).unwrap()
        );
        let mut unknown = value;
        unknown["unexpected"] = json!(true);
        assert!(Orbit.calculate(&unknown).is_err());
    }

    #[test]
    fn schema_is_closed_conditional_and_defines_clinical_inputs() {
        let schema = input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(
            schema["allOf"][0]["then"]["anyOf"][0]["required"],
            json!(["haemoglobin_g_l", "haematocrit_percent"])
        );
        assert_eq!(
            schema["allOf"][0]["then"]["anyOf"][0]["properties"]["haemoglobin_g_l"]["type"],
            json!("number")
        );
        for (branch, sex, field, threshold) in [
            (1, "male", "haemoglobin_g_l", 130),
            (2, "female", "haemoglobin_g_l", 120),
            (3, "male", "haematocrit_percent", 40),
            (4, "female", "haematocrit_percent", 36),
        ] {
            let alternative = &schema["allOf"][0]["then"]["anyOf"][branch];
            assert_eq!(alternative["properties"]["sex"]["const"], json!(sex));
            assert_eq!(
                alternative["properties"][field]["exclusiveMaximum"],
                json!(threshold)
            );
        }
        assert_eq!(
            schema["properties"]["age_years"]["maximum"],
            json!(MAX_AGE_YEARS)
        );
        assert_eq!(
            schema["properties"]["haemoglobin_g_l"]["maximum"],
            json!(MAX_HAEMOGLOBIN_G_L)
        );
        assert_eq!(
            schema["properties"]["egfr_ml_min_1_73_m2"]["maximum"],
            json!(MAX_EGFR_ML_MIN_1_73_M2)
        );
        assert_eq!(
            schema["properties"]["egfr_ml_min_1_73_m2"]["unit"],
            json!("mL/min/1.73 m2")
        );
        for name in [
            "assessment_context",
            "sex",
            "haemoglobin_g_l",
            "haematocrit_percent",
            "history_of_anaemia",
            "bleeding_history",
            "egfr_ml_min_1_73_m2",
            "antiplatelet_treatment",
        ] {
            assert!(
                schema["properties"][name]["definition"].is_object(),
                "{name}"
            );
        }
    }

    #[test]
    fn licence_records_independent_method_implementation() {
        assert!(LICENSE.license.contains("No third-party licence required"));
        assert!(LICENSE.license.contains("independently encoded"));
        assert!(LICENSE.license.contains("WIPO Copyright Treaty Article 2"));
        assert!(
            LICENSE
                .license
                .contains("article expression is not redistributed")
        );
        assert_eq!(
            LICENSE.source_url,
            "https://www.wipo.int/wipolex/en/text/295166"
        );
    }
}
