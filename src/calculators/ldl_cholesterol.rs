// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! LDL-C estimates and non-HDL-C from a standard lipid panel.
//!
//! Friedewald uses a fixed TG:VLDL-C factor of 5. Martin-Hopkins uses the
//! source-published 180-cell factor table below 400 mg/dL triglycerides and the
//! extended 240-cell table from 400 to 799 mg/dL. Sampson-NIH uses the final
//! LDL-C equation published in 2020 and is reported through 800 mg/dL.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "ldl_cholesterol";

pub const REFERENCE: &str = "Friedewald WT, Levy RI, Fredrickson DS. Estimation of the concentration of low-density lipoprotein cholesterol in plasma, without use of the preparative ultracentrifuge. Clin Chem. 1972;18(6):499-502. doi:10.1093/clinchem/18.6.499; Martin SS, Blaha MJ, Elshazly MB, et al. Comparison of a novel method vs the Friedewald equation for estimating low-density lipoprotein cholesterol levels from the standard lipid profile. JAMA. 2013;310(19):2061-2068. doi:10.1001/jama.2013.280532; Sampson M, Ling C, Sun Q, et al. A new equation for calculation of low-density lipoprotein cholesterol in patients with normolipidemia and/or hypertriglyceridemia. JAMA Cardiol. 2020;5(5):540-548. doi:10.1001/jamacardio.2020.0013; Sajja A, Park J, Sathiyakumar V, et al. Comparison of methods to estimate low-density lipoprotein cholesterol in patients with high triglyceride levels. JAMA Netw Open. 2021;4(10):e2128817. doi:10.1001/jamanetworkopen.2021.28817";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain methods - Martin-Hopkins patent application was abandoned to enable use without intellectual property restrictions",
    source_url: "https://www.hopkinsmedicine.org/news/newsroom/news-releases/2023/06/martinhopkins-method-to-calculate-ldl-or-bad-cholesterol-outperforms-other-equations-study-shows",
};

const CHOLESTEROL_MGDL_PER_MMOLL: f64 = 38.67;
const TRIGLYCERIDE_MGDL_PER_MMOLL: f64 = 88.57;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LipidUnit {
    #[serde(rename = "mg/dL")]
    MgDl,
    #[serde(rename = "mmol/L")]
    MmolL,
}

impl LipidUnit {
    fn label(self) -> &'static str {
        match self {
            Self::MgDl => "mg/dL",
            Self::MmolL => "mmol/L",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MartinHopkinsVariant {
    Original180Cell,
    Extended240Cell,
}

impl MartinHopkinsVariant {
    fn slug(self) -> &'static str {
        match self {
            Self::Original180Cell => "original_180_cell",
            Self::Extended240Cell => "extended_240_cell",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdlCholesterolInput {
    pub total_cholesterol: f64,
    pub hdl_cholesterol: f64,
    pub triglycerides: f64,
    pub unit: LipidUnit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LdlCholesterolOutcome {
    pub non_hdl_mgdl: f64,
    pub friedewald_ldl_mgdl: Option<f64>,
    pub martin_hopkins_ldl_mgdl: Option<f64>,
    pub sampson_nih_ldl_mgdl: Option<f64>,
    pub martin_hopkins_factor: Option<f64>,
    pub martin_hopkins_variant: Option<MartinHopkinsVariant>,
    pub suppressed_negative_estimates: Vec<&'static str>,
    pub interpretation: String,
}

// Figure 2 in Martin et al. 2013. Rows are TG strata; columns are non-HDL-C
// <100, 100-129, 130-159, 160-189, 190-219, and >=220 mg/dL.
const ORIGINAL_TG_UPPER_BOUNDS: [f64; 30] = [
    49.0, 56.0, 61.0, 66.0, 71.0, 75.0, 79.0, 83.0, 87.0, 92.0, 96.0, 100.0, 105.0, 110.0, 115.0,
    120.0, 126.0, 132.0, 138.0, 146.0, 154.0, 163.0, 173.0, 185.0, 201.0, 220.0, 247.0, 292.0,
    399.0, 13_975.0,
];

const ORIGINAL_FACTORS: [[f64; 6]; 30] = [
    [3.5, 3.4, 3.3, 3.3, 3.2, 3.1],
    [4.0, 3.9, 3.7, 3.6, 3.6, 3.4],
    [4.3, 4.1, 4.0, 3.9, 3.8, 3.6],
    [4.5, 4.3, 4.1, 4.0, 3.9, 3.9],
    [4.7, 4.4, 4.3, 4.2, 4.1, 3.9],
    [4.8, 4.6, 4.4, 4.2, 4.2, 4.1],
    [4.9, 4.6, 4.5, 4.3, 4.3, 4.2],
    [5.0, 4.8, 4.6, 4.4, 4.3, 4.2],
    [5.1, 4.8, 4.6, 4.5, 4.4, 4.3],
    [5.2, 4.9, 4.7, 4.6, 4.4, 4.3],
    [5.3, 5.0, 4.8, 4.7, 4.5, 4.4],
    [5.4, 5.1, 4.8, 4.7, 4.5, 4.3],
    [5.5, 5.2, 5.0, 4.7, 4.6, 4.5],
    [5.6, 5.3, 5.0, 4.8, 4.6, 4.5],
    [5.7, 5.4, 5.1, 4.9, 4.7, 4.5],
    [5.8, 5.5, 5.2, 5.0, 4.8, 4.6],
    [6.0, 5.5, 5.3, 5.0, 4.8, 4.6],
    [6.1, 5.7, 5.3, 5.1, 4.9, 4.7],
    [6.2, 5.8, 5.4, 5.2, 5.0, 4.7],
    [6.3, 5.9, 5.6, 5.3, 5.0, 4.8],
    [6.5, 6.0, 5.7, 5.4, 5.1, 4.8],
    [6.7, 6.2, 5.8, 5.4, 5.2, 4.9],
    [6.8, 6.3, 5.9, 5.5, 5.3, 5.0],
    [7.0, 6.5, 6.0, 5.7, 5.4, 5.1],
    [7.3, 6.7, 6.2, 5.8, 5.5, 5.2],
    [7.6, 6.9, 6.4, 6.0, 5.6, 5.3],
    [8.0, 7.2, 6.6, 6.2, 5.9, 5.4],
    [8.5, 7.6, 7.0, 6.5, 6.1, 5.6],
    [9.5, 8.3, 7.5, 7.0, 6.5, 5.9],
    [11.9, 10.0, 8.8, 8.1, 7.5, 6.7],
];

// eTable 1A in Sajja et al. 2021. Rows are 10 mg/dL TG strata from 400-409
// through 790-799; columns use the same six non-HDL-C strata as above.
const EXTENDED_FACTORS: [[f64; 6]; 40] = [
    [10.4, 8.7, 7.9, 7.3, 6.7, 6.1],
    [10.7, 8.9, 7.9, 7.3, 6.7, 6.0],
    [10.3, 8.9, 7.9, 7.4, 6.8, 6.0],
    [11.2, 8.9, 8.0, 7.3, 6.8, 6.0],
    [12.0, 9.0, 8.0, 7.5, 6.9, 6.0],
    [11.3, 9.3, 8.2, 7.4, 7.0, 6.0],
    [12.3, 9.2, 8.3, 7.7, 6.9, 6.1],
    [10.6, 9.3, 8.3, 7.6, 7.0, 6.0],
    [11.7, 9.3, 8.4, 7.6, 7.1, 6.1],
    [11.6, 9.6, 8.4, 7.6, 7.2, 6.2],
    [12.1, 9.2, 8.4, 7.5, 7.1, 6.2],
    [12.3, 9.9, 8.5, 7.9, 7.1, 6.3],
    [12.0, 9.8, 8.7, 7.7, 7.1, 6.3],
    [12.0, 9.8, 8.7, 7.8, 7.2, 6.3],
    [11.3, 10.0, 8.8, 7.8, 7.4, 6.3],
    [12.2, 10.2, 8.8, 8.0, 7.4, 6.2],
    [13.8, 10.2, 8.7, 8.1, 7.2, 6.2],
    [15.4, 10.4, 8.9, 8.0, 7.3, 6.2],
    [12.7, 10.5, 9.1, 8.3, 7.3, 6.4],
    [12.5, 10.5, 9.2, 8.3, 7.2, 5.9],
    [13.7, 10.5, 8.9, 8.2, 7.6, 6.3],
    [15.4, 10.5, 9.1, 8.4, 7.5, 6.4],
    [16.4, 11.3, 9.2, 8.5, 7.5, 6.4],
    [14.1, 11.6, 9.4, 8.2, 7.3, 6.2],
    [14.8, 11.0, 9.1, 8.1, 7.5, 6.6],
    [14.2, 11.0, 9.2, 8.3, 7.5, 6.4],
    [15.0, 10.9, 9.2, 8.3, 7.5, 6.5],
    [14.2, 11.0, 9.3, 8.6, 7.6, 6.7],
    [16.7, 11.5, 9.8, 8.3, 7.4, 6.7],
    [15.0, 11.6, 9.8, 8.4, 7.8, 6.5],
    [16.6, 11.5, 9.5, 8.5, 7.8, 6.9],
    [14.5, 10.9, 9.7, 8.5, 7.8, 6.4],
    [16.5, 11.7, 9.5, 8.5, 7.6, 6.6],
    [18.2, 12.2, 9.9, 8.9, 8.2, 6.6],
    [17.5, 11.7, 9.9, 8.5, 7.9, 6.6],
    [17.5, 12.9, 10.2, 8.8, 8.1, 6.4],
    [19.2, 11.4, 9.9, 8.7, 8.3, 6.5],
    [17.3, 13.4, 10.4, 8.6, 8.2, 6.7],
    [23.9, 12.3, 10.4, 9.1, 7.9, 6.7],
    [15.6, 13.0, 10.7, 8.7, 8.0, 6.7],
];

fn non_hdl_column(non_hdl_mgdl: f64) -> usize {
    if non_hdl_mgdl < 100.0 {
        0
    } else if non_hdl_mgdl < 130.0 {
        1
    } else if non_hdl_mgdl < 160.0 {
        2
    } else if non_hdl_mgdl < 190.0 {
        3
    } else if non_hdl_mgdl < 220.0 {
        4
    } else {
        5
    }
}

fn original_factor(triglycerides_mgdl: f64, non_hdl_mgdl: f64) -> Option<f64> {
    if !(7.0..400.0).contains(&triglycerides_mgdl) {
        return None;
    }
    let row = ORIGINAL_TG_UPPER_BOUNDS
        .iter()
        .position(|upper| triglycerides_mgdl <= *upper)?;
    Some(ORIGINAL_FACTORS[row][non_hdl_column(non_hdl_mgdl)])
}

fn extended_factor(triglycerides_mgdl: f64, non_hdl_mgdl: f64) -> Option<f64> {
    if !(400.0..800.0).contains(&triglycerides_mgdl) {
        return None;
    }
    let row = ((triglycerides_mgdl - 400.0) / 10.0).floor() as usize;
    Some(EXTENDED_FACTORS[row][non_hdl_column(non_hdl_mgdl)])
}

fn retain_non_negative(
    method: &'static str,
    value: f64,
    suppressed: &mut Vec<&'static str>,
) -> Option<f64> {
    if value >= 0.0 {
        Some(value)
    } else {
        suppressed.push(method);
        None
    }
}

pub fn compute(input: &LdlCholesterolInput) -> Result<LdlCholesterolOutcome, CalcError> {
    for (name, value) in [
        ("total_cholesterol", input.total_cholesterol),
        ("hdl_cholesterol", input.hdl_cholesterol),
        ("triglycerides", input.triglycerides),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be a non-negative finite number"
            )));
        }
    }
    if input.total_cholesterol <= 0.0 {
        return Err(CalcError::InvalidInput(
            "total_cholesterol must be greater than zero".into(),
        ));
    }
    if input.hdl_cholesterol > input.total_cholesterol {
        return Err(CalcError::InvalidInput(
            "hdl_cholesterol cannot exceed total_cholesterol".into(),
        ));
    }

    let (total_mgdl, hdl_mgdl, triglycerides_mgdl) = match input.unit {
        LipidUnit::MgDl => (
            input.total_cholesterol,
            input.hdl_cholesterol,
            input.triglycerides,
        ),
        LipidUnit::MmolL => (
            input.total_cholesterol * CHOLESTEROL_MGDL_PER_MMOLL,
            input.hdl_cholesterol * CHOLESTEROL_MGDL_PER_MMOLL,
            input.triglycerides * TRIGLYCERIDE_MGDL_PER_MMOLL,
        ),
    };
    let non_hdl_mgdl = total_mgdl - hdl_mgdl;
    let mut suppressed_negative_estimates = Vec::new();

    let friedewald_ldl_mgdl = (triglycerides_mgdl < 400.0)
        .then(|| {
            retain_non_negative(
                "friedewald",
                non_hdl_mgdl - triglycerides_mgdl / 5.0,
                &mut suppressed_negative_estimates,
            )
        })
        .flatten();

    let (martin_hopkins_factor, martin_hopkins_variant) =
        if let Some(factor) = original_factor(triglycerides_mgdl, non_hdl_mgdl) {
            (Some(factor), Some(MartinHopkinsVariant::Original180Cell))
        } else if let Some(factor) = extended_factor(triglycerides_mgdl, non_hdl_mgdl) {
            (Some(factor), Some(MartinHopkinsVariant::Extended240Cell))
        } else {
            (None, None)
        };
    let martin_hopkins_ldl_mgdl = martin_hopkins_factor.and_then(|factor| {
        retain_non_negative(
            "martin_hopkins",
            non_hdl_mgdl - triglycerides_mgdl / factor,
            &mut suppressed_negative_estimates,
        )
    });

    let sampson_nih_ldl_mgdl = ((16.0..=800.0).contains(&triglycerides_mgdl))
        .then(|| {
            let estimate = total_mgdl / 0.948
                - hdl_mgdl / 0.971
                - (triglycerides_mgdl / 8.56 + triglycerides_mgdl * non_hdl_mgdl / 2140.0
                    - triglycerides_mgdl.powi(2) / 16_100.0)
                - 9.44;
            retain_non_negative("sampson_nih", estimate, &mut suppressed_negative_estimates)
        })
        .flatten();

    let range_note = if (400.0..800.0).contains(&triglycerides_mgdl) {
        "Triglycerides are 400-799 mg/dL: Friedewald is not reported. The extended Martin-Hopkins estimate was more accurate than Sampson-NIH in the cited comparative study, but clinically important error remained across all methods; interpret cautiously and consider direct LDL-C or apolipoprotein B."
    } else if triglycerides_mgdl > 800.0 {
        "Triglycerides exceed 800 mg/dL, outside the supported LDL-C estimation ranges, so no LDL-C estimate is reported. Address severe hypertriglyceridaemia and use an appropriate direct or alternative atherogenic-lipoprotein measurement."
    } else if triglycerides_mgdl == 800.0 {
        "Triglycerides are exactly 800 mg/dL: only Sampson-NIH is within its proposed upper boundary; interpret this estimate cautiously and consider direct LDL-C or apolipoprotein B."
    } else if triglycerides_mgdl < 7.0 {
        "The original Martin-Hopkins lookup table begins at 7 mg/dL triglycerides, and the lowest triglyceride concentration in the cited external Sampson-NIH validation data was 16 mg/dL, so only Friedewald is reported."
    } else if triglycerides_mgdl < 16.0 {
        "The lowest triglyceride concentration in the cited external Sampson-NIH validation data was 16 mg/dL, so that estimate is not reported; Friedewald and original Martin-Hopkins remain available."
    } else {
        "Friedewald, original Martin-Hopkins, and Sampson-NIH estimates are reported. Martin-Hopkins generally improves classification when LDL-C is low or triglycerides are elevated, while all calculated LDL-C values retain estimation error."
    };
    let negative_note = if suppressed_negative_estimates.is_empty() {
        ""
    } else {
        " One or more equations produced a negative, physiologically impossible LDL-C value; those estimates were omitted rather than clamped to zero."
    };
    let interpretation = format!(
        "Non-HDL-C is calculated directly as total cholesterol minus HDL-C. {range_note}{negative_note} These LDL-C equations are unreliable in known or suspected type III dysbetalipoproteinaemia, which cannot be excluded reliably from a standard lipid panel alone. LDL-C treatment thresholds are risk-dependent; interpret these results with the full lipid profile and cardiovascular context."
    );

    Ok(LdlCholesterolOutcome {
        non_hdl_mgdl,
        friedewald_ldl_mgdl,
        martin_hopkins_ldl_mgdl,
        sampson_nih_ldl_mgdl,
        martin_hopkins_factor,
        martin_hopkins_variant,
        suppressed_negative_estimates,
        interpretation,
    })
}

fn output_value(value_mgdl: f64, unit: LipidUnit) -> f64 {
    match unit {
        LipidUnit::MgDl => (value_mgdl * 10.0).round() / 10.0,
        LipidUnit::MmolL => (value_mgdl / CHOLESTEROL_MGDL_PER_MMOLL * 1000.0).round() / 1000.0,
    }
}

fn optional_output(value_mgdl: Option<f64>, unit: LipidUnit) -> Value {
    value_mgdl
        .map(|value| json!(output_value(value, unit)))
        .unwrap_or(Value::Null)
}

pub fn build_response(input: &LdlCholesterolInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let result = json!({
        "unit": input.unit.label(),
        "non_hdl_cholesterol": output_value(o.non_hdl_mgdl, input.unit),
        "ldl_friedewald": optional_output(o.friedewald_ldl_mgdl, input.unit),
        "ldl_martin_hopkins": optional_output(o.martin_hopkins_ldl_mgdl, input.unit),
        "ldl_sampson_nih": optional_output(o.sampson_nih_ldl_mgdl, input.unit)
    });

    let mut working = Map::new();
    working.insert("input_unit".into(), json!(input.unit.label()));
    working.insert(
        "non_hdl_formula".into(),
        json!("total_cholesterol - hdl_cholesterol"),
    );
    working.insert("non_hdl_mgdl".into(), json!(o.non_hdl_mgdl));
    working.insert(
        "triglycerides_mgdl".into(),
        json!(match input.unit {
            LipidUnit::MgDl => input.triglycerides,
            LipidUnit::MmolL => input.triglycerides * TRIGLYCERIDE_MGDL_PER_MMOLL,
        }),
    );
    working.insert(
        "friedewald_formula".into(),
        json!("non_hdl_mgdl - triglycerides_mgdl / 5"),
    );
    working.insert(
        "friedewald_applicable".into(),
        json!(match input.unit {
            LipidUnit::MgDl => input.triglycerides < 400.0,
            LipidUnit::MmolL => input.triglycerides * TRIGLYCERIDE_MGDL_PER_MMOLL < 400.0,
        }),
    );
    working.insert(
        "martin_hopkins_factor".into(),
        o.martin_hopkins_factor
            .map_or(Value::Null, |factor| json!(factor)),
    );
    working.insert(
        "martin_hopkins_variant".into(),
        o.martin_hopkins_variant
            .map_or(Value::Null, |variant| json!(variant.slug())),
    );
    working.insert("sampson_nih_tg_range_mgdl".into(), json!([16, 800]));
    working.insert(
        "suppressed_negative_estimates".into(),
        json!(o.suppressed_negative_estimates),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result,
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct LdlCholesterol;

impl Calculator for LdlCholesterol {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "LDL and non-HDL Cholesterol"
    }

    fn description(&self) -> &'static str {
        "Calculates non-HDL cholesterol and estimates LDL cholesterol using Friedewald, Martin-Hopkins, and Sampson-NIH, with method-specific triglyceride limits."
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
            "title": "LdlCholesterolInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["total_cholesterol", "hdl_cholesterol", "triglycerides", "unit"],
            "properties": {
                "total_cholesterol": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Total cholesterol in the shared unit selected by unit",
                    "definition": {
                        "concept": "Total cholesterol",
                        "statement": "Measured total cholesterol from the same lipid panel as HDL-C and triglycerides.",
                        "excludes": ["Do not mix mg/dL and mmol/L values within one input object"],
                        "source": { "citation": "Martin SS et al. JAMA. 2013;310(19):2061-2068.", "url": "https://doi.org/10.1001/jama.2013.280532" },
                        "status": "draft"
                    }
                },
                "hdl_cholesterol": {
                    "type": "number",
                    "minimum": 0,
                    "description": "HDL cholesterol in the shared unit selected by unit; must not exceed total cholesterol",
                    "definition": {
                        "concept": "HDL cholesterol",
                        "statement": "Measured HDL-C from the same lipid panel as total cholesterol and triglycerides.",
                        "excludes": ["Do not supply non-HDL-C or LDL-C", "Do not mix mg/dL and mmol/L values within one input object"],
                        "source": { "citation": "Martin SS et al. JAMA. 2013;310(19):2061-2068.", "url": "https://doi.org/10.1001/jama.2013.280532" },
                        "status": "draft"
                    }
                },
                "triglycerides": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Triglycerides in the shared unit selected by unit. Friedewald is reported below 400 mg/dL; Martin-Hopkins at 7-799 mg/dL using the source-appropriate table; Sampson-NIH at 16-800 mg/dL.",
                    "definition": {
                        "concept": "Triglycerides for LDL-C estimation",
                        "statement": "Measured triglycerides from the same lipid panel. The triglyceride concentration determines which LDL-C equations are within their studied range.",
                        "excludes": ["Do not mix mg/dL and mmol/L values within one input object", "Do not infer an LDL-C estimate above a method's stated triglyceride range"],
                        "caveats": "At 400-799 mg/dL, clinically important LDL-C estimation error remains even with the extended Martin-Hopkins method. Above 800 mg/dL this calculator reports non-HDL-C but no LDL-C estimate. Martin 2013 found marked discordance in type III dysbetalipoproteinaemia, and Sampson 2020 excluded that phenotype; a standard lipid panel cannot reliably exclude it.",
                        "source": { "citation": "Sajja A et al. JAMA Netw Open. 2021;4(10):e2128817.", "url": "https://doi.org/10.1001/jamanetworkopen.2021.28817" },
                        "status": "draft"
                    }
                },
                "unit": {
                    "type": "string",
                    "enum": ["mg/dL", "mmol/L"],
                    "description": "Shared unit for all three lipid measurements"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: LdlCholesterolInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(total: f64, hdl: f64, triglycerides: f64) -> LdlCholesterolInput {
        LdlCholesterolInput {
            total_cholesterol: total,
            hdl_cholesterol: hdl,
            triglycerides,
            unit: LipidUnit::MgDl,
        }
    }

    #[test]
    fn standard_panel_matches_all_three_equations() {
        let o = compute(&input(200.0, 50.0, 150.0)).unwrap();
        assert_eq!(o.non_hdl_mgdl, 150.0);
        assert_eq!(o.friedewald_ldl_mgdl, Some(120.0));
        assert_eq!(o.martin_hopkins_factor, Some(5.7));
        assert_eq!(
            o.martin_hopkins_variant,
            Some(MartinHopkinsVariant::Original180Cell)
        );
        assert!((o.martin_hopkins_ldl_mgdl.unwrap() - 123.684_210_526).abs() < 1e-9);
        assert!((o.sampson_nih_ldl_mgdl.unwrap() - 123.397_290_615).abs() < 1e-9);
    }

    #[test]
    fn original_table_boundaries_match_published_cells() {
        assert_eq!(original_factor(7.0, 99.9), Some(3.5));
        assert_eq!(original_factor(49.0, 220.0), Some(3.1));
        assert_eq!(original_factor(50.0, 100.0), Some(3.9));
        assert_eq!(original_factor(292.0, 159.9), Some(7.0));
        assert_eq!(original_factor(293.0, 160.0), Some(7.0));
        assert_eq!(original_factor(399.0, 220.0), Some(5.9));
        assert_eq!(original_factor(6.99, 150.0), None);
        assert_eq!(original_factor(400.0, 150.0), None);
    }

    #[test]
    fn source_tables_have_all_180_and_240_cells() {
        assert_eq!(ORIGINAL_FACTORS.len() * ORIGINAL_FACTORS[0].len(), 180);
        assert_eq!(EXTENDED_FACTORS.len() * EXTENDED_FACTORS[0].len(), 240);
    }

    #[test]
    fn extended_table_boundaries_match_published_cells() {
        assert_eq!(extended_factor(400.0, 99.9), Some(10.4));
        assert_eq!(extended_factor(409.999, 220.0), Some(6.1));
        assert_eq!(extended_factor(410.0, 100.0), Some(8.9));
        assert_eq!(extended_factor(730.0, 130.0), Some(9.9));
        assert_eq!(extended_factor(799.999, 220.0), Some(6.7));
        assert_eq!(extended_factor(399.999, 150.0), None);
        assert_eq!(extended_factor(800.0, 150.0), None);
    }

    #[test]
    fn high_triglycerides_switch_to_extended_martin_hopkins() {
        let o = compute(&input(260.0, 40.0, 450.0)).unwrap();
        assert_eq!(o.friedewald_ldl_mgdl, None);
        assert_eq!(o.martin_hopkins_factor, Some(6.0));
        assert_eq!(
            o.martin_hopkins_variant,
            Some(MartinHopkinsVariant::Extended240Cell)
        );
        assert_eq!(o.martin_hopkins_ldl_mgdl, Some(145.0));
        assert!(o.sampson_nih_ldl_mgdl.is_some());
        assert!(o.interpretation.contains("clinically important error"));
    }

    #[test]
    fn exact_upper_bound_reports_only_sampson() {
        let o = compute(&input(300.0, 40.0, 800.0)).unwrap();
        assert_eq!(o.friedewald_ldl_mgdl, None);
        assert_eq!(o.martin_hopkins_ldl_mgdl, None);
        assert!(o.sampson_nih_ldl_mgdl.is_some());
    }

    #[test]
    fn sampson_is_not_extrapolated_below_external_validation_range() {
        let below = compute(&input(200.0, 50.0, 15.99)).unwrap();
        assert_eq!(below.sampson_nih_ldl_mgdl, None);
        let boundary = compute(&input(200.0, 50.0, 16.0)).unwrap();
        assert!(boundary.sampson_nih_ldl_mgdl.is_some());
    }

    #[test]
    fn above_800_reports_non_hdl_without_ldl_estimates() {
        let o = compute(&input(300.0, 40.0, 801.0)).unwrap();
        assert_eq!(o.non_hdl_mgdl, 260.0);
        assert_eq!(o.friedewald_ldl_mgdl, None);
        assert_eq!(o.martin_hopkins_ldl_mgdl, None);
        assert_eq!(o.sampson_nih_ldl_mgdl, None);
    }

    #[test]
    fn negative_estimates_are_omitted_not_clamped() {
        let o = compute(&input(100.0, 60.0, 399.0)).unwrap();
        assert_eq!(o.friedewald_ldl_mgdl, None);
        assert_eq!(o.martin_hopkins_ldl_mgdl, None);
        assert!(o.suppressed_negative_estimates.contains(&"friedewald"));
        assert!(o.suppressed_negative_estimates.contains(&"martin_hopkins"));
    }

    #[test]
    fn mmol_l_input_matches_equivalent_mgdl_panel() {
        let mgdl = compute(&input(200.0, 50.0, 150.0)).unwrap();
        let mmol = compute(&LdlCholesterolInput {
            total_cholesterol: 200.0 / CHOLESTEROL_MGDL_PER_MMOLL,
            hdl_cholesterol: 50.0 / CHOLESTEROL_MGDL_PER_MMOLL,
            triglycerides: 150.0 / TRIGLYCERIDE_MGDL_PER_MMOLL,
            unit: LipidUnit::MmolL,
        })
        .unwrap();
        assert!((mgdl.non_hdl_mgdl - mmol.non_hdl_mgdl).abs() < 1e-10);
        assert!(
            (mgdl.friedewald_ldl_mgdl.unwrap() - mmol.friedewald_ldl_mgdl.unwrap()).abs() < 1e-10
        );
        assert!(
            (mgdl.martin_hopkins_ldl_mgdl.unwrap() - mmol.martin_hopkins_ldl_mgdl.unwrap()).abs()
                < 1e-10
        );
        assert!(
            (mgdl.sampson_nih_ldl_mgdl.unwrap() - mmol.sampson_nih_ldl_mgdl.unwrap()).abs() < 1e-10
        );
    }

    #[test]
    fn rejects_invalid_panels() {
        assert!(compute(&input(0.0, 0.0, 100.0)).is_err());
        assert!(compute(&input(100.0, 101.0, 100.0)).is_err());
        assert!(compute(&input(100.0, 50.0, -1.0)).is_err());
        assert!(compute(&input(f64::NAN, 50.0, 100.0)).is_err());
    }

    #[test]
    fn response_uses_input_unit_and_null_for_unavailable_methods() {
        let response = build_response(&input(300.0, 40.0, 801.0)).unwrap();
        assert_eq!(response.result["unit"], json!("mg/dL"));
        assert_eq!(response.result["non_hdl_cholesterol"], json!(260.0));
        assert_eq!(response.result["ldl_friedewald"], Value::Null);
        assert_eq!(response.result["ldl_martin_hopkins"], Value::Null);
        assert_eq!(response.result["ldl_sampson_nih"], Value::Null);
    }

    #[test]
    fn dynamic_calculate_matches_typed_and_rejects_unknown_fields() {
        let typed = input(200.0, 50.0, 150.0);
        let value = json!({
            "total_cholesterol": 200.0,
            "hdl_cholesterol": 50.0,
            "triglycerides": 150.0,
            "unit": "mg/dL"
        });
        assert_eq!(
            LdlCholesterol.calculate(&value).unwrap(),
            build_response(&typed).unwrap()
        );

        let mut unknown = value;
        unknown["fasting"] = json!(true);
        assert!(LdlCholesterol.calculate(&unknown).is_err());
    }

    #[test]
    fn schema_records_unit_and_high_triglyceride_caveats() {
        let schema = LdlCholesterol.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(
            schema["properties"]["unit"]["enum"],
            json!(["mg/dL", "mmol/L"])
        );
        assert!(
            schema["properties"]["triglycerides"]["definition"]["caveats"]
                .as_str()
                .unwrap()
                .contains("400-799")
        );
        assert!(
            schema["properties"]["triglycerides"]["definition"]["caveats"]
                .as_str()
                .unwrap()
                .contains("type III")
        );
        assert!(
            compute(&input(200.0, 50.0, 150.0))
                .unwrap()
                .interpretation
                .contains("type III")
        );
    }
}
