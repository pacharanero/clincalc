// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! New York Heart Association functional classification.
//!
//! The functional-class definitions are adapted from NCI Thesaurus concepts
//! distributed under CC BY 4.0. NCI is credited in the calculator licence and
//! in every response. The adapter code is AGPL-3.0-or-later and adds no
//! treatment recommendation.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "nyha";
pub const REFERENCE: &str = "The Criteria Committee of the New York Heart Association. Nomenclature and Criteria for Diagnosis of Diseases of the Heart and Blood Vessels. 5th ed. New York Heart Association; 1953. Current licensed definitions: NCI Thesaurus concepts C1882084-C1882087, adapted from the 9th ed. (1994:253-256). Limitations: Raphael C, Briscoe C, Davies J, et al. Heart. 2007;93(4):476-482. doi:10.1136/hrt.2006.089656. PMID:17005715.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "CC-BY-4.0 - functional-class definitions adapted from NCI Thesaurus concepts C1882084-C1882087 with attribution",
    source_url: "https://evs.nci.nih.gov/ftp1/NCI_Thesaurus/ThesaurusTermsofUse.htm",
};

const LIMITATIONS: &str = "NYHA class is a subjective functional classification, not an objective exercise-capacity measurement, diagnosis, prognosis, or treatment rule. The terms ordinary activity, slight limitation, and marked limitation require clinical judgement. Raphael et al. found only 54% agreement when two cardiologists independently classified the same 50 patients, principally across Classes II and III. Record the symptoms and activities used to assign the class, consider non-cardiac causes of limitation, reassess when clinical status changes, and do not infer a medication, device, referral, admission, or discharge decision from this class alone.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentContext {
    DefinedOrPresumedCardiacDisease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionalProfile {
    NoLimitationWithOrdinaryActivity,
    ComfortableAtRestSymptomsWithOrdinaryActivity,
    ComfortableAtRestSymptomsWithLessThanOrdinaryActivity,
    SymptomsAtRestOrMinimalExertionAndAnyPhysicalActivityCausesDiscomfort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NyhaClass {
    ClassI,
    ClassIi,
    ClassIii,
    ClassIv,
}

impl NyhaClass {
    fn from_profile(profile: FunctionalProfile) -> Self {
        match profile {
            FunctionalProfile::NoLimitationWithOrdinaryActivity => Self::ClassI,
            FunctionalProfile::ComfortableAtRestSymptomsWithOrdinaryActivity => Self::ClassIi,
            FunctionalProfile::ComfortableAtRestSymptomsWithLessThanOrdinaryActivity => {
                Self::ClassIii
            }
            FunctionalProfile::SymptomsAtRestOrMinimalExertionAndAnyPhysicalActivityCausesDiscomfort => Self::ClassIv,
        }
    }

    pub fn ordinal(self) -> u8 {
        match self {
            Self::ClassI => 1,
            Self::ClassIi => 2,
            Self::ClassIii => 3,
            Self::ClassIv => 4,
        }
    }

    pub fn roman(self) -> &'static str {
        match self {
            Self::ClassI => "I",
            Self::ClassIi => "II",
            Self::ClassIii => "III",
            Self::ClassIv => "IV",
        }
    }

    fn summary(self) -> &'static str {
        match self {
            Self::ClassI => {
                "no resulting physical-activity limitation; ordinary activity does not provoke cardiac symptoms"
            }
            Self::ClassIi => {
                "comfortable at rest, with slight limitation because ordinary activity provokes cardiac symptoms"
            }
            Self::ClassIii => {
                "comfortable at rest, with marked limitation because less-than-ordinary activity provokes cardiac symptoms"
            }
            Self::ClassIv => {
                "cardiac symptoms at rest or minimal exertion, with inability to undertake physical activity without discomfort"
            }
        }
    }

    fn nci_concept_id(self) -> &'static str {
        match self {
            Self::ClassI => "C1882084",
            Self::ClassIi => "C1882085",
            Self::ClassIii => "C1882086",
            Self::ClassIv => "C1882087",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NyhaInput {
    pub assessment_context: AssessmentContext,
    pub functional_profile: FunctionalProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NyhaOutcome {
    pub class: NyhaClass,
    pub ordinal: u8,
    pub interpretation: &'static str,
}

pub fn compute(input: &NyhaInput) -> Result<NyhaOutcome, CalcError> {
    let class = NyhaClass::from_profile(input.functional_profile);
    Ok(NyhaOutcome {
        class,
        ordinal: class.ordinal(),
        interpretation: class.summary(),
    })
}

pub fn build_response(input: &NyhaInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert("assessment_context".into(), json!(input.assessment_context));
    working.insert("functional_profile".into(), json!(input.functional_profile));
    working.insert("nyha_class".into(), json!(outcome.class.roman()));
    working.insert("ordinal".into(), json!(outcome.ordinal));
    working.insert(
        "nci_thesaurus_concept_id".into(),
        json!(outcome.class.nci_concept_id()),
    );
    working.insert(
        "classification_content_attribution".into(),
        json!("NCI Thesaurus, National Cancer Institute; adapted under CC BY 4.0"),
    );
    working.insert(
        "classification_content_license_url".into(),
        json!(LICENSE.source_url),
    );
    working.insert("limitations".into(), json!(LIMITATIONS));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.ordinal),
        interpretation: format!(
            "NYHA Class {}: {}. {LIMITATIONS}",
            outcome.class.roman(),
            outcome.interpretation
        ),
        working,
        reference: REFERENCE.to_string(),
    })
}

fn input_schema() -> Value {
    let source = json!({
        "citation": "Primary classification: The Criteria Committee of the New York Heart Association, 5th ed. (1953). Licensed current definitions: NCI Thesaurus concepts C1882084-C1882087, adapted from the 9th ed. (1994)",
        "url": "https://archive.org/details/in.ernet.dli.2015.547950",
        "classDefinitionUrls": [
            "https://www.ncbi.nlm.nih.gov/medgen/406942",
            "https://www.ncbi.nlm.nih.gov/medgen/364512",
            "https://www.ncbi.nlm.nih.gov/medgen/362153",
            "https://www.ncbi.nlm.nih.gov/medgen/362154"
        ],
        "license": "CC-BY-4.0",
        "licenseUrl": "https://evs.nci.nih.gov/ftp1/NCI_Thesaurus/ThesaurusTermsofUse.htm"
    });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "NyhaInput",
        "description": "Clinician-facing aid for assigning the current NYHA functional class in a patient with defined or presumed cardiac disease. Select the single profile that best represents cardiac-symptom-related physical-activity limitation. This subjective classification is not a diagnosis, objective exercise test, prognosis, or treatment rule.",
        "type": "object",
        "additionalProperties": false,
        "required": ["assessment_context", "functional_profile"],
        "properties": {
            "assessment_context": {
                "type": "string",
                "const": "defined_or_presumed_cardiac_disease",
                "description": "Attestation that the patient has defined or presumed cardiac disease and NYHA functional classification is clinically applicable",
                "definition": {
                    "concept": "NYHA assessment context",
                    "statement": "Use the functional classification only in a patient with defined or presumed cardiac disease.",
                    "includes": ["Defined cardiac disease", "Presumed cardiac disease under clinical assessment"],
                    "excludes": ["Using NYHA to diagnose cardiac disease", "Attributing a non-cardiac activity limitation to cardiac disease without clinical assessment"],
                    "source": source,
                    "snomedEcl": null,
                    "refset": null,
                    "caveats": "NYHA class describes functional limitation attributed to cardiac disease; it does not establish the diagnosis or cause of symptoms.",
                    "status": "draft"
                }
            },
            "functional_profile": {
                "type": "string",
                "enum": [
                    "no_limitation_with_ordinary_activity",
                    "comfortable_at_rest_symptoms_with_ordinary_activity",
                    "comfortable_at_rest_symptoms_with_less_than_ordinary_activity",
                    "symptoms_at_rest_or_minimal_exertion_and_any_physical_activity_causes_discomfort"
                ],
                "description": "Select one current profile. Cardiac symptoms include undue fatigue, breathlessness, palpitations, or chest discomfort: no_limitation_with_ordinary_activity = Class I; comfortable_at_rest_symptoms_with_ordinary_activity = Class II; comfortable_at_rest_symptoms_with_less_than_ordinary_activity = Class III; symptoms_at_rest_or_minimal_exertion_and_any_physical_activity_causes_discomfort = Class IV.",
                "definition": {
                    "concept": "NYHA functional profile",
                    "statement": "The clinician selects the mutually exclusive profile that best represents current physical-activity limitation attributable to cardiac symptoms.",
                    "includes": [
                        "Class I: ordinary activity does not provoke cardiac symptoms or limitation",
                        "Class II: comfortable at rest; ordinary activity provokes cardiac symptoms",
                        "Class III: comfortable at rest; less-than-ordinary activity provokes cardiac symptoms",
                        "Class IV: symptoms occur at rest or minimal exertion and physical activity cannot be undertaken without discomfort"
                    ],
                    "excludes": ["Selecting from walking distance alone", "Treating unclear or missing symptom history as Class I", "Using class alone to direct treatment or disposition"],
                    "source": source,
                    "snomedEcl": null,
                    "refset": null,
                    "caveats": "The boundaries, especially between Classes II and III, are subjective and poorly reproducible. Record the symptoms and activities used and reassess when status changes.",
                    "status": "draft"
                }
            }
        }
    })
}

pub struct Nyha;

impl Calculator for Nyha {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "NYHA Functional Classification"
    }

    fn description(&self) -> &'static str {
        "Classifies current cardiac-symptom-related physical-activity limitation as NYHA Class I-IV without making treatment recommendations."
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
        let parsed: NyhaInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(functional_profile: FunctionalProfile) -> NyhaInput {
        NyhaInput {
            assessment_context: AssessmentContext::DefinedOrPresumedCardiacDisease,
            functional_profile,
        }
    }

    #[test]
    fn primary_and_licensed_source_profiles_map_to_the_four_classes() {
        let vectors = [
            (
                FunctionalProfile::NoLimitationWithOrdinaryActivity,
                NyhaClass::ClassI,
                1,
                "C1882084",
            ),
            (
                FunctionalProfile::ComfortableAtRestSymptomsWithOrdinaryActivity,
                NyhaClass::ClassIi,
                2,
                "C1882085",
            ),
            (
                FunctionalProfile::ComfortableAtRestSymptomsWithLessThanOrdinaryActivity,
                NyhaClass::ClassIii,
                3,
                "C1882086",
            ),
            (
                FunctionalProfile::SymptomsAtRestOrMinimalExertionAndAnyPhysicalActivityCausesDiscomfort,
                NyhaClass::ClassIv,
                4,
                "C1882087",
            ),
        ];

        for (profile, expected_class, expected_ordinal, concept_id) in vectors {
            let outcome = compute(&input(profile)).unwrap();
            assert_eq!(outcome.class, expected_class);
            assert_eq!(outcome.ordinal, expected_ordinal);
            assert_eq!(outcome.class.nci_concept_id(), concept_id);
        }
    }

    #[test]
    fn dynamic_surface_returns_ordinal_and_roman_class() {
        let value = json!({
            "assessment_context": "defined_or_presumed_cardiac_disease",
            "functional_profile": "comfortable_at_rest_symptoms_with_less_than_ordinary_activity"
        });
        let response = Nyha.calculate(&value).unwrap();

        assert_eq!(response.result, json!(3));
        assert_eq!(response.working["nyha_class"], json!("III"));
        assert_eq!(
            response.working["nci_thesaurus_concept_id"],
            json!("C1882086")
        );
        assert_eq!(
            response.working["classification_content_license_url"],
            json!(LICENSE.source_url)
        );
    }

    #[test]
    fn response_preserves_subjectivity_and_no_treatment_rule() {
        let response = build_response(&input(
            FunctionalProfile::SymptomsAtRestOrMinimalExertionAndAnyPhysicalActivityCausesDiscomfort,
        ))
        .unwrap();

        assert!(response.interpretation.contains("subjective"));
        assert!(response.interpretation.contains("54% agreement"));
        assert!(response.interpretation.contains("not an objective"));
        assert!(response.interpretation.contains("treatment rule"));
        assert!(!response.interpretation.contains("refer for"));
        assert!(!response.interpretation.contains("therapy"));
    }

    #[test]
    fn schema_is_closed_required_and_defines_both_inputs() {
        let schema = input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(
            schema["required"],
            json!(["assessment_context", "functional_profile"])
        );
        assert_eq!(
            schema["properties"]["functional_profile"]["enum"],
            json!([
                "no_limitation_with_ordinary_activity",
                "comfortable_at_rest_symptoms_with_ordinary_activity",
                "comfortable_at_rest_symptoms_with_less_than_ordinary_activity",
                "symptoms_at_rest_or_minimal_exertion_and_any_physical_activity_causes_discomfort"
            ])
        );
        assert_eq!(
            schema["properties"]["functional_profile"]["definition"]["source"]["classDefinitionUrls"]
                [3],
            json!("https://www.ncbi.nlm.nih.gov/medgen/362154")
        );
        for field in ["assessment_context", "functional_profile"] {
            assert!(schema["properties"][field]["definition"].is_object());
        }
    }

    #[test]
    fn dynamic_surface_rejects_missing_unknown_and_noncanonical_inputs() {
        for invalid in [
            json!({}),
            json!({
                "assessment_context": "defined_or_presumed_cardiac_disease",
                "functional_profile": "class_iii"
            }),
            json!({
                "assessment_context": "no_known_cardiac_disease",
                "functional_profile": "no_limitation_with_ordinary_activity"
            }),
            json!({
                "assessment_context": "defined_or_presumed_cardiac_disease",
                "functional_profile": "no_limitation_with_ordinary_activity",
                "unexpected": true
            }),
        ] {
            assert!(Nyha.calculate(&invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn licence_records_nci_attribution_and_terms() {
        assert!(LICENSE.license.contains("CC-BY-4.0"));
        assert!(LICENSE.license.contains("NCI Thesaurus"));
        assert_eq!(
            LICENSE.source_url,
            "https://evs.nci.nih.gov/ftp1/NCI_Thesaurus/ThesaurusTermsofUse.htm"
        );
    }
}
