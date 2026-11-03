// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Weight-based unfractionated-heparin dosing nomogram (Raschke et al,
//! 1993). Reports the weight-based initial bolus and infusion rate, and -
//! when an aPTT ratio and the current infusion rate are supplied - the
//! Raschke-nomogram adjustment.
//!
//! This is a calculation aid reflecting one widely cited nomogram; many
//! institutions use a locally adapted nomogram or anti-Xa-based monitoring
//! instead. Verify against local protocol before use.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "heparin_nomogram";
pub const REFERENCE: &str = "Raschke RA, Reilly BM, Guidry JR, Fontana JR, Srinivas S. The weight-based heparin dosing nomogram compared with a \"standard care\" nomogram. A randomized controlled trial. Ann Intern Med. 1993;119(9):874-881.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain nomogram - implemented from the primary literature",
    "https://doi.org/10.7326/0003-4819-119-9-199311010-00002",
);

const INITIAL_BOLUS_UNITS_PER_KG: f64 = 80.0;
const INITIAL_INFUSION_UNITS_PER_KG_PER_HR: f64 = 18.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeparinNomogramInput {
    pub weight_kg: f64,
    /// aPTT expressed as a ratio to the laboratory's control value. Omit for
    /// the initial (pre-first-aPTT) dose.
    #[serde(default)]
    pub aptt_ratio: Option<f64>,
    /// Current infusion rate, units/kg/hr. Required when aptt_ratio is
    /// supplied, since the adjustment is a delta from the current rate.
    #[serde(default)]
    pub current_infusion_rate_units_kg_hr: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdjustmentAction {
    ReboluAndIncrease {
        bolus_units_kg: f64,
        increase_units_kg_hr: f64,
    },
    IncreaseOnly {
        increase_units_kg_hr: f64,
    },
    NoChange,
    DecreaseOnly {
        decrease_units_kg_hr: f64,
    },
    HoldThenDecrease {
        hold_minutes: u32,
        decrease_units_kg_hr: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeparinNomogramOutcome {
    pub initial_bolus_units: f64,
    pub initial_infusion_rate_units_hr: f64,
    pub adjustment: Option<AdjustmentAction>,
    pub adjusted_infusion_rate_units_hr: Option<f64>,
}

pub fn compute(input: &HeparinNomogramInput) -> Result<HeparinNomogramOutcome, CalcError> {
    validate(input.weight_kg, "weight_kg", 20.0, 300.0)?;
    if let Some(ratio) = input.aptt_ratio {
        validate(ratio, "aptt_ratio", 0.3, 10.0)?;
        if input.current_infusion_rate_units_kg_hr.is_none() {
            return Err(CalcError::InvalidInput(
                "current_infusion_rate_units_kg_hr is required when aptt_ratio is supplied"
                    .to_string(),
            ));
        }
    }
    if let Some(rate) = input.current_infusion_rate_units_kg_hr {
        validate(rate, "current_infusion_rate_units_kg_hr", 0.0, 50.0)?;
    }

    let initial_bolus_units = INITIAL_BOLUS_UNITS_PER_KG * input.weight_kg;
    let initial_infusion_rate_units_hr = INITIAL_INFUSION_UNITS_PER_KG_PER_HR * input.weight_kg;

    let (adjustment, adjusted_infusion_rate_units_hr) = match input.aptt_ratio {
        None => (None, None),
        Some(ratio) => {
            let current_rate_units_kg_hr = input.current_infusion_rate_units_kg_hr.unwrap();
            let action = if ratio < 1.2 {
                AdjustmentAction::ReboluAndIncrease {
                    bolus_units_kg: 80.0,
                    increase_units_kg_hr: 4.0,
                }
            } else if ratio < 1.5 {
                AdjustmentAction::ReboluAndIncrease {
                    bolus_units_kg: 40.0,
                    increase_units_kg_hr: 2.0,
                }
            } else if ratio <= 2.3 {
                AdjustmentAction::NoChange
            } else if ratio <= 3.0 {
                AdjustmentAction::DecreaseOnly {
                    decrease_units_kg_hr: 2.0,
                }
            } else {
                AdjustmentAction::HoldThenDecrease {
                    hold_minutes: 60,
                    decrease_units_kg_hr: 3.0,
                }
            };
            let new_rate_units_kg_hr = match action {
                AdjustmentAction::ReboluAndIncrease {
                    increase_units_kg_hr,
                    ..
                } => current_rate_units_kg_hr + increase_units_kg_hr,
                AdjustmentAction::IncreaseOnly {
                    increase_units_kg_hr,
                } => current_rate_units_kg_hr + increase_units_kg_hr,
                AdjustmentAction::NoChange => current_rate_units_kg_hr,
                AdjustmentAction::DecreaseOnly {
                    decrease_units_kg_hr,
                } => (current_rate_units_kg_hr - decrease_units_kg_hr).max(0.0),
                AdjustmentAction::HoldThenDecrease {
                    decrease_units_kg_hr,
                    ..
                } => (current_rate_units_kg_hr - decrease_units_kg_hr).max(0.0),
            };
            (Some(action), Some(new_rate_units_kg_hr * input.weight_kg))
        }
    };

    Ok(HeparinNomogramOutcome {
        initial_bolus_units,
        initial_infusion_rate_units_hr,
        adjustment,
        adjusted_infusion_rate_units_hr,
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

fn action_text(action: AdjustmentAction) -> String {
    match action {
        AdjustmentAction::ReboluAndIncrease {
            bolus_units_kg,
            increase_units_kg_hr,
        } => format!(
            "sub-therapeutic: rebolus {bolus_units_kg:.0} units/kg and increase the infusion by {increase_units_kg_hr:.0} units/kg/hr"
        ),
        AdjustmentAction::IncreaseOnly {
            increase_units_kg_hr,
        } => format!("increase the infusion by {increase_units_kg_hr:.0} units/kg/hr"),
        AdjustmentAction::NoChange => "therapeutic range: no change".to_string(),
        AdjustmentAction::DecreaseOnly {
            decrease_units_kg_hr,
        } => format!(
            "supra-therapeutic: decrease the infusion by {decrease_units_kg_hr:.0} units/kg/hr"
        ),
        AdjustmentAction::HoldThenDecrease {
            hold_minutes,
            decrease_units_kg_hr,
        } => format!(
            "markedly supra-therapeutic: hold the infusion for {hold_minutes} minutes, then decrease by {decrease_units_kg_hr:.0} units/kg/hr"
        ),
    }
}

pub fn build_response(input: &HeparinNomogramInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "initial_infusion_rate_units_hr".into(),
        json!(round0(o.initial_infusion_rate_units_hr)),
    );
    if let Some(adjusted) = o.adjusted_infusion_rate_units_hr {
        working.insert(
            "adjusted_infusion_rate_units_hr".into(),
            json!(round0(adjusted)),
        );
    }

    let interpretation = match o.adjustment {
        None => format!(
            "Initial dose: bolus {:.0} units, then infuse at {:.0} units/hr ({} units/kg/hr). Check the first aPTT 6 hours after starting and apply the nomogram adjustment from that result. This is a calculation aid reflecting the Raschke nomogram; verify target range and adjustment steps against local protocol.",
            o.initial_bolus_units,
            o.initial_infusion_rate_units_hr,
            INITIAL_INFUSION_UNITS_PER_KG_PER_HR
        ),
        Some(action) => format!(
            "aPTT ratio {:.1}x control: {}. New infusion rate approximately {:.0} units/hr. Recheck aPTT 6 hours after any dose change; once two consecutive therapeutic results are obtained, local protocol typically allows less frequent monitoring (e.g. daily). This is a calculation aid; verify against local protocol.",
            input.aptt_ratio.unwrap(),
            action_text(action),
            o.adjusted_infusion_rate_units_hr.unwrap_or(0.0)
        ),
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round0(
            o.adjusted_infusion_rate_units_hr
                .unwrap_or(o.initial_infusion_rate_units_hr)
        )),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round0(v: f64) -> f64 {
    v.round()
}

pub struct HeparinNomogram;
impl Calculator for HeparinNomogram {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Weight-Based Heparin Nomogram"
    }
    fn description(&self) -> &'static str {
        "Weight-based unfractionated-heparin initial dosing and aPTT-ratio-guided adjustment (Raschke nomogram)."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "HeparinNomogramInput", "type": "object", "additionalProperties": false,
            "required": ["weight_kg"],
            "properties": {
                "weight_kg": { "type": "number", "minimum": 20, "maximum": 300 },
                "aptt_ratio": { "type": "number", "minimum": 0.3, "maximum": 10, "description": "aPTT as a ratio to the laboratory's control; omit for the initial (pre-first-aPTT) dose" },
                "current_infusion_rate_units_kg_hr": { "type": "number", "minimum": 0, "maximum": 50, "description": "Required when aptt_ratio is supplied" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: HeparinNomogramInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_dose_for_80kg() {
        let out = compute(&HeparinNomogramInput {
            weight_kg: 80.0,
            aptt_ratio: None,
            current_infusion_rate_units_kg_hr: None,
        })
        .unwrap();
        assert_eq!(out.initial_bolus_units, 6400.0);
        assert_eq!(out.initial_infusion_rate_units_hr, 1440.0);
        assert!(out.adjustment.is_none());
    }

    #[test]
    fn therapeutic_ratio_no_change() {
        let out = compute(&HeparinNomogramInput {
            weight_kg: 80.0,
            aptt_ratio: Some(2.0),
            current_infusion_rate_units_kg_hr: Some(18.0),
        })
        .unwrap();
        assert_eq!(out.adjustment, Some(AdjustmentAction::NoChange));
        assert_eq!(out.adjusted_infusion_rate_units_hr, Some(1440.0));
    }

    #[test]
    fn low_ratio_rebolu_and_increase() {
        let out = compute(&HeparinNomogramInput {
            weight_kg: 80.0,
            aptt_ratio: Some(1.0),
            current_infusion_rate_units_kg_hr: Some(18.0),
        })
        .unwrap();
        assert!(matches!(
            out.adjustment,
            Some(AdjustmentAction::ReboluAndIncrease {
                bolus_units_kg: 80.0,
                increase_units_kg_hr: 4.0
            })
        ));
        // (18 + 4) * 80 = 1760
        assert_eq!(out.adjusted_infusion_rate_units_hr, Some(1760.0));
    }

    #[test]
    fn very_high_ratio_holds_then_decreases() {
        let out = compute(&HeparinNomogramInput {
            weight_kg: 80.0,
            aptt_ratio: Some(3.5),
            current_infusion_rate_units_kg_hr: Some(18.0),
        })
        .unwrap();
        assert!(matches!(
            out.adjustment,
            Some(AdjustmentAction::HoldThenDecrease {
                hold_minutes: 60,
                decrease_units_kg_hr: 3.0
            })
        ));
        // (18 - 3) * 80 = 1200
        assert_eq!(out.adjusted_infusion_rate_units_hr, Some(1200.0));
    }

    #[test]
    fn requires_current_rate_when_ratio_supplied() {
        assert!(
            compute(&HeparinNomogramInput {
                weight_kg: 80.0,
                aptt_ratio: Some(2.0),
                current_infusion_rate_units_kg_hr: None,
            })
            .is_err()
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"weight_kg": 80.0});
        let typed = HeparinNomogramInput {
            weight_kg: 80.0,
            aptt_ratio: None,
            current_infusion_rate_units_kg_hr: None,
        };
        let dynamic = HeparinNomogram.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(HeparinNomogram.calculate(&json!({})).is_err());
    }
}
