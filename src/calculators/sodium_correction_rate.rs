// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Predicted change in serum sodium per litre of a given infusate, for
//! hyponatraemia correction-rate planning (the Adrogue-Madias formula).
//!
//! This is a static, single-litre prediction, not a fluid prescription or an
//! infusion-rate titration tool: actual response depends on ongoing losses,
//! intake, and physiological compensation, and must be followed with
//! frequent repeat sodium measurement.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "sodium_correction_rate";
pub const REFERENCE: &str = "Adrogue HJ, Madias NE. Hyponatremia. N Engl J Med. 2000;342(21):1581-1589. doi:10.1056/NEJM200005253422107. Maximum safe correction-rate guidance per Spasovski G, Vanholder R, Allolio B, et al. Clinical practice guideline on diagnosis and treatment of hyponatraemia. Eur J Endocrinol. 2014;170(3):G1-G47.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Published clinical calculation method - independently implemented from the literature",
    "https://doi.org/10.1056/NEJM200005253422107",
);

const LIMITATIONS: &str = "This is a static, single-litre prediction, not a fluid prescription. It excludes ongoing and insensible losses, oral/other intake, and physiological compensation (e.g. ADH suppression as sodium rises, which can cause faster-than-predicted correction). Over-rapid correction of chronic or chronic-risk hyponatraemia risks osmotic demyelination syndrome: widely cited guideline limits are a maximum of 8 mmol/L rise per 24 hours (lower, around 4-6 mmol/L, in patients at higher risk - e.g. chronic alcohol use, malnutrition, liver disease, severe hypokalaemia). Reassess serum sodium frequently (e.g. every 2-4 hours initially) and adjust or stop the infusion well before the 24-hour limit is reached.";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SodiumCorrectionRateInput {
    pub weight_kg: f64,
    pub current_sodium_mmol_l: f64,
    pub infusate_sodium_mmol_l: f64,
    #[serde(default)]
    pub infusate_potassium_mmol_l: f64,
    pub total_body_water_fraction: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SodiumCorrectionRateOutcome {
    pub estimated_total_body_water_l: f64,
    pub predicted_change_per_litre_mmol_l: f64,
}

pub fn compute(
    input: &SodiumCorrectionRateInput,
) -> Result<SodiumCorrectionRateOutcome, CalcError> {
    for (name, value) in [
        ("weight_kg", input.weight_kg),
        ("current_sodium_mmol_l", input.current_sodium_mmol_l),
        ("infusate_sodium_mmol_l", input.infusate_sodium_mmol_l),
        ("infusate_potassium_mmol_l", input.infusate_potassium_mmol_l),
        ("total_body_water_fraction", input.total_body_water_fraction),
    ] {
        if !value.is_finite() {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be a finite number"
            )));
        }
    }
    if input.weight_kg <= 0.0 {
        return Err(CalcError::InvalidInput("weight_kg must be positive".into()));
    }
    if !(100.0..135.0).contains(&input.current_sodium_mmol_l) {
        return Err(CalcError::InvalidInput(
            "current_sodium_mmol_l must be between 100 and 135 for hyponatraemia correction planning".into(),
        ));
    }
    if !(0.0..=250.0).contains(&input.infusate_sodium_mmol_l) {
        return Err(CalcError::InvalidInput(
            "infusate_sodium_mmol_l must be between 0 and 250".into(),
        ));
    }
    if !(0.0..=80.0).contains(&input.infusate_potassium_mmol_l) {
        return Err(CalcError::InvalidInput(
            "infusate_potassium_mmol_l must be between 0 and 80".into(),
        ));
    }
    if !(0.3..=0.7).contains(&input.total_body_water_fraction) {
        return Err(CalcError::InvalidInput(
            "total_body_water_fraction must be between 0.3 and 0.7".into(),
        ));
    }

    let estimated_total_body_water_l = input.weight_kg * input.total_body_water_fraction;
    let predicted_change_per_litre_mmol_l = (input.infusate_sodium_mmol_l
        + input.infusate_potassium_mmol_l
        - input.current_sodium_mmol_l)
        / (estimated_total_body_water_l + 1.0);

    if !predicted_change_per_litre_mmol_l.is_finite() {
        return Err(CalcError::InvalidInput(
            "inputs produce a non-finite result".into(),
        ));
    }

    Ok(SodiumCorrectionRateOutcome {
        estimated_total_body_water_l,
        predicted_change_per_litre_mmol_l,
    })
}

pub fn build_response(input: &SodiumCorrectionRateInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let rounded = (o.predicted_change_per_litre_mmol_l * 10.0).round() / 10.0;
    let mut working = Map::new();
    working.insert(
        "estimated_total_body_water_l".into(),
        json!(round1(o.estimated_total_body_water_l)),
    );
    working.insert("unit".into(), json!("mmol/L per litre infused"));

    let direction = if rounded >= 0.0 { "rise" } else { "fall" };
    let interpretation = format!(
        "Predicted serum sodium {direction} of {:.1} mmol/L per litre of this infusate, using an estimated total body water of {:.1} L (weight x {:.2}). {LIMITATIONS}",
        rounded.abs(),
        o.estimated_total_body_water_l,
        input.total_body_water_fraction
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(rounded),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct SodiumCorrectionRate;
impl Calculator for SodiumCorrectionRate {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Sodium Correction Rate (Adrogue-Madias)"
    }
    fn description(&self) -> &'static str {
        "Predicted change in serum sodium per litre of a given infusate, for hyponatraemia correction-rate planning."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "SodiumCorrectionRateInput", "type": "object", "additionalProperties": false,
            "required": ["weight_kg", "current_sodium_mmol_l", "infusate_sodium_mmol_l", "total_body_water_fraction"],
            "properties": {
                "weight_kg": { "type": "number", "exclusiveMinimum": 0 },
                "current_sodium_mmol_l": { "type": "number", "minimum": 100, "maximum": 134.999, "description": "Hyponatraemic range (100-135)" },
                "infusate_sodium_mmol_l": { "type": "number", "minimum": 0, "maximum": 250 },
                "infusate_potassium_mmol_l": { "type": "number", "minimum": 0, "maximum": 80, "description": "Optional; defaults to 0 for a potassium-free infusate" },
                "total_body_water_fraction": { "type": "number", "minimum": 0.3, "maximum": 0.7, "description": "Selected from clinical assessment (e.g. ~0.6 adult male, ~0.5 adult female, lower in older age), not inferred automatically" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: SodiumCorrectionRateInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_saline_raises_sodium() {
        // Adrogue-Madias worked example: 70kg male, Na 112, 0.6 TBW fraction, 0.9% saline (Na 154).
        // TBW = 42; delta = (154 - 112) / 43 = 0.977 mmol/L per litre.
        let out = compute(&SodiumCorrectionRateInput {
            weight_kg: 70.0,
            current_sodium_mmol_l: 112.0,
            infusate_sodium_mmol_l: 154.0,
            infusate_potassium_mmol_l: 0.0,
            total_body_water_fraction: 0.6,
        })
        .unwrap();
        assert!((out.estimated_total_body_water_l - 42.0).abs() < 1e-9);
        assert!((out.predicted_change_per_litre_mmol_l - 0.977).abs() < 0.001);
    }

    #[test]
    fn hypotonic_fluid_can_lower_sodium_further() {
        // 5% dextrose (Na 0, K 0) given to a hyponatraemic patient drops sodium further.
        let out = compute(&SodiumCorrectionRateInput {
            weight_kg: 70.0,
            current_sodium_mmol_l: 120.0,
            infusate_sodium_mmol_l: 0.0,
            infusate_potassium_mmol_l: 0.0,
            total_body_water_fraction: 0.6,
        })
        .unwrap();
        assert!(out.predicted_change_per_litre_mmol_l < 0.0);
    }

    #[test]
    fn potassium_contributes_like_sodium() {
        let with_k = compute(&SodiumCorrectionRateInput {
            weight_kg: 70.0,
            current_sodium_mmol_l: 120.0,
            infusate_sodium_mmol_l: 0.0,
            infusate_potassium_mmol_l: 40.0,
            total_body_water_fraction: 0.6,
        })
        .unwrap();
        let without_k = compute(&SodiumCorrectionRateInput {
            weight_kg: 70.0,
            current_sodium_mmol_l: 120.0,
            infusate_sodium_mmol_l: 0.0,
            infusate_potassium_mmol_l: 0.0,
            total_body_water_fraction: 0.6,
        })
        .unwrap();
        assert!(
            with_k.predicted_change_per_litre_mmol_l > without_k.predicted_change_per_litre_mmol_l
        );
    }

    #[test]
    fn rejects_sodium_outside_hyponatraemic_range() {
        assert!(
            compute(&SodiumCorrectionRateInput {
                weight_kg: 70.0,
                current_sodium_mmol_l: 140.0,
                infusate_sodium_mmol_l: 154.0,
                infusate_potassium_mmol_l: 0.0,
                total_body_water_fraction: 0.6,
            })
            .is_err()
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "weight_kg": 70.0, "current_sodium_mmol_l": 112.0, "infusate_sodium_mmol_l": 154.0, "total_body_water_fraction": 0.6
        });
        let typed = SodiumCorrectionRateInput {
            weight_kg: 70.0,
            current_sodium_mmol_l: 112.0,
            infusate_sodium_mmol_l: 154.0,
            infusate_potassium_mmol_l: 0.0,
            total_body_water_fraction: 0.6,
        };
        let dynamic = SodiumCorrectionRate.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            SodiumCorrectionRate
                .calculate(&json!({"weight_kg": 70.0}))
                .is_err()
        );
    }
}
