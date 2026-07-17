// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Cockcroft-Gault creatinine clearance.
//!
//! Estimates creatinine clearance (CrCl) in mL/min from age, weight, sex, and
//! serum creatinine. Published in 1976 and historically used for both CKD
//! staging and renal drug dosing.
//!
//! **Superseded for CKD staging** by CKD-EPI 2021 (see the `egfr` calculator),
//! which is now recommended by KDIGO and NICE. Cockcroft-Gault remains in
//! widespread clinical use for **renal drug dosing** because many drug-dosing
//! guidelines and marketing-authorisation studies were built on CrCl, and
//! because the non-BSA-normalised mL/min value it produces suits dose
//! calculations better than eGFR (mL/min/1.73 m²).
//!
//! Formula:
//! ```text
//! CrCl (mL/min) = [(140 − age) × weight_kg] / (72 × Scr_mg/dL)
//!                 × 0.85  (female only)
//! ```
//!
//! Weight note: the original formula uses actual body weight. For obese
//! patients, many pharmacokinetic guidelines recommend ideal body weight (IBW)
//! or adjusted body weight (AdjBW = IBW + 0.4 × (ABW − IBW)) where ABW > 1.3 ×
//! IBW; the appropriate choice is drug- and guideline-specific and is not
//! resolved by this calculator.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "cockcroft_gault";

pub const REFERENCE: &str = "Cockcroft DW, Gault MH. Prediction of creatinine clearance from serum creatinine. \
Nephron. 1976;16(1):31-41. doi:10.1159/000180580";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1159/000180580",
};

pub const UMOL_PER_MGDL: f64 = 88.4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreatinineUnit {
    #[serde(rename = "mg/dL")]
    MgDl,
    #[serde(rename = "umol/L")]
    UmolL,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CockcroftGaultInput {
    /// Age in years (validated in adults).
    pub age: u8,
    /// Body weight in kg. See module-level weight note for obese patients.
    pub weight_kg: f64,
    pub sex: Sex,
    /// Serum creatinine, in the unit given by `creatinine_unit`.
    pub creatinine: f64,
    pub creatinine_unit: CreatinineUnit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CockcroftGaultOutcome {
    /// Creatinine clearance in mL/min.
    pub crcl_ml_min: f64,
    /// Creatinine in mg/dL (after unit conversion).
    pub scr_mgdl: f64,
    pub interpretation: String,
}

pub fn compute(input: &CockcroftGaultInput) -> Result<CockcroftGaultOutcome, CalcError> {
    if input.age < 18 {
        return Err(CalcError::InvalidInput(
            "Cockcroft-Gault is validated in adults (age 18+)".into(),
        ));
    }
    if !input.weight_kg.is_finite() || input.weight_kg <= 0.0 {
        return Err(CalcError::InvalidInput(
            "weight_kg must be a positive number".into(),
        ));
    }
    if !input.creatinine.is_finite() || input.creatinine <= 0.0 {
        return Err(CalcError::InvalidInput(
            "creatinine must be a positive number".into(),
        ));
    }

    let scr_mgdl = match input.creatinine_unit {
        CreatinineUnit::MgDl => input.creatinine,
        CreatinineUnit::UmolL => input.creatinine / UMOL_PER_MGDL,
    };

    let sex_factor = match input.sex {
        Sex::Male => 1.0,
        Sex::Female => 0.85,
    };

    let crcl = ((140.0 - input.age as f64) * input.weight_kg / (72.0 * scr_mgdl)) * sex_factor;
    let crcl = crcl.max(0.0);

    let sex_label = match input.sex {
        Sex::Male => "male",
        Sex::Female => "female (×0.85)",
    };

    let interpretation = format!(
        "Cockcroft-Gault CrCl {:.1} mL/min (age {}, {:.1} kg, Scr {:.3} mg/dL, {}). \
This is creatinine clearance, not eGFR. Use for drug dosing where the guideline cites \
CrCl. For CKD staging, use CKD-EPI 2021 eGFR instead (Cockcroft-Gault is superseded for \
that purpose - KDIGO/NICE). In obesity, consider whether actual, ideal, or adjusted body \
weight is appropriate per the relevant drug guideline.",
        crcl, input.age, input.weight_kg, scr_mgdl, sex_label,
    );

    Ok(CockcroftGaultOutcome {
        crcl_ml_min: crcl,
        scr_mgdl,
        interpretation,
    })
}

pub fn build_response(input: &CockcroftGaultInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("crcl_ml_min".into(), json!(round1(o.crcl_ml_min)));
    working.insert("scr_mgdl_used".into(), json!(round3(o.scr_mgdl)));
    working.insert(
        "sex_factor".into(),
        json!(match input.sex {
            Sex::Male => 1.0,
            Sex::Female => 0.85,
        }),
    );
    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.crcl_ml_min)),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

pub struct CockcroftGault;

impl Calculator for CockcroftGault {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Cockcroft-Gault Creatinine Clearance"
    }

    fn description(&self) -> &'static str {
        "Creatinine clearance (CrCl, mL/min) from age, weight, sex, and creatinine. Superseded by CKD-EPI 2021 for CKD staging; retained for renal drug dosing."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "CockcroftGaultInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["age", "weight_kg", "sex", "creatinine", "creatinine_unit"],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 120,
                    "description": "Age in years (validated in adults 18+)"
                },
                "weight_kg": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Body weight in kg. Use actual body weight for most patients. For obese patients (ABW > 1.3 × IBW), many pharmacokinetic guidelines recommend ideal or adjusted body weight - the appropriate choice is drug-specific."
                },
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Sex (female applies a 0.85 multiplier to the result)"
                },
                "creatinine": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Serum creatinine, in the unit given by creatinine_unit"
                },
                "creatinine_unit": {
                    "type": "string",
                    "enum": ["mg/dL", "umol/L"],
                    "description": "Unit of the creatinine value (UK labs report umol/L; the equation uses mg/dL internally, 1 mg/dL = 88.4 umol/L)",
                    "definition": {
                        "concept": "Creatinine unit",
                        "statement": "The Cockcroft-Gault formula is defined in mg/dL. UK laboratories report umol/L; the value is converted internally.",
                        "excludes": [
                            "Do NOT pass a umol/L value while labelling it mg/dL: the two differ by ~88x and the wrong label produces a grossly wrong CrCl"
                        ],
                        "source": {
                            "citation": "Cockcroft DW, Gault MH. Nephron. 1976;16(1):31-41.",
                            "url": "https://doi.org/10.1159/000180580"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: CockcroftGaultInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(
        age: u8,
        weight_kg: f64,
        sex: Sex,
        creatinine: f64,
        unit: CreatinineUnit,
    ) -> CockcroftGaultInput {
        CockcroftGaultInput {
            age,
            weight_kg,
            sex,
            creatinine,
            creatinine_unit: unit,
        }
    }

    #[test]
    fn male_worked_example() {
        // (140-85)*72 / (72*1.0) = 55.0
        let o = compute(&inp(85, 72.0, Sex::Male, 1.0, CreatinineUnit::MgDl)).unwrap();
        assert!(
            (o.crcl_ml_min - 55.0).abs() < 0.5,
            "expected ~55.0, got {}",
            o.crcl_ml_min
        );
    }

    #[test]
    fn female_multiplier_applied() {
        // Same as male example * 0.85 = 46.75
        let o = compute(&inp(85, 72.0, Sex::Female, 1.0, CreatinineUnit::MgDl)).unwrap();
        assert!(
            (o.crcl_ml_min - 46.75).abs() < 0.5,
            "expected ~46.75, got {}",
            o.crcl_ml_min
        );
    }

    #[test]
    fn typical_adult_male() {
        // 50yo male, 80kg, Scr 1.0 mg/dL: (140-50)*80/(72*1.0) = 7200/72 = 100.0
        let o = compute(&inp(50, 80.0, Sex::Male, 1.0, CreatinineUnit::MgDl)).unwrap();
        assert!(
            (o.crcl_ml_min - 100.0).abs() < 0.5,
            "expected ~100.0, got {}",
            o.crcl_ml_min
        );
    }

    #[test]
    fn umol_l_matches_mgdl() {
        // 88.4 umol/L == 1.0 mg/dL exactly.
        let a = compute(&inp(50, 80.0, Sex::Male, 1.0, CreatinineUnit::MgDl)).unwrap();
        let b = compute(&inp(50, 80.0, Sex::Male, 88.4, CreatinineUnit::UmolL)).unwrap();
        assert!(
            (a.crcl_ml_min - b.crcl_ml_min).abs() < 0.1,
            "umol/L and mg/dL should give the same result"
        );
    }

    #[test]
    fn high_creatinine_gives_low_crcl() {
        // 70yo male, 70kg, Scr 5.0 mg/dL: (140-70)*70/(72*5.0) = 4900/360 ≈ 13.6
        let o = compute(&inp(70, 70.0, Sex::Male, 5.0, CreatinineUnit::MgDl)).unwrap();
        assert!(
            o.crcl_ml_min < 20.0,
            "expected low CrCl, got {}",
            o.crcl_ml_min
        );
    }

    #[test]
    fn interpretation_contains_supersession_warning() {
        let o = compute(&inp(60, 70.0, Sex::Male, 1.0, CreatinineUnit::MgDl)).unwrap();
        assert!(o.interpretation.contains("superseded"));
    }

    #[test]
    fn rejects_bad_inputs() {
        assert!(compute(&inp(16, 70.0, Sex::Male, 1.0, CreatinineUnit::MgDl)).is_err());
        assert!(compute(&inp(50, 0.0, Sex::Male, 1.0, CreatinineUnit::MgDl)).is_err());
        assert!(compute(&inp(50, 70.0, Sex::Male, 0.0, CreatinineUnit::MgDl)).is_err());
        assert!(compute(&inp(50, 70.0, Sex::Male, f64::NAN, CreatinineUnit::MgDl)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age": 50, "weight_kg": 80.0, "sex": "male",
            "creatinine": 1.0, "creatinine_unit": "mg/dL"
        });
        let dynamic = CockcroftGault.calculate(&value).unwrap();
        let typed = build_response(&inp(50, 80.0, Sex::Male, 1.0, CreatinineUnit::MgDl)).unwrap();
        assert_eq!(dynamic, typed);
    }
}
