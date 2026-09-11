// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Skeletal muscle mass index (SMI) and the FNIH lean-mass ratio.
//!
//! Two independent height/BMI-normalised indices of appendicular lean mass
//! (ALM, also called appendicular skeletal muscle mass, ASM: the combined
//! lean mass of both arms and both legs, typically measured by DXA), each
//! with its own consensus cut-off for "low muscle quantity":
//!
//! - **EWGSOP2** (European Working Group on Sarcopenia in Older People 2):
//!   SMI = ALM (kg) / height (m)^2. Low muscle quantity: <7.0 kg/m^2 (men),
//!   <5.5 kg/m^2 (women).
//! - **FNIH** (Foundation for the National Institutes of Health Sarcopenia
//!   Project): ALM/BMI = ALM (kg) / BMI (kg/m^2). Low lean mass: <0.789
//!   (men), <0.512 (women).
//!
//! Both are reported because the two working groups pin different cut-offs
//! and neither supersedes the other; a host can choose which to display.
//!
//! **This calculator assesses muscle quantity only.** Per EWGSOP2's own
//! case-finding algorithm, low muscle quantity confirms sarcopenia only in
//! combination with low muscle strength (grip strength or chair-stand test,
//! not computed here), and severity is graded separately by physical
//! performance (gait speed, SPPB, or similar, also not computed here). A low
//! or normal result from this calculator alone must not be used to diagnose
//! or exclude sarcopenia.
//!
//! References: Cruz-Jentoft AJ, Bahat G, Bauer J, et al. Sarcopenia: revised
//! European consensus on definition and diagnosis. Age Ageing.
//! 2019;48(1):16-31. doi:10.1093/ageing/afy169. | Studenski SA, Peters KW,
//! Alley DE, et al. The FNIH sarcopenia project: rationale, study
//! description, conference recommendations, and final estimates. J Gerontol
//! A Biol Sci Med Sci. 2014;69(5):547-558. doi:10.1093/gerona/glu010.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "skeletal_muscle_mass_index";

pub const REFERENCE: &str = "Cruz-Jentoft AJ, Bahat G, Bauer J, et al. Sarcopenia: revised European consensus on definition and diagnosis. \
Age Ageing. 2019;48(1):16-31. doi:10.1093/ageing/afy169. | \
Studenski SA, Peters KW, Alley DE, et al. The FNIH sarcopenia project: rationale, study description, conference recommendations, and final estimates. \
J Gerontol A Biol Sci Med Sci. 2014;69(5):547-558. doi:10.1093/gerona/glu010.";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1093/ageing/afy169",
};

const EWGSOP2_CUTOFF_MALE: f64 = 7.0;
const EWGSOP2_CUTOFF_FEMALE: f64 = 5.5;
const FNIH_CUTOFF_MALE: f64 = 0.789;
const FNIH_CUTOFF_FEMALE: f64 = 0.512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletalMuscleMassIndexInput {
    /// Appendicular lean mass (ALM / ASM): combined lean mass of both arms
    /// and both legs, in kilograms, from an independently obtained method
    /// (typically whole-body DXA).
    pub appendicular_lean_mass_kg: f64,
    /// Standing height in centimetres.
    pub height_cm: f64,
    /// Total body weight in kilograms, used to derive BMI for the FNIH ratio.
    pub weight_kg: f64,
    pub sex: Sex,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkeletalMuscleMassIndexOutcome {
    pub bmi: f64,
    pub smi_kg_m2: f64,
    pub low_muscle_quantity_ewgsop2: bool,
    pub alm_bmi_ratio: f64,
    pub low_lean_mass_fnih: bool,
    pub interpretation: String,
}

pub fn compute(
    input: &SkeletalMuscleMassIndexInput,
) -> Result<SkeletalMuscleMassIndexOutcome, CalcError> {
    if !(2.0..=60.0).contains(&input.appendicular_lean_mass_kg)
        || !input.appendicular_lean_mass_kg.is_finite()
    {
        return Err(CalcError::InvalidInput(
            "appendicular_lean_mass_kg must be finite and between 2 and 60".into(),
        ));
    }
    if !(100.0..=250.0).contains(&input.height_cm) || !input.height_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "height_cm must be finite and between 100 and 250".into(),
        ));
    }
    if !(20.0..=400.0).contains(&input.weight_kg) || !input.weight_kg.is_finite() {
        return Err(CalcError::InvalidInput(
            "weight_kg must be finite and between 20 and 400".into(),
        ));
    }
    if input.appendicular_lean_mass_kg >= input.weight_kg {
        return Err(CalcError::InvalidInput(
            "appendicular_lean_mass_kg cannot exceed total body weight".into(),
        ));
    }

    let height_m = input.height_cm / 100.0;
    let bmi = input.weight_kg / height_m.powi(2);
    let smi_kg_m2 = input.appendicular_lean_mass_kg / height_m.powi(2);
    let alm_bmi_ratio = input.appendicular_lean_mass_kg / bmi;

    let (ewgsop2_cutoff, fnih_cutoff) = match input.sex {
        Sex::Male => (EWGSOP2_CUTOFF_MALE, FNIH_CUTOFF_MALE),
        Sex::Female => (EWGSOP2_CUTOFF_FEMALE, FNIH_CUTOFF_FEMALE),
    };
    let low_muscle_quantity_ewgsop2 = smi_kg_m2 < ewgsop2_cutoff;
    let low_lean_mass_fnih = alm_bmi_ratio < fnih_cutoff;

    let sex_label = match input.sex {
        Sex::Male => "male",
        Sex::Female => "female",
    };

    let interpretation = format!(
        "SMI (ALM/height^2) {smi_kg_m2:.1} kg/m^2 against the EWGSOP2 {sex_label} cut-off of {ewgsop2_cutoff:.1} kg/m^2: {ewgsop2_verdict} muscle quantity. \
ALM/BMI ratio {alm_bmi_ratio:.3} against the FNIH {sex_label} cut-off of {fnih_cutoff:.3}: {fnih_verdict} lean mass. \
Both indices assess muscle quantity only. Per EWGSOP2's case-finding algorithm, low muscle quantity confirms sarcopenia only alongside low muscle strength (grip strength or chair-stand test), and severity is graded separately by physical performance (for example gait speed or SPPB); neither is computed here. This result alone must not be used to diagnose or exclude sarcopenia.",
        ewgsop2_verdict = if low_muscle_quantity_ewgsop2 {
            "low"
        } else {
            "normal"
        },
        fnih_verdict = if low_lean_mass_fnih { "low" } else { "normal" },
    );

    Ok(SkeletalMuscleMassIndexOutcome {
        bmi,
        smi_kg_m2,
        low_muscle_quantity_ewgsop2,
        alm_bmi_ratio,
        low_lean_mass_fnih,
        interpretation,
    })
}

pub fn build_response(
    input: &SkeletalMuscleMassIndexInput,
) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let rounded_smi = round1(o.smi_kg_m2);

    let mut working = Map::new();
    working.insert(
        "appendicular_lean_mass_kg".into(),
        json!(input.appendicular_lean_mass_kg),
    );
    working.insert("height_cm".into(), json!(input.height_cm));
    working.insert("weight_kg".into(), json!(input.weight_kg));
    working.insert("sex".into(), json!(input.sex));
    working.insert("bmi_unrounded".into(), json!(o.bmi));
    working.insert("bmi".into(), json!(round1(o.bmi)));
    working.insert("smi_kg_m2_unrounded".into(), json!(o.smi_kg_m2));
    working.insert("smi_kg_m2".into(), json!(rounded_smi));
    working.insert(
        "low_muscle_quantity_ewgsop2".into(),
        json!(o.low_muscle_quantity_ewgsop2),
    );
    working.insert("alm_bmi_ratio_unrounded".into(), json!(o.alm_bmi_ratio));
    working.insert("alm_bmi_ratio".into(), json!(round3(o.alm_bmi_ratio)));
    working.insert("low_lean_mass_fnih".into(), json!(o.low_lean_mass_fnih));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(rounded_smi),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

pub struct SkeletalMuscleMassIndex;

impl Calculator for SkeletalMuscleMassIndex {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Skeletal Muscle Mass Index (SMI)"
    }

    fn description(&self) -> &'static str {
        "Appendicular lean mass normalised to height (EWGSOP2 SMI) and to BMI (FNIH ALM/BMI), each against its own consensus cut-off for low muscle quantity. Assesses muscle quantity only - not a standalone sarcopenia diagnosis, which also requires low muscle strength and, for severity, impaired physical performance."
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
            "title": "SkeletalMuscleMassIndexInput",
            "description": "Skeletal muscle mass index (EWGSOP2 SMI = ALM/height^2) and the FNIH ALM/BMI ratio, each against its own sex-specific low-muscle-quantity cut-off. Assesses muscle quantity only; sarcopenia diagnosis also requires low muscle strength and, for severity, impaired physical performance, neither of which this calculator computes.",
            "type": "object",
            "additionalProperties": false,
            "required": ["appendicular_lean_mass_kg", "height_cm", "weight_kg", "sex"],
            "properties": {
                "appendicular_lean_mass_kg": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 60,
                    "unit": "kg",
                    "description": "Appendicular lean mass (ALM / ASM): combined lean mass of both arms and both legs, from an independently obtained method (typically whole-body DXA)"
                },
                "height_cm": {
                    "type": "number",
                    "minimum": 100,
                    "maximum": 250,
                    "unit": "cm",
                    "description": "Standing height in centimetres"
                },
                "weight_kg": {
                    "type": "number",
                    "minimum": 20,
                    "maximum": 400,
                    "unit": "kg",
                    "description": "Total body weight in kilograms, used to derive BMI for the FNIH ALM/BMI ratio"
                },
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Sex, used to select the EWGSOP2 and FNIH cut-offs"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: SkeletalMuscleMassIndexInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc(alm: f64, height: f64, weight: f64, sex: Sex) -> SkeletalMuscleMassIndexInput {
        SkeletalMuscleMassIndexInput {
            appendicular_lean_mass_kg: alm,
            height_cm: height,
            weight_kg: weight,
            sex,
        }
    }

    #[test]
    fn equation_conformance_vector_male() {
        // ALM 25 kg, height 1.75 m -> height^2 = 3.0625.
        // SMI = 25 / 3.0625 = 8.163265...
        // BMI = 80 / 3.0625 = 26.122449...
        // ALM/BMI = 25 / 26.122449... = 0.957039...
        let o = compute(&calc(25.0, 175.0, 80.0, Sex::Male)).unwrap();
        assert!(
            (o.smi_kg_m2 - 8.163_265_306_122_45).abs() < 1e-9,
            "got {}",
            o.smi_kg_m2
        );
        assert!(
            (o.bmi - 26.122_448_979_591_84).abs() < 1e-9,
            "got {}",
            o.bmi
        );
        // ALM/BMI = 25 / (80/3.0625) = 25 * 3.0625 / 80 = 76.5625 / 80 = 0.95703125 exactly.
        assert!(
            (o.alm_bmi_ratio - 0.957_031_25).abs() < 1e-9,
            "got {}",
            o.alm_bmi_ratio
        );
        assert!(!o.low_muscle_quantity_ewgsop2);
        assert!(!o.low_lean_mass_fnih);
    }

    // ALM computed from the cut-off via the same height^2 arithmetic the
    // implementation uses, so the resulting SMI lands bit-for-bit on the
    // cut-off rather than drifting off it from an independently rounded
    // decimal literal (see the CHA2DS2-VASc boundary fix for why this
    // matters at floating-point equality).
    fn alm_at_smi_cutoff(cutoff: f64, height_cm: f64) -> f64 {
        cutoff * (height_cm / 100.0).powi(2)
    }

    #[test]
    fn ewgsop2_male_boundary() {
        let alm = alm_at_smi_cutoff(EWGSOP2_CUTOFF_MALE, 170.0);
        let at_cutoff = compute(&calc(alm, 170.0, 90.0, Sex::Male)).unwrap();
        assert_eq!(at_cutoff.smi_kg_m2, EWGSOP2_CUTOFF_MALE);
        assert!(!at_cutoff.low_muscle_quantity_ewgsop2);

        let below_cutoff = compute(&calc(20.0, 170.0, 90.0, Sex::Male)).unwrap();
        assert!(below_cutoff.smi_kg_m2 < 7.0);
        assert!(below_cutoff.low_muscle_quantity_ewgsop2);
    }

    #[test]
    fn ewgsop2_female_boundary() {
        let alm = alm_at_smi_cutoff(EWGSOP2_CUTOFF_FEMALE, 160.0);
        let at_cutoff = compute(&calc(alm, 160.0, 70.0, Sex::Female)).unwrap();
        assert_eq!(at_cutoff.smi_kg_m2, EWGSOP2_CUTOFF_FEMALE);
        assert!(!at_cutoff.low_muscle_quantity_ewgsop2);

        let below_cutoff = compute(&calc(13.5, 160.0, 70.0, Sex::Female)).unwrap();
        assert!(below_cutoff.smi_kg_m2 < 5.5);
        assert!(below_cutoff.low_muscle_quantity_ewgsop2);
    }

    #[test]
    fn fnih_cutoffs_flag_low_lean_mass() {
        // Male: BMI = 90 / 1.8^2 = 27.777...; ALM/BMI cutoff 0.789 -> boundary ALM = 0.789 * 27.777... = 21.9166...
        let o = compute(&calc(21.0, 180.0, 90.0, Sex::Male)).unwrap();
        assert!(o.alm_bmi_ratio < FNIH_CUTOFF_MALE);
        assert!(o.low_lean_mass_fnih);

        // Female: BMI = 70 / 1.65^2 = 25.711...; boundary ALM = 0.512 * 25.711... = 13.164...
        let f = compute(&calc(13.0, 165.0, 70.0, Sex::Female)).unwrap();
        assert!(f.alm_bmi_ratio < FNIH_CUTOFF_FEMALE);
        assert!(f.low_lean_mass_fnih);
    }

    #[test]
    fn accepts_boundary_inputs() {
        assert!(compute(&calc(2.0, 100.0, 20.0, Sex::Male)).is_ok());
        assert!(compute(&calc(60.0, 250.0, 400.0, Sex::Female)).is_ok());
    }

    #[test]
    fn rejects_out_of_range_alm() {
        assert!(compute(&calc(1.9, 175.0, 80.0, Sex::Male)).is_err());
        assert!(compute(&calc(60.1, 175.0, 80.0, Sex::Male)).is_err());
        assert!(compute(&calc(f64::NAN, 175.0, 80.0, Sex::Male)).is_err());
    }

    #[test]
    fn rejects_out_of_range_height() {
        assert!(compute(&calc(25.0, 99.9, 80.0, Sex::Male)).is_err());
        assert!(compute(&calc(25.0, 250.1, 80.0, Sex::Male)).is_err());
    }

    #[test]
    fn rejects_out_of_range_weight() {
        assert!(compute(&calc(25.0, 175.0, 19.9, Sex::Male)).is_err());
        assert!(compute(&calc(25.0, 175.0, 400.1, Sex::Male)).is_err());
    }

    #[test]
    fn rejects_alm_exceeding_weight() {
        assert!(compute(&calc(80.0, 175.0, 80.0, Sex::Male)).is_err());
        assert!(compute(&calc(85.0, 175.0, 80.0, Sex::Male)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "appendicular_lean_mass_kg": 25.0,
            "height_cm": 175.0,
            "weight_kg": 80.0,
            "sex": "male"
        });
        let dynamic = SkeletalMuscleMassIndex.calculate(&value).unwrap();
        let typed = build_response(&calc(25.0, 175.0, 80.0, Sex::Male)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn dynamic_surface_rejects_unknown_fields() {
        let invalid = json!({
            "appendicular_lean_mass_kg": 25.0,
            "height_cm": 175.0,
            "weight_kg": 80.0,
            "sex": "male",
            "unexpected": true
        });
        assert!(SkeletalMuscleMassIndex.calculate(&invalid).is_err());
    }

    #[test]
    fn response_preserves_inputs_and_equation_working() {
        let response = build_response(&calc(25.0, 175.0, 80.0, Sex::Male)).unwrap();
        assert_eq!(response.working["appendicular_lean_mass_kg"], json!(25.0));
        assert_eq!(response.working["height_cm"], json!(175.0));
        assert_eq!(response.working["weight_kg"], json!(80.0));
        assert_eq!(response.working["sex"], json!("male"));
        assert_eq!(response.result, response.working["smi_kg_m2"]);
        assert!(
            response
                .interpretation
                .contains("must not be used to diagnose or exclude sarcopenia")
        );
    }

    #[test]
    fn schema_documents_units_and_applicability_contract() {
        let schema = SkeletalMuscleMassIndex.input_schema();
        assert_eq!(
            schema["properties"]["appendicular_lean_mass_kg"]["unit"],
            json!("kg")
        );
        assert_eq!(schema["properties"]["height_cm"]["unit"], json!("cm"));
        assert_eq!(schema["properties"]["weight_kg"]["unit"], json!("kg"));
        assert_eq!(
            schema["properties"]["sex"]["enum"],
            json!(["male", "female"])
        );
        let description = schema["description"].as_str().unwrap();
        assert!(description.contains("muscle quantity only"));
    }
}
