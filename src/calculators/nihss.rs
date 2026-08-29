// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! NIH Stroke Scale (NIHSS).
//!
//! This implements the current standard adult NINDS scoring rubric without
//! redistributing visual stimuli or proprietary training and certification
//! materials. Unscored (`UN`) findings are never
//! converted to zero: as a project safety policy, if any entry is `UN`, no
//! NIHSS total is reported.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "nihss";
pub const REFERENCE: &str = "National Institute of Neurological Disorders and Stroke. NIH Stroke Scale. Updated February 2024. https://www.ninds.nih.gov/health-information/stroke/assess-and-treat/nih-stroke-scale. Brott T, Adams HP Jr, Olinger CP, et al. Measurements of acute cerebral infarction: a clinical examination scale. Stroke. 1989;20(7):864-870. doi:10.1161/01.STR.20.7.864. Hills NK, Josephson SA, Lyden PD, Johnston SC. Is the NIHSS certification process too lenient? Cerebrovasc Dis. 2009;27(5):426-432. doi:10.1159/000209237.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "NINDS-prepared information is public domain and may be freely copied; scoring method independently implemented without third-party visual or training materials",
    source_url: "https://www.ninds.nih.gov/publications/publications-help",
};

const LIMITATIONS: &str = "NIHSS measures deficits detected by the standard examination; it does not diagnose or exclude stroke, measure every disabling deficit, or determine reperfusion treatment. A score of 0 can occur with imaging-confirmed stroke, particularly posterior-circulation stroke. The scale can underrepresent gait or truncal ataxia, vertigo, diplopia, dysphagia, and some right-hemisphere deficits. Treatment decisions require symptom timing, whether a deficit is disabling, imaging, contraindications, vascular findings, and specialist assessment. Use the separately validated PedNIHSS for children. Standardised training and correct administration remain essential.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentContext {
    ClinicianAdministeredStandardAdultNihssUsingAuthorizedScaleMaterials,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LevelOfConsciousness {
    Alert,
    ArousableByMinorStimulation,
    RequiresRepeatedOrStrongStimulation,
    ReflexResponsesOrUnresponsive,
}

impl LevelOfConsciousness {
    fn points(self) -> u8 {
        match self {
            Self::Alert => 0,
            Self::ArousableByMinorStimulation => 1,
            Self::RequiresRepeatedOrStrongStimulation => 2,
            Self::ReflexResponsesOrUnresponsive => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocQuestions {
    BothCorrect,
    OneCorrect,
    NonAphasicCommunicationBarrier,
    NeitherCorrectOrAphasicOrStuporous,
}

impl LocQuestions {
    fn points(self) -> u8 {
        match self {
            Self::BothCorrect => 0,
            Self::OneCorrect | Self::NonAphasicCommunicationBarrier => 1,
            Self::NeitherCorrectOrAphasicOrStuporous => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocCommands {
    BothCorrect,
    OneCorrect,
    NeitherCorrect,
}

impl LocCommands {
    fn points(self) -> u8 {
        match self {
            Self::BothCorrect => 0,
            Self::OneCorrect => 1,
            Self::NeitherCorrect => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BestGaze {
    Normal,
    PartialGazePalsy,
    ForcedDeviationOrTotalParesis,
}

impl BestGaze {
    fn points(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::PartialGazePalsy => 1,
            Self::ForcedDeviationOrTotalParesis => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualFields {
    NoVisualLoss,
    PartialHemianopia,
    CompleteHemianopia,
    BilateralHemianopiaOrBlindness,
}

impl VisualFields {
    fn points(self) -> u8 {
        match self {
            Self::NoVisualLoss => 0,
            Self::PartialHemianopia => 1,
            Self::CompleteHemianopia => 2,
            Self::BilateralHemianopiaOrBlindness => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FacialPalsy {
    NormalSymmetry,
    MinorParalysis,
    PartialLowerFaceParalysis,
    CompleteOneOrBothSides,
}

impl FacialPalsy {
    fn points(self) -> u8 {
        match self {
            Self::NormalSymmetry => 0,
            Self::MinorParalysis => 1,
            Self::PartialLowerFaceParalysis => 2,
            Self::CompleteOneOrBothSides => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotorArm {
    NoDriftForTenSeconds,
    DriftWithoutHittingSupport,
    SomeEffortAgainstGravity,
    NoEffortAgainstGravity,
    NoMovement,
    UntestableAmputationOrShoulderFusion,
}

impl MotorArm {
    fn points(self) -> Option<u8> {
        match self {
            Self::NoDriftForTenSeconds => Some(0),
            Self::DriftWithoutHittingSupport => Some(1),
            Self::SomeEffortAgainstGravity => Some(2),
            Self::NoEffortAgainstGravity => Some(3),
            Self::NoMovement => Some(4),
            Self::UntestableAmputationOrShoulderFusion => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotorLeg {
    NoDriftForFiveSeconds,
    DriftWithoutHittingBed,
    SomeEffortAgainstGravity,
    NoEffortAgainstGravity,
    NoMovement,
    UntestableAmputationOrHipFusion,
}

impl MotorLeg {
    fn points(self) -> Option<u8> {
        match self {
            Self::NoDriftForFiveSeconds => Some(0),
            Self::DriftWithoutHittingBed => Some(1),
            Self::SomeEffortAgainstGravity => Some(2),
            Self::NoEffortAgainstGravity => Some(3),
            Self::NoMovement => Some(4),
            Self::UntestableAmputationOrHipFusion => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LimbAtaxia {
    Absent,
    PresentInOneLimb,
    PresentInTwoLimbs,
    UntestableAmputationOrJointFusion,
}

impl LimbAtaxia {
    fn points(self) -> Option<u8> {
        match self {
            Self::Absent => Some(0),
            Self::PresentInOneLimb => Some(1),
            Self::PresentInTwoLimbs => Some(2),
            Self::UntestableAmputationOrJointFusion => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sensory {
    Normal,
    MildToModerateLoss,
    SevereOrTotalLoss,
}

impl Sensory {
    fn points(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::MildToModerateLoss => 1,
            Self::SevereOrTotalLoss => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BestLanguage {
    NoAphasia,
    MildToModerateAphasia,
    SevereAphasia,
    MuteOrGlobalAphasia,
}

impl BestLanguage {
    fn points(self) -> u8 {
        match self {
            Self::NoAphasia => 0,
            Self::MildToModerateAphasia => 1,
            Self::SevereAphasia => 2,
            Self::MuteOrGlobalAphasia => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dysarthria {
    Normal,
    MildToModerate,
    SevereOrAnarthric,
    UntestableIntubationOrPhysicalBarrier,
}

impl Dysarthria {
    fn points(self) -> Option<u8> {
        match self {
            Self::Normal => Some(0),
            Self::MildToModerate => Some(1),
            Self::SevereOrAnarthric => Some(2),
            Self::UntestableIntubationOrPhysicalBarrier => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtinctionInattention {
    None,
    OneModality,
    ProfoundOrMoreThanOneModality,
}

impl ExtinctionInattention {
    fn points(self) -> u8 {
        match self {
            Self::None => 0,
            Self::OneModality => 1,
            Self::ProfoundOrMoreThanOneModality => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NihssUnscoredExplanations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motor_arm_left: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motor_arm_right: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motor_leg_left: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motor_leg_right: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limb_ataxia: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dysarthria: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NihssInput {
    pub assessment_context: AssessmentContext,
    pub level_of_consciousness: LevelOfConsciousness,
    pub loc_questions: LocQuestions,
    pub loc_commands: LocCommands,
    pub best_gaze: BestGaze,
    pub visual_fields: VisualFields,
    pub facial_palsy: FacialPalsy,
    pub motor_arm_left: MotorArm,
    pub motor_arm_right: MotorArm,
    pub motor_leg_left: MotorLeg,
    pub motor_leg_right: MotorLeg,
    pub limb_ataxia: LimbAtaxia,
    pub sensory: Sensory,
    pub best_language: BestLanguage,
    pub dysarthria: Dysarthria,
    pub extinction_inattention: ExtinctionInattention,
    /// Required when an item is `UN`, with one explanation for each such item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unscored_explanations: Option<NihssUnscoredExplanations>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NihssPoints {
    pub level_of_consciousness: Option<u8>,
    pub loc_questions: Option<u8>,
    pub loc_commands: Option<u8>,
    pub best_gaze: Option<u8>,
    pub visual_fields: Option<u8>,
    pub facial_palsy: Option<u8>,
    pub motor_arm_left: Option<u8>,
    pub motor_arm_right: Option<u8>,
    pub motor_leg_left: Option<u8>,
    pub motor_leg_right: Option<u8>,
    pub limb_ataxia: Option<u8>,
    pub sensory: Option<u8>,
    pub best_language: Option<u8>,
    pub dysarthria: Option<u8>,
    pub extinction_inattention: Option<u8>,
}

impl NihssPoints {
    fn entries(self) -> [(&'static str, Option<u8>); 15] {
        [
            ("level_of_consciousness", self.level_of_consciousness),
            ("loc_questions", self.loc_questions),
            ("loc_commands", self.loc_commands),
            ("best_gaze", self.best_gaze),
            ("visual_fields", self.visual_fields),
            ("facial_palsy", self.facial_palsy),
            ("motor_arm_left", self.motor_arm_left),
            ("motor_arm_right", self.motor_arm_right),
            ("motor_leg_left", self.motor_leg_left),
            ("motor_leg_right", self.motor_leg_right),
            ("limb_ataxia", self.limb_ataxia),
            ("sensory", self.sensory),
            ("best_language", self.best_language),
            ("dysarthria", self.dysarthria),
            ("extinction_inattention", self.extinction_inattention),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NihssOutcome {
    pub points: NihssPoints,
    pub total_score: Option<u8>,
    pub partial_scored_item_sum: u8,
    pub unscored_items: Vec<&'static str>,
    pub interpretation: String,
}

pub fn compute(input: &NihssInput) -> Result<NihssOutcome, CalcError> {
    if input.level_of_consciousness == LevelOfConsciousness::ReflexResponsesOrUnresponsive {
        if input.loc_questions != LocQuestions::NeitherCorrectOrAphasicOrStuporous
            || input.loc_commands != LocCommands::NeitherCorrect
        {
            return Err(CalcError::InvalidInput(
                "reflex_responses_or_unresponsive requires loc_questions=neither_correct_or_aphasic_or_stuporous and loc_commands=neither_correct"
                    .into(),
            ));
        }
        if !matches!(
            input.limb_ataxia,
            LimbAtaxia::Absent | LimbAtaxia::UntestableAmputationOrJointFusion
        ) {
            return Err(CalcError::InvalidInput(
                "limb_ataxia must be absent when level_of_consciousness is reflex_responses_or_unresponsive unless amputation or joint fusion independently requires UN"
                    .into(),
            ));
        }
        if input.sensory != Sensory::SevereOrTotalLoss {
            return Err(CalcError::InvalidInput(
                "sensory must be severe_or_total_loss when level_of_consciousness is reflex_responses_or_unresponsive, because the official NIHSS assigns item 8 a score of 2".into(),
            ));
        }
        if input.best_language != BestLanguage::MuteOrGlobalAphasia {
            return Err(CalcError::InvalidInput(
                "best_language must be mute_or_global_aphasia when level_of_consciousness is reflex_responses_or_unresponsive, because the official NIHSS assigns item 9 a score of 3".into(),
            ));
        }
        if !matches!(
            input.motor_arm_left,
            MotorArm::NoEffortAgainstGravity
                | MotorArm::NoMovement
                | MotorArm::UntestableAmputationOrShoulderFusion
        ) || !matches!(
            input.motor_arm_right,
            MotorArm::NoEffortAgainstGravity
                | MotorArm::NoMovement
                | MotorArm::UntestableAmputationOrShoulderFusion
        ) || !matches!(
            input.motor_leg_left,
            MotorLeg::NoEffortAgainstGravity
                | MotorLeg::NoMovement
                | MotorLeg::UntestableAmputationOrHipFusion
        ) || !matches!(
            input.motor_leg_right,
            MotorLeg::NoEffortAgainstGravity
                | MotorLeg::NoMovement
                | MotorLeg::UntestableAmputationOrHipFusion
        ) {
            return Err(CalcError::InvalidInput(
                "reflex_responses_or_unresponsive requires each motor limb to be no_effort_against_gravity, no_movement, or source-permitted UN because voluntary limb holding is incompatible with item 1a score 3"
                    .into(),
            ));
        }
        if !matches!(
            input.dysarthria,
            Dysarthria::SevereOrAnarthric | Dysarthria::UntestableIntubationOrPhysicalBarrier
        ) {
            return Err(CalcError::InvalidInput(
                "reflex_responses_or_unresponsive requires dysarthria=severe_or_anarthric or source-permitted UN"
                    .into(),
            ));
        }
    }

    let limbs_capable_of_demonstrating_ataxia = [
        !matches!(
            input.motor_arm_left,
            MotorArm::NoMovement | MotorArm::UntestableAmputationOrShoulderFusion
        ),
        !matches!(
            input.motor_arm_right,
            MotorArm::NoMovement | MotorArm::UntestableAmputationOrShoulderFusion
        ),
        !matches!(
            input.motor_leg_left,
            MotorLeg::NoMovement | MotorLeg::UntestableAmputationOrHipFusion
        ),
        !matches!(
            input.motor_leg_right,
            MotorLeg::NoMovement | MotorLeg::UntestableAmputationOrHipFusion
        ),
    ]
    .into_iter()
    .filter(|capable| *capable)
    .count();
    let demonstrated_ataxic_limbs = match input.limb_ataxia {
        LimbAtaxia::PresentInOneLimb => 1,
        LimbAtaxia::PresentInTwoLimbs => 2,
        LimbAtaxia::Absent | LimbAtaxia::UntestableAmputationOrJointFusion => 0,
    };
    if demonstrated_ataxic_limbs > limbs_capable_of_demonstrating_ataxia {
        return Err(CalcError::InvalidInput(
            "limb_ataxia cannot be demonstrated in more non-paralysed, assessable limbs than the motor entries permit".into(),
        ));
    }

    let points = NihssPoints {
        level_of_consciousness: Some(input.level_of_consciousness.points()),
        loc_questions: Some(input.loc_questions.points()),
        loc_commands: Some(input.loc_commands.points()),
        best_gaze: Some(input.best_gaze.points()),
        visual_fields: Some(input.visual_fields.points()),
        facial_palsy: Some(input.facial_palsy.points()),
        motor_arm_left: input.motor_arm_left.points(),
        motor_arm_right: input.motor_arm_right.points(),
        motor_leg_left: input.motor_leg_left.points(),
        motor_leg_right: input.motor_leg_right.points(),
        limb_ataxia: input.limb_ataxia.points(),
        sensory: Some(input.sensory.points()),
        best_language: Some(input.best_language.points()),
        dysarthria: input.dysarthria.points(),
        extinction_inattention: Some(input.extinction_inattention.points()),
    };
    let entries = points.entries();
    let partial_scored_item_sum = entries.iter().filter_map(|(_, value)| *value).sum();
    let unscored_items: Vec<_> = entries
        .iter()
        .filter_map(|(name, value)| value.is_none().then_some(*name))
        .collect();
    let explanations = input.unscored_explanations.as_ref();
    let explanation_entries = [
        (
            "motor_arm_left",
            input.motor_arm_left.points().is_none(),
            explanations.and_then(|value| value.motor_arm_left.as_deref()),
        ),
        (
            "motor_arm_right",
            input.motor_arm_right.points().is_none(),
            explanations.and_then(|value| value.motor_arm_right.as_deref()),
        ),
        (
            "motor_leg_left",
            input.motor_leg_left.points().is_none(),
            explanations.and_then(|value| value.motor_leg_left.as_deref()),
        ),
        (
            "motor_leg_right",
            input.motor_leg_right.points().is_none(),
            explanations.and_then(|value| value.motor_leg_right.as_deref()),
        ),
        (
            "limb_ataxia",
            input.limb_ataxia.points().is_none(),
            explanations.and_then(|value| value.limb_ataxia.as_deref()),
        ),
        (
            "dysarthria",
            input.dysarthria.points().is_none(),
            explanations.and_then(|value| value.dysarthria.as_deref()),
        ),
    ];
    for (name, is_unscored, explanation) in explanation_entries {
        match (is_unscored, explanation) {
            (true, Some(value)) if !value.trim().is_empty() => {}
            (true, _) => {
                return Err(CalcError::InvalidInput(format!(
                    "unscored_explanations.{name} must document the physical barrier for this UN item"
                )));
            }
            (false, Some(value)) if value.trim().is_empty() => {
                return Err(CalcError::InvalidInput(format!(
                    "unscored_explanations.{name} must not be blank"
                )));
            }
            (false, Some(_)) => {}
            (false, None) => {}
        }
    }
    let total_score = unscored_items.is_empty().then_some(partial_scored_item_sum);
    let interpretation = match total_score {
        Some(total) => format!("NIHSS {total}/42. {LIMITATIONS}"),
        None => format!(
            "NIHSS total not calculable because these entries are officially unscored (UN): {}. The sum of scored entries is {partial_scored_item_sum}, but this is not a complete NIHSS total and must not be interpreted with total-score thresholds. {LIMITATIONS}",
            unscored_items.join(", ")
        ),
    };

    Ok(NihssOutcome {
        points,
        total_score,
        partial_scored_item_sum,
        unscored_items,
        interpretation,
    })
}

fn add_working<T: Serialize>(
    working: &mut Map<String, Value>,
    name: &str,
    value: T,
    points: Option<u8>,
) {
    working.insert(name.into(), json!(value));
    if let Some(points) = points {
        working.insert(format!("{name}_points"), json!(points));
    }
}

pub fn build_response(input: &NihssInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert("assessment_context".into(), json!(input.assessment_context));
    add_working(
        &mut working,
        "level_of_consciousness",
        input.level_of_consciousness,
        outcome.points.level_of_consciousness,
    );
    add_working(
        &mut working,
        "loc_questions",
        input.loc_questions,
        outcome.points.loc_questions,
    );
    add_working(
        &mut working,
        "loc_commands",
        input.loc_commands,
        outcome.points.loc_commands,
    );
    add_working(
        &mut working,
        "best_gaze",
        input.best_gaze,
        outcome.points.best_gaze,
    );
    add_working(
        &mut working,
        "visual_fields",
        input.visual_fields,
        outcome.points.visual_fields,
    );
    add_working(
        &mut working,
        "facial_palsy",
        input.facial_palsy,
        outcome.points.facial_palsy,
    );
    add_working(
        &mut working,
        "motor_arm_left",
        input.motor_arm_left,
        outcome.points.motor_arm_left,
    );
    add_working(
        &mut working,
        "motor_arm_right",
        input.motor_arm_right,
        outcome.points.motor_arm_right,
    );
    add_working(
        &mut working,
        "motor_leg_left",
        input.motor_leg_left,
        outcome.points.motor_leg_left,
    );
    add_working(
        &mut working,
        "motor_leg_right",
        input.motor_leg_right,
        outcome.points.motor_leg_right,
    );
    add_working(
        &mut working,
        "limb_ataxia",
        input.limb_ataxia,
        outcome.points.limb_ataxia,
    );
    add_working(
        &mut working,
        "sensory",
        input.sensory,
        outcome.points.sensory,
    );
    add_working(
        &mut working,
        "best_language",
        input.best_language,
        outcome.points.best_language,
    );
    add_working(
        &mut working,
        "dysarthria",
        input.dysarthria,
        outcome.points.dysarthria,
    );
    add_working(
        &mut working,
        "extinction_inattention",
        input.extinction_inattention,
        outcome.points.extinction_inattention,
    );
    working.insert(
        "partial_scored_item_sum".into(),
        json!(outcome.partial_scored_item_sum),
    );
    working.insert("unscored_items".into(), json!(outcome.unscored_items));
    if let Some(explanations) = &input.unscored_explanations {
        working.insert("unscored_explanations".into(), json!(explanations));
    }
    working.insert("maximum_complete_score".into(), json!(42));
    working.insert("limitations".into(), json!(LIMITATIONS));
    if let Some(total) = outcome.total_score {
        working.insert("total_score".into(), json!(total));
    }

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: outcome
            .total_score
            .map_or_else(|| json!("not_calculable"), |score| json!(score)),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn scored_property(
    concept: &str,
    description: &str,
    values: &[&str],
    caveats: &str,
    source: &Value,
) -> Value {
    json!({
        "type": "string",
        "enum": values,
        "description": description,
        "definition": {
            "concept": concept,
            "statement": description,
            "excludes": ["Caller-supplied points", "A guessed category", "Retrospective revision after moving to a later NIHSS item"],
            "caveats": caveats,
            "source": source,
            "status": "draft"
        }
    })
}

fn input_schema() -> Value {
    let source = json!({
        "citation": "National Institute of Neurological Disorders and Stroke. NIH Stroke Scale. Updated February 2024.",
        "url": "https://www.ninds.nih.gov/sites/default/files/documents/NIH-Stroke-Scale_updatedFeb2024_508.pdf"
    });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "NihssInput",
        "description": "Current standard adult NIH Stroke Scale. A trained clinician must administer the 15 scored entries in official order, record each response immediately without revising earlier items, score observed performance rather than presumed ability, and use an authorized version of the required language and visual scale materials. This package does not redistribute the stimuli or certification materials. UN is available only for the source-defined physical barriers. As a project safety policy, any UN entry prevents a complete NIHSS total.",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "assessment_context", "level_of_consciousness", "loc_questions", "loc_commands",
            "best_gaze", "visual_fields", "facial_palsy", "motor_arm_left", "motor_arm_right",
            "motor_leg_left", "motor_leg_right", "limb_ataxia", "sensory", "best_language",
            "dysarthria", "extinction_inattention"
        ],
        "properties": {
            "assessment_context": {
                "type": "string",
                "const": "clinician_administered_standard_adult_nihss_using_authorized_scale_materials",
                "description": "A trained clinician is administering the standard adult NIHSS using an authorized version of the scale materials, including the required language and visual stimuli.",
                "definition": {
                    "concept": "Supported NIHSS assessment context",
                    "statement": "Confirm clinician administration of the standard adult NIHSS in official item order using an authorized picture-description, object-naming, reading, and dysarthria stimulus set.",
                    "excludes": ["Patient self-assessment", "PedNIHSS", "A partial examination represented as complete", "Use without the required language and visual stimuli", "Use of this calculator as NIHSS certification"],
                    "caveats": "The scoring text is public-domain NINDS information. This package deliberately does not redistribute third-party visual stimuli, training videos, certification cases, logos, or course materials and is not endorsed by NIH, NINDS, HHS, AHA, or Apex Innovations.",
                    "source": source,
                    "status": "draft"
                }
            },
            "level_of_consciousness": scored_property(
                "NIHSS 1a level of consciousness",
                "Observed responsiveness: alert=0; arousable_by_minor_stimulation=1; requires_repeated_or_strong_stimulation=2; reflex_responses_or_unresponsive=3.",
                &["alert", "arousable_by_minor_stimulation", "requires_repeated_or_strong_stimulation", "reflex_responses_or_unresponsive"],
                "Choose a numeric response despite an endotracheal tube, language barrier, or oral trauma. Score 3 only for reflex motor or autonomic effects or total unresponsiveness with flaccidity and areflexia.", &source
            ),
            "loc_questions": scored_property(
                "NIHSS 1b LOC questions",
                "Ask current month and age once: both_correct=0; one_correct=1; non_aphasic_communication_barrier=1; neither_correct_or_aphasic_or_stuporous=2.",
                &["both_correct", "one_correct", "non_aphasic_communication_barrier", "neither_correct_or_aphasic_or_stuporous"],
                "Only the initial answers count and near-correct answers receive no credit. Intubation, oral trauma, severe dysarthria, language barrier, or another non-aphasic inability to speak scores 1; aphasic or stuporous failure to comprehend scores 2.", &source
            ),
            "loc_commands": scored_property(
                "NIHSS 1c LOC commands",
                "Ask the patient to open and close the eyes, then grip and release the non-paretic hand: both_correct=0; one_correct=1; neither_correct=2.",
                &["both_correct", "one_correct", "neither_correct"],
                "Use the first attempt. Demonstrate if there is no response and substitute suitable one-step commands for a physical impediment. Credit an unequivocal attempt frustrated by weakness.", &source
            ),
            "best_gaze": scored_property(
                "NIHSS 2 best gaze",
                "Horizontal gaze: normal=0; partial_gaze_palsy=1, including isolated CN III/IV/VI paresis or conjugate deviation overcome by voluntary or oculocephalic activity; forced_deviation_or_total_paresis=2 only when not overcome by the oculocephalic manoeuvre.",
                &["normal", "partial_gaze_palsy", "forced_deviation_or_total_paresis"],
                "Test horizontal movement only. Reflex oculocephalic movement may be used; do not use caloric testing. Aphasia does not prevent testing.", &source
            ),
            "visual_fields": scored_property(
                "NIHSS 3 visual fields",
                "Visual fields: no_visual_loss=0; partial_hemianopia=1; complete_hemianopia=2; bilateral_hemianopia_or_blindness=3.",
                &["no_visual_loss", "partial_hemianopia", "complete_hemianopia", "bilateral_hemianopia_or_blindness"],
                "Test the remaining eye in unilateral blindness or enucleation. Blindness from any cause scores 3. Visual extinction found here scores 1 and also informs item 11.", &source
            ),
            "facial_palsy": scored_property(
                "NIHSS 4 facial palsy",
                "Facial movement: normal_symmetry=0; minor_paralysis=1; partial_lower_face_paralysis=2; complete_one_or_both_sides=3.",
                &["normal_symmetry", "minor_paralysis", "partial_lower_face_paralysis", "complete_one_or_both_sides"],
                "Remove physical barriers as far as possible. In a poorly responsive patient, score symmetry of grimace to noxious stimulation.", &source
            ),
            "motor_arm_left": scored_property(
                "NIHSS 5a left motor arm",
                "Left arm held at 90 degrees sitting or 45 degrees supine for 10 seconds: no_drift_for_ten_seconds=0; drift_without_hitting_support=1; some_effort_against_gravity=2; no_effort_against_gravity=3; no_movement=4; untestable_amputation_or_shoulder_fusion=UN.",
                &["no_drift_for_ten_seconds", "drift_without_hitting_support", "some_effort_against_gravity", "no_effort_against_gravity", "no_movement", "untestable_amputation_or_shoulder_fusion"],
                "Test the non-paretic arm first. UN is permitted only for amputation or joint fusion at the shoulder, not for weakness, pain, poor effort, paralysis, or inability to understand.", &source
            ),
            "motor_arm_right": scored_property(
                "NIHSS 5b right motor arm",
                "Right arm held at 90 degrees sitting or 45 degrees supine for 10 seconds: no_drift_for_ten_seconds=0; drift_without_hitting_support=1; some_effort_against_gravity=2; no_effort_against_gravity=3; no_movement=4; untestable_amputation_or_shoulder_fusion=UN.",
                &["no_drift_for_ten_seconds", "drift_without_hitting_support", "some_effort_against_gravity", "no_effort_against_gravity", "no_movement", "untestable_amputation_or_shoulder_fusion"],
                "Test the non-paretic arm first. UN is permitted only for amputation or joint fusion at the shoulder, not for weakness, pain, poor effort, paralysis, or inability to understand.", &source
            ),
            "motor_leg_left": scored_property(
                "NIHSS 6a left motor leg",
                "Left leg held at 30 degrees supine for 5 seconds: no_drift_for_five_seconds=0; drift_without_hitting_bed=1; some_effort_against_gravity=2; no_effort_against_gravity=3; no_movement=4; untestable_amputation_or_hip_fusion=UN.",
                &["no_drift_for_five_seconds", "drift_without_hitting_bed", "some_effort_against_gravity", "no_effort_against_gravity", "no_movement", "untestable_amputation_or_hip_fusion"],
                "Test the non-paretic leg first. UN is permitted only for amputation or joint fusion at the hip, not for weakness, pain, poor effort, paralysis, or inability to understand.", &source
            ),
            "motor_leg_right": scored_property(
                "NIHSS 6b right motor leg",
                "Right leg held at 30 degrees supine for 5 seconds: no_drift_for_five_seconds=0; drift_without_hitting_bed=1; some_effort_against_gravity=2; no_effort_against_gravity=3; no_movement=4; untestable_amputation_or_hip_fusion=UN.",
                &["no_drift_for_five_seconds", "drift_without_hitting_bed", "some_effort_against_gravity", "no_effort_against_gravity", "no_movement", "untestable_amputation_or_hip_fusion"],
                "Test the non-paretic leg first. UN is permitted only for amputation or joint fusion at the hip, not for weakness, pain, poor effort, paralysis, or inability to understand.", &source
            ),
            "limb_ataxia": scored_property(
                "NIHSS 7 limb ataxia",
                "Finger-nose-finger and heel-shin ataxia out of proportion to weakness: absent=0; present_in_one_limb=1; present_in_two_limbs=2; untestable_amputation_or_joint_fusion=UN.",
                &["absent", "present_in_one_limb", "present_in_two_limbs", "untestable_amputation_or_joint_fusion"],
                "Score 0, not UN, when paralysis or inability to understand prevents demonstration of ataxia. Use UN only when amputation or joint fusion prevents examination.", &source
            ),
            "sensory": scored_property(
                "NIHSS 8 sensory",
                "Stroke-attributable pinprick and touch response: normal=0; mild_to_moderate_loss=1; severe_or_total_loss=2.",
                &["normal", "mild_to_moderate_loss", "severe_or_total_loss"],
                "Count only sensory loss attributed to stroke. A nonresponsive quadriplegic patient and every patient scored 3 on item 1a receive 2 points.", &source
            ),
            "best_language": scored_property(
                "NIHSS 9 best language",
                "Language across the preceding examination and official stimuli: no_aphasia=0; mild_to_moderate_aphasia=1; severe_aphasia=2; mute_or_global_aphasia=3.",
                &["no_aphasia", "mild_to_moderate_aphasia", "severe_aphasia", "mute_or_global_aphasia"],
                "Use picture description, object naming, sentence reading, and performance throughout the examination. Intubation does not make language untestable; request writing. Every patient scored 3 on item 1a receives 3 points.", &source
            ),
            "dysarthria": scored_property(
                "NIHSS 10 dysarthria",
                "Articulation: normal=0; mild_to_moderate=1; severe_or_anarthric=2; untestable_intubation_or_physical_barrier=UN.",
                &["normal", "mild_to_moderate", "severe_or_anarthric", "untestable_intubation_or_physical_barrier"],
                "Do not mark UN merely for aphasia or muteness. UN is reserved for intubation or another physical barrier to producing speech.", &source
            ),
            "extinction_inattention": scored_property(
                "NIHSS 11 extinction and inattention",
                "Double simultaneous stimulation and spatial attention: none=0; one_modality=1; profound_or_more_than_one_modality=2.",
                &["none", "one_modality", "profound_or_more_than_one_modality"],
                "This item is never UN. Severe visual loss alone does not imply neglect; if visual testing is impossible but cutaneous attention is normal, score 0.", &source
            ),
            "unscored_explanations": {
                "type": ["object", "null"],
                "additionalProperties": false,
                "description": "One source-required physical-barrier explanation for each item marked UN.",
                "properties": {
                    "motor_arm_left": { "type": ["string", "null"], "minLength": 1, "pattern": "\\S" },
                    "motor_arm_right": { "type": ["string", "null"], "minLength": 1, "pattern": "\\S" },
                    "motor_leg_left": { "type": ["string", "null"], "minLength": 1, "pattern": "\\S" },
                    "motor_leg_right": { "type": ["string", "null"], "minLength": 1, "pattern": "\\S" },
                    "limb_ataxia": { "type": ["string", "null"], "minLength": 1, "pattern": "\\S" },
                    "dysarthria": { "type": ["string", "null"], "minLength": 1, "pattern": "\\S" }
                },
                "definition": {
                    "concept": "Explanations for unscored NIHSS items",
                    "statement": "For every item marked UN, provide the matching property and clearly document its physical barrier.",
                    "excludes": ["Weakness", "Pain", "Poor effort", "Paralysis", "Inability to understand"],
                    "caveats": "Required by the source whenever UN is used. UN is restricted to the explicitly permitted physical barriers.",
                    "source": source,
                    "status": "draft"
                }
            }
        },
        "allOf": [
            {
                "if": { "properties": { "level_of_consciousness": { "const": "reflex_responses_or_unresponsive" } }, "required": ["level_of_consciousness"] },
                "then": { "properties": {
                    "loc_questions": { "const": "neither_correct_or_aphasic_or_stuporous" },
                    "loc_commands": { "const": "neither_correct" },
                    "motor_arm_left": { "enum": ["no_effort_against_gravity", "no_movement", "untestable_amputation_or_shoulder_fusion"] },
                    "motor_arm_right": { "enum": ["no_effort_against_gravity", "no_movement", "untestable_amputation_or_shoulder_fusion"] },
                    "motor_leg_left": { "enum": ["no_effort_against_gravity", "no_movement", "untestable_amputation_or_hip_fusion"] },
                    "motor_leg_right": { "enum": ["no_effort_against_gravity", "no_movement", "untestable_amputation_or_hip_fusion"] },
                    "limb_ataxia": { "enum": ["absent", "untestable_amputation_or_joint_fusion"] },
                    "sensory": { "const": "severe_or_total_loss" },
                    "best_language": { "const": "mute_or_global_aphasia" },
                    "dysarthria": { "enum": ["severe_or_anarthric", "untestable_intubation_or_physical_barrier"] }
                } }
            },
            {
                "if": { "properties": { "limb_ataxia": { "const": "present_in_one_limb" } }, "required": ["limb_ataxia"] },
                "then": { "anyOf": [
                    { "properties": { "motor_arm_left": { "not": { "enum": ["no_movement", "untestable_amputation_or_shoulder_fusion"] } } } },
                    { "properties": { "motor_arm_right": { "not": { "enum": ["no_movement", "untestable_amputation_or_shoulder_fusion"] } } } },
                    { "properties": { "motor_leg_left": { "not": { "enum": ["no_movement", "untestable_amputation_or_hip_fusion"] } } } },
                    { "properties": { "motor_leg_right": { "not": { "enum": ["no_movement", "untestable_amputation_or_hip_fusion"] } } } }
                ] }
            },
            {
                "if": { "properties": { "limb_ataxia": { "const": "present_in_two_limbs" } }, "required": ["limb_ataxia"] },
                "then": { "anyOf": [
                    { "properties": { "motor_arm_left": { "not": { "enum": ["no_movement", "untestable_amputation_or_shoulder_fusion"] } }, "motor_arm_right": { "not": { "enum": ["no_movement", "untestable_amputation_or_shoulder_fusion"] } } } },
                    { "properties": { "motor_arm_left": { "not": { "enum": ["no_movement", "untestable_amputation_or_shoulder_fusion"] } }, "motor_leg_left": { "not": { "enum": ["no_movement", "untestable_amputation_or_hip_fusion"] } } } },
                    { "properties": { "motor_arm_left": { "not": { "enum": ["no_movement", "untestable_amputation_or_shoulder_fusion"] } }, "motor_leg_right": { "not": { "enum": ["no_movement", "untestable_amputation_or_hip_fusion"] } } } },
                    { "properties": { "motor_arm_right": { "not": { "enum": ["no_movement", "untestable_amputation_or_shoulder_fusion"] } }, "motor_leg_left": { "not": { "enum": ["no_movement", "untestable_amputation_or_hip_fusion"] } } } },
                    { "properties": { "motor_arm_right": { "not": { "enum": ["no_movement", "untestable_amputation_or_shoulder_fusion"] } }, "motor_leg_right": { "not": { "enum": ["no_movement", "untestable_amputation_or_hip_fusion"] } } } },
                    { "properties": { "motor_leg_left": { "not": { "enum": ["no_movement", "untestable_amputation_or_hip_fusion"] } }, "motor_leg_right": { "not": { "enum": ["no_movement", "untestable_amputation_or_hip_fusion"] } } } }
                ] }
            },
            {
                "if": { "properties": { "motor_arm_left": { "const": "untestable_amputation_or_shoulder_fusion" } }, "required": ["motor_arm_left"] },
                "then": { "required": ["unscored_explanations"], "properties": { "unscored_explanations": { "type": "object", "required": ["motor_arm_left"], "properties": { "motor_arm_left": { "type": "string" } } } } }
            },
            {
                "if": { "properties": { "motor_arm_right": { "const": "untestable_amputation_or_shoulder_fusion" } }, "required": ["motor_arm_right"] },
                "then": { "required": ["unscored_explanations"], "properties": { "unscored_explanations": { "type": "object", "required": ["motor_arm_right"], "properties": { "motor_arm_right": { "type": "string" } } } } }
            },
            {
                "if": { "properties": { "motor_leg_left": { "const": "untestable_amputation_or_hip_fusion" } }, "required": ["motor_leg_left"] },
                "then": { "required": ["unscored_explanations"], "properties": { "unscored_explanations": { "type": "object", "required": ["motor_leg_left"], "properties": { "motor_leg_left": { "type": "string" } } } } }
            },
            {
                "if": { "properties": { "motor_leg_right": { "const": "untestable_amputation_or_hip_fusion" } }, "required": ["motor_leg_right"] },
                "then": { "required": ["unscored_explanations"], "properties": { "unscored_explanations": { "type": "object", "required": ["motor_leg_right"], "properties": { "motor_leg_right": { "type": "string" } } } } }
            },
            {
                "if": { "properties": { "limb_ataxia": { "const": "untestable_amputation_or_joint_fusion" } }, "required": ["limb_ataxia"] },
                "then": { "required": ["unscored_explanations"], "properties": { "unscored_explanations": { "type": "object", "required": ["limb_ataxia"], "properties": { "limb_ataxia": { "type": "string" } } } } }
            },
            {
                "if": { "properties": { "dysarthria": { "const": "untestable_intubation_or_physical_barrier" } }, "required": ["dysarthria"] },
                "then": { "required": ["unscored_explanations"], "properties": { "unscored_explanations": { "type": "object", "required": ["dysarthria"], "properties": { "dysarthria": { "type": "string" } } } } }
            }
        ]
    })
}

pub struct Nihss;

impl Calculator for Nihss {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "NIH Stroke Scale (NIHSS)"
    }

    fn description(&self) -> &'static str {
        "Standard adult 0-42 neurological deficit score for clinician-administered stroke assessment; omits the total when any officially permitted entry is unscored."
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
        let parsed: NihssInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normal() -> NihssInput {
        NihssInput {
            assessment_context: AssessmentContext::ClinicianAdministeredStandardAdultNihssUsingAuthorizedScaleMaterials,
            level_of_consciousness: LevelOfConsciousness::Alert,
            loc_questions: LocQuestions::BothCorrect,
            loc_commands: LocCommands::BothCorrect,
            best_gaze: BestGaze::Normal,
            visual_fields: VisualFields::NoVisualLoss,
            facial_palsy: FacialPalsy::NormalSymmetry,
            motor_arm_left: MotorArm::NoDriftForTenSeconds,
            motor_arm_right: MotorArm::NoDriftForTenSeconds,
            motor_leg_left: MotorLeg::NoDriftForFiveSeconds,
            motor_leg_right: MotorLeg::NoDriftForFiveSeconds,
            limb_ataxia: LimbAtaxia::Absent,
            sensory: Sensory::Normal,
            best_language: BestLanguage::NoAphasia,
            dysarthria: Dysarthria::Normal,
            extinction_inattention: ExtinctionInattention::None,
            unscored_explanations: None,
        }
    }

    #[test]
    fn official_item_ranges_sum_from_zero_to_forty_two() {
        assert_eq!(compute(&normal()).unwrap().total_score, Some(0));
        let nominal_maximum = LevelOfConsciousness::ReflexResponsesOrUnresponsive.points()
            + LocQuestions::NeitherCorrectOrAphasicOrStuporous.points()
            + LocCommands::NeitherCorrect.points()
            + BestGaze::ForcedDeviationOrTotalParesis.points()
            + VisualFields::BilateralHemianopiaOrBlindness.points()
            + FacialPalsy::CompleteOneOrBothSides.points()
            + MotorArm::NoMovement.points().unwrap() * 2
            + MotorLeg::NoMovement.points().unwrap() * 2
            + LimbAtaxia::PresentInTwoLimbs.points().unwrap()
            + Sensory::SevereOrTotalLoss.points()
            + BestLanguage::MuteOrGlobalAphasia.points()
            + Dysarthria::SevereOrAnarthric.points().unwrap()
            + ExtinctionInattention::ProfoundOrMoreThanOneModality.points();
        assert_eq!(nominal_maximum, 42);
    }

    #[test]
    fn every_item_anchor_maps_to_official_points() {
        assert_eq!(
            LevelOfConsciousness::ArousableByMinorStimulation.points(),
            1
        );
        assert_eq!(
            LevelOfConsciousness::RequiresRepeatedOrStrongStimulation.points(),
            2
        );
        assert_eq!(LocQuestions::NonAphasicCommunicationBarrier.points(), 1);
        assert_eq!(LocQuestions::NeitherCorrectOrAphasicOrStuporous.points(), 2);
        assert_eq!(LocCommands::OneCorrect.points(), 1);
        assert_eq!(BestGaze::PartialGazePalsy.points(), 1);
        assert_eq!(VisualFields::CompleteHemianopia.points(), 2);
        assert_eq!(FacialPalsy::PartialLowerFaceParalysis.points(), 2);
        assert_eq!(MotorArm::DriftWithoutHittingSupport.points(), Some(1));
        assert_eq!(MotorArm::SomeEffortAgainstGravity.points(), Some(2));
        assert_eq!(MotorArm::NoEffortAgainstGravity.points(), Some(3));
        assert_eq!(MotorLeg::DriftWithoutHittingBed.points(), Some(1));
        assert_eq!(MotorLeg::SomeEffortAgainstGravity.points(), Some(2));
        assert_eq!(MotorLeg::NoEffortAgainstGravity.points(), Some(3));
        assert_eq!(LimbAtaxia::PresentInOneLimb.points(), Some(1));
        assert_eq!(Sensory::MildToModerateLoss.points(), 1);
        assert_eq!(BestLanguage::SevereAphasia.points(), 2);
        assert_eq!(Dysarthria::MildToModerate.points(), Some(1));
        assert_eq!(ExtinctionInattention::OneModality.points(), 1);
    }

    #[test]
    fn official_unscored_states_omit_total_and_preserve_partial_sum() {
        let input = NihssInput {
            motor_arm_left: MotorArm::UntestableAmputationOrShoulderFusion,
            best_gaze: BestGaze::PartialGazePalsy,
            dysarthria: Dysarthria::UntestableIntubationOrPhysicalBarrier,
            unscored_explanations: Some(NihssUnscoredExplanations {
                motor_arm_left: Some("Left arm amputation".into()),
                motor_arm_right: None,
                motor_leg_left: None,
                motor_leg_right: None,
                limb_ataxia: None,
                dysarthria: Some("Endotracheal tube prevented speech".into()),
            }),
            ..normal()
        };
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.total_score, None);
        assert_eq!(outcome.partial_scored_item_sum, 1);
        assert_eq!(outcome.unscored_items, ["motor_arm_left", "dysarthria"]);
        let response = build_response(&input).unwrap();
        assert_eq!(response.result, json!("not_calculable"));
        assert!(!response.working.contains_key("total_score"));
        assert!(!response.working.contains_key("motor_arm_left_points"));
        assert!(
            response
                .interpretation
                .contains("not a complete NIHSS total")
        );
    }

    #[test]
    fn each_permitted_unscored_item_is_representable() {
        for input in [
            NihssInput {
                motor_arm_right: MotorArm::UntestableAmputationOrShoulderFusion,
                unscored_explanations: Some(NihssUnscoredExplanations {
                    motor_arm_left: None,
                    motor_arm_right: Some("Right arm amputation".into()),
                    motor_leg_left: None,
                    motor_leg_right: None,
                    limb_ataxia: None,
                    dysarthria: None,
                }),
                ..normal()
            },
            NihssInput {
                motor_leg_left: MotorLeg::UntestableAmputationOrHipFusion,
                unscored_explanations: Some(NihssUnscoredExplanations {
                    motor_arm_left: None,
                    motor_arm_right: None,
                    motor_leg_left: Some("Left leg amputation".into()),
                    motor_leg_right: None,
                    limb_ataxia: None,
                    dysarthria: None,
                }),
                ..normal()
            },
            NihssInput {
                motor_leg_right: MotorLeg::UntestableAmputationOrHipFusion,
                unscored_explanations: Some(NihssUnscoredExplanations {
                    motor_arm_left: None,
                    motor_arm_right: None,
                    motor_leg_left: None,
                    motor_leg_right: Some("Right hip fusion".into()),
                    limb_ataxia: None,
                    dysarthria: None,
                }),
                ..normal()
            },
            NihssInput {
                limb_ataxia: LimbAtaxia::UntestableAmputationOrJointFusion,
                unscored_explanations: Some(NihssUnscoredExplanations {
                    motor_arm_left: None,
                    motor_arm_right: None,
                    motor_leg_left: None,
                    motor_leg_right: None,
                    limb_ataxia: Some("Joint fusion prevented coordination testing".into()),
                    dysarthria: None,
                }),
                ..normal()
            },
        ] {
            assert_eq!(compute(&input).unwrap().total_score, None);
        }
    }

    #[test]
    fn coma_mandated_sensory_and_language_scores_are_enforced() {
        let valid_coma = NihssInput {
            level_of_consciousness: LevelOfConsciousness::ReflexResponsesOrUnresponsive,
            loc_questions: LocQuestions::NeitherCorrectOrAphasicOrStuporous,
            loc_commands: LocCommands::NeitherCorrect,
            limb_ataxia: LimbAtaxia::Absent,
            sensory: Sensory::SevereOrTotalLoss,
            best_language: BestLanguage::MuteOrGlobalAphasia,
            motor_arm_left: MotorArm::NoMovement,
            motor_arm_right: MotorArm::NoMovement,
            motor_leg_left: MotorLeg::NoMovement,
            motor_leg_right: MotorLeg::NoMovement,
            dysarthria: Dysarthria::SevereOrAnarthric,
            ..normal()
        };
        assert!(compute(&valid_coma).is_ok());
        assert!(
            compute(&NihssInput {
                motor_arm_left: MotorArm::NoEffortAgainstGravity,
                motor_arm_right: MotorArm::NoEffortAgainstGravity,
                motor_leg_left: MotorLeg::NoEffortAgainstGravity,
                motor_leg_right: MotorLeg::NoEffortAgainstGravity,
                ..valid_coma.clone()
            })
            .is_ok()
        );
        let invalid_sensory = NihssInput {
            sensory: Sensory::Normal,
            ..valid_coma.clone()
        };
        assert!(compute(&invalid_sensory).is_err());
        let invalid_language = NihssInput {
            best_language: BestLanguage::SevereAphasia,
            ..valid_coma.clone()
        };
        assert!(compute(&invalid_language).is_err());
        assert!(
            compute(&NihssInput {
                loc_questions: LocQuestions::BothCorrect,
                ..valid_coma.clone()
            })
            .is_err()
        );

        let coma_with_amputation = NihssInput {
            motor_arm_left: MotorArm::UntestableAmputationOrShoulderFusion,
            limb_ataxia: LimbAtaxia::UntestableAmputationOrJointFusion,
            unscored_explanations: Some(NihssUnscoredExplanations {
                motor_arm_left: Some("Left arm amputation".into()),
                motor_arm_right: None,
                motor_leg_left: None,
                motor_leg_right: None,
                limb_ataxia: Some("Left arm amputation prevented full testing".into()),
                dysarthria: None,
            }),
            ..valid_coma.clone()
        };
        assert_eq!(compute(&coma_with_amputation).unwrap().total_score, None);
        assert!(
            compute(&NihssInput {
                loc_commands: LocCommands::BothCorrect,
                ..valid_coma.clone()
            })
            .is_err()
        );
        assert!(
            compute(&NihssInput {
                limb_ataxia: LimbAtaxia::PresentInOneLimb,
                ..valid_coma.clone()
            })
            .is_err()
        );
        assert!(
            compute(&NihssInput {
                motor_arm_left: MotorArm::NoDriftForTenSeconds,
                ..valid_coma.clone()
            })
            .is_err()
        );
        assert!(
            compute(&NihssInput {
                dysarthria: Dysarthria::Normal,
                ..valid_coma
            })
            .is_err()
        );
    }

    #[test]
    fn paralysis_and_unscored_explanation_rules_are_enforced() {
        let paralysed_with_ataxia = NihssInput {
            motor_arm_left: MotorArm::NoMovement,
            motor_arm_right: MotorArm::NoMovement,
            motor_leg_left: MotorLeg::NoMovement,
            motor_leg_right: MotorLeg::NoMovement,
            limb_ataxia: LimbAtaxia::PresentInTwoLimbs,
            ..normal()
        };
        assert!(compute(&paralysed_with_ataxia).is_err());
        let one_capable_limb = NihssInput {
            motor_arm_left: MotorArm::NoMovement,
            motor_arm_right: MotorArm::NoMovement,
            motor_leg_left: MotorLeg::NoMovement,
            limb_ataxia: LimbAtaxia::PresentInOneLimb,
            ..normal()
        };
        assert!(compute(&one_capable_limb).is_ok());
        assert!(
            compute(&NihssInput {
                limb_ataxia: LimbAtaxia::PresentInTwoLimbs,
                ..one_capable_limb
            })
            .is_err()
        );

        let unscored = NihssInput {
            motor_arm_left: MotorArm::UntestableAmputationOrShoulderFusion,
            ..normal()
        };
        assert!(compute(&unscored).is_err());
        assert!(
            compute(&NihssInput {
                unscored_explanations: Some(NihssUnscoredExplanations {
                    motor_arm_left: Some("   ".into()),
                    motor_arm_right: None,
                    motor_leg_left: None,
                    motor_leg_right: None,
                    limb_ataxia: None,
                    dysarthria: None,
                }),
                ..unscored.clone()
            })
            .is_err()
        );
        assert!(
            compute(&NihssInput {
                unscored_explanations: Some(NihssUnscoredExplanations {
                    motor_arm_left: Some("Left arm amputation".into()),
                    motor_arm_right: None,
                    motor_leg_left: None,
                    motor_leg_right: None,
                    limb_ataxia: None,
                    dysarthria: None,
                }),
                ..unscored
            })
            .is_ok()
        );
    }

    #[test]
    fn response_is_measurement_not_diagnosis_or_treatment_rule() {
        let response = build_response(&normal()).unwrap();
        assert_eq!(response.result, json!(0));
        assert_eq!(response.working["maximum_complete_score"], json!(42));
        assert!(
            response
                .interpretation
                .contains("does not diagnose or exclude stroke")
        );
        assert!(
            response
                .interpretation
                .contains("Treatment decisions require")
        );
        assert!(!response.working.contains_key("severity_band"));
    }

    #[test]
    fn dynamic_surface_is_closed_and_matches_typed_response() {
        let value = serde_json::to_value(normal()).unwrap();
        assert_eq!(
            Nihss.calculate(&value).unwrap(),
            build_response(&normal()).unwrap()
        );
        let mut unknown = value;
        unknown["raw_total"] = json!(12);
        assert!(Nihss.calculate(&unknown).is_err());
    }

    #[test]
    fn schema_is_closed_complete_and_restricts_un_to_official_items() {
        let schema = Nihss.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["required"].as_array().unwrap().len(), 16);
        assert_eq!(schema["properties"].as_object().unwrap().len(), 17);
        assert_eq!(schema["allOf"].as_array().unwrap().len(), 9);
        assert!(
            !schema["properties"]["sensory"]["enum"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str().unwrap().contains("untestable"))
        );
        assert!(
            schema["properties"]["dysarthria"]["enum"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str().unwrap().contains("untestable"))
        );
        for property in schema["properties"].as_object().unwrap().values() {
            assert!(property["definition"]["statement"].is_string());
            assert_eq!(property["definition"]["status"], json!("draft"));
        }
    }
}
