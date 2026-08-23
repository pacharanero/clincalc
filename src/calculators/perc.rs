// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! PERC - Pulmonary Embolism Rule-out Criteria.
//!
//! An eight-item block rule for suspected pulmonary embolism, derived by
//! Kline et al. (J Thromb Haemost. 2004;2(8):1247-1255) and validated in a
//! prospective multicentre cohort (Kline et al. J Thromb Haemost.
//! 2008;6(5):772-780).
//!
//! PERC is only validated after the treating clinician has estimated the
//! pretest probability of PE at below 15% by unstructured gestalt. This
//! precondition is required as an input rather than left as a prose warning.
//! A PERC-negative result means all eight criteria are absent and supports no
//! further PE-specific D-dimer or imaging in that selected population. A
//! PERC-positive result only means that PERC cannot exclude PE; it does not
//! diagnose PE, and the number of positive criteria is not a graded risk score.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "perc";

/// Primary citation.
pub const REFERENCE: &str = "Kline JA, Mitchell AM, Kabrhel C, Richman PB, Courtney DM. Clinical criteria to prevent unnecessary diagnostic testing in emergency department patients with suspected pulmonary embolism. J Thromb Haemost. 2004;2(8):1247-1255. doi:10.1111/j.1538-7836.2004.00790.x. Kline JA, Courtney DM, Kabrhel C, et al. Prospective multicenter evaluation of the pulmonary embolism rule-out criteria. J Thromb Haemost. 2008;6(5):772-780. doi:10.1111/j.1538-7836.2008.02944.x. Freund Y, Cachanado M, Aubry A, et al. Effect of the Pulmonary Embolism Rule-Out Criteria on Subsequent Thromboembolic Events Among Low-Risk Emergency Department Patients: The PROPER Randomized Clinical Trial. JAMA. 2018;319(6):559-566. doi:10.1001/jama.2017.21904.";

/// Distribution licence: the rule is a published clinical method from the
/// primary literature, implemented here from that source.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1111/j.1538-7836.2004.00790.x",
};

/// PERC eligibility and observations. Numeric boundaries are derived by the
/// engine so callers cannot submit contradictory measurements and flags.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PercInput {
    /// The treating clinician has estimated PE probability below 15% by
    /// unstructured gestalt, as required by the prospective validation.
    pub low_pretest_probability_confirmed: bool,
    /// Age in completed years. PERC is validated for adults.
    pub age_years: u8,
    /// Measured pulse in beats per minute.
    pub heart_rate_bpm: u16,
    /// Pulse-oximetry oxygen saturation measured on room air.
    pub room_air_oxygen_saturation_percent: u8,
    /// Unilateral leg swelling.
    pub unilateral_leg_swelling: bool,
    /// Haemoptysis.
    pub haemoptysis: bool,
    /// Surgery or trauma requiring hospitalisation within the prior 4 weeks.
    pub recent_surgery_or_trauma_requiring_hospitalisation: bool,
    /// Prior venous thromboembolism (DVT or PE).
    pub prior_vte: bool,
    /// Current exogenous oestrogen use.
    pub exogenous_oestrogen_use: bool,
}

/// Which of the 8 criteria were positive, in a stable order.
const CRITERION_NAMES: [&str; 8] = [
    "age_50_or_over",
    "heart_rate_100_or_over",
    "room_air_oxygen_saturation_below_95",
    "unilateral_leg_swelling",
    "haemoptysis",
    "recent_surgery_or_trauma_requiring_hospitalisation",
    "prior_vte",
    "exogenous_oestrogen_use",
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

fn criterion_flags(input: &PercInput) -> [bool; 8] {
    [
        input.age_years >= 50,
        input.heart_rate_bpm >= 100,
        input.room_air_oxygen_saturation_percent < 95,
        input.unilateral_leg_swelling,
        input.haemoptysis,
        input.recent_surgery_or_trauma_requiring_hospitalisation,
        input.prior_vte,
        input.exogenous_oestrogen_use,
    ]
}

/// Pure scoring.
pub fn compute(input: &PercInput) -> Result<PercOutcome, CalcError> {
    if !input.low_pretest_probability_confirmed {
        return Err(CalcError::InvalidInput(
            "PERC requires the treating clinician to estimate PE pretest probability below 15% by unstructured gestalt before applying the rule".into(),
        ));
    }
    if !(18..=120).contains(&input.age_years) {
        return Err(CalcError::InvalidInput(
            "age_years must be between 18 and 120; PERC is validated for adults".into(),
        ));
    }
    if input.heart_rate_bpm == 0 || input.heart_rate_bpm > 300 {
        return Err(CalcError::InvalidInput(
            "heart_rate_bpm must be between 1 and 300".into(),
        ));
    }
    if input.room_air_oxygen_saturation_percent == 0
        || input.room_air_oxygen_saturation_percent > 100
    {
        return Err(CalcError::InvalidInput(
            "room_air_oxygen_saturation_percent must be between 1 and 100".into(),
        ));
    }

    let flags = criterion_flags(input);

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
            "PERC-negative (0 of 8 criteria present). In a stable adult emergency-department \
outpatient with suspected PE whose treating clinician has estimated the pretest probability below \
15% by unstructured gestalt, the validated PERC strategy supports no further PE-specific D-dimer \
or imaging. PERC does not assess alternative diagnoses; reassess if symptoms persist or worsen."
                .to_string()
        }
        PercResult::Positive => format!(
            "PERC-positive ({positive_count} of 8 criteria present). PE cannot be excluded on PERC \
alone; follow the appropriate diagnostic pathway. This is not a diagnosis of PE, and the number \
of positive criteria is not a graded probability or severity score."
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
    working.insert(
        "low_pretest_probability_confirmed".into(),
        json!(input.low_pretest_probability_confirmed),
    );
    working.insert("age_years".into(), json!(input.age_years));
    working.insert("heart_rate_bpm".into(), json!(input.heart_rate_bpm));
    working.insert(
        "room_air_oxygen_saturation_percent".into(),
        json!(input.room_air_oxygen_saturation_percent),
    );
    for (name, present) in CRITERION_NAMES.into_iter().zip(criterion_flags(input)) {
        working.insert(name.into(), json!(present));
    }

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
        "Eight-item block rule for stable adult emergency-department outpatients with suspected \
pulmonary embolism and clinician gestalt below 15%; PERC-negative can avoid further PE-specific \
testing."
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
                "low_pretest_probability_confirmed", "age_years", "heart_rate_bpm",
                "room_air_oxygen_saturation_percent",
                "unilateral_leg_swelling", "haemoptysis",
                "recent_surgery_or_trauma_requiring_hospitalisation",
                "prior_vte", "exogenous_oestrogen_use"
            ],
            "properties": {
                "low_pretest_probability_confirmed": {
                    "type": "boolean",
                    "description": "Treating clinician has estimated PE pretest probability below 15% by unstructured gestalt (required to apply PERC)",
                    "definition": {
                        "concept": "PERC eligibility: low clinician gestalt",
                        "statement": "Before PERC is applied, the treating clinician must estimate the pretest probability of PE at below 15% by unstructured clinical gestalt.",
                        "includes": ["Treating clinician's unstructured estimate of PE probability below 15%"],
                        "excludes": ["Moderate or high clinical suspicion", "Using PERC itself to establish low pretest probability", "Automatically substituting a low Wells or Geneva score for clinician gestalt"],
                        "caveats": "The prospective validation combined low unstructured clinician gestalt with PERC-negative status. PERC must not be applied to an unselected suspected-PE population.",
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2008;6(5):772-780.", "url": "https://doi.org/10.1111/j.1538-7836.2008.02944.x" },
                        "status": "draft"
                    }
                },
                "age_years": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 120,
                    "description": "Age in completed years (>= 50 is PERC-positive)",
                    "definition": {
                        "concept": "Age criterion",
                        "statement": "Age 50 years or older is PERC-positive; age below 50 is PERC-negative for this item.",
                        "caveats": "The adult PERC evidence does not establish safety in children.",
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2008;6(5):772-780.", "url": "https://doi.org/10.1111/j.1538-7836.2008.02944.x" },
                        "status": "draft"
                    }
                },
                "heart_rate_bpm": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 300,
                    "description": "Measured pulse in beats per minute (>= 100 is PERC-positive)",
                    "definition": {
                        "concept": "Pulse criterion",
                        "statement": "A pulse of 100 beats per minute or greater is PERC-positive; exactly 100 counts.",
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2008;6(5):772-780.", "url": "https://doi.org/10.1111/j.1538-7836.2008.02944.x" },
                        "status": "draft"
                    }
                },
                "room_air_oxygen_saturation_percent": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 100,
                    "description": "Pulse-oximetry oxygen saturation on room air, percent (< 95 is PERC-positive)",
                    "definition": {
                        "concept": "Room-air oxygen saturation criterion",
                        "statement": "Pulse-oximetry oxygen saturation below 95% on room air is PERC-positive; exactly 95% is negative.",
                        "excludes": ["A saturation measured while supplemental oxygen is being administered"],
                        "caveats": "The rule was validated using room-air saturation. Supplemental oxygen can conceal a positive criterion.",
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2008;6(5):772-780.", "url": "https://doi.org/10.1111/j.1538-7836.2008.02944.x" },
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
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2008;6(5):772-780.", "url": "https://doi.org/10.1111/j.1538-7836.2008.02944.x" },
                        "status": "draft"
                    }
                },
                "haemoptysis": {
                    "type": "boolean",
                    "description": "Haemoptysis",
                    "definition": {
                        "concept": "Haemoptysis",
                        "statement": "Coughing up of blood or blood-stained sputum.",
                        "includes": ["Frank haemoptysis", "Blood-streaked sputum"],
                        "excludes": ["Haematemesis", "Blood originating from the nose or upper airway"],
                        "snomedEcl": "<< 66857006 |Hemoptysis (finding)|",
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2008;6(5):772-780.", "url": "https://doi.org/10.1111/j.1538-7836.2008.02944.x" },
                        "status": "draft"
                    }
                },
                "recent_surgery_or_trauma_requiring_hospitalisation": {
                    "type": "boolean",
                    "description": "Surgery or trauma requiring hospitalisation within the prior 4 weeks",
                    "definition": {
                        "concept": "Recent surgery or trauma criterion",
                        "statement": "Surgery or trauma within the prior 4 weeks that required hospitalisation.",
                        "includes": ["Surgery requiring hospitalisation within the preceding 4 weeks", "Trauma requiring hospitalisation within the preceding 4 weeks"],
                        "excludes": ["A procedure that did not require hospitalisation", "Surgery or trauma more than 4 weeks ago", "General anaesthesia alone without hospitalisation"],
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2008;6(5):772-780.", "url": "https://doi.org/10.1111/j.1538-7836.2008.02944.x" },
                        "status": "draft"
                    }
                },
                "prior_vte": {
                    "type": "boolean",
                    "description": "Prior venous thromboembolism (DVT or PE)",
                    "definition": {
                        "concept": "Prior VTE criterion",
                        "statement": "A prior deep vein thrombosis or pulmonary embolism.",
                        "includes": ["Prior deep vein thrombosis", "Prior pulmonary embolism"],
                        "snomedEcl": "<< 128053003 |Deep venous thrombosis (disorder)| OR << 59282003 |Pulmonary embolism (disorder)|",
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2008;6(5):772-780.", "url": "https://doi.org/10.1111/j.1538-7836.2008.02944.x" },
                        "status": "draft"
                    }
                },
                "exogenous_oestrogen_use": {
                    "type": "boolean",
                    "description": "Current exogenous oestrogen use",
                    "definition": {
                        "concept": "Exogenous oestrogen use criterion",
                        "statement": "Current use of medication containing exogenous oestrogen.",
                        "includes": ["Combined oestrogen-containing contraception", "Oestrogen-containing hormone replacement therapy", "Other current exogenous oestrogen therapy"],
                        "excludes": ["Progestogen-only contraception", "Testosterone", "Corticosteroids", "Thyroid hormone", "Other hormone therapy containing no oestrogen"],
                        "source": { "citation": "Kline JA et al. J Thromb Haemost. 2008;6(5):772-780.", "url": "https://doi.org/10.1111/j.1538-7836.2008.02944.x" },
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

    /// PERC-negative boundary vector from Kline et al. 2008: age <50, pulse
    /// <100, room-air SaO2 >=95%, and all five asserted findings absent.
    fn perc_negative() -> PercInput {
        PercInput {
            low_pretest_probability_confirmed: true,
            age_years: 49,
            heart_rate_bpm: 99,
            room_air_oxygen_saturation_percent: 95,
            unilateral_leg_swelling: false,
            haemoptysis: false,
            recent_surgery_or_trauma_requiring_hospitalisation: false,
            prior_vte: false,
            exogenous_oestrogen_use: false,
        }
    }

    fn all_positive() -> PercInput {
        PercInput {
            low_pretest_probability_confirmed: true,
            age_years: 50,
            heart_rate_bpm: 100,
            room_air_oxygen_saturation_percent: 94,
            unilateral_leg_swelling: true,
            haemoptysis: true,
            recent_surgery_or_trauma_requiring_hospitalisation: true,
            prior_vte: true,
            exogenous_oestrogen_use: true,
        }
    }

    #[test]
    fn kline_2008_all_absent_vector_is_perc_negative() {
        let o = compute(&perc_negative()).unwrap();
        assert_eq!(o.positive_count, 0);
        assert!(o.positive_criteria.is_empty());
        assert_eq!(o.result, PercResult::Negative);
        assert!(o.interpretation.contains("PERC-negative"));
        assert!(o.interpretation.contains("below 15%"));
    }

    #[test]
    fn kline_2008_exact_age_and_pulse_boundaries_are_positive() {
        let mut input = perc_negative();
        input.age_years = 50;
        input.heart_rate_bpm = 100;
        let o = compute(&input).unwrap();
        assert_eq!(o.positive_count, 2);
        assert_eq!(
            o.positive_criteria,
            vec!["age_50_or_over", "heart_rate_100_or_over"]
        );
    }

    #[test]
    fn kline_2008_room_air_saturation_boundary_is_95_percent() {
        let mut input = perc_negative();
        assert_eq!(compute(&input).unwrap().result, PercResult::Negative);
        input.room_air_oxygen_saturation_percent = 94;
        let o = compute(&input).unwrap();
        assert_eq!(o.positive_count, 1);
        assert_eq!(
            o.positive_criteria,
            vec!["room_air_oxygen_saturation_below_95"]
        );
    }

    #[test]
    fn all_present_is_perc_positive_with_max_count() {
        let o = compute(&all_positive()).unwrap();
        assert_eq!(o.positive_count, 8);
        assert_eq!(o.positive_criteria.len(), 8);
        assert_eq!(o.result, PercResult::Positive);
        assert!(o.interpretation.contains("PERC-positive"));
        assert!(o.interpretation.contains("not a graded probability"));
    }

    #[test]
    fn single_exogenous_oestrogen_criterion_is_perc_positive() {
        let mut input = perc_negative();
        input.exogenous_oestrogen_use = true;
        let o = compute(&input).unwrap();
        assert_eq!(o.positive_count, 1);
        assert_eq!(o.positive_criteria, vec!["exogenous_oestrogen_use"]);
        assert_eq!(o.result, PercResult::Positive);
    }

    #[test]
    fn low_gestalt_precondition_is_required() {
        let mut input = perc_negative();
        input.low_pretest_probability_confirmed = false;
        let error = compute(&input).unwrap_err().to_string();
        assert!(error.contains("below 15%"));
        assert!(error.contains("unstructured gestalt"));
    }

    #[test]
    fn rejects_out_of_domain_observations() {
        let mut input = perc_negative();
        input.age_years = 17;
        assert!(compute(&input).is_err());

        input = perc_negative();
        input.heart_rate_bpm = 0;
        assert!(compute(&input).is_err());

        input = perc_negative();
        input.room_air_oxygen_saturation_percent = 0;
        assert!(compute(&input).is_err());
    }

    #[test]
    fn build_response_carries_working_and_reference() {
        let mut input = perc_negative();
        input.haemoptysis = true;
        let r = build_response(&input).unwrap();
        assert_eq!(r.calculator, "perc");
        assert_eq!(r.result, json!("perc-positive"));
        assert_eq!(r.working["positive_count"], json!(1));
        assert_eq!(r.working["result"], json!("perc-positive"));
        assert_eq!(r.working["positive_criteria"], json!(["haemoptysis"]));
        assert_eq!(r.working["age_years"], json!(49));
        assert_eq!(r.working["haemoptysis"], json!(true));
        assert!(r.reference.contains("Kline"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "low_pretest_probability_confirmed": true,
            "age_years": 49,
            "heart_rate_bpm": 100,
            "room_air_oxygen_saturation_percent": 95,
            "unilateral_leg_swelling": false,
            "haemoptysis": false,
            "recent_surgery_or_trauma_requiring_hospitalisation": false,
            "prior_vte": false,
            "exogenous_oestrogen_use": false
        });
        let mut typed = perc_negative();
        typed.heart_rate_bpm = 100;
        let dynamic = Perc.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!("perc-positive"));
    }

    #[test]
    fn dynamic_calculate_rejects_garbage() {
        assert!(Perc.calculate(&json!({ "age_years": "49" })).is_err());
    }

    #[test]
    fn dynamic_calculate_rejects_unknown_fields() {
        let mut value = serde_json::to_value(all_positive()).unwrap();
        value["extra_field"] = json!(true);
        assert!(Perc.calculate(&value).is_err());
    }

    #[test]
    fn schema_requires_eligibility_and_all_eight_criteria() {
        let schema = Perc.input_schema();
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 9);
        assert!(required.contains(&json!("low_pretest_probability_confirmed")));
        assert!(required.contains(&json!("exogenous_oestrogen_use")));
    }

    #[test]
    fn schema_preserves_primary_source_definitions() {
        let schema = Perc.input_schema();
        let surgery_exclusions = &schema["properties"]["recent_surgery_or_trauma_requiring_hospitalisation"]
            ["definition"]["excludes"];
        assert!(
            surgery_exclusions
                .as_array()
                .unwrap()
                .contains(&json!("General anaesthesia alone without hospitalisation"))
        );
        let oestrogen_exclusions =
            &schema["properties"]["exogenous_oestrogen_use"]["definition"]["excludes"];
        assert!(
            oestrogen_exclusions
                .as_array()
                .unwrap()
                .contains(&json!("Progestogen-only contraception"))
        );
    }
}
