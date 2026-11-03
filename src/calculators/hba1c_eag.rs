// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! HbA1c to estimated average glucose (eAG) conversion, and between the
//! NGSP (%) and IFCC (mmol/mol) HbA1c reporting units.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "hba1c_eag";
pub const REFERENCE: &str = "eAG(mg/dL) = 28.7 x HbA1c(%) - 46.7 per Nathan DM, Kuenen J, Borg R, Zheng H, Schoenfeld D, Heine RJ; A1c-Derived Average Glucose (ADAG) Study Group. Translating the A1C assay into estimated average glucose values. Diabetes Care. 2008;31(8):1473-1478. NGSP(%)/IFCC(mmol/mol) conversion per Hoelzel W, Weykamp C, Jeppsson JO, et al. IFCC reference system for measurement of hemoglobin A1c in human blood. Clin Chem. 2004;50(1):166-174.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain formulae - implemented from the primary literature",
    "https://doi.org/10.2337/dc08-0545",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hba1cUnit {
    /// NGSP/DCCT percentage, the historical and still-common US unit.
    #[serde(rename = "%")]
    Percent,
    /// IFCC mmol/mol, the UK/international standard unit.
    #[serde(rename = "mmol/mol")]
    MmolMol,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hba1cEagInput {
    pub hba1c: f64,
    pub hba1c_unit: Hba1cUnit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hba1cEagOutcome {
    pub hba1c_percent: f64,
    pub hba1c_mmol_mol: f64,
    pub eag_mg_dl: f64,
    pub eag_mmol_l: f64,
}

/// NGSP(%) -> IFCC(mmol/mol): the IFCC-NGSP master equation.
fn percent_to_mmol_mol(percent: f64) -> f64 {
    (percent - 2.15) * 10.929
}

/// IFCC(mmol/mol) -> NGSP(%): inverse of the master equation above.
fn mmol_mol_to_percent(mmol_mol: f64) -> f64 {
    mmol_mol / 10.929 + 2.15
}

pub fn compute(input: &Hba1cEagInput) -> Result<Hba1cEagOutcome, CalcError> {
    let hba1c_percent = match input.hba1c_unit {
        Hba1cUnit::Percent => {
            validate(input.hba1c, "hba1c", 3.5, 20.0)?;
            input.hba1c
        }
        Hba1cUnit::MmolMol => {
            validate(input.hba1c, "hba1c", 15.0, 195.0)?;
            mmol_mol_to_percent(input.hba1c)
        }
    };
    let hba1c_mmol_mol = match input.hba1c_unit {
        Hba1cUnit::MmolMol => input.hba1c,
        Hba1cUnit::Percent => percent_to_mmol_mol(hba1c_percent),
    };

    let eag_mg_dl = 28.7 * hba1c_percent - 46.7;
    let eag_mmol_l = eag_mg_dl / 18.0;

    Ok(Hba1cEagOutcome {
        hba1c_percent,
        hba1c_mmol_mol,
        eag_mg_dl,
        eag_mmol_l,
    })
}

fn validate(value: f64, name: &str, min: f64, max: f64) -> Result<(), CalcError> {
    if !(min..=max).contains(&value) || !value.is_finite() {
        return Err(CalcError::InvalidInput(format!(
            "{name} must be finite and between {min} and {max}"
        )));
    }
    Ok(())
}

pub fn build_response(input: &Hba1cEagInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("hba1c_percent".into(), json!(round1(o.hba1c_percent)));
    working.insert("hba1c_mmol_mol".into(), json!(round1(o.hba1c_mmol_mol)));
    working.insert("eag_mg_dl".into(), json!(round1(o.eag_mg_dl)));

    let interpretation = format!(
        "HbA1c {:.1}% ({:.0} mmol/mol) corresponds to an estimated average glucose of {:.1} mmol/L ({:.0} mg/dL) over the preceding ~2-3 months. eAG is a population-derived average from the ADAG study; it does not capture glycaemic variability and individual correlation can vary, particularly with conditions affecting red-cell turnover (e.g. haemoglobinopathies, haemolysis, recent transfusion).",
        o.hba1c_percent, o.hba1c_mmol_mol, o.eag_mmol_l, o.eag_mg_dl
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.eag_mmol_l)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct Hba1cEag;
impl Calculator for Hba1cEag {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "HbA1c / Estimated Average Glucose (eAG)"
    }
    fn description(&self) -> &'static str {
        "Converts HbA1c (NGSP % or IFCC mmol/mol) to estimated average glucose, and between the two HbA1c reporting units."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "Hba1cEagInput", "type": "object", "additionalProperties": false,
            "required": ["hba1c", "hba1c_unit"],
            "properties": {
                "hba1c": { "type": "number", "description": "HbA1c value, in the unit given by hba1c_unit" },
                "hba1c_unit": { "type": "string", "enum": ["%", "mmol/mol"], "description": "% is NGSP/DCCT (US); mmol/mol is IFCC (UK/international)" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: Hba1cEagInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_percent_matches_adag_published_value() {
        // Nathan et al 2008: HbA1c 7% -> eAG 154 mg/dL (8.6 mmol/L).
        let out = compute(&Hba1cEagInput {
            hba1c: 7.0,
            hba1c_unit: Hba1cUnit::Percent,
        })
        .unwrap();
        assert!((out.eag_mg_dl - 154.2).abs() < 0.1);
        assert!((out.eag_mmol_l - 8.57).abs() < 0.01);
    }

    #[test]
    fn ifcc_input_converts_to_ngsp_consistently() {
        // IFCC 53 mmol/mol corresponds to NGSP ~7.0%.
        let out = compute(&Hba1cEagInput {
            hba1c: 53.0,
            hba1c_unit: Hba1cUnit::MmolMol,
        })
        .unwrap();
        assert!((out.hba1c_percent - 7.0).abs() < 0.05);
    }

    #[test]
    fn percent_and_mmol_mol_round_trip() {
        let percent = 6.5_f64;
        let mmol_mol = percent_to_mmol_mol(percent);
        let back = mmol_mol_to_percent(mmol_mol);
        assert!((back - percent).abs() < 1e-9);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({"hba1c": 7.0, "hba1c_unit": "%"});
        let typed = Hba1cEagInput {
            hba1c: 7.0,
            hba1c_unit: Hba1cUnit::Percent,
        };
        let dynamic = Hba1cEag.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(Hba1cEag.calculate(&json!({"hba1c": 7.0})).is_err());
    }
}
