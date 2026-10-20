// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Parkland (Baxter) formula for first-24-hour burn crystalloid resuscitation.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "parkland";
pub const REFERENCE: &str = "Baxter CR, Shires T. Physiological response to crystalloid resuscitation of severe burns. Ann N Y Acad Sci. 1968;150(3):874-894.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://pubmed.ncbi.nlm.nih.gov/4973463/",
);

/// Baxter-Shires crystalloid volume, mL per kg body weight per % TBSA burned.
const ML_PER_KG_PER_PERCENT: f64 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParklandInput {
    pub age_years: f64,
    pub weight_kg: f64,
    pub burn_tbsa_percent: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParklandOutcome {
    pub total_24h_ml: f64,
    pub first_8h_ml: f64,
    pub next_16h_ml: f64,
    pub first_8h_ml_per_hour: f64,
    pub next_16h_ml_per_hour: f64,
    pub interpretation: String,
}

pub fn compute(input: &ParklandInput) -> Result<ParklandOutcome, CalcError> {
    validate(input.age_years, "age_years", 16.0, 120.0)?;
    validate(input.weight_kg, "weight_kg", 20.0, 400.0)?;
    validate(input.burn_tbsa_percent, "burn_tbsa_percent", 1.0, 100.0)?;

    let total = ML_PER_KG_PER_PERCENT * input.weight_kg * input.burn_tbsa_percent;
    let half = total / 2.0;
    let interpretation = format!(
        "Parkland formula (4 mL x {:.1} kg x {:.1}% TBSA): {:.0} mL crystalloid over the first 24 hours from the time of burn - {:.0} mL in the first 8 hours ({:.0} mL/h) and {:.0} mL over the following 16 hours ({:.0} mL/h). The formula gives a starting estimate only: titrate to clinical response (for example urine output) and follow local burns-service protocol, as some current guidance starts at lower volumes. Excludes maintenance fluid and fluid already given.",
        input.weight_kg,
        input.burn_tbsa_percent,
        total,
        half,
        half / 8.0,
        half,
        half / 16.0,
    );
    Ok(ParklandOutcome {
        total_24h_ml: total,
        first_8h_ml: half,
        next_16h_ml: half,
        first_8h_ml_per_hour: half / 8.0,
        next_16h_ml_per_hour: half / 16.0,
        interpretation,
    })
}

fn validate(value: f64, name: &str, min: f64, max: f64) -> Result<(), CalcError> {
    if !value.is_finite() || !(min..=max).contains(&value) {
        return Err(CalcError::InvalidInput(format!(
            "{name} must be finite and between {min} and {max}"
        )));
    }
    Ok(())
}

pub fn build_response(input: &ParklandInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "ml_per_kg_per_percent_tbsa".into(),
        json!(ML_PER_KG_PER_PERCENT),
    );
    working.insert("total_24h_ml".into(), json!(round1(o.total_24h_ml)));
    working.insert("first_8h_ml".into(), json!(round1(o.first_8h_ml)));
    working.insert("next_16h_ml".into(), json!(round1(o.next_16h_ml)));
    working.insert(
        "first_8h_ml_per_hour".into(),
        json!(round1(o.first_8h_ml_per_hour)),
    );
    working.insert(
        "next_16h_ml_per_hour".into(),
        json!(round1(o.next_16h_ml_per_hour)),
    );
    working.insert("unit".into(), json!("mL"));
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.total_24h_ml)),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct Parkland;
impl Calculator for Parkland {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Parkland Formula (burns fluid resuscitation)"
    }
    fn description(&self) -> &'static str {
        "Adult first-24-hour crystalloid volume after a major burn (4 mL x kg x % TBSA, half in the first 8 hours from time of burn), from the Baxter-Shires formula. A starting estimate to be titrated to response; not for children, and excludes maintenance fluid."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "ParklandInput", "type": "object", "additionalProperties": false,
            "required": ["age_years", "weight_kg", "burn_tbsa_percent"],
            "properties": {
                "age_years": { "type": "number", "minimum": 16, "maximum": 120, "description": "Age in years; adults only (paediatric burn resuscitation needs maintenance fluid and a different protocol)" },
                "weight_kg": { "type": "number", "minimum": 20, "maximum": 400, "description": "Actual body weight in kg" },
                "burn_tbsa_percent": { "type": "number", "minimum": 1, "maximum": 100, "description": "Percentage of total body surface area with partial-thickness (second-degree) or full-thickness burns; exclude superficial erythema" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: ParklandInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(weight_kg: f64, tbsa: f64) -> ParklandInput {
        ParklandInput {
            age_years: 35.0,
            weight_kg,
            burn_tbsa_percent: tbsa,
        }
    }

    #[test]
    fn four_ml_per_kg_per_percent() {
        // 70 kg x 40% x 4 mL = 11,200 mL; half in first 8 h.
        let o = compute(&input(70.0, 40.0)).unwrap();
        assert_eq!(o.total_24h_ml, 11_200.0);
        assert_eq!(o.first_8h_ml, 5_600.0);
        assert_eq!(o.next_16h_ml, 5_600.0);
        assert_eq!(o.first_8h_ml_per_hour, 700.0);
        assert_eq!(o.next_16h_ml_per_hour, 350.0);
    }

    #[test]
    fn minimum_burn() {
        assert_eq!(compute(&input(100.0, 1.0)).unwrap().total_24h_ml, 400.0);
    }

    #[test]
    fn rejects_children_and_out_of_range() {
        let mut child = input(70.0, 40.0);
        child.age_years = 8.0;
        assert!(compute(&child).is_err());
        assert!(compute(&input(70.0, 0.0)).is_err());
        assert!(compute(&input(70.0, 101.0)).is_err());
        assert!(compute(&input(f64::NAN, 10.0)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"age_years": 35.0, "weight_kg": 70.0, "burn_tbsa_percent": 40.0});
        let dynamic = Parkland.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&input(70.0, 40.0)).unwrap());
        assert_eq!(dynamic.result, json!(11_200.0));
    }

    #[test]
    fn rejects_unknown_and_missing_fields() {
        assert!(
            Parkland
                .calculate(&json!({"age_years": 35.0, "weight_kg": 70.0}))
                .is_err()
        );
        assert!(
            Parkland
                .calculate(&json!({"age_years": 35.0, "weight_kg": 70.0, "burn_tbsa_percent": 10.0, "extra": 1}))
                .is_err()
        );
    }
}
