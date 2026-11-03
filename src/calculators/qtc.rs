// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Heart-rate-corrected QT interval (QTc) by the four standard named
//! formulas: Bazett, Fridericia, Framingham, and Hodges.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "qtc";
pub const REFERENCE: &str = "Bazett HC. An analysis of the time-relations of electrocardiograms. Heart. 1920;7:353-370. Fridericia LS. Die Systolendauer im Elektrokardiogramm bei normalen Menschen und bei Herzkranken. Acta Med Scand. 1920;53:469-486. Framingham formula per Sagie A, Larson MG, Goldberg RJ, Bengtson JR, Levy D. An improved method for adjusting the QT interval for heart rate (the Framingham Heart Study). Am J Cardiol. 1992;70(7):797-801. Hodges formula per Hodges M, Salerno D, Erlien D. Bazett's QT correction reviewed: evidence that a linear QT correction for heart rate is better. J Am Coll Cardiol. 1983;1:694.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain formulae - implemented from the primary literature",
    "https://doi.org/10.1016/0002-9149(92)90562-D",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QtcInput {
    pub qt_ms: f64,
    pub heart_rate_bpm: f64,
    /// Optional; enables sex-specific prolonged-QTc banding.
    #[serde(default)]
    pub sex: Option<Sex>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QtcOutcome {
    pub bazett_ms: f64,
    pub fridericia_ms: f64,
    pub framingham_ms: f64,
    pub hodges_ms: f64,
    pub high_risk: bool,
    pub prolonged: Option<bool>,
}

pub fn compute(input: &QtcInput) -> Result<QtcOutcome, CalcError> {
    validate(input.qt_ms, "qt_ms", 150.0, 800.0)?;
    validate(input.heart_rate_bpm, "heart_rate_bpm", 20.0, 250.0)?;

    let rr_seconds = 60.0 / input.heart_rate_bpm;
    let bazett_ms = input.qt_ms / rr_seconds.sqrt();
    let fridericia_ms = input.qt_ms / rr_seconds.cbrt();
    let framingham_ms = input.qt_ms + 154.0 * (1.0 - rr_seconds);
    let hodges_ms = input.qt_ms + 1.75 * (input.heart_rate_bpm - 60.0);

    let high_risk = bazett_ms > 500.0;
    let prolonged = input.sex.map(|sex| {
        let threshold = match sex {
            Sex::Male => 450.0,
            Sex::Female => 460.0,
        };
        bazett_ms > threshold
    });

    Ok(QtcOutcome {
        bazett_ms,
        fridericia_ms,
        framingham_ms,
        hodges_ms,
        high_risk,
        prolonged,
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

pub fn build_response(input: &QtcInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("fridericia_ms".into(), json!(round0(o.fridericia_ms)));
    working.insert("framingham_ms".into(), json!(round0(o.framingham_ms)));
    working.insert("hodges_ms".into(), json!(round0(o.hodges_ms)));
    working.insert("high_risk_above_500ms".into(), json!(o.high_risk));
    working.insert("unit".into(), json!("ms"));
    if let Some(prolonged) = o.prolonged {
        working.insert("prolonged_for_sex".into(), json!(prolonged));
    }

    let risk_note = if o.high_risk {
        " QTc Bazett exceeds 500 ms, a widely cited threshold for markedly increased torsades de pointes risk."
    } else {
        ""
    };

    let interpretation = format!(
        "QTc (Bazett) {:.0} ms; Fridericia {:.0} ms, Framingham {:.0} ms, Hodges {:.0} ms.{} Bazett over-corrects at heart rates away from 60 bpm; Fridericia or Framingham are generally preferred at tachycardia or bradycardia. Sex-specific thresholds (prolonged >450 ms men, >460 ms women) apply to Bazett-corrected values.",
        o.bazett_ms, o.fridericia_ms, o.framingham_ms, o.hodges_ms, risk_note
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round0(o.bazett_ms)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round0(v: f64) -> f64 {
    v.round()
}

pub struct Qtc;
impl Calculator for Qtc {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "QTc Correction"
    }
    fn description(&self) -> &'static str {
        "Heart-rate-corrected QT interval by Bazett, Fridericia, Framingham, and Hodges."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "QtcInput", "type": "object", "additionalProperties": false,
            "required": ["qt_ms", "heart_rate_bpm"],
            "properties": {
                "qt_ms": { "type": "number", "minimum": 150, "maximum": 800 },
                "heart_rate_bpm": { "type": "number", "minimum": 20, "maximum": 250 },
                "sex": { "type": "string", "enum": ["Male", "Female"], "description": "Optional; enables sex-specific prolonged-QTc banding on the Bazett value" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: QtcInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_hr_60_all_formulas_equal_raw_qt() {
        // RR = 1.0s exactly at HR 60, so every correction collapses to QT itself.
        let out = compute(&QtcInput {
            qt_ms: 400.0,
            heart_rate_bpm: 60.0,
            sex: None,
        })
        .unwrap();
        assert!((out.bazett_ms - 400.0).abs() < 1e-9);
        assert!((out.fridericia_ms - 400.0).abs() < 1e-9);
        assert!((out.framingham_ms - 400.0).abs() < 1e-9);
        assert!((out.hodges_ms - 400.0).abs() < 1e-9);
    }

    #[test]
    fn tachycardia_bazett_overcorrects_relative_to_fridericia() {
        let out = compute(&QtcInput {
            qt_ms: 350.0,
            heart_rate_bpm: 120.0,
            sex: None,
        })
        .unwrap();
        assert!(out.bazett_ms > out.fridericia_ms);
    }

    #[test]
    fn high_risk_flag_above_500ms() {
        let out = compute(&QtcInput {
            qt_ms: 520.0,
            heart_rate_bpm: 60.0,
            sex: None,
        })
        .unwrap();
        assert!(out.high_risk);
    }

    #[test]
    fn sex_specific_prolonged_banding() {
        let out = compute(&QtcInput {
            qt_ms: 460.0,
            heart_rate_bpm: 60.0,
            sex: Some(Sex::Male),
        })
        .unwrap();
        assert_eq!(out.prolonged, Some(true));

        let out_no_sex = compute(&QtcInput {
            qt_ms: 460.0,
            heart_rate_bpm: 60.0,
            sex: None,
        })
        .unwrap();
        assert_eq!(out_no_sex.prolonged, None);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"qt_ms": 400.0, "heart_rate_bpm": 75.0});
        let typed = QtcInput {
            qt_ms: 400.0,
            heart_rate_bpm: 75.0,
            sex: None,
        };
        let dynamic = Qtc.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(Qtc.calculate(&json!({"qt_ms": 400.0})).is_err());
    }
}
