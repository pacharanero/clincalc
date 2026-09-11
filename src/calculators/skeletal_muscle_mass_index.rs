// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Skeletal muscle mass index (SMI) and the FNIH lean-mass ratio.
//!
//! Two height/BMI-normalised indices of appendicular lean mass (ALM, also
//! called appendicular skeletal muscle mass, ASM: the combined lean mass of
//! both arms and both legs, measured by whole-body DXA), each
//! with its own consensus cut-off for "low muscle quantity":
//!
//! - **EWGSOP2** (European Working Group on Sarcopenia in Older People 2):
//!   SMI = ALM (kg) / height (m)^2. Low muscle quantity: <7.0 kg/m^2 (men),
//!   <5.5 kg/m^2 (women).
//! - **FNIH** (Foundation for the National Institutes of Health Sarcopenia
//!   Project): ALM/BMI = ALM (kg) / BMI (kg/m^2). Low lean mass: <0.789
//!   (men), <0.512 (women).
//!
//! Both are reported separately because they use different denominators and
//! can classify the same measurement differently. EWGSOP2 SMI is the primary
//! result; the FNIH ALM/BMI candidate criterion is supplementary.
//!
//! **This calculator assesses muscle quantity only.** Per EWGSOP2's own
//! case-finding algorithm, low muscle quantity confirms sarcopenia only in
//! combination with low muscle strength (grip strength or chair-stand test,
//! not computed here), and severity is graded separately by physical
//! performance (gait speed, SPPB, or similar, also not computed here). A
//! below- or not-below-cut-off result from this calculator alone must not be
//! used to diagnose or exclude sarcopenia.
//!
//! References: Cruz-Jentoft AJ et al. Age Ageing. 2019;48(1):16-31.
//! doi:10.1093/ageing/afy169 (corrected by doi:10.1093/ageing/afz046). |
//! Cawthon PM et al. J Gerontol A Biol Sci Med Sci. 2014;69(5):567-575.
//! doi:10.1093/gerona/glu023. | McLean RR et al. J Gerontol A Biol Sci Med
//! Sci. 2014;69(5):576-583. doi:10.1093/gerona/glu012.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "skeletal_muscle_mass_index";

pub const REFERENCE: &str = "Cruz-Jentoft AJ, Bahat G, Bauer J, et al. Sarcopenia: revised European consensus on definition and diagnosis. Age Ageing. 2019;48(1):16-31. doi:10.1093/ageing/afy169; correction doi:10.1093/ageing/afz046. | Cawthon PM, Peters KW, Shardell MD, et al. Cutpoints for low appendicular lean mass that identify older adults with clinically significant weakness. J Gerontol A Biol Sci Med Sci. 2014;69(5):567-575. doi:10.1093/gerona/glu023. | McLean RR, Shardell MD, Alley DE, et al. Criteria for clinically relevant weakness and low lean mass and their longitudinal association with incident mobility impairment and mortality. J Gerontol A Biol Sci Med Sci. 2014;69(5):576-583. doi:10.1093/gerona/glu012.";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlmMeasurementMethod {
    WholeBodyDxa,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletalMuscleMassIndexInput {
    /// Age in completed years. The combined calculator is restricted to the
    /// FNIH cohorts' older-adult population.
    pub age_years: u32,
    /// Appendicular lean mass (ALM / ASM): combined lean mass of both arms
    /// and both legs, in kilograms, measured by whole-body DXA.
    pub appendicular_lean_mass_kg: f64,
    /// Objectively measured standing height in centimetres.
    pub height_cm: f64,
    /// Objectively measured weight in kilograms, contemporaneous with the DXA
    /// measurement and used to derive BMI for the FNIH ratio.
    pub weight_kg: f64,
    pub sex: Sex,
    pub alm_measurement_method: AlmMeasurementMethod,
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
    if input.age_years < 65 || input.age_years > 120 {
        return Err(CalcError::InvalidInput(
            "age_years must be between 65 and 120; the FNIH criteria were developed and validated in adults aged 65 or older, and 120 is a broad input-safety guard"
                .into(),
        ));
    }
    if !(2.0..=60.0).contains(&input.appendicular_lean_mass_kg)
        || !input.appendicular_lean_mass_kg.is_finite()
    {
        return Err(CalcError::InvalidInput(
            "appendicular_lean_mass_kg must be finite and between 2 and 60; this is a broad input-safety guard, not a source-study eligibility range".into(),
        ));
    }
    if !(100.0..=250.0).contains(&input.height_cm) || !input.height_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "height_cm must be finite and between 100 and 250; this is a broad input-safety guard, not a source-study eligibility range".into(),
        ));
    }
    if !(20.0..=400.0).contains(&input.weight_kg) || !input.weight_kg.is_finite() {
        return Err(CalcError::InvalidInput(
            "weight_kg must be finite and between 20 and 400; this is a broad input-safety guard, not a source-study eligibility range".into(),
        ));
    }
    if input.appendicular_lean_mass_kg >= input.weight_kg {
        return Err(CalcError::InvalidInput(
            "appendicular_lean_mass_kg must be less than total body weight".into(),
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
        "EWGSOP2 SMI (ALM/height^2) {smi_kg_m2:.1} kg/m^2: the unrounded value is {ewgsop2_verdict} the {sex_label} cut-off of {ewgsop2_cutoff:.1} kg/m^2. \
FNIH ALM/BMI ratio {alm_bmi_ratio:.3}: the unrounded value is {fnih_verdict} the {sex_label} cut-off of {fnih_cutoff:.3}. \
The classifications are reported separately because the methods can disagree; neither supersedes the other or defines an overall result. Both assess muscle quantity only. Per EWGSOP2, low muscle quantity confirms sarcopenia only alongside low muscle strength (grip strength or chair-stand test), and severity is graded separately by physical performance (for example gait speed or SPPB); neither is computed here. The FNIH cut-offs are preliminary candidate criteria derived and validated in mostly community-dwelling adults aged 65 or older. This result alone must not be used to diagnose or exclude sarcopenia.",
        ewgsop2_verdict = if low_muscle_quantity_ewgsop2 {
            "below"
        } else {
            "not below"
        },
        fnih_verdict = if low_lean_mass_fnih {
            "below"
        } else {
            "not below"
        },
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
    working.insert("age_years".into(), json!(input.age_years));
    working.insert(
        "appendicular_lean_mass_kg".into(),
        json!(input.appendicular_lean_mass_kg),
    );
    working.insert("height_cm".into(), json!(input.height_cm));
    working.insert("weight_kg".into(), json!(input.weight_kg));
    working.insert("sex".into(), json!(input.sex));
    working.insert(
        "alm_measurement_method".into(),
        json!(input.alm_measurement_method),
    );
    working.insert("bmi_unrounded".into(), json!(o.bmi));
    working.insert("bmi".into(), json!(round1(o.bmi)));
    working.insert("smi_kg_m2_unrounded".into(), json!(o.smi_kg_m2));
    working.insert("smi_kg_m2".into(), json!(rounded_smi));
    working.insert(
        "low_muscle_quantity_ewgsop2".into(),
        json!(o.low_muscle_quantity_ewgsop2),
    );
    working.insert(
        "ewgsop2_cutoff_kg_m2".into(),
        json!(match input.sex {
            Sex::Male => EWGSOP2_CUTOFF_MALE,
            Sex::Female => EWGSOP2_CUTOFF_FEMALE,
        }),
    );
    working.insert("alm_bmi_ratio_unrounded".into(), json!(o.alm_bmi_ratio));
    working.insert("alm_bmi_ratio".into(), json!(round3(o.alm_bmi_ratio)));
    working.insert("low_lean_mass_fnih".into(), json!(o.low_lean_mass_fnih));
    working.insert(
        "fnih_cutoff".into(),
        json!(match input.sex {
            Sex::Male => FNIH_CUTOFF_MALE,
            Sex::Female => FNIH_CUTOFF_FEMALE,
        }),
    );
    working.insert("result_metric".into(), json!("ewgsop2_smi_kg_m2"));

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
        "Whole-body DXA appendicular lean mass normalised to height (primary EWGSOP2 SMI result) and to BMI (supplementary FNIH candidate criterion) in adults aged 65 or older. Assesses muscle quantity only - not a standalone sarcopenia diagnosis or exclusion."
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
            "description": "Whole-body DXA appendicular lean mass normalised to height (primary EWGSOP2 SMI result) and to BMI (supplementary FNIH preliminary candidate criterion) in adults aged 65 or older. The methods can disagree and are reported separately. Assesses muscle quantity only; this result cannot diagnose or exclude sarcopenia.",
            "type": "object",
            "additionalProperties": false,
            "required": ["age_years", "appendicular_lean_mass_kg", "height_cm", "weight_kg", "sex", "alm_measurement_method"],
            "properties": {
                "age_years": {
                    "type": "integer",
                    "minimum": 65,
                    "maximum": 120,
                    "description": "Age in completed years. The FNIH criteria were developed and validated in adults aged 65 or older; 120 is a broad input-safety guard, not a source-study upper age limit."
                },
                "appendicular_lean_mass_kg": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 60,
                    "unit": "kg",
                    "description": "Appendicular lean mass (ALM / ASM): combined lean mass of both arms and both legs measured by whole-body DXA. The 2-60 kg range is a broad input-safety guard, not a source-study eligibility range. Do not substitute BIA-, CT-, MRI-, ultrasound-, or anthropometry-derived estimates for these cut-offs."
                },
                "height_cm": {
                    "type": "number",
                    "minimum": 100,
                    "maximum": 250,
                    "unit": "cm",
                    "description": "Objectively measured standing height in centimetres. The 100-250 cm range is a broad input-safety guard, not a source-study eligibility range."
                },
                "weight_kg": {
                    "type": "number",
                    "minimum": 20,
                    "maximum": 400,
                    "unit": "kg",
                    "description": "Objectively measured total body weight in kilograms, contemporaneous with the DXA measurement and used to derive BMI for the FNIH ALM/BMI ratio. The 20-400 kg range is a broad input-safety guard, not a source-study eligibility range."
                },
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Source-publication cut-off category. The cited groups provide male and female cut-offs only; this calculator cannot classify another or unknown category."
                },
                "alm_measurement_method": {
                    "type": "string",
                    "enum": ["whole_body_dxa"],
                    "description": "Required measurement-method attestation. FNIH ALM/BMI cut-offs were derived and validated using appendicular lean mass from whole-body DXA; values from other modalities are not interchangeable."
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
            age_years: 75,
            appendicular_lean_mass_kg: alm,
            height_cm: height,
            weight_kg: weight,
            sex,
            alm_measurement_method: AlmMeasurementMethod::WholeBodyDxa,
        }
    }

    #[test]
    fn equation_conformance_vector_male() {
        // ALM 25 kg, height 1.75 m -> height^2 = 3.0625.
        // SMI = 25 / 3.0625 = 8.163265...
        // BMI = 80 / 3.0625 = 26.122449...
        // ALM/BMI = 25 / 26.122449... = 0.95703125.
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
    // decimal literal.
    fn alm_at_smi_cutoff(cutoff: f64, height_cm: f64) -> f64 {
        cutoff * (height_cm / 100.0).powi(2)
    }

    #[test]
    fn ewgsop2_male_boundary() {
        // EWGSOP2 corrected Table 3: low ASM/height^2 <7.0 kg/m^2 for men.
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
        // EWGSOP2 corrected Table 3: low ASM/height^2 <5.5 kg/m^2 for women.
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
    fn fnih_male_boundary_is_strictly_less_than() {
        // Cawthon et al 2014 (doi:10.1093/gerona/glu023): low ALM/BMI <0.789.
        let height_cm: f64 = 180.0;
        let weight_kg = 90.0;
        let bmi = weight_kg / (height_cm / 100.0).powi(2);
        let alm_at_cutoff = FNIH_CUTOFF_MALE * bmi;

        let at_cutoff = compute(&calc(alm_at_cutoff, height_cm, weight_kg, Sex::Male)).unwrap();
        assert_eq!(at_cutoff.alm_bmi_ratio, FNIH_CUTOFF_MALE);
        assert!(!at_cutoff.low_lean_mass_fnih);

        let below = compute(&calc(
            alm_at_cutoff - 0.000_001,
            height_cm,
            weight_kg,
            Sex::Male,
        ))
        .unwrap();
        assert!(below.low_lean_mass_fnih);
    }

    #[test]
    fn fnih_female_boundary_is_strictly_less_than() {
        // Cawthon et al 2014 (doi:10.1093/gerona/glu023): low ALM/BMI <0.512.
        let height_cm: f64 = 160.0;
        let weight_kg = 70.0;
        let bmi = weight_kg / (height_cm / 100.0).powi(2);
        let alm_at_cutoff = FNIH_CUTOFF_FEMALE * bmi;

        let at_cutoff = compute(&calc(alm_at_cutoff, height_cm, weight_kg, Sex::Female)).unwrap();
        assert_eq!(at_cutoff.alm_bmi_ratio, FNIH_CUTOFF_FEMALE);
        assert!(!at_cutoff.low_lean_mass_fnih);

        let below = compute(&calc(
            alm_at_cutoff - 0.000_001,
            height_cm,
            weight_kg,
            Sex::Female,
        ))
        .unwrap();
        assert!(below.low_lean_mass_fnih);
    }

    #[test]
    fn reports_discordant_classifications_without_combining_them() {
        let ewgsop2_low_fnih_not_low = compute(&calc(19.0, 170.0, 60.0, Sex::Male)).unwrap();
        assert!(ewgsop2_low_fnih_not_low.low_muscle_quantity_ewgsop2);
        assert!(!ewgsop2_low_fnih_not_low.low_lean_mass_fnih);
        assert!(
            ewgsop2_low_fnih_not_low
                .interpretation
                .contains("methods can disagree")
        );

        let ewgsop2_not_low_fnih_low = compute(&calc(21.0, 170.0, 90.0, Sex::Male)).unwrap();
        assert!(!ewgsop2_not_low_fnih_low.low_muscle_quantity_ewgsop2);
        assert!(ewgsop2_not_low_fnih_low.low_lean_mass_fnih);
    }

    #[test]
    fn never_labels_a_value_as_normal() {
        let o = compute(&calc(25.0, 175.0, 80.0, Sex::Male)).unwrap();
        assert!(!o.interpretation.to_lowercase().contains("normal"));
        assert!(o.interpretation.contains("not below"));
    }

    #[test]
    fn rounded_display_does_not_hide_unrounded_cutoff_classification() {
        let response = build_response(&calc(6.96, 100.0, 50.0, Sex::Male)).unwrap();
        assert_eq!(response.result, json!(7.0));
        assert_eq!(response.working["low_muscle_quantity_ewgsop2"], true);
        assert!(response.interpretation.contains("unrounded value is below"));
    }

    #[test]
    fn accepts_boundary_inputs() {
        assert!(compute(&calc(2.0, 100.0, 20.0, Sex::Male)).is_ok());
        assert!(compute(&calc(60.0, 250.0, 400.0, Sex::Female)).is_ok());
    }

    #[test]
    fn restricts_fnih_classification_to_older_adults() {
        let mut input = calc(25.0, 175.0, 80.0, Sex::Male);
        input.age_years = 64;
        assert!(compute(&input).is_err());
        input.age_years = 65;
        assert!(compute(&input).is_ok());
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
        assert!(compute(&calc(40.0, 175.0, 40.0, Sex::Male)).is_err());
        assert!(compute(&calc(41.0, 175.0, 40.0, Sex::Male)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age_years": 75,
            "appendicular_lean_mass_kg": 25.0,
            "height_cm": 175.0,
            "weight_kg": 80.0,
            "sex": "male",
            "alm_measurement_method": "whole_body_dxa"
        });
        let dynamic = SkeletalMuscleMassIndex.calculate(&value).unwrap();
        let typed = build_response(&calc(25.0, 175.0, 80.0, Sex::Male)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn dynamic_surface_rejects_unknown_fields() {
        let invalid = json!({
            "age_years": 75,
            "appendicular_lean_mass_kg": 25.0,
            "height_cm": 175.0,
            "weight_kg": 80.0,
            "sex": "male",
            "alm_measurement_method": "whole_body_dxa",
            "unexpected": true
        });
        assert!(SkeletalMuscleMassIndex.calculate(&invalid).is_err());
    }

    #[test]
    fn response_preserves_inputs_and_equation_working() {
        let response = build_response(&calc(25.0, 175.0, 80.0, Sex::Male)).unwrap();
        assert_eq!(response.working["age_years"], json!(75));
        assert_eq!(response.working["appendicular_lean_mass_kg"], json!(25.0));
        assert_eq!(response.working["height_cm"], json!(175.0));
        assert_eq!(response.working["weight_kg"], json!(80.0));
        assert_eq!(response.working["sex"], json!("male"));
        assert_eq!(
            response.working["alm_measurement_method"],
            json!("whole_body_dxa")
        );
        assert_eq!(response.result, response.working["smi_kg_m2"]);
        assert_eq!(
            response.working["result_metric"],
            json!("ewgsop2_smi_kg_m2")
        );
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
        assert_eq!(schema["properties"]["age_years"]["minimum"], json!(65));
        assert_eq!(
            schema["properties"]["sex"]["enum"],
            json!(["male", "female"])
        );
        assert_eq!(
            schema["properties"]["alm_measurement_method"]["enum"],
            json!(["whole_body_dxa"])
        );
        let description = schema["description"].as_str().unwrap();
        assert!(description.contains("muscle quantity only"));
        assert!(description.contains("cannot diagnose or exclude sarcopenia"));
        let alm_description = schema["properties"]["appendicular_lean_mass_kg"]["description"]
            .as_str()
            .unwrap();
        assert!(alm_description.contains("whole-body DXA"));
        assert!(alm_description.contains("Do not substitute"));
    }
}
