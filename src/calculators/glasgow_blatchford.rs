// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Glasgow-Blatchford Bleeding Score (GBS).
//!
//! Pre-endoscopy triage for patients aged 16 years or older presenting with
//! acute upper gastrointestinal bleeding. The score uses observations available
//! at first assessment to identify patients at very low risk of needing
//! hospital-based intervention or dying. It does not diagnose bleeding, replace
//! resuscitation, or predict a specific intervention for an individual patient.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "glasgow_blatchford";

/// Primary publication and current guideline context.
pub const REFERENCE: &str = "Blatchford O, Murray WR, Blatchford M. A risk score to predict need for treatment for upper-gastrointestinal haemorrhage. Lancet. 2000;356(9238):1318-1321. doi:10.1016/S0140-6736(00)02816-6. NICE CG141: Acute upper gastrointestinal bleeding in over 16s: management (updated 2016). Laine L, Barkun AN, Saltzman JR, et al. ACG Clinical Guideline: Upper Gastrointestinal and Ulcer Bleeding. Am J Gastroenterol. 2021;116(5):899-917. doi:10.14309/ajg.0000000000001245.";

/// Distribution licence: independently implemented from the published method.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published clinical scoring method - independently implemented from the primary literature",
    source_url: "https://doi.org/10.1016/S0140-6736(00)02816-6",
};

/// Sex category used by the published sex-specific haemoglobin bands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

impl Sex {
    fn label(self) -> &'static str {
        match self {
            Self::Male => "male",
            Self::Female => "female",
        }
    }
}

/// Inputs measured at first assessment for acute upper gastrointestinal bleeding.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlasgowBlatchfordInput {
    /// Confirm the person is aged 16 years or older, as required by NICE CG141.
    pub age_16_or_over: bool,
    /// Confirm acute upper-GI bleeding at first assessment, before endoscopy.
    pub acute_upper_gi_bleeding_at_first_assessment: bool,
    /// Whether the ACG 2021 emergency-department overt-UGIB context is met.
    pub emergency_department_overt_ugib: bool,
    /// Sex category used to select the published haemoglobin scoring bands.
    pub sex: Sex,
    /// Blood urea in mmol/L, not BUN in mg/dL.
    pub urea_mmol_l: f64,
    /// Haemoglobin in g/L.
    pub haemoglobin_g_l: f64,
    /// Systolic blood pressure in mmHg.
    pub systolic_bp_mm_hg: f64,
    /// Heart rate in beats per minute.
    pub heart_rate_bpm: f64,
    /// Melaena associated with the current presentation.
    pub melaena: bool,
    /// Syncope associated with the current presentation.
    pub syncope: bool,
    /// Hepatic disease as defined by the score.
    pub hepatic_disease: bool,
    /// Cardiac failure as defined by the score.
    pub cardiac_failure: bool,
}

/// Guideline-relevant low-risk grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriageBand {
    /// Numeric score 0; guideline applicability also depends on presentation context.
    Zero,
    /// Numeric score 1; guideline applicability also depends on presentation context.
    One,
    /// Score 2-23: outside the commonly recommended very-low-risk threshold.
    AboveOne,
}

impl TriageBand {
    fn from_score(score: u8) -> Self {
        match score {
            0 => Self::Zero,
            1 => Self::One,
            _ => Self::AboveOne,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::Zero => "score-zero",
            Self::One => "score-one",
            Self::AboveOne => "above-very-low-risk-threshold",
        }
    }
}

/// Computed GBS outcome with each component retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlasgowBlatchfordOutcome {
    pub urea_points: u8,
    pub haemoglobin_points: u8,
    pub systolic_bp_points: u8,
    pub heart_rate_points: u8,
    pub melaena_points: u8,
    pub syncope_points: u8,
    pub hepatic_disease_points: u8,
    pub cardiac_failure_points: u8,
    /// Total score, 0-23.
    pub score: u8,
    pub triage_band: TriageBand,
    pub interpretation: String,
}

fn urea_points(urea_mmol_l: f64) -> u8 {
    if urea_mmol_l >= 25.0 {
        6
    } else if urea_mmol_l >= 10.0 {
        4
    } else if urea_mmol_l >= 8.0 {
        3
    } else if urea_mmol_l >= 6.5 {
        2
    } else {
        0
    }
}

fn haemoglobin_points(sex: Sex, haemoglobin_g_l: f64) -> u8 {
    match sex {
        Sex::Male if haemoglobin_g_l < 100.0 => 6,
        Sex::Male if haemoglobin_g_l < 120.0 => 3,
        Sex::Male if haemoglobin_g_l < 130.0 => 1,
        Sex::Female if haemoglobin_g_l < 100.0 => 6,
        Sex::Female if haemoglobin_g_l < 120.0 => 1,
        _ => 0,
    }
}

fn systolic_bp_points(systolic_bp_mm_hg: f64) -> u8 {
    if systolic_bp_mm_hg < 90.0 {
        3
    } else if systolic_bp_mm_hg < 100.0 {
        2
    } else if systolic_bp_mm_hg < 110.0 {
        1
    } else {
        0
    }
}

fn render_interpretation(
    score: u8,
    triage_band: TriageBand,
    emergency_department_overt_ugib: bool,
) -> String {
    match (triage_band, emergency_department_overt_ugib) {
        (TriageBand::Zero, true) => format!(
            "Glasgow-Blatchford score {score} of 23: very low risk by both cited guideline examples. NICE CG141 says to consider early discharge at a pre-endoscopy score of 0. For emergency-department patients with overt upper-GI bleeding, ACG 2021 conditionally suggests discharge with outpatient follow-up for very-low-risk patients, giving GBS 0-1 as an example; its supporting evidence is very low quality. This is not an automatic discharge instruction: use the score only after clinical assessment and resuscitation, and consider ongoing bleeding, comorbidity, anticoagulation, social circumstances, local pathways, and access to follow-up."
        ),
        (TriageBand::Zero, false) => format!(
            "Glasgow-Blatchford score {score} of 23. NICE CG141 says to consider early discharge at a pre-endoscopy score of 0 after assessment. The caller has not confirmed the emergency-department overt-UGIB context for ACG 2021's score-0-to-1 outpatient recommendation, so that recommendation is not applied. The score alone must not determine disposition."
        ),
        (TriageBand::One, true) => format!(
            "Glasgow-Blatchford score {score} of 23: within the ACG 2021 example very-low-risk range of 0-1 for emergency-department patients with overt upper-GI bleeding, but not within NICE CG141's score-0 early-discharge threshold. ACG's outpatient recommendation is conditional and supported by very-low-quality evidence. Consider outpatient management only after clinical assessment and resuscitation and according to local pathways, with reliable follow-up; the score alone must not determine disposition."
        ),
        (TriageBand::One, false) => format!(
            "Glasgow-Blatchford score {score} of 23: above NICE CG141's score-0 early-discharge threshold. The caller has not confirmed the emergency-department overt-UGIB context for ACG 2021's score-0-to-1 outpatient recommendation, so that recommendation is not applied. The score alone must not determine disposition."
        ),
        (TriageBand::AboveOne, _) => format!(
            "Glasgow-Blatchford score {score} of 23: outside the commonly recommended very-low-risk range of 0-1. This does not specify which intervention is required and is not a stand-alone high-risk rule. Continue acute upper-GI-bleeding assessment, resuscitation, specialist management, and endoscopy planning according to clinical status and local guidance."
        ),
    }
}

/// Pure scoring.
pub fn compute(input: &GlasgowBlatchfordInput) -> Result<GlasgowBlatchfordOutcome, CalcError> {
    if !input.age_16_or_over {
        return Err(CalcError::InvalidInput(
            "this implementation follows NICE CG141 for acute upper gastrointestinal bleeding in people aged 16 years and older".into(),
        ));
    }
    if !input.acute_upper_gi_bleeding_at_first_assessment {
        return Err(CalcError::InvalidInput(
            "the Glasgow-Blatchford score is for first pre-endoscopy assessment of acute upper gastrointestinal bleeding".into(),
        ));
    }

    let observations = [
        input.urea_mmol_l,
        input.haemoglobin_g_l,
        input.systolic_bp_mm_hg,
        input.heart_rate_bpm,
    ];
    if observations.iter().any(|value| !value.is_finite()) {
        return Err(CalcError::InvalidInput(
            "urea, haemoglobin, systolic blood pressure, and heart rate must be finite numbers"
                .into(),
        ));
    }
    if input.urea_mmol_l < 0.0 {
        return Err(CalcError::InvalidInput(
            "urea_mmol_l cannot be negative".into(),
        ));
    }
    if input.haemoglobin_g_l <= 0.0 || input.systolic_bp_mm_hg <= 0.0 || input.heart_rate_bpm <= 0.0
    {
        return Err(CalcError::InvalidInput(
            "haemoglobin, systolic blood pressure, and heart rate must be positive".into(),
        ));
    }

    let urea_points = urea_points(input.urea_mmol_l);
    let haemoglobin_points = haemoglobin_points(input.sex, input.haemoglobin_g_l);
    let systolic_bp_points = systolic_bp_points(input.systolic_bp_mm_hg);
    let heart_rate_points = u8::from(input.heart_rate_bpm >= 100.0);
    let melaena_points = u8::from(input.melaena);
    let syncope_points = 2 * u8::from(input.syncope);
    let hepatic_disease_points = 2 * u8::from(input.hepatic_disease);
    let cardiac_failure_points = 2 * u8::from(input.cardiac_failure);
    let score = urea_points
        + haemoglobin_points
        + systolic_bp_points
        + heart_rate_points
        + melaena_points
        + syncope_points
        + hepatic_disease_points
        + cardiac_failure_points;
    let triage_band = TriageBand::from_score(score);

    Ok(GlasgowBlatchfordOutcome {
        urea_points,
        haemoglobin_points,
        systolic_bp_points,
        heart_rate_points,
        melaena_points,
        syncope_points,
        hepatic_disease_points,
        cardiac_failure_points,
        score,
        triage_band,
        interpretation: render_interpretation(
            score,
            triage_band,
            input.emergency_department_overt_ugib,
        ),
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &GlasgowBlatchfordInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();

    working.insert("age_16_or_over".into(), json!(input.age_16_or_over));
    working.insert(
        "acute_upper_gi_bleeding_at_first_assessment".into(),
        json!(input.acute_upper_gi_bleeding_at_first_assessment),
    );
    working.insert(
        "emergency_department_overt_ugib".into(),
        json!(input.emergency_department_overt_ugib),
    );
    working.insert("sex".into(), json!(input.sex.label()));
    working.insert("urea_mmol_l".into(), json!(input.urea_mmol_l));
    working.insert("urea_points".into(), json!(outcome.urea_points));
    working.insert("haemoglobin_g_l".into(), json!(input.haemoglobin_g_l));
    working.insert(
        "haemoglobin_points".into(),
        json!(outcome.haemoglobin_points),
    );
    working.insert("systolic_bp_mm_hg".into(), json!(input.systolic_bp_mm_hg));
    working.insert(
        "systolic_bp_points".into(),
        json!(outcome.systolic_bp_points),
    );
    working.insert("heart_rate_bpm".into(), json!(input.heart_rate_bpm));
    working.insert("heart_rate_points".into(), json!(outcome.heart_rate_points));
    for (name, present, points) in [
        ("melaena", input.melaena, outcome.melaena_points),
        ("syncope", input.syncope, outcome.syncope_points),
        (
            "hepatic_disease",
            input.hepatic_disease,
            outcome.hepatic_disease_points,
        ),
        (
            "cardiac_failure",
            input.cardiac_failure,
            outcome.cardiac_failure_points,
        ),
    ] {
        working.insert(name.into(), json!(present));
        working.insert(format!("{name}_points"), json!(points));
    }
    working.insert("total_score".into(), json!(outcome.score));
    working.insert("maximum_score".into(), json!(23));
    working.insert("triage_band".into(), json!(outcome.triage_band.slug()));
    working.insert(
        "nice_score_zero_early_discharge_consideration".into(),
        json!(outcome.score == 0),
    );
    working.insert(
        "acg_score_zero_to_one_very_low_risk_example".into(),
        json!(input.emergency_department_overt_ugib && outcome.score <= 1),
    );
    working.insert(
        "intended_use".into(),
        json!("pre-endoscopy risk assessment at first presentation with acute upper gastrointestinal bleeding"),
    );
    working.insert(
        "original_outcome".into(),
        json!(
            "need for blood transfusion or intervention to control bleeding, rebleeding, or death"
        ),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.score),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn input_schema() -> Value {
    let primary_source = json!({
        "citation": "Blatchford O, Murray WR, Blatchford M. Lancet. 2000;356(9238):1318-1321.",
        "url": "https://doi.org/10.1016/S0140-6736(00)02816-6"
    });
    let operational_definition_source = json!({
        "citation": "Stanley AJ, Ashley D, Dalton HR, et al. Lancet. 2009;373(9657):42-47.",
        "url": "https://doi.org/10.1016/S0140-6736(08)61769-9"
    });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "GlasgowBlatchfordInput",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "age_16_or_over", "acute_upper_gi_bleeding_at_first_assessment",
            "emergency_department_overt_ugib", "sex", "urea_mmol_l", "haemoglobin_g_l",
            "systolic_bp_mm_hg", "heart_rate_bpm", "melaena", "syncope",
            "hepatic_disease", "cardiac_failure"
        ],
        "properties": {
            "age_16_or_over": {
                "type": "boolean",
                "description": "Confirm the person is aged 16 years or older; not scored, but required by this NICE CG141-based implementation",
                "definition": {
                    "concept": "Age within the NICE CG141 population",
                    "statement": "The person is aged 16 years or older.",
                    "excludes": ["Age under 16 years"],
                    "source": { "citation": "NICE CG141. Acute upper gastrointestinal bleeding in over 16s: management.", "url": "https://www.nice.org.uk/guidance/cg141/chapter/1-guidance" },
                    "status": "draft"
                }
            },
            "acute_upper_gi_bleeding_at_first_assessment": {
                "type": "boolean",
                "description": "Confirm this is the first pre-endoscopy assessment of the current acute upper-GI bleeding presentation; required but not scored",
                "definition": {
                    "concept": "Glasgow-Blatchford intended presentation",
                    "statement": "Use at first assessment of acute upper gastrointestinal bleeding, before endoscopy.",
                    "includes": ["Haematemesis", "Melaena", "Other clinically suspected acute bleeding proximal to the ligament of Treitz"],
                    "excludes": ["Lower gastrointestinal bleeding without suspected upper source", "Chronic iron-deficiency anaemia without acute upper-GI bleeding", "Use after endoscopy as a substitute for post-endoscopic assessment"],
                    "caveats": "Do not delay resuscitation or urgent management to calculate the score. The score does not diagnose the source of bleeding.",
                    "source": { "citation": "NICE CG141. Acute upper gastrointestinal bleeding in over 16s: management.", "url": "https://www.nice.org.uk/guidance/cg141/chapter/1-guidance" },
                    "status": "draft"
                }
            },
            "emergency_department_overt_ugib": {
                "type": "boolean",
                "description": "Whether the patient is presenting to an emergency department with overt upper gastrointestinal bleeding, the population for ACG 2021's conditional GBS 0-1 outpatient recommendation",
                "definition": {
                    "concept": "ACG very-low-risk recommendation population",
                    "statement": "True only when the patient is presenting to an emergency department with overt upper gastrointestinal bleeding.",
                    "includes": ["Emergency-department presentation with haematemesis", "Emergency-department presentation with melaena", "Emergency-department presentation with another overt manifestation of suspected upper-GI bleeding"],
                    "excludes": ["Inpatient-onset bleeding", "Occult bleeding without overt manifestation", "A non-emergency-department assessment"],
                    "caveats": "This field controls whether the ACG 2021 conditional outpatient recommendation is applied; it does not affect the arithmetic score or NICE CG141 context.",
                    "source": { "citation": "Laine L et al. ACG Clinical Guideline: Upper Gastrointestinal and Ulcer Bleeding. Am J Gastroenterol. 2021;116(5):899-917.", "url": "https://doi.org/10.14309/ajg.0000000000001245" },
                    "status": "draft"
                }
            },
            "sex": {
                "type": "string",
                "enum": ["male", "female"],
                "description": "Sex category used for the published sex-specific haemoglobin bands",
                "definition": {
                    "concept": "Published haemoglobin coefficient set",
                    "statement": "Select the male or female haemoglobin band used in the original Glasgow-Blatchford scoring table.",
                    "caveats": "The publication provides only male and female haemoglobin bands and does not specify how to score intersex patients or people receiving gender-affirming hormone therapy. Use clinical judgement where the published categories do not map safely to the patient.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "urea_mmol_l": {
                "type": "number",
                "minimum": 0,
                "description": "Blood urea in mmol/L: 6.5-7.9=2, 8.0-9.9=3, 10.0-24.9=4, >=25.0=6 points",
                "definition": {
                    "concept": "Blood urea at first assessment",
                    "statement": "Score the measured blood urea in mmol/L using the published bands.",
                    "excludes": ["Do not enter blood urea nitrogen (BUN) reported in mg/dL without conversion"],
                    "caveats": "UNIT TRAP: this input is urea in mmol/L, not BUN in mg/dL.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "haemoglobin_g_l": {
                "type": "number",
                "exclusiveMinimum": 0,
                "description": "Haemoglobin in g/L; scored using sex-specific published bands",
                "definition": {
                    "concept": "Haemoglobin at first assessment",
                    "statement": "Use haemoglobin in g/L at initial presentation: male 120-129=1, 100-119=3, <100=6; female 100-119=1, <100=6.",
                    "excludes": ["Do not enter haemoglobin in g/dL without multiplying by 10"],
                    "caveats": "UNIT TRAP: this input is g/L, not g/dL. In major acute blood loss, the initial haemoglobin can underestimate the severity of bleeding and must not override clinical assessment.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "systolic_bp_mm_hg": {
                "type": "number",
                "exclusiveMinimum": 0,
                "description": "Systolic blood pressure in mmHg: 100-109=1, 90-99=2, <90=3 points"
            },
            "heart_rate_bpm": {
                "type": "number",
                "exclusiveMinimum": 0,
                "description": "Heart rate in beats/min at first assessment; >=100 scores 1 point"
            },
            "melaena": {
                "type": "boolean",
                "description": "Presentation with melaena associated with the current acute bleed (1 point)",
                "definition": {
                    "concept": "Presentation with melaena",
                    "statement": "Black, tarry stool consistent with digested gastrointestinal blood as part of the current presentation.",
                    "includes": ["Clinician-observed melaena", "A credible history of black tarry stool during the current bleeding episode"],
                    "excludes": ["Dark stool explained by iron, bismuth, food, or another non-bleeding cause", "A remote history of melaena unrelated to the current presentation"],
                    "caveats": "Distinguish melaena from non-bloody dark stool using clinical assessment.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "syncope": {
                "type": "boolean",
                "description": "Presentation with syncope associated with the current acute bleed (2 points)",
                "definition": {
                    "concept": "Presentation with syncope",
                    "statement": "Transient loss of consciousness with spontaneous recovery associated with the current acute bleeding presentation.",
                    "includes": ["Witnessed syncope during the current episode", "A credible history of transient loss of consciousness during the current episode"],
                    "excludes": ["Presyncope or dizziness without loss of consciousness", "A remote or clearly unrelated episode of syncope", "Seizure without syncope"],
                    "caveats": "Investigate alternative causes of transient loss of consciousness where appropriate.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "hepatic_disease": {
                "type": "boolean",
                "description": "Known or clinically/laboratory-evident acute or chronic hepatic disease (2 points)",
                "definition": {
                    "concept": "Hepatic disease",
                    "statement": "Known history, or clinical and laboratory evidence, of chronic or acute liver disease.",
                    "includes": ["Known chronic liver disease or cirrhosis", "Known acute liver disease", "Clinical and laboratory evidence supporting acute or chronic liver disease"],
                    "excludes": ["An isolated abnormal liver blood test without a clinical diagnosis or supporting evidence", "Alcohol use alone without evidence of liver disease"],
                    "caveats": "The published score does not grade liver-disease severity.",
                    "source": operational_definition_source,
                    "status": "draft"
                }
            },
            "cardiac_failure": {
                "type": "boolean",
                "description": "Known or clinically/echocardiographically-evident cardiac failure (2 points)",
                "definition": {
                    "concept": "Cardiac failure",
                    "statement": "Known history, or clinical and echocardiographic evidence, of cardiac failure.",
                    "includes": ["Known diagnosis of heart failure", "Clinical evidence of heart failure supported by echocardiographic evidence"],
                    "excludes": ["Ischaemic heart disease alone", "Hypertension alone", "Isolated dyspnoea or peripheral oedema without evidence of cardiac failure"],
                    "caveats": "The published score does not distinguish reduced from preserved ejection fraction or grade heart-failure severity.",
                    "source": operational_definition_source,
                    "status": "draft"
                }
            }
        }
    })
}

/// Dynamic calculator implementation.
pub struct GlasgowBlatchford;

impl Calculator for GlasgowBlatchford {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Glasgow-Blatchford Bleeding Score (GBS)"
    }

    fn description(&self) -> &'static str {
        "Pre-endoscopy risk assessment at first presentation with acute upper gastrointestinal bleeding, primarily identifying patients at very low risk."
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
        let parsed: GlasgowBlatchfordInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score_zero() -> GlasgowBlatchfordInput {
        GlasgowBlatchfordInput {
            age_16_or_over: true,
            acute_upper_gi_bleeding_at_first_assessment: true,
            emergency_department_overt_ugib: true,
            sex: Sex::Male,
            urea_mmol_l: 6.4,
            haemoglobin_g_l: 130.0,
            systolic_bp_mm_hg: 110.0,
            heart_rate_bpm: 99.0,
            melaena: false,
            syncope: false,
            hepatic_disease: false,
            cardiac_failure: false,
        }
    }

    #[test]
    fn primary_table_zero_vector_scores_zero() {
        let outcome = compute(&score_zero()).unwrap();
        assert_eq!(outcome.score, 0);
        assert_eq!(outcome.triage_band, TriageBand::Zero);
        assert!(outcome.interpretation.contains("NICE CG141"));
    }

    #[test]
    fn primary_table_maximum_vector_scores_twenty_three() {
        let input = GlasgowBlatchfordInput {
            age_16_or_over: true,
            acute_upper_gi_bleeding_at_first_assessment: true,
            emergency_department_overt_ugib: true,
            sex: Sex::Female,
            urea_mmol_l: 25.0,
            haemoglobin_g_l: 99.0,
            systolic_bp_mm_hg: 89.0,
            heart_rate_bpm: 100.0,
            melaena: true,
            syncope: true,
            hepatic_disease: true,
            cardiac_failure: true,
        };
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.score, 23);
        assert_eq!(outcome.triage_band, TriageBand::AboveOne);
    }

    #[test]
    fn primary_table_urea_boundaries_are_exact() {
        let cases = [
            (6.499, 0),
            (6.5, 2),
            (7.999, 2),
            (8.0, 3),
            (9.999, 3),
            (10.0, 4),
            (24.999, 4),
            (25.0, 6),
        ];
        for (value, expected) in cases {
            assert_eq!(urea_points(value), expected, "urea {value}");
        }
    }

    #[test]
    fn primary_table_male_haemoglobin_boundaries_are_exact() {
        for (value, expected) in [
            (99.999, 6),
            (100.0, 3),
            (119.999, 3),
            (120.0, 1),
            (129.999, 1),
            (130.0, 0),
        ] {
            assert_eq!(haemoglobin_points(Sex::Male, value), expected);
        }
    }

    #[test]
    fn primary_table_female_haemoglobin_boundaries_are_exact() {
        for (value, expected) in [(99.999, 6), (100.0, 1), (119.999, 1), (120.0, 0)] {
            assert_eq!(haemoglobin_points(Sex::Female, value), expected);
        }
    }

    #[test]
    fn primary_table_systolic_pressure_boundaries_are_exact() {
        for (value, expected) in [
            (89.999, 3),
            (90.0, 2),
            (99.999, 2),
            (100.0, 1),
            (109.999, 1),
            (110.0, 0),
        ] {
            assert_eq!(systolic_bp_points(value), expected);
        }
    }

    #[test]
    fn primary_table_other_markers_score_published_points() {
        let mut input = score_zero();
        input.heart_rate_bpm = 100.0;
        input.melaena = true;
        input.syncope = true;
        input.hepatic_disease = true;
        input.cardiac_failure = true;
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.heart_rate_points, 1);
        assert_eq!(outcome.melaena_points, 1);
        assert_eq!(outcome.syncope_points, 2);
        assert_eq!(outcome.hepatic_disease_points, 2);
        assert_eq!(outcome.cardiac_failure_points, 2);
        assert_eq!(outcome.score, 8);
    }

    #[test]
    fn score_one_distinguishes_acg_and_nice_thresholds() {
        let mut input = score_zero();
        input.melaena = true;
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.score, 1);
        assert_eq!(outcome.triage_band, TriageBand::One);
        assert!(outcome.interpretation.contains("ACG 2021"));
        assert!(outcome.interpretation.contains("not within NICE"));
    }

    #[test]
    fn acg_outpatient_guidance_requires_emergency_department_overt_ugib_context() {
        let mut input = score_zero();
        input.emergency_department_overt_ugib = false;
        let outcome = compute(&input).unwrap();
        assert!(outcome.interpretation.contains("not applied"));

        let response = build_response(&input).unwrap();
        assert_eq!(
            response.working["acg_score_zero_to_one_very_low_risk_example"],
            json!(false)
        );
    }

    #[test]
    fn rejects_out_of_domain_or_invalid_observations() {
        let mut input = score_zero();
        input.age_16_or_over = false;
        assert!(compute(&input).is_err());

        input = score_zero();
        input.acute_upper_gi_bleeding_at_first_assessment = false;
        assert!(compute(&input).is_err());

        input = score_zero();
        input.urea_mmol_l = f64::NAN;
        assert!(compute(&input).is_err());

        input = score_zero();
        input.haemoglobin_g_l = 0.0;
        assert!(compute(&input).is_err());

        input = score_zero();
        input.heart_rate_bpm = -1.0;
        assert!(compute(&input).is_err());
    }

    #[test]
    fn response_preserves_raw_inputs_points_and_guideline_context() {
        let mut input = score_zero();
        input.sex = Sex::Female;
        input.urea_mmol_l = 8.0;
        input.haemoglobin_g_l = 110.0;
        input.systolic_bp_mm_hg = 95.0;
        let response = build_response(&input).unwrap();

        assert_eq!(response.result, json!(6));
        assert_eq!(response.working["urea_mmol_l"], json!(8.0));
        assert_eq!(response.working["urea_points"], json!(3));
        assert_eq!(response.working["haemoglobin_points"], json!(1));
        assert_eq!(response.working["systolic_bp_points"], json!(2));
        assert_eq!(response.working["maximum_score"], json!(23));
        assert_eq!(
            response.working["acg_score_zero_to_one_very_low_risk_example"],
            json!(false)
        );
        assert!(response.reference.contains("Blatchford O"));
        assert!(response.reference.contains("NICE CG141"));
    }

    #[test]
    fn dynamic_calculation_matches_typed_contract_and_rejects_unknown_fields() {
        let input = score_zero();
        let dynamic = GlasgowBlatchford
            .calculate(&serde_json::to_value(input).unwrap())
            .unwrap();
        assert_eq!(dynamic, build_response(&input).unwrap());

        let mut value = serde_json::to_value(input).unwrap();
        value["unexpected"] = json!(true);
        assert!(GlasgowBlatchford.calculate(&value).is_err());
    }

    #[test]
    fn schema_is_closed_and_defines_unit_and_predicate_traps() {
        let schema = GlasgowBlatchford.input_schema();
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"].as_array().unwrap().len(), 12);
        assert!(
            schema["properties"]["urea_mmol_l"]["definition"]["caveats"]
                .as_str()
                .unwrap()
                .contains("BUN")
        );
        assert!(
            schema["properties"]["hepatic_disease"]["definition"]["statement"]
                .as_str()
                .unwrap()
                .contains("clinical and laboratory evidence")
        );
        assert!(
            schema["properties"]["cardiac_failure"]["definition"]["statement"]
                .as_str()
                .unwrap()
                .contains("echocardiographic evidence")
        );
        assert!(
            schema["properties"]["hepatic_disease"]["definition"]["source"]["citation"]
                .as_str()
                .unwrap()
                .contains("Stanley AJ")
        );
    }
}
