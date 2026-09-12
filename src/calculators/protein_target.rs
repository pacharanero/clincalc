// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Resistance-training protein target.
//!
//! Morton et al (2018) pooled 49 studies (1863 participants) and used
//! meta-regression to model total dietary protein intake (g/kg total body
//! weight/day) against resistance-training-induced gains in fat-free mass.
//! The model's break point - the intake beyond which additional protein
//! produced no further fat-free-mass gain - was 1.62 g/kg/day (95% CI 1.03 to
//! 2.20 g/kg/day). This calculator reports that finding as a practical range:
//! ~1.6 g/kg/day as the point estimate, up to ~2.2 g/kg/day (the upper
//! confidence bound) as a ceiling with no established added benefit above it.
//!
//! This is a single evidence-based range for one population and one goal:
//! healthy adults undertaking resistance training who want to maximise (or,
//! by extension, maintain) training-induced fat-free mass. It is not a
//! general dietary minimum, not validated specifically for a caloric deficit,
//! not adjusted for lean body mass (the model used total body weight), and
//! not intended for renal or hepatic disease, where protein intake requires
//! individualised medical/dietetic advice.
//!
//! Reference: Morton RW, Murphy KT, McKellar SR, et al. A systematic review,
//! meta-analysis and meta-regression of the effect of protein supplementation
//! on resistance training-induced gains in muscle mass and strength in
//! healthy adults. Br J Sports Med. 2018;52(6):376-384.
//! doi:10.1136/bjsports-2017-097608.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "protein_target";

pub const REFERENCE: &str = "Morton RW, Murphy KT, McKellar SR, et al. A systematic review, meta-analysis and meta-regression of the effect of protein supplementation on resistance training-induced gains in muscle mass and strength in healthy adults. Br J Sports Med. 2018;52(6):376-384. doi:10.1136/bjsports-2017-097608.";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published mathematical method - independently implemented; formulas and algorithms are not protected by US copyright",
    source_url: "https://www.copyright.gov/circs/circ31.pdf",
};

/// Break point of the Morton 2018 meta-regression, in g protein/kg body weight/day.
const BREAK_POINT_G_PER_KG: f64 = 1.62;
/// Practical lower bound of the range (rounded break point).
const LOW_G_PER_KG: f64 = 1.6;
/// Practical upper bound: the break point's 95% CI upper bound.
const HIGH_G_PER_KG: f64 = 2.2;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProteinTargetInput {
    /// Total body weight in kilograms.
    pub weight_kg: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProteinTargetOutcome {
    pub low_g_per_day: f64,
    pub high_g_per_day: f64,
    pub break_point_g_per_day: f64,
}

pub fn compute(input: &ProteinTargetInput) -> Result<ProteinTargetOutcome, CalcError> {
    if !(20.0..=300.0).contains(&input.weight_kg) || !input.weight_kg.is_finite() {
        return Err(CalcError::InvalidInput(
            "weight_kg must be finite and between 20 and 300".into(),
        ));
    }

    Ok(ProteinTargetOutcome {
        low_g_per_day: input.weight_kg * LOW_G_PER_KG,
        high_g_per_day: input.weight_kg * HIGH_G_PER_KG,
        break_point_g_per_day: input.weight_kg * BREAK_POINT_G_PER_KG,
    })
}

pub fn build_response(input: &ProteinTargetInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let low = round1(o.low_g_per_day);
    let high = round1(o.high_g_per_day);
    let break_point = round1(o.break_point_g_per_day);

    let mut working = Map::new();
    working.insert("weight_kg".into(), json!(input.weight_kg));
    working.insert("low_g_per_kg".into(), json!(LOW_G_PER_KG));
    working.insert("high_g_per_kg".into(), json!(HIGH_G_PER_KG));
    working.insert("break_point_g_per_kg".into(), json!(BREAK_POINT_G_PER_KG));
    working.insert("low_g_per_day".into(), json!(low));
    working.insert("high_g_per_day".into(), json!(high));
    working.insert("break_point_g_per_day".into(), json!(break_point));

    let interpretation = format!(
        "Resistance-training protein target {low:.0}-{high:.0} g/day ({LOW_G_PER_KG:.1}-{HIGH_G_PER_KG:.1} g/kg body weight/day), from a total body weight of {weight:.1} kg. Morton et al's 2018 meta-regression across 49 studies (1863 participants) found a break point of {BREAK_POINT_G_PER_KG:.2} g/kg/day ({break_point:.0} g/day here), 95% CI 1.03 to 2.20 g/kg/day - the total daily protein intake beyond which no further resistance-training-induced gain in fat-free mass was observed. This range applies to healthy adults undertaking resistance training aiming to maximise or maintain training-induced fat-free mass; it is not a general dietary protein minimum, is not lean-body-mass adjusted (the model used total body weight), and is not validated specifically for a caloric deficit. Individualise further for renal or hepatic disease, pregnancy, or paediatric care.",
        weight = input.weight_kg
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(format!("{low:.0}-{high:.0} g/day")),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct ProteinTarget;

impl Calculator for ProteinTarget {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Resistance-Training Protein Target"
    }

    fn description(&self) -> &'static str {
        "Daily protein target (1.6-2.2 g/kg body weight/day) for healthy adults undertaking resistance training, from Morton et al 2018's meta-regression break point for training-induced fat-free-mass gain. Not a general dietary minimum and not validated specifically for a caloric deficit."
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
            "title": "ProteinTargetInput",
            "description": "Daily protein target for a healthy adult undertaking resistance training, from the Morton et al 2018 meta-regression break point (1.6-2.2 g/kg total body weight/day) for training-induced fat-free-mass gain. Not a general dietary minimum, not lean-body-mass adjusted, and not validated specifically for a caloric deficit.",
            "type": "object",
            "additionalProperties": false,
            "required": ["weight_kg"],
            "properties": {
                "weight_kg": {
                    "type": "number",
                    "minimum": 20,
                    "maximum": 300,
                    "unit": "kg",
                    "description": "Total body weight in kilograms"
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

    #[test]
    fn equation_conformance_vector() {
        // 70 kg: low 112.0, high 154.0, break point 113.4 g/day.
        let o = compute(&ProteinTargetInput { weight_kg: 70.0 }).unwrap();
        assert!(
            (o.low_g_per_day - 112.0).abs() < 1e-9,
            "got {}",
            o.low_g_per_day
        );
        assert!(
            (o.high_g_per_day - 154.0).abs() < 1e-9,
            "got {}",
            o.high_g_per_day
        );
        assert!(
            (o.break_point_g_per_day - 113.4).abs() < 1e-9,
            "got {}",
            o.break_point_g_per_day
        );
    }

    #[test]
    fn accepts_boundary_inputs() {
        assert!(compute(&ProteinTargetInput { weight_kg: 20.0 }).is_ok());
        assert!(compute(&ProteinTargetInput { weight_kg: 300.0 }).is_ok());
    }

    #[test]
    fn rejects_out_of_range_weight() {
        assert!(compute(&ProteinTargetInput { weight_kg: 19.9 }).is_err());
        assert!(compute(&ProteinTargetInput { weight_kg: 300.1 }).is_err());
        assert!(
            compute(&ProteinTargetInput {
                weight_kg: f64::NAN
            })
            .is_err()
        );
        assert!(
            compute(&ProteinTargetInput {
                weight_kg: f64::INFINITY
            })
            .is_err()
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "weight_kg": 90.0 });
        let dynamic = ProteinTarget.calculate(&value).unwrap();
        let typed = build_response(&ProteinTargetInput { weight_kg: 90.0 }).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn dynamic_surface_rejects_unknown_fields() {
        let invalid = json!({ "weight_kg": 90.0, "unexpected": true });
        assert!(ProteinTarget.calculate(&invalid).is_err());
    }

    #[test]
    fn response_reports_range_result_and_working() {
        let response = build_response(&ProteinTargetInput { weight_kg: 70.0 }).unwrap();
        assert_eq!(response.result, json!("112-154 g/day"));
        assert_eq!(response.working["low_g_per_day"], json!(112.0));
        assert_eq!(response.working["high_g_per_day"], json!(154.0));
        assert_eq!(response.working["break_point_g_per_day"], json!(113.4));
        assert_eq!(response.working["low_g_per_kg"], json!(1.6));
        assert_eq!(response.working["high_g_per_kg"], json!(2.2));
        assert!(
            response
                .interpretation
                .contains("not a general dietary protein minimum")
        );
        assert!(
            response
                .interpretation
                .contains("not validated specifically for a caloric deficit")
        );
    }

    #[test]
    fn schema_documents_scope_limits() {
        let schema = ProteinTarget.input_schema();
        assert_eq!(schema["properties"]["weight_kg"]["unit"], json!("kg"));
        let description = schema["description"].as_str().unwrap();
        assert!(description.contains("Not a general dietary minimum"));
        assert!(description.contains("not validated specifically for a caloric deficit"));
    }
}
