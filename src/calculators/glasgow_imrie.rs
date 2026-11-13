// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Glasgow-Imrie Criteria - 8-parameter severity score for acute
//! pancreatitis (Blamey et al. Gut 1984), the UK-preferred alternative to
//! Ranson's criteria, scored once at 48 hours.
//!
//! As with [`crate::calculators::ranson`], each field is the pre-evaluated
//! boolean result of comparing a peak/trough 48-hour value against its fixed
//! threshold, since that is how the criteria are specified and applied.
//! Lactate dehydrogenase and AST/ALT are treated as a single "enzymes"
//! criterion worth one point if either threshold is met, matching the
//! standard PANCREAS-mnemonic presentation of this score.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "glasgow_imrie";

pub const REFERENCE: &str = "Blamey SL, Imrie CW, O'Neill J, Gilmour WH, Carter DC. Prognostic factors in acute \
pancreatitis. Gut. 1984;25(12):1340-1346.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.1136/gut.25.12.1340",
);

const SEVERE_THRESHOLD: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlasgowImrieInput {
    pub age_over_55_years: bool,
    pub pao2_under_8kpa: bool,
    pub wbc_over_15_x10_9_per_l: bool,
    pub calcium_under_2_mmol_l: bool,
    /// Serum urea over 16 mmol/L, with no response to intravenous fluids.
    pub urea_over_16_mmol_l: bool,
    pub ldh_over_600_iu_l: bool,
    pub ast_over_200_iu_l: bool,
    pub albumin_under_32_g_l: bool,
    pub glucose_over_10_mmol_l: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlasgowImrieOutcome {
    pub score: u8,
    pub enzymes_criterion: bool,
    pub predicts_severe_disease: bool,
}

pub fn compute(input: &GlasgowImrieInput) -> GlasgowImrieOutcome {
    let enzymes_criterion = input.ldh_over_600_iu_l || input.ast_over_200_iu_l;
    let score = u8::from(input.age_over_55_years)
        + u8::from(input.pao2_under_8kpa)
        + u8::from(input.wbc_over_15_x10_9_per_l)
        + u8::from(input.calcium_under_2_mmol_l)
        + u8::from(input.urea_over_16_mmol_l)
        + u8::from(enzymes_criterion)
        + u8::from(input.albumin_under_32_g_l)
        + u8::from(input.glucose_over_10_mmol_l);
    GlasgowImrieOutcome {
        score,
        enzymes_criterion,
        predicts_severe_disease: score >= SEVERE_THRESHOLD,
    }
}

pub fn build_response(input: &GlasgowImrieInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input);
    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("enzymes_criterion".into(), json!(o.enzymes_criterion));
    working.insert(
        "predicts_severe_disease".into(),
        json!(o.predicts_severe_disease),
    );

    let interpretation = if o.predicts_severe_disease {
        format!(
            "Glasgow-Imrie score {score}/8 (>= {SEVERE_THRESHOLD} at 48 hours): predicts severe pancreatitis. Criteria must be assessed with peak/trough values taken at 48 hours after admission.",
            score = o.score
        )
    } else {
        format!(
            "Glasgow-Imrie score {score}/8 (< {SEVERE_THRESHOLD} at 48 hours): does not predict severe disease by this cutoff. Criteria must be assessed with peak/trough values taken at 48 hours after admission.",
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

pub struct GlasgowImrie;
impl Calculator for GlasgowImrie {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Glasgow-Imrie Criteria (Acute Pancreatitis)"
    }
    fn description(&self) -> &'static str {
        "8-parameter UK-preferred severity score for acute pancreatitis, scored once at 48 hours."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "GlasgowImrieInput", "type": "object", "additionalProperties": false,
            "required": [
                "age_over_55_years", "pao2_under_8kpa", "wbc_over_15_x10_9_per_l", "calcium_under_2_mmol_l",
                "urea_over_16_mmol_l", "ldh_over_600_iu_l", "ast_over_200_iu_l", "albumin_under_32_g_l",
                "glucose_over_10_mmol_l"
            ],
            "properties": {
                "age_over_55_years": { "type": "boolean", "description": "At 48 hours: age over 55 years" },
                "pao2_under_8kpa": { "type": "boolean", "description": "At 48 hours: arterial PaO2 under 8 kPa (< 60 mmHg)" },
                "wbc_over_15_x10_9_per_l": { "type": "boolean", "description": "At 48 hours: white cell count over 15 x 10^9/L" },
                "calcium_under_2_mmol_l": { "type": "boolean", "description": "At 48 hours: serum calcium under 2 mmol/L (< 8 mg/dL)" },
                "urea_over_16_mmol_l": { "type": "boolean", "description": "At 48 hours: serum urea over 16 mmol/L, with no response to IV fluids" },
                "ldh_over_600_iu_l": { "type": "boolean", "description": "At 48 hours: serum LDH over 600 IU/L (combines with ast_over_200_iu_l into a single 'enzymes' point)" },
                "ast_over_200_iu_l": { "type": "boolean", "description": "At 48 hours: serum AST over 200 IU/L (combines with ldh_over_600_iu_l into a single 'enzymes' point)" },
                "albumin_under_32_g_l": { "type": "boolean", "description": "At 48 hours: serum albumin under 32 g/L (< 3.2 g/dL)" },
                "glucose_over_10_mmol_l": { "type": "boolean", "description": "At 48 hours: blood glucose over 10 mmol/L (> 180 mg/dL)" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: GlasgowImrieInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> GlasgowImrieInput {
        GlasgowImrieInput {
            age_over_55_years: false,
            pao2_under_8kpa: false,
            wbc_over_15_x10_9_per_l: false,
            calcium_under_2_mmol_l: false,
            urea_over_16_mmol_l: false,
            ldh_over_600_iu_l: false,
            ast_over_200_iu_l: false,
            albumin_under_32_g_l: false,
            glucose_over_10_mmol_l: false,
        }
    }

    #[test]
    fn all_absent_scores_zero() {
        let o = compute(&none());
        assert_eq!(o.score, 0);
        assert!(!o.predicts_severe_disease);
    }

    #[test]
    fn enzymes_criterion_scores_only_one_point_when_both_present() {
        let mut i = none();
        i.ldh_over_600_iu_l = true;
        i.ast_over_200_iu_l = true;
        let o = compute(&i);
        assert_eq!(o.score, 1);
        assert!(o.enzymes_criterion);
    }

    #[test]
    fn three_criteria_predicts_severe() {
        let mut i = none();
        i.age_over_55_years = true;
        i.pao2_under_8kpa = true;
        i.wbc_over_15_x10_9_per_l = true;
        let o = compute(&i);
        assert_eq!(o.score, 3);
        assert!(o.predicts_severe_disease);
    }

    #[test]
    fn all_present_scores_eight() {
        let i = GlasgowImrieInput {
            age_over_55_years: true,
            pao2_under_8kpa: true,
            wbc_over_15_x10_9_per_l: true,
            calcium_under_2_mmol_l: true,
            urea_over_16_mmol_l: true,
            ldh_over_600_iu_l: true,
            ast_over_200_iu_l: true,
            albumin_under_32_g_l: true,
            glucose_over_10_mmol_l: true,
        };
        assert_eq!(compute(&i).score, 8);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let mut typed = none();
        typed.calcium_under_2_mmol_l = true;
        typed.albumin_under_32_g_l = true;
        let value = json!({
            "age_over_55_years": false, "pao2_under_8kpa": false, "wbc_over_15_x10_9_per_l": false,
            "calcium_under_2_mmol_l": true, "urea_over_16_mmol_l": false, "ldh_over_600_iu_l": false,
            "ast_over_200_iu_l": false, "albumin_under_32_g_l": true, "glucose_over_10_mmol_l": false
        });
        let dynamic = GlasgowImrie.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(2));
    }

    #[test]
    fn rejects_missing_field() {
        assert!(
            GlasgowImrie
                .calculate(&json!({"age_over_55_years": true}))
                .is_err()
        );
    }
}
