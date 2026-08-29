// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Clinical Opiate Withdrawal Scale (COWS).
//!
//! COWS quantifies current clinician-assessed findings apparently attributable
//! to opioid withdrawal. It does not determine medication timing or dosage.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "cows";
pub const REFERENCE: &str = "Wesson DR, Ling W. The Clinical Opiate Withdrawal Scale (COWS). J Psychoactive Drugs. 2003;35(2):253-259. doi:10.1080/02791072.2003.10400007. PMID:12924748. Tompkins DA, Bigelow GE, Harrison JA, Johnson RE, Fudala PJ, Strain EC. Concurrent validation of the Clinical Opiate Withdrawal Scale against the Clinical Institute Narcotic Assessment. Drug Alcohol Depend. 2009;105(1-2):154-159. doi:10.1016/j.drugalcdep.2009.07.001.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "No copyright indicated by NIH; the published COWS template may be copied and used clinically",
    source_url: "https://www.nih.gov/node/21071",
};

const LIMITATIONS: &str = "COWS quantifies current clinician-assessed signs and symptoms apparently attributable to opioid withdrawal. It does not independently diagnose withdrawal, establish opioid tolerance, distinguish every alternative cause, or determine medication timing or dosage. Account for exercise and other causes of tachycardia, room temperature or activity causing sweating, allergy or infection causing rhinorrhoea, pre-existing pain, other substances, medications, anxiety, gastrointestinal illness, and autonomic disorders. Never infer that a score makes buprenorphine administration safe.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentContext {
    ClinicianAssessmentOfCurrentPossibleOpioidWithdrawal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CowsInput {
    pub assessment_context: AssessmentContext,
    pub resting_pulse_rate_bpm: u16,
    pub sweating: u8,
    pub restlessness: u8,
    pub pupil_size: u8,
    pub bone_or_joint_aches: u8,
    pub runny_nose_or_tearing: u8,
    pub gastrointestinal_upset: u8,
    pub tremor: u8,
    pub yawning: u8,
    pub anxiety_or_irritability: u8,
    pub gooseflesh_skin: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WithdrawalSeverity {
    BelowMildThreshold,
    Mild,
    Moderate,
    ModeratelySevere,
    Severe,
}

impl WithdrawalSeverity {
    fn from_total(total: u8) -> Self {
        match total {
            0..=4 => Self::BelowMildThreshold,
            5..=12 => Self::Mild,
            13..=24 => Self::Moderate,
            25..=36 => Self::ModeratelySevere,
            _ => Self::Severe,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::BelowMildThreshold => "below_mild_threshold",
            Self::Mild => "mild",
            Self::Moderate => "moderate",
            Self::ModeratelySevere => "moderately_severe",
            Self::Severe => "severe",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CowsOutcome {
    pub pulse_points: u8,
    pub total: u8,
    pub severity: WithdrawalSeverity,
    pub interpretation: &'static str,
}

fn pulse_points(pulse: u16) -> u8 {
    match pulse {
        0..=80 => 0,
        81..=100 => 1,
        101..=120 => 2,
        _ => 4,
    }
}

fn valid_score(name: &str, value: u8) -> bool {
    match name {
        "sweating" => value <= 4,
        "restlessness" => matches!(value, 0 | 1 | 3 | 5),
        "pupil_size" => matches!(value, 0 | 1 | 2 | 5),
        "bone_or_joint_aches"
        | "runny_nose_or_tearing"
        | "tremor"
        | "yawning"
        | "anxiety_or_irritability" => matches!(value, 0 | 1 | 2 | 4),
        "gastrointestinal_upset" => matches!(value, 0 | 1 | 2 | 3 | 5),
        "gooseflesh_skin" => matches!(value, 0 | 3 | 5),
        _ => false,
    }
}

fn named_scores(input: &CowsInput) -> [(&'static str, u8); 10] {
    [
        ("sweating", input.sweating),
        ("restlessness", input.restlessness),
        ("pupil_size", input.pupil_size),
        ("bone_or_joint_aches", input.bone_or_joint_aches),
        ("runny_nose_or_tearing", input.runny_nose_or_tearing),
        ("gastrointestinal_upset", input.gastrointestinal_upset),
        ("tremor", input.tremor),
        ("yawning", input.yawning),
        ("anxiety_or_irritability", input.anxiety_or_irritability),
        ("gooseflesh_skin", input.gooseflesh_skin),
    ]
}

pub fn compute(input: &CowsInput) -> Result<CowsOutcome, CalcError> {
    if !(1..=300).contains(&input.resting_pulse_rate_bpm) {
        return Err(CalcError::InvalidInput(
            "resting_pulse_rate_bpm must be between 1 and 300".into(),
        ));
    }
    for (name, score) in named_scores(input) {
        if !valid_score(name, score) {
            return Err(CalcError::InvalidInput(format!(
                "{name} score {score} is not a published COWS option"
            )));
        }
    }

    let pulse_points = pulse_points(input.resting_pulse_rate_bpm);
    let total = pulse_points
        + named_scores(input)
            .iter()
            .map(|(_, score)| score)
            .sum::<u8>();
    let severity = WithdrawalSeverity::from_total(total);
    let interpretation = match severity {
        WithdrawalSeverity::BelowMildThreshold => {
            "below the instrument's mild-withdrawal threshold"
        }
        WithdrawalSeverity::Mild => "within the mild range",
        WithdrawalSeverity::Moderate => "within the moderate range",
        WithdrawalSeverity::ModeratelySevere => "within the moderately severe range",
        WithdrawalSeverity::Severe => "within the severe range",
    };

    Ok(CowsOutcome {
        pulse_points,
        total,
        severity,
        interpretation,
    })
}

pub fn build_response(input: &CowsInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let interpretation = format!(
        "COWS score {}/48: {}. {LIMITATIONS}",
        outcome.total, outcome.interpretation
    );
    let mut working = Map::new();
    working.insert("assessment_context".into(), json!(input.assessment_context));
    working.insert(
        "resting_pulse_rate_bpm".into(),
        json!(input.resting_pulse_rate_bpm),
    );
    working.insert(
        "resting_pulse_rate_points".into(),
        json!(outcome.pulse_points),
    );
    for (name, score) in named_scores(input) {
        working.insert(name.into(), json!(score));
    }
    working.insert("total_score".into(), json!(outcome.total));
    working.insert("maximum_score".into(), json!(48));
    working.insert("severity_band".into(), json!(outcome.severity.slug()));
    working.insert("limitations".into(), json!(LIMITATIONS));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.total),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn scored_property(description: &str, allowed: &[u8], source: &Value) -> Value {
    json!({
        "type": "integer",
        "enum": allowed,
        "description": description,
        "definition": {
            "concept": "COWS clinician rating",
            "statement": description,
            "excludes": ["A value not present in the published non-contiguous options", "A finding attributable to another cause rather than opioid withdrawal"],
            "caveats": "Rate only the apparent relationship to opioid withdrawal and account for relevant baseline findings and alternative causes.",
            "source": source,
            "status": "draft"
        }
    })
}

fn input_schema() -> Value {
    let source = json!({
        "citation": "Wesson DR, Ling W. J Psychoactive Drugs. 2003;35(2):253-259.",
        "url": "https://doi.org/10.1080/02791072.2003.10400007"
    });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "CowsInput",
        "description": "COWS clinician assessment of current signs and symptoms apparently attributable to opioid withdrawal. The total quantifies observed severity but does not independently diagnose withdrawal, establish tolerance, or determine medication timing or dosage.",
        "type": "object",
        "additionalProperties": false,
        "required": ["assessment_context", "resting_pulse_rate_bpm", "sweating", "restlessness", "pupil_size", "bone_or_joint_aches", "runny_nose_or_tearing", "gastrointestinal_upset", "tremor", "yawning", "anxiety_or_irritability", "gooseflesh_skin"],
        "properties": {
            "assessment_context": {
                "type": "string",
                "const": "clinician_assessment_of_current_possible_opioid_withdrawal",
                "description": "A clinician is assessing current findings for their apparent relationship to possible opioid withdrawal",
                "definition": {
                    "concept": "COWS assessment context",
                    "statement": "Use during a clinician-administered assessment in which every finding is rated only for its apparent relationship to opioid withdrawal.",
                    "excludes": ["Automatic medication selection", "Use as proof of opioid tolerance", "Scoring findings known to arise from another cause"],
                    "caveats": "COWS does not establish that buprenorphine or another medication is safe to administer; timing and treatment require a complete clinical protocol.",
                    "source": source, "status": "draft"
                }
            },
            "resting_pulse_rate_bpm": {
                "type": "integer", "minimum": 1, "maximum": 300, "unit": "beats/min",
                "description": "Resting pulse after sitting or lying for at least one minute: <=80=0; 81-100=1; 101-120=2; >120=4 points. The 1-300 beats/min range is a broad software safety guardrail, not an instrument threshold.",
                "definition": {
                    "concept": "COWS resting pulse rate",
                    "statement": "Measure pulse after the patient has been sitting or lying for at least one minute; enter the raw beats per minute and let the calculator derive points.",
                    "excludes": ["Pulse elevation attributable to recent exercise or another known cause", "Caller-supplied pulse points"],
                    "caveats": "Medications, anxiety, fever, pain, arrhythmia, dehydration, and autonomic disorders can alter pulse independently of opioid withdrawal.",
                    "source": source, "status": "draft"
                }
            },
            "sweating": scored_property("Sweating over the past 30 minutes, excluding room temperature/activity: 0 none; 1 subjective chills/flushing; 2 flushed or observable facial moisture; 3 beads on brow/face; 4 sweat streaming off face", &[0,1,2,3,4], &source),
            "restlessness": scored_property("Restlessness during assessment: 0 sits still; 1 reports difficulty but can sit; 3 frequent shifting/extraneous limb movement; 5 cannot sit still for more than a few seconds", &[0,1,3,5], &source),
            "pupil_size": scored_property("Pupil size for room light: 0 pinned/normal; 1 possibly larger than normal; 2 moderately dilated; 5 only the rim of iris visible", &[0,1,2,5], &source),
            "bone_or_joint_aches": scored_property("Additional bone/joint aches attributable to withdrawal: 0 absent; 1 mild diffuse discomfort; 2 severe diffuse joint/muscle ache reported; 4 rubbing joints/muscles and unable to sit still", &[0,1,2,4], &source),
            "runny_nose_or_tearing": scored_property("Runny nose/tearing not explained by cold or allergy: 0 absent; 1 stuffiness/unusually moist eyes; 2 running nose or tearing; 4 constant running or tears streaming", &[0,1,2,4], &source),
            "gastrointestinal_upset": scored_property("GI upset over the past 30 minutes: 0 none; 1 cramps; 2 nausea or loose stool; 3 vomiting or diarrhoea; 5 multiple episodes", &[0,1,2,3,5], &source),
            "tremor": scored_property("Tremor with outstretched hands: 0 none; 1 felt but not observed; 2 slight observable; 4 gross tremor or muscle twitching", &[0,1,2,4], &source),
            "yawning": scored_property("Yawning during assessment: 0 none; 1 once or twice; 2 three or more times; 4 several times per minute", &[0,1,2,4], &source),
            "anxiety_or_irritability": scored_property("Anxiety/irritability: 0 none; 1 increasing by report; 2 obviously irritable/anxious; 4 participation difficult because of severity", &[0,1,2,4], &source),
            "gooseflesh_skin": scored_property("Gooseflesh: 0 smooth skin; 3 piloerection felt or hairs standing on arms; 5 prominent piloerection", &[0,3,5], &source)
        }
    })
}

pub struct Cows;

impl Calculator for Cows {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Clinical Opiate Withdrawal Scale (COWS)"
    }
    fn description(&self) -> &'static str {
        "Quantifies current clinician-assessed signs and symptoms apparently attributable to opioid withdrawal."
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
        let parsed: CowsInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero() -> CowsInput {
        CowsInput {
            assessment_context:
                AssessmentContext::ClinicianAssessmentOfCurrentPossibleOpioidWithdrawal,
            resting_pulse_rate_bpm: 80,
            sweating: 0,
            restlessness: 0,
            pupil_size: 0,
            bone_or_joint_aches: 0,
            runny_nose_or_tearing: 0,
            gastrointestinal_upset: 0,
            tremor: 0,
            yawning: 0,
            anxiety_or_irritability: 0,
            gooseflesh_skin: 0,
        }
    }

    #[test]
    fn zero_and_mathematical_maximum_are_exact() {
        assert_eq!(compute(&zero()).unwrap().total, 0);
        let maximum = CowsInput {
            resting_pulse_rate_bpm: 121,
            sweating: 4,
            restlessness: 5,
            pupil_size: 5,
            bone_or_joint_aches: 4,
            runny_nose_or_tearing: 4,
            gastrointestinal_upset: 5,
            tremor: 4,
            yawning: 4,
            anxiety_or_irritability: 4,
            gooseflesh_skin: 5,
            ..zero()
        };
        assert_eq!(compute(&maximum).unwrap().total, 48);
    }

    #[test]
    fn published_severity_boundaries_are_exact() {
        for (total, expected) in [
            (4, WithdrawalSeverity::BelowMildThreshold),
            (5, WithdrawalSeverity::Mild),
            (12, WithdrawalSeverity::Mild),
            (13, WithdrawalSeverity::Moderate),
            (24, WithdrawalSeverity::Moderate),
            (25, WithdrawalSeverity::ModeratelySevere),
            (36, WithdrawalSeverity::ModeratelySevere),
            (37, WithdrawalSeverity::Severe),
            (48, WithdrawalSeverity::Severe),
        ] {
            assert_eq!(WithdrawalSeverity::from_total(total), expected);
        }
    }

    #[test]
    fn pulse_boundaries_are_derived_from_raw_rate() {
        for (pulse, expected) in [(80, 0), (81, 1), (100, 1), (101, 2), (120, 2), (121, 4)] {
            assert_eq!(pulse_points(pulse), expected);
        }
    }

    #[test]
    fn every_non_contiguous_domain_rejects_invented_values() {
        for (name, invalid) in [
            ("sweating", 5),
            ("restlessness", 2),
            ("pupil_size", 3),
            ("bone_or_joint_aches", 3),
            ("runny_nose_or_tearing", 3),
            ("gastrointestinal_upset", 4),
            ("tremor", 3),
            ("yawning", 3),
            ("anxiety_or_irritability", 3),
            ("gooseflesh_skin", 4),
        ] {
            assert!(!valid_score(name, invalid), "accepted {name}={invalid}");
        }
    }

    #[test]
    fn response_does_not_diagnose_or_authorise_medication() {
        let input = CowsInput {
            gastrointestinal_upset: 5,
            ..zero()
        };
        let response = build_response(&input).unwrap();
        assert_eq!(response.result, json!(5));
        assert_eq!(response.working["severity_band"], json!("mild"));
        for text in [
            "does not independently diagnose",
            "medication timing or dosage",
            "Never infer",
            "buprenorphine",
        ] {
            assert!(response.interpretation.contains(text));
        }
        assert!(!response.interpretation.contains("safe for"));
    }

    #[test]
    fn rejects_zero_pulse_unknown_fields_and_invalid_context() {
        let mut input = zero();
        input.resting_pulse_rate_bpm = 0;
        assert!(compute(&input).is_err());
        input.resting_pulse_rate_bpm = 301;
        assert!(compute(&input).is_err());
        let value = serde_json::to_value(zero()).unwrap();
        let mut unknown = value.clone();
        unknown["buprenorphine_planned"] = json!(true);
        assert!(Cows.calculate(&unknown).is_err());
        let mut context = value;
        context["assessment_context"] = json!("self_assessment");
        assert!(Cows.calculate(&context).is_err());
    }

    #[test]
    fn dynamic_response_matches_typed_and_preserves_points() {
        let input = CowsInput {
            resting_pulse_rate_bpm: 101,
            sweating: 3,
            ..zero()
        };
        let dynamic = Cows
            .calculate(&serde_json::to_value(input).unwrap())
            .unwrap();
        assert_eq!(dynamic, build_response(&input).unwrap());
        assert_eq!(dynamic.result, json!(5));
        assert_eq!(dynamic.working["resting_pulse_rate_points"], json!(2));
        assert_eq!(dynamic.working["sweating"], json!(3));
    }

    #[test]
    fn schema_is_closed_and_records_exact_domains_and_rights() {
        let schema = Cows.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["required"].as_array().unwrap().len(), 12);
        assert_eq!(
            schema["properties"]["resting_pulse_rate_bpm"]["maximum"],
            json!(300)
        );
        assert_eq!(
            schema["properties"]["restlessness"]["enum"],
            json!([0, 1, 3, 5])
        );
        assert_eq!(
            schema["properties"]["gooseflesh_skin"]["enum"],
            json!([0, 3, 5])
        );
        assert!(
            schema["description"]
                .as_str()
                .unwrap()
                .contains("does not independently diagnose")
        );
        assert!(Cows.license().license.contains("No copyright"));
        for property in schema["properties"].as_object().unwrap().values() {
            assert!(property["definition"]["statement"].is_string());
            assert_eq!(property["definition"]["status"], json!("draft"));
        }
    }
}
