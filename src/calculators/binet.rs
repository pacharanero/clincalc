// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Binet staging for established chronic lymphocytic leukaemia (CLL).
//!
//! Stage C cytopenia takes precedence over the number of involved lymphoid
//! areas. In the absence of stage C cytopenia, three or more involved areas are
//! stage B and fewer than three are stage A. This historical prognostic stage
//! is based on physical examination and blood counts, not CT-only nodes.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "binet";

/// Original Binet method and modern iwCLL clinical operationalisation.
pub const REFERENCE: &str = "Binet JL, Auquier A, Dighiero G, et al. A new prognostic classification of chronic lymphocytic leukemia derived from a multivariate survival analysis. Cancer. 1981;48(1):198-206. PMID:7237385. Hallek M, Cheson BD, Catovsky D, et al. iwCLL guidelines for diagnosis, indications for treatment, response assessment, and supportive management of CLL. Blood. 2018;131(25):2745-2760. doi:10.1182/blood-2017-09-806398.";

/// Distribution licence: independently implemented from the published method.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Published clinical method - independently implemented from the primary literature",
    source_url: "https://pubmed.ncbi.nlm.nih.gov/7237385/",
};

/// Inputs for Binet staging of established CLL.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinetInput {
    /// Confirm that CLL has already been diagnosed.
    pub cll_diagnosis_confirmed: bool,
    /// Haemoglobin concentration in g/dL.
    pub haemoglobin_g_dl: f64,
    /// Platelet count in 10^9/L.
    pub platelet_count_10_9_l: f64,
    /// Physical-examination involvement of head/neck, including Waldeyer ring.
    pub head_and_neck_involved: bool,
    /// Physical-examination involvement of either or both axillae.
    pub axillae_involved: bool,
    /// Physical-examination involvement of either or both groins, including superficial femoral nodes.
    pub groins_involved: bool,
    /// Palpable splenic involvement.
    pub spleen_involved: bool,
    /// Palpable clinically enlarged liver.
    pub liver_involved: bool,
}

/// Binet stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinetStage {
    A,
    B,
    C,
}

impl BinetStage {
    fn letter(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
        }
    }
}

/// Typed Binet staging outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinetOutcome {
    pub stage: BinetStage,
    pub involved_area_count: u8,
    pub haemoglobin_below_10_g_dl: bool,
    pub platelet_count_below_100_10_9_l: bool,
    pub interpretation: String,
}

/// Calculate the Binet stage from physical-examination areas and blood counts.
pub fn compute(input: &BinetInput) -> Result<BinetOutcome, CalcError> {
    if !input.cll_diagnosis_confirmed {
        return Err(CalcError::InvalidInput(
            "Binet staging requires an established CLL diagnosis; do not use it to diagnose CLL or stage MBL or SLL"
                .into(),
        ));
    }
    if !input.haemoglobin_g_dl.is_finite()
        || input.haemoglobin_g_dl <= 0.0
        || input.haemoglobin_g_dl > 30.0
    {
        return Err(CalcError::InvalidInput(
            "haemoglobin_g_dl must be finite, greater than 0, and no greater than 30 g/dL".into(),
        ));
    }
    if !input.platelet_count_10_9_l.is_finite()
        || !(0.0..=2000.0).contains(&input.platelet_count_10_9_l)
    {
        return Err(CalcError::InvalidInput(
            "platelet_count_10_9_l must be finite and between 0 and 2000 x10^9/L inclusive".into(),
        ));
    }

    let involved_area_count = [
        input.head_and_neck_involved,
        input.axillae_involved,
        input.groins_involved,
        input.spleen_involved,
        input.liver_involved,
    ]
    .into_iter()
    .filter(|involved| *involved)
    .count() as u8;
    let haemoglobin_below_10_g_dl = input.haemoglobin_g_dl < 10.0;
    let platelet_count_below_100_10_9_l = input.platelet_count_10_9_l < 100.0;

    let stage = if haemoglobin_below_10_g_dl || platelet_count_below_100_10_9_l {
        BinetStage::C
    } else if involved_area_count >= 3 {
        BinetStage::B
    } else {
        BinetStage::A
    };

    let interpretation = format!(
        "Binet stage {} ({} of 5 lymphoid areas involved; haemoglobin below 10 g/dL: {}; platelet count below 100 x10^9/L: {}). This is a historical prognostic stage, not a treatment instruction. The cause and trajectory of cytopenias require clinician attribution. Binet staging omits molecular and other modern prognostic factors.",
        stage.letter(),
        involved_area_count,
        haemoglobin_below_10_g_dl,
        platelet_count_below_100_10_9_l,
    );

    Ok(BinetOutcome {
        stage,
        involved_area_count,
        haemoglobin_below_10_g_dl,
        platelet_count_below_100_10_9_l,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &BinetInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "cll_diagnosis_confirmed".into(),
        json!(input.cll_diagnosis_confirmed),
    );
    working.insert("haemoglobin_g_dl".into(), json!(input.haemoglobin_g_dl));
    working.insert("haemoglobin_unit".into(), json!("g/dL"));
    working.insert(
        "platelet_count_10_9_l".into(),
        json!(input.platelet_count_10_9_l),
    );
    working.insert("platelet_count_unit".into(), json!("x10^9/L"));
    working.insert(
        "head_and_neck_involved".into(),
        json!(input.head_and_neck_involved),
    );
    working.insert("axillae_involved".into(), json!(input.axillae_involved));
    working.insert("groins_involved".into(), json!(input.groins_involved));
    working.insert("spleen_involved".into(), json!(input.spleen_involved));
    working.insert("liver_involved".into(), json!(input.liver_involved));
    working.insert(
        "involved_area_count".into(),
        json!(outcome.involved_area_count),
    );
    working.insert(
        "haemoglobin_below_10_g_dl".into(),
        json!(outcome.haemoglobin_below_10_g_dl),
    );
    working.insert(
        "platelet_count_below_100_10_9_l".into(),
        json!(outcome.platelet_count_below_100_10_9_l),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.stage.letter()),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn input_schema() -> Value {
    let source = json!({
        "citation": "Hallek M, Cheson BD, Catovsky D, et al. Blood. 2018;131(25):2745-2760.",
        "url": "https://doi.org/10.1182/blood-2017-09-806398"
    });
    let area_caveat = "Determine involvement by physical examination. CT-only nodes do not alter Binet stage. Bilateral or multiple nodal groups within one named area count once.";

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "BinetInput",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "cll_diagnosis_confirmed",
            "haemoglobin_g_dl",
            "platelet_count_10_9_l",
            "head_and_neck_involved",
            "axillae_involved",
            "groins_involved",
            "spleen_involved",
            "liver_involved"
        ],
        "properties": {
            "cll_diagnosis_confirmed": {
                "type": "boolean",
                "const": true,
                "description": "Confirm that chronic lymphocytic leukaemia (CLL) is already diagnosed; Binet staging is not a diagnostic test",
                "definition": {
                    "concept": "Established CLL diagnosis",
                    "statement": "Use Binet staging only after CLL has been established by appropriate diagnostic criteria.",
                    "excludes": ["Diagnosis of suspected CLL", "Monoclonal B-cell lymphocytosis (MBL)", "Small lymphocytic lymphoma (SLL) without established CLL"],
                    "caveats": "This input confirms applicability but does not contribute to the stage.",
                    "source": source,
                    "status": "draft"
                }
            },
            "haemoglobin_g_dl": {
                "type": "number",
                "exclusiveMinimum": 0,
                "maximum": 30,
                "unit": "g/dL",
                "description": "Haemoglobin concentration in g/dL; stage C threshold is strictly below 10 g/dL",
                "definition": {
                    "concept": "Haemoglobin concentration",
                    "statement": "Record the current haemoglobin concentration. A value below 10 g/dL meets the Binet stage C cytopenia threshold.",
                    "caveats": "The accepted range is broad analytic plausibility, not a staging interval. Clinicians must attribute the cause and assess the trajectory of anaemia.",
                    "source": source,
                    "status": "draft"
                }
            },
            "platelet_count_10_9_l": {
                "type": "number",
                "minimum": 0,
                "maximum": 2000,
                "unit": "x10^9/L",
                "description": "Platelet count in x10^9/L; stage C threshold is strictly below 100 x10^9/L",
                "definition": {
                    "concept": "Platelet count",
                    "statement": "Record the current platelet count. A value below 100 x10^9/L meets the Binet stage C cytopenia threshold.",
                    "caveats": "The accepted range is broad analytic plausibility, not a staging interval. Clinicians must attribute the cause and assess the trajectory of thrombocytopenia.",
                    "source": source,
                    "status": "draft"
                }
            },
            "head_and_neck_involved": {
                "type": "boolean",
                "description": "Physical-examination involvement of the head and neck lymphoid area, including Waldeyer ring; clinically enlarged nodes are at least 1 cm, and all groups in this area count once",
                "definition": {
                    "concept": "Head and neck lymphoid area involvement",
                    "statement": "Clinically enlarged lymph nodes in the head or neck, including Waldeyer ring, are present on physical examination.",
                    "includes": ["Waldeyer ring", "One or more head or neck nodal groups", "Bilateral head or neck nodes, counted as one area"],
                    "excludes": ["CT-only lymphadenopathy", "Counting separate or bilateral groups more than once"],
                    "caveats": area_caveat,
                    "source": source,
                    "status": "draft"
                }
            },
            "axillae_involved": {
                "type": "boolean",
                "description": "Physical-examination involvement of the axillary lymphoid area; clinically enlarged nodes are at least 1 cm, and unilateral, bilateral, or multiple axillary groups count once",
                "definition": {
                    "concept": "Axillary lymphoid area involvement",
                    "statement": "Clinically enlarged lymph nodes in either or both axillae are present on physical examination.",
                    "includes": ["Unilateral axillary nodes", "Bilateral axillary nodes, counted as one area", "Multiple axillary groups, counted as one area"],
                    "excludes": ["CT-only lymphadenopathy", "Counting left and right axillae separately"],
                    "caveats": area_caveat,
                    "source": source,
                    "status": "draft"
                }
            },
            "groins_involved": {
                "type": "boolean",
                "description": "Physical-examination involvement of the groin lymphoid area, including superficial femoral nodes; clinically enlarged nodes are at least 1 cm, and both groins count once",
                "definition": {
                    "concept": "Groin lymphoid area involvement",
                    "statement": "Clinically enlarged lymph nodes in either or both groins, including superficial femoral nodes, are present on physical examination.",
                    "includes": ["Inguinal nodes", "Superficial femoral nodes", "Bilateral groin nodes, counted as one area"],
                    "excludes": ["CT-only lymphadenopathy", "Counting left and right groins separately"],
                    "caveats": area_caveat,
                    "source": source,
                    "status": "draft"
                }
            },
            "spleen_involved": {
                "type": "boolean",
                "description": "Palpable splenic enlargement on physical examination; the spleen is one lymphoid area",
                "definition": {
                    "concept": "Splenic involvement",
                    "statement": "The spleen is palpable on physical examination and therefore counts as one involved lymphoid area.",
                    "excludes": ["Imaging-only splenic enlargement", "Counting degree of enlargement as multiple areas"],
                    "caveats": area_caveat,
                    "source": source,
                    "status": "draft"
                }
            },
            "liver_involved": {
                "type": "boolean",
                "description": "Palpable clinically enlarged liver on physical examination; the liver is one lymphoid area",
                "definition": {
                    "concept": "Hepatic involvement",
                    "statement": "The liver is palpably and clinically enlarged on physical examination and therefore counts as one involved lymphoid area.",
                    "excludes": ["Imaging-only liver enlargement", "A normally palpable liver edge without clinical enlargement", "Counting degree of enlargement as multiple areas"],
                    "caveats": area_caveat,
                    "source": source,
                    "status": "draft"
                }
            }
        }
    })
}

/// Dynamic calculator implementation.
pub struct Binet;

impl Calculator for Binet {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Binet Stage for Chronic Lymphocytic Leukaemia"
    }

    fn description(&self) -> &'static str {
        "Assigns historical Binet stage A, B, or C for established CLL from five physical-examination lymphoid areas, haemoglobin, and platelet count."
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
        let parsed: BinetInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::SupportedLocale;

    fn input(haemoglobin_g_dl: f64, platelet_count_10_9_l: f64, areas: [bool; 5]) -> BinetInput {
        BinetInput {
            cll_diagnosis_confirmed: true,
            haemoglobin_g_dl,
            platelet_count_10_9_l,
            head_and_neck_involved: areas[0],
            axillae_involved: areas[1],
            groins_involved: areas[2],
            spleen_involved: areas[3],
            liver_involved: areas[4],
        }
    }

    #[test]
    fn exact_stage_vectors() {
        for (haemoglobin, platelets, areas, expected) in [
            (14.0, 200.0, [false; 5], BinetStage::A),
            (
                10.0,
                100.0,
                [true, true, false, false, false],
                BinetStage::A,
            ),
            (10.0, 100.0, [true, true, true, false, false], BinetStage::B),
            (12.0, 150.0, [true; 5], BinetStage::B),
            (9.999, 100.0, [false; 5], BinetStage::C),
            (10.0, 99.999, [false; 5], BinetStage::C),
        ] {
            assert_eq!(
                compute(&input(haemoglobin, platelets, areas))
                    .unwrap()
                    .stage,
                expected
            );
        }
    }

    #[test]
    fn stage_c_takes_precedence_over_all_five_areas() {
        let outcome = compute(&input(9.0, 50.0, [true; 5])).unwrap();
        assert_eq!(outcome.stage, BinetStage::C);
        assert_eq!(outcome.involved_area_count, 5);
        assert!(outcome.haemoglobin_below_10_g_dl);
        assert!(outcome.platelet_count_below_100_10_9_l);
    }

    #[test]
    fn bilateral_or_multiple_groups_in_one_named_area_count_once() {
        let outcome = compute(&input(14.0, 200.0, [false, true, false, false, false])).unwrap();
        assert_eq!(outcome.involved_area_count, 1);

        let schema = Binet.input_schema();
        assert!(
            schema["properties"]["axillae_involved"]["description"]
                .as_str()
                .unwrap()
                .contains("bilateral")
        );
        assert!(
            schema["properties"]["axillae_involved"]["description"]
                .as_str()
                .unwrap()
                .contains("count once")
        );
    }

    #[test]
    fn rejects_unconfirmed_diagnosis_nonfinite_and_out_of_range_values() {
        let mut unconfirmed = input(14.0, 200.0, [false; 5]);
        unconfirmed.cll_diagnosis_confirmed = false;
        assert!(compute(&unconfirmed).is_err());

        for haemoglobin in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            0.0,
            -1.0,
            30.001,
        ] {
            assert!(compute(&input(haemoglobin, 200.0, [false; 5])).is_err());
        }
        for platelets in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.001, 2000.001] {
            assert!(compute(&input(14.0, platelets, [false; 5])).is_err());
        }
    }

    #[test]
    fn plausibility_boundaries_are_accepted_and_stage_still_derives() {
        let high = compute(&input(30.0, 2000.0, [false; 5])).unwrap();
        assert_eq!(high.stage, BinetStage::A);

        let zero_platelets = compute(&input(30.0, 0.0, [false; 5])).unwrap();
        assert_eq!(zero_platelets.stage, BinetStage::C);
        assert!(zero_platelets.platelet_count_below_100_10_9_l);
    }

    #[test]
    fn response_contains_canonical_result_and_complete_working() {
        let value = input(12.0, 150.0, [true, true, true, false, false]);
        let response = build_response(&value).unwrap();

        assert_eq!(response.result, json!("B"));
        assert_eq!(response.working["cll_diagnosis_confirmed"], json!(true));
        assert_eq!(response.working["haemoglobin_g_dl"], json!(12.0));
        assert_eq!(response.working["haemoglobin_unit"], json!("g/dL"));
        assert_eq!(response.working["platelet_count_10_9_l"], json!(150.0));
        assert_eq!(response.working["platelet_count_unit"], json!("x10^9/L"));
        assert_eq!(response.working["head_and_neck_involved"], json!(true));
        assert_eq!(response.working["axillae_involved"], json!(true));
        assert_eq!(response.working["groins_involved"], json!(true));
        assert_eq!(response.working["spleen_involved"], json!(false));
        assert_eq!(response.working["liver_involved"], json!(false));
        assert_eq!(response.working["involved_area_count"], json!(3));
        assert_eq!(response.working["haemoglobin_below_10_g_dl"], json!(false));
        assert_eq!(
            response.working["platelet_count_below_100_10_9_l"],
            json!(false)
        );
        assert!(
            response
                .interpretation
                .contains("not a treatment instruction")
        );
        assert!(!response.interpretation.contains("survival"));
    }

    #[test]
    fn dynamic_calculation_matches_typed_and_rejects_invalid_objects() {
        let typed_input = input(12.0, 150.0, [true, true, true, false, false]);
        let dynamic = Binet
            .calculate(&serde_json::to_value(typed_input).unwrap())
            .unwrap();
        assert_eq!(dynamic, build_response(&typed_input).unwrap());

        let mut unknown = serde_json::to_value(typed_input).unwrap();
        unknown["ct_only_nodes_involved"] = json!(true);
        assert!(Binet.calculate(&unknown).is_err());

        let mut unconfirmed = serde_json::to_value(typed_input).unwrap();
        unconfirmed["cll_diagnosis_confirmed"] = json!(false);
        assert!(Binet.calculate(&unconfirmed).is_err());

        let mut out_of_range = serde_json::to_value(typed_input).unwrap();
        out_of_range["haemoglobin_g_dl"] = json!(31.0);
        assert!(Binet.calculate(&out_of_range).is_err());
    }

    #[test]
    fn schema_is_closed_complete_and_documents_area_semantics() {
        let schema = Binet.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["required"].as_array().unwrap().len(), 8);
        assert_eq!(
            schema["properties"]["cll_diagnosis_confirmed"]["const"],
            json!(true)
        );
        assert_eq!(
            schema["properties"]["haemoglobin_g_dl"]["unit"],
            json!("g/dL")
        );
        assert_eq!(
            schema["properties"]["platelet_count_10_9_l"]["unit"],
            json!("x10^9/L")
        );

        for area in [
            "head_and_neck_involved",
            "axillae_involved",
            "groins_involved",
            "spleen_involved",
            "liver_involved",
        ] {
            let definition = &schema["properties"][area]["definition"];
            assert!(
                definition["caveats"]
                    .as_str()
                    .unwrap()
                    .contains("physical examination")
            );
            assert!(
                definition["caveats"]
                    .as_str()
                    .unwrap()
                    .contains("count once")
            );
            assert!(definition["caveats"].as_str().unwrap().contains("CT-only"));
        }
    }

    #[test]
    fn calculate_for_records_english_content_locale() {
        let response = Binet
            .calculate_for(
                &serde_json::to_value(input(14.0, 200.0, [false; 5])).unwrap(),
                SupportedLocale::En,
            )
            .unwrap();
        assert_eq!(response.working["content_locale"], json!("en"));
    }
}
