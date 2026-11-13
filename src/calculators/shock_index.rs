// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Shock Index - heart rate / systolic blood pressure, with the
//! paediatric age-adjusted (SIPA) threshold variant.
//!
//! The index itself (Allgöwer & Burri, 1967) is a single ratio; the
//! age-adjusted thresholds used to flag an abnormal value are specific to
//! each population. This implementation uses SIPA's published age bands for
//! ages 4-16 (Acker et al. 2015) and the commonly used adult threshold of
//! 0.9 for ages 17 and over (the 1967 paper itself does not state a formal
//! threshold; 0.9 is the value validated as predicting mortality/transfusion
//! need in later adult trauma and sepsis literature). Below age 4, no
//! validated threshold is reported here.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "shock_index";

pub const REFERENCE: &str = "Allgower M, Burri C. Schockindex. Dtsch Med Wochenschr. 1967;92(43):1947-1950 \
(adult index); Acker SN, Ross JT, Partrick DA, Tong S, Bensard DD. Pediatric specific shock index \
accurately identifies severely injured children. J Pediatr Surg. 2015;50(2):331-334 (SIPA, age-adjusted \
thresholds).";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.1016/j.jpedsurg.2014.10.048",
);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShockIndexInput {
    pub age_years: f64,
    pub heart_rate_bpm: f64,
    pub systolic_bp_mmhg: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShockIndexOutcome {
    pub shock_index: f64,
    /// `None` when no validated threshold applies (age under 4).
    pub threshold: Option<f64>,
    pub elevated: Option<bool>,
    pub population: &'static str,
}

fn population_and_threshold(age_years: f64) -> (&'static str, Option<f64>) {
    if age_years < 4.0 {
        ("unvalidated (age under 4)", None)
    } else if age_years < 7.0 {
        ("paediatric age-adjusted (SIPA), 4-6 years", Some(1.22))
    } else if age_years < 13.0 {
        ("paediatric age-adjusted (SIPA), 7-12 years", Some(1.0))
    } else if age_years < 17.0 {
        ("paediatric age-adjusted (SIPA), 13-16 years", Some(0.9))
    } else {
        ("adult", Some(0.9))
    }
}

pub fn compute(input: &ShockIndexInput) -> Result<ShockIndexOutcome, CalcError> {
    if !input.age_years.is_finite() || input.age_years < 0.0 {
        return Err(CalcError::InvalidInput(
            "age_years must be a non-negative finite number".into(),
        ));
    }
    if !input.heart_rate_bpm.is_finite() || input.heart_rate_bpm <= 0.0 {
        return Err(CalcError::InvalidInput(
            "heart_rate_bpm must be a positive finite number".into(),
        ));
    }
    if !input.systolic_bp_mmhg.is_finite() || input.systolic_bp_mmhg <= 0.0 {
        return Err(CalcError::InvalidInput(
            "systolic_bp_mmhg must be a positive finite number".into(),
        ));
    }

    let shock_index = input.heart_rate_bpm / input.systolic_bp_mmhg;
    let (population, threshold) = population_and_threshold(input.age_years);
    let elevated = threshold.map(|t| shock_index > t);

    Ok(ShockIndexOutcome {
        shock_index,
        threshold,
        elevated,
        population,
    })
}

pub fn build_response(input: &ShockIndexInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("shock_index".into(), json!(round2(o.shock_index)));
    working.insert("population".into(), json!(o.population));
    if let Some(threshold) = o.threshold {
        working.insert("threshold".into(), json!(threshold));
    }
    if let Some(elevated) = o.elevated {
        working.insert("elevated".into(), json!(elevated));
    }

    let interpretation = match (o.threshold, o.elevated) {
        (Some(threshold), Some(true)) => format!(
            "Shock index {:.2} (heart rate / systolic BP), {} population, threshold {:.2}: elevated. An elevated shock index is associated with increased risk of mortality, transfusion need, or severe injury in the relevant population.",
            o.shock_index, o.population, threshold
        ),
        (Some(threshold), Some(false)) => format!(
            "Shock index {:.2} (heart rate / systolic BP), {} population, threshold {:.2}: not elevated.",
            o.shock_index, o.population, threshold
        ),
        _ => format!(
            "Shock index {:.2} (heart rate / systolic BP). No validated threshold is reported for this age ({} population); interpret in clinical context.",
            o.shock_index, o.population
        ),
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round2(o.shock_index)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub struct ShockIndex;
impl Calculator for ShockIndex {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Shock Index"
    }
    fn description(&self) -> &'static str {
        "Heart rate / systolic blood pressure ratio for early shock recognition, with adult and paediatric age-adjusted (SIPA) thresholds."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "ShockIndexInput", "type": "object",
            "additionalProperties": false, "required": ["age_years", "heart_rate_bpm", "systolic_bp_mmhg"],
            "properties": {
                "age_years": { "type": "number", "minimum": 0, "maximum": 130, "description": "Age in years; selects the adult or SIPA age-adjusted threshold" },
                "heart_rate_bpm": { "type": "number", "exclusiveMinimum": 0, "maximum": 300, "description": "Heart rate in beats per minute" },
                "systolic_bp_mmhg": { "type": "number", "exclusiveMinimum": 0, "maximum": 300, "description": "Systolic blood pressure in mmHg" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: ShockIndexInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(age: f64, hr: f64, sbp: f64) -> ShockIndexInput {
        ShockIndexInput {
            age_years: age,
            heart_rate_bpm: hr,
            systolic_bp_mmhg: sbp,
        }
    }

    #[test]
    fn adult_threshold_is_point_nine() {
        let o = compute(&input(40.0, 115.0, 120.0)).unwrap();
        assert_eq!(o.threshold, Some(0.9));
        assert!(o.elevated.unwrap());
        assert_eq!(o.population, "adult");
    }

    #[test]
    fn adult_not_elevated_below_threshold() {
        let o = compute(&input(40.0, 90.0, 120.0)).unwrap();
        assert!(!o.elevated.unwrap());
    }

    #[test]
    fn sipa_age_4_to_6_uses_1_22() {
        let o = compute(&input(5.0, 110.0, 90.0)).unwrap();
        assert_eq!(o.threshold, Some(1.22));
    }

    #[test]
    fn sipa_age_7_to_12_uses_1_0() {
        let o = compute(&input(10.0, 100.0, 100.0)).unwrap();
        assert_eq!(o.threshold, Some(1.0));
        assert!(!o.elevated.unwrap());
    }

    #[test]
    fn sipa_age_13_to_16_uses_0_9() {
        let o = compute(&input(14.0, 100.0, 100.0)).unwrap();
        assert_eq!(o.threshold, Some(0.9));
        assert!(o.elevated.unwrap());
    }

    #[test]
    fn under_4_has_no_validated_threshold() {
        let o = compute(&input(2.0, 130.0, 90.0)).unwrap();
        assert_eq!(o.threshold, None);
        assert_eq!(o.elevated, None);
    }

    #[test]
    fn rejects_non_positive_vitals() {
        assert!(compute(&input(40.0, 0.0, 120.0)).is_err());
        assert!(compute(&input(40.0, 90.0, 0.0)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value =
            json!({ "age_years": 40.0, "heart_rate_bpm": 110.0, "systolic_bp_mmhg": 100.0 });
        let typed = input(40.0, 110.0, 100.0);
        let dynamic = ShockIndex.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(1.1));
    }
}
