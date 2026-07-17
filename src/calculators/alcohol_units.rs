// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! UK alcohol units from volume and ABV, with optional weekly tally.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "alcohol_units";
pub const REFERENCE: &str = "UK Chief Medical Officers. UK Chief Medical Officers' Low Risk Drinking Guidelines. Department of Health; 2016. UK alcohol unit definition: 10 mL / 8 g pure alcohol.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain public-health method - implemented from UK Chief Medical Officers' guidance",
    source_url: "https://www.gov.uk/government/publications/alcohol-consumption-advice-on-low-risk-drinking",
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlcoholUnitsInput {
    /// Drink container or serving volume in millilitres.
    pub volume_ml: f64,
    /// Alcohol by volume percentage, e.g. 5 for 5% ABV.
    pub abv_percent: f64,
    /// Number of such servings. Defaults to 1.
    #[serde(default)]
    pub servings: Option<f64>,
    /// Optional frequency per week for weekly units.
    #[serde(default)]
    pub servings_per_week: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlcoholUnitsOutcome {
    pub units: f64,
    pub ethanol_g: f64,
    pub kcal: f64,
    pub weekly_units: Option<f64>,
    pub interpretation: String,
}

pub fn compute(input: &AlcoholUnitsInput) -> Result<AlcoholUnitsOutcome, CalcError> {
    validate_positive(input.volume_ml, "volume_ml")?;
    validate_positive(input.abv_percent, "abv_percent")?;
    if input.abv_percent > 80.0 {
        return Err(CalcError::InvalidInput("abv_percent must be <= 80".into()));
    }
    let servings = input.servings.unwrap_or(1.0);
    validate_positive(servings, "servings")?;
    let ethanol_ml = input.volume_ml * input.abv_percent / 100.0 * servings;
    let units = ethanol_ml / 10.0;
    let ethanol_g = ethanol_ml * 0.789;
    let kcal = ethanol_g * 7.0;

    let weekly_units = if let Some(per_week) = input.servings_per_week {
        validate_positive(per_week, "servings_per_week")?;
        Some(units / servings * per_week)
    } else {
        None
    };

    let interpretation = if let Some(weekly) = weekly_units {
        if weekly <= 14.0 {
            format!(
                "{units:.1} UK units in the supplied serving(s); weekly estimate {weekly:.1} units, within the UK low-risk guideline of no more than 14 units/week if spread over several days."
            )
        } else {
            format!(
                "{units:.1} UK units in the supplied serving(s); weekly estimate {weekly:.1} units, above the UK low-risk guideline of no more than 14 units/week."
            )
        }
    } else {
        format!(
            "{units:.1} UK units in the supplied serving(s). One UK unit is 10 mL (about 8 g) of pure alcohol; the low-risk guideline is no more than 14 units/week, spread over several days."
        )
    };

    Ok(AlcoholUnitsOutcome {
        units,
        ethanol_g,
        kcal,
        weekly_units,
        interpretation,
    })
}

fn validate_positive(value: f64, name: &str) -> Result<(), CalcError> {
    if value <= 0.0 || !value.is_finite() {
        return Err(CalcError::InvalidInput(format!(
            "{name} must be a positive finite number"
        )));
    }
    Ok(())
}

pub fn build_response(input: &AlcoholUnitsInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("uk_units".into(), json!(round1(o.units)));
    working.insert("ethanol_g".into(), json!(round1(o.ethanol_g)));
    working.insert("alcohol_kcal".into(), json!(round0(o.kcal)));
    if let Some(weekly) = o.weekly_units {
        working.insert("weekly_units".into(), json!(round1(weekly)));
        working.insert("uk_low_risk_weekly_limit_units".into(), json!(14));
    }
    working.insert(
        "formula".into(),
        json!("units = volume_ml * ABV_percent / 1000 * servings"),
    );
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.units)),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}
fn round0(v: f64) -> u32 {
    v.round() as u32
}

pub struct AlcoholUnits;

impl Calculator for AlcoholUnits {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Alcohol Units (UK)"
    }
    fn description(&self) -> &'static str {
        "Calculates UK alcohol units from drink volume and ABV, with optional weekly tally and alcohol kcal."
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
            "title": "AlcoholUnitsInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["volume_ml", "abv_percent"],
            "properties": {
                "volume_ml": { "type": "number", "exclusiveMinimum": 0, "description": "Volume per serving in mL" },
                "abv_percent": { "type": "number", "exclusiveMinimum": 0, "maximum": 80, "description": "Alcohol by volume percentage" },
                "servings": { "type": "number", "exclusiveMinimum": 0, "description": "Number of servings in this calculation; defaults to 1" },
                "servings_per_week": { "type": "number", "exclusiveMinimum": 0, "description": "Optional frequency per week for weekly units" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AlcoholUnitsInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pint_of_five_percent_is_about_2_8_units() {
        let out = compute(&AlcoholUnitsInput {
            volume_ml: 568.0,
            abv_percent: 5.0,
            servings: None,
            servings_per_week: None,
        })
        .unwrap();
        assert_eq!(round1(out.units), 2.8);
    }

    #[test]
    fn weekly_units_use_frequency_not_total_servings() {
        let out = compute(&AlcoholUnitsInput {
            volume_ml: 175.0,
            abv_percent: 13.0,
            servings: None,
            servings_per_week: Some(7.0),
        })
        .unwrap();
        assert_eq!(round1(out.weekly_units.unwrap()), 15.9);
    }
}
