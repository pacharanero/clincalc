// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Resistance-training protein intake from Morton et al. (2018).
//!
//! Morton et al. pooled 49 trials (1,863 participants) and examined whether
//! protein supplementation augmented resistance-training-induced gains in
//! fat-free mass and strength in healthy, non-energy-restricted adults. An
//! exploratory unadjusted breakpoint analysis of 42 study arms (723
//! participants) estimated a plateau at 1.62 g/kg body weight/day (95% CI 1.03
//! to 2.20; p=0.079; R^2=0.19). Because of that uncertainty, the authors stated
//! that it may be prudent to recommend approximately 2.2 g/kg/day to people
//! seeking to maximise resistance-training-induced gains in fat-free mass.
//!
//! The confidence interval is not a validated target range, and 2.2 g/kg/day
//! is not a safety ceiling. The source excluded energy restriction and did not
//! establish a lean-mass-retention target during a caloric deficit, a general
//! dietary requirement, or an individual optimum.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "protein_target";

pub const REFERENCE: &str = "Morton RW, Murphy KT, McKellar SR, et al. A systematic review, meta-analysis and meta-regression of the effect of protein supplementation on resistance training-induced gains in muscle mass and strength in healthy adults. Br J Sports Med. 2018;52(6):376-384. doi:10.1136/bjsports-2017-097608. Correction: Br J Sports Med. 2020;54(19):e7. doi:10.1136/bjsports-2017-097608corr1.";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published mathematical method - independently implemented; formulas and algorithms are not protected by US copyright",
    source_url: "https://www.copyright.gov/circs/circ31.pdf",
};

const BREAKPOINT_G_PER_KG: f64 = 1.62;
const BREAKPOINT_CI_LOW_G_PER_KG: f64 = 1.03;
const BREAKPOINT_CI_HIGH_G_PER_KG: f64 = 2.20;
const BREAKPOINT_P_VALUE: f64 = 0.079;
const BREAKPOINT_R_SQUARED: f64 = 0.19;
const BREAKPOINT_STUDY_ARMS: u32 = 42;
const BREAKPOINT_PARTICIPANTS: u32 = 723;
const STUDIED_INTAKE_MIN_G_PER_KG: f64 = 0.9;
const STUDIED_INTAKE_MAX_G_PER_KG: f64 = 2.4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceContext {
    HealthyAdultNotEnergyRestrictedResistanceTraining,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProteinTargetInput {
    /// Attestation that the person matches the source population and training context.
    pub evidence_context: EvidenceContext,
    /// Age in completed years.
    pub age_years: u32,
    /// Total body weight in kilograms.
    pub weight_kg: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProteinTargetOutcome {
    /// The source authors' prudent recommendation, using the upper breakpoint CI.
    pub prudent_amount_g_per_day: f64,
    pub breakpoint_estimate_g_per_day: f64,
    pub breakpoint_ci_low_g_per_day: f64,
    pub breakpoint_ci_high_g_per_day: f64,
}

pub fn compute(input: &ProteinTargetInput) -> Result<ProteinTargetOutcome, CalcError> {
    if !(18..=120).contains(&input.age_years) {
        return Err(CalcError::InvalidInput(
            "age_years must be between 18 and 120; the source studied adults, and 120 is a broad input-safety guard"
                .into(),
        ));
    }
    if !(20.0..=300.0).contains(&input.weight_kg) || !input.weight_kg.is_finite() {
        return Err(CalcError::InvalidInput(
            "weight_kg must be finite and between 20 and 300; this is a broad input-safety guard, not a source-study eligibility range"
                .into(),
        ));
    }

    Ok(ProteinTargetOutcome {
        prudent_amount_g_per_day: input.weight_kg * BREAKPOINT_CI_HIGH_G_PER_KG,
        breakpoint_estimate_g_per_day: input.weight_kg * BREAKPOINT_G_PER_KG,
        breakpoint_ci_low_g_per_day: input.weight_kg * BREAKPOINT_CI_LOW_G_PER_KG,
        breakpoint_ci_high_g_per_day: input.weight_kg * BREAKPOINT_CI_HIGH_G_PER_KG,
    })
}

pub fn build_response(input: &ProteinTargetInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let prudent_amount = round1(outcome.prudent_amount_g_per_day);

    let mut working = Map::new();
    working.insert("evidence_context".into(), json!(input.evidence_context));
    working.insert("age_years".into(), json!(input.age_years));
    working.insert("weight_kg".into(), json!(input.weight_kg));
    working.insert(
        "prudent_amount_g_per_kg".into(),
        json!(BREAKPOINT_CI_HIGH_G_PER_KG),
    );
    working.insert("prudent_amount_g_per_day".into(), json!(prudent_amount));
    working.insert("breakpoint_g_per_kg".into(), json!(BREAKPOINT_G_PER_KG));
    working.insert(
        "breakpoint_estimate_g_per_day".into(),
        json!(round1(outcome.breakpoint_estimate_g_per_day)),
    );
    working.insert(
        "breakpoint_ci_g_per_kg".into(),
        json!([BREAKPOINT_CI_LOW_G_PER_KG, BREAKPOINT_CI_HIGH_G_PER_KG]),
    );
    working.insert(
        "breakpoint_ci_g_per_day".into(),
        json!([
            round1(outcome.breakpoint_ci_low_g_per_day),
            round1(outcome.breakpoint_ci_high_g_per_day)
        ]),
    );
    working.insert("breakpoint_p_value".into(), json!(BREAKPOINT_P_VALUE));
    working.insert("breakpoint_r_squared".into(), json!(BREAKPOINT_R_SQUARED));
    working.insert("breakpoint_study_arms".into(), json!(BREAKPOINT_STUDY_ARMS));
    working.insert(
        "breakpoint_participants".into(),
        json!(BREAKPOINT_PARTICIPANTS),
    );
    working.insert(
        "studied_total_intake_g_per_kg".into(),
        json!([STUDIED_INTAKE_MIN_G_PER_KG, STUDIED_INTAKE_MAX_G_PER_KG]),
    );
    working.insert("result_metric".into(), json!("prudent_amount_g_per_day"));

    let interpretation = format!(
        "Morton et al.'s evidence-derived prudent protein amount is approximately {prudent_amount:.1} g/day ({BREAKPOINT_CI_HIGH_G_PER_KG:.1} g/kg total body weight/day) for a healthy, non-energy-restricted adult undertaking resistance training. The authors proposed approximately 2.2 g/kg/day because their exploratory unadjusted breakpoint estimate was {BREAKPOINT_G_PER_KG:.2} g/kg/day ({breakpoint:.1} g/day here), with a wide 95% CI of {BREAKPOINT_CI_LOW_G_PER_KG:.2}-{BREAKPOINT_CI_HIGH_G_PER_KG:.2} g/kg/day, p={BREAKPOINT_P_VALUE:.3}, and R^2={BREAKPOINT_R_SQUARED:.2}. The confidence interval is not a validated 1.6-2.2 g/kg/day target range, and 2.2 g/kg/day is not a safety ceiling. The source included resistance training at least twice weekly for at least six weeks and studied total protein intake from 0.9 to 2.4 g/kg/day. It did not establish an individual optimum, a general dietary minimum, a lean-mass-retention target during a caloric deficit, or efficacy or safety above the studied range. This calculator is not for children, pregnancy, renal or hepatic disease, eating disorders, serious illness, or other circumstances requiring individual medical or dietetic assessment.",
        breakpoint = round1(outcome.breakpoint_estimate_g_per_day),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(prudent_amount),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

pub struct ProteinTarget;

impl Calculator for ProteinTarget {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Resistance-Training Protein Intake"
    }

    fn description(&self) -> &'static str {
        "Calculates Morton et al.'s prudent 2.2 g/kg/day protein amount for healthy, non-energy-restricted adults undertaking resistance training and reports the uncertain exploratory breakpoint behind it."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        let source = json!({
            "citation": REFERENCE,
            "url": "https://doi.org/10.1136/bjsports-2017-097608"
        });
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "ProteinTargetInput",
            "description": "Calculates Morton et al.'s prudent protein amount of approximately 2.2 g/kg total body weight/day for healthy, non-energy-restricted adults undertaking resistance training. Reports the exploratory 1.62 g/kg/day breakpoint and its 95% CI separately; the CI is not a target range and 2.2 g/kg/day is not a safety ceiling.",
            "type": "object",
            "additionalProperties": false,
            "required": ["evidence_context", "age_years", "weight_kg"],
            "properties": {
                "evidence_context": {
                    "type": "string",
                    "const": "healthy_adult_not_energy_restricted_resistance_training",
                    "unit": "none",
                    "description": "Attestation that the person matches the source population and resistance-training context",
                    "definition": {
                        "concept": "Morton 2018 protein meta-analysis evidence context",
                        "statement": "Select this value only for a healthy adult who is not energy restricted and is undertaking resistance training at least twice weekly for at least six weeks.",
                        "includes": ["Age 18 years or older", "Healthy source population", "Not energy restricted", "Resistance training at least twice weekly for at least six weeks"],
                        "excludes": ["Children", "Caloric deficit or other energy restriction", "Pregnancy", "Renal or hepatic disease", "Eating disorders", "Serious illness", "No qualifying resistance-training programme"],
                        "source": source,
                        "snomedEcl": null,
                        "refset": null,
                        "caveats": "The source studied total protein intake from 0.9 to 2.4 g/kg/day. It did not establish an individual optimum, a general dietary requirement, a lean-mass-retention target during energy restriction, or efficacy or safety above the studied range.",
                        "status": "draft"
                    }
                },
                "age_years": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 120,
                    "unit": "years",
                    "description": "Age in completed years. The source studied adults; 120 is a broad input-safety guard, not a source-study upper age limit."
                },
                "weight_kg": {
                    "type": "number",
                    "minimum": 20,
                    "maximum": 300,
                    "unit": "kg",
                    "description": "Current total body weight in kilograms. Do not substitute lean body mass. The 20-300 kg range is a broad input-safety guard, not a source-study eligibility range."
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: ProteinTargetInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(age_years: u32, weight_kg: f64) -> ProteinTargetInput {
        ProteinTargetInput {
            evidence_context: EvidenceContext::HealthyAdultNotEnergyRestrictedResistanceTraining,
            age_years,
            weight_kg,
        }
    }

    #[test]
    fn source_constants_and_weight_conversion() {
        // Morton et al.: breakpoint 1.62 g/kg/day (95% CI 1.03-2.20);
        // the authors state that ~2.2 g/kg/day may be a prudent recommendation.
        let outcome = compute(&input(35, 70.0)).unwrap();
        assert!((outcome.prudent_amount_g_per_day - 154.0).abs() < 1e-9);
        assert!((outcome.breakpoint_estimate_g_per_day - 113.4).abs() < 1e-9);
        assert!((outcome.breakpoint_ci_low_g_per_day - 72.1).abs() < 1e-9);
        assert!((outcome.breakpoint_ci_high_g_per_day - 154.0).abs() < 1e-9);
    }

    #[test]
    fn accepts_boundary_inputs() {
        assert!(compute(&input(18, 20.0)).is_ok());
        assert!(compute(&input(120, 300.0)).is_ok());
    }

    #[test]
    fn rejects_out_of_population_age() {
        assert!(compute(&input(17, 70.0)).is_err());
        assert!(compute(&input(121, 70.0)).is_err());
    }

    #[test]
    fn rejects_out_of_range_or_non_finite_weight() {
        assert!(compute(&input(35, 19.9)).is_err());
        assert!(compute(&input(35, 300.1)).is_err());
        assert!(compute(&input(35, f64::NAN)).is_err());
        assert!(compute(&input(35, f64::INFINITY)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "evidence_context": "healthy_adult_not_energy_restricted_resistance_training",
            "age_years": 35,
            "weight_kg": 90.0
        });
        let dynamic = ProteinTarget.calculate(&value).unwrap();
        let typed = build_response(&input(35, 90.0)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn dynamic_surface_rejects_unknown_fields_and_wrong_context() {
        let unknown = json!({
            "evidence_context": "healthy_adult_not_energy_restricted_resistance_training",
            "age_years": 35,
            "weight_kg": 90.0,
            "unexpected": true
        });
        assert!(ProteinTarget.calculate(&unknown).is_err());

        let wrong_context = json!({
            "evidence_context": "caloric_deficit",
            "age_years": 35,
            "weight_kg": 90.0
        });
        assert!(ProteinTarget.calculate(&wrong_context).is_err());
    }

    #[test]
    fn response_separates_recommendation_from_breakpoint_uncertainty() {
        let response = build_response(&input(35, 70.0)).unwrap();
        assert_eq!(response.result, json!(154.0));
        assert_eq!(
            response.working["result_metric"],
            json!("prudent_amount_g_per_day")
        );
        assert_eq!(
            response.working["breakpoint_estimate_g_per_day"],
            json!(113.4)
        );
        assert_eq!(
            response.working["breakpoint_ci_g_per_kg"],
            json!([1.03, 2.2])
        );
        assert_eq!(response.working["breakpoint_p_value"], json!(0.079));
        assert_eq!(response.working["breakpoint_study_arms"], json!(42));
        assert_eq!(
            response.working["studied_total_intake_g_per_kg"],
            json!([0.9, 2.4])
        );
        assert!(response.interpretation.contains("not a validated 1.6-2.2"));
        assert!(response.interpretation.contains("not a safety ceiling"));
        assert!(response.interpretation.contains("caloric deficit"));
    }

    #[test]
    fn schema_enforces_and_defines_source_context() {
        let schema = ProteinTarget.input_schema();
        assert_eq!(
            schema["required"],
            json!(["evidence_context", "age_years", "weight_kg"])
        );
        assert_eq!(schema["properties"]["age_years"]["minimum"], json!(18));
        assert_eq!(schema["properties"]["weight_kg"]["unit"], json!("kg"));
        let context = &schema["properties"]["evidence_context"];
        assert_eq!(
            context["const"],
            json!("healthy_adult_not_energy_restricted_resistance_training")
        );
        assert!(
            context["definition"]["statement"]
                .as_str()
                .unwrap()
                .contains("at least twice weekly for at least six weeks")
        );
    }
}
