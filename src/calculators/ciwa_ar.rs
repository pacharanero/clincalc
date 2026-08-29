// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Clinical Institute Withdrawal Assessment for Alcohol, Revised (CIWA-Ar).
//!
//! CIWA-Ar measures current withdrawal severity after alcohol withdrawal has
//! been clinically identified. It is not a diagnostic or medication protocol.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "ciwa_ar";
pub const REFERENCE: &str = "Sullivan JT, Sykora K, Schneiderman J, Naranjo CA, Sellers EM. Assessment of alcohol withdrawal: the revised Clinical Institute Withdrawal Assessment for Alcohol scale (CIWA-Ar). Br J Addict. 1989;84(11):1353-1357. doi:10.1111/j.1360-0443.1989.tb00737.x. The ASAM Clinical Practice Guideline on Alcohol Withdrawal Management. J Addict Med. 2020;14(3S Suppl 1):1-72.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Not copyrighted - the CIWA-Ar may be reproduced freely",
    source_url: "https://www.ci2i.research.va.gov/paws/pdfs/ciwa-ar.pdf",
};

const LIMITATIONS: &str = "CIWA-Ar measures current symptom severity after alcohol withdrawal has been clinically identified. It does not diagnose withdrawal, independently predict future seizures or delirium, or prescribe treatment; a high initial score may contribute to risk assessment but must not be used alone. Seven components require reliable patient communication. Do not use a total when delirium, dementia, psychosis, intubation, severe cognitive impairment, a mechanical communication barrier, or an unmanaged language barrier prevents reliable participation. Pain, head injury, psychiatric symptoms, intoxication, baseline tremor, infection, and medication effects can confound scores.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentContext {
    ClinicallyIdentifiedAlcoholWithdrawalWithReliablePatientParticipation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiwaArInput {
    pub assessment_context: AssessmentContext,
    pub nausea_and_vomiting: u8,
    pub tremor: u8,
    pub paroxysmal_sweats: u8,
    pub anxiety: u8,
    pub agitation: u8,
    pub tactile_disturbances: u8,
    pub auditory_disturbances: u8,
    pub visual_disturbances: u8,
    pub headache_or_fullness: u8,
    pub orientation_and_clouding: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WithdrawalSeverity {
    Mild,
    Moderate,
    Severe,
}

impl WithdrawalSeverity {
    fn from_total(total: u8) -> Self {
        match total {
            0..=9 => Self::Mild,
            10..=18 => Self::Moderate,
            _ => Self::Severe,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::Mild => "mild",
            Self::Moderate => "moderate",
            Self::Severe => "severe",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CiwaArOutcome {
    pub total: u8,
    pub severity: WithdrawalSeverity,
    pub interpretation: String,
}

fn named_scores(input: &CiwaArInput) -> [(&'static str, u8, u8); 10] {
    [
        ("nausea_and_vomiting", input.nausea_and_vomiting, 7),
        ("tremor", input.tremor, 7),
        ("paroxysmal_sweats", input.paroxysmal_sweats, 7),
        ("anxiety", input.anxiety, 7),
        ("agitation", input.agitation, 7),
        ("tactile_disturbances", input.tactile_disturbances, 7),
        ("auditory_disturbances", input.auditory_disturbances, 7),
        ("visual_disturbances", input.visual_disturbances, 7),
        ("headache_or_fullness", input.headache_or_fullness, 7),
        (
            "orientation_and_clouding",
            input.orientation_and_clouding,
            4,
        ),
    ]
}

pub fn compute(input: &CiwaArInput) -> Result<CiwaArOutcome, CalcError> {
    let scores = named_scores(input);
    for (name, score, maximum) in scores {
        if score > maximum {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be between 0 and {maximum}"
            )));
        }
    }

    let total = scores.iter().map(|(_, score, _)| score).sum();
    let severity = WithdrawalSeverity::from_total(total);
    let interpretation = format!(
        "CIWA-Ar score {total}/67: within the ASAM example {} withdrawal-severity range. {LIMITATIONS}",
        severity.slug()
    );

    Ok(CiwaArOutcome {
        total,
        severity,
        interpretation,
    })
}

pub fn build_response(input: &CiwaArInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert("assessment_context".into(), json!(input.assessment_context));
    for (name, score, _) in named_scores(input) {
        working.insert(name.into(), json!(score));
    }
    working.insert("total_score".into(), json!(outcome.total));
    working.insert("maximum_score".into(), json!(67));
    working.insert("severity_band".into(), json!(outcome.severity.slug()));
    working.insert("band_source".into(), json!("ASAM_2020_examples"));
    working.insert("limitations".into(), json!(LIMITATIONS));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.total),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn scored_property(description: &str, maximum: u8, source: &Value) -> Value {
    json!({
        "type": "integer",
        "minimum": 0,
        "maximum": maximum,
        "description": description,
        "definition": {
            "concept": "CIWA-Ar clinician rating",
            "statement": description,
            "excludes": ["A guessed value when the item cannot be assessed reliably", "A symptom clearly attributable to another cause without clinical interpretation"],
            "caveats": "Select the published integer rating that best describes the current assessment. Never silently score an unassessable item as zero.",
            "source": source,
            "status": "draft"
        }
    })
}

fn input_schema() -> Value {
    let source = json!({
        "citation": "Sullivan JT, Sykora K, Schneiderman J, Naranjo CA, Sellers EM. Br J Addict. 1989;84(11):1353-1357.",
        "url": "https://doi.org/10.1111/j.1360-0443.1989.tb00737.x"
    });
    let context_source = json!({
        "citation": "The ASAM Clinical Practice Guideline on Alcohol Withdrawal Management. 2020.",
        "url": "https://downloads.asam.org/sitefinity-production-blobs/docs/default-source/guidelines/awg-3-20-20.pdf"
    });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "CiwaArInput",
        "description": "CIWA-Ar current alcohol-withdrawal symptom-severity assessment. All ten ratings and confirmation of the supported assessment context are required. The scale is not diagnostic, cannot be completed reliably without patient participation, and does not prescribe medication.",
        "type": "object",
        "additionalProperties": false,
        "required": ["assessment_context", "nausea_and_vomiting", "tremor", "paroxysmal_sweats", "anxiety", "agitation", "tactile_disturbances", "auditory_disturbances", "visual_disturbances", "headache_or_fullness", "orientation_and_clouding"],
        "properties": {
            "assessment_context": {
                "type": "string",
                "const": "clinically_identified_alcohol_withdrawal_with_reliable_patient_participation",
                "description": "Alcohol withdrawal has been clinically identified and the alert, communicative patient can participate reliably in every required item",
                "definition": {
                    "concept": "CIWA-Ar supported assessment context",
                    "statement": "Confirm that alcohol withdrawal has already been clinically identified and that the patient can communicate and participate reliably throughout this assessment.",
                    "excludes": ["Use as a diagnostic test", "Delirium, dementia, psychosis, intubation, severe cognitive impairment, or another state preventing reliable answers", "Mechanical or unmanaged language barriers preventing reliable communication"],
                    "caveats": "Seven of ten components require patient communication. Use a more objective clinical approach when reliable participation is unavailable.",
                    "source": context_source,
                    "status": "draft"
                }
            },
            "nausea_and_vomiting": scored_property("Nausea/vomiting rating 0-7: 0 none; 1 mild nausea without vomiting; 4 intermittent nausea with dry heaves; 7 constant nausea with frequent dry heaves and vomiting", 7, &source),
            "tremor": scored_property("Tremor rating 0-7 with arms extended and fingers spread: 0 none; 1 felt fingertip-to-fingertip but not visible; 4 moderate with arms extended; 7 severe even without extension", 7, &source),
            "paroxysmal_sweats": scored_property("Paroxysmal-sweats rating 0-7: 0 none visible; 1 barely perceptible/palms moist; 4 beads obvious on forehead; 7 drenching sweats", 7, &source),
            "anxiety": scored_property("Anxiety rating 0-7: 0 at ease; 1 mildly anxious; 4 moderately anxious or guarded so anxiety is inferred; 7 equivalent to an acute panic state", 7, &source),
            "agitation": scored_property("Agitation rating 0-7: 0 normal activity; 1 somewhat increased; 4 moderately fidgety/restless; 7 pacing most of the interview or constantly thrashing", 7, &source),
            "tactile_disturbances": scored_property("Tactile-disturbance rating 0-7 from none through increasing itching, pins-and-needles, burning, numbness, or tactile hallucinations; 4 moderately severe hallucinations and 7 continuous hallucinations", 7, &source),
            "auditory_disturbances": scored_property("Auditory-disturbance rating 0-7 from none through increasing harshness/frightening sounds or auditory hallucinations; 4 moderately severe hallucinations and 7 continuous hallucinations", 7, &source),
            "visual_disturbances": scored_property("Visual-disturbance rating 0-7 from none through increasing light sensitivity/visual disturbance or hallucinations; 4 moderately severe hallucinations and 7 continuous hallucinations", 7, &source),
            "headache_or_fullness": scored_property("Headache/fullness rating 0-7: rate severity from absent to extremely severe, excluding dizziness or light-headedness", 7, &source),
            "orientation_and_clouding": scored_property("Orientation/clouding rating 0-4: 0 oriented and can do serial additions; 1 cannot do additions or uncertain date; 2 date error no more than 2 days; 3 date error over 2 days; 4 disoriented to place or person", 4, &source)
        }
    })
}

pub struct CiwaAr;

impl Calculator for CiwaAr {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "CIWA-Ar Alcohol Withdrawal Severity"
    }
    fn description(&self) -> &'static str {
        "Quantifies current alcohol-withdrawal symptom severity in an alert, communicative patient after withdrawal has been clinically identified."
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
        let parsed: CiwaArInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero() -> CiwaArInput {
        CiwaArInput {
            assessment_context: AssessmentContext::ClinicallyIdentifiedAlcoholWithdrawalWithReliablePatientParticipation,
            nausea_and_vomiting: 0,
            tremor: 0,
            paroxysmal_sweats: 0,
            anxiety: 0,
            agitation: 0,
            tactile_disturbances: 0,
            auditory_disturbances: 0,
            visual_disturbances: 0,
            headache_or_fullness: 0,
            orientation_and_clouding: 0,
        }
    }

    #[test]
    fn zero_and_published_maximum_are_exact() {
        assert_eq!(compute(&zero()).unwrap().total, 0);
        let maximum = CiwaArInput {
            nausea_and_vomiting: 7,
            tremor: 7,
            paroxysmal_sweats: 7,
            anxiety: 7,
            agitation: 7,
            tactile_disturbances: 7,
            auditory_disturbances: 7,
            visual_disturbances: 7,
            headache_or_fullness: 7,
            orientation_and_clouding: 4,
            ..zero()
        };
        assert_eq!(compute(&maximum).unwrap().total, 67);
    }

    #[test]
    fn asam_example_band_boundaries_are_exact() {
        for (total, expected) in [
            (9, WithdrawalSeverity::Mild),
            (10, WithdrawalSeverity::Moderate),
            (18, WithdrawalSeverity::Moderate),
            (19, WithdrawalSeverity::Severe),
        ] {
            assert_eq!(WithdrawalSeverity::from_total(total), expected);
        }
    }

    #[test]
    fn rejects_out_of_range_item_scores() {
        let mut input = zero();
        input.nausea_and_vomiting = 8;
        assert!(compute(&input).is_err());
        input = zero();
        input.orientation_and_clouding = 5;
        assert!(compute(&input).is_err());
    }

    #[test]
    fn response_is_a_severity_measure_not_a_treatment_protocol() {
        let input = CiwaArInput {
            anxiety: 7,
            agitation: 7,
            tremor: 7,
            ..zero()
        };
        let response = build_response(&input).unwrap();
        assert_eq!(response.result, json!(21));
        assert_eq!(response.working["severity_band"], json!("severe"));
        for text in [
            "does not diagnose",
            "independently predict",
            "must not be used alone",
            "prescribe treatment",
            "reliable patient communication",
        ] {
            assert!(response.interpretation.contains(text));
        }
        assert!(!response.interpretation.contains("benzodiazepine"));
    }

    #[test]
    fn dynamic_api_is_closed_and_matches_typed_response() {
        let input = zero();
        let value = serde_json::to_value(input).unwrap();
        assert_eq!(
            CiwaAr.calculate(&value).unwrap(),
            build_response(&input).unwrap()
        );
        let mut unknown = value.clone();
        unknown["pulse_bpm"] = json!(100);
        assert!(CiwaAr.calculate(&unknown).is_err());
        let mut invalid_context = value;
        invalid_context["assessment_context"] = json!("unable_to_communicate");
        assert!(CiwaAr.calculate(&invalid_context).is_err());
    }

    #[test]
    fn schema_is_closed_complete_and_records_free_reproduction() {
        let schema = CiwaAr.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["required"].as_array().unwrap().len(), 11);
        assert_eq!(
            schema["properties"]["orientation_and_clouding"]["maximum"],
            json!(4)
        );
        assert!(
            schema["properties"]["assessment_context"]["definition"]["excludes"]
                .to_string()
                .contains("intubation")
        );
        assert!(CiwaAr.license().license.contains("reproduced freely"));
        for property in schema["properties"].as_object().unwrap().values() {
            assert!(property["definition"]["statement"].is_string());
            assert_eq!(property["definition"]["status"], json!("draft"));
        }
    }
}
