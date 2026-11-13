// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Winters' Formula - expected respiratory (PaCO2) compensation for a
//! primary metabolic acidosis (Albert, Dell, Winters. Ann Intern Med 1967).
//!
//! Expected PaCO2 = 1.5 x [HCO3-] + 8, +/- 2 mmHg. Comparing the measured
//! PaCO2 against this window identifies a concurrent respiratory disorder:
//! within the window is appropriate compensation, above it a concurrent
//! respiratory acidosis, below it a concurrent respiratory alkalosis.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "winters_formula";

pub const REFERENCE: &str = "Albert MS, Dell RB, Winters RW. Quantitative displacement of acid-base equilibrium \
in metabolic acidosis. Ann Intern Med. 1967;66(2):312-322.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.7326/0003-4819-66-2-312",
);

const WINDOW: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WintersFormulaInput {
    /// Serum bicarbonate in mEq/L (mmol/L).
    pub bicarbonate_mmol_l: f64,
    /// Measured arterial PaCO2 in mmHg.
    pub measured_paco2_mmhg: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompensationVerdict {
    Appropriate,
    ConcurrentRespiratoryAcidosis,
    ConcurrentRespiratoryAlkalosis,
}

impl CompensationVerdict {
    fn slug(self) -> &'static str {
        match self {
            Self::Appropriate => "appropriate_compensation",
            Self::ConcurrentRespiratoryAcidosis => "concurrent_respiratory_acidosis",
            Self::ConcurrentRespiratoryAlkalosis => "concurrent_respiratory_alkalosis",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WintersFormulaOutcome {
    pub expected_paco2_mmhg: f64,
    pub expected_range_low: f64,
    pub expected_range_high: f64,
    pub verdict: CompensationVerdict,
}

pub fn compute(input: &WintersFormulaInput) -> Result<WintersFormulaOutcome, CalcError> {
    if !input.bicarbonate_mmol_l.is_finite() || input.bicarbonate_mmol_l <= 0.0 {
        return Err(CalcError::InvalidInput(
            "bicarbonate_mmol_l must be a positive finite number".into(),
        ));
    }
    if !input.measured_paco2_mmhg.is_finite() || input.measured_paco2_mmhg <= 0.0 {
        return Err(CalcError::InvalidInput(
            "measured_paco2_mmhg must be a positive finite number".into(),
        ));
    }

    let expected_paco2_mmhg = 1.5 * input.bicarbonate_mmol_l + 8.0;
    let expected_range_low = expected_paco2_mmhg - WINDOW;
    let expected_range_high = expected_paco2_mmhg + WINDOW;

    let verdict = if input.measured_paco2_mmhg > expected_range_high {
        CompensationVerdict::ConcurrentRespiratoryAcidosis
    } else if input.measured_paco2_mmhg < expected_range_low {
        CompensationVerdict::ConcurrentRespiratoryAlkalosis
    } else {
        CompensationVerdict::Appropriate
    };

    Ok(WintersFormulaOutcome {
        expected_paco2_mmhg,
        expected_range_low,
        expected_range_high,
        verdict,
    })
}

pub fn build_response(input: &WintersFormulaInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "expected_paco2_mmhg".into(),
        json!(round1(o.expected_paco2_mmhg)),
    );
    working.insert(
        "expected_range_low_mmhg".into(),
        json!(round1(o.expected_range_low)),
    );
    working.insert(
        "expected_range_high_mmhg".into(),
        json!(round1(o.expected_range_high)),
    );
    working.insert("verdict".into(), json!(o.verdict.slug()));

    let interpretation = match o.verdict {
        CompensationVerdict::Appropriate => format!(
            "Measured PaCO2 {:.1} mmHg is within the expected range {:.1}-{:.1} mmHg: appropriate respiratory compensation for a simple metabolic acidosis. Full compensation can take 12-24 hours to develop.",
            input.measured_paco2_mmhg, o.expected_range_low, o.expected_range_high
        ),
        CompensationVerdict::ConcurrentRespiratoryAcidosis => format!(
            "Measured PaCO2 {:.1} mmHg is above the expected range {:.1}-{:.1} mmHg: a concurrent primary respiratory acidosis is present, not just better-than-expected compensation.",
            input.measured_paco2_mmhg, o.expected_range_low, o.expected_range_high
        ),
        CompensationVerdict::ConcurrentRespiratoryAlkalosis => format!(
            "Measured PaCO2 {:.1} mmHg is below the expected range {:.1}-{:.1} mmHg: a concurrent primary respiratory alkalosis is present.",
            input.measured_paco2_mmhg, o.expected_range_low, o.expected_range_high
        ),
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.expected_paco2_mmhg)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct WintersFormula;
impl Calculator for WintersFormula {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Winters' Formula"
    }
    fn description(&self) -> &'static str {
        "Expected respiratory (PaCO2) compensation for a primary metabolic acidosis, compared against the measured PaCO2."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "WintersFormulaInput", "type": "object",
            "additionalProperties": false, "required": ["bicarbonate_mmol_l", "measured_paco2_mmhg"],
            "properties": {
                "bicarbonate_mmol_l": { "type": "number", "exclusiveMinimum": 0, "maximum": 50, "description": "Serum bicarbonate (HCO3-) in mEq/L (mmol/L)" },
                "measured_paco2_mmhg": { "type": "number", "exclusiveMinimum": 0, "maximum": 150, "description": "Measured arterial PaCO2 in mmHg, from the same arterial blood gas" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: WintersFormulaInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(hco3: f64, paco2: f64) -> WintersFormulaInput {
        WintersFormulaInput {
            bicarbonate_mmol_l: hco3,
            measured_paco2_mmhg: paco2,
        }
    }

    #[test]
    fn expected_value_matches_formula() {
        let o = compute(&input(12.0, 26.0)).unwrap();
        assert_eq!(o.expected_paco2_mmhg, 26.0);
        assert_eq!(o.verdict, CompensationVerdict::Appropriate);
    }

    #[test]
    fn measured_above_range_indicates_concurrent_respiratory_acidosis() {
        let o = compute(&input(14.0, 38.0)).unwrap();
        // expected = 1.5*14+8 = 29, range 27-31; 38 is above.
        assert_eq!(
            o.verdict,
            CompensationVerdict::ConcurrentRespiratoryAcidosis
        );
    }

    #[test]
    fn measured_below_range_indicates_concurrent_respiratory_alkalosis() {
        let o = compute(&input(22.0, 30.0)).unwrap();
        // expected = 1.5*22+8 = 41, range 39-43; 30 is below.
        assert_eq!(
            o.verdict,
            CompensationVerdict::ConcurrentRespiratoryAlkalosis
        );
    }

    #[test]
    fn boundary_of_window_is_still_appropriate() {
        let o = compute(&input(12.0, 24.0)).unwrap(); // expected 26, low bound 24
        assert_eq!(o.verdict, CompensationVerdict::Appropriate);
    }

    #[test]
    fn rejects_non_positive_inputs() {
        assert!(compute(&input(0.0, 30.0)).is_err());
        assert!(compute(&input(12.0, 0.0)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let typed = input(12.0, 26.0);
        let value = json!({ "bicarbonate_mmol_l": 12.0, "measured_paco2_mmhg": 26.0 });
        let dynamic = WintersFormula.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(26.0));
    }
}
