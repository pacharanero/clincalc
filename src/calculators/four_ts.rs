// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! 4Ts pretest probability score for heparin-induced thrombocytopenia (HIT).
//!
//! This implements the standard Lo/Cuker score using clear onset on days 5-10.
//! Each of the four clinical judgements is a required semantic enum. Callers
//! cannot supply points, omit a judgement, or represent uncertainty by asking
//! the calculator to guess.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Stable machine name.
pub const NAME: &str = "four_ts";

/// Primary validation, meta-analysis, guideline, and current practical guidance.
pub const REFERENCE: &str = "Lo GK, Juhl D, Warkentin TE, Sigouin CS, Eichler P, Greinacher A. Evaluation of pretest clinical score (4 T's) for the diagnosis of heparin-induced thrombocytopenia in two clinical settings. J Thromb Haemost. 2006;4(4):759-765. doi:10.1111/j.1538-7836.2006.01787.x. Cuker A, Gimotty PA, Crowther MA, Warkentin TE. Predictive value of the 4Ts scoring system for heparin-induced thrombocytopenia: a systematic review and meta-analysis. Blood. 2012;120(20):4160-4167. doi:10.1182/blood-2012-07-443051. Cuker A, Arepally GM, Chong BH, et al. American Society of Hematology 2018 guidelines for management of venous thromboembolism: heparin-induced thrombocytopenia. Blood Adv. 2018;2(22):3360-3392. doi:10.1182/bloodadvances.2018024489. May J, Cuker A. Practical guide to the diagnosis and management of heparin-induced thrombocytopenia. Hematology Am Soc Hematol Educ Program. 2024;2024(1):388-395. doi:10.1182/hematology.2024000566.";

/// Distribution licence: independently implemented from the published method.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published clinical scoring method - independently implemented from the primary literature",
    source_url: "https://doi.org/10.1111/j.1538-7836.2006.01787.x",
};

/// Magnitude of platelet-count fall and the platelet nadir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Thrombocytopenia {
    #[serde(rename = "fall_gt_50_nadir_ge_20")]
    FallGt50NadirGe20,
    #[serde(rename = "fall_30_to_50_or_nadir_10_to_19")]
    Fall30To50OrNadir10To19,
    #[serde(rename = "fall_lt_30_or_nadir_lt_10")]
    FallLt30OrNadirLt10,
}

impl Thrombocytopenia {
    fn points(self) -> u8 {
        match self {
            Self::FallGt50NadirGe20 => 2,
            Self::Fall30To50OrNadir10To19 => 1,
            Self::FallLt30OrNadirLt10 => 0,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::FallGt50NadirGe20 => "fall_gt_50_nadir_ge_20",
            Self::Fall30To50OrNadir10To19 => "fall_30_to_50_or_nadir_10_to_19",
            Self::FallLt30OrNadirLt10 => "fall_lt_30_or_nadir_lt_10",
        }
    }
}

/// Timing of the platelet-count fall relative to heparin exposure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Timing {
    #[serde(rename = "clear_day_5_to_10_or_rapid_with_prior_exposure_within_30_days")]
    ClearDay5To10OrRapidWithPriorExposureWithin30Days,
    #[serde(
        rename = "compatible_but_unclear_or_after_day_10_or_rapid_with_prior_exposure_31_to_100_days"
    )]
    CompatibleButUnclearOrAfterDay10OrRapidWithPriorExposure31To100Days,
    #[serde(rename = "day_0_to_4_without_recent_exposure")]
    Day0To4WithoutRecentExposure,
}

impl Timing {
    fn points(self) -> u8 {
        match self {
            Self::ClearDay5To10OrRapidWithPriorExposureWithin30Days => 2,
            Self::CompatibleButUnclearOrAfterDay10OrRapidWithPriorExposure31To100Days => 1,
            Self::Day0To4WithoutRecentExposure => 0,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::ClearDay5To10OrRapidWithPriorExposureWithin30Days => {
                "clear_day_5_to_10_or_rapid_with_prior_exposure_within_30_days"
            }
            Self::CompatibleButUnclearOrAfterDay10OrRapidWithPriorExposure31To100Days => {
                "compatible_but_unclear_or_after_day_10_or_rapid_with_prior_exposure_31_to_100_days"
            }
            Self::Day0To4WithoutRecentExposure => "day_0_to_4_without_recent_exposure",
        }
    }
}

/// Thrombosis or another recognised clinical sequela of HIT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThrombosisOrSequelae {
    NewConfirmedThrombosisSkinNecrosisOrAcuteIvHeparinReaction,
    ProgressiveRecurrentOrSuspectedThrombosisOrErythematousSkinLesion,
    None,
}

impl ThrombosisOrSequelae {
    fn points(self) -> u8 {
        match self {
            Self::NewConfirmedThrombosisSkinNecrosisOrAcuteIvHeparinReaction => 2,
            Self::ProgressiveRecurrentOrSuspectedThrombosisOrErythematousSkinLesion => 1,
            Self::None => 0,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::NewConfirmedThrombosisSkinNecrosisOrAcuteIvHeparinReaction => {
                "new_confirmed_thrombosis_skin_necrosis_or_acute_iv_heparin_reaction"
            }
            Self::ProgressiveRecurrentOrSuspectedThrombosisOrErythematousSkinLesion => {
                "progressive_recurrent_or_suspected_thrombosis_or_erythematous_skin_lesion"
            }
            Self::None => "none",
        }
    }
}

/// Strength of alternative explanations for thrombocytopenia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtherCauses {
    NoneApparent,
    Possible,
    Definite,
}

impl OtherCauses {
    fn points(self) -> u8 {
        match self {
            Self::NoneApparent => 2,
            Self::Possible => 1,
            Self::Definite => 0,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::NoneApparent => "none_apparent",
            Self::Possible => "possible",
            Self::Definite => "definite",
        }
    }
}

/// One complete clinician-resolved 4Ts assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FourTsInput {
    pub thrombocytopenia: Thrombocytopenia,
    pub timing: Timing,
    pub thrombosis_or_sequelae: ThrombosisOrSequelae,
    pub other_causes: OtherCauses,
}

/// Per-category points retained in the typed outcome for auditability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FourTsPoints {
    pub thrombocytopenia: u8,
    pub timing: u8,
    pub thrombosis_or_sequelae: u8,
    pub other_causes: u8,
}

/// 4Ts pretest-probability band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PretestProbability {
    Low,
    Intermediate,
    High,
}

impl PretestProbability {
    fn from_total(total: u8) -> Self {
        match total {
            0..=3 => Self::Low,
            4..=5 => Self::Intermediate,
            _ => Self::High,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Intermediate => "intermediate",
            Self::High => "high",
        }
    }
}

/// Typed 4Ts result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FourTsOutcome {
    /// Total score, 0-8.
    pub total: u8,
    pub points: FourTsPoints,
    pub pretest_probability: PretestProbability,
    pub interpretation: String,
}

fn render_interpretation(total: u8, probability: PretestProbability) -> String {
    let next_step = match probability {
        PretestProbability::Low => {
            "A low score has strong rule-out performance only when all four inputs are complete and correctly classified."
        }
        PretestProbability::Intermediate | PretestProbability::High => {
            "A score of 4 or more requires prompt clinician-led assessment and laboratory testing under an ASH HIT pathway."
        }
    };

    format!(
        "4Ts score {total}/8: {} pretest probability of HIT. This is a pretest probability, not a diagnosis. {next_step} This result does not select an anticoagulant or dose, assess bleeding risk, or autonomously order stopping or starting treatment. Reassess and recalculate if the clinical picture or available information changes.",
        probability.slug()
    )
}

/// Calculate the standard 4Ts score from four resolved clinical categories.
pub fn compute(input: &FourTsInput) -> Result<FourTsOutcome, CalcError> {
    let points = FourTsPoints {
        thrombocytopenia: input.thrombocytopenia.points(),
        timing: input.timing.points(),
        thrombosis_or_sequelae: input.thrombosis_or_sequelae.points(),
        other_causes: input.other_causes.points(),
    };
    let total = points.thrombocytopenia
        + points.timing
        + points.thrombosis_or_sequelae
        + points.other_causes;
    let pretest_probability = PretestProbability::from_total(total);

    Ok(FourTsOutcome {
        total,
        points,
        pretest_probability,
        interpretation: render_interpretation(total, pretest_probability),
    })
}

/// Build the dispatchable response with semantic choices and derived points.
pub fn build_response(input: &FourTsInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "thrombocytopenia".into(),
        json!(input.thrombocytopenia.slug()),
    );
    working.insert(
        "thrombocytopenia_points".into(),
        json!(outcome.points.thrombocytopenia),
    );
    working.insert("timing".into(), json!(input.timing.slug()));
    working.insert("timing_points".into(), json!(outcome.points.timing));
    working.insert(
        "thrombosis_or_sequelae".into(),
        json!(input.thrombosis_or_sequelae.slug()),
    );
    working.insert(
        "thrombosis_or_sequelae_points".into(),
        json!(outcome.points.thrombosis_or_sequelae),
    );
    working.insert("other_causes".into(), json!(input.other_causes.slug()));
    working.insert(
        "other_causes_points".into(),
        json!(outcome.points.other_causes),
    );
    working.insert("total_score".into(), json!(outcome.total));
    working.insert("band".into(), json!(outcome.pretest_probability.slug()));
    working.insert("standard_variant".into(), json!("days_5_to_10"));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.total),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn input_schema() -> Value {
    let lo_source = json!({
        "citation": "Lo GK, Juhl D, Warkentin TE, Sigouin CS, Eichler P, Greinacher A. J Thromb Haemost. 2006;4(4):759-765.",
        "url": "https://doi.org/10.1111/j.1538-7836.2006.01787.x"
    });
    let may_cuker_source = json!({
        "citation": "May J, Cuker A. Hematology Am Soc Hematol Educ Program. 2024;2024(1):388-395.",
        "url": "https://doi.org/10.1182/hematology.2024000566"
    });
    let unresolved = "If the available information cannot support exactly one category, do not guess. The calculator cannot be used until a clinician resolves the category.";

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "FourTsInput",
        "description": "Standard 4Ts pretest probability score for suspected heparin-induced thrombocytopenia, using the Lo/Cuker clear-onset days 5-10 timing variant. All four clinician-resolved categories are required; callers supply semantic categories, never points. If the record cannot support exactly one category for every input, do not guess and do not use the calculator until a clinician resolves the missing or ambiguous information. The result is pretest probability, not a diagnosis or autonomous treatment instruction.",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "thrombocytopenia",
            "timing",
            "thrombosis_or_sequelae",
            "other_causes"
        ],
        "properties": {
            "thrombocytopenia": {
                "type": "string",
                "enum": [
                    "fall_gt_50_nadir_ge_20",
                    "fall_30_to_50_or_nadir_10_to_19",
                    "fall_lt_30_or_nadir_lt_10"
                ],
                "description": "Platelet-count fall and nadir: fall_gt_50_nadir_ge_20=2; fall_30_to_50_or_nadir_10_to_19=1; fall_lt_30_or_nadir_lt_10=0 points",
                "definition": {
                    "concept": "Magnitude of thrombocytopenia",
                    "statement": "Calculate percentage fall from the peak platelet count after heparin exposure to the subsequent nadir. Apply nadir precedence: a nadir below 10 x10^9/L scores 0; a nadir of 10-19 x10^9/L scores 1 regardless of percentage fall; with a nadir at least 20 x10^9/L, a fall greater than 50% scores 2, 30%-50% inclusive scores 1, and a fall below 30% scores 0.",
                    "includes": [
                        "Use the peak platelet count reached after heparin was started, even when that peak is within the laboratory reference interval",
                        "A relative fall can qualify even if the nadir is not below the laboratory lower limit"
                    ],
                    "excludes": [
                        "Do not use caller-supplied points",
                        "Do not ignore the nadir when the percentage fall appears to fit a higher category"
                    ],
                    "caveats": unresolved,
                    "source": lo_source,
                    "status": "draft"
                }
            },
            "timing": {
                "type": "string",
                "enum": [
                    "clear_day_5_to_10_or_rapid_with_prior_exposure_within_30_days",
                    "compatible_but_unclear_or_after_day_10_or_rapid_with_prior_exposure_31_to_100_days",
                    "day_0_to_4_without_recent_exposure"
                ],
                "description": "Timing of the first consistent platelet decline: clear day 5-10 or rapid with prior exposure within 30 days=2; compatible but unclear, after day 10, or rapid with exposure 31-100 days ago=1; day 0-4 without recent exposure=0 points",
                "definition": {
                    "concept": "Timing of platelet-count fall after heparin exposure",
                    "statement": "The day heparin starts is day 0. Date onset from the first day of the platelet-count sequence that shows a consistent decline, not from the day thrombocytopenia crosses a laboratory threshold. Clear onset on days 5-10 scores 2. A fall within 1 day scores 2 when prior heparin exposure was within 30 days, including exactly 30 days, and scores 1 when exposure was 31-100 days earlier. Compatible but unclear onset or onset after day 10 scores 1. Onset on days 0-4 without recent exposure scores 0.",
                    "includes": [
                        "Exactly 30 days since prior heparin exposure is in the 2-point category",
                        "Prior exposure 31 through 100 days earlier is in the 1-point category"
                    ],
                    "excludes": [
                        "Do not date onset from the first count below the reference interval when an earlier consistent decline is visible",
                        "Do not treat absent platelet-count observations as evidence of a clear onset"
                    ],
                    "caveats": "This calculator intentionally uses the canonical standard clear-onset days 5-10 variant described by Lo, Cuker, and May/Cuker. It is not the days 5-14 wording shown in the 2018 ASH pocket guide. If timing still cannot be assigned to exactly one category after review, do not guess; a clinician must resolve it before use.",
                    "source": may_cuker_source,
                    "status": "draft"
                }
            },
            "thrombosis_or_sequelae": {
                "type": "string",
                "enum": [
                    "new_confirmed_thrombosis_skin_necrosis_or_acute_iv_heparin_reaction",
                    "progressive_recurrent_or_suspected_thrombosis_or_erythematous_skin_lesion",
                    "none"
                ],
                "description": "Temporally relevant thrombosis or HIT sequela: new confirmed thrombosis, skin necrosis, or acute reaction after IV heparin=2; progressive, recurrent, or suspected thrombosis, or erythematous skin lesion=1; none=0 points",
                "definition": {
                    "concept": "Thrombosis or another HIT-associated clinical sequela",
                    "statement": "Classify a finding only when it is temporally relevant to the current platelet fall and suspected HIT episode. A new objectively confirmed thrombosis, heparin-site skin necrosis, or acute systemic reaction after an intravenous unfractionated-heparin bolus scores 2. Progressive or recurrent thrombosis, clinically suspected but unconfirmed thrombosis, or a non-necrotising erythematous heparin-site lesion scores 1. No relevant finding scores 0.",
                    "includes": [
                        "New venous, arterial, or other objectively confirmed thrombosis arising in the current episode",
                        "Acute systemic reaction temporally following an intravenous unfractionated-heparin bolus"
                    ],
                    "excludes": [
                        "Remote stable thrombosis with no temporal relevance to the current episode",
                        "A historical skin lesion that is unrelated to a current heparin site"
                    ],
                    "caveats": unresolved,
                    "source": lo_source,
                    "status": "draft"
                }
            },
            "other_causes": {
                "type": "string",
                "enum": ["none_apparent", "possible", "definite"],
                "description": "Alternative cause of thrombocytopenia: none_apparent=2; possible=1; definite=0 points",
                "definition": {
                    "concept": "Alternative causes of thrombocytopenia",
                    "statement": "After clinician review of the differential diagnosis, select none_apparent only when no credible alternative is evident, possible when one or more alternatives could explain the fall, and definite when another cause accounts for the thrombocytopenia.",
                    "includes": [
                        "Examples to assess include recent surgery, sepsis, disseminated intravascular coagulation, non-heparin drugs, transfusion or haemodilution, and platelet-consuming intravascular devices",
                        "After cardiac surgery, assess the full platelet trajectory: an early postoperative fall followed by recovery and then a second fall around days 5-10 is the characteristic biphasic pattern that increases concern for HIT"
                    ],
                    "excludes": [
                        "Do not select none_apparent merely because no alternative cause has yet been sought",
                        "Do not assume every early platelet fall after cardiopulmonary bypass is HIT; surgery and bypass are common alternative explanations"
                    ],
                    "caveats": "Alternative-cause assessment is a clinical judgement and should be revisited as investigations and the platelet trajectory evolve. A persistent early postoperative fall without recovery is rarely the usual HIT pattern after cardiac surgery. If the differential cannot support exactly one category, do not guess; a clinician must resolve it before use.",
                    "source": may_cuker_source,
                    "status": "draft"
                }
            }
        }
    })
}

/// Dynamic calculator implementation.
pub struct FourTs;

impl Calculator for FourTs {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "4Ts Score for Heparin-Induced Thrombocytopenia"
    }

    fn description(&self) -> &'static str {
        "Estimates low, intermediate, or high pretest probability of heparin-induced thrombocytopenia from four clinician-resolved categories."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        input_schema()
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: FourTsInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::SupportedLocale;

    fn minimum_input() -> FourTsInput {
        FourTsInput {
            thrombocytopenia: Thrombocytopenia::FallLt30OrNadirLt10,
            timing: Timing::Day0To4WithoutRecentExposure,
            thrombosis_or_sequelae: ThrombosisOrSequelae::None,
            other_causes: OtherCauses::Definite,
        }
    }

    fn input_with_total(total: u8) -> FourTsInput {
        let values = [
            total.min(2),
            total.saturating_sub(2).min(2),
            total.saturating_sub(4).min(2),
            total.saturating_sub(6).min(2),
        ];
        FourTsInput {
            thrombocytopenia: match values[0] {
                0 => Thrombocytopenia::FallLt30OrNadirLt10,
                1 => Thrombocytopenia::Fall30To50OrNadir10To19,
                _ => Thrombocytopenia::FallGt50NadirGe20,
            },
            timing: match values[1] {
                0 => Timing::Day0To4WithoutRecentExposure,
                1 => Timing::CompatibleButUnclearOrAfterDay10OrRapidWithPriorExposure31To100Days,
                _ => Timing::ClearDay5To10OrRapidWithPriorExposureWithin30Days,
            },
            thrombosis_or_sequelae: match values[2] {
                0 => ThrombosisOrSequelae::None,
                1 => ThrombosisOrSequelae::ProgressiveRecurrentOrSuspectedThrombosisOrErythematousSkinLesion,
                _ => ThrombosisOrSequelae::NewConfirmedThrombosisSkinNecrosisOrAcuteIvHeparinReaction,
            },
            other_causes: match values[3] {
                0 => OtherCauses::Definite,
                1 => OtherCauses::Possible,
                _ => OtherCauses::NoneApparent,
            },
        }
    }

    #[test]
    fn every_enum_value_maps_to_published_points_and_wire_slug() {
        for (value, points, slug) in [
            (
                Thrombocytopenia::FallGt50NadirGe20,
                2,
                "fall_gt_50_nadir_ge_20",
            ),
            (
                Thrombocytopenia::Fall30To50OrNadir10To19,
                1,
                "fall_30_to_50_or_nadir_10_to_19",
            ),
            (
                Thrombocytopenia::FallLt30OrNadirLt10,
                0,
                "fall_lt_30_or_nadir_lt_10",
            ),
        ] {
            assert_eq!(value.points(), points);
            assert_eq!(value.slug(), slug);
            assert_eq!(serde_json::to_value(value).unwrap(), json!(slug));
        }

        for (value, points, slug) in [
            (
                Timing::ClearDay5To10OrRapidWithPriorExposureWithin30Days,
                2,
                "clear_day_5_to_10_or_rapid_with_prior_exposure_within_30_days",
            ),
            (
                Timing::CompatibleButUnclearOrAfterDay10OrRapidWithPriorExposure31To100Days,
                1,
                "compatible_but_unclear_or_after_day_10_or_rapid_with_prior_exposure_31_to_100_days",
            ),
            (
                Timing::Day0To4WithoutRecentExposure,
                0,
                "day_0_to_4_without_recent_exposure",
            ),
        ] {
            assert_eq!(value.points(), points);
            assert_eq!(value.slug(), slug);
            assert_eq!(serde_json::to_value(value).unwrap(), json!(slug));
        }

        for (value, points, slug) in [
            (ThrombosisOrSequelae::NewConfirmedThrombosisSkinNecrosisOrAcuteIvHeparinReaction, 2, "new_confirmed_thrombosis_skin_necrosis_or_acute_iv_heparin_reaction"),
            (ThrombosisOrSequelae::ProgressiveRecurrentOrSuspectedThrombosisOrErythematousSkinLesion, 1, "progressive_recurrent_or_suspected_thrombosis_or_erythematous_skin_lesion"),
            (ThrombosisOrSequelae::None, 0, "none"),
        ] {
            assert_eq!(value.points(), points);
            assert_eq!(value.slug(), slug);
            assert_eq!(serde_json::to_value(value).unwrap(), json!(slug));
        }

        for (value, points, slug) in [
            (OtherCauses::NoneApparent, 2, "none_apparent"),
            (OtherCauses::Possible, 1, "possible"),
            (OtherCauses::Definite, 0, "definite"),
        ] {
            assert_eq!(value.points(), points);
            assert_eq!(value.slug(), slug);
            assert_eq!(serde_json::to_value(value).unwrap(), json!(slug));
        }
    }

    #[test]
    fn minimum_and_maximum_vectors_score_zero_and_eight() {
        let minimum = compute(&minimum_input()).unwrap();
        assert_eq!(minimum.total, 0);
        assert_eq!(minimum.pretest_probability, PretestProbability::Low);

        let maximum = compute(&input_with_total(8)).unwrap();
        assert_eq!(maximum.total, 8);
        assert_eq!(maximum.pretest_probability, PretestProbability::High);
    }

    #[test]
    fn probability_boundaries_are_exact() {
        for (total, expected) in [
            (3, PretestProbability::Low),
            (4, PretestProbability::Intermediate),
            (5, PretestProbability::Intermediate),
            (6, PretestProbability::High),
        ] {
            let outcome = compute(&input_with_total(total)).unwrap();
            assert_eq!(outcome.total, total);
            assert_eq!(outcome.pretest_probability, expected);
        }
    }

    #[test]
    fn interpretation_is_pretest_probability_with_required_safety_limits() {
        let low = compute(&input_with_total(3)).unwrap().interpretation;
        assert!(low.contains("low pretest probability"));
        assert!(low.contains("not a diagnosis"));
        assert!(low.contains("strong rule-out performance only when all four inputs are complete"));

        let intermediate = compute(&input_with_total(4)).unwrap().interpretation;
        assert!(intermediate.contains("prompt clinician-led assessment and laboratory testing"));
        for limit in [
            "does not select an anticoagulant or dose",
            "assess bleeding risk",
            "autonomously order stopping or starting treatment",
            "Reassess and recalculate",
        ] {
            assert!(intermediate.contains(limit));
        }
    }

    #[test]
    fn dynamic_calculation_matches_typed_and_working_is_complete() {
        let input = input_with_total(5);
        let dynamic = FourTs
            .calculate(&serde_json::to_value(input).unwrap())
            .unwrap();
        assert_eq!(dynamic, build_response(&input).unwrap());
        assert_eq!(dynamic.result, json!(5));
        assert_eq!(
            dynamic.working["thrombocytopenia"],
            json!(input.thrombocytopenia.slug())
        );
        assert_eq!(dynamic.working["thrombocytopenia_points"], json!(2));
        assert_eq!(dynamic.working["timing"], json!(input.timing.slug()));
        assert_eq!(dynamic.working["timing_points"], json!(2));
        assert_eq!(
            dynamic.working["thrombosis_or_sequelae"],
            json!(input.thrombosis_or_sequelae.slug())
        );
        assert_eq!(dynamic.working["thrombosis_or_sequelae_points"], json!(1));
        assert_eq!(
            dynamic.working["other_causes"],
            json!(input.other_causes.slug())
        );
        assert_eq!(dynamic.working["other_causes_points"], json!(0));
        assert_eq!(dynamic.working["total_score"], json!(5));
        assert_eq!(dynamic.working["band"], json!("intermediate"));
        assert_eq!(dynamic.working["standard_variant"], json!("days_5_to_10"));
    }

    #[test]
    fn rejects_unknown_missing_and_invalid_enum_inputs() {
        let valid = serde_json::to_value(input_with_total(4)).unwrap();

        let mut unknown = valid.clone();
        unknown["caller_supplied_points"] = json!(8);
        assert!(FourTs.calculate(&unknown).is_err());

        let mut missing = valid.clone();
        missing.as_object_mut().unwrap().remove("timing");
        assert!(FourTs.calculate(&missing).is_err());

        let mut invalid_enum = valid;
        invalid_enum["other_causes"] = json!("unable_to_determine");
        assert!(FourTs.calculate(&invalid_enum).is_err());
    }

    #[test]
    fn schema_is_closed_required_and_defines_every_semantic_input() {
        let schema = FourTs.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(
            schema["required"],
            json!([
                "thrombocytopenia",
                "timing",
                "thrombosis_or_sequelae",
                "other_causes"
            ])
        );
        assert!(
            schema["description"]
                .as_str()
                .unwrap()
                .contains("do not guess")
        );

        for name in [
            "thrombocytopenia",
            "timing",
            "thrombosis_or_sequelae",
            "other_causes",
        ] {
            let property = &schema["properties"][name];
            assert!(property["description"].is_string());
            assert!(property["definition"]["statement"].is_string());
            assert!(property["definition"]["caveats"].is_string());
            assert!(property["definition"]["source"]["url"].is_string());
            assert_eq!(property["definition"]["status"], json!("draft"));
        }
    }

    #[test]
    fn schema_records_boundaries_variant_and_clinical_caveats() {
        let schema = FourTs.input_schema();
        let thrombocytopenia = &schema["properties"]["thrombocytopenia"]["definition"];
        assert!(
            thrombocytopenia["statement"]
                .as_str()
                .unwrap()
                .contains("peak platelet count after heparin exposure")
        );
        assert!(
            thrombocytopenia["statement"]
                .as_str()
                .unwrap()
                .contains("nadir precedence")
        );

        let timing = &schema["properties"]["timing"]["definition"];
        let timing_statement = timing["statement"].as_str().unwrap();
        assert!(timing_statement.contains("day 0"));
        assert!(timing_statement.contains("first day"));
        assert!(timing_statement.contains("exactly 30 days"));
        assert!(timing_statement.contains("31-100 days"));
        assert!(timing["caveats"].as_str().unwrap().contains("days 5-10"));
        assert!(timing["caveats"].as_str().unwrap().contains("days 5-14"));

        let thrombosis = &schema["properties"]["thrombosis_or_sequelae"]["definition"];
        assert!(
            thrombosis["statement"]
                .as_str()
                .unwrap()
                .contains("temporally relevant")
        );

        let other_causes = &schema["properties"]["other_causes"]["definition"];
        assert!(
            other_causes["includes"][0]
                .as_str()
                .unwrap()
                .contains("sepsis")
        );
        assert!(
            other_causes["includes"][1]
                .as_str()
                .unwrap()
                .contains("biphasic")
        );
        assert!(
            other_causes["caveats"]
                .as_str()
                .unwrap()
                .contains("cardiac surgery")
        );
    }

    #[test]
    fn english_locale_is_supported_and_recorded() {
        assert_eq!(FourTs.supported_locales(), crate::locale::ENGLISH_ONLY);
        let response = FourTs
            .calculate_for(
                &serde_json::to_value(input_with_total(4)).unwrap(),
                SupportedLocale::En,
            )
            .unwrap();
        assert_eq!(response.working["content_locale"], json!("en"));
    }
}
