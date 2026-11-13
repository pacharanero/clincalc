// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Ranson's Criteria - 11-parameter severity/mortality score for acute
//! pancreatitis (Ranson et al. Surg Gynecol Obstet 1974).
//!
//! This is the original 11-item criteria set derived from a predominantly
//! alcohol-related cohort; a separate 10-item set exists for gallstone
//! pancreatitis with different thresholds and is not implemented here. Five
//! criteria are assessed on admission and six at 48 hours; each caller
//! supplies the pre-evaluated boolean result of each criterion (the
//! thresholds are fixed and documented on each field) rather than raw
//! values, since the original study and its criteria are presented this way
//! and several thresholds (e.g. "BUN increase" or "fluid sequestration") are
//! deltas or estimates rather than single lab values.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "ranson";

pub const REFERENCE: &str = "Ranson JH, Rifkind KM, Roses DF, Fink SD, Eng K, Localio SA. Prognostic signs and the \
role of operative management in acute pancreatitis. Surg Gynecol Obstet. 1974;139(1):69-81.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://pubmed.ncbi.nlm.nih.gov/4834279/",
);

/// Severe-disease cutoff (inclusive): score 3 or more predicts a severe
/// attack in the original derivation and validation.
const SEVERE_THRESHOLD: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RansonInput {
    // On admission.
    pub age_over_55_years: bool,
    pub wbc_over_16000_per_mm3: bool,
    pub glucose_over_200_mg_dl: bool,
    pub ast_over_250_iu_l: bool,
    pub ldh_over_350_iu_l: bool,
    // Within 48 hours.
    pub calcium_under_8_mg_dl: bool,
    pub hematocrit_fall_over_10_percent: bool,
    pub pao2_under_60_mmhg: bool,
    /// Blood urea nitrogen rise of 5 mg/dL or more despite intravenous
    /// fluid hydration.
    pub bun_rise_5mg_dl_or_more_despite_iv_fluids: bool,
    pub base_deficit_over_4_meq_l: bool,
    pub fluid_sequestration_over_6l: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RansonOutcome {
    pub score: u8,
    pub predicts_severe_disease: bool,
}

pub fn compute(input: &RansonInput) -> RansonOutcome {
    let score = u8::from(input.age_over_55_years)
        + u8::from(input.wbc_over_16000_per_mm3)
        + u8::from(input.glucose_over_200_mg_dl)
        + u8::from(input.ast_over_250_iu_l)
        + u8::from(input.ldh_over_350_iu_l)
        + u8::from(input.calcium_under_8_mg_dl)
        + u8::from(input.hematocrit_fall_over_10_percent)
        + u8::from(input.pao2_under_60_mmhg)
        + u8::from(input.bun_rise_5mg_dl_or_more_despite_iv_fluids)
        + u8::from(input.base_deficit_over_4_meq_l)
        + u8::from(input.fluid_sequestration_over_6l);
    RansonOutcome {
        score,
        predicts_severe_disease: score >= SEVERE_THRESHOLD,
    }
}

pub fn build_response(input: &RansonInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input);
    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert(
        "predicts_severe_disease".into(),
        json!(o.predicts_severe_disease),
    );

    let interpretation = if o.predicts_severe_disease {
        format!(
            "Ranson's score {score}/11 (>= {SEVERE_THRESHOLD}): predicts a severe attack in the original derivation and validation cohorts. The score is complete only once both the admission criteria and the 48-hour criteria have been assessed; mortality rises with increasing score.",
            score = o.score
        )
    } else {
        format!(
            "Ranson's score {score}/11 (< {SEVERE_THRESHOLD}): does not predict severe disease by the original cutoff. The score is complete only once both the admission criteria and the 48-hour criteria have been assessed.",
            score = o.score
        )
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Ranson;
impl Calculator for Ranson {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Ranson's Criteria (Acute Pancreatitis)"
    }
    fn description(&self) -> &'static str {
        "11-parameter severity/mortality score for acute pancreatitis, scored at admission (5 criteria) and 48 hours (6 criteria)."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "RansonInput", "type": "object", "additionalProperties": false,
            "required": [
                "age_over_55_years", "wbc_over_16000_per_mm3", "glucose_over_200_mg_dl", "ast_over_250_iu_l",
                "ldh_over_350_iu_l", "calcium_under_8_mg_dl", "hematocrit_fall_over_10_percent",
                "pao2_under_60_mmhg", "bun_rise_5mg_dl_or_more_despite_iv_fluids", "base_deficit_over_4_meq_l",
                "fluid_sequestration_over_6l"
            ],
            "properties": {
                "age_over_55_years": { "type": "boolean", "description": "On admission: age over 55 years" },
                "wbc_over_16000_per_mm3": { "type": "boolean", "description": "On admission: white cell count over 16,000/mm3" },
                "glucose_over_200_mg_dl": { "type": "boolean", "description": "On admission: blood glucose over 200 mg/dL (> 11.1 mmol/L)" },
                "ast_over_250_iu_l": { "type": "boolean", "description": "On admission: serum AST over 250 IU/L" },
                "ldh_over_350_iu_l": { "type": "boolean", "description": "On admission: serum LDH over 350 IU/L" },
                "calcium_under_8_mg_dl": { "type": "boolean", "description": "At 48 hours: serum calcium under 8 mg/dL (< 2.0 mmol/L)" },
                "hematocrit_fall_over_10_percent": { "type": "boolean", "description": "At 48 hours: haematocrit fall of more than 10 percentage points from admission" },
                "pao2_under_60_mmhg": { "type": "boolean", "description": "At 48 hours: arterial PaO2 under 60 mmHg (< 8 kPa)" },
                "bun_rise_5mg_dl_or_more_despite_iv_fluids": { "type": "boolean", "description": "At 48 hours: blood urea nitrogen risen by 5 mg/dL or more (>= 1.8 mmol/L urea) despite IV fluid hydration" },
                "base_deficit_over_4_meq_l": { "type": "boolean", "description": "At 48 hours: base deficit over 4 mEq/L" },
                "fluid_sequestration_over_6l": { "type": "boolean", "description": "At 48 hours: estimated fluid sequestration over 6 litres" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: RansonInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> RansonInput {
        RansonInput {
            age_over_55_years: false,
            wbc_over_16000_per_mm3: false,
            glucose_over_200_mg_dl: false,
            ast_over_250_iu_l: false,
            ldh_over_350_iu_l: false,
            calcium_under_8_mg_dl: false,
            hematocrit_fall_over_10_percent: false,
            pao2_under_60_mmhg: false,
            bun_rise_5mg_dl_or_more_despite_iv_fluids: false,
            base_deficit_over_4_meq_l: false,
            fluid_sequestration_over_6l: false,
        }
    }

    #[test]
    fn all_absent_scores_zero() {
        let o = compute(&none());
        assert_eq!(o.score, 0);
        assert!(!o.predicts_severe_disease);
    }

    #[test]
    fn two_criteria_does_not_predict_severe() {
        let mut i = none();
        i.age_over_55_years = true;
        i.wbc_over_16000_per_mm3 = true;
        assert!(!compute(&i).predicts_severe_disease);
    }

    #[test]
    fn three_criteria_predicts_severe() {
        let mut i = none();
        i.age_over_55_years = true;
        i.wbc_over_16000_per_mm3 = true;
        i.glucose_over_200_mg_dl = true;
        let o = compute(&i);
        assert_eq!(o.score, 3);
        assert!(o.predicts_severe_disease);
    }

    #[test]
    fn all_present_scores_eleven() {
        let i = RansonInput {
            age_over_55_years: true,
            wbc_over_16000_per_mm3: true,
            glucose_over_200_mg_dl: true,
            ast_over_250_iu_l: true,
            ldh_over_350_iu_l: true,
            calcium_under_8_mg_dl: true,
            hematocrit_fall_over_10_percent: true,
            pao2_under_60_mmhg: true,
            bun_rise_5mg_dl_or_more_despite_iv_fluids: true,
            base_deficit_over_4_meq_l: true,
            fluid_sequestration_over_6l: true,
        };
        assert_eq!(compute(&i).score, 11);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let mut typed = none();
        typed.ldh_over_350_iu_l = true;
        typed.pao2_under_60_mmhg = true;
        typed.base_deficit_over_4_meq_l = true;
        let value = json!({
            "age_over_55_years": false, "wbc_over_16000_per_mm3": false, "glucose_over_200_mg_dl": false,
            "ast_over_250_iu_l": false, "ldh_over_350_iu_l": true, "calcium_under_8_mg_dl": false,
            "hematocrit_fall_over_10_percent": false, "pao2_under_60_mmhg": true,
            "bun_rise_5mg_dl_or_more_despite_iv_fluids": false, "base_deficit_over_4_meq_l": true,
            "fluid_sequestration_over_6l": false
        });
        let dynamic = Ranson.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(3));
    }

    #[test]
    fn rejects_missing_field() {
        assert!(
            Ranson
                .calculate(&json!({"age_over_55_years": true}))
                .is_err()
        );
    }
}
