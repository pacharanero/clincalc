// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Khorana score for chemotherapy-associated venous thromboembolism (VTE).
//!
//! The score uses pretreatment cancer site, full blood count, and BMI, plus
//! erythropoiesis-stimulating-agent use associated with the chemotherapy
//! course. It applies to adult ambulatory patients before a new chemotherapy
//! regimen. The original low, intermediate, and high bands describe observed
//! short-term symptomatic VTE risk; the later score >=2 guideline threshold is
//! reported separately and does not relabel an original score of 2 as high.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "khorana";

/// Original derivation/validation publication and guideline context.
pub const REFERENCE: &str = "Khorana AA, Kuderer NM, Culakova E, Lyman GH, Francis CW. Development and validation of a predictive model for chemotherapy-associated thrombosis. Blood. 2008;111(10):4902-4907. doi:10.1182/blood-2007-10-116327. PMID:18216292. Key NS, Khorana AA, Kuderer NM, et al. Venous Thromboembolism Prophylaxis and Treatment in Patients With Cancer: ASCO Clinical Practice Guideline Update. J Clin Oncol. 2020;38(5):496-520. doi:10.1200/JCO.19.01461. Farge D, Frere C, Connors JM, et al. 2022 international clinical practice guidelines for the treatment and prophylaxis of venous thromboembolism in patients with cancer, including patients with COVID-19. Lancet Oncol. 2022;23(7):e334-e347. doi:10.1016/S1470-2045(22)00160-7.";

/// Distribution licence: independently implemented from the published method.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published clinical scoring method - independently implemented from the primary literature",
    source_url: "https://doi.org/10.1182/blood-2007-10-116327",
};

/// The sole assessment context supported by this implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentContext {
    AdultAmbulatoryBeforeNewChemotherapyRegimen,
}

impl AssessmentContext {
    fn slug(self) -> &'static str {
        match self {
            Self::AdultAmbulatoryBeforeNewChemotherapyRegimen => {
                "adult_ambulatory_before_new_chemotherapy_regimen"
            }
        }
    }
}

/// Primary cancer site categories from Table 3 of Khorana et al.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CancerSite {
    Stomach,
    Pancreas,
    Lung,
    Lymphoma,
    Gynaecological,
    Bladder,
    Testicular,
    Other,
}

impl CancerSite {
    fn points(self) -> u8 {
        match self {
            Self::Stomach | Self::Pancreas => 2,
            Self::Lung
            | Self::Lymphoma
            | Self::Gynaecological
            | Self::Bladder
            | Self::Testicular => 1,
            Self::Other => 0,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::Stomach => "stomach",
            Self::Pancreas => "pancreas",
            Self::Lung => "lung",
            Self::Lymphoma => "lymphoma",
            Self::Gynaecological => "gynaecological",
            Self::Bladder => "bladder",
            Self::Testicular => "testicular",
            Self::Other => "other",
        }
    }
}

/// Inputs recorded before a new chemotherapy regimen, except that ESA use
/// means use associated with the chemotherapy course.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KhoranaInput {
    pub assessment_context: AssessmentContext,
    pub cancer_site: CancerSite,
    /// Pretreatment platelet count in 10^9/L.
    pub platelet_count_10_9_l: f64,
    /// Pretreatment haemoglobin in g/dL.
    pub haemoglobin_g_dl: f64,
    /// ESA use associated with the chemotherapy course.
    pub uses_erythropoiesis_stimulating_agent: bool,
    /// Pretreatment leukocyte count in 10^9/L.
    pub leukocyte_count_10_9_l: f64,
    /// Pretreatment BMI in kg/m^2.
    pub bmi_kg_m2: f64,
}

/// Original Khorana short-term symptomatic VTE risk band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginalRiskBand {
    Low,
    Intermediate,
    High,
}

impl OriginalRiskBand {
    fn from_score(score: u8) -> Self {
        match score {
            0 => Self::Low,
            1 | 2 => Self::Intermediate,
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

    fn original_observed_rates_percent(self) -> (f64, f64) {
        match self {
            Self::Low => (0.8, 0.3),
            Self::Intermediate => (1.8, 2.0),
            Self::High => (7.1, 6.7),
        }
    }
}

/// Typed Khorana outcome with each component retained.
#[derive(Debug, Clone, PartialEq)]
pub struct KhoranaOutcome {
    pub cancer_site_points: u8,
    pub platelet_count_points: u8,
    pub haemoglobin_or_esa_points: u8,
    pub leukocyte_count_points: u8,
    pub bmi_points: u8,
    /// Total score, 0-6.
    pub score: u8,
    pub original_risk_band: OriginalRiskBand,
    pub meets_guideline_consideration_threshold: bool,
    pub original_derivation_observed_rate_percent: f64,
    pub original_validation_observed_rate_percent: f64,
    pub interpretation: String,
}

/// Calculate the Khorana score.
pub fn compute(input: &KhoranaInput) -> Result<KhoranaOutcome, CalcError> {
    let counts = [input.platelet_count_10_9_l, input.leukocyte_count_10_9_l];
    if counts.iter().any(|value| !value.is_finite()) {
        return Err(CalcError::InvalidInput(
            "platelet_count_10_9_l and leukocyte_count_10_9_l must be finite numbers".into(),
        ));
    }
    if counts.iter().any(|value| *value < 0.0) {
        return Err(CalcError::InvalidInput(
            "platelet_count_10_9_l and leukocyte_count_10_9_l cannot be negative".into(),
        ));
    }
    if !input.haemoglobin_g_dl.is_finite() || input.haemoglobin_g_dl <= 0.0 {
        return Err(CalcError::InvalidInput(
            "haemoglobin_g_dl must be finite and positive".into(),
        ));
    }
    if !input.bmi_kg_m2.is_finite() || input.bmi_kg_m2 <= 0.0 {
        return Err(CalcError::InvalidInput(
            "bmi_kg_m2 must be finite and positive".into(),
        ));
    }

    let cancer_site_points = input.cancer_site.points();
    let platelet_count_points = u8::from(input.platelet_count_10_9_l >= 350.0);
    let haemoglobin_or_esa_points =
        u8::from(input.haemoglobin_g_dl < 10.0 || input.uses_erythropoiesis_stimulating_agent);
    let leukocyte_count_points = u8::from(input.leukocyte_count_10_9_l > 11.0);
    let bmi_points = u8::from(input.bmi_kg_m2 >= 35.0);
    let score = cancer_site_points
        + platelet_count_points
        + haemoglobin_or_esa_points
        + leukocyte_count_points
        + bmi_points;
    let original_risk_band = OriginalRiskBand::from_score(score);
    let meets_guideline_consideration_threshold = score >= 2;
    let (original_derivation_observed_rate_percent, original_validation_observed_rate_percent) =
        original_risk_band.original_observed_rates_percent();
    let threshold_text = if meets_guideline_consideration_threshold {
        "meets"
    } else {
        "does not meet"
    };

    let interpretation = format!(
        "Khorana score {score} of 6: original {} short-term symptomatic VTE risk band. In the original derivation and validation cohorts, this band had observed symptomatic VTE rates of {original_derivation_observed_rate_percent:.1}% and {original_validation_observed_rate_percent:.1}%, respectively, over median follow-up of approximately 2.5 months; these are historical cohort observations, not an individualised probability. This score {threshold_text} the current ASCO 2019 and ITAC 2022 score >=2 threshold for individualised thromboprophylaxis assessment; score 2 remains in the original intermediate band, and meeting the threshold is not automatic anticoagulation. Assess bleeding risk, contraindications, drug interactions, the chemotherapy regimen, prognosis, and patient preferences. This is not a VTE diagnostic test, an inpatient risk score, or a VTE treatment score, and a low score does not rule out VTE.",
        original_risk_band.slug(),
    );

    Ok(KhoranaOutcome {
        cancer_site_points,
        platelet_count_points,
        haemoglobin_or_esa_points,
        leukocyte_count_points,
        bmi_points,
        score,
        original_risk_band,
        meets_guideline_consideration_threshold,
        original_derivation_observed_rate_percent,
        original_validation_observed_rate_percent,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &KhoranaInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();

    working.insert(
        "assessment_context".into(),
        json!(input.assessment_context.slug()),
    );
    working.insert("cancer_site".into(), json!(input.cancer_site.slug()));
    working.insert(
        "cancer_site_points".into(),
        json!(outcome.cancer_site_points),
    );
    working.insert(
        "platelet_count_10_9_l".into(),
        json!(input.platelet_count_10_9_l),
    );
    working.insert("platelet_count_unit".into(), json!("x10^9/L"));
    working.insert(
        "platelet_count_points".into(),
        json!(outcome.platelet_count_points),
    );
    working.insert("haemoglobin_g_dl".into(), json!(input.haemoglobin_g_dl));
    working.insert("haemoglobin_unit".into(), json!("g/dL"));
    working.insert(
        "uses_erythropoiesis_stimulating_agent".into(),
        json!(input.uses_erythropoiesis_stimulating_agent),
    );
    working.insert(
        "haemoglobin_or_esa_points".into(),
        json!(outcome.haemoglobin_or_esa_points),
    );
    working.insert(
        "leukocyte_count_10_9_l".into(),
        json!(input.leukocyte_count_10_9_l),
    );
    working.insert("leukocyte_count_unit".into(), json!("x10^9/L"));
    working.insert(
        "leukocyte_count_points".into(),
        json!(outcome.leukocyte_count_points),
    );
    working.insert("bmi_kg_m2".into(), json!(input.bmi_kg_m2));
    working.insert("bmi_unit".into(), json!("kg/m^2"));
    working.insert("bmi_points".into(), json!(outcome.bmi_points));
    working.insert("total_score".into(), json!(outcome.score));
    working.insert("maximum_score".into(), json!(6));
    working.insert(
        "original_risk_band".into(),
        json!(outcome.original_risk_band.slug()),
    );
    working.insert(
        "meets_guideline_consideration_threshold".into(),
        json!(outcome.meets_guideline_consideration_threshold),
    );
    working.insert(
        "guideline_consideration_threshold".into(),
        json!("Khorana score >=2"),
    );
    working.insert(
        "original_derivation_cohort_observed_symptomatic_vte_rate_percent".into(),
        json!(outcome.original_derivation_observed_rate_percent),
    );
    working.insert(
        "original_validation_cohort_observed_symptomatic_vte_rate_percent".into(),
        json!(outcome.original_validation_observed_rate_percent),
    );
    working.insert(
        "original_observed_rate_time_horizon".into(),
        json!("median follow-up approximately 2.5 months"),
    );
    working.insert(
        "original_observed_rate_qualification".into(),
        json!("historical rates of symptomatic VTE observed in the original derivation and split-sample validation cohorts, not individualised current probabilities"),
    );
    working.insert(
        "intended_use".into(),
        json!("adult ambulatory patient before starting a new chemotherapy regimen"),
    );
    working.insert(
        "limitations".into(),
        json!("not a VTE diagnostic test, inpatient risk score, or VTE treatment score; a low score does not rule out VTE; thromboprophylaxis decisions require individual bleeding-risk, contraindication, interaction, regimen, prognosis, and preference assessment"),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.score),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn input_schema() -> Value {
    let primary_source = json!({
        "citation": "Khorana AA, Kuderer NM, Culakova E, Lyman GH, Francis CW. Blood. 2008;111(10):4902-4907. PMID:18216292.",
        "url": "https://doi.org/10.1182/blood-2007-10-116327"
    });
    let guideline_source = json!({
        "citation": "Key NS, Khorana AA, Kuderer NM, et al. ASCO Clinical Practice Guideline Update. J Clin Oncol. 2020;38(5):496-520; Farge D, Frere C, Connors JM, et al. ITAC 2022 guidelines. Lancet Oncol. 2022;23(7):e334-e347.",
        "url": "https://doi.org/10.1016/S1470-2045(22)00160-7"
    });

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "KhoranaInput",
        "description": "Khorana short-term symptomatic VTE risk score for an adult ambulatory patient before a new chemotherapy regimen. Enter pretreatment values, except that erythropoiesis-stimulating-agent use means use associated with the chemotherapy course. The original score bands and current score >=2 guideline consideration threshold are distinct. This score does not diagnose or exclude VTE and is not an inpatient or treatment score.",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "assessment_context", "cancer_site", "platelet_count_10_9_l",
            "haemoglobin_g_dl", "uses_erythropoiesis_stimulating_agent",
            "leukocyte_count_10_9_l", "bmi_kg_m2"
        ],
        "properties": {
            "assessment_context": {
                "type": "string",
                "const": "adult_ambulatory_before_new_chemotherapy_regimen",
                "description": "Exact supported context: an adult ambulatory patient assessed before starting a new chemotherapy regimen; required but not scored",
                "definition": {
                    "concept": "Khorana intended assessment context",
                    "statement": "Use for an adult ambulatory patient before starting a new chemotherapy regimen.",
                    "includes": ["Age 18 years or older", "Ambulatory cancer care", "Assessment before a new chemotherapy regimen"],
                    "excludes": ["Paediatric use", "Hospital inpatient VTE risk assessment", "Assessment after treatment values have changed", "Diagnosis of suspected VTE", "Selection of treatment for established VTE"],
                    "caveats": "The original cohorts comprised cancer outpatients initiating chemotherapy. Do not substitute this score for immediate diagnostic assessment when VTE is suspected.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "cancer_site": {
                "type": "string",
                "enum": ["stomach", "pancreas", "lung", "lymphoma", "gynaecological", "bladder", "testicular", "other"],
                "description": "Primary cancer site from Table 3: stomach/pancreas=2; lung/lymphoma/gynaecological/bladder/testicular=1; other=0. 'other' explicitly includes all sites not in Table 3, including renal, brain, prostate, breast, colorectal, head/neck, myeloma, and other genitourinary sites",
                "definition": {
                    "concept": "Khorana Table 3 primary cancer site",
                    "statement": "Select the single primary cancer site category exactly as specified in Table 3 of the original score.",
                    "includes": ["Stomach or pancreas: 2 points", "Lung, lymphoma, gynaecological, bladder, or testicular: 1 point", "Other: 0 points"],
                    "excludes": ["Do not award points to renal, brain, prostate, breast, colorectal, head/neck, myeloma, or another genitourinary site under this original Table 3 implementation", "Do not infer additional site points from metastatic disease or histological aggressiveness"],
                    "caveats": "The original cohorts underrepresented some high-thrombosis-risk cancers, including brain, renal, and myeloma. This implementation preserves Table 3 rather than extending it beyond the published score.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "platelet_count_10_9_l": {
                "type": "number",
                "minimum": 0,
                "unit": "x10^9/L",
                "description": "Pretreatment platelet count in x10^9/L; >=350 scores 1 point",
                "definition": {
                    "concept": "Prechemotherapy platelet count",
                    "statement": "Record the platelet count before chemotherapy; 350 x10^9/L or greater scores one point.",
                    "excludes": ["A platelet count measured only after the new regimen began", "A value in a different unit without conversion"],
                    "caveats": "Exactly 350 x10^9/L scores one point. No unsupported upper validity limit is imposed.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "haemoglobin_g_dl": {
                "type": "number",
                "exclusiveMinimum": 0,
                "unit": "g/dL",
                "description": "Pretreatment haemoglobin in g/dL; <10 scores 1 point together with the ESA criterion",
                "definition": {
                    "concept": "Prechemotherapy haemoglobin",
                    "statement": "Record pretreatment haemoglobin in g/dL. A value below 10 g/dL satisfies the combined haemoglobin-or-ESA criterion.",
                    "excludes": ["Do not enter g/L without dividing by 10", "Exactly 10 g/dL does not satisfy the strictly-below threshold"],
                    "caveats": "Haemoglobin below 10 g/dL and ESA use together still score only one point.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "uses_erythropoiesis_stimulating_agent": {
                "type": "boolean",
                "description": "Whether an erythropoiesis-stimulating agent is used in association with the chemotherapy course; haemoglobin <10 g/dL OR ESA use scores 1 point total",
                "definition": {
                    "concept": "Erythropoiesis-stimulating-agent use",
                    "statement": "True when an erythropoiesis-stimulating agent is used in association with the chemotherapy course assessed by this score.",
                    "includes": ["ESA use planned or given in association with this chemotherapy course"],
                    "excludes": ["Myeloid growth factors", "Remote ESA use unrelated to the chemotherapy course"],
                    "caveats": "This is the exception to the otherwise pretreatment inputs. ESA use and haemoglobin below 10 g/dL form one combined criterion and never score two points.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "leukocyte_count_10_9_l": {
                "type": "number",
                "minimum": 0,
                "unit": "x10^9/L",
                "description": "Pretreatment leukocyte count in x10^9/L; >11 scores 1 point",
                "definition": {
                    "concept": "Prechemotherapy leukocyte count",
                    "statement": "Record the leukocyte count before chemotherapy; a value strictly above 11 x10^9/L scores one point.",
                    "excludes": ["A leukocyte count measured only after the new regimen began", "Exactly 11 x10^9/L, which does not satisfy the strictly-above threshold"],
                    "caveats": "No unsupported upper validity limit is imposed.",
                    "source": primary_source,
                    "status": "draft"
                }
            },
            "bmi_kg_m2": {
                "type": "number",
                "exclusiveMinimum": 0,
                "unit": "kg/m^2",
                "description": "Pretreatment body mass index in kg/m^2; >=35 scores 1 point",
                "definition": {
                    "concept": "Pretreatment body mass index",
                    "statement": "Record pretreatment BMI; 35 kg/m^2 or greater scores one point.",
                    "excludes": ["Weight in kilograms without division by height in metres squared", "BMI calculated from values obtained only after the new regimen began"],
                    "caveats": "Exactly 35 kg/m^2 scores one point. No unsupported upper validity limit is imposed.",
                    "source": primary_source,
                    "status": "draft"
                }
            }
        },
        "guideline_context": {
            "statement": "ASCO 2019 and ITAC 2022 use Khorana score >=2 to identify ambulatory systemic-therapy patients for individualised thromboprophylaxis consideration, not automatic anticoagulation.",
            "caveats": "Assess bleeding risk, contraindications, drug interactions, regimen, prognosis, and patient preferences. The original score 2 band remains intermediate risk.",
            "source": guideline_source
        }
    })
}

/// Dynamic calculator implementation.
pub struct Khorana;

impl Calculator for Khorana {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Khorana Score"
    }

    fn description(&self) -> &'static str {
        "Short-term symptomatic VTE risk score for adult ambulatory patients assessed before a new chemotherapy regimen."
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
        let parsed: KhoranaInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::SupportedLocale;

    fn zero_input() -> KhoranaInput {
        KhoranaInput {
            assessment_context: AssessmentContext::AdultAmbulatoryBeforeNewChemotherapyRegimen,
            cancer_site: CancerSite::Other,
            platelet_count_10_9_l: 349.999,
            haemoglobin_g_dl: 10.0,
            uses_erythropoiesis_stimulating_agent: false,
            leukocyte_count_10_9_l: 11.0,
            bmi_kg_m2: 34.999,
        }
    }

    #[test]
    fn exact_numeric_threshold_pairs() {
        let mut input = zero_input();
        assert_eq!(compute(&input).unwrap().platelet_count_points, 0);
        input.platelet_count_10_9_l = 350.0;
        assert_eq!(compute(&input).unwrap().platelet_count_points, 1);

        input = zero_input();
        input.haemoglobin_g_dl = 9.999;
        assert_eq!(compute(&input).unwrap().haemoglobin_or_esa_points, 1);
        input.haemoglobin_g_dl = 10.0;
        assert_eq!(compute(&input).unwrap().haemoglobin_or_esa_points, 0);

        input = zero_input();
        input.leukocyte_count_10_9_l = 11.0;
        assert_eq!(compute(&input).unwrap().leukocyte_count_points, 0);
        input.leukocyte_count_10_9_l = 11.001;
        assert_eq!(compute(&input).unwrap().leukocyte_count_points, 1);

        input = zero_input();
        input.bmi_kg_m2 = 34.999;
        assert_eq!(compute(&input).unwrap().bmi_points, 0);
        input.bmi_kg_m2 = 35.0;
        assert_eq!(compute(&input).unwrap().bmi_points, 1);
    }

    #[test]
    fn haemoglobin_or_esa_is_one_combined_point() {
        let mut input = zero_input();
        input.uses_erythropoiesis_stimulating_agent = true;
        assert_eq!(compute(&input).unwrap().haemoglobin_or_esa_points, 1);

        input.haemoglobin_g_dl = 9.0;
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.haemoglobin_or_esa_points, 1);
        assert_eq!(outcome.score, 1);
    }

    #[test]
    fn every_cancer_site_scores_exact_table_three_points() {
        for (site, expected) in [
            (CancerSite::Stomach, 2),
            (CancerSite::Pancreas, 2),
            (CancerSite::Lung, 1),
            (CancerSite::Lymphoma, 1),
            (CancerSite::Gynaecological, 1),
            (CancerSite::Bladder, 1),
            (CancerSite::Testicular, 1),
            (CancerSite::Other, 0),
        ] {
            let input = KhoranaInput {
                cancer_site: site,
                ..zero_input()
            };
            assert_eq!(compute(&input).unwrap().cancer_site_points, expected);
        }
    }

    #[test]
    fn scores_zero_one_two_and_three_preserve_original_bands_and_threshold() {
        for (score, expected_band, expected_threshold) in [
            (0, OriginalRiskBand::Low, false),
            (1, OriginalRiskBand::Intermediate, false),
            (2, OriginalRiskBand::Intermediate, true),
            (3, OriginalRiskBand::High, true),
        ] {
            let mut input = zero_input();
            input.cancer_site = match score {
                0 => CancerSite::Other,
                1 => CancerSite::Lung,
                _ => CancerSite::Stomach,
            };
            if score == 3 {
                input.platelet_count_10_9_l = 350.0;
            }
            let outcome = compute(&input).unwrap();
            assert_eq!(outcome.score, score);
            assert_eq!(outcome.original_risk_band, expected_band);
            assert_eq!(
                outcome.meets_guideline_consideration_threshold,
                expected_threshold
            );
        }

        let score_two = compute(&KhoranaInput {
            cancer_site: CancerSite::Stomach,
            ..zero_input()
        })
        .unwrap();
        assert!(score_two.interpretation.contains("score 2 remains"));
        assert!(
            score_two
                .interpretation
                .contains("not automatic anticoagulation")
        );
    }

    #[test]
    fn maximum_vector_scores_six() {
        let input = KhoranaInput {
            cancer_site: CancerSite::Pancreas,
            platelet_count_10_9_l: 350.0,
            haemoglobin_g_dl: 9.0,
            uses_erythropoiesis_stimulating_agent: true,
            leukocyte_count_10_9_l: 11.001,
            bmi_kg_m2: 35.0,
            ..zero_input()
        };
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.score, 6);
        assert_eq!(outcome.original_risk_band, OriginalRiskBand::High);
    }

    #[test]
    fn working_preserves_raw_values_points_units_rates_and_qualifiers() {
        let input = KhoranaInput {
            cancer_site: CancerSite::Lung,
            platelet_count_10_9_l: 400.0,
            haemoglobin_g_dl: 9.5,
            uses_erythropoiesis_stimulating_agent: false,
            leukocyte_count_10_9_l: 12.0,
            bmi_kg_m2: 36.0,
            ..zero_input()
        };
        let response = build_response(&input).unwrap();

        assert_eq!(response.result, json!(5));
        assert_eq!(response.working["cancer_site"], json!("lung"));
        assert_eq!(response.working["cancer_site_points"], json!(1));
        assert_eq!(response.working["platelet_count_10_9_l"], json!(400.0));
        assert_eq!(response.working["platelet_count_unit"], json!("x10^9/L"));
        assert_eq!(response.working["haemoglobin_g_dl"], json!(9.5));
        assert_eq!(response.working["haemoglobin_unit"], json!("g/dL"));
        assert_eq!(response.working["haemoglobin_or_esa_points"], json!(1));
        assert_eq!(response.working["leukocyte_count_points"], json!(1));
        assert_eq!(response.working["bmi_unit"], json!("kg/m^2"));
        assert_eq!(response.working["bmi_points"], json!(1));
        assert_eq!(response.working["original_risk_band"], json!("high"));
        assert_eq!(
            response.working["original_derivation_cohort_observed_symptomatic_vte_rate_percent"],
            json!(7.1)
        );
        assert_eq!(
            response.working["original_validation_cohort_observed_symptomatic_vte_rate_percent"],
            json!(6.7)
        );
        assert!(
            response.working["original_observed_rate_time_horizon"]
                .as_str()
                .unwrap()
                .contains("2.5 months")
        );
        assert!(
            response.working["original_observed_rate_qualification"]
                .as_str()
                .unwrap()
                .contains("historical")
        );
    }

    #[test]
    fn all_original_band_rates_match_derivation_and_validation_cohorts() {
        for (band, derivation, validation) in [
            (OriginalRiskBand::Low, 0.8, 0.3),
            (OriginalRiskBand::Intermediate, 1.8, 2.0),
            (OriginalRiskBand::High, 7.1, 6.7),
        ] {
            assert_eq!(
                band.original_observed_rates_percent(),
                (derivation, validation)
            );
        }
    }

    #[test]
    fn interpretation_carries_applicability_and_safety_limits() {
        let interpretation = compute(&zero_input()).unwrap().interpretation;
        assert!(interpretation.contains("short-term symptomatic VTE"));
        assert!(interpretation.contains("historical cohort observations"));
        assert!(interpretation.contains("not an individualised probability"));
        assert!(interpretation.contains("bleeding risk"));
        assert!(interpretation.contains("contraindications"));
        assert!(interpretation.contains("drug interactions"));
        assert!(interpretation.contains("chemotherapy regimen"));
        assert!(interpretation.contains("prognosis"));
        assert!(interpretation.contains("patient preferences"));
        assert!(interpretation.contains("not a VTE diagnostic test"));
        assert!(interpretation.contains("not rule out VTE"));
    }

    #[test]
    fn rejects_nonfinite_and_out_of_range_numeric_values_without_maxima() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.001] {
            let mut input = zero_input();
            input.platelet_count_10_9_l = value;
            assert!(compute(&input).is_err());

            input = zero_input();
            input.leukocyte_count_10_9_l = value;
            assert!(compute(&input).is_err());
        }
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -0.001] {
            let mut input = zero_input();
            input.haemoglobin_g_dl = value;
            assert!(compute(&input).is_err());

            input = zero_input();
            input.bmi_kg_m2 = value;
            assert!(compute(&input).is_err());
        }

        let very_large = KhoranaInput {
            platelet_count_10_9_l: f64::MAX,
            haemoglobin_g_dl: f64::MAX,
            leukocyte_count_10_9_l: f64::MAX,
            bmi_kg_m2: f64::MAX,
            ..zero_input()
        };
        assert!(compute(&very_large).is_ok());
    }

    #[test]
    fn dynamic_calculation_matches_typed_and_rejects_invalid_objects() {
        let input = KhoranaInput {
            cancer_site: CancerSite::Stomach,
            platelet_count_10_9_l: 350.0,
            ..zero_input()
        };
        let dynamic = Khorana
            .calculate(&serde_json::to_value(input).unwrap())
            .unwrap();
        assert_eq!(dynamic, build_response(&input).unwrap());

        let mut invalid_context = serde_json::to_value(input).unwrap();
        invalid_context["assessment_context"] = json!("inpatient");
        assert!(Khorana.calculate(&invalid_context).is_err());

        let mut invalid_site = serde_json::to_value(input).unwrap();
        invalid_site["cancer_site"] = json!("renal");
        assert!(Khorana.calculate(&invalid_site).is_err());

        let mut unknown = serde_json::to_value(input).unwrap();
        unknown["metastatic"] = json!(true);
        assert!(Khorana.calculate(&unknown).is_err());

        let mut missing = serde_json::to_value(input).unwrap();
        missing.as_object_mut().unwrap().remove("bmi_kg_m2");
        assert!(Khorana.calculate(&missing).is_err());

        let mut negative = serde_json::to_value(input).unwrap();
        negative["platelet_count_10_9_l"] = json!(-1.0);
        assert!(Khorana.calculate(&negative).is_err());
    }

    #[test]
    fn categorical_wire_values_are_exact() {
        let value = serde_json::to_value(zero_input()).unwrap();
        assert_eq!(
            value["assessment_context"],
            json!("adult_ambulatory_before_new_chemotherapy_regimen")
        );
        assert_eq!(value["cancer_site"], json!("other"));
    }

    #[test]
    fn schema_is_closed_required_and_carries_safety_and_unit_semantics() {
        let schema = Khorana.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["required"].as_array().unwrap().len(), 7);
        assert_eq!(
            schema["properties"]["assessment_context"]["const"],
            json!("adult_ambulatory_before_new_chemotherapy_regimen")
        );
        assert_eq!(
            schema["properties"]["cancer_site"]["enum"],
            json!([
                "stomach",
                "pancreas",
                "lung",
                "lymphoma",
                "gynaecological",
                "bladder",
                "testicular",
                "other"
            ])
        );
        let other_description = schema["properties"]["cancer_site"]["description"]
            .as_str()
            .unwrap();
        for site in [
            "renal",
            "brain",
            "prostate",
            "breast",
            "colorectal",
            "head/neck",
            "myeloma",
            "other genitourinary",
        ] {
            assert!(other_description.contains(site), "missing site {site}");
        }
        assert_eq!(
            schema["properties"]["platelet_count_10_9_l"]["unit"],
            json!("x10^9/L")
        );
        assert_eq!(
            schema["properties"]["haemoglobin_g_dl"]["unit"],
            json!("g/dL")
        );
        assert_eq!(schema["properties"]["bmi_kg_m2"]["unit"], json!("kg/m^2"));
        for numeric in [
            "platelet_count_10_9_l",
            "haemoglobin_g_dl",
            "leukocyte_count_10_9_l",
            "bmi_kg_m2",
        ] {
            assert!(schema["properties"][numeric].get("maximum").is_none());
        }
        assert!(
            schema["description"]
                .as_str()
                .unwrap()
                .contains("does not diagnose or exclude VTE")
        );
        assert!(
            schema["guideline_context"]["caveats"]
                .as_str()
                .unwrap()
                .contains("score 2 band remains intermediate")
        );
        assert!(
            schema["properties"]
                .as_object()
                .unwrap()
                .values()
                .all(|property| property["description"].is_string()
                    && property["definition"]["statement"].is_string())
        );
    }

    #[test]
    fn calculate_for_english_records_content_locale() {
        let response = Khorana
            .calculate_for(
                &serde_json::to_value(zero_input()).unwrap(),
                SupportedLocale::En,
            )
            .unwrap();
        assert_eq!(response.working["content_locale"], json!("en"));
    }
}
