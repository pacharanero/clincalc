// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! MELD 3.0 under current OPTN allocation rules for candidates registered at
//! age 12 years or older.
//!
//! The published 2021 equation adds sodium, albumin, and an adult female term
//! to MELD, with bilirubin-sodium and albumin-creatinine interactions. Current
//! OPTN policy gives candidates registered aged 12-17 the same 1.33-point term
//! regardless of sex, rounds the result to an integer, and constrains the
//! allocation score to 6-40. The uncapped policy-formula result is retained in
//! the outcome and response working so these two contracts are not conflated.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "meld_3";
pub const REFERENCE: &str = "Kim WR, Mannalithara A, Heimbach JK, et al. MELD 3.0: The Model for End-Stage Liver Disease Updated for the Modern Era. Gastroenterology. 2021;161(6):1887-1895.e4. doi:10.1053/j.gastro.2021.08.050. Allocation bounds and age handling per current OPTN MELD calculator policy.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Uncopyrightable method under 17 U.S.C. Section 102(b) - independently implemented from primary literature",
    source_url: "https://uscode.house.gov/view.xhtml?req=granuleid:USC-prelim-title17-section102&num=0&edition=prelim",
};

pub const BILIRUBIN_UMOL_PER_MGDL: f64 = 17.1;
pub const CREATININE_UMOL_PER_MGDL: f64 = 88.4;
pub const BILIRUBIN_MIN_MGDL: f64 = 1.0;
pub const INR_MIN: f64 = 1.0;
pub const CREATININE_MIN_MGDL: f64 = 1.0;
pub const CREATININE_MAX_MGDL: f64 = 3.0;
pub const SODIUM_MIN_MMOL_L: f64 = 125.0;
pub const SODIUM_MAX_MMOL_L: f64 = 137.0;
pub const ALBUMIN_MIN_G_DL: f64 = 1.5;
pub const ALBUMIN_MAX_G_DL: f64 = 3.5;
pub const SEX_OR_ADOLESCENT_POINTS: f64 = 1.33;
pub const SCORE_MIN: i32 = 6;
pub const SCORE_MAX: i32 = 40;
pub const REGISTRATION_AGE_MAX_YEARS: u16 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MassConcentrationUnit {
    #[serde(rename = "mg/dL")]
    MgDl,
    #[serde(rename = "umol/L")]
    UmolL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlbuminUnit {
    #[serde(rename = "g/dL")]
    GDl,
    #[serde(rename = "g/L")]
    GL,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meld3Input {
    /// Candidate age in completed years when registered for transplant.
    pub registration_age_years: u16,
    /// Whether the OPTN adult female coefficient applies. Ignored at ages
    /// 12-17, where policy applies the 1.33-point term regardless of sex.
    pub female_for_adult_meld: bool,
    pub bilirubin: f64,
    pub bilirubin_unit: MassConcentrationUnit,
    pub inr: f64,
    pub creatinine: f64,
    pub creatinine_unit: MassConcentrationUnit,
    pub sodium_mmol_l: f64,
    pub albumin: f64,
    pub albumin_unit: AlbuminUnit,
    /// True only for at least two dialysis treatments or at least 24 hours of
    /// CVVHD in the seven days before the creatinine test.
    pub qualifying_dialysis_in_prior_7_days: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Meld3Outcome {
    /// Current OPTN allocation score, rounded and constrained to 6-40.
    pub score: i32,
    /// Policy formula rounded to an integer without the OPTN 40 cap.
    pub rounded_uncapped_policy_score: i32,
    pub raw_policy_score: f64,
    pub bilirubin_mgdl_used: f64,
    pub inr_used: f64,
    pub creatinine_mgdl_used: f64,
    pub sodium_mmol_l_used: f64,
    pub albumin_g_dl_used: f64,
    pub sex_or_adolescent_points: f64,
    pub interpretation: String,
}

pub fn compute(input: &Meld3Input) -> Result<Meld3Outcome, CalcError> {
    if !(12..=REGISTRATION_AGE_MAX_YEARS).contains(&input.registration_age_years) {
        return Err(CalcError::InvalidInput(format!(
            "registration_age_years must be between 12 and {REGISTRATION_AGE_MAX_YEARS}; younger candidates use PELD rather than MELD 3.0"
        )));
    }

    for (name, value) in [
        ("bilirubin", input.bilirubin),
        ("inr", input.inr),
        ("creatinine", input.creatinine),
        ("sodium_mmol_l", input.sodium_mmol_l),
        ("albumin", input.albumin),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be a finite positive number"
            )));
        }
    }

    let bilirubin_mgdl = match input.bilirubin_unit {
        MassConcentrationUnit::MgDl => input.bilirubin,
        MassConcentrationUnit::UmolL => input.bilirubin / BILIRUBIN_UMOL_PER_MGDL,
    };
    let creatinine_mgdl = match input.creatinine_unit {
        MassConcentrationUnit::MgDl => input.creatinine,
        MassConcentrationUnit::UmolL => input.creatinine / CREATININE_UMOL_PER_MGDL,
    };
    let albumin_g_dl = match input.albumin_unit {
        AlbuminUnit::GDl => input.albumin,
        AlbuminUnit::GL => input.albumin / 10.0,
    };

    let bilirubin_mgdl_used = bilirubin_mgdl.max(BILIRUBIN_MIN_MGDL);
    let inr_used = input.inr.max(INR_MIN);
    let creatinine_mgdl_used = if input.qualifying_dialysis_in_prior_7_days {
        CREATININE_MAX_MGDL
    } else {
        creatinine_mgdl.clamp(CREATININE_MIN_MGDL, CREATININE_MAX_MGDL)
    };
    let sodium_mmol_l_used = input
        .sodium_mmol_l
        .clamp(SODIUM_MIN_MMOL_L, SODIUM_MAX_MMOL_L);
    let albumin_g_dl_used = albumin_g_dl.clamp(ALBUMIN_MIN_G_DL, ALBUMIN_MAX_G_DL);
    let sex_or_adolescent_points =
        if input.registration_age_years < 18 || input.female_for_adult_meld {
            SEX_OR_ADOLESCENT_POINTS
        } else {
            0.0
        };

    let log_bilirubin = bilirubin_mgdl_used.ln();
    let log_creatinine = creatinine_mgdl_used.ln();
    let sodium_deficit = SODIUM_MAX_MMOL_L - sodium_mmol_l_used;
    let albumin_deficit = ALBUMIN_MAX_G_DL - albumin_g_dl_used;
    let raw_policy_score = sex_or_adolescent_points + 4.56 * log_bilirubin + 0.82 * sodium_deficit
        - 0.24 * sodium_deficit * log_bilirubin
        + 9.09 * inr_used.ln()
        + 11.14 * log_creatinine
        + 1.85 * albumin_deficit
        - 1.83 * albumin_deficit * log_creatinine
        + 6.0;

    if !raw_policy_score.is_finite() {
        return Err(CalcError::InvalidInput(
            "inputs produce a non-finite MELD 3.0 result".into(),
        ));
    }

    let rounded_uncapped_policy_score = raw_policy_score.round() as i32;
    let score = rounded_uncapped_policy_score.clamp(SCORE_MIN, SCORE_MAX);
    let interpretation = format!(
        "OPTN MELD 3.0 allocation score {score} (nearest integer, constrained to {SCORE_MIN}-{SCORE_MAX}); the uncapped policy formula rounds to {rounded_uncapped_policy_score}. Higher scores indicate greater short-term waitlist mortality risk. This local calculation does not assign transplant priority and does not include Status 1, exception scores, laboratory reporting rules, or other allocation factors; the official OPTN system and current policy are authoritative."
    );

    Ok(Meld3Outcome {
        score,
        rounded_uncapped_policy_score,
        raw_policy_score,
        bilirubin_mgdl_used,
        inr_used,
        creatinine_mgdl_used,
        sodium_mmol_l_used,
        albumin_g_dl_used,
        sex_or_adolescent_points,
        interpretation,
    })
}

pub fn build_response(input: &Meld3Input) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "registration_age_years".into(),
        json!(input.registration_age_years),
    );
    working.insert(
        "female_for_adult_meld".into(),
        json!(input.female_for_adult_meld),
    );
    working.insert(
        "qualifying_dialysis_in_prior_7_days".into(),
        json!(input.qualifying_dialysis_in_prior_7_days),
    );
    working.insert(
        "bilirubin_mgdl_used".into(),
        json!(outcome.bilirubin_mgdl_used),
    );
    working.insert("inr_used".into(), json!(outcome.inr_used));
    working.insert(
        "creatinine_mgdl_used".into(),
        json!(outcome.creatinine_mgdl_used),
    );
    working.insert(
        "sodium_mmol_l_used".into(),
        json!(outcome.sodium_mmol_l_used),
    );
    working.insert("albumin_g_dl_used".into(), json!(outcome.albumin_g_dl_used));
    working.insert(
        "sex_or_adolescent_points".into(),
        json!(outcome.sex_or_adolescent_points),
    );
    working.insert("raw_policy_score".into(), json!(outcome.raw_policy_score));
    working.insert(
        "rounded_uncapped_policy_score".into(),
        json!(outcome.rounded_uncapped_policy_score),
    );
    working.insert("optn_score".into(), json!(outcome.score));
    working.insert("score_min".into(), json!(SCORE_MIN));
    working.insert("score_max".into(), json!(SCORE_MAX));
    working.insert(
        "limitations".into(),
        json!("Short-term waitlist mortality ranking model, not a diagnosis, individual mortality percentage, treatment instruction, or complete transplant-allocation decision. INR may be misleading during anticoagulation, and albumin administration may alter the measured albumin."),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.score),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Meld3;

impl Calculator for Meld3 {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "MELD 3.0 (OPTN)"
    }

    fn description(&self) -> &'static str {
        "Current OPTN MELD 3.0 allocation score for liver-transplant candidates registered at age 12 or older, preserving the uncapped policy-formula result in the calculation working."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        let paper = json!({
            "citation": "Kim WR et al. Gastroenterology. 2021;161(6):1887-1895.e4.",
            "url": "https://doi.org/10.1053/j.gastro.2021.08.050"
        });
        let optn = json!({
            "citation": "OPTN MELD calculator and current liver allocation policy.",
            "url": "https://www.hrsa.gov/optn/data-calculators/allocation-calculators/meld-calculator"
        });
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Meld3Input",
            "description": "Calculates current OPTN MELD 3.0 for candidates registered at age 12 or older. It also preserves the uncapped policy-formula score in working. This is not an official allocation determination.",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "registration_age_years", "female_for_adult_meld", "bilirubin",
                "bilirubin_unit", "inr", "creatinine", "creatinine_unit",
                "sodium_mmol_l", "albumin", "albumin_unit",
                "qualifying_dialysis_in_prior_7_days"
            ],
            "properties": {
                "registration_age_years": {
                    "type": "integer", "minimum": 12, "maximum": REGISTRATION_AGE_MAX_YEARS,
                    "description": "Age in completed years at transplant registration, not current age. Candidates under 12 use PELD.",
                    "definition": {
                        "concept": "Registration age for MELD 3.0",
                        "statement": "Use age in completed years when the candidate was registered. OPTN applies the 1.33-point term to every candidate registered aged 12-17 and uses the adult sex coefficient from age 18.",
                        "excludes": ["Current age when it differs from age at registration", "Candidates registered before age 12"],
                        "source": optn, "status": "draft"
                    }
                },
                "female_for_adult_meld": {
                    "type": "boolean",
                    "description": "Whether the OPTN adult female coefficient applies. Used only at registration age 18 or older; candidates aged 12-17 receive the same 1.33 points regardless.",
                    "definition": {
                        "concept": "Adult MELD female coefficient",
                        "statement": "For a candidate registered at age 18 or older, set true only when the female coefficient applies under the candidate's OPTN record. At ages 12-17 the calculator applies 1.33 points regardless of this value.",
                        "caveats": "This is the source equation's allocation coefficient, not a general inference about gender identity or physiology.",
                        "source": optn, "status": "draft"
                    }
                },
                "bilirubin": {
                    "type": "number", "exclusiveMinimum": 0,
                    "description": "Total serum bilirubin in bilirubin_unit; values below 1.0 mg/dL are set to 1.0."
                },
                "bilirubin_unit": {
                    "type": "string", "enum": ["mg/dL", "umol/L"],
                    "description": "Unit of total bilirubin; 1 mg/dL = 17.1 umol/L.",
                    "definition": {
                        "concept": "Total bilirubin unit",
                        "statement": "The equation uses total bilirubin in mg/dL. A value labelled umol/L is converted internally using 17.1 umol/L per mg/dL.",
                        "excludes": ["Direct bilirubin", "A umol/L value labelled as mg/dL or vice versa"],
                        "source": paper, "status": "draft"
                    }
                },
                "inr": {
                    "type": "number", "exclusiveMinimum": 0,
                    "description": "International normalised ratio; values below 1.0 are set to 1.0."
                },
                "creatinine": {
                    "type": "number", "exclusiveMinimum": 0,
                    "description": "Serum creatinine in creatinine_unit; constrained to 1.0-3.0 mg/dL before the equation."
                },
                "creatinine_unit": {
                    "type": "string", "enum": ["mg/dL", "umol/L"],
                    "description": "Unit of serum creatinine; 1 mg/dL = 88.4 umol/L.",
                    "definition": {
                        "concept": "Serum creatinine unit",
                        "statement": "The equation uses serum creatinine in mg/dL. A value labelled umol/L is converted internally using 88.4 umol/L per mg/dL.",
                        "excludes": ["A umol/L value labelled as mg/dL or vice versa"],
                        "source": paper, "status": "draft"
                    }
                },
                "sodium_mmol_l": {
                    "type": "number", "exclusiveMinimum": 0, "unit": "mmol/L",
                    "description": "Serum sodium in mmol/L; constrained to 125-137. mmol/L and mEq/L are numerically equivalent for sodium."
                },
                "albumin": {
                    "type": "number", "exclusiveMinimum": 0,
                    "description": "Serum albumin in albumin_unit; constrained to 1.5-3.5 g/dL before the equation."
                },
                "albumin_unit": {
                    "type": "string", "enum": ["g/dL", "g/L"],
                    "description": "Unit of serum albumin; g/L is divided by 10 to obtain g/dL.",
                    "definition": {
                        "concept": "Serum albumin unit",
                        "statement": "The equation uses serum albumin in g/dL. A value labelled g/L is converted internally by dividing by 10.",
                        "excludes": ["A g/L value labelled as g/dL or vice versa"],
                        "source": paper, "status": "draft"
                    }
                },
                "qualifying_dialysis_in_prior_7_days": {
                    "type": "boolean",
                    "description": "True only for at least two dialysis treatments or at least 24 hours of CVVHD in the 7 days before the creatinine test; sets creatinine to 3.0 mg/dL.",
                    "definition": {
                        "concept": "Qualifying renal replacement therapy for MELD 3.0",
                        "statement": "Set true when the candidate received at least two dialysis treatments or at least 24 hours of continuous veno-venous haemodialysis in the seven days before the serum creatinine test.",
                        "includes": ["At least two dialysis treatments in the prior seven days", "At least 24 hours of CVVHD in the prior seven days"],
                        "excludes": ["A single intermittent dialysis treatment", "Dialysis outside the seven-day lookback", "Any dialysis history without confirming the threshold"],
                        "caveats": "A true value substitutes creatinine 3.0 mg/dL regardless of the measured result.",
                        "source": optn, "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: Meld3Input = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        female: bool,
        bilirubin: f64,
        sodium: f64,
        inr: f64,
        creatinine: f64,
        albumin: f64,
    ) -> Meld3Input {
        Meld3Input {
            registration_age_years: 40,
            female_for_adult_meld: female,
            bilirubin,
            bilirubin_unit: MassConcentrationUnit::MgDl,
            inr,
            creatinine,
            creatinine_unit: MassConcentrationUnit::MgDl,
            sodium_mmol_l: sodium,
            albumin,
            albumin_unit: AlbuminUnit::GDl,
            qualifying_dialysis_in_prior_7_days: false,
        }
    }

    #[test]
    fn primary_paper_table_four_vectors() {
        let vectors = [
            (false, 2.5, 131.0, 1.0, 1.2, 3.8, 16),
            (true, 2.5, 131.0, 1.0, 1.2, 3.8, 17),
            (false, 6.0, 131.0, 1.5, 1.5, 3.5, 25),
            (false, 6.0, 131.0, 1.5, 1.5, 2.2, 26),
            (true, 6.0, 131.0, 1.5, 1.5, 2.2, 27),
            (false, 12.0, 128.0, 2.2, 1.8, 2.0, 34),
            (false, 12.0, 128.0, 2.2, 2.8, 2.0, 38),
            (true, 12.0, 128.0, 2.2, 2.8, 2.0, 39),
        ];

        for (female, bilirubin, sodium, inr, creatinine, albumin, expected) in vectors {
            let outcome =
                compute(&input(female, bilirubin, sodium, inr, creatinine, albumin)).unwrap();
            assert_eq!(outcome.rounded_uncapped_policy_score, expected);
            assert_eq!(outcome.score, expected);
        }
    }

    #[test]
    fn primary_vector_raw_values_match_published_coefficients() {
        let low = compute(&input(false, 2.5, 131.0, 1.0, 1.2, 3.8)).unwrap();
        assert!((low.raw_policy_score - 15.809_889_226).abs() < 1e-9);
        assert_eq!(low.albumin_g_dl_used, 3.5);

        let high = compute(&input(true, 12.0, 128.0, 2.2, 2.8, 2.0)).unwrap();
        assert!((high.raw_policy_score - 39.259_508_372).abs() < 1e-9);
    }

    #[test]
    fn applies_all_published_input_bounds() {
        let outcome = compute(&input(false, 0.5, 120.0, 0.8, 0.5, 5.0)).unwrap();
        assert_eq!(outcome.bilirubin_mgdl_used, 1.0);
        assert_eq!(outcome.inr_used, 1.0);
        assert_eq!(outcome.creatinine_mgdl_used, 1.0);
        assert_eq!(outcome.sodium_mmol_l_used, 125.0);
        assert_eq!(outcome.albumin_g_dl_used, 3.5);

        let upper = compute(&input(false, 2.0, 150.0, 1.2, 8.0, 0.5)).unwrap();
        assert_eq!(upper.creatinine_mgdl_used, 3.0);
        assert_eq!(upper.sodium_mmol_l_used, 137.0);
        assert_eq!(upper.albumin_g_dl_used, 1.5);
    }

    #[test]
    fn qualifying_dialysis_forces_creatinine_to_three() {
        let mut dialysis = input(false, 2.0, 130.0, 1.5, 1.0, 2.5);
        dialysis.qualifying_dialysis_in_prior_7_days = true;
        let at_three = input(false, 2.0, 130.0, 1.5, 3.0, 2.5);
        assert_eq!(compute(&dialysis).unwrap(), compute(&at_three).unwrap());
    }

    #[test]
    fn adolescent_term_applies_regardless_of_adult_sex_flag() {
        let mut first = input(false, 2.5, 131.0, 1.0, 1.2, 3.5);
        first.registration_age_years = 12;
        let mut second = first;
        second.female_for_adult_meld = true;

        let first_outcome = compute(&first).unwrap();
        let second_outcome = compute(&second).unwrap();
        assert_eq!(first_outcome.sex_or_adolescent_points, 1.33);
        assert_eq!(
            first_outcome.raw_policy_score,
            second_outcome.raw_policy_score
        );
    }

    #[test]
    fn adult_female_term_adds_exactly_one_point_three_three() {
        let male = compute(&input(false, 6.0, 131.0, 1.5, 1.5, 2.2)).unwrap();
        let female = compute(&input(true, 6.0, 131.0, 1.5, 1.5, 2.2)).unwrap();
        assert!((female.raw_policy_score - male.raw_policy_score - 1.33).abs() < 1e-12);
    }

    #[test]
    fn preserves_uncapped_score_but_caps_optn_result() {
        let outcome = compute(&input(true, 100.0, 125.0, 10.0, 3.0, 1.5)).unwrap();
        assert!(outcome.rounded_uncapped_policy_score > 40);
        assert_eq!(outcome.score, 40);
    }

    #[test]
    fn alternate_units_match_published_units() {
        let expected = compute(&input(false, 2.5, 131.0, 1.0, 1.2, 3.5)).unwrap();
        let mut alternate = input(
            false,
            2.5 * BILIRUBIN_UMOL_PER_MGDL,
            131.0,
            1.0,
            1.2 * CREATININE_UMOL_PER_MGDL,
            35.0,
        );
        alternate.bilirubin_unit = MassConcentrationUnit::UmolL;
        alternate.creatinine_unit = MassConcentrationUnit::UmolL;
        alternate.albumin_unit = AlbuminUnit::GL;
        let actual = compute(&alternate).unwrap();
        assert!((actual.raw_policy_score - expected.raw_policy_score).abs() < 1e-12);
    }

    #[test]
    fn rejects_invalid_age_nonpositive_and_nonfinite_inputs() {
        let mut under_twelve = input(false, 2.0, 130.0, 1.5, 1.0, 2.5);
        under_twelve.registration_age_years = 11;
        assert!(compute(&under_twelve).is_err());

        let mut implausible_age = input(false, 2.0, 130.0, 1.5, 1.0, 2.5);
        implausible_age.registration_age_years = REGISTRATION_AGE_MAX_YEARS + 1;
        assert!(compute(&implausible_age).is_err());

        let mut zero = input(false, 2.0, 130.0, 1.5, 1.0, 2.5);
        zero.sodium_mmol_l = 0.0;
        assert!(compute(&zero).is_err());

        let mut nan = input(false, 2.0, 130.0, 1.5, 1.0, 2.5);
        nan.albumin = f64::NAN;
        assert!(compute(&nan).is_err());
    }

    #[test]
    fn dynamic_surface_matches_typed_calculation() {
        let value = json!({
            "registration_age_years": 40,
            "female_for_adult_meld": false,
            "bilirubin": 6.0,
            "bilirubin_unit": "mg/dL",
            "inr": 1.5,
            "creatinine": 1.5,
            "creatinine_unit": "mg/dL",
            "sodium_mmol_l": 131.0,
            "albumin": 3.5,
            "albumin_unit": "g/dL",
            "qualifying_dialysis_in_prior_7_days": false
        });
        let dynamic = Meld3.calculate(&value).unwrap();
        let typed = build_response(&input(false, 6.0, 131.0, 1.5, 1.5, 3.5)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(25));
    }

    #[test]
    fn schema_records_dialysis_exclusions_and_age_rule() {
        let schema = Meld3.input_schema();
        assert!(
            schema["properties"]["qualifying_dialysis_in_prior_7_days"]["definition"]["excludes"]
                [0]
            .as_str()
            .unwrap()
            .contains("single")
        );
        assert!(
            schema["properties"]["registration_age_years"]["definition"]["statement"]
                .as_str()
                .unwrap()
                .contains("12-17")
        );
    }
}
