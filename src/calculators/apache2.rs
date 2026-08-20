// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! APACHE II acute physiology and chronic health score.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "apache2";
pub const REFERENCE: &str = "Knaus WA, Draper EA, Wagner DP, Zimmerman JE. APACHE II: a severity of disease classification system. Crit Care Med. 1985;13(10):818-829.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published clinical score - acute physiology point table implemented from the primary literature",
    source_url: "https://pubmed.ncbi.nlm.nih.gov/3928249/",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChronicHealthStatus {
    None,
    ElectivePostoperative,
    NonoperativeOrEmergencyPostoperative,
}
impl ChronicHealthStatus {
    fn points(self) -> u8 {
        match self {
            ChronicHealthStatus::None => 0,
            ChronicHealthStatus::ElectivePostoperative => 2,
            ChronicHealthStatus::NonoperativeOrEmergencyPostoperative => 5,
        }
    }
    fn slug(self) -> &'static str {
        match self {
            ChronicHealthStatus::None => "none",
            ChronicHealthStatus::ElectivePostoperative => "elective_postoperative",
            ChronicHealthStatus::NonoperativeOrEmergencyPostoperative => {
                "nonoperative_or_emergency_postoperative"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Apache2Input {
    pub temperature_c: f64,
    pub mean_arterial_pressure_mm_hg: f64,
    pub heart_rate: f64,
    pub respiratory_rate: f64,
    pub fio2: f64,
    #[serde(default)]
    pub pao2_mm_hg: Option<f64>,
    #[serde(default)]
    pub aa_gradient_mm_hg: Option<f64>,
    pub arterial_ph: f64,
    pub sodium_mmol_l: f64,
    pub potassium_mmol_l: f64,
    pub creatinine_mg_dl: f64,
    #[serde(default)]
    pub acute_renal_failure: bool,
    pub hematocrit_percent: f64,
    pub wbc_10_9_l: f64,
    pub glasgow_coma_scale: u8,
    pub age: u8,
    pub chronic_health_status: ChronicHealthStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Apache2Outcome {
    pub acute_physiology_score: u8,
    pub age_points: u8,
    pub chronic_health_points: u8,
    pub total_score: u8,
    pub interpretation: String,
}

pub fn compute(i: &Apache2Input) -> Result<Apache2Outcome, CalcError> {
    validate_ranges(i)?;
    let oxygenation = if i.fio2 >= 0.5 {
        oxygen_gradient_points(i.aa_gradient_mm_hg.ok_or_else(|| {
            CalcError::InvalidInput("aa_gradient_mm_hg is required when fio2 >= 0.5".into())
        })?)
    } else {
        oxygen_pao2_points(i.pao2_mm_hg.ok_or_else(|| {
            CalcError::InvalidInput("pao2_mm_hg is required when fio2 < 0.5".into())
        })?)
    };
    let creatinine_base = creatinine_points(i.creatinine_mg_dl);
    let creatinine = if i.acute_renal_failure {
        creatinine_base * 2
    } else {
        creatinine_base
    };
    let gcs_points = 15 - i.glasgow_coma_scale;
    let acute_physiology_score = temp_points(i.temperature_c)
        + map_points(i.mean_arterial_pressure_mm_hg)
        + heart_rate_points(i.heart_rate)
        + respiratory_rate_points(i.respiratory_rate)
        + oxygenation
        + ph_points(i.arterial_ph)
        + sodium_points(i.sodium_mmol_l)
        + potassium_points(i.potassium_mmol_l)
        + creatinine
        + hematocrit_points(i.hematocrit_percent)
        + wbc_points(i.wbc_10_9_l)
        + gcs_points;
    let age_points = age_points(i.age);
    let chronic_health_points = i.chronic_health_status.points();
    let total_score = acute_physiology_score + age_points + chronic_health_points;
    let interpretation = format!(
        "APACHE II score {total_score}: acute physiology {acute_physiology_score}, age {age_points}, chronic health {chronic_health_points}. APACHE II is an ICU severity score derived from worst values in the first 24 hours; mortality prediction also depends on diagnostic category and should not be inferred from the score alone."
    );
    Ok(Apache2Outcome {
        acute_physiology_score,
        age_points,
        chronic_health_points,
        total_score,
        interpretation,
    })
}

fn validate_ranges(i: &Apache2Input) -> Result<(), CalcError> {
    for (name, value) in [
        ("temperature_c", i.temperature_c),
        (
            "mean_arterial_pressure_mm_hg",
            i.mean_arterial_pressure_mm_hg,
        ),
        ("heart_rate", i.heart_rate),
        ("respiratory_rate", i.respiratory_rate),
        ("fio2", i.fio2),
        ("arterial_ph", i.arterial_ph),
        ("sodium_mmol_l", i.sodium_mmol_l),
        ("potassium_mmol_l", i.potassium_mmol_l),
        ("creatinine_mg_dl", i.creatinine_mg_dl),
        ("hematocrit_percent", i.hematocrit_percent),
        ("wbc_10_9_l", i.wbc_10_9_l),
    ] {
        if !value.is_finite() {
            return Err(CalcError::InvalidInput(format!("{name} must be finite")));
        }
    }
    if !(0.21..=1.0).contains(&i.fio2) {
        return Err(CalcError::InvalidInput(
            "fio2 must be a fraction from 0.21 to 1.0".into(),
        ));
    }
    if !(3..=15).contains(&i.glasgow_coma_scale) {
        return Err(CalcError::InvalidInput(
            "glasgow_coma_scale must be 3-15".into(),
        ));
    }
    if i.age > 120 {
        return Err(CalcError::InvalidInput("age must be <= 120".into()));
    }
    if let Some(pao2) = i.pao2_mm_hg
        && (pao2 <= 0.0 || !pao2.is_finite())
    {
        return Err(CalcError::InvalidInput(
            "pao2_mm_hg must be positive".into(),
        ));
    }
    if let Some(aa) = i.aa_gradient_mm_hg
        && (aa < 0.0 || !aa.is_finite())
    {
        return Err(CalcError::InvalidInput(
            "aa_gradient_mm_hg must be non-negative".into(),
        ));
    }
    Ok(())
}

fn temp_points(v: f64) -> u8 {
    if v >= 41.0 {
        4
    } else if v >= 39.0 {
        3
    } else if v >= 38.5 {
        1
    } else if v >= 36.0 {
        0
    } else if v >= 34.0 {
        1
    } else if v >= 32.0 {
        2
    } else if v >= 30.0 {
        3
    } else {
        4
    }
}
fn map_points(v: f64) -> u8 {
    if v >= 160.0 {
        4
    } else if v >= 130.0 {
        3
    } else if v >= 110.0 {
        2
    } else if v >= 70.0 {
        0
    } else if v >= 50.0 {
        2
    } else {
        4
    }
}
fn heart_rate_points(v: f64) -> u8 {
    if v >= 180.0 {
        4
    } else if v >= 140.0 {
        3
    } else if v >= 110.0 {
        2
    } else if v >= 70.0 {
        0
    } else if v >= 55.0 {
        2
    } else if v >= 40.0 {
        3
    } else {
        4
    }
}
fn respiratory_rate_points(v: f64) -> u8 {
    if v >= 50.0 {
        4
    } else if v >= 35.0 {
        3
    } else if v >= 25.0 {
        1
    } else if v >= 12.0 {
        0
    } else if v >= 10.0 {
        1
    } else if v >= 6.0 {
        2
    } else {
        4
    }
}
fn oxygen_gradient_points(v: f64) -> u8 {
    if v >= 500.0 {
        4
    } else if v >= 350.0 {
        3
    } else if v >= 200.0 {
        2
    } else {
        0
    }
}
fn oxygen_pao2_points(v: f64) -> u8 {
    if v > 70.0 {
        0
    } else if v >= 61.0 {
        1
    } else if v >= 55.0 {
        3
    } else {
        4
    }
}
fn ph_points(v: f64) -> u8 {
    if v >= 7.70 {
        4
    } else if v >= 7.60 {
        3
    } else if v >= 7.50 {
        1
    } else if v >= 7.33 {
        0
    } else if v >= 7.25 {
        2
    } else if v >= 7.15 {
        3
    } else {
        4
    }
}
fn sodium_points(v: f64) -> u8 {
    if v >= 180.0 {
        4
    } else if v >= 160.0 {
        3
    } else if v >= 155.0 {
        2
    } else if v >= 150.0 {
        1
    } else if v >= 130.0 {
        0
    } else if v >= 120.0 {
        2
    } else if v >= 111.0 {
        3
    } else {
        4
    }
}
fn potassium_points(v: f64) -> u8 {
    if v >= 7.0 {
        4
    } else if v >= 6.0 {
        3
    } else if v >= 5.5 {
        1
    } else if v >= 3.5 {
        0
    } else if v >= 3.0 {
        1
    } else if v >= 2.5 {
        2
    } else {
        4
    }
}
fn creatinine_points(v: f64) -> u8 {
    if v >= 3.5 {
        4
    } else if v >= 2.0 {
        3
    } else if v >= 1.5 {
        2
    } else if v >= 0.6 {
        0
    } else {
        2
    }
}
fn hematocrit_points(v: f64) -> u8 {
    if v >= 60.0 {
        4
    } else if v >= 50.0 {
        2
    } else if v >= 46.0 {
        1
    } else if v >= 30.0 {
        0
    } else if v >= 20.0 {
        2
    } else {
        4
    }
}
fn wbc_points(v: f64) -> u8 {
    if v >= 40.0 {
        4
    } else if v >= 20.0 {
        2
    } else if v >= 15.0 {
        1
    } else if v >= 3.0 {
        0
    } else if v >= 1.0 {
        2
    } else {
        4
    }
}
fn age_points(age: u8) -> u8 {
    if age >= 75 {
        6
    } else if age >= 65 {
        5
    } else if age >= 55 {
        3
    } else if age >= 45 {
        2
    } else {
        0
    }
}

pub fn build_response(input: &Apache2Input) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "acute_physiology_score".into(),
        json!(o.acute_physiology_score),
    );
    working.insert("age_points".into(), json!(o.age_points));
    working.insert(
        "chronic_health_points".into(),
        json!(o.chronic_health_points),
    );
    working.insert(
        "chronic_health_status".into(),
        json!(input.chronic_health_status.slug()),
    );
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.total_score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Apache2;
impl Calculator for Apache2 {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "APACHE II"
    }
    fn description(&self) -> &'static str {
        "ICU acute physiology, age, and chronic health severity score (0-71)."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({ "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "Apache2Input", "type": "object", "additionalProperties": false, "required": ["temperature_c", "mean_arterial_pressure_mm_hg", "heart_rate", "respiratory_rate", "fio2", "arterial_ph", "sodium_mmol_l", "potassium_mmol_l", "creatinine_mg_dl", "hematocrit_percent", "wbc_10_9_l", "glasgow_coma_scale", "age", "chronic_health_status"], "properties": { "temperature_c": { "type": "number" }, "mean_arterial_pressure_mm_hg": { "type": "number" }, "heart_rate": { "type": "number" }, "respiratory_rate": { "type": "number" }, "fio2": { "type": "number", "minimum": 0.21, "maximum": 1.0 }, "pao2_mm_hg": { "type": "number", "description": "Required when fio2 < 0.5" }, "aa_gradient_mm_hg": { "type": "number", "description": "Required when fio2 >= 0.5" }, "arterial_ph": { "type": "number" }, "sodium_mmol_l": { "type": "number" }, "potassium_mmol_l": { "type": "number" }, "creatinine_mg_dl": { "type": "number" }, "acute_renal_failure": { "type": "boolean", "description": "Double creatinine points when acute renal failure is present" }, "hematocrit_percent": { "type": "number" }, "wbc_10_9_l": { "type": "number" }, "glasgow_coma_scale": { "type": "integer", "minimum": 3, "maximum": 15 }, "age": { "type": "integer", "minimum": 0, "maximum": 120 }, "chronic_health_status": { "type": "string", "enum": ["none", "elective_postoperative", "nonoperative_or_emergency_postoperative"] } } })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: Apache2Input = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn normal() -> Apache2Input {
        Apache2Input {
            temperature_c: 37.0,
            mean_arterial_pressure_mm_hg: 90.0,
            heart_rate: 80.0,
            respiratory_rate: 16.0,
            fio2: 0.21,
            pao2_mm_hg: Some(90.0),
            aa_gradient_mm_hg: None,
            arterial_ph: 7.40,
            sodium_mmol_l: 140.0,
            potassium_mmol_l: 4.0,
            creatinine_mg_dl: 1.0,
            acute_renal_failure: false,
            hematocrit_percent: 40.0,
            wbc_10_9_l: 8.0,
            glasgow_coma_scale: 15,
            age: 44,
            chronic_health_status: ChronicHealthStatus::None,
        }
    }
    #[test]
    fn normal_young_no_chronic_scores_zero() {
        assert_eq!(compute(&normal()).unwrap().total_score, 0);
    }
    #[test]
    fn gcs_and_age_and_chronic_add() {
        let mut i = normal();
        i.glasgow_coma_scale = 10;
        i.age = 70;
        i.chronic_health_status = ChronicHealthStatus::NonoperativeOrEmergencyPostoperative;
        let out = compute(&i).unwrap();
        assert_eq!(out.total_score, 15);
    }
    #[test]
    fn high_fio2_requires_aa_gradient_and_scores_it() {
        let mut i = normal();
        i.fio2 = 0.6;
        i.pao2_mm_hg = None;
        i.aa_gradient_mm_hg = Some(360.0);
        assert_eq!(compute(&i).unwrap().acute_physiology_score, 3);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let mut i = normal();
        i.glasgow_coma_scale = 10;
        i.age = 70;
        i.chronic_health_status = ChronicHealthStatus::NonoperativeOrEmergencyPostoperative;
        let value = json!({
            "temperature_c": 37.0, "mean_arterial_pressure_mm_hg": 90.0, "heart_rate": 80.0,
            "respiratory_rate": 16.0, "fio2": 0.21, "pao2_mm_hg": 90.0,
            "arterial_ph": 7.40, "sodium_mmol_l": 140.0, "potassium_mmol_l": 4.0,
            "creatinine_mg_dl": 1.0, "hematocrit_percent": 40.0, "wbc_10_9_l": 8.0,
            "glasgow_coma_scale": 10, "age": 70,
            "chronic_health_status": "nonoperative_or_emergency_postoperative"
        });
        let dynamic = Apache2.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&i).unwrap());
        assert_eq!(dynamic.result, json!(15));
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(Apache2.calculate(&json!({"temperature_c": 37.0})).is_err());
    }
}
