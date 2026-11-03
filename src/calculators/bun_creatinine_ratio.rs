// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! BUN (urea nitrogen) to creatinine ratio, for prerenal-versus-intrinsic
//! azotaemia screening.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "bun_creatinine_ratio";
pub const REFERENCE: &str = "Standard nephrology teaching ratio: BUN:creatinine >20:1 (both in mg/dL) suggests a prerenal cause of azotaemia, vs a ratio closer to 10-15:1 in intrinsic renal disease; see e.g. Brenner BM, Rector FC (eds). Brenner and Rector's The Kidney. 10th ed. Elsevier; 2016.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from standard nephrology teaching",
    "https://www.ncbi.nlm.nih.gov/books/NBK507821/",
);

/// Serum urea/BUN unit. `mmol/L` is reported as urea (the UK/SI convention);
/// `mg/dL` is reported directly as blood urea nitrogen (the US convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UreaUnit {
    #[serde(rename = "mmol/L")]
    MmolL,
    #[serde(rename = "mg/dL")]
    MgDl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreatinineUnit {
    #[serde(rename = "umol/L")]
    UmolL,
    #[serde(rename = "mg/dL")]
    MgDl,
}

/// mg/dL BUN per mmol/L urea: urea (mmol/L) x 6.006 g urea/mol-fraction to
/// mg/dL, then x 28/60.06 nitrogen mass fraction of urea, combining to 2.80.
const BUN_MGDL_PER_UREA_MMOL_L: f64 = 2.80;
const UMOL_PER_CREATININE_MGDL: f64 = 88.4;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BunCreatinineRatioInput {
    pub urea: f64,
    pub urea_unit: UreaUnit,
    pub creatinine: f64,
    pub creatinine_unit: CreatinineUnit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BunCreatinineRatioOutcome {
    pub bun_mg_dl: f64,
    pub creatinine_mg_dl: f64,
    pub ratio: f64,
    pub pattern: &'static str,
}

pub fn compute(input: &BunCreatinineRatioInput) -> Result<BunCreatinineRatioOutcome, CalcError> {
    let urea_range = match input.urea_unit {
        UreaUnit::MmolL => (0.5, 60.0),
        UreaUnit::MgDl => (1.4, 168.0),
    };
    validate(input.urea, "urea", urea_range.0, urea_range.1)?;
    let creatinine_range = match input.creatinine_unit {
        CreatinineUnit::UmolL => (10.0, 2000.0),
        CreatinineUnit::MgDl => (0.11, 22.6),
    };
    validate(
        input.creatinine,
        "creatinine",
        creatinine_range.0,
        creatinine_range.1,
    )?;

    let bun_mg_dl = match input.urea_unit {
        UreaUnit::MmolL => input.urea * BUN_MGDL_PER_UREA_MMOL_L,
        UreaUnit::MgDl => input.urea,
    };
    let creatinine_mg_dl = match input.creatinine_unit {
        CreatinineUnit::UmolL => input.creatinine / UMOL_PER_CREATININE_MGDL,
        CreatinineUnit::MgDl => input.creatinine,
    };
    let ratio = bun_mg_dl / creatinine_mg_dl;
    let pattern = if ratio > 20.0 {
        "prerenal-pattern (>20:1)"
    } else if ratio >= 10.0 {
        "typical/intrinsic-renal-pattern (10-20:1)"
    } else {
        "low (<10:1)"
    };

    Ok(BunCreatinineRatioOutcome {
        bun_mg_dl,
        creatinine_mg_dl,
        ratio,
        pattern,
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

pub fn build_response(input: &BunCreatinineRatioInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("bun_mg_dl".into(), json!(round1(o.bun_mg_dl)));
    working.insert("creatinine_mg_dl".into(), json!(round2(o.creatinine_mg_dl)));
    working.insert("pattern".into(), json!(o.pattern));

    let interpretation = format!(
        "BUN:creatinine ratio {:.1}:1 ({}). A ratio above 20:1 classically suggests a prerenal cause (e.g. volume depletion, heart failure, GI bleeding increasing urea production); a ratio in the 10-20:1 range is typical of intrinsic renal disease. This is a teaching heuristic, not a validated diagnostic rule - interpret alongside volume status, urine studies, and the wider clinical picture.",
        o.ratio, o.pattern
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.ratio)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub struct BunCreatinineRatio;
impl Calculator for BunCreatinineRatio {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "BUN:Creatinine Ratio"
    }
    fn description(&self) -> &'static str {
        "Blood urea nitrogen to creatinine ratio, for prerenal-versus-intrinsic azotaemia screening."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "BunCreatinineRatioInput", "type": "object", "additionalProperties": false,
            "required": ["urea", "urea_unit", "creatinine", "creatinine_unit"],
            "properties": {
                "urea": { "type": "number", "description": "Serum urea or BUN, in the unit given by urea_unit" },
                "urea_unit": { "type": "string", "enum": ["mmol/L", "mg/dL"], "description": "mmol/L reports urea (UK/SI); mg/dL reports blood urea nitrogen (US)" },
                "creatinine": { "type": "number", "description": "Serum creatinine, in the unit given by creatinine_unit" },
                "creatinine_unit": { "type": "string", "enum": ["umol/L", "mg/dL"] }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: BunCreatinineRatioInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_prerenal_pattern_us_units() {
        let out = compute(&BunCreatinineRatioInput {
            urea: 40.0,
            urea_unit: UreaUnit::MgDl,
            creatinine: 1.0,
            creatinine_unit: CreatinineUnit::MgDl,
        })
        .unwrap();
        assert_eq!(out.ratio, 40.0);
        assert_eq!(out.pattern, "prerenal-pattern (>20:1)");
    }

    #[test]
    fn typical_pattern_si_units() {
        // Urea 7 mmol/L -> BUN 19.6 mg/dL; creatinine 88.4 umol/L -> 1.0 mg/dL.
        let out = compute(&BunCreatinineRatioInput {
            urea: 7.0,
            urea_unit: UreaUnit::MmolL,
            creatinine: 88.4,
            creatinine_unit: CreatinineUnit::UmolL,
        })
        .unwrap();
        assert!((out.ratio - 19.6).abs() < 1e-6);
        assert_eq!(out.pattern, "typical/intrinsic-renal-pattern (10-20:1)");
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"urea": 40.0, "urea_unit": "mg/dL", "creatinine": 1.0, "creatinine_unit": "mg/dL"});
        let typed = BunCreatinineRatioInput {
            urea: 40.0,
            urea_unit: UreaUnit::MgDl,
            creatinine: 1.0,
            creatinine_unit: CreatinineUnit::MgDl,
        };
        let dynamic = BunCreatinineRatio.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            BunCreatinineRatio
                .calculate(&json!({"urea": 40.0, "urea_unit": "mg/dL"}))
                .is_err()
        );
    }
}
