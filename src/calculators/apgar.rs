// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Conventional Apgar score for one observation set after birth.
//!
//! The five observed signs are each scored 0, 1, or 2. This implementation
//! records the assessment time, gestational context, and whether resuscitation
//! was in progress so that the total is not presented without its limitations.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "apgar";

/// Primary scoring publication and current interpretation guidance.
pub const REFERENCE: &str = "Apgar V. A proposal for a new method of evaluation of the newborn infant. Curr Res Anesth Analg. 1953;32(4):260-267. PMID:13083014. American Academy of Pediatrics Committee on Fetus and Newborn; American College of Obstetricians and Gynecologists Committee on Obstetric Practice. The Apgar Score. Pediatrics. 2015;136(4):819-822. doi:10.1542/peds.2015-2651. American Heart Association; American Academy of Pediatrics. Part 5: Neonatal Resuscitation: 2025 American Heart Association and American Academy of Pediatrics Guidelines for Cardiopulmonary Resuscitation and Emergency Cardiovascular Care. Pediatrics. 2026;157(1):e2025074352. doi:10.1542/peds.2025-074352.";

/// Distribution licence: independently implemented from the published method.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published clinical scoring method - independently implemented from the primary literature",
    source_url: "https://collections.nlm.nih.gov/catalog/nlm:nlmuid-101584647X152-doc",
};

/// Gestational context relevant to interpretation of the 5-minute total.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GestationalContext {
    TermOrLatePreterm,
    EarlierPreterm,
    Unknown,
}

impl GestationalContext {
    fn slug(self) -> &'static str {
        match self {
            Self::TermOrLatePreterm => "term_or_late_preterm",
            Self::EarlierPreterm => "earlier_preterm",
            Self::Unknown => "unknown",
        }
    }
}

/// Heart rate observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeartRate {
    Absent,
    #[serde(rename = "below_100")]
    Below100,
    #[serde(rename = "at_least_100")]
    AtLeast100,
}

impl HeartRate {
    fn points(self) -> u8 {
        match self {
            Self::Absent => 0,
            Self::Below100 => 1,
            Self::AtLeast100 => 2,
        }
    }
}

/// Respiratory effort observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RespiratoryEffort {
    Absent,
    SlowOrIrregular,
    GoodWithVigorousCry,
}

impl RespiratoryEffort {
    fn points(self) -> u8 {
        match self {
            Self::Absent => 0,
            Self::SlowOrIrregular => 1,
            Self::GoodWithVigorousCry => 2,
        }
    }
}

/// Muscle tone observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MuscleTone {
    Flaccid,
    SomeFlexion,
    ActiveMotion,
}

impl MuscleTone {
    fn points(self) -> u8 {
        match self {
            Self::Flaccid => 0,
            Self::SomeFlexion => 1,
            Self::ActiveMotion => 2,
        }
    }
}

/// Reflex response to stimulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReflexIrritability {
    NoResponse,
    Grimace,
    CoughSneezeOrActiveWithdrawal,
}

impl ReflexIrritability {
    fn points(self) -> u8 {
        match self {
            Self::NoResponse => 0,
            Self::Grimace => 1,
            Self::CoughSneezeOrActiveWithdrawal => 2,
        }
    }
}

/// Visual appearance (colour) observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Appearance {
    BlueOrPale,
    PinkBodyBlueExtremities,
    CompletelyPink,
}

impl Appearance {
    fn points(self) -> u8 {
        match self {
            Self::BlueOrPale => 0,
            Self::PinkBodyBlueExtremities => 1,
            Self::CompletelyPink => 2,
        }
    }
}

/// One complete Apgar observation set at a specified minute after birth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApgarInput {
    /// Completed minute after birth: 1, 5, 10, 15, or 20.
    pub minute_after_birth: u8,
    /// Whether assisted resuscitation was in progress during this observation set.
    pub assessment_during_resuscitation: bool,
    /// Gestational context used only to determine whether a 5-minute band applies.
    pub gestational_context: GestationalContext,
    pub heart_rate: HeartRate,
    pub respiratory_effort: RespiratoryEffort,
    pub muscle_tone: MuscleTone,
    pub reflex_irritability: ReflexIrritability,
    pub appearance: Appearance,
}

/// Validated descriptive band for a 5-minute term or late-preterm total.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FiveMinuteBand {
    Low,
    ModeratelyAbnormal,
    Reassuring,
}

impl FiveMinuteBand {
    fn from_total(total: u8) -> Self {
        match total {
            0..=3 => Self::Low,
            4..=6 => Self::ModeratelyAbnormal,
            _ => Self::Reassuring,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::ModeratelyAbnormal => "moderately abnormal",
            Self::Reassuring => "reassuring",
        }
    }
}

/// Computed Apgar total with every component retained for audit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApgarOutcome {
    pub heart_rate_points: u8,
    pub respiratory_effort_points: u8,
    pub muscle_tone_points: u8,
    pub reflex_irritability_points: u8,
    pub appearance_points: u8,
    /// Total score, 0-10.
    pub total: u8,
    /// Present only for the 5-minute term or late-preterm context.
    pub five_minute_band: Option<FiveMinuteBand>,
    pub interpretation: String,
}

fn render_interpretation(
    input: &ApgarInput,
    total: u8,
    five_minute_band: Option<FiveMinuteBand>,
) -> String {
    let mut statements = vec![format!(
        "Apgar score {total}/10 at {} minute{} after birth.",
        input.minute_after_birth,
        if input.minute_after_birth == 1 {
            ""
        } else {
            "s"
        }
    )];

    if let Some(band) = five_minute_band {
        statements.push(format!(
            "For a term or late-preterm infant at 5 minutes, this is in the {} band.",
            band.label()
        ));
    } else {
        statements.push(
            "No validated 5-minute term or late-preterm interpretation band applies to this assessment time and gestational context."
                .into(),
        );
    }

    if input.minute_after_birth == 1 {
        statements.push("The routine 5-minute Apgar score is still required.".into());
    }
    if input.minute_after_birth == 5 && total < 7 {
        statements.push(
            "Published guidance states that scoring should continue at 5-minute intervals through 20 minutes when the 5-minute score is below 7."
                .into(),
        );
    }
    if input.minute_after_birth == 5 && total <= 5 {
        statements.push(
            "AAP/ACOG guidance recommends obtaining an umbilical arterial blood gas from a clamped section of cord, if possible, when the 5-minute score is 5 or less."
                .into(),
        );
    }
    if input.assessment_during_resuscitation {
        statements.push(
            "This score was assessed during resuscitation: assisted and unassisted scores are not equivalent, and concurrent interventions need separate documentation."
                .into(),
        );
    }

    statements.push(
        "The Apgar score does not determine the need for or steps of initial resuscitation, does not diagnose asphyxia, and is not an individual outcome predictor. It describes one observation set and does not by itself prescribe treatment."
            .into(),
    );
    statements.join(" ")
}

/// Pure scoring of one complete observation set.
pub fn compute(input: &ApgarInput) -> Result<ApgarOutcome, CalcError> {
    if !matches!(input.minute_after_birth, 1 | 5 | 10 | 15 | 20) {
        return Err(CalcError::InvalidInput(
            "minute_after_birth must be one of 1, 5, 10, 15, or 20".into(),
        ));
    }

    let heart_rate_points = input.heart_rate.points();
    let respiratory_effort_points = input.respiratory_effort.points();
    let muscle_tone_points = input.muscle_tone.points();
    let reflex_irritability_points = input.reflex_irritability.points();
    let appearance_points = input.appearance.points();
    let total = heart_rate_points
        + respiratory_effort_points
        + muscle_tone_points
        + reflex_irritability_points
        + appearance_points;
    let five_minute_band = (input.minute_after_birth == 5
        && input.gestational_context == GestationalContext::TermOrLatePreterm)
        .then(|| FiveMinuteBand::from_total(total));

    Ok(ApgarOutcome {
        heart_rate_points,
        respiratory_effort_points,
        muscle_tone_points,
        reflex_irritability_points,
        appearance_points,
        total,
        five_minute_band,
        interpretation: render_interpretation(input, total, five_minute_band),
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &ApgarInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert("minute_after_birth".into(), json!(input.minute_after_birth));
    working.insert("time_unit".into(), json!("completed minutes after birth"));
    working.insert(
        "assessment_during_resuscitation".into(),
        json!(input.assessment_during_resuscitation),
    );
    working.insert(
        "gestational_context".into(),
        json!(input.gestational_context.slug()),
    );
    working.insert("heart_rate_points".into(), json!(outcome.heart_rate_points));
    working.insert(
        "respiratory_effort_points".into(),
        json!(outcome.respiratory_effort_points),
    );
    working.insert(
        "muscle_tone_points".into(),
        json!(outcome.muscle_tone_points),
    );
    working.insert(
        "reflex_irritability_points".into(),
        json!(outcome.reflex_irritability_points),
    );
    working.insert("appearance_points".into(), json!(outcome.appearance_points));
    working.insert("total_score".into(), json!(outcome.total));
    working.insert("maximum_score".into(), json!(10));
    working.insert("five_minute_band".into(), json!(outcome.five_minute_band));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.total),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn input_schema() -> Value {
    let primary_source = json!({
        "citation": "Apgar V. Curr Res Anesth Analg. 1953;32(4):260-267. PMID:13083014.",
        "url": "https://collections.nlm.nih.gov/catalog/nlm:nlmuid-101584647X152-doc"
    });
    let guidance_source = json!({
        "citation": "AAP Committee on Fetus and Newborn; ACOG Committee on Obstetric Practice. The Apgar Score. Pediatrics. 2015;136(4):819-822.",
        "url": "https://doi.org/10.1542/peds.2015-2651"
    });

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "ApgarInput",
        "description": "One complete conventional Apgar observation set. All five signs must describe the same infant at the same completed minute after birth. The score documents clinical status and response to resuscitation; it does not determine initial resuscitation, diagnose asphyxia, predict an individual outcome, or prescribe treatment.",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "minute_after_birth", "assessment_during_resuscitation", "gestational_context",
            "heart_rate", "respiratory_effort", "muscle_tone", "reflex_irritability",
            "appearance"
        ],
        "properties": {
            "minute_after_birth": {
                "type": "integer",
                "enum": [1, 5, 10, 15, 20],
                "description": "Completed minutes after birth for this single observation set; unit: minutes. Routine scores are recorded at 1 and 5 minutes, then at 5-minute intervals through 20 minutes when the 5-minute score is below 7",
                "definition": {
                    "concept": "Apgar assessment time",
                    "statement": "Select the completed minute after birth at which all five signs in this observation set were assessed.",
                    "excludes": ["Do not combine signs observed at different times", "Do not enter clock time or minutes since resuscitation began"],
                    "caveats": "A 1-minute result does not replace the routine 5-minute score. Only the 5-minute term or late-preterm result receives the cited descriptive band.",
                    "source": guidance_source,
                    "status": "draft"
                }
            },
            "assessment_during_resuscitation": {
                "type": "boolean",
                "description": "True when assisted resuscitative interventions were in progress during this observation set; not scored, but required because assisted and unassisted Apgar scores are not equivalent",
                "definition": {
                    "concept": "Concurrent resuscitation",
                    "statement": "Record whether this observation set was made while the infant was receiving assisted resuscitative intervention.",
                    "includes": ["Positive-pressure ventilation", "Continuous positive airway pressure", "Endotracheal intubation", "Chest compressions", "Resuscitation medication"],
                    "excludes": ["Routine drying, warming, positioning, or observation without assisted intervention"],
                    "caveats": "Document each concurrent intervention separately; this boolean does not encode which intervention occurred or convert the score into an expanded Apgar form.",
                    "source": guidance_source,
                    "status": "draft"
                }
            },
            "gestational_context": {
                "type": "string",
                "enum": ["term_or_late_preterm", "earlier_preterm", "unknown"],
                "description": "Gestational context for interpretation only: term_or_late_preterm, earlier_preterm, or unknown. Prematurity can lower tone, colour, and reflex scores without asphyxia",
                "definition": {
                    "concept": "Gestational context for Apgar interpretation",
                    "statement": "Select term_or_late_preterm only when that context is established; otherwise select earlier_preterm or unknown.",
                    "excludes": ["Do not apply the term or late-preterm 5-minute bands to an earlier-preterm infant", "Do not assume gestational context when it is unknown"],
                    "caveats": "Gestational age affects several score components. The conventional score is not adjusted for prematurity.",
                    "source": guidance_source,
                    "status": "draft"
                }
            },
            "heart_rate": {
                "type": "string",
                "enum": ["absent", "below_100", "at_least_100"],
                "description": "Heart rate in beats per minute at the assessment: absent=0, below_100=1, at_least_100=2 points",
                "definition": {
                    "concept": "Apgar heart rate",
                    "statement": "Classify the heart rate observed in this assessment set as absent, below 100 beats/min, or at least 100 beats/min.",
                    "excludes": ["Do not infer heart rate from colour or respiratory effort"],
                    "caveats": "This category records a heart-rate band, not the measured numeric heart rate.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "respiratory_effort": {
                "type": "string",
                "enum": ["absent", "slow_or_irregular", "good_with_vigorous_cry"],
                "description": "Respiratory effort at the assessment: absent=0, slow_or_irregular=1, good_with_vigorous_cry=2 points",
                "definition": {
                    "concept": "Apgar respiratory effort",
                    "statement": "Classify spontaneous respiratory effort and cry observed in this assessment set.",
                    "excludes": ["Do not score ventilator-delivered breaths as spontaneous vigorous respiratory effort"],
                    "caveats": "Record concurrent assisted ventilation separately with assessment_during_resuscitation and in the clinical record.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "muscle_tone": {
                "type": "string",
                "enum": ["flaccid", "some_flexion", "active_motion"],
                "description": "Muscle tone at the assessment: flaccid=0, some_flexion=1, active_motion=2 points",
                "definition": {
                    "concept": "Apgar muscle tone",
                    "statement": "Classify observed tone as flaccid, some flexion of extremities, or active motion.",
                    "caveats": "Tone is partly subjective and is affected by gestational maturity, maternal medication, and neurologic or neuromuscular conditions.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "reflex_irritability": {
                "type": "string",
                "enum": ["no_response", "grimace", "cough_sneeze_or_active_withdrawal"],
                "description": "Response to stimulation at the assessment: no_response=0, grimace=1, cough_sneeze_or_active_withdrawal=2 points",
                "definition": {
                    "concept": "Apgar reflex irritability",
                    "statement": "Classify the infant's response to stimulation as no response, grimace, or cough, sneeze, or active withdrawal.",
                    "excludes": ["Do not infer reflex response from spontaneous movement without stimulation"],
                    "caveats": "Reflex response is partly subjective and can be affected by gestational maturity and maternal medication.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "appearance": {
                "type": "string",
                "enum": ["blue_or_pale", "pink_body_blue_extremities", "completely_pink"],
                "description": "Visual colour at the assessment: blue_or_pale=0, pink_body_blue_extremities=1, completely_pink=2 points. Visual colour is subjective, is affected by skin pigmentation, and is not oxygen saturation",
                "definition": {
                    "concept": "Apgar appearance (visual colour)",
                    "statement": "Classify visual colour as blue or pale, pink body with blue extremities, or completely pink.",
                    "excludes": ["Do not substitute pulse-oximeter oxygen saturation for this conventional visual category", "Do not infer oxygen saturation from visible colour"],
                    "caveats": "Visual colour assessment is subjective, is affected by skin pigmentation and lighting, and is not a measurement of oxygen saturation.",
                    "source": primary_source,
                    "status": "draft"
                }
            }
        }
    })
}

/// Dynamic calculator implementation.
pub struct Apgar;

impl Calculator for Apgar {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Apgar Score"
    }

    fn description(&self) -> &'static str {
        "Scores one newborn observation set from heart rate, respiratory effort, muscle tone, reflex irritability, and appearance."
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
        let parsed: ApgarInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::SupportedLocale;

    fn all_zero() -> ApgarInput {
        ApgarInput {
            minute_after_birth: 5,
            assessment_during_resuscitation: false,
            gestational_context: GestationalContext::TermOrLatePreterm,
            heart_rate: HeartRate::Absent,
            respiratory_effort: RespiratoryEffort::Absent,
            muscle_tone: MuscleTone::Flaccid,
            reflex_irritability: ReflexIrritability::NoResponse,
            appearance: Appearance::BlueOrPale,
        }
    }

    fn all_two() -> ApgarInput {
        ApgarInput {
            heart_rate: HeartRate::AtLeast100,
            respiratory_effort: RespiratoryEffort::GoodWithVigorousCry,
            muscle_tone: MuscleTone::ActiveMotion,
            reflex_irritability: ReflexIrritability::CoughSneezeOrActiveWithdrawal,
            appearance: Appearance::CompletelyPink,
            ..all_zero()
        }
    }

    fn input_with_total(total: u8) -> ApgarInput {
        let point_values = [
            total.min(2),
            total.saturating_sub(2).min(2),
            total.saturating_sub(4).min(2),
            total.saturating_sub(6).min(2),
            total.saturating_sub(8).min(2),
        ];
        ApgarInput {
            heart_rate: match point_values[0] {
                0 => HeartRate::Absent,
                1 => HeartRate::Below100,
                _ => HeartRate::AtLeast100,
            },
            respiratory_effort: match point_values[1] {
                0 => RespiratoryEffort::Absent,
                1 => RespiratoryEffort::SlowOrIrregular,
                _ => RespiratoryEffort::GoodWithVigorousCry,
            },
            muscle_tone: match point_values[2] {
                0 => MuscleTone::Flaccid,
                1 => MuscleTone::SomeFlexion,
                _ => MuscleTone::ActiveMotion,
            },
            reflex_irritability: match point_values[3] {
                0 => ReflexIrritability::NoResponse,
                1 => ReflexIrritability::Grimace,
                _ => ReflexIrritability::CoughSneezeOrActiveWithdrawal,
            },
            appearance: match point_values[4] {
                0 => Appearance::BlueOrPale,
                1 => Appearance::PinkBodyBlueExtremities,
                _ => Appearance::CompletelyPink,
            },
            ..all_zero()
        }
    }

    #[test]
    fn all_zero_and_all_two_vectors_score_zero_and_ten() {
        let minimum = compute(&all_zero()).unwrap();
        assert_eq!(minimum.total, 0);
        assert_eq!(minimum.five_minute_band, Some(FiveMinuteBand::Low));

        let maximum = compute(&all_two()).unwrap();
        assert_eq!(maximum.total, 10);
        assert_eq!(maximum.five_minute_band, Some(FiveMinuteBand::Reassuring));
    }

    #[test]
    fn every_category_maps_to_the_exact_published_points() {
        for (value, expected) in [
            (HeartRate::Absent, 0),
            (HeartRate::Below100, 1),
            (HeartRate::AtLeast100, 2),
        ] {
            assert_eq!(value.points(), expected);
        }
        for (value, expected) in [
            (RespiratoryEffort::Absent, 0),
            (RespiratoryEffort::SlowOrIrregular, 1),
            (RespiratoryEffort::GoodWithVigorousCry, 2),
        ] {
            assert_eq!(value.points(), expected);
        }
        for (value, expected) in [
            (MuscleTone::Flaccid, 0),
            (MuscleTone::SomeFlexion, 1),
            (MuscleTone::ActiveMotion, 2),
        ] {
            assert_eq!(value.points(), expected);
        }
        for (value, expected) in [
            (ReflexIrritability::NoResponse, 0),
            (ReflexIrritability::Grimace, 1),
            (ReflexIrritability::CoughSneezeOrActiveWithdrawal, 2),
        ] {
            assert_eq!(value.points(), expected);
        }
        for (value, expected) in [
            (Appearance::BlueOrPale, 0),
            (Appearance::PinkBodyBlueExtremities, 1),
            (Appearance::CompletelyPink, 2),
        ] {
            assert_eq!(value.points(), expected);
        }
    }

    #[test]
    fn categorical_wire_values_are_exact() {
        let value = serde_json::to_value(all_two()).unwrap();
        assert_eq!(value["gestational_context"], json!("term_or_late_preterm"));
        assert_eq!(value["heart_rate"], json!("at_least_100"));
        assert_eq!(value["respiratory_effort"], json!("good_with_vigorous_cry"));
        assert_eq!(value["muscle_tone"], json!("active_motion"));
        assert_eq!(
            value["reflex_irritability"],
            json!("cough_sneeze_or_active_withdrawal")
        );
        assert_eq!(value["appearance"], json!("completely_pink"));

        let below_100 = ApgarInput {
            heart_rate: HeartRate::Below100,
            ..all_zero()
        };
        assert_eq!(
            serde_json::to_value(below_100).unwrap()["heart_rate"],
            json!("below_100")
        );
    }

    #[test]
    fn five_minute_band_boundaries_are_exact() {
        for (total, expected) in [
            (3, FiveMinuteBand::Low),
            (4, FiveMinuteBand::ModeratelyAbnormal),
            (6, FiveMinuteBand::ModeratelyAbnormal),
            (7, FiveMinuteBand::Reassuring),
        ] {
            let outcome = compute(&input_with_total(total)).unwrap();
            assert_eq!(outcome.total, total);
            assert_eq!(outcome.five_minute_band, Some(expected));
        }
    }

    #[test]
    fn one_minute_has_no_band_and_requires_the_routine_five_minute_score() {
        let input = ApgarInput {
            minute_after_birth: 1,
            ..all_two()
        };
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.five_minute_band, None);
        assert!(outcome.interpretation.contains("No validated"));
        assert!(
            outcome
                .interpretation
                .contains("5-minute Apgar score is still required")
        );
    }

    #[test]
    fn low_five_minute_scores_carry_follow_up_documentation_statements() {
        let six = compute(&input_with_total(6)).unwrap();
        assert!(six.interpretation.contains("through 20 minutes"));
        assert!(!six.interpretation.contains("umbilical arterial blood gas"));

        let five = compute(&input_with_total(5)).unwrap();
        assert!(five.interpretation.contains("through 20 minutes"));
        assert!(five.interpretation.contains("umbilical arterial blood gas"));
        assert!(five.interpretation.contains("AAP/ACOG"));
    }

    #[test]
    fn term_band_is_not_applied_to_other_gestational_contexts() {
        for gestational_context in [
            GestationalContext::EarlierPreterm,
            GestationalContext::Unknown,
        ] {
            let input = ApgarInput {
                gestational_context,
                ..all_two()
            };
            let outcome = compute(&input).unwrap();
            assert_eq!(outcome.five_minute_band, None);
            assert!(outcome.interpretation.contains("No validated"));
        }
    }

    #[test]
    fn resuscitation_context_is_explicitly_qualified() {
        let input = ApgarInput {
            assessment_during_resuscitation: true,
            ..all_two()
        };
        let interpretation = compute(&input).unwrap().interpretation;
        assert!(interpretation.contains("assisted and unassisted scores are not equivalent"));
        assert!(interpretation.contains("interventions need separate documentation"));
    }

    #[test]
    fn every_interpretation_carries_core_safety_limits() {
        let interpretation = compute(&all_two()).unwrap().interpretation;
        assert!(interpretation.contains("does not determine"));
        assert!(interpretation.contains("does not diagnose asphyxia"));
        assert!(interpretation.contains("not an individual outcome predictor"));
        assert!(interpretation.contains("does not by itself prescribe treatment"));
    }

    #[test]
    fn dynamic_calculation_matches_the_typed_contract() {
        let input = input_with_total(7);
        let dynamic = Apgar
            .calculate(&serde_json::to_value(input).unwrap())
            .unwrap();
        assert_eq!(dynamic, build_response(&input).unwrap());
        assert_eq!(dynamic.working["heart_rate_points"], json!(2));
        assert_eq!(dynamic.working["total_score"], json!(7));
    }

    #[test]
    fn rejects_invalid_minute_enum_and_unknown_fields() {
        let mut invalid_minute = serde_json::to_value(all_two()).unwrap();
        invalid_minute["minute_after_birth"] = json!(2);
        assert!(Apgar.calculate(&invalid_minute).is_err());

        let mut invalid_enum = serde_json::to_value(all_two()).unwrap();
        invalid_enum["heart_rate"] = json!("over_100");
        assert!(Apgar.calculate(&invalid_enum).is_err());

        let mut unknown_field = serde_json::to_value(all_two()).unwrap();
        unknown_field["oxygen_saturation"] = json!(95);
        assert!(Apgar.calculate(&unknown_field).is_err());

        let typed_invalid_minute = ApgarInput {
            minute_after_birth: 2,
            ..all_two()
        };
        assert!(compute(&typed_invalid_minute).is_err());
    }

    #[test]
    fn schema_is_closed_required_and_carries_safety_and_unit_semantics() {
        let schema = Apgar.input_schema();
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"].as_array().unwrap().len(), 8);
        assert_eq!(
            schema["properties"]["minute_after_birth"]["enum"],
            json!([1, 5, 10, 15, 20])
        );
        assert!(
            schema["properties"]["minute_after_birth"]["description"]
                .as_str()
                .unwrap()
                .contains("unit: minutes")
        );
        let appearance = &schema["properties"]["appearance"];
        assert!(
            appearance["description"]
                .as_str()
                .unwrap()
                .contains("skin pigmentation")
        );
        assert!(
            appearance["description"]
                .as_str()
                .unwrap()
                .contains("not oxygen saturation")
        );
        assert!(
            schema["description"]
                .as_str()
                .unwrap()
                .contains("does not determine initial resuscitation")
        );
        assert!(
            schema["properties"]
                .as_object()
                .unwrap()
                .values()
                .all(|property| {
                    property["description"].is_string()
                        && property["definition"]["statement"].is_string()
                })
        );
    }

    #[test]
    fn calculate_for_english_records_the_content_locale() {
        let response = Apgar
            .calculate_for(
                &serde_json::to_value(all_two()).unwrap(),
                SupportedLocale::En,
            )
            .unwrap();
        assert_eq!(response.working["content_locale"], json!("en"));
    }
}
