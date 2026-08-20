// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Familial Hypercholesterolaemia diagnostic criteria.
//!
//! Implements two widely-used scoring systems:
//!
//! 1. Simon Broome criteria (UK Heart Foundation, 1991)
//!    Definite FH: cholesterol criterion + tendon xanthomata or pathogenic mutation
//!    Possible FH: cholesterol criterion + family history of premature CHD or raised cholesterol
//!
//! 2. Dutch Lipid Clinic Network (DLCN) criteria (Familial Hypercholesterolaemia Score)
//!    Points-based system; score > 8 = definite, 6-8 = probable, 3-5 = possible, < 3 = unlikely

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "familial_hypercholesterolaemia";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1093/eurheartj/ehu531",
};

pub const REFERENCE: &str = "Nordestgaard BG et al. Familial hypercholesterolaemia is underdiagnosed and \
undertreated in the general population: guidance for clinicians to prevent coronary heart disease. \
Eur Heart J. 2013;34(45):3478-3490. doi:10.1093/eurheartj/eht273 | \
Scientific Steering Committee on behalf of the Simon Broome Register Group. \
Risk of fatal coronary heart disease in familial hypercholesterolaemia. BMJ. 1991;303:893-896.";

// --- Simon Broome ---

/// Simon Broome cholesterol criterion (total cholesterol or LDL-C).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimonBroomeCholesterol {
    /// Total cholesterol > 7.5 mmol/L (adult) or LDL-C > 4.9 mmol/L
    MeetsThreshold,
    /// Does not meet the cholesterol threshold
    BelowThreshold,
}

/// Simon Broome classification result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimonBroomeResult {
    DefiniteFh,
    PossibleFh,
    Unlikely,
}

impl SimonBroomeResult {
    fn label(self) -> &'static str {
        match self {
            SimonBroomeResult::DefiniteFh => "Definite FH",
            SimonBroomeResult::PossibleFh => "Possible FH",
            SimonBroomeResult::Unlikely => "FH unlikely by Simon Broome criteria",
        }
    }
}

// --- Dutch Lipid Clinic Network (DLCN) ---

/// Family history points for DLCN.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DlcnFamilyHistory {
    /// No relevant family history (0 pts)
    None,
    /// First-degree relative with premature CVD or LDL-C > 95th percentile (1 pt)
    RelativeWithPrematureCvdOrHighLdl,
    /// First-degree relative with tendon xanthomata or arcus cornealis, or child < 18 with LDL > 95th percentile (2 pts)
    RelativeWithXanthomasOrArcus,
}

/// LDL-C band for DLCN scoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DlcnLdl {
    /// LDL-C >= 8.5 mmol/L (8 pts)
    Above8_5,
    /// LDL-C 6.5-8.4 mmol/L (5 pts)
    From6_5To8_4,
    /// LDL-C 5.0-6.4 mmol/L (3 pts)
    From5_0To6_4,
    /// LDL-C 4.0-4.9 mmol/L (1 pt)
    From4_0To4_9,
    /// LDL-C < 4.0 mmol/L (0 pts)
    Below4_0,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FamilialHypercholesterolaemiaInput {
    // --- Simon Broome fields ---
    pub simon_broome_cholesterol: SimonBroomeCholesterol,
    /// Tendon xanthomata in patient or first-degree relative (Definite FH criterion)
    pub tendon_xanthomata: bool,
    /// Pathogenic DNA mutation (Definite FH criterion)
    pub pathogenic_mutation: bool,
    /// Family history of premature CHD in first-degree relative < 60 years, or elevated cholesterol
    /// in first-degree relative (Possible FH criterion)
    pub family_history_premature_chd_or_raised_cholesterol: bool,

    // --- DLCN fields ---
    pub dlcn_family_history: DlcnFamilyHistory,
    /// Personal history of premature coronary artery disease (men < 55, women < 60) (2 pts)
    pub premature_coronary_disease: bool,
    /// Premature cerebral or peripheral vascular disease (men < 55, women < 60) (1 pt)
    pub premature_cerebrovascular_disease: bool,
    /// Tendon xanthomata (6 pts) - shared with Simon Broome above
    // Note: tendon_xanthomata field above is shared
    /// Arcus cornealis before age 45 (4 pts)
    pub arcus_cornealis_under_45: bool,
    pub dlcn_ldl: DlcnLdl,
    /// Causative LDLR, APOB, or PCSK9 mutation confirmed (8 pts)
    pub causative_mutation_confirmed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FamilialHypercholesterolaemiaOutcome {
    pub simon_broome: SimonBroomeResult,
    pub dlcn_score: u8,
    pub dlcn_category: &'static str,
    pub interpretation: String,
}

fn dlcn_category(score: u8) -> &'static str {
    match score {
        0..=2 => "Unlikely FH",
        3..=5 => "Possible FH",
        6..=8 => "Probable FH",
        _ => "Definite FH",
    }
}

pub fn compute(
    input: &FamilialHypercholesterolaemiaInput,
) -> Result<FamilialHypercholesterolaemiaOutcome, CalcError> {
    // Simon Broome
    let simon_broome = if input.simon_broome_cholesterol == SimonBroomeCholesterol::MeetsThreshold
        && (input.tendon_xanthomata || input.pathogenic_mutation)
    {
        SimonBroomeResult::DefiniteFh
    } else if input.simon_broome_cholesterol == SimonBroomeCholesterol::MeetsThreshold
        && input.family_history_premature_chd_or_raised_cholesterol
    {
        SimonBroomeResult::PossibleFh
    } else {
        SimonBroomeResult::Unlikely
    };

    // DLCN
    let family_pts: u8 = match input.dlcn_family_history {
        DlcnFamilyHistory::None => 0,
        DlcnFamilyHistory::RelativeWithPrematureCvdOrHighLdl => 1,
        DlcnFamilyHistory::RelativeWithXanthomasOrArcus => 2,
    };
    let clinical_pts: u8 = (if input.premature_coronary_disease {
        2
    } else {
        0
    }) + (if input.premature_cerebrovascular_disease {
        1
    } else {
        0
    });
    let physical_pts: u8 = (if input.tendon_xanthomata { 6 } else { 0 })
        + (if input.arcus_cornealis_under_45 { 4 } else { 0 });
    let ldl_pts: u8 = match input.dlcn_ldl {
        DlcnLdl::Above8_5 => 8,
        DlcnLdl::From6_5To8_4 => 5,
        DlcnLdl::From5_0To6_4 => 3,
        DlcnLdl::From4_0To4_9 => 1,
        DlcnLdl::Below4_0 => 0,
    };
    let mutation_pts: u8 = if input.causative_mutation_confirmed {
        8
    } else {
        0
    };

    let dlcn_score = family_pts + clinical_pts + physical_pts + ldl_pts + mutation_pts;
    let dlcn_cat = dlcn_category(dlcn_score);

    let interpretation = format!(
        "Simon Broome: {}. Dutch Lipid Clinic Network score {dlcn_score} - {dlcn_cat}. \
Patients with possible or definite FH should be referred to a lipid specialist. \
Cascade screening of first-degree relatives is recommended.",
        simon_broome.label()
    );

    Ok(FamilialHypercholesterolaemiaOutcome {
        simon_broome,
        dlcn_score,
        dlcn_category: dlcn_cat,
        interpretation,
    })
}

pub fn build_response(
    input: &FamilialHypercholesterolaemiaInput,
) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("simon_broome".into(), json!(o.simon_broome.label()));
    working.insert("dlcn_score".into(), json!(o.dlcn_score));
    working.insert("dlcn_category".into(), json!(o.dlcn_category));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.dlcn_score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct FamilialHypercholesterolaemia;

impl Calculator for FamilialHypercholesterolaemia {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Familial Hypercholesterolaemia (Simon Broome + DLCN)"
    }

    fn description(&self) -> &'static str {
        "Diagnoses familial hypercholesterolaemia using both the Simon Broome (UK) and Dutch Lipid Clinic Network (DLCN) criteria."
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
            "title": "FamilialHypercholesterolaemiaInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "simon_broome_cholesterol", "tendon_xanthomata", "pathogenic_mutation",
                "family_history_premature_chd_or_raised_cholesterol",
                "dlcn_family_history", "premature_coronary_disease",
                "premature_cerebrovascular_disease", "arcus_cornealis_under_45",
                "dlcn_ldl", "causative_mutation_confirmed"
            ],
            "properties": {
                "simon_broome_cholesterol": {
                    "type": "string",
                    "enum": ["meets_threshold", "below_threshold"],
                    "description": "Simon Broome cholesterol criterion: total cholesterol > 7.5 mmol/L (adult) or LDL-C > 4.9 mmol/L"
                },
                "tendon_xanthomata": {
                    "type": "boolean",
                    "description": "Tendon xanthomata in patient or first-degree relative (Simon Broome Definite; DLCN 6 pts)"
                },
                "pathogenic_mutation": {
                    "type": "boolean",
                    "description": "Pathogenic mutation in LDLR, APOB, or PCSK9 (Simon Broome Definite criterion)"
                },
                "family_history_premature_chd_or_raised_cholesterol": {
                    "type": "boolean",
                    "description": "Family history of premature CHD in first-degree relative < 60, or raised cholesterol in first-degree relative (Simon Broome Possible criterion)"
                },
                "dlcn_family_history": {
                    "type": "string",
                    "enum": ["none", "relative_with_premature_cvd_or_high_ldl", "relative_with_xanthomas_or_arcus"],
                    "description": "DLCN family history: none=0, relative_with_premature_cvd_or_high_ldl=1, relative_with_xanthomas_or_arcus=2 pts"
                },
                "premature_coronary_disease": {
                    "type": "boolean",
                    "description": "Personal premature coronary artery disease (men < 55, women < 60) (DLCN 2 pts)"
                },
                "premature_cerebrovascular_disease": {
                    "type": "boolean",
                    "description": "Personal premature cerebral or peripheral vascular disease (men < 55, women < 60) (DLCN 1 pt)"
                },
                "arcus_cornealis_under_45": {
                    "type": "boolean",
                    "description": "Arcus cornealis before age 45 (DLCN 4 pts)"
                },
                "dlcn_ldl": {
                    "type": "string",
                    "enum": ["above8_5", "from6_5_to8_4", "from5_0_to6_4", "from4_0_to4_9", "below4_0"],
                    "description": "LDL-C band for DLCN scoring (mmol/L): above8_5=8pts, from6_5_to8_4=5pts, from5_0_to6_4=3pts, from4_0_to4_9=1pt, below4_0=0pts"
                },
                "causative_mutation_confirmed": {
                    "type": "boolean",
                    "description": "Causative mutation in LDLR, APOB, or PCSK9 confirmed by DNA analysis (DLCN 8 pts)"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: FamilialHypercholesterolaemiaInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> FamilialHypercholesterolaemiaInput {
        FamilialHypercholesterolaemiaInput {
            simon_broome_cholesterol: SimonBroomeCholesterol::BelowThreshold,
            tendon_xanthomata: false,
            pathogenic_mutation: false,
            family_history_premature_chd_or_raised_cholesterol: false,
            dlcn_family_history: DlcnFamilyHistory::None,
            premature_coronary_disease: false,
            premature_cerebrovascular_disease: false,
            arcus_cornealis_under_45: false,
            dlcn_ldl: DlcnLdl::Below4_0,
            causative_mutation_confirmed: false,
        }
    }

    #[test]
    fn unlikely_when_no_criteria() {
        let o = compute(&base()).unwrap();
        assert_eq!(o.simon_broome, SimonBroomeResult::Unlikely);
        assert_eq!(o.dlcn_score, 0);
        assert_eq!(o.dlcn_category, "Unlikely FH");
    }

    #[test]
    fn simon_broome_definite_with_xanthomata() {
        let o = compute(&FamilialHypercholesterolaemiaInput {
            simon_broome_cholesterol: SimonBroomeCholesterol::MeetsThreshold,
            tendon_xanthomata: true,
            ..base()
        })
        .unwrap();
        assert_eq!(o.simon_broome, SimonBroomeResult::DefiniteFh);
    }

    #[test]
    fn simon_broome_possible_with_family_history() {
        let o = compute(&FamilialHypercholesterolaemiaInput {
            simon_broome_cholesterol: SimonBroomeCholesterol::MeetsThreshold,
            family_history_premature_chd_or_raised_cholesterol: true,
            ..base()
        })
        .unwrap();
        assert_eq!(o.simon_broome, SimonBroomeResult::PossibleFh);
    }

    #[test]
    fn dlcn_definite_with_mutation() {
        // causative mutation = 8 pts -> Definite FH
        let o = compute(&FamilialHypercholesterolaemiaInput {
            causative_mutation_confirmed: true,
            dlcn_ldl: DlcnLdl::From6_5To8_4, // +5
            ..base()
        })
        .unwrap();
        assert_eq!(o.dlcn_score, 13);
        assert_eq!(o.dlcn_category, "Definite FH");
    }

    #[test]
    fn dlcn_possible_moderate_ldl() {
        // LDL 5.0-6.4 = 3pts + family history 1pt = 4 -> Possible
        let o = compute(&FamilialHypercholesterolaemiaInput {
            dlcn_ldl: DlcnLdl::From5_0To6_4,
            dlcn_family_history: DlcnFamilyHistory::RelativeWithPrematureCvdOrHighLdl,
            ..base()
        })
        .unwrap();
        assert_eq!(o.dlcn_score, 4);
        assert_eq!(o.dlcn_category, "Possible FH");
    }

    #[test]
    fn xanthomata_count_in_dlcn() {
        // tendon xanthomata = 6 pts in DLCN
        let o = compute(&FamilialHypercholesterolaemiaInput {
            tendon_xanthomata: true,
            ..base()
        })
        .unwrap();
        assert_eq!(o.dlcn_score, 6);
        assert_eq!(o.dlcn_category, "Probable FH");
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let b = base();
        let value = json!({
            "simon_broome_cholesterol": "below_threshold",
            "tendon_xanthomata": false,
            "pathogenic_mutation": false,
            "family_history_premature_chd_or_raised_cholesterol": false,
            "dlcn_family_history": "none",
            "premature_coronary_disease": false,
            "premature_cerebrovascular_disease": false,
            "arcus_cornealis_under_45": false,
            "dlcn_ldl": "below4_0",
            "causative_mutation_confirmed": false
        });
        let dynamic = FamilialHypercholesterolaemia.calculate(&value).unwrap();
        let typed = build_response(&b).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn rejects_invalid_enum_value() {
        assert!(FamilialHypercholesterolaemia
            .calculate(&json!({"simon_broome_cholesterol": "very_high", "tendon_xanthomata": false, "pathogenic_mutation": false, "family_history_premature_chd_or_raised_cholesterol": false, "dlcn_family_history": "none", "premature_coronary_disease": false, "premature_cerebrovascular_disease": false, "arcus_cornealis_under_45": false, "dlcn_ldl": "below4_0", "causative_mutation_confirmed": false}))
            .is_err());
    }
}
