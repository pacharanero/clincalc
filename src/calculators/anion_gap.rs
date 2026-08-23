// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Serum anion gap, with optional potassium and albumin correction.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "anion_gap";
pub const REFERENCE: &str = "Emmett M, Narins RG. Clinical use of the anion gap. Medicine (Baltimore). 1977;56(1):38-54. Albumin correction commonly attributed to Figge J, Jabor A, Kazda A, Fencl V. Anion gap and hypoalbuminemia. Crit Care Med. 1998;26(11):1807-1810.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://pubmed.ncbi.nlm.nih.gov/830929/",
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnionGapInput {
    pub sodium_mmol_l: f64,
    pub chloride_mmol_l: f64,
    pub bicarbonate_mmol_l: f64,
    #[serde(default)]
    pub potassium_mmol_l: Option<f64>,
    #[serde(default)]
    pub albumin_g_l: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnionGapOutcome {
    pub anion_gap: f64,
    pub albumin_corrected_anion_gap: Option<f64>,
    pub interpretation: String,
}

pub fn compute(input: &AnionGapInput) -> Result<AnionGapOutcome, CalcError> {
    validate(input.sodium_mmol_l, "sodium_mmol_l", 80.0, 220.0)?;
    validate(input.chloride_mmol_l, "chloride_mmol_l", 50.0, 160.0)?;
    validate(input.bicarbonate_mmol_l, "bicarbonate_mmol_l", 1.0, 60.0)?;
    if let Some(k) = input.potassium_mmol_l {
        validate(k, "potassium_mmol_l", 1.0, 10.0)?;
    }
    if let Some(albumin) = input.albumin_g_l {
        validate(albumin, "albumin_g_l", 5.0, 70.0)?;
    }

    let anion_gap = input.sodium_mmol_l + input.potassium_mmol_l.unwrap_or(0.0)
        - input.chloride_mmol_l
        - input.bicarbonate_mmol_l;
    let corrected = input
        .albumin_g_l
        .map(|albumin| anion_gap + 0.25 * (40.0 - albumin));
    let value = corrected.unwrap_or(anion_gap);
    let band = if value < 8.0 {
        "low"
    } else if value <= 16.0 {
        "normal/reference-range"
    } else {
        "raised"
    };
    let interpretation = if corrected.is_some() {
        format!(
            "Anion gap {anion_gap:.1} mmol/L; albumin-corrected anion gap {value:.1} mmol/L ({band}). The albumin correction adds about 0.25 mmol/L for each 1 g/L albumin below 40 g/L."
        )
    } else {
        format!(
            "Anion gap {anion_gap:.1} mmol/L ({band}, using the supplied electrolytes). Local laboratory reference intervals vary and potassium inclusion changes the numeric value."
        )
    };
    Ok(AnionGapOutcome {
        anion_gap,
        albumin_corrected_anion_gap: corrected,
        interpretation,
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

pub fn build_response(input: &AnionGapInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("anion_gap_mmol_l".into(), json!(round1(o.anion_gap)));
    working.insert(
        "includes_potassium".into(),
        json!(input.potassium_mmol_l.is_some()),
    );
    if let Some(corrected) = o.albumin_corrected_anion_gap {
        working.insert(
            "albumin_corrected_anion_gap_mmol_l".into(),
            json!(round1(corrected)),
        );
    }
    working.insert("unit".into(), json!("mmol/L"));
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.albumin_corrected_anion_gap.unwrap_or(o.anion_gap))),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}
fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct AnionGap;
impl Calculator for AnionGap {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Anion Gap"
    }
    fn description(&self) -> &'static str {
        "Serum anion gap from sodium, chloride, bicarbonate, optional potassium, and optional albumin correction."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "AnionGapInput", "type": "object", "additionalProperties": false,
            "required": ["sodium_mmol_l", "chloride_mmol_l", "bicarbonate_mmol_l"],
            "properties": {
                "sodium_mmol_l": { "type": "number", "minimum": 80, "maximum": 220 },
                "chloride_mmol_l": { "type": "number", "minimum": 50, "maximum": 160 },
                "bicarbonate_mmol_l": { "type": "number", "minimum": 1, "maximum": 60 },
                "potassium_mmol_l": { "type": "number", "minimum": 1, "maximum": 10, "description": "Optional; many reference ranges omit potassium" },
                "albumin_g_l": { "type": "number", "minimum": 5, "maximum": 70, "description": "Optional albumin for Figge-style correction to albumin 40 g/L" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AnionGapInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standard_gap_without_potassium() {
        let out = compute(&AnionGapInput {
            sodium_mmol_l: 140.0,
            chloride_mmol_l: 104.0,
            bicarbonate_mmol_l: 24.0,
            potassium_mmol_l: None,
            albumin_g_l: None,
        })
        .unwrap();
        assert_eq!(out.anion_gap, 12.0);
    }
    #[test]
    fn albumin_correction_adds_for_low_albumin() {
        let out = compute(&AnionGapInput {
            sodium_mmol_l: 140.0,
            chloride_mmol_l: 104.0,
            bicarbonate_mmol_l: 24.0,
            potassium_mmol_l: None,
            albumin_g_l: Some(20.0),
        })
        .unwrap();
        assert_eq!(out.albumin_corrected_anion_gap.unwrap(), 17.0);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value =
            json!({"sodium_mmol_l": 140.0, "chloride_mmol_l": 104.0, "bicarbonate_mmol_l": 24.0});
        let typed = AnionGapInput {
            sodium_mmol_l: 140.0,
            chloride_mmol_l: 104.0,
            bicarbonate_mmol_l: 24.0,
            potassium_mmol_l: None,
            albumin_g_l: None,
        };
        let dynamic = AnionGap.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            AnionGap
                .calculate(&json!({"sodium_mmol_l": 140.0, "chloride_mmol_l": 104.0}))
                .is_err()
        );
    }
}
