// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Fat-free mass index (FFMI).
//!
//! An anthropometric index normalising fat-free (lean) body mass to height,
//! analogous to BMI but excluding fat mass. Kouri et al (1995) compared FFMI
//! in 157 male athletes (83 anabolic-androgenic steroid users, 74 non-users)
//! and in 20 Mr. America winners from the pre-steroid era (1939-1959, mean
//! FFMI 25.4). Non-users' normalized FFMI extended up to a well-defined limit
//! of about 25.0 kg/m^2; many steroid users exceeded it, some above 30. The
//! source cohort was exclusively adult male athletes, so this natural-limit
//! context does not establish an equivalent boundary for women.
//!
//! FFMI = fat-free mass (kg) / (height in m)^2
//!
//! Fat-free mass is derived here from weight and body fat percentage:
//! fat-free mass (kg) = weight (kg) x (1 - body fat percent / 100).
//!
//! Because taller individuals carry more fat-free mass for a given build,
//! Kouri et al normalized FFMI to a reference height of 1.8 m:
//!
//! Normalized FFMI = FFMI + 6.3 x (1.8 - height in m)
//!
//! Reference: Kouri EM, Pope HG Jr, Katz DL, Oliva P. Fat-free mass index in
//! users and nonusers of anabolic-androgenic steroids. Clin J Sport Med.
//! 1995;5(4):223-228. doi:10.1097/00042752-199510000-00003.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "fat_free_mass_index";

pub const REFERENCE: &str = "Kouri EM, Pope HG Jr, Katz DL, Oliva P. Fat-free mass index in users and nonusers of anabolic-androgenic steroids. Clin J Sport Med. 1995;5(4):223-228. doi:10.1097/00042752-199510000-00003.";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published mathematical method - independently implemented; formulas and algorithms are not protected by US copyright",
    source_url: "https://www.copyright.gov/circs/circ31.pdf",
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FatFreeMassIndexInput {
    /// Total body weight in kilograms.
    pub weight_kg: f64,
    /// Standing height in centimetres.
    pub height_cm: f64,
    /// Body fat percentage (0-100), from any independently obtained method
    /// (for example DXA, bioimpedance, skinfold, or circumference estimate).
    pub body_fat_percent: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FatFreeMassIndexOutcome {
    pub fat_free_mass_kg: f64,
    pub ffmi: f64,
    pub normalized_ffmi: f64,
    pub interpretation: String,
}

const NORMALIZATION_REFERENCE_HEIGHT_M: f64 = 1.8;
const NORMALIZATION_COEFFICIENT: f64 = 6.3;
const NATURAL_LIMIT_FFMI: f64 = 25.0;

pub fn compute(input: &FatFreeMassIndexInput) -> Result<FatFreeMassIndexOutcome, CalcError> {
    if !(20.0..=400.0).contains(&input.weight_kg) || !input.weight_kg.is_finite() {
        return Err(CalcError::InvalidInput(
            "weight_kg must be finite and between 20 and 400".into(),
        ));
    }
    if !(100.0..=250.0).contains(&input.height_cm) || !input.height_cm.is_finite() {
        return Err(CalcError::InvalidInput(
            "height_cm must be finite and between 100 and 250".into(),
        ));
    }
    if !(2.0..=70.0).contains(&input.body_fat_percent) || !input.body_fat_percent.is_finite() {
        return Err(CalcError::InvalidInput(
            "body_fat_percent must be finite and between 2 and 70 - the essential-fat floor and a practical upper bound"
                .into(),
        ));
    }

    let height_m = input.height_cm / 100.0;
    let fat_free_mass_kg = input.weight_kg * (1.0 - input.body_fat_percent / 100.0);
    let ffmi = fat_free_mass_kg / height_m.powi(2);
    let normalized_ffmi =
        ffmi + NORMALIZATION_COEFFICIENT * (NORMALIZATION_REFERENCE_HEIGHT_M - height_m);

    if !ffmi.is_finite() || ffmi <= 0.0 {
        return Err(CalcError::InvalidInput(
            "computed FFMI is invalid - check inputs".into(),
        ));
    }

    let limit_note = if normalized_ffmi >= NATURAL_LIMIT_FFMI {
        format!(
            "This is at or above the well-defined upper limit (about {NATURAL_LIMIT_FFMI:.1} kg/m^2) reported in non-steroid-using male athletes; the source study found many anabolic-androgenic steroid users exceeded this limit, some above 30."
        )
    } else {
        format!(
            "This is below the well-defined upper limit (about {NATURAL_LIMIT_FFMI:.1} kg/m^2) reported in non-steroid-using male athletes."
        )
    };

    let interpretation = format!(
        "Fat-free mass index (FFMI) {ffmi:.1} kg/m^2, normalized to a 1.8 m reference height as {normalized_ffmi:.1} kg/m^2. {limit_note} The reference cohort (157 male athletes; 20 Mr. America winners from the 1939-1959 pre-steroid era, mean FFMI 25.4) was exclusively adult men, so this natural-limit context does not establish an equivalent boundary for women. Fat-free mass here is derived from weight and an independently obtained body fat percentage, not measured directly; its accuracy depends entirely on that input."
    );

    Ok(FatFreeMassIndexOutcome {
        fat_free_mass_kg,
        ffmi,
        normalized_ffmi,
        interpretation,
    })
}

pub fn build_response(input: &FatFreeMassIndexInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let rounded_ffmi = round1(o.ffmi);
    let rounded_normalized_ffmi = round1(o.normalized_ffmi);

    let mut working = Map::new();
    working.insert("weight_kg".into(), json!(input.weight_kg));
    working.insert("height_cm".into(), json!(input.height_cm));
    working.insert("body_fat_percent".into(), json!(input.body_fat_percent));
    let height_m = input.height_cm / 100.0;
    working.insert("height_m".into(), json!(height_m));
    working.insert("fat_free_mass_kg".into(), json!(round1(o.fat_free_mass_kg)));
    working.insert("ffmi_unrounded".into(), json!(o.ffmi));
    working.insert("ffmi".into(), json!(rounded_ffmi));
    working.insert(
        "normalization_reference_height_m".into(),
        json!(NORMALIZATION_REFERENCE_HEIGHT_M),
    );
    working.insert(
        "normalization_coefficient".into(),
        json!(NORMALIZATION_COEFFICIENT),
    );
    working.insert("normalized_ffmi_unrounded".into(), json!(o.normalized_ffmi));
    working.insert("normalized_ffmi".into(), json!(rounded_normalized_ffmi));
    working.insert("natural_limit_ffmi".into(), json!(NATURAL_LIMIT_FFMI));
    working.insert(
        "at_or_above_natural_limit".into(),
        json!(o.normalized_ffmi >= NATURAL_LIMIT_FFMI),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(rounded_normalized_ffmi),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct FatFreeMassIndex;

impl Calculator for FatFreeMassIndex {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Fat-Free Mass Index (FFMI)"
    }

    fn description(&self) -> &'static str {
        "Height-normalized fat-free (lean) body mass (Kouri 1995), with the height-1.8m-normalized value compared against the well-defined natural limit (about 25.0 kg/m^2) reported in non-steroid-using male athletes; that limit does not establish an equivalent boundary for women."
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
            "title": "FatFreeMassIndexInput",
            "description": "Fat-free mass index (FFMI) from weight, height, and an independently obtained body fat percentage. The Kouri 1995 natural-limit context (about 25.0 kg/m^2 normalized) was derived exclusively from adult male athletes and does not establish an equivalent boundary for women.",
            "type": "object",
            "additionalProperties": false,
            "required": ["weight_kg", "height_cm", "body_fat_percent"],
            "properties": {
                "weight_kg": {
                    "type": "number",
                    "minimum": 20,
                    "maximum": 400,
                    "unit": "kg",
                    "description": "Total body weight in kilograms"
                },
                "height_cm": {
                    "type": "number",
                    "minimum": 100,
                    "maximum": 250,
                    "unit": "cm",
                    "description": "Standing height in centimetres"
                },
                "body_fat_percent": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 70,
                    "unit": "%",
                    "description": "Body fat percentage (2-70) from any independently obtained method (for example DXA, bioimpedance, skinfold, or a circumference-based estimate such as this crate's body_fat_circumference calculator). FFMI accuracy depends entirely on this input."
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: FatFreeMassIndexInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc(weight: f64, height: f64, body_fat_percent: f64) -> FatFreeMassIndexInput {
        FatFreeMassIndexInput {
            weight_kg: weight,
            height_cm: height,
            body_fat_percent,
        }
    }

    #[test]
    fn equation_conformance_vector() {
        // 90 kg at 15% body fat -> 76.5 kg fat-free mass; height 1.80 m so no normalization shift.
        let o = compute(&calc(90.0, 180.0, 15.0)).unwrap();
        assert!(
            (o.fat_free_mass_kg - 76.5).abs() < 1e-9,
            "got {}",
            o.fat_free_mass_kg
        );
        // 76.5 / 1.8^2 = 23.611...
        assert!(
            (o.ffmi - 23.611_111_111_111_11).abs() < 1e-9,
            "got {}",
            o.ffmi
        );
        // height already 1.8 m so normalization adds nothing.
        assert!(
            (o.normalized_ffmi - o.ffmi).abs() < 1e-9,
            "got {} vs {}",
            o.normalized_ffmi,
            o.ffmi
        );
    }

    #[test]
    fn normalization_shifts_shorter_height_upward() {
        // Same fat-free mass (76.5 kg) at 170 cm instead of 180 cm.
        // FFMI = 76.5 / 1.7^2 = 26.470588...
        // Normalized = FFMI + 6.3 * (1.8 - 1.7) = FFMI + 0.63
        let o = compute(&calc(90.0, 170.0, 15.0)).unwrap();
        assert!(
            (o.ffmi - 26.470_588_235_294_12).abs() < 1e-9,
            "got {}",
            o.ffmi
        );
        assert!(
            (o.normalized_ffmi - (o.ffmi + 0.63)).abs() < 1e-9,
            "got {} vs {}",
            o.normalized_ffmi,
            o.ffmi + 0.63
        );
    }

    #[test]
    fn mr_america_era_reference_point_is_near_natural_limit() {
        // A build broadly consistent with the pre-steroid-era Mr. America mean
        // (normalized FFMI 25.4): ~90 kg at 8% body fat, 180 cm.
        let o = compute(&calc(90.0, 180.0, 8.0)).unwrap();
        assert!(
            o.normalized_ffmi > NATURAL_LIMIT_FFMI,
            "got {}",
            o.normalized_ffmi
        );
    }

    #[test]
    fn accepts_boundary_inputs() {
        assert!(compute(&calc(20.0, 100.0, 2.0)).is_ok());
        assert!(compute(&calc(400.0, 250.0, 70.0)).is_ok());
    }

    #[test]
    fn rejects_out_of_range_weight() {
        assert!(compute(&calc(19.9, 180.0, 15.0)).is_err());
        assert!(compute(&calc(400.1, 180.0, 15.0)).is_err());
        assert!(compute(&calc(f64::NAN, 180.0, 15.0)).is_err());
    }

    #[test]
    fn rejects_out_of_range_height() {
        assert!(compute(&calc(90.0, 99.9, 15.0)).is_err());
        assert!(compute(&calc(90.0, 250.1, 15.0)).is_err());
        assert!(compute(&calc(90.0, f64::INFINITY, 15.0)).is_err());
    }

    #[test]
    fn rejects_out_of_range_body_fat_percent() {
        assert!(compute(&calc(90.0, 180.0, 1.9)).is_err());
        assert!(compute(&calc(90.0, 180.0, 70.1)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "weight_kg": 90.0,
            "height_cm": 180.0,
            "body_fat_percent": 15.0
        });
        let dynamic = FatFreeMassIndex.calculate(&value).unwrap();
        let typed = build_response(&calc(90.0, 180.0, 15.0)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn dynamic_surface_rejects_unknown_fields() {
        let invalid = json!({
            "weight_kg": 90.0,
            "height_cm": 180.0,
            "body_fat_percent": 15.0,
            "unexpected": true
        });
        assert!(FatFreeMassIndex.calculate(&invalid).is_err());
    }

    #[test]
    fn response_preserves_inputs_and_equation_working() {
        let response = build_response(&calc(90.0, 170.0, 15.0)).unwrap();
        assert_eq!(response.working["weight_kg"], json!(90.0));
        assert_eq!(response.working["height_cm"], json!(170.0));
        assert_eq!(response.working["body_fat_percent"], json!(15.0));
        assert_eq!(
            response.working["normalization_reference_height_m"],
            json!(1.8)
        );
        assert_eq!(response.working["normalization_coefficient"], json!(6.3));
        assert_eq!(response.working["natural_limit_ffmi"], json!(25.0));
        assert_eq!(response.result, response.working["normalized_ffmi"]);
        assert!(
            response
                .interpretation
                .contains("does not establish an equivalent boundary for women")
        );
    }

    #[test]
    fn schema_documents_units_and_applicability_contract() {
        let schema = FatFreeMassIndex.input_schema();
        assert_eq!(schema["properties"]["weight_kg"]["unit"], json!("kg"));
        assert_eq!(schema["properties"]["height_cm"]["unit"], json!("cm"));
        assert_eq!(schema["properties"]["body_fat_percent"]["unit"], json!("%"));
        let description = schema["description"].as_str().unwrap();
        assert!(description.contains("does not establish an equivalent boundary for women"));
    }
}
