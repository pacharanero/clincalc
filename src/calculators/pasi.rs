// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Psoriasis Area and Severity Index (PASI).
//!
//! PASI combines clinician-rated plaque characteristics and affected-area
//! grades across four body regions. It does not diagnose psoriasis or encode a
//! treatment threshold.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "pasi";
pub const REFERENCE: &str = "Fredriksson T, Pettersson U. Severe psoriasis - oral therapy with a new retinoid. Dermatologica. 1978;157(4):238-244. doi:10.1159/000250839. Berth-Jones J, Grotzinger K, Rainville C, et al. A study examining inter- and intrarater reliability of three scales for measuring severity of psoriasis: Psoriasis Area and Severity Index, Physician's Global Assessment and Lattice System Physician's Global Assessment. Br J Dermatol. 2006;155(4):707-713. doi:10.1111/j.1365-2133.2006.07389.x. Youn SW, Choi CW, Kim BR, Chae JB. Reduction of inter-rater and intra-rater variability in psoriasis area and severity index assessment by photographic training. Ann Dermatol. 2015;27(5):557-562. doi:10.5021/ad.2015.27.5.557.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Uncopyrightable method under 17 U.S.C. Section 102(b) - independently implemented from primary literature",
    source_url: "https://uscode.house.gov/view.xhtml?req=granuleid:USC-prelim-title17-section102&num=0&edition=prelim",
};

const FORMULA: &str = "sum(region_weight * area_grade * (erythema + induration + desquamation))";
const LIMITATIONS: &str = "PASI is a clinician-rated severity measure. It does not diagnose psoriasis and is not a treatment rule. It does not measure itch, pain, quality of life, nail disease, psoriatic arthritis, or the disproportionate effect of disease at sensitive or functionally important sites. Its nonlinear area grades have limited sensitivity at low body-surface involvement, and area and intensity ratings vary between assessors; trained, consistent assessment is important. A low score can coexist with substantial individual burden. Absolute PASI thresholds and PASI 75/90/100 response targets are context-specific and are not calculated here.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentContext {
    ClinicianAssessedPlaquePsoriasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionAssessment {
    pub area_grade: u8,
    pub erythema: u8,
    pub induration: u8,
    pub desquamation: u8,
}

impl RegionAssessment {
    fn intensity_sum(self) -> u8 {
        self.erythema + self.induration + self.desquamation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PasiInput {
    pub assessment_context: AssessmentContext,
    pub head_and_neck: RegionAssessment,
    pub upper_limbs: RegionAssessment,
    pub trunk: RegionAssessment,
    pub lower_limbs: RegionAssessment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionOutcome {
    pub intensity_sum: u8,
    pub weighted_tenths: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PasiOutcome {
    pub pasi: f64,
    pub total_tenths: u16,
    pub head_and_neck: RegionOutcome,
    pub upper_limbs: RegionOutcome,
    pub trunk: RegionOutcome,
    pub lower_limbs: RegionOutcome,
}

fn validate_region(name: &str, region: RegionAssessment) -> Result<(), CalcError> {
    if region.area_grade > 6 {
        return Err(CalcError::InvalidInput(format!(
            "{name}.area_grade must be between 0 and 6"
        )));
    }
    for (field, value) in [
        ("erythema", region.erythema),
        ("induration", region.induration),
        ("desquamation", region.desquamation),
    ] {
        if value > 4 {
            return Err(CalcError::InvalidInput(format!(
                "{name}.{field} must be between 0 and 4"
            )));
        }
    }
    if (region.area_grade == 0) != (region.intensity_sum() == 0) {
        return Err(CalcError::InvalidInput(format!(
            "{name} must have area_grade 0 exactly when all three intensity grades are 0"
        )));
    }
    Ok(())
}

fn score_region(region: RegionAssessment, weight_tenths: u16) -> RegionOutcome {
    let intensity_sum = region.intensity_sum();
    RegionOutcome {
        intensity_sum,
        weighted_tenths: weight_tenths * u16::from(region.area_grade) * u16::from(intensity_sum),
    }
}

pub fn compute(input: &PasiInput) -> Result<PasiOutcome, CalcError> {
    for (name, region) in [
        ("head_and_neck", input.head_and_neck),
        ("upper_limbs", input.upper_limbs),
        ("trunk", input.trunk),
        ("lower_limbs", input.lower_limbs),
    ] {
        validate_region(name, region)?;
    }

    let head_and_neck = score_region(input.head_and_neck, 1);
    let upper_limbs = score_region(input.upper_limbs, 2);
    let trunk = score_region(input.trunk, 3);
    let lower_limbs = score_region(input.lower_limbs, 4);
    let total_tenths = head_and_neck.weighted_tenths
        + upper_limbs.weighted_tenths
        + trunk.weighted_tenths
        + lower_limbs.weighted_tenths;

    Ok(PasiOutcome {
        pasi: f64::from(total_tenths) / 10.0,
        total_tenths,
        head_and_neck,
        upper_limbs,
        trunk,
        lower_limbs,
    })
}

fn region_working(input: RegionAssessment, outcome: RegionOutcome, weight: f64) -> Value {
    json!({
        "area_grade": input.area_grade,
        "erythema": input.erythema,
        "induration": input.induration,
        "desquamation": input.desquamation,
        "intensity_sum": outcome.intensity_sum,
        "region_weight": weight,
        "weighted_score": f64::from(outcome.weighted_tenths) / 10.0
    })
}

pub fn build_response(input: &PasiInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert("assessment_context".into(), json!(input.assessment_context));
    working.insert(
        "head_and_neck".into(),
        region_working(input.head_and_neck, outcome.head_and_neck, 0.1),
    );
    working.insert(
        "upper_limbs".into(),
        region_working(input.upper_limbs, outcome.upper_limbs, 0.2),
    );
    working.insert(
        "trunk".into(),
        region_working(input.trunk, outcome.trunk, 0.3),
    );
    working.insert(
        "lower_limbs".into(),
        region_working(input.lower_limbs, outcome.lower_limbs, 0.4),
    );
    working.insert("total_tenths".into(), json!(outcome.total_tenths));
    working.insert("maximum_score".into(), json!(72));
    working.insert("formula".into(), json!(FORMULA));
    working.insert("variant".into(), json!("standard_pasi"));
    working.insert("limitations".into(), json!(LIMITATIONS));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.pasi),
        interpretation: format!("PASI score {:.1}/72. {LIMITATIONS}", outcome.pasi),
        working,
        reference: REFERENCE.to_string(),
    })
}

fn intensity_property(concept: &str, description: &str, source: &Value) -> Value {
    json!({
        "type": "integer",
        "minimum": 0,
        "maximum": 4,
        "description": description,
        "definition": {
            "concept": concept,
            "statement": description,
            "excludes": ["A caller-supplied regional point total", "A finding outside the named body region", "A guessed grade when the region cannot be assessed"],
            "caveats": "Grade the average severity across all psoriatic plaques in the named region, not the single worst plaque: 0=absent, 1=slight, 2=moderate, 3=severe, and 4=very severe. Maintain a consistent assessment method across serial measurements.",
            "source": source,
            "status": "draft"
        }
    })
}

fn region_property(name: &str, anatomy: &str, source: &Value) -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["area_grade", "erythema", "induration", "desquamation"],
        "description": format!("PASI extent and average plaque-severity ratings for the {name} region. {anatomy}"),
        "allOf": [
            {
                "if": { "properties": { "area_grade": { "const": 0 } } },
                "then": { "properties": {
                    "erythema": { "const": 0 },
                    "induration": { "const": 0 },
                    "desquamation": { "const": 0 }
                }}
            },
            {
                "if": { "properties": { "area_grade": { "minimum": 1 } } },
                "then": { "anyOf": [
                    { "properties": { "erythema": { "minimum": 1 } } },
                    { "properties": { "induration": { "minimum": 1 } } },
                    { "properties": { "desquamation": { "minimum": 1 } } }
                ]}
            }
        ],
        "properties": {
            "area_grade": {
                "type": "integer",
                "minimum": 0,
                "maximum": 6,
                "description": "Affected proportion of this region: 0=none; 1=greater than 0% but less than 10%; 2=10-29%; 3=30-49%; 4=50-69%; 5=70-89%; 6=90-100%.",
                "definition": {
                    "concept": format!("PASI affected-area grade for {name}"),
                    "statement": format!("Estimate the proportion of the entire {name} region affected by psoriasis and select the corresponding PASI area grade: 0 for none, 1 for greater than 0% but less than 10%, 2 for 10-29%, 3 for 30-49%, 4 for 50-69%, 5 for 70-89%, or 6 for 90-100%. {anatomy}"),
                    "excludes": ["Percentage of total-body surface area", "A caller-supplied regional point total", "Unaffected skin"],
                    "caveats": "The PASI area categories are nonlinear and visual area estimation is a major source of assessor variability.",
                    "source": source,
                    "status": "draft"
                }
            },
            "erythema": intensity_property(
                &format!("PASI erythema grade for {name}"),
                &format!("Clinician-rated average erythema across psoriatic plaques in the {name} region, from 0=absent to 4=very severe."),
                source
            ),
            "induration": intensity_property(
                &format!("PASI induration grade for {name}"),
                &format!("Clinician-rated average plaque induration or thickness across psoriatic plaques in the {name} region, from 0=absent to 4=very severe."),
                source
            ),
            "desquamation": intensity_property(
                &format!("PASI desquamation grade for {name}"),
                &format!("Clinician-rated average desquamation or scaling across psoriatic plaques in the {name} region, from 0=absent to 4=very severe."),
                source
            )
        }
    })
}

fn input_schema() -> Value {
    let source = json!({
        "citation": "Fredriksson T, Pettersson U. Dermatologica. 1978;157(4):238-244.",
        "url": "https://doi.org/10.1159/000250839"
    });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "PasiInput",
        "description": "Standard clinician-assessed Psoriasis Area and Severity Index for plaque psoriasis. Enter extent and three plaque-severity ratings separately for all four body regions. This calculator does not diagnose psoriasis, calculate PASI 75/90/100 change from baseline, or determine treatment.",
        "type": "object",
        "additionalProperties": false,
        "required": ["assessment_context", "head_and_neck", "upper_limbs", "trunk", "lower_limbs"],
        "properties": {
            "assessment_context": {
                "type": "string",
                "const": "clinician_assessed_plaque_psoriasis",
                "description": "A clinician trained in PASI is assessing plaque psoriasis across all four required body regions.",
                "definition": {
                    "concept": "Supported PASI assessment context",
                    "statement": "Confirm that a clinician is performing a standard PASI assessment of plaque psoriasis and can assess all four required body regions.",
                    "excludes": ["Use to diagnose psoriasis", "Patient self-assessment", "A partial examination", "PASI-HD or another PASI variant"],
                    "caveats": "PASI was developed and validated principally for clinician assessment of plaque psoriasis. Other phenotypes, nails, symptoms, and patient impact require separate assessment.",
                    "source": source,
                    "status": "draft"
                }
            },
            "head_and_neck": region_property("head and neck", "Include the neck in this region.", &source),
            "upper_limbs": region_property("upper limbs", "Include the palms in this region.", &source),
            "trunk": region_property("trunk", "Include the axillae and genital area in this region; the inguinal boundary separates trunk from lower limbs.", &source),
            "lower_limbs": region_property("lower limbs", "Include the buttocks and soles in this region; the inguinal boundary separates lower limbs from trunk.", &source)
        }
    })
}

pub struct Pasi;

impl Calculator for Pasi {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Psoriasis Area and Severity Index (PASI)"
    }

    fn description(&self) -> &'static str {
        "Calculates standard PASI from clinician-rated extent, erythema, induration, and desquamation across four body regions without imposing a treatment threshold."
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
        let parsed: PasiInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(area_grade: u8, erythema: u8, induration: u8, desquamation: u8) -> RegionAssessment {
        RegionAssessment {
            area_grade,
            erythema,
            induration,
            desquamation,
        }
    }

    fn zero() -> PasiInput {
        PasiInput {
            assessment_context: AssessmentContext::ClinicianAssessedPlaquePsoriasis,
            head_and_neck: region(0, 0, 0, 0),
            upper_limbs: region(0, 0, 0, 0),
            trunk: region(0, 0, 0, 0),
            lower_limbs: region(0, 0, 0, 0),
        }
    }

    #[test]
    fn source_formula_minimum_and_maximum_are_exact() {
        assert_eq!(compute(&zero()).unwrap().pasi, 0.0);
        let maximum = PasiInput {
            head_and_neck: region(6, 4, 4, 4),
            upper_limbs: region(6, 4, 4, 4),
            trunk: region(6, 4, 4, 4),
            lower_limbs: region(6, 4, 4, 4),
            ..zero()
        };
        let outcome = compute(&maximum).unwrap();
        assert_eq!(outcome.total_tenths, 720);
        assert_eq!(outcome.pasi, 72.0);
    }

    #[test]
    fn source_formula_regional_and_mixed_vectors_are_exact() {
        // Fredriksson and Pettersson's published formula gives a maximum trunk
        // contribution of 0.3 * 6 * 12 = 21.6.
        let trunk_only = PasiInput {
            trunk: region(6, 4, 4, 4),
            ..zero()
        };
        assert_eq!(compute(&trunk_only).unwrap().pasi, 21.6);

        let mixed = PasiInput {
            head_and_neck: region(1, 1, 2, 3),
            upper_limbs: region(2, 2, 2, 2),
            trunk: region(3, 3, 3, 3),
            lower_limbs: region(4, 4, 4, 4),
            ..zero()
        };
        let outcome = compute(&mixed).unwrap();
        assert_eq!(outcome.total_tenths, 303);
        assert_eq!(outcome.pasi, 30.3);
    }

    #[test]
    fn rejects_every_out_of_range_domain() {
        let mut input = zero();
        input.head_and_neck.area_grade = 7;
        assert!(compute(&input).is_err());

        for field in ["erythema", "induration", "desquamation"] {
            let mut input = zero();
            match field {
                "erythema" => input.upper_limbs.erythema = 5,
                "induration" => input.upper_limbs.induration = 5,
                _ => input.upper_limbs.desquamation = 5,
            }
            assert!(compute(&input).is_err());
        }
    }

    #[test]
    fn rejects_internally_contradictory_region_assessments() {
        let mut no_area_with_signs = zero();
        no_area_with_signs.head_and_neck.erythema = 1;
        assert!(compute(&no_area_with_signs).is_err());

        let mut area_without_signs = zero();
        area_without_signs.head_and_neck.area_grade = 1;
        assert!(compute(&area_without_signs).is_err());
    }

    #[test]
    fn response_is_a_measure_not_a_treatment_rule() {
        let input = PasiInput {
            trunk: region(3, 3, 3, 3),
            ..zero()
        };
        let response = build_response(&input).unwrap();
        assert_eq!(response.result, json!(8.1));
        assert_eq!(response.working["variant"], json!("standard_pasi"));
        assert_eq!(response.working["trunk"]["weighted_score"], json!(8.1));
        assert!(!response.working.contains_key("severity_band"));
        assert!(response.interpretation.contains("does not diagnose"));
        assert!(response.interpretation.contains("not a treatment rule"));
        assert!(!response.interpretation.contains("biologic"));
    }

    #[test]
    fn dynamic_surface_is_closed_at_both_object_levels() {
        let value = serde_json::to_value(zero()).unwrap();
        assert_eq!(
            Pasi.calculate(&value).unwrap(),
            build_response(&zero()).unwrap()
        );

        let mut top_level_unknown = value.clone();
        top_level_unknown["baseline_pasi"] = json!(12.0);
        assert!(Pasi.calculate(&top_level_unknown).is_err());

        let mut nested_unknown = value;
        nested_unknown["trunk"]["affected_percent"] = json!(25);
        assert!(Pasi.calculate(&nested_unknown).is_err());
    }

    #[test]
    fn schema_is_closed_complete_and_defines_each_clinical_input() {
        let schema = Pasi.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["required"].as_array().unwrap().len(), 5);
        assert_eq!(
            schema["properties"]["head_and_neck"]["additionalProperties"],
            json!(false)
        );
        assert_eq!(
            schema["properties"]["lower_limbs"]["properties"]["area_grade"]["maximum"],
            json!(6)
        );
        assert_eq!(
            schema["properties"]["trunk"]["properties"]["erythema"]["maximum"],
            json!(4)
        );
        assert_eq!(
            schema["properties"]["trunk"]["allOf"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert!(Pasi.license().license.contains("independently implemented"));
        assert!(
            schema["properties"]["head_and_neck"]["description"]
                .as_str()
                .unwrap()
                .contains("neck")
        );
        assert!(
            schema["properties"]["trunk"]["description"]
                .as_str()
                .unwrap()
                .contains("inguinal")
        );

        for region_name in ["head_and_neck", "upper_limbs", "trunk", "lower_limbs"] {
            for property in schema["properties"][region_name]["properties"]
                .as_object()
                .unwrap()
                .values()
            {
                assert!(property["definition"]["statement"].is_string());
                assert_eq!(property["definition"]["status"], json!("draft"));
            }
        }
    }
}
