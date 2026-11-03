// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Transferrin saturation from serum iron and total iron-binding capacity
//! (TIBC), for iron-deficiency and haemochromatosis workup.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "transferrin_saturation";
pub const REFERENCE: &str = "Standard clinical chemistry ratio (serum iron / TIBC x 100). The >45% hereditary-haemochromatosis screening threshold is per Kowdley KV, Brown KE, Ahn J, Sundaram V. ACG Clinical Guideline: Hereditary Hemochromatosis. Am J Gastroenterol. 2019;114(8):1202-1218.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain formula and guideline threshold - implemented from standard clinical chemistry and the cited guideline",
    "https://doi.org/10.14309/ajg.0000000000000315",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IronUnit {
    #[serde(rename = "ug/dL")]
    UgDl,
    #[serde(rename = "umol/L")]
    UmolL,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferrinSaturationInput {
    /// Serum iron, in the unit given by `unit`.
    pub serum_iron: f64,
    /// Total iron-binding capacity, in the same unit as `serum_iron`.
    pub tibc: f64,
    /// Unit shared by both `serum_iron` and `tibc`; the ratio is
    /// unit-independent as long as both use the same one.
    pub unit: IronUnit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransferrinSaturationOutcome {
    pub saturation_percent: f64,
    pub band: &'static str,
}

pub fn compute(
    input: &TransferrinSaturationInput,
) -> Result<TransferrinSaturationOutcome, CalcError> {
    let (iron_min, iron_max, tibc_min, tibc_max) = match input.unit {
        IronUnit::UgDl => (5.0, 500.0, 50.0, 700.0),
        IronUnit::UmolL => (0.9, 89.5, 9.0, 125.0),
    };
    validate(input.serum_iron, "serum_iron", iron_min, iron_max)?;
    validate(input.tibc, "tibc", tibc_min, tibc_max)?;
    if input.serum_iron > input.tibc {
        return Err(CalcError::InvalidInput(
            "serum_iron cannot exceed tibc".to_string(),
        ));
    }

    let saturation_percent = input.serum_iron / input.tibc * 100.0;
    let band = if saturation_percent > 45.0 {
        "raised (>45%) - meets the usual haemochromatosis screening threshold"
    } else if saturation_percent < 20.0 {
        "low (<20%) - consistent with iron deficiency"
    } else {
        "within the usual 20-45% reference range"
    };

    Ok(TransferrinSaturationOutcome {
        saturation_percent,
        band,
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

pub fn build_response(
    input: &TransferrinSaturationInput,
) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("band".into(), json!(o.band));
    working.insert("unit".into(), json!("percent"));

    let interpretation = format!(
        "Transferrin saturation {:.1}% ({}). A raised value needs confirmation (typically with ferritin and repeat fasting sample) before a haemochromatosis workup; a low value should be interpreted alongside ferritin, as inflammation lowers TIBC independently of iron status.",
        o.saturation_percent, o.band
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.saturation_percent)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct TransferrinSaturation;
impl Calculator for TransferrinSaturation {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Transferrin Saturation"
    }
    fn description(&self) -> &'static str {
        "Transferrin saturation from serum iron and TIBC, for iron-deficiency and haemochromatosis workup."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "TransferrinSaturationInput", "type": "object", "additionalProperties": false,
            "required": ["serum_iron", "tibc", "unit"],
            "properties": {
                "serum_iron": { "type": "number", "description": "Serum iron, in the unit given by unit" },
                "tibc": { "type": "number", "description": "Total iron-binding capacity, same unit as serum_iron" },
                "unit": { "type": "string", "enum": ["ug/dL", "umol/L"], "description": "Unit shared by serum_iron and tibc" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: TransferrinSaturationInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_range() {
        let out = compute(&TransferrinSaturationInput {
            serum_iron: 100.0,
            tibc: 350.0,
            unit: IronUnit::UgDl,
        })
        .unwrap();
        assert!((out.saturation_percent - 28.571_428_571_428_57).abs() < 1e-9);
        assert_eq!(out.band, "within the usual 20-45% reference range");
    }

    #[test]
    fn raised_haemochromatosis_threshold() {
        let out = compute(&TransferrinSaturationInput {
            serum_iron: 180.0,
            tibc: 300.0,
            unit: IronUnit::UgDl,
        })
        .unwrap();
        assert_eq!(out.saturation_percent, 60.0);
        assert_eq!(
            out.band,
            "raised (>45%) - meets the usual haemochromatosis screening threshold"
        );
    }

    #[test]
    fn rejects_iron_exceeding_tibc() {
        assert!(
            compute(&TransferrinSaturationInput {
                serum_iron: 400.0,
                tibc: 300.0,
                unit: IronUnit::UgDl,
            })
            .is_err()
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"serum_iron": 100.0, "tibc": 350.0, "unit": "ug/dL"});
        let typed = TransferrinSaturationInput {
            serum_iron: 100.0,
            tibc: 350.0,
            unit: IronUnit::UgDl,
        };
        let dynamic = TransferrinSaturation.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            TransferrinSaturation
                .calculate(&json!({"serum_iron": 100.0, "tibc": 350.0}))
                .is_err()
        );
    }
}
