// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! PESI and simplified PESI (sPESI) - 30-day mortality risk after a
//! diagnosis of pulmonary embolism.
//!
//! The original PESI (Aujesky et al. Am J Respir Crit Care Med 2005) sums
//! age in years plus weighted points for ten clinical variables into five
//! risk classes. The simplified PESI (Jimenez et al. Arch Intern Med 2010)
//! reduces this to six binary variables; a score of 0 identifies the
//! low-risk group. Both are computed here from one shared set of raw
//! clinical inputs, since they draw on an overlapping variable set.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "pesi";

pub const REFERENCE: &str = "Aujesky D, Obrosky DS, Stone RA, et al. Derivation and validation of a prognostic \
model for pulmonary embolism. Am J Respir Crit Care Med. 2005;172(8):1041-1046 (original PESI); \
Jimenez D, Aujesky D, Moores L, et al. Simplification of the pulmonary embolism severity index for \
prognostication in patients with acute symptomatic pulmonary embolism. Arch Intern Med. \
2010;170(15):1383-1389 (simplified PESI).";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.1164/rccm.200506-862OC",
);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PesiInput {
    pub age_years: f64,
    pub male_sex: bool,
    pub history_of_cancer: bool,
    pub history_of_heart_failure: bool,
    pub history_of_chronic_lung_disease: bool,
    pub pulse_bpm: f64,
    pub systolic_bp_mmhg: f64,
    pub respiratory_rate_per_min: f64,
    pub temperature_celsius: f64,
    /// Disorientation, lethargy, stupor, or coma.
    pub altered_mental_status: bool,
    pub oxygen_saturation_percent: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PesiClass {
    I,
    II,
    III,
    IV,
    V,
}

impl PesiClass {
    fn from_points(points: i32) -> Self {
        match points {
            p if p <= 65 => Self::I,
            p if p <= 85 => Self::II,
            p if p <= 105 => Self::III,
            p if p <= 125 => Self::IV,
            _ => Self::V,
        }
    }
    fn slug(self) -> &'static str {
        match self {
            Self::I => "I",
            Self::II => "II",
            Self::III => "III",
            Self::IV => "IV",
            Self::V => "V",
        }
    }
    /// 30-day mortality range across the derivation/validation samples.
    fn mortality_range(self) -> &'static str {
        match self {
            Self::I => "0-1.6%",
            Self::II => "1.7-3.5%",
            Self::III => "3.2-7.1%",
            Self::IV => "4.0-11.4%",
            Self::V => "10.0-24.5%",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PesiOutcome {
    pub original_points: i32,
    pub original_class: PesiClass,
    pub simplified_score: u8,
    pub simplified_high_risk: bool,
    pub chronic_cardiopulmonary_disease: bool,
}

pub fn compute(input: &PesiInput) -> Result<PesiOutcome, CalcError> {
    if !input.age_years.is_finite() || !(0.0..=130.0).contains(&input.age_years) {
        return Err(CalcError::InvalidInput(
            "age_years must be between 0 and 130".into(),
        ));
    }
    for (name, value) in [
        ("pulse_bpm", input.pulse_bpm),
        ("systolic_bp_mmhg", input.systolic_bp_mmhg),
        ("respiratory_rate_per_min", input.respiratory_rate_per_min),
        ("oxygen_saturation_percent", input.oxygen_saturation_percent),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be a positive finite number"
            )));
        }
    }
    if !input.temperature_celsius.is_finite() {
        return Err(CalcError::InvalidInput(
            "temperature_celsius must be a finite number".into(),
        ));
    }

    let pulse_over_110 = input.pulse_bpm >= 110.0;
    let sbp_under_100 = input.systolic_bp_mmhg < 100.0;
    let rr_over_30 = input.respiratory_rate_per_min >= 30.0;
    let temp_under_36 = input.temperature_celsius < 36.0;
    let spo2_under_90 = input.oxygen_saturation_percent < 90.0;
    let chronic_cardiopulmonary_disease =
        input.history_of_heart_failure || input.history_of_chronic_lung_disease;

    let original_points = input.age_years.round() as i32
        + i32::from(input.male_sex) * 10
        + i32::from(input.history_of_cancer) * 30
        + i32::from(input.history_of_heart_failure) * 10
        + i32::from(input.history_of_chronic_lung_disease) * 10
        + i32::from(pulse_over_110) * 20
        + i32::from(sbp_under_100) * 30
        + i32::from(rr_over_30) * 20
        + i32::from(temp_under_36) * 20
        + i32::from(input.altered_mental_status) * 60
        + i32::from(spo2_under_90) * 20;

    let simplified_score = u8::from(input.age_years > 80.0)
        + u8::from(input.history_of_cancer)
        + u8::from(chronic_cardiopulmonary_disease)
        + u8::from(pulse_over_110)
        + u8::from(sbp_under_100)
        + u8::from(spo2_under_90);

    Ok(PesiOutcome {
        original_points,
        original_class: PesiClass::from_points(original_points),
        simplified_score,
        simplified_high_risk: simplified_score >= 1,
        chronic_cardiopulmonary_disease,
    })
}

pub fn build_response(input: &PesiInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("original_pesi_points".into(), json!(o.original_points));
    working.insert("original_pesi_class".into(), json!(o.original_class.slug()));
    working.insert(
        "original_pesi_mortality_range".into(),
        json!(o.original_class.mortality_range()),
    );
    working.insert("simplified_pesi_score".into(), json!(o.simplified_score));
    working.insert(
        "simplified_pesi_high_risk".into(),
        json!(o.simplified_high_risk),
    );
    working.insert(
        "simplified_pesi_mortality".into(),
        json!(if o.simplified_high_risk {
            "8.9%"
        } else {
            "1.1%"
        }),
    );

    let interpretation = format!(
        "PESI {} points, risk class {} (30-day mortality {} across derivation/validation samples). Simplified PESI {}/6: {} (30-day mortality {}).",
        o.original_points,
        o.original_class.slug(),
        o.original_class.mortality_range(),
        o.simplified_score,
        if o.simplified_high_risk {
            "not low risk"
        } else {
            "low risk, score 0"
        },
        if o.simplified_high_risk {
            "8.9%"
        } else {
            "1.1%"
        }
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.original_points),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Pesi;
impl Calculator for Pesi {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "PESI / Simplified PESI (Pulmonary Embolism)"
    }
    fn description(&self) -> &'static str {
        "30-day mortality risk stratification after pulmonary embolism, reporting both the original 5-class PESI and the binary simplified PESI."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "PesiInput", "type": "object",
            "additionalProperties": false,
            "required": [
                "age_years", "male_sex", "history_of_cancer", "history_of_heart_failure",
                "history_of_chronic_lung_disease", "pulse_bpm", "systolic_bp_mmhg",
                "respiratory_rate_per_min", "temperature_celsius", "altered_mental_status",
                "oxygen_saturation_percent"
            ],
            "properties": {
                "age_years": { "type": "number", "minimum": 0, "maximum": 130, "description": "Age in years" },
                "male_sex": { "type": "boolean", "description": "Male sex (original PESI only, +10 points)" },
                "history_of_cancer": { "type": "boolean", "description": "Active or recently treated cancer" },
                "history_of_heart_failure": { "type": "boolean", "description": "History of heart failure" },
                "history_of_chronic_lung_disease": { "type": "boolean", "description": "History of chronic lung disease" },
                "pulse_bpm": { "type": "number", "exclusiveMinimum": 0, "maximum": 300, "description": "Pulse in beats per minute; >= 110 scores" },
                "systolic_bp_mmhg": { "type": "number", "exclusiveMinimum": 0, "maximum": 300, "description": "Systolic BP in mmHg; < 100 scores" },
                "respiratory_rate_per_min": { "type": "number", "exclusiveMinimum": 0, "maximum": 80, "description": "Respiratory rate; >= 30 scores (original PESI only)" },
                "temperature_celsius": { "type": "number", "minimum": 25, "maximum": 45, "description": "Temperature in Celsius; < 36 scores (original PESI only)" },
                "altered_mental_status": { "type": "boolean", "description": "Disorientation, lethargy, stupor, or coma (original PESI only, +60 points)" },
                "oxygen_saturation_percent": { "type": "number", "exclusiveMinimum": 0, "maximum": 100, "description": "Arterial oxygen saturation, with or without supplemental oxygen; < 90% scores" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: PesiInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn well(age: f64) -> PesiInput {
        PesiInput {
            age_years: age,
            male_sex: false,
            history_of_cancer: false,
            history_of_heart_failure: false,
            history_of_chronic_lung_disease: false,
            pulse_bpm: 80.0,
            systolic_bp_mmhg: 120.0,
            respiratory_rate_per_min: 16.0,
            temperature_celsius: 37.0,
            altered_mental_status: false,
            oxygen_saturation_percent: 97.0,
        }
    }

    #[test]
    fn age_alone_sets_original_points() {
        let o = compute(&well(50.0)).unwrap();
        assert_eq!(o.original_points, 50);
        assert_eq!(o.original_class, PesiClass::I);
        assert_eq!(o.simplified_score, 0);
        assert!(!o.simplified_high_risk);
    }

    #[test]
    fn male_sex_adds_ten_points_to_original_only() {
        let mut i = well(50.0);
        i.male_sex = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.original_points, 60);
        assert_eq!(o.simplified_score, 0); // sPESI has no sex item
    }

    #[test]
    fn altered_mental_status_adds_sixty_to_original_only() {
        let mut i = well(50.0);
        i.altered_mental_status = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.original_points, 110);
        assert_eq!(o.original_class, PesiClass::IV);
        assert_eq!(o.simplified_score, 0);
    }

    #[test]
    fn heart_failure_or_lung_disease_both_count_as_one_spesi_point() {
        let mut i = well(50.0);
        i.history_of_heart_failure = true;
        i.history_of_chronic_lung_disease = true;
        let o = compute(&i).unwrap();
        assert!(o.chronic_cardiopulmonary_disease);
        assert_eq!(o.simplified_score, 1); // one point, not two
        assert_eq!(o.original_points, 50 + 10 + 10); // but two separate 10s in original
    }

    #[test]
    fn age_over_80_scores_one_spesi_point() {
        let mut i = well(85.0);
        i.pulse_bpm = 80.0;
        let o = compute(&i).unwrap();
        assert_eq!(o.simplified_score, 1);
        assert!(o.simplified_high_risk);
    }

    #[test]
    fn high_risk_class_v_example() {
        let i = PesiInput {
            age_years: 70.0,
            male_sex: true,
            history_of_cancer: true,
            history_of_heart_failure: true,
            history_of_chronic_lung_disease: false,
            pulse_bpm: 120.0,
            systolic_bp_mmhg: 90.0,
            respiratory_rate_per_min: 32.0,
            temperature_celsius: 35.0,
            altered_mental_status: true,
            oxygen_saturation_percent: 85.0,
        };
        let o = compute(&i).unwrap();
        // 70 + 10 + 30 + 10 + 20 + 30 + 20 + 20 + 60 + 20 = 290
        assert_eq!(o.original_points, 290);
        assert_eq!(o.original_class, PesiClass::V);
    }

    #[test]
    fn rejects_non_positive_vitals() {
        let mut i = well(50.0);
        i.pulse_bpm = 0.0;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let typed = well(60.0);
        let value = json!({
            "age_years": 60.0, "male_sex": false, "history_of_cancer": false, "history_of_heart_failure": false,
            "history_of_chronic_lung_disease": false, "pulse_bpm": 80.0, "systolic_bp_mmhg": 120.0,
            "respiratory_rate_per_min": 16.0, "temperature_celsius": 37.0, "altered_mental_status": false,
            "oxygen_saturation_percent": 97.0
        });
        let dynamic = Pesi.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(60));
    }
}
