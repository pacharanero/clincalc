// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Maximum heart rate and training-zone estimates.
//!
//! Estimates maximal heart rate (HRmax) and derives aerobic training zones.
//! Default age-predicted HRmax uses the Tanaka formula (2001), which is more
//! accurate across older adults than the older 220 - age formula. If a resting
//! heart rate is supplied, zones are computed from heart-rate reserve (Karvonen
//! method); otherwise they are computed as simple percentages of HRmax.
//!
//! References:
//! - Tanaka H, Monahan KD, Seals DR. Age-predicted maximal heart rate revisited.
//!   J Am Coll Cardiol. 2001;37(1):153-156.
//! - Karvonen MJ, Kentala E, Mustala O. The effects of training on heart rate;
//!   a longitudinal study. Ann Med Exp Biol Fenn. 1957;35(3):307-315.
//! - ACSM's Guidelines for Exercise Testing and Prescription, 11th ed. (2021)
//!   provides the common zone percentages.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "max_heart_rate";
pub const REFERENCE: &str = "Tanaka H, Monahan KD, Seals DR. Age-predicted maximal heart rate revisited. J Am Coll Cardiol. 2001;37(1):153-156. Karvonen MJ, Kentala E, Mustala O. The effects of training on heart rate. Ann Med Exp Biol Fenn. 1957;35(3):307-315. ACSM's Guidelines for Exercise Testing and Prescription. 11th ed. 2021.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - exercise physiology formulae",
    source_url: "https://doi.org/10.1016/S0735-1097(00)01054-8",
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaxHeartRateInput {
    /// Age in years
    pub age_years: u32,
    /// Resting heart rate in bpm (optional; triggers Karvonen zones when present)
    pub resting_hr: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeartRateZone {
    pub name: &'static str,
    pub lower_fraction: f64,
    pub upper_fraction: f64,
}

const ZONES: &[HeartRateZone] = &[
    HeartRateZone {
        name: "very light",
        lower_fraction: 0.50,
        upper_fraction: 0.60,
    },
    HeartRateZone {
        name: "light",
        lower_fraction: 0.60,
        upper_fraction: 0.70,
    },
    HeartRateZone {
        name: "moderate",
        lower_fraction: 0.70,
        upper_fraction: 0.80,
    },
    HeartRateZone {
        name: "vigorous",
        lower_fraction: 0.80,
        upper_fraction: 0.90,
    },
    HeartRateZone {
        name: "maximum",
        lower_fraction: 0.90,
        upper_fraction: 1.00,
    },
];

#[derive(Debug, Clone, PartialEq)]
pub struct MaxHeartRateOutcome {
    pub hrmax: u32,
    pub method: &'static str,
    pub zones: Vec<(&'static str, u32, u32)>,
    pub interpretation: String,
}

pub fn compute(input: &MaxHeartRateInput) -> Result<MaxHeartRateOutcome, CalcError> {
    if input.age_years < 10 || input.age_years > 120 {
        return Err(CalcError::InvalidInput(
            "age_years must be between 10 and 120".into(),
        ));
    }

    if let Some(rhr) = input.resting_hr
        && !(30..=150).contains(&rhr)
    {
        return Err(CalcError::InvalidInput(
            "resting_hr must be between 30 and 150 when provided".into(),
        ));
    }

    // Tanaka 2001: HRmax = 208 - 0.7 * age
    let hrmax = (208.0 - 0.7 * input.age_years as f64).round() as u32;
    let hrmax = hrmax.max(1);

    let (reserve, method) = if let Some(rhr) = input.resting_hr {
        let reserve = hrmax.saturating_sub(rhr);
        (reserve as f64, "Karvonen (heart-rate reserve)")
    } else {
        (hrmax as f64, "percentage of HRmax")
    };

    let mut zones = Vec::with_capacity(ZONES.len());
    for zone in ZONES {
        let lower = if input.resting_hr.is_some() {
            // Karvonen: RHR + reserve * fraction
            (rhr_or(input.resting_hr) as f64 + reserve * zone.lower_fraction).round() as u32
        } else {
            (hrmax as f64 * zone.lower_fraction).round() as u32
        };
        let upper = if input.resting_hr.is_some() {
            (rhr_or(input.resting_hr) as f64 + reserve * zone.upper_fraction).round() as u32
        } else {
            (hrmax as f64 * zone.upper_fraction).round() as u32
        };
        let upper = upper.max(lower);
        zones.push((zone.name, lower, upper));
    }

    let rhr_note = if let Some(rhr) = input.resting_hr {
        format!("; Karvonen zones based on resting HR {rhr} bpm")
    } else {
        "; supply resting_hr for Karvonen (heart-rate reserve) zones".to_string()
    };

    let interpretation = format!(
        "Estimated maximum heart rate {hrmax} bpm by Tanaka 208 - 0.7 x age ({method}){rhr_note}. \
Zones are rounded estimates for healthy adults; individual testing (e.g. graded exercise test) is more accurate for athletes or patients with cardiovascular disease."
    );

    Ok(MaxHeartRateOutcome {
        hrmax,
        method,
        zones,
        interpretation,
    })
}

fn rhr_or(rhr: Option<u32>) -> u32 {
    rhr.unwrap_or(0)
}

pub fn build_response(input: &MaxHeartRateInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("hrmax".into(), json!(o.hrmax));
    working.insert("method".into(), json!(o.method));
    let zones_json: Vec<Value> = o
        .zones
        .iter()
        .map(|(name, lower, upper)| {
            json!({
                "zone": *name,
                "lower_bpm": *lower,
                "upper_bpm": *upper
            })
        })
        .collect();
    working.insert("zones".into(), json!(zones_json));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.hrmax),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct MaxHeartRate;

impl Calculator for MaxHeartRate {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Max Heart Rate & Training Zones"
    }

    fn description(&self) -> &'static str {
        "Estimates HRmax from age (Tanaka formula) and derives aerobic training zones, optionally using Karvonen heart-rate reserve when resting HR is supplied."
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
            "title": "MaxHeartRateInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["age_years"],
            "properties": {
                "age_years": {
                    "type": "integer",
                    "minimum": 10,
                    "maximum": 120,
                    "description": "Age in years"
                },
                "resting_hr": {
                    "type": ["integer", "null"],
                    "minimum": 30,
                    "maximum": 150,
                    "description": "Resting heart rate in bpm; when provided, zones are computed with the Karvonen (heart-rate reserve) method"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: MaxHeartRateInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(age: u32, rhr: Option<u32>) -> MaxHeartRateInput {
        MaxHeartRateInput {
            age_years: age,
            resting_hr: rhr,
        }
    }

    #[test]
    fn tanaka_only() {
        let o = compute(&input(40, None)).unwrap();
        assert_eq!(o.hrmax, 180);
        assert_eq!(o.method, "percentage of HRmax");
        // moderate zone 70-80% of 180 = 126-144
        assert_eq!(
            o.zones.iter().find(|(n, _, _)| *n == "moderate").copied(),
            Some(("moderate", 126, 144))
        );
    }

    #[test]
    fn karvonen_zones() {
        let o = compute(&input(40, Some(60))).unwrap();
        assert_eq!(o.hrmax, 180);
        assert_eq!(o.method, "Karvonen (heart-rate reserve)");
        // reserve = 120; moderate 70-80% => RHR + reserve*0.7 .. 0.8 = 144..156
        assert_eq!(
            o.zones.iter().find(|(n, _, _)| *n == "moderate").copied(),
            Some(("moderate", 144, 156))
        );
    }

    #[test]
    fn rejects_extreme_age() {
        assert!(compute(&input(5, None)).is_err());
        assert!(compute(&input(130, None)).is_err());
    }

    #[test]
    fn rejects_bad_resting_hr() {
        assert!(compute(&input(40, Some(20))).is_err());
        assert!(compute(&input(40, Some(200))).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age_years": 30,
            "resting_hr": 55
        });
        let dynamic = MaxHeartRate.calculate(&value).unwrap();
        let typed = build_response(&input(30, Some(55))).unwrap();
        assert_eq!(dynamic, typed);
    }
}
