// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Body mass index (BMI).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "bmi";
pub const REFERENCE: &str = "World Health Organization. Obesity: preventing and managing the global epidemic. WHO Technical Report Series 894. 2000. BMI = weight(kg) / height(m)^2.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - standard anthropometric index",
    source_url: "https://apps.who.int/iris/handle/10665/42330",
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BmiInput {
    pub weight_kg: f64,
    pub height_cm: f64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BmiCategory {
    Underweight,
    Healthy,
    Overweight,
    ObesityClass1,
    ObesityClass2,
    ObesityClass3,
}
impl BmiCategory {
    fn label(self) -> &'static str {
        match self {
            BmiCategory::Underweight => "underweight",
            BmiCategory::Healthy => "healthy weight",
            BmiCategory::Overweight => "overweight",
            BmiCategory::ObesityClass1 => "obesity class I",
            BmiCategory::ObesityClass2 => "obesity class II",
            BmiCategory::ObesityClass3 => "obesity class III",
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct BmiOutcome {
    pub bmi: f64,
    pub category: BmiCategory,
    pub interpretation: String,
}

pub fn compute(input: &BmiInput) -> Result<BmiOutcome, CalcError> {
    if !(20.0..=500.0).contains(&input.weight_kg) || !input.weight_kg.is_finite() {
        return Err(CalcError::InvalidInput(
            "weight_kg must be finite and between 20 and 500".into(),
        ));
    }
    if !(50.0..=250.0).contains(&input.height_cm) || !input.height_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "height_cm must be finite and between 50 and 250".into(),
        ));
    }
    let h = input.height_cm / 100.0;
    let bmi = input.weight_kg / (h * h);
    let category = if bmi < 18.5 {
        BmiCategory::Underweight
    } else if bmi < 25.0 {
        BmiCategory::Healthy
    } else if bmi < 30.0 {
        BmiCategory::Overweight
    } else if bmi < 35.0 {
        BmiCategory::ObesityClass1
    } else if bmi < 40.0 {
        BmiCategory::ObesityClass2
    } else {
        BmiCategory::ObesityClass3
    };
    let interpretation = format!(
        "BMI {:.1} kg/m2: {} by standard WHO adult categories. BMI is a screening index and does not directly measure body composition or cardiometabolic risk.",
        bmi,
        category.label()
    );
    Ok(BmiOutcome {
        bmi,
        category,
        interpretation,
    })
}

pub fn build_response(input: &BmiInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("category".into(), json!(o.category.label()));
    working.insert("unit".into(), json!("kg/m2"));
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.bmi)),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}
fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct Bmi;
impl Calculator for Bmi {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "BMI (Body Mass Index)"
    }
    fn description(&self) -> &'static str {
        "Body mass index from weight and height, with standard adult category."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({ "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "BmiInput", "type": "object", "additionalProperties": false, "required": ["weight_kg", "height_cm"], "properties": { "weight_kg": { "type": "number", "minimum": 20, "maximum": 500 }, "height_cm": { "type": "number", "minimum": 50, "maximum": 250 } } })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: BmiInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn computes_bmi() {
        let out = compute(&BmiInput {
            weight_kg: 70.0,
            height_cm: 175.0,
        })
        .unwrap();
        assert_eq!(round1(out.bmi), 22.9);
        assert_eq!(out.category, BmiCategory::Healthy);
    }
    #[test]
    fn obesity_boundary() {
        assert_eq!(
            compute(&BmiInput {
                weight_kg: 90.0,
                height_cm: 173.2
            })
            .unwrap()
            .category,
            BmiCategory::ObesityClass1
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let typed = BmiInput {
            weight_kg: 70.0,
            height_cm: 175.0,
        };
        let value = json!({"weight_kg": 70.0, "height_cm": 175.0});
        let dynamic = Bmi.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_height() {
        assert!(Bmi.calculate(&json!({"weight_kg": 70.0})).is_err());
    }
}
