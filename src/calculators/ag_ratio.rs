// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Albumin/globulin (A/G) ratio from total protein and albumin, for
//! liver, kidney, and paraprotein screening.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "ag_ratio";
pub const REFERENCE: &str = "Standard clinical chemistry ratio: globulin = total protein - albumin, then A/G ratio = albumin / globulin. Reference range and interpretation per standard clinical chemistry teaching; see e.g. Busher JT. Serum Albumin and Globulin. In: Walker HK, Hall WD, Hurst JW, eds. Clinical Methods: The History, Physical, and Laboratory Examinations. 3rd ed. Butterworths; 1990. Chapter 101.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from standard clinical chemistry",
    "https://www.ncbi.nlm.nih.gov/books/NBK204/",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProteinUnit {
    #[serde(rename = "g/L")]
    GPerL,
    #[serde(rename = "g/dL")]
    GPerDl,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgRatioInput {
    pub total_protein: f64,
    /// Same unit as `total_protein`.
    pub albumin: f64,
    pub unit: ProteinUnit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgRatioOutcome {
    pub globulin: f64,
    pub ratio: f64,
    pub band: &'static str,
}

pub fn compute(input: &AgRatioInput) -> Result<AgRatioOutcome, CalcError> {
    let (protein_min, protein_max) = match input.unit {
        ProteinUnit::GPerL => (20.0, 140.0),
        ProteinUnit::GPerDl => (2.0, 14.0),
    };
    validate(
        input.total_protein,
        "total_protein",
        protein_min,
        protein_max,
    )?;
    validate(input.albumin, "albumin", protein_min * 0.1, protein_max)?;
    if input.albumin >= input.total_protein {
        return Err(CalcError::InvalidInput(
            "albumin must be lower than total_protein".to_string(),
        ));
    }

    let globulin = input.total_protein - input.albumin;
    let ratio = input.albumin / globulin;
    let band = if ratio < 1.0 {
        "low (<1.0) - raised globulin and/or low albumin"
    } else if ratio > 2.5 {
        "high (>2.5) - low globulin relative to albumin"
    } else {
        "within the usual 1.0-2.5 reference range"
    };

    Ok(AgRatioOutcome {
        globulin,
        ratio,
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

pub fn build_response(input: &AgRatioInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("globulin".into(), json!(round2(o.globulin)));
    working.insert("band".into(), json!(o.band));
    working.insert(
        "unit".into(),
        json!(match input.unit {
            ProteinUnit::GPerL => "g/L",
            ProteinUnit::GPerDl => "g/dL",
        }),
    );

    let interpretation = format!(
        "A/G ratio {:.2} ({}). A low ratio can reflect raised globulins (chronic inflammation, infection, paraproteinaemia) or low albumin (liver disease, nephrotic syndrome, malnutrition); a high ratio is less commonly significant but can reflect reduced globulin production. This ratio screens for an abnormality - it does not identify the cause, and a raised globulin fraction with a low A/G ratio should prompt serum protein electrophoresis if a paraprotein is suspected.",
        o.ratio, o.band
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round2(o.ratio)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub struct AgRatio;
impl Calculator for AgRatio {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Albumin/Globulin (A/G) Ratio"
    }
    fn description(&self) -> &'static str {
        "Albumin/globulin ratio from total protein and albumin, for liver, kidney, and paraprotein screening."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "AgRatioInput", "type": "object", "additionalProperties": false,
            "required": ["total_protein", "albumin", "unit"],
            "properties": {
                "total_protein": { "type": "number", "description": "Total serum protein, in the unit given by unit" },
                "albumin": { "type": "number", "description": "Serum albumin, same unit as total_protein; must be lower than total_protein" },
                "unit": { "type": "string", "enum": ["g/L", "g/dL"], "description": "Unit shared by total_protein and albumin" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AgRatioInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_range_si_units() {
        let out = compute(&AgRatioInput {
            total_protein: 72.0,
            albumin: 42.0,
            unit: ProteinUnit::GPerL,
        })
        .unwrap();
        assert_eq!(out.globulin, 30.0);
        assert_eq!(out.ratio, 1.4);
        assert_eq!(out.band, "within the usual 1.0-2.5 reference range");
    }

    #[test]
    fn low_ratio_from_raised_globulin() {
        let out = compute(&AgRatioInput {
            total_protein: 80.0,
            albumin: 30.0,
            unit: ProteinUnit::GPerL,
        })
        .unwrap();
        assert_eq!(out.globulin, 50.0);
        assert_eq!(out.ratio, 0.6);
        assert_eq!(out.band, "low (<1.0) - raised globulin and/or low albumin");
    }

    #[test]
    fn rejects_albumin_not_below_total_protein() {
        assert!(
            compute(&AgRatioInput {
                total_protein: 40.0,
                albumin: 42.0,
                unit: ProteinUnit::GPerL,
            })
            .is_err()
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"total_protein": 72.0, "albumin": 42.0, "unit": "g/L"});
        let typed = AgRatioInput {
            total_protein: 72.0,
            albumin: 42.0,
            unit: ProteinUnit::GPerL,
        };
        let dynamic = AgRatio.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            AgRatio
                .calculate(&json!({"total_protein": 72.0, "albumin": 42.0}))
                .is_err()
        );
    }
}
