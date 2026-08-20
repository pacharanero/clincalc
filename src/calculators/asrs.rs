// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! ASRS v1.1 six-question Adult ADHD Screener scoring adapter.
//!
//! This module implements both official scoring methods over six coded responses.
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

/// Primary validation citations and required attribution.
pub const REFERENCE: &str = "Kessler RC, Adler L, Ames M, et al. The World Health Organization adult ADHD self-report scale (ASRS): a short screening scale for use in the general population. Psychol Med. 2005;35(2):245-256. doi:10.1017/S0033291704002892. Kessler RC, Adler LA, Gruber MJ, et al. Validity of the World Health Organization Adult ADHD Self-Report Scale (ASRS) Screener in a representative sample of health plan members. Int J Methods Psychiatr Res. 2007;16(2):52-65. doi:10.1002/mpr.208. Harvard Medical School. ASRS v1.1 Scoring update. 2024-02-28. https://www.hcp.med.harvard.edu/ncs/ftpdir/adhd/ASRS_v1.1_screener(6Q)_scoring_update.pdf. The 6-question ASRS-v1.1 Screener is a subset of the 18-question Adult ASRS-v1.1 Symptom Checklist. © New York University and the President and Fellows of Harvard College.";

/// Official clarification of the alternative continuous scoring method.
pub const SCORING_UPDATE_URL: &str =
    "https://www.hcp.med.harvard.edu/ncs/ftpdir/adhd/ASRS_v1.1_screener(6Q)_scoring_update.pdf";

/// Stable machine codes for the two official scoring methods.
pub const CLASSIC_SCORING_METHOD: &str = "classic_dichotomous";
pub const CONTINUOUS_SCORING_METHOD: &str = "continuous_total";

/// Number of responses accepted by the six-question screener.
pub const ITEM_COUNT: usize = 6;

/// Classic positive-response threshold for each item, using response codes
/// 0 (Never) through 4 (Very Often).
const POSITIVE_THRESHOLDS: [u8; ITEM_COUNT] = [2, 2, 2, 3, 3, 3];

/// Six coded responses in official item order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AsrsInput {
    /// The official screener is intended only for adults aged 18 or older.
    pub age_at_least_18: bool,
    /// Responses were recorded using the form's required past-six-month period.
    pub responses_cover_past_six_months: bool,
    pub responses: Vec<u8>,
}

/// Four-stratum interpretation of the continuous 0-24 method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuousStratum {
    LowNegative,
    HighNegative,
    LowPositive,
    HighPositive,
}

impl ContinuousStratum {
    pub fn slug(self) -> &'static str {
        match self {
            Self::LowNegative => "low_negative",
            Self::HighNegative => "high_negative",
            Self::LowPositive => "low_positive",
            Self::HighPositive => "high_positive",
        }
    }

    fn display(self) -> &'static str {
        match self {
            Self::LowNegative => "low negative",
            Self::HighNegative => "high negative",
            Self::LowPositive => "low positive",
            Self::HighPositive => "high positive",
        }
    }
}

/// The computed six-question screening outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsrsOutcome {
    /// Count of items meeting their frequency threshold (0-6).
    pub classic_positive_item_count: u8,
    /// True when at least four items meet their classic frequency threshold.
    pub classic_screen_positive: bool,
    /// Sum of the six response codes (0-24).
    pub continuous_total_score: u8,
    /// True when the continuous total is at least 14.
    pub continuous_screen_positive: bool,
    /// Four-stratum interpretation of the continuous total.
    pub continuous_stratum: ContinuousStratum,
    /// Per-item threshold result in official item order.
    pub classic_item_positive: [bool; ITEM_COUNT],
    pub interpretation: String,
}

fn continuous_stratum(total: u8) -> ContinuousStratum {
    match total {
        0..=9 => ContinuousStratum::LowNegative,
        10..=13 => ContinuousStratum::HighNegative,
        14..=17 => ContinuousStratum::LowPositive,
        _ => ContinuousStratum::HighPositive,
    }
}

fn screen_label(positive: bool) -> &'static str {
    if positive { "positive" } else { "negative" }
}

fn interpret(
    classic_positive_item_count: u8,
    classic_screen_positive: bool,
    continuous_total_score: u8,
    continuous_screen_positive: bool,
    continuous_stratum: ContinuousStratum,
) -> String {
    let disagreement = if classic_screen_positive == continuous_screen_positive {
        String::new()
    } else {
        " The two official methods differ for this response pattern; record which method is being used and interpret the result clinically.".to_string()
    };

    format!(
        "ASRS-v1.1 six-question screen for an adult, covering the past six months. Classic dichotomous clinical method: {} ({classic_positive_item_count}/6 threshold-positive items; 4 required). Alternative continuous total method for research and prevalence studies: {} ({continuous_total_score}/24; {} stratum; 14 required).{disagreement} A positive result indicates higher risk of adult ADHD and should be followed by clinical evaluation; a negative result does not exclude ADHD. The screener is not diagnostic, and diagnosis or prescribing must not be based solely on it.",
        screen_label(classic_screen_positive),
        screen_label(continuous_screen_positive),
        continuous_stratum.display(),
    )
}

/// Score six response codes from an authorised ASRS-v1.1 form.
pub fn compute(input: &AsrsInput) -> Result<AsrsOutcome, CalcError> {
    if !input.age_at_least_18 {
        return Err(CalcError::InvalidInput(
            "the ASRS-v1.1 six-question screener is intended for people aged 18 or older".into(),
        ));
    }
    if !input.responses_cover_past_six_months {
        return Err(CalcError::InvalidInput(
            "ASRS-v1.1 responses must cover the past six months".into(),
        ));
    }
    if input.responses.len() != ITEM_COUNT {
        return Err(CalcError::InvalidInput(format!(
            "expected {ITEM_COUNT} responses, got {}",
            input.responses.len()
        )));
    }

    let mut classic_item_positive = [false; ITEM_COUNT];
    let mut classic_positive_item_count = 0u8;
    let mut continuous_total_score = 0u8;

    for (index, &response) in input.responses.iter().enumerate() {
        if response > 4 {
            return Err(CalcError::InvalidInput(format!(
                "response {} = {response} is out of range 0-4",
                index + 1
            )));
        }

        continuous_total_score += response;
        let positive = response >= POSITIVE_THRESHOLDS[index];
        classic_item_positive[index] = positive;
        classic_positive_item_count += u8::from(positive);
    }

    let classic_screen_positive = classic_positive_item_count >= 4;
    let continuous_screen_positive = continuous_total_score >= 14;
    let continuous_stratum = continuous_stratum(continuous_total_score);

    Ok(AsrsOutcome {
        classic_positive_item_count,
        classic_screen_positive,
        continuous_total_score,
        continuous_screen_positive,
        continuous_stratum,
        classic_item_positive,
        interpretation: interpret(
            classic_positive_item_count,
            classic_screen_positive,
            continuous_total_score,
            continuous_screen_positive,
            continuous_stratum,
        ),
    })
}

/// Build the registry-wide response without reproducing questionnaire text.
pub fn build_response(input: &AsrsInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;

    let mut working = Map::new();
    working.insert(
        "result_scoring_method".into(),
        json!(CLASSIC_SCORING_METHOD),
    );
    working.insert(
        "classic_dichotomous_screen_result".into(),
        json!(if outcome.classic_screen_positive {
            "POSITIVE"
        } else {
            "NEGATIVE"
        }),
    );
    working.insert(
        "classic_dichotomous_positive_item_count".into(),
        json!(outcome.classic_positive_item_count),
    );
    working.insert(
        "classic_dichotomous_item_positive".into(),
        json!(outcome.classic_item_positive),
    );
    working.insert(
        "continuous_total_scoring_method".into(),
        json!(CONTINUOUS_SCORING_METHOD),
    );
    working.insert(
        "continuous_total_score".into(),
        json!(outcome.continuous_total_score),
    );
    working.insert(
        "continuous_total_screen_result".into(),
        json!(if outcome.continuous_screen_positive {
            "POSITIVE"
        } else {
            "NEGATIVE"
        }),
    );
    working.insert(
        "continuous_total_stratum".into(),
        json!(outcome.continuous_stratum.slug()),
    );
    working.insert("age_at_least_18".into(), json!(input.age_at_least_18));
    working.insert("recall_period_months".into(), json!(6));
    working.insert("answers".into(), json!(input.responses));
    working.insert("attribution".into(), json!(ATTRIBUTION));
    working.insert("official_form".into(), json!(LICENSE.source_url));
    working.insert("scoring_update".into(), json!(SCORING_UPDATE_URL));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.classic_positive_item_count),
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
            "required": ["age_at_least_18", "responses_cover_past_six_months", "responses"],
            "properties": {
                "age_at_least_18": {
                    "type": "boolean",
                    "description": "Confirm that the respondent is aged 18 years or older; the ASRS-v1.1 six-question screener is an adult instrument.",
                    "definition": {
                        "concept": "Adult eligibility for ASRS-v1.1",
                        "statement": "The respondent is aged 18 years or older.",
                        "excludes": ["People younger than 18 years"],
                        "source": {
                            "citation": "ASRS v1.1 6-Question Screener, New York University and the President and Fellows of Harvard College.",
                            "url": "https://license.tov.med.nyu.edu/product/asrs6Qscreener"
                        },
                        "caveats": "This assertion must be true before the adult screener is scored.",
                        "status": "draft"
                    }
                },
                "responses_cover_past_six_months": {
                    "type": "boolean",
                    "description": "Confirm that all six responses describe the respondent over the past six months, as required by the authorised form.",
                    "definition": {
                        "concept": "ASRS-v1.1 recall period",
                        "statement": "All six responses cover the past six months.",
                        "excludes": ["Responses about a shorter, longer, or unspecified period"],
                        "source": {
                            "citation": "ASRS v1.1 6-Question Screener, New York University and the President and Fellows of Harvard College.",
                            "url": "https://license.tov.med.nyu.edu/product/asrs6Qscreener"
                        },
                        "caveats": "This assertion must be true before the screener is scored.",
                        "status": "draft"
                    }
                },
                "responses": {
                    "type": "array",
                    "description": "Exactly six past-six-month response codes from the authorised ASRS-v1.1 six-question form, in official item order: 0=Never, 1=Rarely, 2=Sometimes, 3=Often, 4=Very Often. Obtain the item wording from the official form.",
                    "items": { "type": "integer", "minimum": 0, "maximum": 4 },
                    "minItems": 6,
                    "maxItems": 6,
                    "definition": {
                        "concept": "ASRS-v1.1 six-question response codes",
                        "statement": "Six frequency responses covering the past six months, recorded from the authorised form in official item order.",
                        "source": {
                            "citation": "ASRS v1.1 6-Question Screener, New York University and the President and Fellows of Harvard College.",
                            "url": "https://license.tov.med.nyu.edu/product/asrs6Qscreener"
                        },
                        "caveats": "For adults aged 18 or older. The questionnaire wording is not included. Do not infer, reorder, translate, or modify the items. Results expose the classic dichotomous method and the alternative continuous total method separately.",
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
            age_at_least_18: true,
            responses_cover_past_six_months: true,
            responses: values.to_vec(),
        }
    }

    #[test]
    fn all_zero_is_negative() {
        let outcome = compute(&responses([0; ITEM_COUNT])).unwrap();
        assert_eq!(outcome.classic_positive_item_count, 0);
        assert_eq!(outcome.continuous_total_score, 0);
        assert!(!outcome.classic_screen_positive);
        assert!(!outcome.continuous_screen_positive);
        assert_eq!(outcome.continuous_stratum, ContinuousStratum::LowNegative);
    }

    #[test]
    fn official_item_thresholds_are_applied() {
        let below_threshold = compute(&responses([1, 1, 1, 2, 2, 2])).unwrap();
        assert_eq!(below_threshold.classic_item_positive, [false; ITEM_COUNT]);

        let at_threshold = compute(&responses([2, 2, 2, 3, 3, 3])).unwrap();
        assert_eq!(at_threshold.classic_item_positive, [true; ITEM_COUNT]);
        assert_eq!(at_threshold.classic_positive_item_count, 6);
        assert!(at_threshold.classic_screen_positive);
    }

    #[test]
    fn four_positive_items_is_a_positive_screen() {
        let outcome = compute(&responses([2, 2, 2, 3, 0, 0])).unwrap();
        assert_eq!(outcome.classic_positive_item_count, 4);
        assert!(outcome.classic_screen_positive);
        assert!(outcome.interpretation.contains("not diagnostic"));
    }

    #[test]
    fn both_official_scoring_methods_are_reported_separately() {
        let outcome = compute(&responses([4, 4, 4, 2, 0, 0])).unwrap();

        assert_eq!(outcome.classic_positive_item_count, 3);
        assert!(!outcome.classic_screen_positive);
        assert_eq!(outcome.continuous_total_score, 14);
        assert!(outcome.continuous_screen_positive);
        assert_eq!(outcome.continuous_stratum, ContinuousStratum::LowPositive);
        assert!(outcome.interpretation.contains("methods differ"));
    }

    #[test]
    fn continuous_strata_match_the_2024_scoring_update() {
        for (score, expected) in [
            (0, ContinuousStratum::LowNegative),
            (9, ContinuousStratum::LowNegative),
            (10, ContinuousStratum::HighNegative),
            (13, ContinuousStratum::HighNegative),
            (14, ContinuousStratum::LowPositive),
            (17, ContinuousStratum::LowPositive),
            (18, ContinuousStratum::HighPositive),
            (24, ContinuousStratum::HighPositive),
        ] {
            assert_eq!(continuous_stratum(score), expected, "score: {score}");
        }
    }

    #[test]
    fn adult_eligibility_and_six_month_recall_are_required() {
        let mut input = responses([0; ITEM_COUNT]);
        input.age_at_least_18 = false;
        assert_eq!(
            compute(&input),
            Err(CalcError::InvalidInput(
                "the ASRS-v1.1 six-question screener is intended for people aged 18 or older"
                    .into()
            ))
        );

        let mut input = responses([0; ITEM_COUNT]);
        input.responses_cover_past_six_months = false;
        assert_eq!(
            compute(&input),
            Err(CalcError::InvalidInput(
                "ASRS-v1.1 responses must cover the past six months".into()
            ))
        );
    }

    #[test]
    fn exactly_six_responses_are_required() {
        for count in [5, 7, 18] {
            assert!(matches!(
                compute(&AsrsInput {
                    age_at_least_18: true,
                    responses_cover_past_six_months: true,
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
    fn dynamic_surface_preserves_attribution_and_qualifies_scoring_methods() {
        let values = [2, 2, 2, 3, 0, 0];
        let dynamic = Asrs
            .calculate(&json!({
                "age_at_least_18": true,
                "responses_cover_past_six_months": true,
                "responses": values
            }))
            .unwrap();
        let typed = build_response(&responses(values)).unwrap();

        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(4));
        assert_eq!(
            dynamic.working["result_scoring_method"],
            json!(CLASSIC_SCORING_METHOD)
        );
        assert_eq!(
            dynamic.working["classic_dichotomous_screen_result"],
            json!("POSITIVE")
        );
        assert_eq!(
            dynamic.working["continuous_total_scoring_method"],
            json!(CONTINUOUS_SCORING_METHOD)
        );
        assert_eq!(dynamic.working["continuous_total_score"], json!(9));
        assert_eq!(dynamic.working["official_form"], json!(LICENSE.source_url));
        assert!(
            dynamic.working["attribution"]
                .as_str()
                .unwrap()
                .contains("New York University")
        );

        let schema = Asrs.input_schema();
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["properties"]["responses"]["minItems"], 6);
        assert_eq!(schema["properties"]["responses"]["maxItems"], 6);
        assert_eq!(schema["properties"]["age_at_least_18"]["type"], "boolean");
        assert_eq!(
            schema["properties"]["responses_cover_past_six_months"]["type"],
            "boolean"
        );
    }
}
