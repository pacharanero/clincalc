// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! ASRS v1.1 six-question Adult ADHD Screener scoring adapter.
//!
//! This module implements the official scoring rule over six coded responses.
//! It does not reproduce or paraphrase the questionnaire. Obtain the item text
//! from the authorised form linked by [`LICENSE`].

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "asrs";

/// The rights holder permits electronic versions of the six-question screener
/// with attribution, but does not permit other modifications.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "© New York University and the President and Fellows of Harvard College. Free clinical and non-clinical use, including commercial use, with attribution; electronic versions permitted; no other modifications.",
    source_url: "https://license.tov.med.nyu.edu/product/asrs6Qscreener",
};

/// Attribution required by the rights holder.
pub const ATTRIBUTION: &str = "The 6-question Adult Self-Report Scale-Version1.1 (ASRS-V1.1) Screener is a subset of the 18-question Adult ADHD Self-Report Scale-Version1.1 (Adult ASRSV1.1) Symptom Checklist. © New York University and the President and Fellows of Harvard College.";

/// Primary validation citation and required attribution.
pub const REFERENCE: &str = "Kessler RC, Adler L, Ames M, et al. The World Health Organization adult ADHD self-report scale (ASRS): a short screening scale for use in the general population. Psychol Med. 2005;35(2):245-256. doi:10.1017/S0033291704002892. The 6-question ASRS-v1.1 Screener is a subset of the 18-question Adult ASRS-v1.1 Symptom Checklist. © New York University and the President and Fellows of Harvard College.";

/// Number of responses accepted by the six-question screener.
pub const ITEM_COUNT: usize = 6;

/// Official positive-response threshold for each item, using response codes
/// 0 (Never) through 4 (Very Often).
const POSITIVE_THRESHOLDS: [u8; ITEM_COUNT] = [2, 2, 2, 3, 3, 3];

/// Six coded responses in official item order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AsrsInput {
    pub responses: Vec<u8>,
}

/// The computed six-question screening outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsrsOutcome {
    /// Count of items meeting their frequency threshold (0-6).
    pub part_a_positive: u8,
    /// Raw sum of the six response codes (0-24). This is not the classic
    /// screener decision rule, which uses `part_a_positive`.
    pub part_a_total: u16,
    /// True when at least four items meet their frequency threshold.
    pub screen_positive: bool,
    /// Per-item threshold result in official item order.
    pub part_a_item_positive: [bool; ITEM_COUNT],
    pub interpretation: String,
}

fn interpret(part_a_positive: u8) -> String {
    if part_a_positive >= 4 {
        format!(
            "Positive ASRS-v1.1 six-question screen: {part_a_positive}/6 items meet the official frequency threshold. This indicates a higher risk of adult ADHD and should be followed by clinical evaluation. The screener is not diagnostic, and diagnosis or prescribing must not be based solely on this result."
        )
    } else {
        format!(
            "Negative ASRS-v1.1 six-question screen: {part_a_positive}/6 items meet the official frequency threshold (4 required for a positive screen). A negative screen does not exclude ADHD; use clinical judgement and assess further if concern persists. The screener is not diagnostic."
        )
    }
}

/// Score six response codes from an authorised ASRS-v1.1 form.
pub fn compute(input: &AsrsInput) -> Result<AsrsOutcome, CalcError> {
    if input.responses.len() != ITEM_COUNT {
        return Err(CalcError::InvalidInput(format!(
            "expected {ITEM_COUNT} responses, got {}",
            input.responses.len()
        )));
    }

    let mut part_a_item_positive = [false; ITEM_COUNT];
    let mut part_a_positive = 0u8;
    let mut part_a_total = 0u16;

    for (index, &response) in input.responses.iter().enumerate() {
        if response > 4 {
            return Err(CalcError::InvalidInput(format!(
                "response {} = {response} is out of range 0-4",
                index + 1
            )));
        }

        part_a_total += u16::from(response);
        let positive = response >= POSITIVE_THRESHOLDS[index];
        part_a_item_positive[index] = positive;
        part_a_positive += u8::from(positive);
    }

    Ok(AsrsOutcome {
        part_a_positive,
        part_a_total,
        screen_positive: part_a_positive >= 4,
        part_a_item_positive,
        interpretation: interpret(part_a_positive),
    })
}

/// Build the registry-wide response without reproducing questionnaire text.
pub fn build_response(input: &AsrsInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;

    let mut working = Map::new();
    working.insert(
        "part_a_screen_result".into(),
        json!(if outcome.screen_positive {
            "POSITIVE"
        } else {
            "NEGATIVE"
        }),
    );
    working.insert(
        "part_a_positive_item_count".into(),
        json!(outcome.part_a_positive),
    );
    working.insert("part_a_total_score".into(), json!(outcome.part_a_total));
    working.insert(
        "part_a_item_positive".into(),
        json!(outcome.part_a_item_positive),
    );
    working.insert("answers".into(), json!(input.responses));
    working.insert("attribution".into(), json!(ATTRIBUTION));
    working.insert("official_form".into(), json!(LICENSE.source_url));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.part_a_positive),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Dynamic calculator surface used by every host.
pub struct Asrs;

impl Calculator for Asrs {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "ASRS-v1.1 Six-Question Adult ADHD Screener"
    }

    fn description(&self) -> &'static str {
        "Scores six coded responses from the authorised ASRS-v1.1 form; questionnaire text is not bundled."
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
            "title": "AsrsInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["responses"],
            "properties": {
                "responses": {
                    "type": "array",
                    "description": "Exactly six response codes from the authorised ASRS-v1.1 six-question form, in official item order: 0=Never, 1=Rarely, 2=Sometimes, 3=Often, 4=Very Often. Obtain the item wording from the official form.",
                    "items": { "type": "integer", "minimum": 0, "maximum": 4 },
                    "minItems": 6,
                    "maxItems": 6,
                    "definition": {
                        "concept": "ASRS-v1.1 six-question response codes",
                        "statement": "Six frequency responses recorded from the authorised form in official item order.",
                        "source": {
                            "citation": "ASRS v1.1 6-Question Screener, New York University and the President and Fellows of Harvard College.",
                            "url": "https://license.tov.med.nyu.edu/product/asrs6Qscreener"
                        },
                        "caveats": "The questionnaire wording is not included. Do not infer, reorder, translate, or modify the items.",
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AsrsInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn responses(values: [u8; ITEM_COUNT]) -> AsrsInput {
        AsrsInput {
            responses: values.to_vec(),
        }
    }

    #[test]
    fn all_zero_is_negative() {
        let outcome = compute(&responses([0; ITEM_COUNT])).unwrap();
        assert_eq!(outcome.part_a_positive, 0);
        assert_eq!(outcome.part_a_total, 0);
        assert!(!outcome.screen_positive);
    }

    #[test]
    fn official_item_thresholds_are_applied() {
        let below_threshold = compute(&responses([1, 1, 1, 2, 2, 2])).unwrap();
        assert_eq!(below_threshold.part_a_item_positive, [false; ITEM_COUNT]);

        let at_threshold = compute(&responses([2, 2, 2, 3, 3, 3])).unwrap();
        assert_eq!(at_threshold.part_a_item_positive, [true; ITEM_COUNT]);
        assert_eq!(at_threshold.part_a_positive, 6);
        assert!(at_threshold.screen_positive);
    }

    #[test]
    fn four_positive_items_is_a_positive_screen() {
        let outcome = compute(&responses([2, 2, 2, 3, 0, 0])).unwrap();
        assert_eq!(outcome.part_a_positive, 4);
        assert!(outcome.screen_positive);
        assert!(outcome.interpretation.contains("not diagnostic"));
    }

    #[test]
    fn exactly_six_responses_are_required() {
        for count in [5, 7, 18] {
            assert!(matches!(
                compute(&AsrsInput {
                    responses: vec![0; count]
                }),
                Err(CalcError::InvalidInput(_))
            ));
        }
    }

    #[test]
    fn out_of_range_response_is_rejected() {
        assert!(matches!(
            compute(&responses([0, 0, 0, 0, 0, 5])),
            Err(CalcError::InvalidInput(_))
        ));
    }

    #[test]
    fn dynamic_surface_preserves_attribution_without_question_text() {
        let values = [2, 2, 2, 3, 0, 0];
        let dynamic = Asrs.calculate(&json!({ "responses": values })).unwrap();
        let typed = build_response(&responses(values)).unwrap();

        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(4));
        assert_eq!(dynamic.working["part_a_screen_result"], json!("POSITIVE"));
        assert_eq!(dynamic.working["official_form"], json!(LICENSE.source_url));
        assert!(
            dynamic.working["attribution"]
                .as_str()
                .unwrap()
                .contains("New York University")
        );

        let schema = Asrs.input_schema().to_string();
        assert!(schema.contains("authorised ASRS-v1.1 six-question form"));
        assert!(!schema.contains("trouble wrapping"));
    }
}
