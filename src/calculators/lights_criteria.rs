// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Light's Criteria - exudate-versus-transudate classification for pleural
//! effusion (Light et al. Ann Intern Med 1972).
//!
//! An effusion is exudative if it meets any one of three criteria. The
//! first two are ratios and are unit-independent provided the pleural and
//! serum values are supplied in matching units; the third needs the local
//! laboratory's upper limit of normal for serum LDH, which is lab-specific
//! and so is a required input rather than a constant.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "lights_criteria";

pub const REFERENCE: &str = "Light RW, Macgregor MI, Luchsinger PC, Ball WC Jr. Pleural effusions: the diagnostic \
separation of transudates and exudates. Ann Intern Med. 1972;77(4):507-513.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.7326/0003-4819-77-4-507",
);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightsCriteriaInput {
    pub pleural_fluid_protein: f64,
    pub serum_protein: f64,
    pub pleural_fluid_ldh_iu_l: f64,
    pub serum_ldh_iu_l: f64,
    /// The local laboratory's upper limit of normal for serum LDH, in the
    /// same units as `pleural_fluid_ldh_iu_l` and `serum_ldh_iu_l`.
    pub serum_ldh_upper_limit_of_normal_iu_l: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightsCriteriaOutcome {
    pub protein_ratio: f64,
    pub ldh_ratio: f64,
    pub protein_ratio_criterion: bool,
    pub ldh_ratio_criterion: bool,
    pub ldh_absolute_criterion: bool,
    pub is_exudate: bool,
}

pub fn compute(input: &LightsCriteriaInput) -> Result<LightsCriteriaOutcome, CalcError> {
    for (name, value) in [
        ("pleural_fluid_protein", input.pleural_fluid_protein),
        ("serum_protein", input.serum_protein),
        ("pleural_fluid_ldh_iu_l", input.pleural_fluid_ldh_iu_l),
        ("serum_ldh_iu_l", input.serum_ldh_iu_l),
        (
            "serum_ldh_upper_limit_of_normal_iu_l",
            input.serum_ldh_upper_limit_of_normal_iu_l,
        ),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be a positive finite number"
            )));
        }
    }

    let protein_ratio = input.pleural_fluid_protein / input.serum_protein;
    let ldh_ratio = input.pleural_fluid_ldh_iu_l / input.serum_ldh_iu_l;
    let protein_ratio_criterion = protein_ratio > 0.5;
    let ldh_ratio_criterion = ldh_ratio > 0.6;
    let ldh_absolute_criterion =
        input.pleural_fluid_ldh_iu_l > (2.0 / 3.0) * input.serum_ldh_upper_limit_of_normal_iu_l;

    Ok(LightsCriteriaOutcome {
        protein_ratio,
        ldh_ratio,
        protein_ratio_criterion,
        ldh_ratio_criterion,
        ldh_absolute_criterion,
        is_exudate: protein_ratio_criterion || ldh_ratio_criterion || ldh_absolute_criterion,
    })
}

pub fn build_response(input: &LightsCriteriaInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("protein_ratio".into(), json!(round2(o.protein_ratio)));
    working.insert("ldh_ratio".into(), json!(round2(o.ldh_ratio)));
    working.insert(
        "protein_ratio_criterion".into(),
        json!(o.protein_ratio_criterion),
    );
    working.insert("ldh_ratio_criterion".into(), json!(o.ldh_ratio_criterion));
    working.insert(
        "ldh_absolute_criterion".into(),
        json!(o.ldh_absolute_criterion),
    );

    let interpretation = if o.is_exudate {
        "Exudate: at least one of Light's three criteria is met (pleural/serum protein ratio > 0.5, pleural/serum LDH ratio > 0.6, or pleural LDH > two-thirds the upper limit of normal for serum LDH). Light's criteria are highly sensitive but not fully specific for exudate; if the clinical picture suggests a transudate (e.g. heart failure on diuretics), consider the serum-pleural albumin gradient (transudate favoured if > 1.2 g/dL).".to_string()
    } else {
        "Transudate: none of Light's three criteria is met.".to_string()
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.is_exudate),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub struct LightsCriteria;
impl Calculator for LightsCriteria {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Light's Criteria (Pleural Effusion)"
    }
    fn description(&self) -> &'static str {
        "Exudate-versus-transudate classification of pleural effusion from pleural-fluid-to-serum protein and LDH ratios."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "LightsCriteriaInput", "type": "object",
            "additionalProperties": false,
            "required": [
                "pleural_fluid_protein", "serum_protein", "pleural_fluid_ldh_iu_l", "serum_ldh_iu_l",
                "serum_ldh_upper_limit_of_normal_iu_l"
            ],
            "properties": {
                "pleural_fluid_protein": { "type": "number", "exclusiveMinimum": 0, "description": "Pleural fluid total protein, same units as serum_protein (e.g. g/L or g/dL)" },
                "serum_protein": { "type": "number", "exclusiveMinimum": 0, "description": "Serum total protein, same units as pleural_fluid_protein" },
                "pleural_fluid_ldh_iu_l": { "type": "number", "exclusiveMinimum": 0, "description": "Pleural fluid LDH in IU/L" },
                "serum_ldh_iu_l": { "type": "number", "exclusiveMinimum": 0, "description": "Serum LDH in IU/L" },
                "serum_ldh_upper_limit_of_normal_iu_l": { "type": "number", "exclusiveMinimum": 0, "description": "The local laboratory's upper limit of normal for serum LDH, in IU/L; this is lab-specific and must be supplied" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: LightsCriteriaInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> LightsCriteriaInput {
        LightsCriteriaInput {
            pleural_fluid_protein: 2.0,
            serum_protein: 7.0,
            pleural_fluid_ldh_iu_l: 100.0,
            serum_ldh_iu_l: 200.0,
            serum_ldh_upper_limit_of_normal_iu_l: 250.0,
        }
    }

    #[test]
    fn clear_transudate_meets_no_criterion() {
        let o = compute(&base()).unwrap();
        assert!(!o.is_exudate);
    }

    #[test]
    fn protein_ratio_over_half_is_exudate() {
        let mut i = base();
        i.pleural_fluid_protein = 4.0; // ratio 4/7 ~ 0.57
        let o = compute(&i).unwrap();
        assert!(o.protein_ratio_criterion);
        assert!(o.is_exudate);
    }

    #[test]
    fn ldh_ratio_over_point_six_is_exudate() {
        let mut i = base();
        i.pleural_fluid_ldh_iu_l = 150.0; // ratio 150/200 = 0.75
        let o = compute(&i).unwrap();
        assert!(o.ldh_ratio_criterion);
        assert!(o.is_exudate);
    }

    #[test]
    fn absolute_ldh_over_two_thirds_uln_is_exudate() {
        let mut i = base();
        // Keep ratios below threshold but push absolute LDH above 2/3 * 250 = 166.7.
        i.pleural_fluid_ldh_iu_l = 170.0;
        i.serum_ldh_iu_l = 300.0; // ratio 170/300 = 0.567, below 0.6
        let o = compute(&i).unwrap();
        assert!(!o.ldh_ratio_criterion);
        assert!(o.ldh_absolute_criterion);
        assert!(o.is_exudate);
    }

    #[test]
    fn rejects_non_positive_values() {
        let mut i = base();
        i.serum_protein = 0.0;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let typed = base();
        let value = json!({
            "pleural_fluid_protein": 2.0, "serum_protein": 7.0, "pleural_fluid_ldh_iu_l": 100.0,
            "serum_ldh_iu_l": 200.0, "serum_ldh_upper_limit_of_normal_iu_l": 250.0
        });
        let dynamic = LightsCriteria.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(false));
    }
}
