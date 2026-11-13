// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Calculated Serum Osmolality and Osmolar Gap.
//!
//! Calculated osmolality = 2 x [Na+] + [glucose] + [urea], all in mmol/L
//! (SI units). When a measured osmolality (from freezing-point depression)
//! is also supplied, the osmolar gap (measured minus calculated) is
//! reported; a gap over 10 mOsm/kg suggests an unmeasured osmotically
//! active substance, classically a toxic alcohol.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "serum_osmolality";

pub const REFERENCE: &str = "Standard calculated-osmolality formula (2 x Na + glucose + urea, SI units); \
osmolar-gap interpretation per Glasser L, Sternglanz PD, Combie J, Robinson A. Serum osmolality and its \
applicability to drug overdose. Am J Clin Pathol. 1973;60(5):695-699.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.1093/ajcp/60.5.695",
);

const NORMAL_OSMOLAR_GAP_UPPER_LIMIT: f64 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SerumOsmolalityInput {
    pub sodium_mmol_l: f64,
    pub glucose_mmol_l: f64,
    pub urea_mmol_l: f64,
    /// Measured serum osmolality (freezing-point depression), mOsm/kg. If
    /// supplied, the osmolar gap is computed.
    #[serde(default)]
    pub measured_osmolality_mosm_kg: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SerumOsmolalityOutcome {
    pub calculated_osmolality: f64,
    pub osmolar_gap: Option<f64>,
    pub osmolar_gap_elevated: Option<bool>,
}

pub fn compute(input: &SerumOsmolalityInput) -> Result<SerumOsmolalityOutcome, CalcError> {
    for (name, value) in [
        ("sodium_mmol_l", input.sodium_mmol_l),
        ("glucose_mmol_l", input.glucose_mmol_l),
        ("urea_mmol_l", input.urea_mmol_l),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be a non-negative finite number"
            )));
        }
    }
    if let Some(measured) = input.measured_osmolality_mosm_kg
        && (!measured.is_finite() || measured < 0.0)
    {
        return Err(CalcError::InvalidInput(
            "measured_osmolality_mosm_kg must be a non-negative finite number".into(),
        ));
    }

    let calculated_osmolality =
        2.0 * input.sodium_mmol_l + input.glucose_mmol_l + input.urea_mmol_l;
    let osmolar_gap = input
        .measured_osmolality_mosm_kg
        .map(|measured| measured - calculated_osmolality);
    let osmolar_gap_elevated = osmolar_gap.map(|gap| gap > NORMAL_OSMOLAR_GAP_UPPER_LIMIT);

    Ok(SerumOsmolalityOutcome {
        calculated_osmolality,
        osmolar_gap,
        osmolar_gap_elevated,
    })
}

pub fn build_response(input: &SerumOsmolalityInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "calculated_osmolality_mosm_kg".into(),
        json!(round1(o.calculated_osmolality)),
    );
    if let Some(gap) = o.osmolar_gap {
        working.insert("osmolar_gap_mosm_kg".into(), json!(round1(gap)));
    }
    if let Some(elevated) = o.osmolar_gap_elevated {
        working.insert("osmolar_gap_elevated".into(), json!(elevated));
    }

    let interpretation = match (o.osmolar_gap, o.osmolar_gap_elevated) {
        (Some(gap), Some(true)) => format!(
            "Calculated osmolality {:.1} mOsm/kg; osmolar gap {:.1} mOsm/kg (elevated, normal < {:.0}). An elevated gap suggests an unmeasured osmotically active substance - classically ethanol, methanol, isopropyl alcohol, or ethylene glycol - but is also seen in ketoacidosis, lactic acidosis, and renal failure.",
            o.calculated_osmolality, gap, NORMAL_OSMOLAR_GAP_UPPER_LIMIT
        ),
        (Some(gap), Some(false)) => format!(
            "Calculated osmolality {:.1} mOsm/kg; osmolar gap {:.1} mOsm/kg (not elevated, normal < {:.0}). A normal gap does not fully exclude toxic alcohol ingestion.",
            o.calculated_osmolality, gap, NORMAL_OSMOLAR_GAP_UPPER_LIMIT
        ),
        _ => format!(
            "Calculated osmolality {:.1} mOsm/kg. Supply measured_osmolality_mosm_kg to also compute the osmolar gap.",
            o.calculated_osmolality
        ),
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.calculated_osmolality)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct SerumOsmolality;
impl Calculator for SerumOsmolality {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Calculated Serum Osmolality / Osmolar Gap"
    }
    fn description(&self) -> &'static str {
        "Calculated serum osmolality from sodium, glucose, and urea, with an optional osmolar gap against a measured value."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "SerumOsmolalityInput", "type": "object",
            "additionalProperties": false, "required": ["sodium_mmol_l", "glucose_mmol_l", "urea_mmol_l"],
            "properties": {
                "sodium_mmol_l": { "type": "number", "minimum": 0, "maximum": 220, "description": "Serum sodium in mmol/L" },
                "glucose_mmol_l": { "type": "number", "minimum": 0, "maximum": 100, "description": "Serum glucose in mmol/L" },
                "urea_mmol_l": { "type": "number", "minimum": 0, "maximum": 100, "description": "Serum urea in mmol/L" },
                "measured_osmolality_mosm_kg": { "type": "number", "minimum": 0, "maximum": 500, "description": "Optional measured serum osmolality (freezing-point depression) in mOsm/kg, to compute the osmolar gap" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: SerumOsmolalityInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> SerumOsmolalityInput {
        SerumOsmolalityInput {
            sodium_mmol_l: 140.0,
            glucose_mmol_l: 5.0,
            urea_mmol_l: 5.0,
            measured_osmolality_mosm_kg: None,
        }
    }

    #[test]
    fn calculated_osmolality_matches_formula() {
        let o = compute(&base()).unwrap();
        assert_eq!(o.calculated_osmolality, 290.0); // 2*140 + 5 + 5
        assert_eq!(o.osmolar_gap, None);
    }

    #[test]
    fn normal_gap_when_measured_matches_calculated() {
        let mut i = base();
        i.measured_osmolality_mosm_kg = Some(295.0);
        let o = compute(&i).unwrap();
        assert_eq!(o.osmolar_gap, Some(5.0));
        assert_eq!(o.osmolar_gap_elevated, Some(false));
    }

    #[test]
    fn elevated_gap_above_ten() {
        let mut i = base();
        i.measured_osmolality_mosm_kg = Some(320.0);
        let o = compute(&i).unwrap();
        assert_eq!(o.osmolar_gap, Some(30.0));
        assert_eq!(o.osmolar_gap_elevated, Some(true));
    }

    #[test]
    fn rejects_negative_values() {
        let mut i = base();
        i.sodium_mmol_l = -1.0;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let mut typed = base();
        typed.measured_osmolality_mosm_kg = Some(310.0);
        let value = json!({
            "sodium_mmol_l": 140.0, "glucose_mmol_l": 5.0, "urea_mmol_l": 5.0,
            "measured_osmolality_mosm_kg": 310.0
        });
        let dynamic = SerumOsmolality.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(290.0));
    }
}
