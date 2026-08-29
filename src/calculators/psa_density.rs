// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Prostate-specific antigen density (PSAD).
//!
//! PSAD is total serum PSA divided by imaging-derived prostate volume. It is a
//! continuous risk modifier, not a diagnosis or a stand-alone biopsy rule.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "psa_density";
pub const REFERENCE: &str = "Yusim I, Krenawi M, Mazor E, Novack V, Mabjeesh NJ. The use of prostate specific antigen density to predict clinically significant prostate cancer. Sci Rep. 2020;10:20015. doi:10.1038/s41598-020-76786-9. Benson MC, Whang IS, Pantuck A, et al. Prostate specific antigen density: a means of distinguishing benign prostatic hypertrophy and prostate cancer. J Urol. 1992;147(3 Pt 2):815-816. PMID:1371554.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "CC-BY-4.0 - formula and supporting clinical evidence adapted from Yusim et al. 2020",
    source_url: "https://doi.org/10.1038/s41598-020-76786-9",
};

const FORMULA: &str = "total_psa_ng_ml / prostate_volume_ml";
const UNIT: &str = "ng/mL/cc";
const LIMITATIONS: &str = "PSA density is a continuous risk modifier, not a diagnosis or a stand-alone biopsy decision. No cutoff is universal: interpretation depends on MRI findings and quality, examination, PSA history, family history, prior biopsy status, treatment, population, and the method used to estimate prostate volume. The cited CC BY study was a retrospective, single-centre TRUS-biopsy cohort restricted to PSA at or below 20 ng/mL, so its observed risks and thresholds do not transfer automatically to other settings. A low value does not exclude clinically significant cancer and a high value does not establish it.";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PsaDensityInput {
    pub total_psa_ng_ml: f64,
    pub prostate_volume_ml: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PsaDensityOutcome {
    pub psa_density_ng_ml_per_ml: f64,
    pub interpretation: String,
}

pub fn compute(input: &PsaDensityInput) -> Result<PsaDensityOutcome, CalcError> {
    if !input.total_psa_ng_ml.is_finite() || input.total_psa_ng_ml < 0.0 {
        return Err(CalcError::InvalidInput(
            "total_psa_ng_ml must be finite and non-negative".into(),
        ));
    }
    if !input.prostate_volume_ml.is_finite() || input.prostate_volume_ml <= 0.0 {
        return Err(CalcError::InvalidInput(
            "prostate_volume_ml must be finite and positive".into(),
        ));
    }

    let psa_density_ng_ml_per_ml = input.total_psa_ng_ml / input.prostate_volume_ml;
    if !psa_density_ng_ml_per_ml.is_finite() {
        return Err(CalcError::InvalidInput(
            "inputs produce a non-finite PSA density".into(),
        ));
    }
    let interpretation = format!(
        "PSA density is {psa_density_ng_ml_per_ml:.3} {UNIT}. Higher values are associated with a greater probability of clinically significant prostate cancer, but published reference values are context-specific. {LIMITATIONS}"
    );

    Ok(PsaDensityOutcome {
        psa_density_ng_ml_per_ml,
        interpretation,
    })
}

pub fn build_response(input: &PsaDensityInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let rounded_result = round3(outcome.psa_density_ng_ml_per_ml);
    if !rounded_result.is_finite() {
        return Err(CalcError::InvalidInput(
            "inputs are too large to round PSA density safely".into(),
        ));
    }
    let mut working = Map::new();
    working.insert("total_psa_ng_ml".into(), json!(input.total_psa_ng_ml));
    working.insert("prostate_volume_ml".into(), json!(input.prostate_volume_ml));
    working.insert(
        "psa_density_ng_ml_per_ml_unrounded".into(),
        json!(outcome.psa_density_ng_ml_per_ml),
    );
    working.insert("formula".into(), json!(FORMULA));
    working.insert("result_unit".into(), json!(UNIT));
    working.insert("limitations".into(), json!(LIMITATIONS));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(rounded_result),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

pub struct PsaDensity;

impl Calculator for PsaDensity {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Prostate-specific Antigen Density"
    }

    fn description(&self) -> &'static str {
        "Calculates continuous PSA density from total serum PSA and imaging-derived prostate volume without imposing a universal diagnostic or biopsy cutoff."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        let source = json!({
            "citation": "Yusim I et al. Sci Rep. 2020;10:20015.",
            "url": "https://doi.org/10.1038/s41598-020-76786-9"
        });
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "PsaDensityInput",
            "description": "Calculates total serum PSA divided by imaging-derived total prostate volume. PSA density is a continuous adjunct to clinical and MRI-based assessment, not a diagnosis or a stand-alone biopsy rule.",
            "type": "object",
            "additionalProperties": false,
            "required": ["total_psa_ng_ml", "prostate_volume_ml"],
            "properties": {
                "total_psa_ng_ml": {
                    "type": "number", "minimum": 0, "unit": "ng/mL",
                    "description": "Measured total serum prostate-specific antigen in ng/mL.",
                    "definition": {
                        "concept": "Total serum prostate-specific antigen for PSA density",
                        "statement": "Enter the total serum PSA result in ng/mL that is being interpreted with the supplied prostate-volume measurement.",
                        "excludes": ["Free PSA", "Percent free PSA", "A value in a different unit without conversion"],
                        "caveats": "PSA can vary with infection, inflammation, retention, instrumentation, medication, and biological or laboratory variation.",
                        "source": source, "status": "draft"
                    }
                },
                "prostate_volume_ml": {
                    "type": "number", "exclusiveMinimum": 0, "unit": "mL",
                    "description": "Imaging-derived total prostate volume in mL; 1 mL is numerically equal to 1 cc or 1 cm3.",
                    "definition": {
                        "concept": "Total prostate volume for PSA density",
                        "statement": "Enter total prostate volume measured or estimated from MRI or ultrasound in mL.",
                        "excludes": ["A linear prostate dimension", "Lesion volume", "Volume in an unconverted unit"],
                        "caveats": "The imaging modality and volume-estimation method materially affect PSA density; use the reported total volume and interpret it in that context.",
                        "source": source, "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: PsaDensityInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(psa: f64, volume: f64) -> PsaDensityInput {
        PsaDensityInput {
            total_psa_ng_ml: psa,
            prostate_volume_ml: volume,
        }
    }

    #[test]
    fn computes_source_formula_conformance_vectors() {
        // The cited paper states the quotient but does not publish paired
        // patient-level PSA and volume values as worked calculation vectors.
        for (psa, volume, expected) in [
            (6.0, 40.0, 0.15),
            (8.0, 50.0, 0.16),
            (4.0, 50.0, 0.08),
            (6.2, 31.0, 0.20),
        ] {
            let outcome = compute(&input(psa, volume)).unwrap();
            assert!((outcome.psa_density_ng_ml_per_ml - expected).abs() < 1e-12);
        }
    }

    #[test]
    fn preserves_unrounded_value_and_rounds_only_response_result() {
        let response = build_response(&input(5.99, 40.0)).unwrap();
        assert_eq!(response.result, json!(0.15));
        assert_eq!(
            response.working["psa_density_ng_ml_per_ml_unrounded"],
            json!(0.14975)
        );
        assert!(!response.working.contains_key("risk_band"));
    }

    #[test]
    fn zero_psa_is_valid() {
        assert_eq!(
            compute(&input(0.0, 40.0)).unwrap().psa_density_ng_ml_per_ml,
            0.0
        );
    }

    #[test]
    fn rejects_invalid_and_nonfinite_inputs() {
        for invalid in [
            input(-0.01, 40.0),
            input(5.0, 0.0),
            input(5.0, -1.0),
            input(f64::NAN, 40.0),
            input(5.0, f64::INFINITY),
        ] {
            assert!(compute(&invalid).is_err());
        }

        assert!(build_response(&input(1.0e308, 1.0)).is_err());
    }

    #[test]
    fn dynamic_surface_is_closed_and_matches_typed_response() {
        let value = json!({"total_psa_ng_ml": 6.0, "prostate_volume_ml": 40.0});
        assert_eq!(
            PsaDensity.calculate(&value).unwrap(),
            build_response(&input(6.0, 40.0)).unwrap()
        );
        let mut unknown = value;
        unknown["mri_result"] = json!("negative");
        assert!(PsaDensity.calculate(&unknown).is_err());
    }

    #[test]
    fn response_records_formula_unit_and_safety_limits() {
        let response = build_response(&input(6.0, 40.0)).unwrap();
        assert_eq!(response.working["formula"], json!(FORMULA));
        assert_eq!(response.working["result_unit"], json!(UNIT));
        assert!(response.interpretation.contains("not a diagnosis"));
        assert!(response.interpretation.contains("No cutoff is universal"));
        assert!(response.interpretation.contains("MRI"));
    }

    #[test]
    fn schema_is_closed_unit_explicit_and_has_no_artificial_maxima() {
        let schema = PsaDensity.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["required"].as_array().unwrap().len(), 2);
        assert_eq!(
            schema["properties"]["total_psa_ng_ml"]["unit"],
            json!("ng/mL")
        );
        assert_eq!(
            schema["properties"]["prostate_volume_ml"]["unit"],
            json!("mL")
        );
        assert!(
            schema["properties"]["total_psa_ng_ml"]
                .get("maximum")
                .is_none()
        );
        assert!(
            schema["properties"]["prostate_volume_ml"]
                .get("maximum")
                .is_none()
        );
    }
}
