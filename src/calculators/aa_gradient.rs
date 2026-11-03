// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Alveolar-arterial (A-a) oxygen gradient, from the alveolar gas equation.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "aa_gradient";
pub const REFERENCE: &str = "Alveolar gas equation: standard respiratory physiology, see e.g. West JB, Luks AM. West's Respiratory Physiology: The Essentials. 11th ed. Wolters Kluwer; 2020. Age-adjusted expected room-air gradient per Mellemgaard K. The alveolar-arterial oxygen difference: its size and components in normal man. Acta Physiol Scand. 1966;67(1):10-20.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain formula - implemented from standard respiratory physiology and the primary literature",
    "https://doi.org/10.1111/j.1748-1716.1966.tb03281.x",
);

/// Standard atmospheric pressure at sea level, mmHg.
const PATM_MM_HG: f64 = 760.0;
/// Water vapour pressure at body temperature (37 degC), mmHg.
const PH2O_MM_HG: f64 = 47.0;
/// Standard assumed respiratory quotient.
const RESPIRATORY_QUOTIENT: f64 = 0.8;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AaGradientInput {
    /// Fraction of inspired oxygen, 0.21 (room air) to 1.0.
    pub fio2: f64,
    pub paco2_mm_hg: f64,
    /// Measured arterial oxygen tension from the same arterial blood gas.
    pub pao2_mm_hg: f64,
    /// Optional age, for the commonly cited age-adjusted expected room-air
    /// gradient (age/4 + 4). Only meaningful when fio2 is 0.21 (room air).
    #[serde(default)]
    pub age_years: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AaGradientOutcome {
    pub alveolar_pao2_mm_hg: f64,
    pub aa_gradient_mm_hg: f64,
    pub expected_room_air_gradient_mm_hg: Option<f64>,
}

pub fn compute(input: &AaGradientInput) -> Result<AaGradientOutcome, CalcError> {
    validate(input.fio2, "fio2", 0.21, 1.0)?;
    validate(input.paco2_mm_hg, "paco2_mm_hg", 10.0, 100.0)?;
    validate(input.pao2_mm_hg, "pao2_mm_hg", 20.0, 650.0)?;
    if let Some(age) = input.age_years {
        validate(age, "age_years", 0.0, 120.0)?;
    }

    let alveolar_pao2 =
        input.fio2 * (PATM_MM_HG - PH2O_MM_HG) - input.paco2_mm_hg / RESPIRATORY_QUOTIENT;
    let aa_gradient = alveolar_pao2 - input.pao2_mm_hg;
    let expected_room_air_gradient_mm_hg = if (input.fio2 - 0.21).abs() < 1e-9 {
        input.age_years.map(|age| age / 4.0 + 4.0)
    } else {
        None
    };

    Ok(AaGradientOutcome {
        alveolar_pao2_mm_hg: alveolar_pao2,
        aa_gradient_mm_hg: aa_gradient,
        expected_room_air_gradient_mm_hg,
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

pub fn build_response(input: &AaGradientInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "alveolar_pao2_mm_hg".into(),
        json!(round1(o.alveolar_pao2_mm_hg)),
    );
    working.insert("unit".into(), json!("mmHg"));
    if let Some(expected) = o.expected_room_air_gradient_mm_hg {
        working.insert(
            "expected_room_air_gradient_mm_hg".into(),
            json!(round1(expected)),
        );
    }

    let interpretation = match o.expected_room_air_gradient_mm_hg {
        Some(expected) => format!(
            "A-a gradient {:.0} mmHg (calculated alveolar PAO2 {:.0} mmHg), against an age-adjusted expected room-air gradient of {:.0} mmHg (age/4 + 4). A gradient above the expected value suggests a diffusion, V/Q mismatch, or shunt problem rather than pure hypoventilation. Assumes sea-level barometric pressure (760 mmHg) and a respiratory quotient of 0.8.",
            o.aa_gradient_mm_hg, o.alveolar_pao2_mm_hg, expected
        ),
        None => format!(
            "A-a gradient {:.0} mmHg (calculated alveolar PAO2 {:.0} mmHg). The commonly cited age-adjusted expected-gradient comparison only applies on room air (FiO2 0.21); on supplemental oxygen, interpret the gradient trend and the clinical context rather than a fixed normal range. Assumes sea-level barometric pressure (760 mmHg) and a respiratory quotient of 0.8.",
            o.aa_gradient_mm_hg, o.alveolar_pao2_mm_hg
        ),
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.aa_gradient_mm_hg)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct AaGradient;
impl Calculator for AaGradient {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Alveolar-arterial (A-a) Gradient"
    }
    fn description(&self) -> &'static str {
        "Alveolar-arterial oxygen gradient from the alveolar gas equation, with the age-adjusted expected room-air gradient."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "AaGradientInput", "type": "object", "additionalProperties": false,
            "required": ["fio2", "paco2_mm_hg", "pao2_mm_hg"],
            "properties": {
                "fio2": { "type": "number", "minimum": 0.21, "maximum": 1.0, "description": "Fraction of inspired oxygen; 0.21 is room air" },
                "paco2_mm_hg": { "type": "number", "minimum": 10, "maximum": 100 },
                "pao2_mm_hg": { "type": "number", "minimum": 20, "maximum": 650, "description": "Measured arterial oxygen tension from the same blood gas" },
                "age_years": { "type": "number", "minimum": 0, "maximum": 120, "description": "Optional; enables the age-adjusted expected room-air gradient (fio2 must be 0.21)" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AaGradientInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_room_air_gas() {
        // Classic normal ABG: FiO2 0.21, PaCO2 40, PaO2 95.
        // PAO2 = 0.21*(713) - 40/0.8 = 149.73 - 50 = 99.73
        let out = compute(&AaGradientInput {
            fio2: 0.21,
            paco2_mm_hg: 40.0,
            pao2_mm_hg: 95.0,
            age_years: None,
        })
        .unwrap();
        assert!((out.alveolar_pao2_mm_hg - 99.73).abs() < 0.01);
        assert!((out.aa_gradient_mm_hg - 4.73).abs() < 0.01);
    }

    #[test]
    fn age_adjusted_expected_only_on_room_air() {
        let out = compute(&AaGradientInput {
            fio2: 0.21,
            paco2_mm_hg: 40.0,
            pao2_mm_hg: 95.0,
            age_years: Some(60.0),
        })
        .unwrap();
        assert_eq!(out.expected_room_air_gradient_mm_hg, Some(19.0));

        let on_oxygen = compute(&AaGradientInput {
            fio2: 0.5,
            paco2_mm_hg: 40.0,
            pao2_mm_hg: 200.0,
            age_years: Some(60.0),
        })
        .unwrap();
        assert_eq!(on_oxygen.expected_room_air_gradient_mm_hg, None);
    }

    #[test]
    fn rejects_out_of_range_fio2() {
        assert!(
            compute(&AaGradientInput {
                fio2: 0.1,
                paco2_mm_hg: 40.0,
                pao2_mm_hg: 95.0,
                age_years: None,
            })
            .is_err()
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"fio2": 0.21, "paco2_mm_hg": 40.0, "pao2_mm_hg": 95.0});
        let typed = AaGradientInput {
            fio2: 0.21,
            paco2_mm_hg: 40.0,
            pao2_mm_hg: 95.0,
            age_years: None,
        };
        let dynamic = AaGradient.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            AaGradient
                .calculate(&json!({"fio2": 0.21, "paco2_mm_hg": 40.0}))
                .is_err()
        );
    }
}
