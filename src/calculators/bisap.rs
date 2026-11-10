// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! BISAP - Bedside Index for Severity in Acute Pancreatitis.
//!
//! A five-item score (Wu et al. Gut 2008) for early prediction of in-hospital
//! mortality in acute pancreatitis, usable within the first 24 hours. One point
//! each for:
//!
//! - **B**lood urea nitrogen > 25 mg/dL
//! - **I**mpaired mental status
//! - **S**IRS (two or more SIRS criteria)
//! - **A**ge > 60 years
//! - **P**leural effusion
//!
//! The primary abstract states that mortality ranged from under 1% in the
//! lowest-risk group to over 20% in the highest. The per-score mortality
//! table sits in the (non-open-access) full text and has not been verified
//! against the primary source, so this calculator deliberately reports the
//! score and that verified range only, and does not assert per-score
//! percentages or a treatment threshold.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "bisap";

/// Primary citation.
pub const REFERENCE: &str = "Wu BU, Johannes RS, Sun X, Tabak Y, Conwell DL, Banks PA. The early prediction of mortality in \
acute pancreatitis: a large population-based study. Gut. 2008;57(12):1698-1703.";

/// Distribution licence: a published clinical scoring method, implemented from
/// the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature",
    "https://doi.org/10.1136/gut.2008.152702",
);

/// Blood urea nitrogen threshold (exclusive), mg/dL.
const BUN_THRESHOLD_MG_DL: f64 = 25.0;
/// Age threshold (exclusive), years.
const AGE_THRESHOLD_YEARS: f64 = 60.0;
/// Minimum SIRS criteria (inclusive) for the S point.
const SIRS_MIN_CRITERIA: u8 = 2;

/// BISAP inputs, assessed within the first 24 hours.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BisapInput {
    /// Blood urea nitrogen in mg/dL. Scores a point when > 25.
    pub bun_mg_dl: f64,
    /// Impaired mental status (disorientation, lethargy, somnolence, coma or stupor).
    pub impaired_mental_status: bool,
    /// Number of SIRS criteria met (0-4). Scores a point when >= 2.
    pub sirs_criteria_count: u8,
    /// Age in years. Scores a point when > 60.
    pub age_years: f64,
    /// Pleural effusion on imaging.
    pub pleural_effusion: bool,
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BisapOutcome {
    /// Total score (0-5).
    pub score: u8,
    pub bun_criterion: bool,
    pub mental_status_criterion: bool,
    pub sirs_criterion: bool,
    pub age_criterion: bool,
    pub pleural_effusion_criterion: bool,
    pub interpretation: String,
}

/// Validate inputs.
pub fn compute(input: &BisapInput) -> Result<BisapInput, CalcError> {
    if !input.bun_mg_dl.is_finite() || input.bun_mg_dl < 0.0 {
        return Err(CalcError::InvalidInput(
            "bun_mg_dl must be a non-negative number".into(),
        ));
    }
    if !input.age_years.is_finite() || input.age_years < 0.0 {
        return Err(CalcError::InvalidInput(
            "age_years must be a non-negative number".into(),
        ));
    }
    if input.sirs_criteria_count > 4 {
        return Err(CalcError::InvalidInput(
            "sirs_criteria_count must be between 0 and 4".into(),
        ));
    }
    Ok(*input)
}

fn outcome(input: &BisapInput) -> BisapOutcome {
    let bun_criterion = input.bun_mg_dl > BUN_THRESHOLD_MG_DL;
    let mental_status_criterion = input.impaired_mental_status;
    let sirs_criterion = input.sirs_criteria_count >= SIRS_MIN_CRITERIA;
    let age_criterion = input.age_years > AGE_THRESHOLD_YEARS;
    let pleural_effusion_criterion = input.pleural_effusion;

    let score = u8::from(bun_criterion)
        + u8::from(mental_status_criterion)
        + u8::from(sirs_criterion)
        + u8::from(age_criterion)
        + u8::from(pleural_effusion_criterion);

    let interpretation = format!(
        "BISAP {score}/5. In the derivation study (Wu 2008) in-hospital mortality ranged from under \
1% in the lowest-risk group to over 20% in the highest-risk group; a higher score means higher \
risk. BISAP is an early (first 24 hours) prognostic aid for acute pancreatitis and does not \
replace clinical judgement or assessment for organ failure."
    );

    BisapOutcome {
        score,
        bun_criterion,
        mental_status_criterion,
        sirs_criterion,
        age_criterion,
        pleural_effusion_criterion,
        interpretation,
    }
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &BisapInput) -> Result<CalculationResponse, CalcError> {
    let validated = compute(input)?;
    let o = outcome(&validated);

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("bun_mg_dl".into(), json!(input.bun_mg_dl));
    working.insert("bun_criterion".into(), json!(o.bun_criterion));
    working.insert(
        "impaired_mental_status_criterion".into(),
        json!(o.mental_status_criterion),
    );
    working.insert(
        "sirs_criteria_count".into(),
        json!(input.sirs_criteria_count),
    );
    working.insert("sirs_criterion".into(), json!(o.sirs_criterion));
    working.insert("age_years".into(), json!(input.age_years));
    working.insert("age_criterion".into(), json!(o.age_criterion));
    working.insert(
        "pleural_effusion_criterion".into(),
        json!(o.pleural_effusion_criterion),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Bisap;

impl Calculator for Bisap {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "BISAP Score (acute pancreatitis)"
    }

    fn description(&self) -> &'static str {
        "Five-item (0-5) early, first-24-hour mortality-risk score for acute pancreatitis: BUN, impaired mental status, SIRS, age and pleural effusion."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        let source = json!({ "citation": "Wu BU et al. Gut. 2008;57(12):1698-1703.", "url": "https://doi.org/10.1136/gut.2008.152702" });
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "BisapInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["bun_mg_dl", "impaired_mental_status", "sirs_criteria_count", "age_years", "pleural_effusion"],
            "properties": {
                "bun_mg_dl": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 500,
                    "description": "Blood urea nitrogen in mg/dL, first 24 hours (> 25 scores 1 point). Urea in mmol/L x 2.8 approximates BUN in mg/dL.",
                    "definition": {
                        "concept": "Blood urea nitrogen criterion",
                        "statement": "A blood urea nitrogen above 25 mg/dL during the first 24 hours scores 1 point.",
                        "caveats": "BUN (mg/dL) is not the same number as UK urea (mmol/L); convert before entry.",
                        "source": source,
                        "status": "draft"
                    }
                },
                "impaired_mental_status": {
                    "type": "boolean",
                    "description": "Impaired mental status in the first 24 hours (scores 1 point)",
                    "definition": {
                        "concept": "Impaired mental status criterion",
                        "statement": "Impaired mental status (e.g. disorientation, lethargy, somnolence, coma or stupor) scores 1 point.",
                        "caveats": "The criterion targets acute change attributable to the illness.",
                        "source": source,
                        "status": "draft"
                    }
                },
                "sirs_criteria_count": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 4,
                    "description": "Number of SIRS criteria met (temperature, heart rate, respiratory rate/PaCO2, white cell count); 2 or more scores 1 point",
                    "definition": {
                        "concept": "SIRS criterion",
                        "statement": "Two or more systemic inflammatory response syndrome criteria scores 1 point.",
                        "caveats": "The count is asserted by the clinician using the standard four SIRS criteria.",
                        "source": source,
                        "status": "draft"
                    }
                },
                "age_years": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 130,
                    "description": "Age in years (> 60 scores 1 point)"
                },
                "pleural_effusion": {
                    "type": "boolean",
                    "description": "Pleural effusion present on imaging (scores 1 point)",
                    "definition": {
                        "concept": "Pleural effusion criterion",
                        "statement": "Presence of a pleural effusion on imaging scores 1 point.",
                        "source": source,
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: BisapInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> BisapInput {
        BisapInput {
            bun_mg_dl: 15.0,
            impaired_mental_status: false,
            sirs_criteria_count: 0,
            age_years: 45.0,
            pleural_effusion: false,
        }
    }

    fn score_of(i: &BisapInput) -> u8 {
        outcome(&compute(i).unwrap()).score
    }

    #[test]
    fn all_absent_scores_zero() {
        assert_eq!(score_of(&base()), 0);
    }

    #[test]
    fn bun_threshold_is_exclusive() {
        let mut i = base();
        i.bun_mg_dl = 25.0;
        assert_eq!(score_of(&i), 0);
        i.bun_mg_dl = 25.1;
        assert_eq!(score_of(&i), 1);
    }

    #[test]
    fn age_threshold_is_exclusive() {
        let mut i = base();
        i.age_years = 60.0;
        assert_eq!(score_of(&i), 0);
        i.age_years = 61.0;
        assert_eq!(score_of(&i), 1);
    }

    #[test]
    fn sirs_needs_two_criteria() {
        let mut i = base();
        i.sirs_criteria_count = 1;
        assert_eq!(score_of(&i), 0);
        i.sirs_criteria_count = 2;
        assert_eq!(score_of(&i), 1);
        i.sirs_criteria_count = 4;
        assert_eq!(score_of(&i), 1);
    }

    #[test]
    fn mental_status_and_effusion_score_one_each() {
        let mut i = base();
        i.impaired_mental_status = true;
        assert_eq!(score_of(&i), 1);
        i.pleural_effusion = true;
        assert_eq!(score_of(&i), 2);
    }

    #[test]
    fn maximum_score_is_five() {
        let i = BisapInput {
            bun_mg_dl: 40.0,
            impaired_mental_status: true,
            sirs_criteria_count: 3,
            age_years: 78.0,
            pleural_effusion: true,
        };
        assert_eq!(score_of(&i), 5);
    }

    #[test]
    fn rejects_invalid_inputs() {
        let mut i = base();
        i.bun_mg_dl = f64::NAN;
        assert!(compute(&i).is_err());
        let mut i = base();
        i.age_years = -1.0;
        assert!(compute(&i).is_err());
        let mut i = base();
        i.sirs_criteria_count = 5;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn build_response_carries_criteria() {
        let i = BisapInput {
            bun_mg_dl: 30.0,
            impaired_mental_status: false,
            sirs_criteria_count: 2,
            age_years: 70.0,
            pleural_effusion: false,
        };
        let r = build_response(&i).unwrap();
        assert_eq!(r.calculator, "bisap");
        assert_eq!(r.result, json!(3));
        assert_eq!(r.working["bun_criterion"], json!(true));
        assert_eq!(r.working["pleural_effusion_criterion"], json!(false));
        assert!(r.reference.contains("Wu BU"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "bun_mg_dl": 30.0,
            "impaired_mental_status": true,
            "sirs_criteria_count": 1,
            "age_years": 50.0,
            "pleural_effusion": true
        });
        let typed = BisapInput {
            bun_mg_dl: 30.0,
            impaired_mental_status: true,
            sirs_criteria_count: 1,
            age_years: 50.0,
            pleural_effusion: true,
        };
        let dynamic = Bisap.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(3));
    }

    #[test]
    fn dynamic_calculate_rejects_garbage() {
        assert!(Bisap.calculate(&json!({ "bun_mg_dl": "high" })).is_err());
        assert!(Bisap.calculate(&json!({})).is_err());
    }
}
