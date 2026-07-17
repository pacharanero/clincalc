// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! ACC/AHA 2013 Pooled Cohort Equations for 10-year ASCVD risk.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "ascvd";
pub const REFERENCE: &str = "Goff DC Jr, Lloyd-Jones DM, Bennett G, et al. 2013 ACC/AHA Guideline on the Assessment of Cardiovascular Risk. Circulation. 2014;129(25 Suppl 2):S49-S73. Pooled Cohort Equations coefficients from the guideline risk-assessment report.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain guideline equation - implemented from ACC/AHA published coefficients",
    source_url: "https://doi.org/10.1161/01.cir.0000437741.48606.98",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Race {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AscvdInput {
    pub age: u8,
    pub sex: Sex,
    pub race: Race,
    pub total_cholesterol_mg_dl: f64,
    pub hdl_cholesterol_mg_dl: f64,
    pub systolic_bp_mm_hg: f64,
    pub treated_bp: bool,
    pub current_smoker: bool,
    pub diabetes: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AscvdOutcome {
    pub risk_percent: f64,
    pub risk_band: RiskBand,
    pub interpretation: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskBand {
    Low,
    Borderline,
    Intermediate,
    High,
}
impl RiskBand {
    fn slug(self) -> &'static str {
        match self {
            RiskBand::Low => "low",
            RiskBand::Borderline => "borderline",
            RiskBand::Intermediate => "intermediate",
            RiskBand::High => "high",
        }
    }
}

pub fn compute(i: &AscvdInput) -> Result<AscvdOutcome, CalcError> {
    validate(i)?;
    let age = f64::from(i.age).ln();
    let tc = i.total_cholesterol_mg_dl.ln();
    let hdl = i.hdl_cholesterol_mg_dl.ln();
    let sbp = i.systolic_bp_mm_hg.ln();
    let sum = match (i.sex, i.race) {
        (Sex::Male, Race::White) => {
            12.344 * age + 11.853 * tc - 2.664 * age * tc - 7.990 * hdl
                + 1.769 * age * hdl
                + if i.treated_bp {
                    1.797 * sbp
                } else {
                    1.764 * sbp
                }
                + if i.current_smoker {
                    7.837 - 1.795 * age
                } else {
                    0.0
                }
                + if i.diabetes { 0.658 } else { 0.0 }
        }
        (Sex::Female, Race::White) => {
            -29.799 * age + 4.884 * age * age + 13.540 * tc - 3.114 * age * tc - 13.578 * hdl
                + 3.149 * age * hdl
                + if i.treated_bp {
                    2.019 * sbp
                } else {
                    1.957 * sbp
                }
                + if i.current_smoker {
                    7.574 - 1.665 * age
                } else {
                    0.0
                }
                + if i.diabetes { 0.661 } else { 0.0 }
        }
        (Sex::Male, Race::Black) => {
            2.469 * age + 0.302 * tc - 0.307 * hdl
                + if i.treated_bp {
                    1.916 * sbp
                } else {
                    1.809 * sbp
                }
                + if i.current_smoker { 0.549 } else { 0.0 }
                + if i.diabetes { 0.645 } else { 0.0 }
        }
        (Sex::Female, Race::Black) => {
            17.114 * age + 0.940 * tc - 18.920 * hdl
                + 4.475 * age * hdl
                + if i.treated_bp {
                    29.291 * sbp - 6.432 * age * sbp
                } else {
                    27.820 * sbp - 6.087 * age * sbp
                }
                + if i.current_smoker { 0.691 } else { 0.0 }
                + if i.diabetes { 0.874 } else { 0.0 }
        }
    };
    let (baseline, mean): (f64, f64) = match (i.sex, i.race) {
        (Sex::Male, Race::White) => (0.9144, 61.18),
        (Sex::Female, Race::White) => (0.9665, -29.18),
        (Sex::Male, Race::Black) => (0.8954, 19.54),
        (Sex::Female, Race::Black) => (0.9533, 86.61),
    };
    let risk = 1.0 - baseline.powf((sum - mean).exp());
    let risk_percent = risk * 100.0;
    let risk_band = if risk_percent < 5.0 {
        RiskBand::Low
    } else if risk_percent < 7.5 {
        RiskBand::Borderline
    } else if risk_percent < 20.0 {
        RiskBand::Intermediate
    } else {
        RiskBand::High
    };
    let interpretation = format!(
        "10-year ASCVD risk {:.1}% ({:?}). The Pooled Cohort Equations are intended for US adults aged 40-79 without established ASCVD; calibration varies by population and they are not the UK QRISK standard.",
        risk_percent, risk_band
    );
    Ok(AscvdOutcome {
        risk_percent,
        risk_band,
        interpretation,
    })
}

fn validate(i: &AscvdInput) -> Result<(), CalcError> {
    if !(40..=79).contains(&i.age) {
        return Err(CalcError::InvalidInput(
            "age must be 40-79 for the Pooled Cohort Equations".into(),
        ));
    }
    for (name, value, min, max) in [
        (
            "total_cholesterol_mg_dl",
            i.total_cholesterol_mg_dl,
            130.0,
            320.0,
        ),
        (
            "hdl_cholesterol_mg_dl",
            i.hdl_cholesterol_mg_dl,
            20.0,
            100.0,
        ),
        ("systolic_bp_mm_hg", i.systolic_bp_mm_hg, 90.0, 200.0),
    ] {
        if !(min..=max).contains(&value) || !value.is_finite() {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be finite and between {min} and {max}"
            )));
        }
    }
    Ok(())
}

pub fn build_response(i: &AscvdInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(i)?;
    let mut working = Map::new();
    working.insert("risk_percent".into(), json!(round1(o.risk_percent)));
    working.insert("risk_band".into(), json!(o.risk_band.slug()));
    working.insert("unit".into(), json!("%"));
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.risk_percent)),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}
fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct Ascvd;
impl Calculator for Ascvd {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "ASCVD Pooled Cohort Equations"
    }
    fn description(&self) -> &'static str {
        "ACC/AHA 2013 10-year ASCVD risk estimate for US adults aged 40-79."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({ "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "AscvdInput", "type": "object", "additionalProperties": false, "required": ["age", "sex", "race", "total_cholesterol_mg_dl", "hdl_cholesterol_mg_dl", "systolic_bp_mm_hg", "treated_bp", "current_smoker", "diabetes"], "properties": { "age": { "type": "integer", "minimum": 40, "maximum": 79 }, "sex": { "type": "string", "enum": ["male", "female"] }, "race": { "type": "string", "enum": ["white", "black"], "description": "PCE coefficient set. The original equations provide Black and White coefficient sets only." }, "total_cholesterol_mg_dl": { "type": "number", "minimum": 130, "maximum": 320 }, "hdl_cholesterol_mg_dl": { "type": "number", "minimum": 20, "maximum": 100 }, "systolic_bp_mm_hg": { "type": "number", "minimum": 90, "maximum": 200 }, "treated_bp": { "type": "boolean" }, "current_smoker": { "type": "boolean" }, "diabetes": { "type": "boolean" } } })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AscvdInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn computes_plausible_white_male_example() {
        let out = compute(&AscvdInput {
            age: 55,
            sex: Sex::Male,
            race: Race::White,
            total_cholesterol_mg_dl: 213.0,
            hdl_cholesterol_mg_dl: 50.0,
            systolic_bp_mm_hg: 120.0,
            treated_bp: false,
            current_smoker: false,
            diabetes: false,
        })
        .unwrap();
        assert_eq!(round1(out.risk_percent), 5.4);
    }
    #[test]
    fn diabetes_and_smoking_raise_risk() {
        let base = AscvdInput {
            age: 60,
            sex: Sex::Female,
            race: Race::Black,
            total_cholesterol_mg_dl: 200.0,
            hdl_cholesterol_mg_dl: 50.0,
            systolic_bp_mm_hg: 130.0,
            treated_bp: true,
            current_smoker: false,
            diabetes: false,
        };
        let low = compute(&base).unwrap().risk_percent;
        let high = compute(&AscvdInput {
            current_smoker: true,
            diabetes: true,
            ..base
        })
        .unwrap()
        .risk_percent;
        assert!(high > low);
    }
}
