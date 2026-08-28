// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Wasvary/Kaiser modified Hinchey classification for diverticulitis.
//!
//! This implementation keeps the named classification variants separate. It
//! uses the Wasvary stages as operationalised for CT by Kaiser and reports
//! Kaiser's fistula and obstruction categories without pretending they are
//! numbered Hinchey stages. Purulent stage III and fecal stage IV require
//! operative evidence because CT findings alone do not distinguish them.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "hinchey";

/// Primary classification lineage and CT operationalisation.
pub const REFERENCE: &str = "Hinchey EJ, Schaal PG, Richards GK. Treatment of perforated diverticular disease of the colon. Adv Surg. 1978;12:85-109. PMID:735943. Wasvary H, Turfah F, Kadro O, et al. Same hospitalization resection for acute diverticulitis. Am Surg. 1999;65(7):632-635. PMID:10399971. Kaiser AM, Jiang JK, Lake JP, et al. The management of complicated diverticulitis and the role of computed tomography. Am J Gastroenterol. 2005;100(4):910-917. doi:10.1111/j.1572-0241.2005.41154.x.";

/// Distribution licence: independently implemented from published methods.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published clinical classification - independently implemented from the primary literature",
    source_url: "https://doi.org/10.1111/j.1572-0241.2005.41154.x",
};

/// Evidence used to select the anatomical category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentBasis {
    Clinical,
    Ct,
    Operative,
    CtAndOperative,
}

impl AssessmentBasis {
    fn slug(self) -> &'static str {
        match self {
            Self::Clinical => "clinical",
            Self::Ct => "ct",
            Self::Operative => "operative",
            Self::CtAndOperative => "ct_and_operative",
        }
    }

    fn includes_operative_evidence(self) -> bool {
        matches!(self, Self::Operative | Self::CtAndOperative)
    }
}

/// Mutually exclusive presentation from the Wasvary/Kaiser classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiverticulitisFinding {
    MildClinicalDiverticulitis,
    ConfinedPericolicInflammationOrPhlegmon,
    ConfinedPericolicOrMesocolicAbscess,
    PelvicDistantIntraAbdominalOrRetroperitonealAbscess,
    GeneralizedPurulentPeritonitis,
    GeneralizedFecalPeritonitis,
    Fistula,
    Obstruction,
    GeneralizedPeritonitisContaminationNotEstablished,
}

impl DiverticulitisFinding {
    fn result(self) -> &'static str {
        match self {
            Self::MildClinicalDiverticulitis => "0",
            Self::ConfinedPericolicInflammationOrPhlegmon => "Ia",
            Self::ConfinedPericolicOrMesocolicAbscess => "Ib",
            Self::PelvicDistantIntraAbdominalOrRetroperitonealAbscess => "II",
            Self::GeneralizedPurulentPeritonitis => "III",
            Self::GeneralizedFecalPeritonitis => "IV",
            Self::Fistula => "fistula",
            Self::Obstruction => "obstruction",
            Self::GeneralizedPeritonitisContaminationNotEstablished => "indeterminate_iii_or_iv",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::MildClinicalDiverticulitis => "mild clinical diverticulitis",
            Self::ConfinedPericolicInflammationOrPhlegmon => {
                "confined pericolic inflammation or phlegmon"
            }
            Self::ConfinedPericolicOrMesocolicAbscess => "confined pericolic or mesocolic abscess",
            Self::PelvicDistantIntraAbdominalOrRetroperitonealAbscess => {
                "pelvic, distant intra-abdominal, or retroperitoneal abscess"
            }
            Self::GeneralizedPurulentPeritonitis => "generalized purulent peritonitis",
            Self::GeneralizedFecalPeritonitis => "generalized fecal peritonitis",
            Self::Fistula => "diverticular fistula",
            Self::Obstruction => "obstruction attributable to diverticular disease",
            Self::GeneralizedPeritonitisContaminationNotEstablished => {
                "generalized peritonitis with contamination not established"
            }
        }
    }

    fn category_kind(self) -> &'static str {
        match self {
            Self::MildClinicalDiverticulitis
            | Self::ConfinedPericolicInflammationOrPhlegmon
            | Self::ConfinedPericolicOrMesocolicAbscess
            | Self::PelvicDistantIntraAbdominalOrRetroperitonealAbscess
            | Self::GeneralizedPurulentPeritonitis
            | Self::GeneralizedFecalPeritonitis => "modified_hinchey_stage",
            Self::Fistula | Self::Obstruction => "kaiser_additional_category",
            Self::GeneralizedPeritonitisContaminationNotEstablished => "safety_indeterminate",
        }
    }

    fn interpretation(self) -> String {
        match self {
            Self::MildClinicalDiverticulitis
            | Self::ConfinedPericolicInflammationOrPhlegmon
            | Self::ConfinedPericolicOrMesocolicAbscess
            | Self::PelvicDistantIntraAbdominalOrRetroperitonealAbscess
            | Self::GeneralizedPurulentPeritonitis
            | Self::GeneralizedFecalPeritonitis => format!(
                "Modified Hinchey stage {}: {}. This is an anatomical classification, not a stand-alone treatment recommendation; integrate clinical condition, imaging, operative findings, comorbidity, and current local guidance.",
                self.result(),
                self.label()
            ),
            Self::Fistula | Self::Obstruction => format!(
                "Kaiser additional category: {}. Kaiser reported this separately from numbered modified Hinchey stages; do not relabel it as stage II. This classification does not prescribe treatment.",
                self.label()
            ),
            Self::GeneralizedPeritonitisContaminationNotEstablished => {
                "Generalized peritonitis is present, but purulent versus fecal contamination has not been established. Do not infer modified Hinchey stage III or IV from CT free gas or fluid alone; definitive staging requires operative findings. Continue urgent surgical assessment according to clinical status and local guidance.".to_string()
            }
        }
    }
}

/// Inputs for the modified Hinchey classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HincheyInput {
    /// Confirm diverticular disease is the established cause of the finding.
    pub diverticular_disease_is_cause: bool,
    pub assessment_basis: AssessmentBasis,
    pub finding: DiverticulitisFinding,
}

/// Computed classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HincheyOutcome {
    pub result: &'static str,
    pub label: &'static str,
    pub category_kind: &'static str,
    pub interpretation: String,
}

/// Pure classification.
pub fn compute(input: &HincheyInput) -> Result<HincheyOutcome, CalcError> {
    if !input.diverticular_disease_is_cause {
        return Err(CalcError::InvalidInput(
            "the classification requires diverticular disease to be the established cause; exclude mimics such as colorectal cancer and non-diverticular perforation"
                .into(),
        ));
    }

    match input.finding {
        DiverticulitisFinding::MildClinicalDiverticulitis
            if input.assessment_basis != AssessmentBasis::Clinical =>
        {
            return Err(CalcError::InvalidInput(
                "stage 0 is a purely clinical diagnosis without imaging or operative confirmation"
                    .into(),
            ));
        }
        DiverticulitisFinding::ConfinedPericolicInflammationOrPhlegmon
        | DiverticulitisFinding::ConfinedPericolicOrMesocolicAbscess
        | DiverticulitisFinding::PelvicDistantIntraAbdominalOrRetroperitonealAbscess
            if input.assessment_basis == AssessmentBasis::Clinical =>
        {
            return Err(CalcError::InvalidInput(
                "stages Ia, Ib, and II require imaging or operative anatomical evidence".into(),
            ));
        }
        DiverticulitisFinding::GeneralizedPurulentPeritonitis
        | DiverticulitisFinding::GeneralizedFecalPeritonitis
            if !input.assessment_basis.includes_operative_evidence() =>
        {
            return Err(CalcError::InvalidInput(
                "formal stage III or IV requires operative evidence of purulent or fecal contamination; use generalized_peritonitis_contamination_not_established when CT or clinical assessment cannot distinguish them"
                    .into(),
            ));
        }
        DiverticulitisFinding::GeneralizedPeritonitisContaminationNotEstablished
            if input.assessment_basis.includes_operative_evidence() =>
        {
            return Err(CalcError::InvalidInput(
                "indeterminate stage III or IV is reserved for non-operative assessment; use the observed contamination to select stage III or IV when operative evidence is available"
                    .into(),
            ));
        }
        _ => {}
    }

    Ok(HincheyOutcome {
        result: input.finding.result(),
        label: input.finding.label(),
        category_kind: input.finding.category_kind(),
        interpretation: input.finding.interpretation(),
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &HincheyInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "diverticular_disease_is_cause".into(),
        json!(input.diverticular_disease_is_cause),
    );
    working.insert(
        "assessment_basis".into(),
        json!(input.assessment_basis.slug()),
    );
    working.insert("category_kind".into(), json!(outcome.category_kind));
    working.insert("anatomical_finding".into(), json!(outcome.label));
    working.insert(
        "classification_system".into(),
        json!(
            "Wasvary modified Hinchey classification as operationalised and extended by Kaiser 2005"
        ),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.result),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn input_schema() -> Value {
    let source = json!({
        "citation": "Kaiser AM, Jiang JK, Lake JP, et al. Am J Gastroenterol. 2005;100(4):910-917.",
        "url": "https://doi.org/10.1111/j.1572-0241.2005.41154.x"
    });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "HincheyInput",
        "type": "object",
        "additionalProperties": false,
        "required": ["diverticular_disease_is_cause", "assessment_basis", "finding"],
        "allOf": [
            {
                "if": { "properties": { "finding": { "const": "mild_clinical_diverticulitis" } }, "required": ["finding"] },
                "then": { "properties": { "assessment_basis": { "const": "clinical" } } }
            },
            {
                "if": {
                    "properties": {
                        "finding": {
                            "enum": [
                                "confined_pericolic_inflammation_or_phlegmon",
                                "confined_pericolic_or_mesocolic_abscess",
                                "pelvic_distant_intra_abdominal_or_retroperitoneal_abscess"
                            ]
                        }
                    },
                    "required": ["finding"]
                },
                "then": { "properties": { "assessment_basis": { "enum": ["ct", "operative", "ct_and_operative"] } } }
            },
            {
                "if": {
                    "properties": {
                        "finding": { "enum": ["generalized_purulent_peritonitis", "generalized_fecal_peritonitis"] }
                    },
                    "required": ["finding"]
                },
                "then": { "properties": { "assessment_basis": { "enum": ["operative", "ct_and_operative"] } } }
            },
            {
                "if": { "properties": { "finding": { "const": "generalized_peritonitis_contamination_not_established" } }, "required": ["finding"] },
                "then": { "properties": { "assessment_basis": { "enum": ["clinical", "ct"] } } }
            }
        ],
        "properties": {
            "diverticular_disease_is_cause": {
                "type": "boolean",
                "description": "Confirm diverticular disease is the established cause of the current finding; required but not itself a stage",
                "definition": {
                    "concept": "Diverticular disease as cause",
                    "statement": "The current inflammation, abscess, peritonitis, fistula, or obstruction is attributable to colonic diverticular disease.",
                    "includes": ["Acute colonic diverticulitis", "Perforation, abscess, fistula, or obstruction established as a complication of diverticular disease"],
                    "excludes": ["Asymptomatic diverticulosis", "Colorectal cancer mimicking diverticulitis", "Inflammation, perforation, fistula, or obstruction from another cause"],
                    "caveats": "The classification does not establish the diagnosis or exclude malignancy.",
                    "source": source,
                    "status": "draft"
                }
            },
            "assessment_basis": {
                "type": "string",
                "enum": ["clinical", "ct", "operative", "ct_and_operative"],
                "description": "Evidence used for classification. Formal stages III and IV require operative evidence because CT alone cannot establish purulent versus fecal contamination",
                "definition": {
                    "concept": "Assessment basis",
                    "statement": "Record whether the selected category is based on clinical assessment, CT, operative findings, or both CT and operative findings.",
                    "excludes": ["Do not infer stage III versus IV from the amount of CT free gas or fluid"],
                    "caveats": "Kaiser reports the same CT pattern for stages III and IV; operative contamination distinguishes them.",
                    "source": source,
                    "status": "draft"
                }
            },
            "finding": {
                "type": "string",
                "enum": [
                    "mild_clinical_diverticulitis",
                    "confined_pericolic_inflammation_or_phlegmon",
                    "confined_pericolic_or_mesocolic_abscess",
                    "pelvic_distant_intra_abdominal_or_retroperitoneal_abscess",
                    "generalized_purulent_peritonitis",
                    "generalized_fecal_peritonitis",
                    "fistula",
                    "obstruction",
                    "generalized_peritonitis_contamination_not_established"
                ],
                "description": "Single best anatomical category. Abscess location, not an unsupported size threshold, distinguishes confined stage Ib from distant stage II",
                "definition": {
                    "concept": "Wasvary/Kaiser modified Hinchey finding",
                    "statement": "Select the single category best supported by the current clinical, CT, or operative findings.",
                    "includes": [
                        "Stage 0: mild clinical diverticulitis",
                        "Stage Ia: confined pericolic inflammation or phlegmon",
                        "Stage Ib: confined pericolic or mesocolic abscess",
                        "Stage II: pelvic, distant intra-abdominal, or retroperitoneal abscess",
                        "Stage III: generalized purulent peritonitis without open bowel communication",
                        "Stage IV: fecal peritonitis from free perforation or open bowel communication",
                        "Kaiser additional categories: fistula or obstruction"
                    ],
                    "excludes": [
                        "Do not use abscess diameter to distinguish stage Ib from II",
                        "Do not classify free gas or free fluid alone as stage III or IV",
                        "Fistula and obstruction are Kaiser additional categories, not numbered stages"
                    ],
                    "caveats": "Use generalized_peritonitis_contamination_not_established rather than guessing III or IV when contamination has not been directly established.",
                    "source": source,
                    "status": "draft"
                }
            }
        }
    })
}

/// Dynamic calculator implementation.
pub struct Hinchey;

impl Calculator for Hinchey {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Modified Hinchey Classification (Wasvary/Kaiser)"
    }

    fn description(&self) -> &'static str {
        "Classifies diverticulitis anatomy using the Wasvary modified Hinchey stages and Kaiser 2005 CT operationalisation, without inferring purulent versus fecal peritonitis from CT alone."
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
        let parsed: HincheyInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(finding: DiverticulitisFinding, assessment_basis: AssessmentBasis) -> HincheyInput {
        HincheyInput {
            diverticular_disease_is_cause: true,
            assessment_basis,
            finding,
        }
    }

    #[test]
    fn wasvary_kaiser_stages_map_exactly() {
        for (finding, basis, expected) in [
            (
                DiverticulitisFinding::MildClinicalDiverticulitis,
                AssessmentBasis::Clinical,
                "0",
            ),
            (
                DiverticulitisFinding::ConfinedPericolicInflammationOrPhlegmon,
                AssessmentBasis::Ct,
                "Ia",
            ),
            (
                DiverticulitisFinding::ConfinedPericolicOrMesocolicAbscess,
                AssessmentBasis::Ct,
                "Ib",
            ),
            (
                DiverticulitisFinding::PelvicDistantIntraAbdominalOrRetroperitonealAbscess,
                AssessmentBasis::Ct,
                "II",
            ),
            (
                DiverticulitisFinding::GeneralizedPurulentPeritonitis,
                AssessmentBasis::Operative,
                "III",
            ),
            (
                DiverticulitisFinding::GeneralizedFecalPeritonitis,
                AssessmentBasis::CtAndOperative,
                "IV",
            ),
        ] {
            assert_eq!(compute(&input(finding, basis)).unwrap().result, expected);
        }
    }

    #[test]
    fn kaiser_fistula_and_obstruction_are_not_numbered_stages() {
        for finding in [
            DiverticulitisFinding::Fistula,
            DiverticulitisFinding::Obstruction,
        ] {
            let outcome = compute(&input(finding, AssessmentBasis::Ct)).unwrap();
            assert_eq!(outcome.category_kind, "kaiser_additional_category");
            assert!(
                outcome
                    .interpretation
                    .contains("not relabel it as stage II")
            );
        }
    }

    #[test]
    fn ct_only_generalized_peritonitis_remains_indeterminate() {
        let outcome = compute(&input(
            DiverticulitisFinding::GeneralizedPeritonitisContaminationNotEstablished,
            AssessmentBasis::Ct,
        ))
        .unwrap();
        assert_eq!(outcome.result, "indeterminate_iii_or_iv");
        assert!(outcome.interpretation.contains("Do not infer"));
    }

    #[test]
    fn rejects_ct_only_stage_three_or_four() {
        for finding in [
            DiverticulitisFinding::GeneralizedPurulentPeritonitis,
            DiverticulitisFinding::GeneralizedFecalPeritonitis,
        ] {
            let error = compute(&input(finding, AssessmentBasis::Ct)).unwrap_err();
            assert!(error.to_string().contains("operative evidence"));
        }
    }

    #[test]
    fn rejects_unconfirmed_diverticular_cause() {
        let mut value = input(
            DiverticulitisFinding::ConfinedPericolicInflammationOrPhlegmon,
            AssessmentBasis::Ct,
        );
        value.diverticular_disease_is_cause = false;
        assert!(compute(&value).is_err());
    }

    #[test]
    fn assessment_basis_must_match_the_selected_category() {
        for basis in [
            AssessmentBasis::Ct,
            AssessmentBasis::Operative,
            AssessmentBasis::CtAndOperative,
        ] {
            assert!(
                compute(&input(
                    DiverticulitisFinding::MildClinicalDiverticulitis,
                    basis,
                ))
                .is_err()
            );
        }

        for finding in [
            DiverticulitisFinding::ConfinedPericolicInflammationOrPhlegmon,
            DiverticulitisFinding::ConfinedPericolicOrMesocolicAbscess,
            DiverticulitisFinding::PelvicDistantIntraAbdominalOrRetroperitonealAbscess,
        ] {
            assert!(compute(&input(finding, AssessmentBasis::Clinical)).is_err());
        }

        for basis in [AssessmentBasis::Operative, AssessmentBasis::CtAndOperative] {
            assert!(
                compute(&input(
                    DiverticulitisFinding::GeneralizedPeritonitisContaminationNotEstablished,
                    basis,
                ))
                .is_err()
            );
        }
    }

    #[test]
    fn response_preserves_classification_provenance() {
        let response = build_response(&input(
            DiverticulitisFinding::ConfinedPericolicOrMesocolicAbscess,
            AssessmentBasis::Ct,
        ))
        .unwrap();
        assert_eq!(response.result, json!("Ib"));
        assert_eq!(response.working["assessment_basis"], json!("ct"));
        assert_eq!(
            response.working["category_kind"],
            json!("modified_hinchey_stage")
        );
        assert!(response.reference.contains("Kaiser AM"));
    }

    #[test]
    fn dynamic_calculation_matches_typed_contract_and_rejects_unknown_fields() {
        let typed_input = input(
            DiverticulitisFinding::PelvicDistantIntraAbdominalOrRetroperitonealAbscess,
            AssessmentBasis::Ct,
        );
        let dynamic = Hinchey
            .calculate(&serde_json::to_value(typed_input).unwrap())
            .unwrap();
        assert_eq!(dynamic, build_response(&typed_input).unwrap());

        let mut value = serde_json::to_value(typed_input).unwrap();
        value["abscess_diameter_cm"] = json!(5.0);
        assert!(Hinchey.calculate(&value).is_err());
    }

    #[test]
    fn schema_is_closed_and_documents_classification_traps() {
        let schema = Hinchey.input_schema();
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"].as_array().unwrap().len(), 3);
        assert_eq!(schema["allOf"].as_array().unwrap().len(), 4);
        let finding = &schema["properties"]["finding"]["definition"];
        assert!(
            finding["excludes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str().unwrap().contains("diameter"))
        );
        assert!(
            finding["caveats"]
                .as_str()
                .unwrap()
                .contains("rather than guessing")
        );
    }
}
