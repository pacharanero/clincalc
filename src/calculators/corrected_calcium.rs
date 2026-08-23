// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Albumin-corrected calcium using the Payne-style correction.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "corrected_calcium";
pub const REFERENCE: &str = "Payne RB, Little AJ, Williams RB, Milner JR. Interpretation of serum calcium in patients with abnormal serum proteins. Br Med J. 1973;4(5893):643-646. doi:10.1136/bmj.4.5893.643";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1136/bmj.4.5893.643",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalciumUnit {
    #[serde(rename = "mmol/L")]
    MmolL,
    #[serde(rename = "mg/dL")]
    MgDl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlbuminUnit {
    #[serde(rename = "g/L")]
    GL,
    #[serde(rename = "g/dL")]
    GDl,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorrectedCalciumInput {
    pub calcium: f64,
    pub calcium_unit: CalciumUnit,
    pub albumin: f64,
    pub albumin_unit: AlbuminUnit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CorrectedCalciumOutcome {
    pub corrected_calcium: f64,
    pub corrected_unit: CalciumUnit,
    pub albumin_g_l: f64,
    pub interpretation: String,
}

pub fn compute(input: &CorrectedCalciumInput) -> Result<CorrectedCalciumOutcome, CalcError> {
    if !input.calcium.is_finite() || input.calcium <= 0.0 {
        return Err(CalcError::InvalidInput(
            "calcium must be a positive finite number".into(),
        ));
    }
    if !input.albumin.is_finite() || input.albumin <= 0.0 {
        return Err(CalcError::InvalidInput(
            "albumin must be a positive finite number".into(),
        ));
    }

    let albumin_g_l = match input.albumin_unit {
        AlbuminUnit::GL => input.albumin,
        AlbuminUnit::GDl => input.albumin * 10.0,
    };
    if !(10.0..=70.0).contains(&albumin_g_l) {
        return Err(CalcError::InvalidInput(
            "albumin must be between 10 and 70 g/L (or equivalent)".into(),
        ));
    }

    let corrected_calcium = match input.calcium_unit {
        CalciumUnit::MmolL => input.calcium + 0.02 * (40.0 - albumin_g_l),
        CalciumUnit::MgDl => input.calcium + 0.8 * (4.0 - albumin_g_l / 10.0),
    };

    let unit = match input.calcium_unit {
        CalciumUnit::MmolL => "mmol/L",
        CalciumUnit::MgDl => "mg/dL",
    };
    let interpretation = format!(
        "Albumin-corrected calcium is {:.2} {unit}. This correction estimates what total calcium might be at albumin 40 g/L (4 g/dL); ionised calcium is preferred when calcium status is clinically uncertain, especially in critical illness or acid-base disturbance.",
        corrected_calcium
    );

    Ok(CorrectedCalciumOutcome {
        corrected_calcium,
        corrected_unit: input.calcium_unit,
        albumin_g_l,
        interpretation,
    })
}

pub fn build_response(input: &CorrectedCalciumInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("albumin_g_l".into(), json!(o.albumin_g_l));
    working.insert(
        "formula".into(),
        json!(match o.corrected_unit {
            CalciumUnit::MmolL => "calcium_mmol_l + 0.02 * (40 - albumin_g_l)",
            CalciumUnit::MgDl => "calcium_mg_dl + 0.8 * (4 - albumin_g_dl)",
        }),
    );
    working.insert(
        "unit".into(),
        json!(match o.corrected_unit {
            CalciumUnit::MmolL => "mmol/L",
            CalciumUnit::MgDl => "mg/dL",
        }),
    );
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!((o.corrected_calcium * 100.0).round() / 100.0),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct CorrectedCalcium;

impl Calculator for CorrectedCalcium {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Albumin-corrected Calcium"
    }
    fn description(&self) -> &'static str {
        "Corrects total serum calcium for abnormal albumin using the Payne-style correction."
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
            "title": "CorrectedCalciumInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["calcium", "calcium_unit", "albumin", "albumin_unit"],
            "properties": {
                "calcium": { "type": "number", "exclusiveMinimum": 0, "description": "Measured total calcium" },
                "calcium_unit": { "type": "string", "enum": ["mmol/L", "mg/dL"] },
                "albumin": { "type": "number", "exclusiveMinimum": 0, "description": "Serum albumin" },
                "albumin_unit": { "type": "string", "enum": ["g/L", "g/dL"] }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: CorrectedCalciumInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrects_mmol_l_payne_formula() {
        let out = compute(&CorrectedCalciumInput {
            calcium: 2.0,
            calcium_unit: CalciumUnit::MmolL,
            albumin: 30.0,
            albumin_unit: AlbuminUnit::GL,
        })
        .unwrap();
        assert_eq!((out.corrected_calcium * 100.0).round() / 100.0, 2.20);
    }

    #[test]
    fn mgdl_formula_matches_common_equivalent() {
        let out = compute(&CorrectedCalciumInput {
            calcium: 8.0,
            calcium_unit: CalciumUnit::MgDl,
            albumin: 3.0,
            albumin_unit: AlbuminUnit::GDl,
        })
        .unwrap();
        assert_eq!((out.corrected_calcium * 10.0).round() / 10.0, 8.8);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let typed = CorrectedCalciumInput {
            calcium: 2.0,
            calcium_unit: CalciumUnit::MmolL,
            albumin: 30.0,
            albumin_unit: AlbuminUnit::GL,
        };
        let value = json!({"calcium": 2.0, "calcium_unit": "mmol/L", "albumin": 30.0, "albumin_unit": "g/L"});
        let dynamic = CorrectedCalcium.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            CorrectedCalcium
                .calculate(&json!({"calcium": 2.0, "calcium_unit": "mmol/L"}))
                .is_err()
        );
    }
}
