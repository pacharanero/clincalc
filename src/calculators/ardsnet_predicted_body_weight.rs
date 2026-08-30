// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! ARDSNet predicted body weight (PBW) for adult lung-protective ventilation protocols.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "ardsnet_predicted_body_weight";
pub const REFERENCE: &str = "NIH-NHLBI ARDS Network. ARDSNet tools and mechanical ventilation protocol. https://www.ardsnet.org/tools.html. The Acute Respiratory Distress Syndrome Network. Ventilation with lower tidal volumes as compared with traditional tidal volumes for acute lung injury and the acute respiratory distress syndrome. N Engl J Med. 2000;342(18):1301-1308. doi:10.1056/NEJM200005043421801. PMID: 10793162.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Free use with attribution - cite the NIH-NHLBI ARDS Network as source",
    source_url: "https://www.ardsnet.org/tools.html",
};

const MIN_TABLE_HEIGHT_IN: f64 = 48.0;
const MAX_TABLE_HEIGHT_IN: f64 = 84.0;
const MIN_TABLE_HEIGHT_CM: f64 = 121.92;
const MAX_TABLE_HEIGHT_CM: f64 = 213.36;
const MALE_FORMULA: &str = "50.0 + 2.3 * (height_inches - 60.0)";
const FEMALE_FORMULA: &str = "45.5 + 2.3 * (height_inches - 60.0)";
const LIMITATIONS: &str = "Adult-only. Predicted body weight (PBW) is not actual weight, nutritional ideal weight, adjusted weight, or drug-dosing weight. This calculator does not prescribe tidal volume or any other ventilator setting. Verify measured height and the historical ARDSNet coefficient branch. Use only within an appropriate clinician-directed ventilation protocol with pressure, gas exchange, pH, synchrony, haemodynamic, and clinical monitoring. No paediatric use. Actual weight must not substitute where PBW is required.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentContext {
    AdultLungProtectiveVentilationProtocolUsingArdsnetPredictedBodyWeight,
}

/// Historical ARDSNet coefficient branch required by the protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormulaBranch {
    Male,
    Female,
}

impl FormulaBranch {
    fn slug(self) -> &'static str {
        match self {
            FormulaBranch::Male => "male",
            FormulaBranch::Female => "female",
        }
    }

    fn formula(self) -> &'static str {
        match self {
            FormulaBranch::Male => MALE_FORMULA,
            FormulaBranch::Female => FEMALE_FORMULA,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArdsnetPredictedBodyWeightInput {
    /// Attests adult age and use within an appropriate clinician-directed protocol.
    pub assessment_context: AssessmentContext,
    /// Measured adult height in centimetres.
    pub height_cm: f64,
    /// Historical ARDSNet coefficient branch required by the protocol.
    pub formula_branch: FormulaBranch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArdsnetPredictedBodyWeightOutcome {
    pub predicted_body_weight_kg: f64,
    pub outside_official_reference_table_range: bool,
    pub interpretation: String,
}

pub fn compute(
    input: &ArdsnetPredictedBodyWeightInput,
) -> Result<ArdsnetPredictedBodyWeightOutcome, CalcError> {
    if !input.height_cm.is_finite() || input.height_cm <= 0.0 {
        return Err(CalcError::InvalidInput(
            "height_cm must be a finite positive measured adult height".into(),
        ));
    }

    let height_inches = input.height_cm / 2.54;
    let predicted_body_weight_kg = match input.formula_branch {
        FormulaBranch::Male => 50.0 + 2.3 * (height_inches - 60.0),
        FormulaBranch::Female => 45.5 + 2.3 * (height_inches - 60.0),
    };
    if !predicted_body_weight_kg.is_finite() || predicted_body_weight_kg <= 0.0 {
        return Err(CalcError::InvalidInput(
            "height_cm and formula_branch produce a non-positive or non-finite predicted body weight"
                .into(),
        ));
    }

    let outside_official_reference_table_range =
        !(MIN_TABLE_HEIGHT_CM..=MAX_TABLE_HEIGHT_CM).contains(&input.height_cm);
    let warning = if outside_official_reference_table_range {
        " WARNING: measured height is outside the official ARDSNet reference table range of 48-84 inches (121.92-213.36 cm); the formula result is an explicit extrapolation and has not been clamped."
    } else {
        ""
    };
    let interpretation = format!(
        "ARDSNet predicted body weight is {:.1} kg using the historical {} coefficient branch.{warning} {LIMITATIONS}",
        round1(predicted_body_weight_kg),
        input.formula_branch.slug(),
    );

    Ok(ArdsnetPredictedBodyWeightOutcome {
        predicted_body_weight_kg,
        outside_official_reference_table_range,
        interpretation,
    })
}

pub fn build_response(
    input: &ArdsnetPredictedBodyWeightInput,
) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let height_inches = input.height_cm / 2.54;
    let mut working = Map::new();
    working.insert("assessment_context".into(), json!(input.assessment_context));
    working.insert("height_cm".into(), json!(input.height_cm));
    working.insert("height_inches".into(), json!(height_inches));
    working.insert("formula_branch".into(), json!(input.formula_branch.slug()));
    working.insert("formula".into(), json!(input.formula_branch.formula()));
    working.insert(
        "predicted_body_weight_kg_unrounded".into(),
        json!(outcome.predicted_body_weight_kg),
    );
    working.insert("result_unit".into(), json!("kg"));
    working.insert(
        "official_reference_table_height_inches_min".into(),
        json!(MIN_TABLE_HEIGHT_IN),
    );
    working.insert(
        "official_reference_table_height_inches_max".into(),
        json!(MAX_TABLE_HEIGHT_IN),
    );
    working.insert(
        "official_reference_table_height_cm_min".into(),
        json!(MIN_TABLE_HEIGHT_CM),
    );
    working.insert(
        "official_reference_table_height_cm_max".into(),
        json!(MAX_TABLE_HEIGHT_CM),
    );
    working.insert(
        "outside_official_reference_table_range".into(),
        json!(outcome.outside_official_reference_table_range),
    );
    working.insert("limitations".into(), json!(LIMITATIONS));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(outcome.predicted_body_weight_kg)),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(value: f64) -> f64 {
    if value.abs() > f64::MAX / 10.0 {
        value
    } else {
        (value * 10.0).round() / 10.0
    }
}

fn input_schema() -> Value {
    let source = json!({
        "citation": "NIH-NHLBI ARDS Network. ARDSNet tools and mechanical ventilation protocol; The Acute Respiratory Distress Syndrome Network. N Engl J Med. 2000;342(18):1301-1308. doi:10.1056/NEJM200005043421801. PMID: 10793162.",
        "url": "https://www.ardsnet.org/tools.html"
    });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "ArdsnetPredictedBodyWeightInput",
        "description": "Calculates ARDSNet predicted body weight for adults aged 18 years or older within an appropriate clinician-directed lung-protective ventilation protocol. PBW is not actual weight, nutritional ideal weight, adjusted weight, or drug-dosing weight and this calculator does not prescribe tidal volume or ventilator settings. The official reference table covers 48-84 inches (121.92-213.36 cm), inclusive. Finite positive heights outside that range are accepted only when the derived PBW remains positive, are not clamped, and produce an explicit extrapolation warning. Verify measured height and branch; use with pressure, gas exchange, pH, synchrony, haemodynamic, and clinical monitoring. No paediatric use. Actual weight must not substitute where PBW is required.",
        "type": "object",
        "additionalProperties": false,
        "required": ["assessment_context", "height_cm", "formula_branch"],
        "properties": {
            "assessment_context": {
                "type": "string",
                "const": "adult_lung_protective_ventilation_protocol_using_ardsnet_predicted_body_weight",
                "unit": "none",
                "description": "Attestation that the patient is an adult aged 18 years or older and PBW is being used within an appropriate clinician-directed lung-protective ventilation protocol",
                "definition": {
                    "concept": "Adult ARDSNet PBW protocol assessment context",
                    "statement": "Select this value only to attest that the patient is aged 18 years or older and the calculation is being used within an appropriate clinician-directed lung-protective ventilation protocol using ARDSNet predicted body weight.",
                    "includes": ["Adult age 18 years or older", "Clinician-directed protocol that explicitly requires ARDSNet predicted body weight"],
                    "excludes": ["Paediatric use", "Use outside a clinician-directed ventilation protocol", "Use as a stand-alone prescription of tidal volume or ventilator settings"],
                    "source": source,
                    "snomedEcl": null,
                    "refset": null,
                    "caveats": "The calculation does not establish an indication for ventilation, diagnose ARDS, prescribe ventilator settings, or replace pressure, gas exchange, pH, synchrony, haemodynamic, and clinical monitoring.",
                    "status": "draft"
                }
            },
            "height_cm": {
                "type": "number",
                "exclusiveMinimum": 0,
                "unit": "cm",
                "description": "Measured adult height in centimetres. The official table range is 121.92-213.36 cm inclusive; there is deliberately no arbitrary schema maximum.",
                "definition": {
                    "concept": "Measured adult height for ARDSNet PBW",
                    "statement": "Enter measured adult height in centimetres; the formula converts centimetres to inches exactly by dividing by 2.54.",
                    "includes": ["Measured adult height expressed in centimetres"],
                    "excludes": ["Actual body weight", "Estimated height without checking the protocol's permitted measurement method", "Inches entered without conversion to centimetres"],
                    "source": source,
                    "snomedEcl": null,
                    "refset": null,
                    "caveats": "The official ARDSNet table spans 48-84 inches (121.92-213.36 cm), inclusive. Outside-range values are explicit, unclamped formula extrapolations and trigger a warning. Runtime validation also rejects any height and branch combination that derives a non-positive or non-finite PBW.",
                    "status": "draft"
                }
            },
            "formula_branch": {
                "type": "string",
                "enum": ["male", "female"],
                "unit": "none",
                "description": "Historical male or female ARDSNet coefficient branch required by the protocol; this is not an inferred modern sex or gender definition",
                "definition": {
                    "concept": "Historical ARDSNet PBW coefficient branch",
                    "statement": "Select the male or female coefficient branch required by the governing ARDSNet protocol and verify that selection clinically.",
                    "includes": ["male: 50.0 kg intercept at 60 inches", "female: 45.5 kg intercept at 60 inches"],
                    "excludes": ["Automatic inference from name, appearance, gender identity, or other unstated data", "Interpretation as a modern definition of sex or gender"],
                    "source": source,
                    "snomedEcl": null,
                    "refset": null,
                    "caveats": "These labels identify historical protocol coefficient branches. They do not resolve how to select a branch for every patient; follow the clinician-directed protocol and document the verified branch.",
                    "status": "draft"
                }
            }
        }
    })
}

pub struct ArdsnetPredictedBodyWeight;

impl Calculator for ArdsnetPredictedBodyWeight {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "ARDSNet Predicted Body Weight"
    }

    fn description(&self) -> &'static str {
        "Calculates adult ARDSNet predicted body weight from measured height and an explicit historical protocol coefficient branch."
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
        let parsed: ArdsnetPredictedBodyWeightInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(height_cm: f64, formula_branch: FormulaBranch) -> ArdsnetPredictedBodyWeightInput {
        ArdsnetPredictedBodyWeightInput {
            assessment_context: AssessmentContext::AdultLungProtectiveVentilationProtocolUsingArdsnetPredictedBodyWeight,
            height_cm,
            formula_branch,
        }
    }

    #[test]
    fn official_table_vectors_match() {
        let vectors = [
            (152.4, FormulaBranch::Female, 45.5, 45.5),
            (152.4, FormulaBranch::Male, 50.0, 50.0),
            (121.92, FormulaBranch::Female, 17.9, 17.9),
            (213.36, FormulaBranch::Male, 105.2, 105.2),
        ];
        for (height_cm, branch, expected_unrounded, expected_rounded) in vectors {
            let typed = input(height_cm, branch);
            let outcome = compute(&typed).unwrap();
            assert!((outcome.predicted_body_weight_kg - expected_unrounded).abs() < 1e-12);
            assert_eq!(
                build_response(&typed).unwrap().result,
                json!(expected_rounded)
            );
        }
    }

    #[test]
    fn converts_centimetres_to_inches_exactly() {
        let response = build_response(&input(152.4, FormulaBranch::Male)).unwrap();
        assert_eq!(response.working["height_inches"], json!(152.4 / 2.54));
        assert_eq!(response.working["height_inches"], json!(60.0));
    }

    #[test]
    fn table_boundaries_are_inclusive_and_just_outside_warns() {
        for height_cm in [MIN_TABLE_HEIGHT_CM, MAX_TABLE_HEIGHT_CM] {
            let outcome = compute(&input(height_cm, FormulaBranch::Male)).unwrap();
            assert!(!outcome.outside_official_reference_table_range);
            assert!(!outcome.interpretation.contains("WARNING:"));
        }
        for height_cm in [MIN_TABLE_HEIGHT_CM - 0.01, MAX_TABLE_HEIGHT_CM + 0.01] {
            let outcome = compute(&input(height_cm, FormulaBranch::Male)).unwrap();
            assert!(outcome.outside_official_reference_table_range);
            assert!(outcome.interpretation.contains("WARNING:"));
            assert!(outcome.interpretation.contains("explicit extrapolation"));
            assert!(outcome.interpretation.contains("not been clamped"));
        }
    }

    #[test]
    fn both_historical_branches_produce_distinct_outputs() {
        let male = compute(&input(170.0, FormulaBranch::Male)).unwrap();
        let female = compute(&input(170.0, FormulaBranch::Female)).unwrap();
        assert!(
            (male.predicted_body_weight_kg - female.predicted_body_weight_kg - 4.5).abs() < 1e-12
        );
    }

    #[test]
    fn rejects_nonfinite_nonpositive_and_nonpositive_pbw_inputs() {
        for height_cm in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -1.0] {
            assert!(compute(&input(height_cm, FormulaBranch::Male)).is_err());
        }
        assert!(compute(&input(1.0, FormulaBranch::Male)).is_err());
        assert!(compute(&input(1.0, FormulaBranch::Female)).is_err());
    }

    #[test]
    fn extreme_finite_extrapolation_does_not_overflow_rounding() {
        let response = build_response(&input(f64::MAX, FormulaBranch::Male)).unwrap();
        assert!(response.result.as_f64().unwrap().is_finite());
        assert_eq!(
            response.working["outside_official_reference_table_range"],
            json!(true)
        );
        assert!(response.interpretation.contains("WARNING:"));
    }

    #[test]
    fn dynamic_typed_parity_and_unknown_field_rejection() {
        let typed_input = input(170.0, FormulaBranch::Male);
        let value = serde_json::to_value(typed_input).unwrap();
        assert_eq!(
            ArdsnetPredictedBodyWeight.calculate(&value).unwrap(),
            build_response(&typed_input).unwrap()
        );
        let mut unknown = value;
        unknown["actual_weight_kg"] = json!(80.0);
        assert!(ArdsnetPredictedBodyWeight.calculate(&unknown).is_err());
    }

    #[test]
    fn response_preserves_provenance_unrounded_value_and_formula() {
        let response = build_response(&input(170.0, FormulaBranch::Male)).unwrap();
        let expected = 50.0 + 2.3 * (170.0 / 2.54 - 60.0);
        assert_eq!(response.calculator, NAME);
        assert_eq!(response.result, json!(65.9));
        assert_eq!(response.working["height_cm"], json!(170.0));
        assert_eq!(response.working["formula_branch"], json!("male"));
        assert_eq!(response.working["formula"], json!(MALE_FORMULA));
        assert_eq!(
            response.working["predicted_body_weight_kg_unrounded"],
            json!(expected)
        );
        assert_eq!(response.working["result_unit"], json!("kg"));
        assert_eq!(
            response.working["official_reference_table_height_cm_min"],
            json!(121.92)
        );
        assert_eq!(
            response.working["outside_official_reference_table_range"],
            json!(false)
        );
        assert!(response.reference.contains("ardsnet.org/tools.html"));
        assert!(response.reference.contains("10.1056/NEJM200005043421801"));
        assert!(response.reference.contains("10793162"));
    }

    #[test]
    fn safety_wording_is_complete_and_no_tidal_volume_is_output() {
        let response = build_response(&input(170.0, FormulaBranch::Female)).unwrap();
        for wording in [
            "Adult-only",
            "not actual weight",
            "nutritional ideal weight",
            "adjusted weight",
            "drug-dosing weight",
            "does not prescribe tidal volume",
            "Verify measured height",
            "pressure, gas exchange, pH, synchrony, haemodynamic, and clinical monitoring",
            "No paediatric use",
            "Actual weight must not substitute where PBW is required",
        ] {
            assert!(
                response.interpretation.contains(wording),
                "missing {wording}"
            );
        }
        assert!(!response.working.contains_key("tidal_volume_ml"));
        assert!(!response.working.contains_key("tidal_volume_ml_per_kg"));
    }

    #[test]
    fn schema_is_closed_required_defined_unit_explicit_and_has_no_height_maximum() {
        let schema = ArdsnetPredictedBodyWeight.input_schema();
        assert_eq!(
            schema["$schema"],
            json!("https://json-schema.org/draft/2020-12/schema")
        );
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["required"].as_array().unwrap().len(), 3);
        assert_eq!(
            schema["properties"]["assessment_context"]["const"],
            json!("adult_lung_protective_ventilation_protocol_using_ardsnet_predicted_body_weight")
        );
        assert_eq!(
            schema["properties"]["height_cm"]["exclusiveMinimum"],
            json!(0)
        );
        assert_eq!(schema["properties"]["height_cm"]["unit"], json!("cm"));
        assert!(schema["properties"]["height_cm"].get("maximum").is_none());
        assert!(
            schema["properties"]["height_cm"]["definition"]["caveats"]
                .as_str()
                .unwrap()
                .contains("non-positive or non-finite PBW")
        );
        for property in schema["properties"].as_object().unwrap().values() {
            assert!(property["unit"].is_string());
            let definition = &property["definition"];
            for key in [
                "concept",
                "statement",
                "includes",
                "excludes",
                "source",
                "snomedEcl",
                "refset",
                "caveats",
                "status",
            ] {
                assert!(definition.get(key).is_some(), "missing definition.{key}");
            }
            assert!(definition["source"]["citation"].is_string());
            assert!(definition["source"]["url"].is_string());
            assert_eq!(definition["status"], json!("draft"));
        }
    }

    #[test]
    fn license_records_required_attribution_and_source() {
        assert_eq!(
            ArdsnetPredictedBodyWeight.license().license,
            "Free use with attribution - cite the NIH-NHLBI ARDS Network as source"
        );
        assert_eq!(
            ArdsnetPredictedBodyWeight.license().source_url,
            "https://www.ardsnet.org/tools.html"
        );
    }

    #[test]
    fn committed_example_deserializes_and_computes() {
        let parsed: ArdsnetPredictedBodyWeightInput = serde_json::from_str(include_str!(
            "../../examples/ardsnet-predicted-body-weight.json"
        ))
        .unwrap();
        assert_eq!(parsed, input(170.0, FormulaBranch::Male));
        assert_eq!(build_response(&parsed).unwrap().result, json!(65.9));
    }
}
