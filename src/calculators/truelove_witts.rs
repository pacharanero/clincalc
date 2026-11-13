// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Truelove and Witts' Severity Index - classic mild/moderate/severe
//! classification of an ulcerative colitis flare (Truelove, Witts. Br Med J
//! 1955).
//!
//! Severe disease requires 6 or more bloody stools per day PLUS at least one
//! systemic-toxicity feature (pyrexia, tachycardia, anaemia, or a raised
//! ESR); frequency alone, without a toxicity feature, does not meet the
//! severe threshold. The qualitative "amount of visible blood" criterion
//! from the original table is not separately scored here: it is described
//! as a continuum ("between mild and severe") rather than a discrete,
//! actionable threshold, and does not change the mild/moderate/severe
//! boundary on its own in any widely used operationalisation of this index.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "truelove_witts";

pub const REFERENCE: &str = "Truelove SC, Witts LJ. Cortisone in ulcerative colitis; final report on a therapeutic trial. \
Br Med J. 1955;2(4947):1041-1048.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.1136/bmj.2.4947.1041",
);

const SEVERE_STOOL_THRESHOLD: u32 = 6;
const MODERATE_STOOL_THRESHOLD: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrueloveWittsInput {
    pub bloody_stools_per_day: u32,
    pub pyrexia_over_37_8c: bool,
    pub pulse_over_90_bpm: bool,
    pub anaemia: bool,
    pub esr_over_30_mm_hr: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColitisSeverity {
    Mild,
    Moderate,
    Severe,
}

impl ColitisSeverity {
    fn slug(self) -> &'static str {
        match self {
            Self::Mild => "mild",
            Self::Moderate => "moderate",
            Self::Severe => "severe",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrueloveWittsOutcome {
    pub systemic_toxicity_feature_present: bool,
    pub severity: ColitisSeverity,
}

pub fn compute(input: &TrueloveWittsInput) -> TrueloveWittsOutcome {
    let systemic_toxicity_feature_present = input.pyrexia_over_37_8c
        || input.pulse_over_90_bpm
        || input.anaemia
        || input.esr_over_30_mm_hr;

    let severity = if input.bloody_stools_per_day >= SEVERE_STOOL_THRESHOLD
        && systemic_toxicity_feature_present
    {
        ColitisSeverity::Severe
    } else if input.bloody_stools_per_day >= MODERATE_STOOL_THRESHOLD {
        ColitisSeverity::Moderate
    } else {
        ColitisSeverity::Mild
    };

    TrueloveWittsOutcome {
        systemic_toxicity_feature_present,
        severity,
    }
}

pub fn build_response(input: &TrueloveWittsInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input);
    let mut working = Map::new();
    working.insert(
        "bloody_stools_per_day".into(),
        json!(input.bloody_stools_per_day),
    );
    working.insert(
        "systemic_toxicity_feature_present".into(),
        json!(o.systemic_toxicity_feature_present),
    );
    working.insert("severity".into(), json!(o.severity.slug()));

    let interpretation = match o.severity {
        ColitisSeverity::Severe => "Severe: 6 or more bloody stools per day with at least one systemic-toxicity feature (pyrexia >37.8C, pulse >90 bpm, anaemia, or ESR >30 mm/hr). This is a medical emergency warranting admission for intensive medical (and potentially surgical) management.".to_string(),
        ColitisSeverity::Moderate => "Moderate: 4 or more bloody stools per day without meeting the severe threshold (which needs both >=6 stools and a systemic-toxicity feature).".to_string(),
        ColitisSeverity::Mild => "Mild: fewer than 4 bloody stools per day.".to_string(),
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.severity.slug()),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct TrueloveWitts;
impl Calculator for TrueloveWitts {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Truelove and Witts' Severity Index"
    }
    fn description(&self) -> &'static str {
        "Classic mild/moderate/severe classification of an ulcerative colitis flare from stool frequency and systemic-toxicity features."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "TrueloveWittsInput", "type": "object",
            "additionalProperties": false,
            "required": ["bloody_stools_per_day", "pyrexia_over_37_8c", "pulse_over_90_bpm", "anaemia", "esr_over_30_mm_hr"],
            "properties": {
                "bloody_stools_per_day": { "type": "integer", "minimum": 0, "maximum": 50, "description": "Number of bloody stools per day" },
                "pyrexia_over_37_8c": { "type": "boolean", "description": "Temperature over 37.8C" },
                "pulse_over_90_bpm": { "type": "boolean", "description": "Pulse over 90 bpm" },
                "anaemia": { "type": "boolean", "description": "Anaemia present" },
                "esr_over_30_mm_hr": { "type": "boolean", "description": "Erythrocyte sedimentation rate over 30 mm/hr" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: TrueloveWittsInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        stools: u32,
        pyrexia: bool,
        pulse: bool,
        anaemia: bool,
        esr: bool,
    ) -> TrueloveWittsInput {
        TrueloveWittsInput {
            bloody_stools_per_day: stools,
            pyrexia_over_37_8c: pyrexia,
            pulse_over_90_bpm: pulse,
            anaemia,
            esr_over_30_mm_hr: esr,
        }
    }

    #[test]
    fn under_four_stools_is_mild() {
        let o = compute(&input(3, false, false, false, false));
        assert_eq!(o.severity, ColitisSeverity::Mild);
    }

    #[test]
    fn four_to_five_stools_is_moderate_regardless_of_toxicity() {
        let o = compute(&input(5, true, false, false, false));
        assert_eq!(o.severity, ColitisSeverity::Moderate);
    }

    #[test]
    fn six_or_more_stools_without_toxicity_is_moderate_not_severe() {
        let o = compute(&input(8, false, false, false, false));
        assert_eq!(o.severity, ColitisSeverity::Moderate);
        assert!(!o.systemic_toxicity_feature_present);
    }

    #[test]
    fn six_or_more_stools_with_any_toxicity_feature_is_severe() {
        for input_combo in [
            input(6, true, false, false, false),
            input(6, false, true, false, false),
            input(6, false, false, true, false),
            input(6, false, false, false, true),
        ] {
            assert_eq!(compute(&input_combo).severity, ColitisSeverity::Severe);
        }
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let typed = input(7, true, true, false, false);
        let value = json!({
            "bloody_stools_per_day": 7, "pyrexia_over_37_8c": true, "pulse_over_90_bpm": true,
            "anaemia": false, "esr_over_30_mm_hr": false
        });
        let dynamic = TrueloveWitts.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!("severe"));
    }

    #[test]
    fn rejects_missing_field() {
        assert!(
            TrueloveWitts
                .calculate(&json!({"bloody_stools_per_day": 5}))
                .is_err()
        );
    }
}
