// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! PERC - Pulmonary Embolism Rule-out Criteria.
//!
//! An 8-item rule-out checklist for suspected pulmonary embolism, derived by
//! Kline et al. (J Thromb Haemost. 2004;2(8):1247-1255) and validated in a
//! prospective multicentre cohort (Kline et al. J Thromb Haemost.
//! 2008;6(5):772-780).
//!
//! PERC is only interpretable in patients whom the assessing clinician has
//! **already judged to be low pretest probability** for PE (typically by
//! gestalt, or a low Wells score - see [`crate::calculators::wells_pe`]).
//! Within that population, if all 8 criteria are absent the patient is
//! "PERC-negative" and the risk of PE is low enough (under 2%, per both Kline
//! papers) that no further testing is needed. If any criterion is present the
//! patient is "PERC-positive" and PE cannot be excluded on this rule alone -
//! further workup (e.g. D-dimer per Wells) is indicated. PERC does not itself
//! compute or require a pretest-probability score; it assumes the clinician
//! has already established a low pretest probability before applying it, and
//! is not valid in moderate- or high-probability patients.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "perc";

/// Primary citation.
pub const REFERENCE: &str = "Kline JA, Mitchell AM, Kabrhel C, Richman PB, Courtney DM. Clinical criteria to prevent \
unnecessary diagnostic testing in emergency department patients with suspected pulmonary embolism. \
J Thromb Haemost. 2004;2(8):1247-1255. Validated in Kline JA et al. J Thromb Haemost. \
2008;6(5):772-780.";

/// Distribution licence: the rule is a published clinical method from the
/// primary literature, implemented here from that source.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1111/j.1538-7836.2004.00790.x",
};

/// PERC inputs: eight clinician-asserted boolean criteria. Each `true` marks
/// the criterion as present (i.e. the abnormal finding is there).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PercInput {
    /// Age 50 years or over.
    pub age_50_or_over: bool,
    /// Heart rate 100 beats per minute or over.
    pub heart_rate_100_or_over: bool,
    /// Oxygen saturation on room air below 95%.
    pub spo2_below_95: bool,
    /// Unilateral leg swelling.
    pub unilateral_leg_swelling: bool,
    /// Haemoptysis.
    pub haemoptysis: bool,
    /// Recent surgery or trauma: within the prior 4 weeks, requiring
    /// hospitalisation or general anaesthesia.
    pub recent_surgery_or_trauma: bool,
    /// Prior venous thromboembolism (DVT or PE).
    pub prior_vte: bool,
    /// Hormone use: oral contraceptives, hormone replacement, or
    /// oestrogen-based hormone therapy.
    pub hormone_use: bool,
}

/// Which of the 8 criteria were positive, in a stable order.
const CRITERION_NAMES: [&str; 8] = [
    "age_50_or_over",
    "heart_rate_100_or_over",
    "spo2_below_95",
    "unilateral_leg_swelling",
    "haemoptysis",
    "recent_surgery_or_trauma",
    "prior_vte",
    "hormone_use",
];

/// Overall PERC result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PercResult {
    /// All 8 criteria absent: PE can be excluded without further testing, in
    /// a patient already judged low pretest probability.
    Negative,
    /// At least one criterion present: PE cannot be excluded on PERC alone.
    Positive,
}

impl PercResult {
    /// Stable slug.
    pub fn slug(self) -> &'static str {
        match self {
            PercResult::Negative => "perc-negative",
            PercResult::Positive => "perc-positive",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PercOutcome {
    /// Number of criteria present (0-8).
    pub positive_count: u8,
    /// Machine names of the criteria that were present, in a stable order.
    pub positive_criteria: Vec<&'static str>,
    pub result: PercResult,
    pub interpretation: String,
}

/// Pure scoring.
pub fn compute(input: &PercInput) -> Result<PercOutcome, CalcError> {
    let flags = [
        input.age_50_or_over,
        input.heart_rate_100_or_over,
        input.spo2_below_95,
        input.unilateral_leg_swelling,
        input.haemoptysis,
        input.recent_surgery_or_trauma,
        input.prior_vte,
        input.hormone_use,
    ];

    let positive_criteria: Vec<&'static str> = CRITERION_NAMES
        .iter()
        .zip(flags.iter())
        .filter(|(_, present)| **present)
        .map(|(name, _)| *name)
        .collect();

    let positive_count = positive_criteria.len() as u8;

    let result = if positive_count == 0 {
        PercResult::Negative
    } else {
        PercResult::Positive
    };

    let interpretation = match result {
        PercResult::Negative => {
            "PERC-negative (0 of 8 criteria present). In a patient whom the clinician has \
already judged to be low pretest probability for PE, this identifies a risk of PE low enough \
(under 2%, Kline 2004 and the 2008 multicentre validation) that no further testing is needed. \
PERC is not valid in patients with moderate or high pretest probability - it must only be applied \
after that clinical judgement (e.g. a low Wells PE score) has already been made."
                .to_string()
        }
        PercResult::Positive => format!(
            "PERC-positive ({positive_count} of 8 criteria present). PE cannot be excluded on PERC \
alone; proceed to further workup (e.g. D-dimer, guided by Wells PE). PERC is only interpretable \
in patients whom the clinician has already judged to be low pretest probability for PE - it is \
not valid in moderate- or high-probability patients."
        ),
    };

    Ok(PercOutcome {
        positive_count,
        positive_criteria,
        result,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &PercInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("positive_count".into(), json!(o.positive_count));
    working.insert("result".into(), json!(o.result.slug()));
    working.insert("positive_criteria".into(), json!(o.positive_criteria));
    working.insert("age_50_or_over".into(), json!(input.age_50_or_over));
    working.insert(
        "heart_rate_100_or_over".into(),
        json!(input.heart_rate_100_or_over),
    );
    working.insert("spo2_below_95".into(), json!(input.spo2_below_95));
    working.insert(
        "unilateral_leg_swelling".into(),
        json!(input.unilateral_leg_swelling),
    );
    working.insert("haemoptysis".into(), json!(input.haemoptysis));
    working.insert(
        "recent_surgery_or_trauma".into(),
        json!(input.recent_surgery_or_trauma),
    );
    working.insert("prior_vte".into(), json!(input.prior_vte));
    working.insert("hormone_use".into(), json!(input.hormone_use));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.result.slug()),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Perc;

impl Calculator for Perc {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "PERC Rule (PE Rule-out Criteria)"
    }

    fn description(&self) -> &'static str {
        "8-item rule-out checklist for suspected pulmonary embolism, used only in patients already \
judged low pretest probability; PERC-negative (all 8 absent) needs no further testing."
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
            "title": "PercInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age_50_or_over", "heart_rate_100_or_over", "spo2_below_95",
                "unilateral_leg_swelling", "haemoptysis", "recent_surgery_or_trauma",
                "prior_vte", "hormone_use"
            ],
            "properties": {
                "age_50_or_over": {
                    "type": "boolean",
                    "description": "Age 50 years or over",
                    "definition": {
                        "concept": "Age criterion",
                        "statement": "Patient age is 50 years or over.",
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2004;2(8):1247-1255.", "url": "https://doi.org/10.1111/j.1538-7836.2004.00790.x" },
                        "status": "draft"
                    }
                },
                "heart_rate_100_or_over": {
                    "type": "boolean",
                    "description": "Heart rate 100 beats per minute or over",
                    "definition": {
                        "concept": "Tachycardia criterion",
                        "statement": "Measured heart rate of 100 bpm or greater.",
                        "snomedEcl": "<< 3424008 |Tachycardia (finding)|",
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2004;2(8):1247-1255.", "url": "https://doi.org/10.1111/j.1538-7836.2004.00790.x" },
                        "status": "draft"
                    }
                },
                "spo2_below_95": {
                    "type": "boolean",
                    "description": "Oxygen saturation on room air below 95%",
                    "definition": {
                        "concept": "Hypoxaemia criterion",
                        "statement": "Pulse oximetry on room air below 95%.",
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2004;2(8):1247-1255.", "url": "https://doi.org/10.1111/j.1538-7836.2004.00790.x" },
                        "status": "draft"
                    }
                },
                "unilateral_leg_swelling": {
                    "type": "boolean",
                    "description": "Unilateral leg swelling",
                    "definition": {
                        "concept": "Unilateral leg swelling criterion",
                        "statement": "Visible unilateral leg swelling on examination.",
                        "excludes": ["Bilateral leg oedema with no lateralising finding"],
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2004;2(8):1247-1255.", "url": "https://doi.org/10.1111/j.1538-7836.2004.00790.x" },
                        "status": "draft"
                    }
                },
                "haemoptysis": {
                    "type": "boolean",
                    "description": "Haemoptysis",
                    "definition": {
                        "concept": "Haemoptysis",
                        "statement": "Coughing up of blood or blood-stained sputum.",
                        "snomedEcl": "<< 66857006 |Hemoptysis (finding)|",
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2004;2(8):1247-1255.", "url": "https://doi.org/10.1111/j.1538-7836.2004.00790.x" },
                        "status": "draft"
                    }
                },
                "recent_surgery_or_trauma": {
                    "type": "boolean",
                    "description": "Recent surgery or trauma requiring hospitalisation or general anaesthesia, within the prior 4 weeks",
                    "definition": {
                        "concept": "Recent surgery or trauma criterion",
                        "statement": "Surgery or trauma within the prior 4 weeks that required hospitalisation or general anaesthesia.",
                        "excludes": ["Minor procedures not requiring hospitalisation or general anaesthesia", "Surgery or trauma more than 4 weeks ago"],
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2004;2(8):1247-1255.", "url": "https://doi.org/10.1111/j.1538-7836.2004.00790.x" },
                        "status": "draft"
                    }
                },
                "prior_vte": {
                    "type": "boolean",
                    "description": "Prior venous thromboembolism (DVT or PE)",
                    "definition": {
                        "concept": "Prior VTE criterion",
                        "statement": "A prior deep vein thrombosis or pulmonary embolism.",
                        "snomedEcl": "<< 128053003 |Deep venous thrombosis (disorder)| OR << 59282003 |Pulmonary embolism (disorder)|",
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2004;2(8):1247-1255.", "url": "https://doi.org/10.1111/j.1538-7836.2004.00790.x" },
                        "status": "draft"
                    }
                },
                "hormone_use": {
                    "type": "boolean",
                    "description": "Hormone use: oral contraceptives, hormone replacement, or oestrogen-based hormone therapy",
                    "definition": {
                        "concept": "Hormone use criterion",
                        "statement": "Current use of oestrogen-containing oral contraceptives, hormone replacement therapy, or other oestrogen-based hormone therapy.",
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2004;2(8):1247-1255.", "url": "https://doi.org/10.1111/j.1538-7836.2004.00790.x" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: PercInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> PercInput {
        PercInput {
            age_50_or_over: false,
            heart_rate_100_or_over: false,
            spo2_below_95: false,
            unilateral_leg_swelling: false,
            haemoptysis: false,
            recent_surgery_or_trauma: false,
            prior_vte: false,
            hormone_use: false,
        }
    }

    fn all() -> PercInput {
        PercInput {
            age_50_or_over: true,
            heart_rate_100_or_over: true,
            spo2_below_95: true,
            unilateral_leg_swelling: true,
            haemoptysis: true,
            recent_surgery_or_trauma: true,
            prior_vte: true,
            hormone_use: true,
        }
    }

    #[test]
    fn all_absent_is_perc_negative() {
        let o = compute(&none()).unwrap();
        assert_eq!(o.positive_count, 0);
        assert!(o.positive_criteria.is_empty());
        assert_eq!(o.result, PercResult::Negative);
        assert!(o.interpretation.contains("PERC-negative"));
        assert!(o.interpretation.contains("low pretest probability"));
    }

    #[test]
    fn all_present_is_perc_positive_with_max_count() {
        let o = compute(&all()).unwrap();
        assert_eq!(o.positive_count, 8);
        assert_eq!(o.positive_criteria.len(), 8);
        assert_eq!(o.result, PercResult::Positive);
        assert!(o.interpretation.contains("PERC-positive"));
    }

    #[test]
    fn single_criterion_age_is_perc_positive() {
        let mut i = none();
        i.age_50_or_over = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.positive_count, 1);
        assert_eq!(o.positive_criteria, vec!["age_50_or_over"]);
        assert_eq!(o.result, PercResult::Positive);
    }

    #[test]
    fn single_criterion_hormone_use_is_perc_positive() {
        let mut i = none();
        i.hormone_use = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.positive_count, 1);
        assert_eq!(o.positive_criteria, vec!["hormone_use"]);
        assert_eq!(o.result, PercResult::Positive);
    }

    #[test]
    fn positive_criteria_lists_only_the_present_ones_in_order() {
        let mut i = none();
        i.spo2_below_95 = true;
        i.prior_vte = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.positive_count, 2);
        assert_eq!(o.positive_criteria, vec!["spo2_below_95", "prior_vte"]);
    }

    #[test]
    fn build_response_carries_working_and_reference() {
        let mut i = none();
        i.haemoptysis = true;
        let r = build_response(&i).unwrap();
        assert_eq!(r.calculator, "perc");
        assert_eq!(r.result, json!("perc-positive"));
        assert_eq!(r.working["positive_count"], json!(1));
        assert_eq!(r.working["result"], json!("perc-positive"));
        assert_eq!(r.working["positive_criteria"], json!(["haemoptysis"]));
        assert!(r.reference.contains("Kline"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age_50_or_over": false,
            "heart_rate_100_or_over": true,
            "spo2_below_95": false,
            "unilateral_leg_swelling": false,
            "haemoptysis": false,
            "recent_surgery_or_trauma": false,
            "prior_vte": false,
            "hormone_use": false
        });
        let mut typed = none();
        typed.heart_rate_100_or_over = true;
        let dynamic = Perc.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!("perc-positive"));
    }

    #[test]
    fn dynamic_calculate_rejects_garbage() {
        assert!(Perc.calculate(&json!({ "age_50_or_over": "yes" })).is_err());
    }

    #[test]
    fn dynamic_calculate_rejects_unknown_fields() {
        let mut value = serde_json::to_value(all()).unwrap();
        value["extra_field"] = json!(true);
        assert!(Perc.calculate(&value).is_err());
    }

    #[test]
    fn schema_lists_all_eight_criteria() {
        let schema = Perc.input_schema();
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 8);
    }
}
