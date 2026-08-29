// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! ISTH overt disseminated intravascular coagulation score, 2025 update.
//!
//! The current score derives all points from measurements and uses D-dimer as a
//! multiple of the local assay upper limit of normal. It applies only when a
//! clinician has identified an underlying disorder known to cause DIC.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "isth_overt_dic";
pub const REFERENCE: &str = "Iba T, Levy JH, Maier CL, et al. Updated definition and scoring of disseminated intravascular coagulation in 2025: communication from the ISTH SSC Subcommittee on Disseminated Intravascular Coagulation. J Thromb Haemost. 2025;23(7):2356-2362. doi:10.1016/j.jtha.2025.03.038. PMID:40216223.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published clinical scoring method - independently implemented from the primary literature",
    source_url: "https://doi.org/10.1016/j.jtha.2025.03.038",
};

const SERIAL_GUIDANCE: &str = "Recalculate when clinically indicated after repeat laboratory testing. The 2025 communication does not specify a fixed interval; repeat sooner when the clinical condition or laboratory trend changes.";
const PREGNANCY_WARNING: &str = "In pregnancy, physiologic changes in D-dimer and fibrinogen can alter this generic score; use specialist interpretation and pregnancy-specific obstetric DIC criteria where available.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DicAssociatedEtiology {
    SepsisOrSevereInfection,
    Malignancy,
    MajorTraumaOrTissueInjury,
    ObstetricComplication,
    VascularAbnormality,
    SevereImmunologicalOrToxicReaction,
    HeatStroke,
    PostCardiopulmonaryResuscitation,
    OtherClinicianConfirmedDicAssociatedEtiology,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IsthOvertDicInput {
    pub underlying_etiology: DicAssociatedEtiology,
    pub platelet_count_10_9_l: f64,
    pub d_dimer_multiple_of_uln: f64,
    pub pt_prolongation_seconds: f64,
    pub fibrinogen_g_l: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IsthOvertDicPoints {
    pub platelet_count: u8,
    pub d_dimer: u8,
    pub pt_prolongation: u8,
    pub fibrinogen: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OvertDicBand {
    BelowOvertDicThreshold,
    ConsistentWithOvertDic,
}

impl OvertDicBand {
    fn slug(self) -> &'static str {
        match self {
            Self::BelowOvertDicThreshold => "below_overt_dic_threshold",
            Self::ConsistentWithOvertDic => "consistent_with_overt_dic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsthOvertDicOutcome {
    pub score: u8,
    pub points: IsthOvertDicPoints,
    pub band: OvertDicBand,
    pub interpretation: String,
}

pub fn compute(input: &IsthOvertDicInput) -> Result<IsthOvertDicOutcome, CalcError> {
    for (name, value) in [
        ("platelet_count_10_9_l", input.platelet_count_10_9_l),
        ("d_dimer_multiple_of_uln", input.d_dimer_multiple_of_uln),
        ("pt_prolongation_seconds", input.pt_prolongation_seconds),
        ("fibrinogen_g_l", input.fibrinogen_g_l),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be a finite non-negative number"
            )));
        }
    }

    let points = IsthOvertDicPoints {
        platelet_count: if input.platelet_count_10_9_l < 50.0 {
            2
        } else if input.platelet_count_10_9_l < 100.0 {
            1
        } else {
            0
        },
        d_dimer: if input.d_dimer_multiple_of_uln > 7.0 {
            3
        } else if input.d_dimer_multiple_of_uln > 3.0 {
            2
        } else {
            0
        },
        pt_prolongation: if input.pt_prolongation_seconds >= 6.0 {
            2
        } else if input.pt_prolongation_seconds >= 3.0 {
            1
        } else {
            0
        },
        fibrinogen: u8::from(input.fibrinogen_g_l < 1.0),
    };
    let score = points.platelet_count + points.d_dimer + points.pt_prolongation + points.fibrinogen;
    let band = if score >= 5 {
        OvertDicBand::ConsistentWithOvertDic
    } else {
        OvertDicBand::BelowOvertDicThreshold
    };
    let mut interpretation = match band {
        OvertDicBand::ConsistentWithOvertDic => format!(
            "ISTH overt DIC score (2025) {score}/8: consistent with overt (late-phase) DIC in a patient with a recognised underlying etiology. This supports but does not independently establish the diagnosis. Interpret with the clinical picture, differential diagnosis, and serial laboratory results. The score does not select anticoagulants, blood products, doses, or treatment. {SERIAL_GUIDANCE}"
        ),
        OvertDicBand::BelowOvertDicThreshold => format!(
            "ISTH overt DIC score (2025) {score}/8: does not meet the threshold for overt (late-phase) DIC. Early-phase or evolving DIC is not excluded. Use etiology-specific early-phase criteria where available and reassess clinically with serial laboratory testing. The score does not select anticoagulants, blood products, doses, or treatment. {SERIAL_GUIDANCE}"
        ),
    };
    if input.underlying_etiology == DicAssociatedEtiology::ObstetricComplication {
        interpretation.push(' ');
        interpretation.push_str(PREGNANCY_WARNING);
    }

    Ok(IsthOvertDicOutcome {
        score,
        points,
        band,
        interpretation,
    })
}

pub fn build_response(input: &IsthOvertDicInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert("score_version".into(), json!("2025"));
    working.insert(
        "underlying_etiology".into(),
        json!(input.underlying_etiology),
    );
    working.insert(
        "platelet_count_10_9_l".into(),
        json!(input.platelet_count_10_9_l),
    );
    working.insert(
        "platelet_count_points".into(),
        json!(outcome.points.platelet_count),
    );
    working.insert(
        "d_dimer_multiple_of_uln".into(),
        json!(input.d_dimer_multiple_of_uln),
    );
    working.insert("d_dimer_points".into(), json!(outcome.points.d_dimer));
    working.insert(
        "pt_prolongation_seconds".into(),
        json!(input.pt_prolongation_seconds),
    );
    working.insert(
        "pt_prolongation_points".into(),
        json!(outcome.points.pt_prolongation),
    );
    working.insert("fibrinogen_g_l".into(), json!(input.fibrinogen_g_l));
    working.insert("fibrinogen_points".into(), json!(outcome.points.fibrinogen));
    working.insert("total_score".into(), json!(outcome.score));
    working.insert("maximum_score".into(), json!(8));
    working.insert("band".into(), json!(outcome.band.slug()));
    working.insert("serial_reassessment".into(), json!(SERIAL_GUIDANCE));
    working.insert("limitations".into(), json!(format!("Overt/late-phase DIC clinical-laboratory adjunct only; below-threshold results do not exclude early or evolving DIC. D-dimer is nonspecific and assay-dependent; fibrinogen can remain normal or high as an acute-phase reactant; liver disease, anticoagulants, vitamin K deficiency, dilution, transfusion, and other disorders can alter components independently. {PREGNANCY_WARNING}")));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.score),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn input_schema() -> Value {
    let source = json!({
        "citation": "Iba T, Levy JH, Maier CL, et al. J Thromb Haemost. 2025;23(7):2356-2362.",
        "url": "https://doi.org/10.1016/j.jtha.2025.03.038"
    });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "IsthOvertDicInput",
        "description": "ISTH 2025 score for overt/late-phase DIC in a patient with a clinician-confirmed underlying disorder known to cause DIC. Enter measurements from the same assessment episode where possible. This score neither independently establishes nor excludes DIC, does not assess early-phase DIC, and does not select treatment.",
        "type": "object",
        "additionalProperties": false,
        "required": ["underlying_etiology", "platelet_count_10_9_l", "d_dimer_multiple_of_uln", "pt_prolongation_seconds", "fibrinogen_g_l"],
        "properties": {
            "underlying_etiology": {
                "type": "string",
                "enum": ["sepsis_or_severe_infection", "malignancy", "major_trauma_or_tissue_injury", "obstetric_complication", "vascular_abnormality", "severe_immunological_or_toxic_reaction", "heat_stroke", "post_cardiopulmonary_resuscitation", "other_clinician_confirmed_dic_associated_etiology"],
                "description": "Clinician-confirmed underlying disorder known to cause DIC; no 'none' or unknown category is accepted",
                "definition": {
                    "concept": "DIC-associated underlying etiology",
                    "statement": "Select the clinician-confirmed underlying disorder responsible for the present concern for DIC; the overt-DIC algorithm is not applicable without one.",
                    "includes": ["Severe infection", "Malignancy", "Major tissue injury", "Obstetric complication", "Vascular abnormality", "Severe immunological or toxic reaction", "Heat stroke", "Following cardiopulmonary resuscitation"],
                    "excludes": ["Unexplained abnormal coagulation tests without an identified DIC-associated disorder", "An unknown or merely suspected etiology entered as though confirmed"],
                    "caveats": "Use other_clinician_confirmed_dic_associated_etiology only when the responsible clinician confirms that the disorder is recognised to cause DIC. In pregnancy, physiologic D-dimer and fibrinogen changes can alter this generic score; specialist interpretation and pregnancy-specific obstetric DIC criteria are preferable where available.",
                    "source": source, "status": "draft"
                }
            },
            "platelet_count_10_9_l": {
                "type": "number", "minimum": 0, "unit": "x10^9/L",
                "description": "Platelet count: <50=2 points; 50 to <100=1; >=100=0",
                "definition": {
                    "concept": "Platelet count for the current DIC assessment",
                    "statement": "Enter the current platelet count in x10^9/L, preferably from the same assessment episode as the other components.",
                    "excludes": ["A value in another unit without conversion", "Caller-supplied component points"],
                    "caveats": "Platelet count can be affected by marrow disease, immune destruction, dilution, transfusion, thrombotic microangiopathy, HIT, and other causes.",
                    "source": source, "status": "draft"
                }
            },
            "d_dimer_multiple_of_uln": {
                "type": "number", "minimum": 0,
                "description": "D-dimer divided by the upper limit of normal for the same assay and units: <=3=0 points; >3 to <=7=2; >7=3",
                "definition": {
                    "concept": "D-dimer multiple of assay ULN",
                    "statement": "Divide the patient D-dimer result by the upper limit of normal from the same assay, calibration, and reporting units.",
                    "excludes": ["A raw D-dimer concentration", "An FDP or qualitative fibrin-marker category", "Mixing FEU and DDU or different assay units"],
                    "caveats": "D-dimer assays are not fully standardised and elevations are nonspecific, including in infection, cancer, trauma, surgery, pregnancy, thrombosis, renal impairment, liver disease, inflammation, and older age.",
                    "source": source, "status": "draft"
                }
            },
            "pt_prolongation_seconds": {
                "type": "number", "minimum": 0, "unit": "seconds above local control or upper normal value",
                "description": "PT prolongation above local control/upper normal value: <3 seconds=0 points; 3 to <6=1; >=6=2",
                "definition": {
                    "concept": "Prothrombin-time prolongation",
                    "statement": "Enter seconds by which patient PT exceeds the applicable laboratory control or upper normal value.",
                    "excludes": ["Absolute PT", "INR", "Caller-supplied component points"],
                    "caveats": "Anticoagulants, vitamin K deficiency, liver dysfunction, dilution, transfusion, and factor deficiencies can prolong PT independently of DIC.",
                    "source": source, "status": "draft"
                }
            },
            "fibrinogen_g_l": {
                "type": "number", "minimum": 0, "unit": "g/L",
                "description": "Functional fibrinogen: <1.0 g/L=1 point; >=1.0=0",
                "definition": {
                    "concept": "Functional fibrinogen concentration",
                    "statement": "Enter functional fibrinogen in g/L, preferably from the same assessment episode as the other components.",
                    "excludes": ["mg/dL without conversion; 100 mg/dL equals 1 g/L", "Caller-supplied component points"],
                    "caveats": "Fibrinogen is an acute-phase reactant and may remain normal or elevated despite consumption, particularly in sepsis.",
                    "source": source, "status": "draft"
                }
            }
        }
    })
}

pub struct IsthOvertDic;

impl Calculator for IsthOvertDic {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "ISTH Overt DIC Score (2025)"
    }
    fn description(&self) -> &'static str {
        "Scores overt/late-phase DIC from measured platelets, D-dimer relative to assay ULN, PT prolongation, and fibrinogen in an eligible underlying disorder."
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
        let parsed: IsthOvertDicInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> IsthOvertDicInput {
        IsthOvertDicInput {
            underlying_etiology: DicAssociatedEtiology::SepsisOrSevereInfection,
            platelet_count_10_9_l: 100.0,
            d_dimer_multiple_of_uln: 3.0,
            pt_prolongation_seconds: 2.999,
            fibrinogen_g_l: 1.0,
        }
    }

    #[test]
    fn all_2025_component_boundaries_are_exact() {
        for (value, points) in [(49.999, 2), (50.0, 1), (99.999, 1), (100.0, 0)] {
            let input = IsthOvertDicInput {
                platelet_count_10_9_l: value,
                ..baseline()
            };
            assert_eq!(compute(&input).unwrap().points.platelet_count, points);
        }
        for (value, points) in [(3.0, 0), (3.001, 2), (7.0, 2), (7.001, 3)] {
            let input = IsthOvertDicInput {
                d_dimer_multiple_of_uln: value,
                ..baseline()
            };
            assert_eq!(compute(&input).unwrap().points.d_dimer, points);
        }
        for (value, points) in [(2.999, 0), (3.0, 1), (5.999, 1), (6.0, 2)] {
            let input = IsthOvertDicInput {
                pt_prolongation_seconds: value,
                ..baseline()
            };
            assert_eq!(compute(&input).unwrap().points.pt_prolongation, points);
        }
        for (value, points) in [(0.999, 1), (1.0, 0)] {
            let input = IsthOvertDicInput {
                fibrinogen_g_l: value,
                ..baseline()
            };
            assert_eq!(compute(&input).unwrap().points.fibrinogen, points);
        }
    }

    #[test]
    fn every_total_zero_through_eight_is_reachable() {
        let vectors = [
            (100.0, 3.0, 2.999, 1.0, 0),
            (99.999, 3.0, 2.999, 1.0, 1),
            (49.999, 3.0, 2.999, 1.0, 2),
            (100.0, 7.001, 2.999, 1.0, 3),
            (99.999, 7.001, 2.999, 1.0, 4),
            (49.999, 7.001, 2.999, 1.0, 5),
            (49.999, 7.001, 3.0, 1.0, 6),
            (49.999, 7.001, 6.0, 1.0, 7),
            (49.999, 7.001, 6.0, 0.999, 8),
        ];
        for (platelets, d_dimer, pt, fibrinogen, expected) in vectors {
            let outcome = compute(&IsthOvertDicInput {
                platelet_count_10_9_l: platelets,
                d_dimer_multiple_of_uln: d_dimer,
                pt_prolongation_seconds: pt,
                fibrinogen_g_l: fibrinogen,
                ..baseline()
            })
            .unwrap();
            assert_eq!(outcome.score, expected);
        }
    }

    #[test]
    fn overt_threshold_and_wording_are_non_diagnostic() {
        let below = compute(&IsthOvertDicInput {
            platelet_count_10_9_l: 99.999,
            d_dimer_multiple_of_uln: 7.001,
            ..baseline()
        })
        .unwrap();
        assert_eq!(below.score, 4);
        assert_eq!(below.band, OvertDicBand::BelowOvertDicThreshold);
        assert!(below.interpretation.contains("does not meet"));
        assert!(below.interpretation.contains("not excluded"));

        let overt = compute(&IsthOvertDicInput {
            platelet_count_10_9_l: 49.999,
            d_dimer_multiple_of_uln: 3.001,
            pt_prolongation_seconds: 3.0,
            ..baseline()
        })
        .unwrap();
        assert_eq!(overt.score, 5);
        assert_eq!(overt.band, OvertDicBand::ConsistentWithOvertDic);
        assert!(overt.interpretation.contains("consistent with"));
        assert!(
            overt
                .interpretation
                .contains("does not independently establish")
        );
        assert!(
            overt
                .interpretation
                .contains("does not select anticoagulants")
        );
        assert!(
            overt
                .interpretation
                .contains("does not specify a fixed interval")
        );
    }

    #[test]
    fn rejects_negative_nonfinite_unknown_and_missing_inputs() {
        for value in [-0.001, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut input = baseline();
            input.d_dimer_multiple_of_uln = value;
            assert!(compute(&input).is_err());
        }
        let value = serde_json::to_value(baseline()).unwrap();
        let mut unknown = value.clone();
        unknown["caller_supplied_score"] = json!(5);
        assert!(IsthOvertDic.calculate(&unknown).is_err());
        let mut missing = value.clone();
        missing
            .as_object_mut()
            .unwrap()
            .remove("underlying_etiology");
        assert!(IsthOvertDic.calculate(&missing).is_err());
        let mut invalid = value;
        invalid["underlying_etiology"] = json!("none");
        assert!(IsthOvertDic.calculate(&invalid).is_err());
    }

    #[test]
    fn every_etiology_is_accepted_without_changing_points() {
        for underlying_etiology in [
            DicAssociatedEtiology::SepsisOrSevereInfection,
            DicAssociatedEtiology::Malignancy,
            DicAssociatedEtiology::MajorTraumaOrTissueInjury,
            DicAssociatedEtiology::ObstetricComplication,
            DicAssociatedEtiology::VascularAbnormality,
            DicAssociatedEtiology::SevereImmunologicalOrToxicReaction,
            DicAssociatedEtiology::HeatStroke,
            DicAssociatedEtiology::PostCardiopulmonaryResuscitation,
            DicAssociatedEtiology::OtherClinicianConfirmedDicAssociatedEtiology,
        ] {
            let outcome = compute(&IsthOvertDicInput {
                underlying_etiology,
                ..baseline()
            })
            .unwrap();
            assert_eq!(outcome.score, 0);
        }
    }

    #[test]
    fn obstetric_context_carries_pregnancy_specific_warning() {
        let input = IsthOvertDicInput {
            underlying_etiology: DicAssociatedEtiology::ObstetricComplication,
            ..baseline()
        };
        let response = build_response(&input).unwrap();
        for text in [
            response.interpretation.as_str(),
            response.working["limitations"].as_str().unwrap(),
            IsthOvertDic.input_schema()["properties"]["underlying_etiology"]["definition"]
                ["caveats"]
                .as_str()
                .unwrap(),
        ] {
            assert!(text.contains("pregnancy-specific obstetric DIC criteria"));
        }
    }

    #[test]
    fn dynamic_response_matches_typed_and_preserves_audit_working() {
        let input = IsthOvertDicInput {
            platelet_count_10_9_l: 72.0,
            d_dimer_multiple_of_uln: 8.2,
            pt_prolongation_seconds: 4.1,
            fibrinogen_g_l: 1.4,
            ..baseline()
        };
        let dynamic = IsthOvertDic
            .calculate(&serde_json::to_value(input).unwrap())
            .unwrap();
        assert_eq!(dynamic, build_response(&input).unwrap());
        assert_eq!(dynamic.result, json!(5));
        assert_eq!(dynamic.working["score_version"], json!("2025"));
        assert_eq!(dynamic.working["platelet_count_points"], json!(1));
        assert_eq!(dynamic.working["d_dimer_points"], json!(3));
        assert_eq!(dynamic.working["pt_prolongation_points"], json!(1));
        assert_eq!(dynamic.working["fibrinogen_points"], json!(0));
        assert_eq!(dynamic.working["band"], json!("consistent_with_overt_dic"));
    }

    #[test]
    fn schema_is_closed_versioned_and_prevents_unit_shortcuts() {
        let schema = IsthOvertDic.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["required"].as_array().unwrap().len(), 5);
        assert!(schema["description"].as_str().unwrap().contains("2025"));
        assert!(
            schema["properties"]["d_dimer_multiple_of_uln"]["definition"]["excludes"]
                .to_string()
                .contains("raw D-dimer")
        );
        assert!(
            schema["properties"]["pt_prolongation_seconds"]["definition"]["excludes"]
                .to_string()
                .contains("INR")
        );
        for property in schema["properties"].as_object().unwrap().values() {
            assert!(property["definition"]["statement"].is_string());
            assert_eq!(property["definition"]["status"], json!("draft"));
        }
    }
}
