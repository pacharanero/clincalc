// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Energy requirement - adult BMR/RMR estimates, optional activity factor, and target kcal/day.
//!
//! This is deliberately one calculator with an `equation` selector rather than five separate registry entries: the inputs and outputs are the same clinical concept, and one schema keeps every surface consistent.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "energy_requirement";

/// Distribution licence: these are published predictive equations, implemented here from the cited primary sources and official reports.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain published equations - implemented from primary literature and official reports; see reference() for all sources",
    source_url: "https://doi.org/10.1093/ajcn/51.2.241",
};

/// Primary citations.
pub const REFERENCE: &str = "Mifflin MD, St Jeor ST, Hill LA, Scott BJ, Daugherty SA, Koh YO. A new predictive equation for resting energy expenditure in healthy individuals. Am J Clin Nutr. 1990;51(2):241-247. doi:10.1093/ajcn/51.2.241; Harris JA, Benedict FG. A biometric study of human basal metabolism in man. Carnegie Institution of Washington; 1919; Roza AM, Shizgal HM. The Harris Benedict equation reevaluated: resting energy requirements and the body cell mass. Am J Clin Nutr. 1984;40(1):168-182. doi:10.1093/ajcn/40.1.168; Schofield WN. Predicting basal metabolic rate, new standards and review of previous work. Hum Nutr Clin Nutr. 1985;39 Suppl 1:5-41; Cunningham JJ. A reanalysis of the factors influencing basal metabolic rate in normal adults. Am J Clin Nutr. 1980;33(11):2372-2374. doi:10.1093/ajcn/33.11.2372";

/// Equation to use for the basal/resting energy estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Equation {
    /// Mifflin-St Jeor resting energy expenditure equation (1990).
    MifflinStJeor,
    /// Original Harris-Benedict basal metabolic rate equation (1919).
    HarrisBenedictOriginal,
    /// Revised Harris-Benedict / Roza-Shizgal equation (1984).
    HarrisBenedictRevised,
    /// FAO/WHO/UNU Schofield adult weight-only equations (1985).
    Schofield,
    /// Cunningham lean-body-mass resting metabolic rate equation (1980).
    Cunningham,
}

impl Equation {
    fn slug(self) -> &'static str {
        match self {
            Equation::MifflinStJeor => "mifflin_st_jeor",
            Equation::HarrisBenedictOriginal => "harris_benedict_original",
            Equation::HarrisBenedictRevised => "harris_benedict_revised",
            Equation::Schofield => "schofield",
            Equation::Cunningham => "cunningham",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Equation::MifflinStJeor => "Mifflin-St Jeor",
            Equation::HarrisBenedictOriginal => "Harris-Benedict original",
            Equation::HarrisBenedictRevised => "Harris-Benedict revised (Roza-Shizgal)",
            Equation::Schofield => "Schofield / FAO-WHO-UNU",
            Equation::Cunningham => "Cunningham",
        }
    }

    fn needs_height(self) -> bool {
        matches!(
            self,
            Equation::MifflinStJeor
                | Equation::HarrisBenedictOriginal
                | Equation::HarrisBenedictRevised
        )
    }
}

/// Sex used to select equation coefficients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

/// Adult Schofield age band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgeBand {
    Age18To29,
    Age30To59,
    Age60Plus,
}

impl AgeBand {
    fn from_age(age: u8) -> Self {
        if age < 30 {
            AgeBand::Age18To29
        } else if age < 60 {
            AgeBand::Age30To59
        } else {
            AgeBand::Age60Plus
        }
    }

    fn slug(self) -> &'static str {
        match self {
            AgeBand::Age18To29 => "18-29",
            AgeBand::Age30To59 => "30-59",
            AgeBand::Age60Plus => "60+",
        }
    }
}

/// Inputs for adult energy requirement estimation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyRequirementInput {
    /// Predictive equation to use.
    pub equation: Equation,
    /// Sex for equations with sex-specific coefficients. Required except for Cunningham.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sex: Option<Sex>,
    /// Adult age in years. Required except for Cunningham.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age: Option<u8>,
    /// Weight in kilograms. Required except for Cunningham.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight_kg: Option<f64>,
    /// Height in centimetres. Required for Mifflin-St Jeor and both Harris-Benedict variants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height_cm: Option<f64>,
    /// Lean body mass in kilograms. Required for Cunningham.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lean_body_mass_kg: Option<f64>,
    /// Optional physical activity level / activity factor. If supplied, basal estimate is multiplied by this to estimate TDEE.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity_factor: Option<f64>,
    /// Optional calorie adjustment in kcal/day: negative for weight loss, positive for gain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calorie_adjustment_kcal_day: Option<f64>,
}

/// Computed energy requirement outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct EnergyRequirementOutcome {
    pub equation: Equation,
    pub basal_kcal_day: u32,
    pub maintenance_kcal_day: u32,
    pub target_kcal_day: u32,
    pub activity_factor: Option<f64>,
    pub calorie_adjustment_kcal_day: f64,
    pub schofield_age_band: Option<AgeBand>,
    pub interpretation: String,
}

/// Pure calculation: adult BMR/RMR estimate, optional activity multiplier, and optional target adjustment.
pub fn compute(input: &EnergyRequirementInput) -> Result<EnergyRequirementOutcome, CalcError> {
    let activity_factor = validate_activity_factor(input.activity_factor)?;
    let adjustment = validate_adjustment(input.calorie_adjustment_kcal_day)?;

    let (basal_raw, schofield_age_band) = match input.equation {
        Equation::MifflinStJeor => {
            let sex = required_sex(input.sex, input.equation)?;
            let age = required_age(input.age, input.equation)?;
            let weight = required_weight(input.weight_kg, input.equation)?;
            let height = required_height(input.height_cm, input.equation)?;
            let sex_term = match sex {
                Sex::Male => 5.0,
                Sex::Female => -161.0,
            };
            (
                10.0 * weight + 6.25 * height - 5.0 * f64::from(age) + sex_term,
                None,
            )
        }
        Equation::HarrisBenedictOriginal => {
            let sex = required_sex(input.sex, input.equation)?;
            let age = required_age(input.age, input.equation)?;
            let weight = required_weight(input.weight_kg, input.equation)?;
            let height = required_height(input.height_cm, input.equation)?;
            let age = f64::from(age);
            let basal = match sex {
                Sex::Male => 66.4730 + 13.7516 * weight + 5.0033 * height - 6.7550 * age,
                Sex::Female => 655.0955 + 9.5634 * weight + 1.8496 * height - 4.6756 * age,
            };
            (basal, None)
        }
        Equation::HarrisBenedictRevised => {
            let sex = required_sex(input.sex, input.equation)?;
            let age = required_age(input.age, input.equation)?;
            let weight = required_weight(input.weight_kg, input.equation)?;
            let height = required_height(input.height_cm, input.equation)?;
            let age = f64::from(age);
            let basal = match sex {
                Sex::Male => 88.362 + 13.397 * weight + 4.799 * height - 5.677 * age,
                Sex::Female => 447.593 + 9.247 * weight + 3.098 * height - 4.330 * age,
            };
            (basal, None)
        }
        Equation::Schofield => {
            let sex = required_sex(input.sex, input.equation)?;
            let age = required_age(input.age, input.equation)?;
            let weight = required_weight(input.weight_kg, input.equation)?;
            let band = AgeBand::from_age(age);
            let basal = match (sex, band) {
                (Sex::Male, AgeBand::Age18To29) => 15.057 * weight + 692.2,
                (Sex::Male, AgeBand::Age30To59) => 11.472 * weight + 873.1,
                (Sex::Male, AgeBand::Age60Plus) => 11.711 * weight + 587.7,
                (Sex::Female, AgeBand::Age18To29) => 14.818 * weight + 486.6,
                (Sex::Female, AgeBand::Age30To59) => 8.126 * weight + 845.6,
                (Sex::Female, AgeBand::Age60Plus) => 9.082 * weight + 658.5,
            };
            (basal, Some(band))
        }
        Equation::Cunningham => {
            let lean_body_mass = required_lean_body_mass(input.lean_body_mass_kg)?;
            if let Some(weight) = input.weight_kg {
                let weight = validate_weight(weight)?;
                if lean_body_mass > weight {
                    return Err(CalcError::InvalidInput(
                        "lean_body_mass_kg cannot be greater than weight_kg".into(),
                    ));
                }
            }
            if let Some(age) = input.age {
                validate_age(age)?;
            }
            if let Some(height) = input.height_cm {
                validate_height(height)?;
            }
            (500.0 + 22.0 * lean_body_mass, None)
        }
    };

    if basal_raw <= 0.0 || !basal_raw.is_finite() {
        return Err(CalcError::InvalidInput(
            "equation produced a non-positive basal energy estimate".into(),
        ));
    }

    let maintenance_raw = basal_raw * activity_factor.unwrap_or(1.0);
    let target_raw = maintenance_raw + adjustment;
    if target_raw <= 0.0 || !target_raw.is_finite() {
        return Err(CalcError::InvalidInput(
            "calorie_adjustment_kcal_day produces a non-positive target".into(),
        ));
    }

    let basal_kcal_day = round_kcal(basal_raw)?;
    let maintenance_kcal_day = round_kcal(maintenance_raw)?;
    let target_kcal_day = round_kcal(target_raw)?;
    let interpretation = interpretation(
        input.equation,
        basal_kcal_day,
        maintenance_kcal_day,
        target_kcal_day,
        activity_factor,
        adjustment,
    );

    Ok(EnergyRequirementOutcome {
        equation: input.equation,
        basal_kcal_day,
        maintenance_kcal_day,
        target_kcal_day,
        activity_factor,
        calorie_adjustment_kcal_day: adjustment,
        schofield_age_band,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &EnergyRequirementInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("equation".into(), json!(o.equation.slug()));
    working.insert("basal_kcal_day".into(), json!(o.basal_kcal_day));
    if let Some(factor) = o.activity_factor {
        working.insert("activity_factor".into(), json!(factor));
        working.insert("maintenance_kcal_day".into(), json!(o.maintenance_kcal_day));
    }
    if input.calorie_adjustment_kcal_day.is_some() {
        working.insert(
            "calorie_adjustment_kcal_day".into(),
            json!(o.calorie_adjustment_kcal_day),
        );
    }
    if o.activity_factor.is_some() || input.calorie_adjustment_kcal_day.is_some() {
        working.insert("target_kcal_day".into(), json!(o.target_kcal_day));
    }
    if let Some(band) = o.schofield_age_band {
        working.insert("schofield_age_band".into(), json!(band.slug()));
    }
    let (result_label, result_value) = result_summary(&o);
    working.insert("result_label".into(), json!(result_label));
    working.insert("unit".into(), json!("kcal/day"));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(result_value),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn result_summary(outcome: &EnergyRequirementOutcome) -> (&'static str, u32) {
    if outcome.calorie_adjustment_kcal_day != 0.0 {
        ("Target intake", outcome.target_kcal_day)
    } else if outcome.activity_factor.is_some() {
        ("TDEE", outcome.maintenance_kcal_day)
    } else {
        ("BMR/RMR", outcome.basal_kcal_day)
    }
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct EnergyRequirement;

impl Calculator for EnergyRequirement {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Energy Requirement (BMR/RMR/TDEE)"
    }

    fn description(&self) -> &'static str {
        "Adult basal/resting energy estimate using Mifflin-St Jeor, Harris-Benedict, Schofield, or Cunningham, with optional activity factor and calorie target adjustment."
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
            "title": "EnergyRequirementInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["equation"],
            "description": "Adult basal/resting energy estimate. Conditional requirements: Mifflin-St Jeor and both Harris-Benedict equations require sex, age, weight_kg, and height_cm; Schofield requires sex, age, and weight_kg; Cunningham requires lean_body_mass_kg.",
            "allOf": [
                {
                    "if": { "properties": { "equation": { "const": "mifflin_st_jeor" } } },
                    "then": { "required": ["sex", "age", "weight_kg", "height_cm"] }
                },
                {
                    "if": { "properties": { "equation": { "const": "harris_benedict_original" } } },
                    "then": { "required": ["sex", "age", "weight_kg", "height_cm"] }
                },
                {
                    "if": { "properties": { "equation": { "const": "harris_benedict_revised" } } },
                    "then": { "required": ["sex", "age", "weight_kg", "height_cm"] }
                },
                {
                    "if": { "properties": { "equation": { "const": "schofield" } } },
                    "then": { "required": ["sex", "age", "weight_kg"] }
                },
                {
                    "if": { "properties": { "equation": { "const": "cunningham" } } },
                    "then": { "required": ["lean_body_mass_kg"] }
                }
            ],
            "properties": {
                "equation": {
                    "type": "string",
                    "enum": ["mifflin_st_jeor", "harris_benedict_original", "harris_benedict_revised", "schofield", "cunningham"],
                    "description": "Predictive equation to use: Mifflin-St Jeor, Harris-Benedict original, Harris-Benedict revised/Roza-Shizgal, Schofield adult, or Cunningham lean-body-mass equation"
                },
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Sex used to select equation coefficients. Required except for Cunningham.",
                    "definition": {
                        "concept": "Sex for energy equation coefficient",
                        "statement": "Most adult BMR/RMR predictive equations were fitted with male and female coefficient sets. Use the sex coefficient that matches the source equation and clinical context.",
                        "caveats": "The equations predate modern sex and gender recording. In trans, intersex, or otherwise complex physiology, treat the result as an estimate and apply clinician or dietitian judgement.",
                        "source": {
                            "citation": "Mifflin MD et al. Am J Clin Nutr. 1990;51(2):241-247; Roza AM, Shizgal HM. Am J Clin Nutr. 1984;40(1):168-182.",
                            "url": "https://doi.org/10.1093/ajcn/51.2.241"
                        },
                        "status": "draft"
                    }
                },
                "age": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 120,
                    "description": "Adult age in years. Required except for Cunningham. Schofield uses adult bands 18-29, 30-59, and 60+."
                },
                "weight_kg": {
                    "type": "number",
                    "minimum": 20,
                    "maximum": 500,
                    "description": "Body weight in kilograms. Required except for Cunningham."
                },
                "height_cm": {
                    "type": "number",
                    "minimum": 50,
                    "maximum": 250,
                    "description": "Height in centimetres. Required for Mifflin-St Jeor and both Harris-Benedict variants."
                },
                "lean_body_mass_kg": {
                    "type": "number",
                    "minimum": 5,
                    "maximum": 250,
                    "description": "Lean body mass in kilograms. Required for Cunningham; should not exceed weight_kg if weight_kg is also supplied."
                },
                "activity_factor": {
                    "type": "number",
                    "minimum": 1.0,
                    "maximum": 3.0,
                    "description": "Optional physical activity level / activity factor. Common values: sedentary 1.2, light 1.375, moderate 1.55, very active 1.725, extra active 1.9. FAO/WHO/UNU uses PAL ranges around 1.40-1.69 sedentary/light, 1.70-1.99 active/moderate, 2.00-2.40 vigorous."
                },
                "calorie_adjustment_kcal_day": {
                    "type": "number",
                    "minimum": -2000,
                    "maximum": 2000,
                    "description": "Optional target adjustment in kcal/day: negative for weight loss, positive for weight gain. Applied after the activity factor if one is supplied."
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: EnergyRequirementInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

fn required_sex(sex: Option<Sex>, equation: Equation) -> Result<Sex, CalcError> {
    sex.ok_or_else(|| CalcError::InvalidInput(format!("sex is required for {}", equation.label())))
}

fn required_age(age: Option<u8>, equation: Equation) -> Result<u8, CalcError> {
    let age = age.ok_or_else(|| {
        CalcError::InvalidInput(format!("age is required for {}", equation.label()))
    })?;
    validate_age(age)
}

fn required_weight(weight: Option<f64>, equation: Equation) -> Result<f64, CalcError> {
    let weight = weight.ok_or_else(|| {
        CalcError::InvalidInput(format!("weight_kg is required for {}", equation.label()))
    })?;
    validate_weight(weight)
}

fn required_height(height: Option<f64>, equation: Equation) -> Result<f64, CalcError> {
    debug_assert!(equation.needs_height());
    let height = height.ok_or_else(|| {
        CalcError::InvalidInput(format!("height_cm is required for {}", equation.label()))
    })?;
    validate_height(height)
}

fn required_lean_body_mass(lean_body_mass: Option<f64>) -> Result<f64, CalcError> {
    let lean_body_mass = lean_body_mass.ok_or_else(|| {
        CalcError::InvalidInput("lean_body_mass_kg is required for Cunningham".into())
    })?;
    if !(5.0..=250.0).contains(&lean_body_mass) || !lean_body_mass.is_finite() {
        return Err(CalcError::InvalidInput(
            "lean_body_mass_kg must be a finite number between 5 and 250 kg".into(),
        ));
    }
    Ok(lean_body_mass)
}

fn validate_age(age: u8) -> Result<u8, CalcError> {
    if !(18..=120).contains(&age) {
        return Err(CalcError::InvalidInput(
            "energy requirement equations here are adult-only (age 18-120)".into(),
        ));
    }
    Ok(age)
}

fn validate_weight(weight: f64) -> Result<f64, CalcError> {
    if !(20.0..=500.0).contains(&weight) || !weight.is_finite() {
        return Err(CalcError::InvalidInput(
            "weight_kg must be a finite number between 20 and 500 kg".into(),
        ));
    }
    Ok(weight)
}

fn validate_height(height: f64) -> Result<f64, CalcError> {
    if !(50.0..=250.0).contains(&height) || !height.is_finite() {
        return Err(CalcError::InvalidInput(
            "height_cm must be a finite number between 50 and 250 cm".into(),
        ));
    }
    Ok(height)
}

fn validate_activity_factor(activity_factor: Option<f64>) -> Result<Option<f64>, CalcError> {
    if let Some(factor) = activity_factor
        && (!(1.0..=3.0).contains(&factor) || !factor.is_finite())
    {
        return Err(CalcError::InvalidInput(
            "activity_factor must be a finite number between 1.0 and 3.0".into(),
        ));
    }
    Ok(activity_factor)
}

fn validate_adjustment(adjustment: Option<f64>) -> Result<f64, CalcError> {
    let adjustment = adjustment.unwrap_or(0.0);
    if !(-2000.0..=2000.0).contains(&adjustment) || !adjustment.is_finite() {
        return Err(CalcError::InvalidInput(
            "calorie_adjustment_kcal_day must be a finite number between -2000 and 2000".into(),
        ));
    }
    Ok(adjustment)
}

fn round_kcal(value: f64) -> Result<u32, CalcError> {
    let rounded = value.round();
    if rounded <= 0.0 || rounded > f64::from(u32::MAX) || !rounded.is_finite() {
        return Err(CalcError::InvalidInput(
            "energy estimate is outside representable kcal/day range".into(),
        ));
    }
    Ok(rounded as u32)
}

fn interpretation(
    equation: Equation,
    basal_kcal_day: u32,
    maintenance_kcal_day: u32,
    target_kcal_day: u32,
    activity_factor: Option<f64>,
    adjustment: f64,
) -> String {
    let basis = if activity_factor.is_some() && adjustment != 0.0 {
        format!(
            "Estimated target intake {target_kcal_day} kcal/day after activity and calorie adjustment. Basal/resting estimate by {} is {basal_kcal_day} kcal/day; estimated maintenance/TDEE is {maintenance_kcal_day} kcal/day.",
            equation.label()
        )
    } else if activity_factor.is_some() {
        format!(
            "Estimated maintenance/TDEE {maintenance_kcal_day} kcal/day using activity factor. Basal/resting estimate by {} is {basal_kcal_day} kcal/day.",
            equation.label()
        )
    } else if adjustment != 0.0 {
        format!(
            "Estimated target intake {target_kcal_day} kcal/day after calorie adjustment. Basal/resting estimate by {} is {basal_kcal_day} kcal/day.",
            equation.label()
        )
    } else {
        format!(
            "Estimated basal/resting energy expenditure {basal_kcal_day} kcal/day by {}.",
            equation.label()
        )
    };

    format!(
        "{basis} This is a predictive estimate, not indirect calorimetry. Weight change varies with adherence, adaptive thermogenesis, fluid/glycogen shifts, activity, illness, pregnancy/lactation, medications, and body composition. Use clinician or dietitian judgement for serious illness, eating disorders, pregnancy, children, frailty, or complex nutritional care."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(equation: Equation) -> EnergyRequirementInput {
        EnergyRequirementInput {
            equation,
            sex: Some(Sex::Male),
            age: Some(30),
            weight_kg: Some(70.0),
            height_cm: Some(175.0),
            lean_body_mass_kg: None,
            activity_factor: None,
            calorie_adjustment_kcal_day: None,
        }
    }

    #[test]
    fn mifflin_st_jeor_male_and_female() {
        let male = compute(&input(Equation::MifflinStJeor)).unwrap();
        assert_eq!(male.basal_kcal_day, 1649);
        assert_eq!(male.target_kcal_day, 1649);

        let mut female_input = input(Equation::MifflinStJeor);
        female_input.sex = Some(Sex::Female);
        let female = compute(&female_input).unwrap();
        assert_eq!(female.basal_kcal_day, 1483);
    }

    #[test]
    fn harris_benedict_original_male_and_female() {
        let male = compute(&input(Equation::HarrisBenedictOriginal)).unwrap();
        assert_eq!(male.basal_kcal_day, 1702);

        let mut female_input = input(Equation::HarrisBenedictOriginal);
        female_input.sex = Some(Sex::Female);
        let female = compute(&female_input).unwrap();
        assert_eq!(female.basal_kcal_day, 1508);
    }

    #[test]
    fn harris_benedict_revised_male_and_female() {
        let male = compute(&input(Equation::HarrisBenedictRevised)).unwrap();
        assert_eq!(male.basal_kcal_day, 1696);

        let mut female_input = input(Equation::HarrisBenedictRevised);
        female_input.sex = Some(Sex::Female);
        let female = compute(&female_input).unwrap();
        assert_eq!(female.basal_kcal_day, 1507);
    }

    #[test]
    fn schofield_adult_age_bands_and_boundaries() {
        let mut i = input(Equation::Schofield);
        i.height_cm = None;

        i.age = Some(29);
        let age_29 = compute(&i).unwrap();
        assert_eq!(age_29.basal_kcal_day, 1746);
        assert_eq!(age_29.schofield_age_band, Some(AgeBand::Age18To29));

        i.age = Some(30);
        let age_30 = compute(&i).unwrap();
        assert_eq!(age_30.basal_kcal_day, 1676);
        assert_eq!(age_30.schofield_age_band, Some(AgeBand::Age30To59));

        i.age = Some(59);
        let age_59 = compute(&i).unwrap();
        assert_eq!(age_59.schofield_age_band, Some(AgeBand::Age30To59));

        i.age = Some(60);
        let age_60 = compute(&i).unwrap();
        assert_eq!(age_60.basal_kcal_day, 1407);
        assert_eq!(age_60.schofield_age_band, Some(AgeBand::Age60Plus));
    }

    #[test]
    fn schofield_female_coefficients() {
        let mut i = input(Equation::Schofield);
        i.sex = Some(Sex::Female);
        i.height_cm = None;

        i.age = Some(29);
        assert_eq!(compute(&i).unwrap().basal_kcal_day, 1524);

        i.age = Some(30);
        assert_eq!(compute(&i).unwrap().basal_kcal_day, 1414);

        i.age = Some(60);
        assert_eq!(compute(&i).unwrap().basal_kcal_day, 1294);
    }

    #[test]
    fn cunningham_uses_lean_body_mass() {
        let i = EnergyRequirementInput {
            equation: Equation::Cunningham,
            sex: None,
            age: None,
            weight_kg: None,
            height_cm: None,
            lean_body_mass_kg: Some(60.0),
            activity_factor: None,
            calorie_adjustment_kcal_day: None,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.basal_kcal_day, 1820);
        assert_eq!(o.target_kcal_day, 1820);
    }

    #[test]
    fn activity_factor_and_adjustment_produce_target() {
        let mut i = input(Equation::MifflinStJeor);
        i.activity_factor = Some(1.55);
        i.calorie_adjustment_kcal_day = Some(-500.0);
        let o = compute(&i).unwrap();
        assert_eq!(o.basal_kcal_day, 1649);
        assert_eq!(o.maintenance_kcal_day, 2556);
        assert_eq!(o.target_kcal_day, 2056);
    }

    #[test]
    fn response_result_label_matches_supplied_energy_mode() {
        let basal = build_response(&input(Equation::MifflinStJeor)).unwrap();
        assert_eq!(basal.result, json!(1649));
        assert_eq!(basal.working["result_label"], "BMR/RMR");
        assert_eq!(basal.working["basal_kcal_day"], 1649);

        let mut tdee_input = input(Equation::MifflinStJeor);
        tdee_input.activity_factor = Some(1.55);
        let tdee = build_response(&tdee_input).unwrap();
        assert_eq!(tdee.result, json!(2556));
        assert_eq!(tdee.working["result_label"], "TDEE");
        assert_eq!(tdee.working["basal_kcal_day"], 1649);
        assert_eq!(tdee.working["maintenance_kcal_day"], 2556);

        tdee_input.calorie_adjustment_kcal_day = Some(-500.0);
        let target = build_response(&tdee_input).unwrap();
        assert_eq!(target.result, json!(2056));
        assert_eq!(target.working["result_label"], "Target intake");
        assert_eq!(target.working["maintenance_kcal_day"], 2556);
        assert_eq!(target.working["target_kcal_day"], 2056);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "equation": "mifflin_st_jeor",
            "sex": "male",
            "age": 30,
            "weight_kg": 70.0,
            "height_cm": 175.0,
            "activity_factor": 1.55,
            "calorie_adjustment_kcal_day": -500.0
        });
        let dynamic = EnergyRequirement.calculate(&value).unwrap();
        let mut typed_input = input(Equation::MifflinStJeor);
        typed_input.activity_factor = Some(1.55);
        typed_input.calorie_adjustment_kcal_day = Some(-500.0);
        let typed = build_response(&typed_input).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(2056));
    }

    #[test]
    fn rejects_missing_inputs_per_equation() {
        let mut missing_height = input(Equation::MifflinStJeor);
        missing_height.height_cm = None;
        assert!(compute(&missing_height).is_err());

        let mut missing_weight = input(Equation::Schofield);
        missing_weight.weight_kg = None;
        missing_weight.height_cm = None;
        assert!(compute(&missing_weight).is_err());

        let missing_lbm = EnergyRequirementInput {
            equation: Equation::Cunningham,
            sex: None,
            age: None,
            weight_kg: None,
            height_cm: None,
            lean_body_mass_kg: None,
            activity_factor: None,
            calorie_adjustment_kcal_day: None,
        };
        assert!(compute(&missing_lbm).is_err());
    }

    #[test]
    fn rejects_out_of_range_values() {
        let mut bad_age = input(Equation::MifflinStJeor);
        bad_age.age = Some(17);
        assert!(compute(&bad_age).is_err());

        let mut bad_activity = input(Equation::MifflinStJeor);
        bad_activity.activity_factor = Some(3.1);
        assert!(compute(&bad_activity).is_err());

        let mut bad_adjustment = input(Equation::MifflinStJeor);
        bad_adjustment.calorie_adjustment_kcal_day = Some(-5000.0);
        assert!(compute(&bad_adjustment).is_err());
    }

    #[test]
    fn schema_documents_equation_options_and_caveats() {
        let schema = EnergyRequirement.input_schema();
        let equations = schema["properties"]["equation"]["enum"].as_array().unwrap();
        assert_eq!(equations.len(), 5);
        assert!(
            schema["description"]
                .as_str()
                .unwrap()
                .contains("Cunningham requires")
        );
        assert!(
            schema["properties"]["sex"]["definition"]["caveats"]
                .as_str()
                .unwrap()
                .contains("trans")
        );
    }
}
