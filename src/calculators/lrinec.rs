// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! LRINEC - Laboratory Risk Indicator for Necrotizing Fasciitis.
//!
//! Sums points across six routine blood tests - CRP, total white cell count,
//! haemoglobin, sodium, creatinine, and glucose - to flag soft-tissue
//! infections where necrotising fasciitis should be considered more seriously
//! than ordinary cellulitis or abscess. Derived and internally validated on a
//! 314-patient Singapore cohort (89 confirmed necrotising fasciitis, 225
//! severe cellulitis/abscess controls) by Wong CH, Khin LW, Heng KS, Tan KC,
//! Low CO. "The LRINEC (Laboratory Risk Indicator for Necrotizing Fasciitis)
//! score: a tool for distinguishing necrotizing fasciitis from other soft
//! tissue infections." Crit Care Med. 2004;32(7):1535-1541.
//! doi:10.1097/01.CCM.0000129486.35458.7D.
//!
//! Point thresholds and risk-category bands below are transcribed from that
//! publication's Table 2 and accompanying text (verified against a faithful
//! reproduction of the same table circulated as a bedside reference by the
//! University of Colorado Department of Surgery, and cross-checked against
//! independent secondary reproductions). Maximum score is 13. A score of 6 or
//! more was the paper's own cutoff (positive predictive value 92.0%, negative
//! predictive value 96.0% in the derivation cohort); the paper additionally
//! stratified the full score range into three risk bands: low risk (score
//! <=5, <50% probability of necrotising fasciitis), intermediate risk (score
//! 6-7, 50-75% probability), and high risk (score >=8, >75% probability).
//!
//! Units are fixed to the paper's own reporting units rather than accepting
//! caller-selected alternatives, because a silently wrong unit here changes
//! sub-scores that matter for detecting a surgical emergency: CRP in mg/L,
//! total white cell count in x10^9/L (equivalently x1000/microL, or the
//! "thousands" figure UK/SI differentials report), haemoglobin in g/dL,
//! sodium in mmol/L, creatinine in umol/L, and glucose in mmol/L. A host
//! integrating a US laboratory (creatinine typically mg/dL, glucose typically
//! mg/dL) must convert before calling this calculator.
//!
//! LRINEC is a screening aid derived from a single-centre retrospective
//! cohort. It has shown inconsistent sensitivity in later external
//! validations and must never be used to defer surgical exploration or
//! specialist referral in a patient whose clinical picture is concerning for
//! necrotising fasciitis, regardless of the computed score.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "lrinec";

/// Primary citation.
pub const REFERENCE: &str = "Wong CH, Khin LW, Heng KS, Tan KC, Low CO. The LRINEC (Laboratory Risk Indicator for Necrotizing Fasciitis) score: a tool for distinguishing necrotizing fasciitis from other soft tissue infections. Crit Care Med. 2004;32(7):1535-1541. doi:10.1097/01.CCM.0000129486.35458.7D";

/// Distribution licence: the score is a published clinical method from the
/// primary literature, implemented here from that source.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1097/01.CCM.0000129486.35458.7D",
};

/// LRINEC inputs. Every value is fixed to the unit the original paper reports
/// it in - see the module docs for the conversions a caller must apply first.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LrinecInput {
    /// C-reactive protein, mg/L (>= 150 scores 4 points).
    pub crp_mg_l: f64,
    /// Total white cell count, x10^9/L (equivalently x1000/microL). <15 -> 0,
    /// 15-25 -> 1, >25 -> 2 points.
    pub wbc_x10_9_l: f64,
    /// Haemoglobin, g/dL. >13.5 -> 0, 11-13.5 -> 1, <11 -> 2 points.
    pub haemoglobin_g_dl: f64,
    /// Serum sodium, mmol/L (< 135 scores 2 points).
    pub sodium_mmol_l: f64,
    /// Serum creatinine, umol/L (> 141 scores 2 points).
    pub creatinine_umol_l: f64,
    /// Serum glucose, mmol/L (> 10 scores 1 point).
    pub glucose_mmol_l: f64,
}

/// Overall LRINEC risk band, per the original paper's stratification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskCategory {
    /// Score 0-5: <50% probability of necrotising fasciitis.
    Low,
    /// Score 6-7: 50-75% probability.
    Intermediate,
    /// Score 8-13: >75% probability.
    High,
}

impl RiskCategory {
    fn from_score(score: u8) -> Self {
        if score <= 5 {
            RiskCategory::Low
        } else if score <= 7 {
            RiskCategory::Intermediate
        } else {
            RiskCategory::High
        }
    }

    fn slug(self) -> &'static str {
        match self {
            RiskCategory::Low => "low",
            RiskCategory::Intermediate => "intermediate",
            RiskCategory::High => "high",
        }
    }

    fn probability_descriptor(self) -> &'static str {
        match self {
            RiskCategory::Low => "<50%",
            RiskCategory::Intermediate => "50-75%",
            RiskCategory::High => ">75%",
        }
    }
}

/// The computed outcome, with each parameter's sub-score retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LrinecOutcome {
    pub crp_points: u8,
    pub wbc_points: u8,
    pub haemoglobin_points: u8,
    pub sodium_points: u8,
    pub creatinine_points: u8,
    pub glucose_points: u8,
    /// Total score, 0-13.
    pub score: u8,
    pub risk_category: RiskCategory,
    pub interpretation: String,
}

fn crp_points(mg_l: f64) -> u8 {
    if mg_l >= 150.0 { 4 } else { 0 }
}

fn wbc_points(x10_9_l: f64) -> u8 {
    if x10_9_l < 15.0 {
        0
    } else if x10_9_l <= 25.0 {
        1
    } else {
        2
    }
}

fn haemoglobin_points(g_dl: f64) -> u8 {
    if g_dl > 13.5 {
        0
    } else if g_dl >= 11.0 {
        1
    } else {
        2
    }
}

fn sodium_points(mmol_l: f64) -> u8 {
    if mmol_l < 135.0 { 2 } else { 0 }
}

fn creatinine_points(umol_l: f64) -> u8 {
    if umol_l > 141.0 { 2 } else { 0 }
}

fn glucose_points(mmol_l: f64) -> u8 {
    if mmol_l > 10.0 { 1 } else { 0 }
}

/// Plausibility bounds only - not part of the original scoring criteria.
/// These exist to catch an obviously wrong unit or a mistyped value (for
/// example a raw WBC count of 18000 entered where 18 was meant), not to
/// encode any clinical judgement.
fn validate(value: f64, name: &str, min: f64, max: f64) -> Result<(), CalcError> {
    if !value.is_finite() || !(min..=max).contains(&value) {
        return Err(CalcError::InvalidInput(format!(
            "{name} must be finite and between {min} and {max}"
        )));
    }
    Ok(())
}

/// Pure scoring.
pub fn compute(input: &LrinecInput) -> Result<LrinecOutcome, CalcError> {
    validate(input.crp_mg_l, "crp_mg_l", 0.0, 600.0)?;
    validate(input.wbc_x10_9_l, "wbc_x10_9_l", 0.0, 100.0)?;
    validate(input.haemoglobin_g_dl, "haemoglobin_g_dl", 2.0, 24.0)?;
    validate(input.sodium_mmol_l, "sodium_mmol_l", 100.0, 200.0)?;
    validate(input.creatinine_umol_l, "creatinine_umol_l", 10.0, 2000.0)?;
    validate(input.glucose_mmol_l, "glucose_mmol_l", 0.5, 100.0)?;

    let crp_points = crp_points(input.crp_mg_l);
    let wbc_points = wbc_points(input.wbc_x10_9_l);
    let haemoglobin_points = haemoglobin_points(input.haemoglobin_g_dl);
    let sodium_points = sodium_points(input.sodium_mmol_l);
    let creatinine_points = creatinine_points(input.creatinine_umol_l);
    let glucose_points = glucose_points(input.glucose_mmol_l);

    let score = crp_points
        + wbc_points
        + haemoglobin_points
        + sodium_points
        + creatinine_points
        + glucose_points;

    let risk_category = RiskCategory::from_score(score);

    let interpretation = format!(
        "LRINEC score {score} of a possible 13: {} risk of necrotising fasciitis ({} probability). \
Wong et al. (2004) found a score of 6 or more had a positive predictive value of 92.0% and a \
negative predictive value of 96.0% in their derivation cohort. LRINEC is a screening aid derived \
from a single-centre retrospective cohort with inconsistent sensitivity on later external \
validation: a low score does not exclude necrotising fasciitis, and urgent surgical exploration \
or specialist referral should never be deferred for a low or intermediate score in a patient whose \
clinical picture is concerning for necrotising fasciitis.",
        risk_category.slug(),
        risk_category.probability_descriptor()
    );

    Ok(LrinecOutcome {
        crp_points,
        wbc_points,
        haemoglobin_points,
        sodium_points,
        creatinine_points,
        glucose_points,
        score,
        risk_category,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &LrinecInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("crp_points".into(), json!(o.crp_points));
    working.insert("wbc_points".into(), json!(o.wbc_points));
    working.insert("haemoglobin_points".into(), json!(o.haemoglobin_points));
    working.insert("sodium_points".into(), json!(o.sodium_points));
    working.insert("creatinine_points".into(), json!(o.creatinine_points));
    working.insert("glucose_points".into(), json!(o.glucose_points));
    working.insert("total_score".into(), json!(o.score));
    working.insert("risk_category".into(), json!(o.risk_category.slug()));
    working.insert(
        "probability_of_necrotising_fasciitis".into(),
        json!(o.risk_category.probability_descriptor()),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Lrinec;

impl Calculator for Lrinec {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "LRINEC Score (Necrotising Fasciitis Risk Indicator)"
    }

    fn description(&self) -> &'static str {
        "Six-variable laboratory score (CRP, WBC, haemoglobin, sodium, creatinine, glucose) \
distinguishing necrotising fasciitis from other soft-tissue infections; score >=6 warrants \
increased suspicion, >=8 is high risk."
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
            "title": "LrinecInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "crp_mg_l", "wbc_x10_9_l", "haemoglobin_g_dl",
                "sodium_mmol_l", "creatinine_umol_l", "glucose_mmol_l"
            ],
            "properties": {
                "crp_mg_l": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 600,
                    "description": "C-reactive protein, mg/L (>= 150 scores 4 points)",
                    "definition": {
                        "concept": "CRP sub-score",
                        "statement": "CRP of 150 mg/L or greater scores 4 points; below 150 mg/L scores 0. This is the largest single weight in the score.",
                        "excludes": ["A value reported in mg/dL: mg/dL and mg/L differ by 10x for CRP and would silently change the sub-score"],
                        "source": { "citation": "Wong CH et al. Crit Care Med. 2004;32(7):1535-1541.", "url": "https://doi.org/10.1097/01.CCM.0000129486.35458.7D" },
                        "status": "draft"
                    }
                },
                "wbc_x10_9_l": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 100,
                    "description": "Total white cell count, x10^9/L (equivalently x1000/microL). <15 -> 0, 15-25 -> 1, >25 -> 2 points",
                    "definition": {
                        "concept": "Total WBC sub-score",
                        "statement": "Below 15 x10^9/L scores 0; 15 to 25 x10^9/L inclusive scores 1; above 25 x10^9/L scores 2.",
                        "excludes": ["A raw cells/microL count (e.g. 18000): divide by 1000 first - x10^9/L is the thousands figure"],
                        "source": { "citation": "Wong CH et al. Crit Care Med. 2004;32(7):1535-1541.", "url": "https://doi.org/10.1097/01.CCM.0000129486.35458.7D" },
                        "status": "draft"
                    }
                },
                "haemoglobin_g_dl": {
                    "type": "number",
                    "minimum": 2,
                    "maximum": 24,
                    "description": "Haemoglobin, g/dL. >13.5 -> 0, 11-13.5 -> 1, <11 -> 2 points",
                    "definition": {
                        "concept": "Haemoglobin sub-score",
                        "statement": "Above 13.5 g/dL scores 0; 11 to 13.5 g/dL inclusive scores 1; below 11 g/dL scores 2.",
                        "excludes": ["A value reported in g/L: g/L and g/dL differ by 10x for haemoglobin and would silently change the sub-score"],
                        "source": { "citation": "Wong CH et al. Crit Care Med. 2004;32(7):1535-1541.", "url": "https://doi.org/10.1097/01.CCM.0000129486.35458.7D" },
                        "status": "draft"
                    }
                },
                "sodium_mmol_l": {
                    "type": "number",
                    "minimum": 100,
                    "maximum": 200,
                    "description": "Serum sodium, mmol/L (< 135 scores 2 points)",
                    "definition": {
                        "concept": "Sodium sub-score",
                        "statement": "Below 135 mmol/L scores 2 points; 135 mmol/L or above scores 0.",
                        "source": { "citation": "Wong CH et al. Crit Care Med. 2004;32(7):1535-1541.", "url": "https://doi.org/10.1097/01.CCM.0000129486.35458.7D" },
                        "status": "draft"
                    }
                },
                "creatinine_umol_l": {
                    "type": "number",
                    "minimum": 10,
                    "maximum": 2000,
                    "description": "Serum creatinine, umol/L (> 141 scores 2 points)",
                    "definition": {
                        "concept": "Creatinine sub-score",
                        "statement": "Above 141 umol/L scores 2 points; 141 umol/L or below scores 0.",
                        "excludes": ["A value reported in mg/dL (e.g. from a US laboratory): multiply mg/dL by 88.4 to get umol/L before calling this calculator, or the sub-score will silently be wrong"],
                        "source": { "citation": "Wong CH et al. Crit Care Med. 2004;32(7):1535-1541.", "url": "https://doi.org/10.1097/01.CCM.0000129486.35458.7D" },
                        "status": "draft"
                    }
                },
                "glucose_mmol_l": {
                    "type": "number",
                    "minimum": 0.5,
                    "maximum": 100,
                    "description": "Serum glucose, mmol/L (> 10 scores 1 point)",
                    "definition": {
                        "concept": "Glucose sub-score",
                        "statement": "Above 10 mmol/L scores 1 point; 10 mmol/L or below scores 0.",
                        "excludes": ["A value reported in mg/dL (e.g. from a US laboratory): divide mg/dL by 18.016 to get mmol/L before calling this calculator, or the sub-score will silently be wrong"],
                        "source": { "citation": "Wong CH et al. Crit Care Med. 2004;32(7):1535-1541.", "url": "https://doi.org/10.1097/01.CCM.0000129486.35458.7D" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: LrinecInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All parameters at their most reassuring values -> score 0, low risk.
    fn all_reassuring() -> LrinecInput {
        LrinecInput {
            crp_mg_l: 10.0,
            wbc_x10_9_l: 8.0,
            haemoglobin_g_dl: 14.0,
            sodium_mmol_l: 140.0,
            creatinine_umol_l: 80.0,
            glucose_mmol_l: 6.0,
        }
    }

    /// All six criteria at their most severe values -> maximum score of 13.
    fn all_severe() -> LrinecInput {
        LrinecInput {
            crp_mg_l: 150.0,
            wbc_x10_9_l: 26.0,
            haemoglobin_g_dl: 10.0,
            sodium_mmol_l: 130.0,
            creatinine_umol_l: 150.0,
            glucose_mmol_l: 11.0,
        }
    }

    #[test]
    fn wong_2004_all_reassuring_scores_zero_and_low_risk() {
        let o = compute(&all_reassuring()).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.risk_category, RiskCategory::Low);
        assert!(o.interpretation.contains("low risk"));
        assert!(o.interpretation.contains("<50%"));
    }

    #[test]
    fn wong_2004_all_severe_scores_maximum_thirteen_and_high_risk() {
        let o = compute(&all_severe()).unwrap();
        assert_eq!(o.crp_points, 4);
        assert_eq!(o.wbc_points, 2);
        assert_eq!(o.haemoglobin_points, 2);
        assert_eq!(o.sodium_points, 2);
        assert_eq!(o.creatinine_points, 2);
        assert_eq!(o.glucose_points, 1);
        assert_eq!(o.score, 13);
        assert_eq!(o.risk_category, RiskCategory::High);
        assert!(o.interpretation.contains("high risk"));
        assert!(o.interpretation.contains(">75%"));
    }

    #[test]
    fn crp_boundary_is_150_mg_l() {
        assert_eq!(crp_points(149.9), 0);
        assert_eq!(crp_points(150.0), 4);
    }

    #[test]
    fn wbc_boundaries_match_published_bands() {
        assert_eq!(wbc_points(14.9), 0);
        assert_eq!(wbc_points(15.0), 1);
        assert_eq!(wbc_points(25.0), 1);
        assert_eq!(wbc_points(25.1), 2);
    }

    #[test]
    fn haemoglobin_boundaries_match_published_bands() {
        assert_eq!(haemoglobin_points(13.6), 0);
        assert_eq!(haemoglobin_points(13.5), 1);
        assert_eq!(haemoglobin_points(11.0), 1);
        assert_eq!(haemoglobin_points(10.9), 2);
    }

    #[test]
    fn sodium_boundary_is_135_mmol_l() {
        assert_eq!(sodium_points(135.0), 0);
        assert_eq!(sodium_points(134.9), 2);
    }

    #[test]
    fn creatinine_boundary_is_141_umol_l() {
        assert_eq!(creatinine_points(141.0), 0);
        assert_eq!(creatinine_points(141.1), 2);
    }

    #[test]
    fn glucose_boundary_is_10_mmol_l() {
        assert_eq!(glucose_points(10.0), 0);
        assert_eq!(glucose_points(10.1), 1);
    }

    /// Score 5 is the top of the low-risk band; score 6 crosses into
    /// intermediate; score 8 crosses into high risk. Wong et al. defined
    /// intermediate as 6-7 and high as >=8.
    #[test]
    fn risk_category_boundaries() {
        assert_eq!(RiskCategory::from_score(5), RiskCategory::Low);
        assert_eq!(RiskCategory::from_score(6), RiskCategory::Intermediate);
        assert_eq!(RiskCategory::from_score(7), RiskCategory::Intermediate);
        assert_eq!(RiskCategory::from_score(8), RiskCategory::High);
        assert_eq!(RiskCategory::from_score(13), RiskCategory::High);
    }

    #[test]
    fn published_cutoff_of_six_is_intermediate_risk() {
        // The paper's headline >=6 cutoff sits at the low end of the
        // intermediate band, not automatically "high risk".
        let mut input = all_reassuring();
        input.crp_mg_l = 150.0; // +4
        input.sodium_mmol_l = 130.0; // +2
        let o = compute(&input).unwrap();
        assert_eq!(o.score, 6);
        assert_eq!(o.risk_category, RiskCategory::Intermediate);
    }

    #[test]
    fn rejects_out_of_plausible_range_values() {
        let mut i = all_reassuring();
        i.crp_mg_l = -1.0;
        assert!(compute(&i).is_err());

        let mut i = all_reassuring();
        i.wbc_x10_9_l = f64::NAN;
        assert!(compute(&i).is_err());

        let mut i = all_reassuring();
        i.sodium_mmol_l = 50.0;
        assert!(compute(&i).is_err());

        let mut i = all_reassuring();
        i.creatinine_umol_l = 5.0;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn build_response_carries_working_and_reference() {
        let r = build_response(&all_severe()).unwrap();
        assert_eq!(r.calculator, "lrinec");
        assert_eq!(r.result, json!(13));
        assert_eq!(r.working["total_score"], json!(13));
        assert_eq!(r.working["risk_category"], json!("high"));
        assert_eq!(
            r.working["probability_of_necrotising_fasciitis"],
            json!(">75%")
        );
        assert!(r.reference.contains("Wong"));
        assert!(r.reference.contains("2004"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "crp_mg_l": 10.0,
            "wbc_x10_9_l": 8.0,
            "haemoglobin_g_dl": 14.0,
            "sodium_mmol_l": 140.0,
            "creatinine_umol_l": 80.0,
            "glucose_mmol_l": 6.0
        });
        let dynamic = Lrinec.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&all_reassuring()).unwrap());
        assert_eq!(dynamic.result, json!(0));
    }

    #[test]
    fn dynamic_calculate_rejects_garbage() {
        assert!(Lrinec.calculate(&json!({ "crp_mg_l": "high" })).is_err());
    }

    #[test]
    fn dynamic_calculate_rejects_unknown_fields() {
        let mut value = serde_json::to_value(all_reassuring()).unwrap();
        value["extra_field"] = json!(true);
        assert!(Lrinec.calculate(&value).is_err());
    }

    #[test]
    fn schema_requires_all_six_variables() {
        let schema = Lrinec.input_schema();
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 6);
        assert!(required.contains(&json!("crp_mg_l")));
        assert!(required.contains(&json!("glucose_mmol_l")));
    }

    #[test]
    fn schema_flags_unit_traps() {
        let schema = Lrinec.input_schema();
        let creatinine = &schema["properties"]["creatinine_umol_l"]["definition"];
        assert!(creatinine["excludes"][0].as_str().unwrap().contains("88.4"));
        let glucose = &schema["properties"]["glucose_mmol_l"]["definition"];
        assert!(glucose["excludes"][0].as_str().unwrap().contains("18.016"));
    }
}
