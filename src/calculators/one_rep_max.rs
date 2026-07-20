// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! One-repetition maximum (1RM) estimators from submaximal lift data.
//!
//! Estimates the maximum weight that can be lifted for one full repetition
//! from a lighter weight lifted for a known number of reps. Useful in strength
//! programming, rehabilitation, and bodybuilding load prescription.
//!
//! Supported formulae:
//! - Epley: 1RM = w * (1 + r / 30)
//! - Brzycki: 1RM = w * 36 / (37 - r)
//! - Lombardi: 1RM = w * r^0.10
//!
//! Validity falls as reps increase; most formulae are reasonable up to ~10
//! reps and become unreliable above ~20 reps. For r = 1 the estimate equals
//! the lifted weight for all formulae.
//!
//! References:
//! - Epley B. Poundage chart. Boyd Epley Workout. 1985.
//! - Brzycki M. Strength testing: predicting a one-rep max from a reps-to-fatigue chart.
//!   J Phys Ed Rec Dance. 1993;64(1):88-90.
//! - Lombardi VP. Beginning Weight Training. Brown & Benchmark. 1989.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "one_rep_max";
pub const REFERENCE: &str = "Epley B. Poundage chart. Boyd Epley Workout. 1985. Brzycki M. Strength testing: predicting a one-rep max from a reps-to-fatigue chart. J Phys Ed Rec Dance. 1993;64(1):88-90. Lombardi VP. Beginning Weight Training. Brown & Benchmark. 1989.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain formulae - exercise physiology",
    source_url: "https://doi.org/10.1080%2F07303084.1993.10607288",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OneRepMaxFormula {
    Epley,
    Brzycki,
    Lombardi,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OneRepMaxInput {
    /// Weight lifted in kilograms
    pub weight_kg: f64,
    /// Number of completed, full repetitions
    pub reps: u32,
    /// Estimation formula to use
    pub formula: OneRepMaxFormula,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OneRepMaxOutcome {
    pub one_rm_kg: f64,
    pub formula: &'static str,
    pub interpretation: String,
}

pub fn compute(input: &OneRepMaxInput) -> Result<OneRepMaxOutcome, CalcError> {
    if !(0.0..=700.0).contains(&input.weight_kg) || !input.weight_kg.is_finite() {
        return Err(CalcError::InvalidInput(
            "weight_kg must be finite and between 0 and 700".into(),
        ));
    }
    if input.weight_kg <= 0.0 {
        return Err(CalcError::InvalidInput("weight_kg must be positive".into()));
    }
    if input.reps == 0 {
        return Err(CalcError::InvalidInput("reps must be at least 1".into()));
    }
    if input.reps > 30 {
        return Err(CalcError::InvalidInput(
            "reps above 30 produce unreliable estimates and are not supported".into(),
        ));
    }

    let r = input.reps as f64;
    let w = input.weight_kg;

    let one_rm = match input.formula {
        OneRepMaxFormula::Epley => w * (1.0 + r / 30.0),
        OneRepMaxFormula::Brzycki => {
            let denom = 37.0 - r;
            if denom <= 0.0 {
                return Err(CalcError::InvalidInput(
                    "Brzycki formula is not valid at or above 37 reps".into(),
                ));
            }
            w * 36.0 / denom
        }
        OneRepMaxFormula::Lombardi => w * r.powf(0.10),
    };

    if !one_rm.is_finite() || one_rm <= 0.0 {
        return Err(CalcError::InvalidInput(
            "computed 1RM is invalid - check inputs".into(),
        ));
    }

    let reliability_note = if input.reps > 10 {
        "High-rep estimates are less accurate; use a low-rep test when possible."
    } else {
        "Reasonable estimate for submaximal testing up to ~10 reps."
    };

    let formula_label: &'static str = match input.formula {
        OneRepMaxFormula::Epley => "Epley",
        OneRepMaxFormula::Brzycki => "Brzycki",
        OneRepMaxFormula::Lombardi => "Lombardi",
    };

    let interpretation = format!(
        "Estimated 1RM {:.1} kg from {w:.1} kg x {} reps using the {formula_label} formula. {reliability_note}",
        one_rm, input.reps
    );

    Ok(OneRepMaxOutcome {
        one_rm_kg: one_rm,
        formula: formula_label,
        interpretation,
    })
}

pub fn build_response(input: &OneRepMaxInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("one_rm_kg".into(), json!(round1(o.one_rm_kg)));
    working.insert("formula".into(), json!(o.formula));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.one_rm_kg)),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct OneRepMax;

impl Calculator for OneRepMax {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "One-Rep Max Estimator"
    }

    fn description(&self) -> &'static str {
        "Estimates 1RM from a submaximal weight and reps using Epley, Brzycki, or Lombardi."
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
            "title": "OneRepMaxInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["weight_kg", "reps", "formula"],
            "properties": {
                "weight_kg": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "maximum": 700,
                    "description": "Weight lifted in kilograms"
                },
                "reps": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 30,
                    "description": "Number of completed, full repetitions"
                },
                "formula": {
                    "type": "string",
                    "enum": ["epley", "brzycki", "lombardi"],
                    "description": "Estimation formula"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: OneRepMaxInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(weight: f64, reps: u32, formula: OneRepMaxFormula) -> OneRepMaxInput {
        OneRepMaxInput {
            weight_kg: weight,
            reps,
            formula,
        }
    }

    #[test]
    fn epley_five_reps() {
        let o = compute(&input(80.0, 5, OneRepMaxFormula::Epley)).unwrap();
        assert!((o.one_rm_kg - 93.3).abs() < 0.1, "got {:.1}", o.one_rm_kg);
        assert_eq!(o.formula, "Epley");
    }

    #[test]
    fn brzycki_five_reps() {
        let o = compute(&input(80.0, 5, OneRepMaxFormula::Brzycki)).unwrap();
        // 80 * 36 / (37 - 5) = 2880 / 32 = 90.0
        assert!((o.one_rm_kg - 90.0).abs() < 0.1, "got {:.1}", o.one_rm_kg);
        assert_eq!(o.formula, "Brzycki");
    }

    #[test]
    fn one_rep_returns_weight() {
        let o = compute(&input(100.0, 1, OneRepMaxFormula::Brzycki)).unwrap();
        assert!((o.one_rm_kg - 100.0).abs() < 0.01);
    }

    #[test]
    fn rejects_zero_reps() {
        assert!(compute(&input(80.0, 0, OneRepMaxFormula::Epley)).is_err());
    }

    #[test]
    fn rejects_too_many_reps() {
        assert!(compute(&input(80.0, 31, OneRepMaxFormula::Epley)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "weight_kg": 80.0,
            "reps": 5,
            "formula": "lombardi"
        });
        let dynamic = OneRepMax.calculate(&value).unwrap();
        let typed = build_response(&input(80.0, 5, OneRepMaxFormula::Lombardi)).unwrap();
        assert_eq!(dynamic, typed);
    }
}
