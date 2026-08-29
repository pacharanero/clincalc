// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Pitt Bacteraemia Score for acute severity of illness in infection.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "pitt_bacteraemia";
pub const REFERENCE: &str = "Henderson H, Luterbach CL, Cober E, et al. The Pitt Bacteremia Score Predicts Mortality in Nonbacteremic Infections. Clin Infect Dis. 2020;70(9):1826-1833. doi:10.1093/cid/ciz528. Hilf M, Yu VL, Sharp J, Zuravleff JJ, Korvick JA, Muder RR. Antibiotic therapy for Pseudomonas aeruginosa bacteremia: outcome correlations in a prospective study of 200 patients. Am J Med. 1989;87(5):540-546. PMID:2816969.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Uncopyrightable method under 17 U.S.C. Section 102(b) - independently implemented from primary literature",
    source_url: "https://uscode.house.gov/view.xhtml?req=granuleid:USC-prelim-title17-section102&num=0&edition=prelim",
};

const LIMITATIONS: &str = "The Pitt Bacteraemia Score is an acute severity and mortality-risk stratification measure, primarily established in hospitalised infection and bloodstream-infection cohorts. It is not a diagnosis, patient-specific mortality probability, antibiotic recommendation, ICU-admission rule, or goals-of-care decision. The score must not replace assessment of infection source, organ dysfunction, comorbidity, treatment response, or clinical trajectory.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentContext {
    HospitalisedPatientWithBloodstreamInfectionAndIndexCulture,
    HospitalisedPatientWithCreInfectionAndIndexCulture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MentalStatus {
    Normal,
    Disoriented,
    Stuporous,
    Comatose,
    NotAssessable,
}

impl MentalStatus {
    fn points(self) -> Option<u8> {
        match self {
            MentalStatus::Normal => Some(0),
            MentalStatus::Disoriented => Some(1),
            MentalStatus::Stuporous => Some(2),
            MentalStatus::Comatose => Some(4),
            MentalStatus::NotAssessable => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PittBacteraemiaInput {
    pub assessment_context: AssessmentContext,
    pub maximum_temperature_c: f64,
    pub acute_hypotension_on_index_culture_day: bool,
    pub mechanical_ventilation_on_index_culture_day: bool,
    pub cardiac_arrest_on_index_day_or_prior_48_hours: bool,
    pub worst_mental_status_on_index_culture_day: MentalStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PittBacteraemiaOutcome {
    pub temperature_points: u8,
    pub hypotension_points: u8,
    pub mechanical_ventilation_points: u8,
    pub cardiac_arrest_points: u8,
    pub mental_status_points: u8,
    pub score: u8,
    pub score_at_least_four: bool,
    pub interpretation: String,
}

fn temperature_points(temperature_c: f64) -> u8 {
    if temperature_c <= 35.0 || temperature_c >= 40.0 {
        2
    } else if temperature_c <= 36.0 || temperature_c >= 39.0 {
        1
    } else {
        0
    }
}

pub fn compute(input: &PittBacteraemiaInput) -> Result<PittBacteraemiaOutcome, CalcError> {
    if !input.maximum_temperature_c.is_finite() {
        return Err(CalcError::InvalidInput(
            "maximum_temperature_c must be finite".into(),
        ));
    }
    if !(20.0..=45.0).contains(&input.maximum_temperature_c) {
        return Err(CalcError::InvalidInput(
            "maximum_temperature_c must be between 20.0 and 45.0 degrees Celsius".into(),
        ));
    }
    let temperature_tenths = input.maximum_temperature_c * 10.0;
    if !temperature_tenths.is_finite()
        || (temperature_tenths - temperature_tenths.round()).abs() > 1e-9
    {
        return Err(CalcError::InvalidInput(
            "maximum_temperature_c must be recorded to one decimal place because the source bands leave finer values undefined".into(),
        ));
    }
    let mental_status_points = input
        .worst_mental_status_on_index_culture_day
        .points()
        .ok_or_else(|| {
            CalcError::InvalidInput(
                "worst_mental_status_on_index_culture_day must be clinically assessable; do not score iatrogenic sedation or another unassessable state as coma".into(),
            )
        })?;
    let temperature_points = temperature_points(input.maximum_temperature_c);
    let hypotension_points = if input.acute_hypotension_on_index_culture_day {
        2
    } else {
        0
    };
    let mechanical_ventilation_points = if input.mechanical_ventilation_on_index_culture_day {
        2
    } else {
        0
    };
    let cardiac_arrest_points = if input.cardiac_arrest_on_index_day_or_prior_48_hours {
        4
    } else {
        0
    };
    let score = temperature_points
        + hypotension_points
        + mechanical_ventilation_points
        + cardiac_arrest_points
        + mental_status_points;
    let score_at_least_four = score >= 4;
    let threshold_result = if score_at_least_four {
        "meets"
    } else {
        "does not meet"
    };
    let threshold_context = match input.assessment_context {
        AssessmentContext::HospitalisedPatientWithBloodstreamInfectionAndIndexCulture => {
            "the commonly used higher-risk threshold in bloodstream-infection studies"
        }
        AssessmentContext::HospitalisedPatientWithCreInfectionAndIndexCulture => {
            "the higher-risk threshold validated in the cited hospitalised CRE cohort"
        }
    };

    Ok(PittBacteraemiaOutcome {
        temperature_points,
        hypotension_points,
        mechanical_ventilation_points,
        cardiac_arrest_points,
        mental_status_points,
        score,
        score_at_least_four,
        interpretation: format!(
            "Pitt Bacteraemia Score {score}/14; this {threshold_result} the score >=4 threshold, {threshold_context}. {LIMITATIONS}"
        ),
    })
}

pub fn build_response(input: &PittBacteraemiaInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert("assessment_context".into(), json!(input.assessment_context));
    working.insert(
        "maximum_temperature_c".into(),
        json!(input.maximum_temperature_c),
    );
    working.insert(
        "temperature_points".into(),
        json!(outcome.temperature_points),
    );
    working.insert(
        "acute_hypotension_on_index_culture_day".into(),
        json!(input.acute_hypotension_on_index_culture_day),
    );
    working.insert(
        "hypotension_points".into(),
        json!(outcome.hypotension_points),
    );
    working.insert(
        "mechanical_ventilation_on_index_culture_day".into(),
        json!(input.mechanical_ventilation_on_index_culture_day),
    );
    working.insert(
        "mechanical_ventilation_points".into(),
        json!(outcome.mechanical_ventilation_points),
    );
    working.insert(
        "cardiac_arrest_on_index_day_or_prior_48_hours".into(),
        json!(input.cardiac_arrest_on_index_day_or_prior_48_hours),
    );
    working.insert(
        "cardiac_arrest_points".into(),
        json!(outcome.cardiac_arrest_points),
    );
    working.insert(
        "worst_mental_status_on_index_culture_day".into(),
        json!(input.worst_mental_status_on_index_culture_day),
    );
    working.insert(
        "mental_status_points".into(),
        json!(outcome.mental_status_points),
    );
    working.insert(
        "score_at_least_four".into(),
        json!(outcome.score_at_least_four),
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

pub struct PittBacteraemia;

impl Calculator for PittBacteraemia {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Pitt Bacteraemia Score"
    }

    fn description(&self) -> &'static str {
        "Acute severity score for hospitalised bloodstream infection or CRE infection, using index-culture-day physiology and cardiac arrest on that day or in the preceding 48 hours."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        let source = json!({
            "citation": "Henderson H et al. Clin Infect Dis. 2020;70(9):1826-1833.",
            "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC7156778/"
        });
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "PittBacteraemiaInput",
            "description": "Calculates the full 0-14 Pitt Bacteraemia Score for a hospitalised patient with bloodstream infection or CRE infection, using source-defined observation windows around an index culture. This is an acute severity stratifier, not a treatment or disposition rule.",
            "type": "object",
            "additionalProperties": false,
            "required": ["assessment_context", "maximum_temperature_c", "acute_hypotension_on_index_culture_day", "mechanical_ventilation_on_index_culture_day", "cardiac_arrest_on_index_day_or_prior_48_hours", "worst_mental_status_on_index_culture_day"],
            "properties": {
                "assessment_context": {
                    "type": "string", "enum": ["hospitalised_patient_with_bloodstream_infection_and_index_culture", "hospitalised_patient_with_cre_infection_and_index_culture"],
                    "description": "Hospitalised patient with a bloodstream infection or clinically identified carbapenem-resistant Enterobacteriaceae (CRE) infection and a defined index culture date.",
                    "definition": {
                        "concept": "Pitt Bacteraemia Score assessment context",
                        "statement": "Use for acute severity stratification in a hospitalised patient with bloodstream infection or clinically identified CRE infection and a defined index culture date.",
                        "excludes": ["Colonisation without clinical infection", "A non-bloodstream infection not caused by CRE", "Use without a defined index culture date", "Use as an antibiotic or ICU-admission rule"],
                        "caveats": "The score was established mainly in bloodstream-infection cohorts and later validated in a hospitalised CRE infection cohort dominated by Klebsiella pneumoniae. Generalisability to other pathogens and non-bloodstream infection cohorts is not established.",
                        "source": source, "status": "draft"
                    }
                },
                "maximum_temperature_c": {
                    "type": "number", "minimum": 20.0, "maximum": 45.0, "multipleOf": 0.1, "unit": "Cel",
                    "description": "Maximum temperature in degrees Celsius on the calendar day of index-culture collection, recorded to one decimal place.",
                    "definition": {
                        "concept": "Index-culture-day maximum temperature for Pitt Bacteraemia Score",
                        "statement": "Enter the maximum measured temperature on the calendar day the index culture was collected, in degrees Celsius to one decimal place.",
                        "excludes": ["Temperature from a different calendar day", "Fahrenheit without conversion", "A selected worst absolute deviation rather than the maximum temperature"],
                        "source": source, "status": "draft"
                    }
                },
                "acute_hypotension_on_index_culture_day": {
                    "type": "boolean",
                    "description": "Whether source-defined acute hypotension occurred on the index-culture day.",
                    "definition": {
                        "concept": "Acute hypotension for Pitt Bacteraemia Score",
                        "statement": "True when systolic pressure was below 90 mmHg, intravenous vasopressors were required, or there was an acute fall greater than 30 mmHg systolic and greater than 20 mmHg diastolic on the index-culture day.",
                        "excludes": ["A chronic low baseline without an acute qualifying event", "An isolated event outside the index-culture calendar day"],
                        "source": source, "status": "draft"
                    }
                },
                "mechanical_ventilation_on_index_culture_day": {
                    "type": "boolean",
                    "description": "Whether invasive mechanical ventilation was present on the index-culture day.",
                    "definition": {
                        "concept": "Mechanical ventilation for Pitt Bacteraemia Score",
                        "statement": "True when the patient received invasive mechanical ventilation on the calendar day of index-culture collection.",
                        "excludes": ["Non-invasive ventilation", "High-flow oxygen", "Ventilation outside the index-culture calendar day"],
                        "source": source, "status": "draft"
                    }
                },
                "cardiac_arrest_on_index_day_or_prior_48_hours": {
                    "type": "boolean",
                    "description": "Whether cardiac arrest occurred on the index day or during the preceding 48 hours.",
                    "definition": {
                        "concept": "Recent cardiac arrest for Pitt Bacteraemia Score",
                        "statement": "True when cardiac arrest occurred on the index-culture day or within the 48 hours before index-culture collection.",
                        "excludes": ["Cardiac arrest more than 48 hours before index-culture collection"],
                        "source": source, "status": "draft"
                    }
                },
                "worst_mental_status_on_index_culture_day": {
                    "type": "string", "enum": ["normal", "disoriented", "stuporous", "comatose", "not_assessable"],
                    "description": "Worst clinically assessable mental status on the index-culture day. Select not_assessable when sedation or another barrier prevents valid classification; calculation will stop rather than treating this as coma.",
                    "definition": {
                        "concept": "Index-culture-day mental status for Pitt Bacteraemia Score",
                        "statement": "Select the worst clinically assessable mental-status category on the index-culture calendar day.",
                        "excludes": ["Iatrogenic sedation classified as clinical coma", "A status from a different calendar day", "Assuming normal when assessment was not possible"],
                        "caveats": "Use not_assessable when a valid clinical category cannot be assigned; the score will not be calculated.",
                        "source": source, "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: PittBacteraemiaInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(temperature: f64) -> PittBacteraemiaInput {
        PittBacteraemiaInput {
            assessment_context:
                AssessmentContext::HospitalisedPatientWithBloodstreamInfectionAndIndexCulture,
            maximum_temperature_c: temperature,
            acute_hypotension_on_index_culture_day: false,
            mechanical_ventilation_on_index_culture_day: false,
            cardiac_arrest_on_index_day_or_prior_48_hours: false,
            worst_mental_status_on_index_culture_day: MentalStatus::Normal,
        }
    }

    #[test]
    fn source_temperature_boundaries_are_exact() {
        for (temperature, expected) in [
            (35.0, 2),
            (35.1, 1),
            (36.0, 1),
            (36.1, 0),
            (38.9, 0),
            (39.0, 1),
            (39.9, 1),
            (40.0, 2),
        ] {
            assert_eq!(
                compute(&input(temperature)).unwrap().temperature_points,
                expected
            );
        }
    }

    #[test]
    fn source_component_weights_and_maximum_are_exact() {
        let maximum = PittBacteraemiaInput {
            maximum_temperature_c: 40.0,
            acute_hypotension_on_index_culture_day: true,
            mechanical_ventilation_on_index_culture_day: true,
            cardiac_arrest_on_index_day_or_prior_48_hours: true,
            worst_mental_status_on_index_culture_day: MentalStatus::Comatose,
            ..input(37.0)
        };
        let outcome = compute(&maximum).unwrap();
        assert_eq!(outcome.score, 14);
        assert_eq!(outcome.hypotension_points, 2);
        assert_eq!(outcome.mechanical_ventilation_points, 2);
        assert_eq!(outcome.cardiac_arrest_points, 4);
        assert_eq!(outcome.mental_status_points, 4);
    }

    #[test]
    fn mental_status_weights_are_exact() {
        for (status, expected) in [
            (MentalStatus::Normal, 0),
            (MentalStatus::Disoriented, 1),
            (MentalStatus::Stuporous, 2),
            (MentalStatus::Comatose, 4),
        ] {
            let mut value = input(37.0);
            value.worst_mental_status_on_index_culture_day = status;
            assert_eq!(compute(&value).unwrap().mental_status_points, expected);
        }
    }

    #[test]
    fn validated_threshold_is_inclusive_four() {
        let below = PittBacteraemiaInput {
            acute_hypotension_on_index_culture_day: true,
            worst_mental_status_on_index_culture_day: MentalStatus::Disoriented,
            ..input(37.0)
        };
        let at = PittBacteraemiaInput {
            worst_mental_status_on_index_culture_day: MentalStatus::Comatose,
            ..input(37.0)
        };
        assert_eq!(compute(&below).unwrap().score, 3);
        assert!(!compute(&below).unwrap().score_at_least_four);
        assert_eq!(compute(&at).unwrap().score, 4);
        assert!(compute(&at).unwrap().score_at_least_four);
    }

    #[test]
    fn rejects_unassessable_status_nonfinite_implausible_and_finer_temperature() {
        let mut unassessable = input(37.0);
        unassessable.worst_mental_status_on_index_culture_day = MentalStatus::NotAssessable;
        assert!(compute(&unassessable).is_err());
        assert!(compute(&input(f64::NAN)).is_err());
        assert!(compute(&input(102.0)).is_err());
        assert!(compute(&input(19.9)).is_err());
        assert!(compute(&input(36.05)).is_err());
    }

    #[test]
    fn dynamic_surface_is_closed_and_matches_typed_response() {
        let value = serde_json::to_value(input(36.0)).unwrap();
        assert_eq!(
            PittBacteraemia.calculate(&value).unwrap(),
            build_response(&input(36.0)).unwrap()
        );
        let mut unknown = value;
        unknown["respiratory_rate"] = json!(28);
        assert!(PittBacteraemia.calculate(&unknown).is_err());
    }

    #[test]
    fn response_is_risk_stratification_not_a_treatment_rule() {
        let response = build_response(&input(40.0)).unwrap();
        assert_eq!(response.result, json!(2));
        assert_eq!(response.working["score_at_least_four"], json!(false));
        assert!(response.interpretation.contains("not a diagnosis"));
        assert!(response.interpretation.contains("not"));
        assert!(!response.interpretation.contains("start antibiotics"));
    }

    #[test]
    fn schema_is_closed_and_defines_observation_windows() {
        let schema = PittBacteraemia.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["required"].as_array().unwrap().len(), 6);
        assert_eq!(
            schema["properties"]["maximum_temperature_c"]["multipleOf"],
            json!(0.1)
        );
        assert_eq!(
            schema["properties"]["maximum_temperature_c"]["maximum"],
            json!(45.0)
        );
        assert!(
            schema["properties"]["cardiac_arrest_on_index_day_or_prior_48_hours"]
                ["definition"]["statement"]
                .as_str()
                .unwrap()
                .contains("48 hours")
        );
    }
}
