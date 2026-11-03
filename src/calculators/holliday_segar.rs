// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Holliday-Segar maintenance intravenous fluid rate (the "4-2-1 rule"),
//! derived from and primarily used in paediatric practice.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "holliday_segar";
pub const REFERENCE: &str = "Holliday MA, Segar WE. The maintenance need for water in parenteral fluid therapy. Pediatrics. 1957;19(5):823-832.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain formula - implemented from the primary literature",
    "https://doi.org/10.1542/peds.19.5.823",
);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HollidaySegarInput {
    pub weight_kg: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HollidaySegarOutcome {
    pub daily_ml: f64,
    pub hourly_ml: f64,
}

pub fn compute(input: &HollidaySegarInput) -> Result<HollidaySegarOutcome, CalcError> {
    validate(input.weight_kg, "weight_kg", 1.0, 150.0)?;

    let w = input.weight_kg;
    let daily_ml = if w <= 10.0 {
        w * 100.0
    } else if w <= 20.0 {
        1000.0 + (w - 10.0) * 50.0
    } else {
        1500.0 + (w - 20.0) * 20.0
    };

    Ok(HollidaySegarOutcome {
        daily_ml,
        hourly_ml: daily_ml / 24.0,
    })
}

fn validate(value: f64, name: &str, min: f64, max: f64) -> Result<(), CalcError> {
    if !(min..=max).contains(&value) || !value.is_finite() {
        return Err(CalcError::InvalidInput(format!(
            "{name} must be finite and between {min} and {max}"
        )));
    }
    Ok(())
}

pub fn build_response(input: &HollidaySegarInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("hourly_ml".into(), json!(round1(o.hourly_ml)));
    working.insert("unit".into(), json!("mL"));

    let interpretation = format!(
        "Holliday-Segar maintenance fluid rate: {:.0} mL/24h ({:.1} mL/h), the \"4-2-1 rule\" (4 mL/kg/h for the first 10 kg, 2 mL/kg/h for the next 10 kg, 1 mL/kg/h for each kg thereafter). This is a maintenance estimate derived in and primarily validated for children; it excludes resuscitation, ongoing losses, and deficit replacement, and does not apply unmodified to critically unwell, fluid-restricted, or adult patients.",
        o.daily_ml, o.hourly_ml
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round0(o.daily_ml)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round0(v: f64) -> f64 {
    v.round()
}
fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct HollidaySegar;
impl Calculator for HollidaySegar {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Holliday-Segar Maintenance IV Fluid Rate"
    }
    fn description(&self) -> &'static str {
        "Paediatric maintenance intravenous fluid rate by the Holliday-Segar '4-2-1 rule'."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "HollidaySegarInput", "type": "object", "additionalProperties": false,
            "required": ["weight_kg"],
            "properties": {
                "weight_kg": { "type": "number", "minimum": 1, "maximum": 150 }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: HollidaySegarInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_10kg_uses_first_band() {
        let out = compute(&HollidaySegarInput { weight_kg: 8.0 }).unwrap();
        assert_eq!(out.daily_ml, 800.0);
    }

    #[test]
    fn exactly_15kg_uses_second_band() {
        let out = compute(&HollidaySegarInput { weight_kg: 15.0 }).unwrap();
        // 1000 + 5*50 = 1250
        assert_eq!(out.daily_ml, 1250.0);
    }

    #[test]
    fn over_20kg_uses_third_band() {
        let out = compute(&HollidaySegarInput { weight_kg: 30.0 }).unwrap();
        // 1500 + 10*20 = 1700
        assert_eq!(out.daily_ml, 1700.0);
        assert!((out.hourly_ml - 70.833_333_333).abs() < 1e-6);
    }

    #[test]
    fn rejects_out_of_range_weight() {
        assert!(compute(&HollidaySegarInput { weight_kg: 0.0 }).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"weight_kg": 15.0});
        let typed = HollidaySegarInput { weight_kg: 15.0 };
        let dynamic = HollidaySegar.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(HollidaySegar.calculate(&json!({})).is_err());
    }
}
