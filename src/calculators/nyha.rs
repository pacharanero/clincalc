// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! New York Heart Association (NYHA) functional classification.
//!
//! Classifies the severity of symptomatic limitation from heart disease into
//! four classes, based on how much physical activity provokes fatigue,
//! palpitation, dyspnoea, or anginal pain. It is a clinician's subjective
//! assessment, not a summed score, and does not itself constitute the
//! complete NYHA classification (which also records aetiology, anatomy, and
//! physiology) - see Hurst 2007 for the case for using the entire system.
//!
//! Class I:   No limitation of physical activity.
//! Class II:  Slight limitation; comfortable at rest.
//! Class III: Marked limitation; comfortable at rest.
//! Class IV:  Symptomatic at rest; discomfort with any activity.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "nyha";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1136/hrt.2006.089656",
};

pub const REFERENCE: &str = "The Criteria Committee of the New York Heart Association. Nomenclature and \
Criteria for Diagnosis of Diseases of the Heart and Great Vessels. 9th ed. Boston, MA: Little, Brown & Co; \
1994:253-256. | Raphael C, Briscoe C, Davies J, et al. Limitations of the New York Heart Association \
functional classification system and self-reported walking distances in chronic heart failure. Heart. \
2007;93(4):476-482. doi:10.1136/hrt.2006.089656";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NyhaClass {
    /// No limitation of physical activity
    ClassI,
    /// Slight limitation of physical activity
    ClassIi,
    /// Marked limitation of physical activity
    ClassIii,
    /// Symptomatic at rest
    ClassIv,
}

impl NyhaClass {
    fn slug(self) -> &'static str {
        match self {
            NyhaClass::ClassI => "I",
            NyhaClass::ClassIi => "II",
            NyhaClass::ClassIii => "III",
            NyhaClass::ClassIv => "IV",
        }
    }

    fn definition(self) -> &'static str {
        match self {
            NyhaClass::ClassI => {
                "No limitation of physical activity. Ordinary physical activity does not cause \
undue fatigue, palpitation, dyspnoea, or anginal pain."
            }
            NyhaClass::ClassIi => {
                "Slight limitation of physical activity. Comfortable at rest. Ordinary physical \
activity results in fatigue, palpitation, dyspnoea, or anginal pain."
            }
            NyhaClass::ClassIii => {
                "Marked limitation of physical activity. Comfortable at rest. Less than ordinary \
activity causes fatigue, palpitation, dyspnoea, or anginal pain."
            }
            NyhaClass::ClassIv => {
                "Unable to carry on any physical activity without discomfort. Symptoms of cardiac \
insufficiency may be present even at rest; discomfort increases with any physical activity."
            }
        }
    }

    fn guidance(self) -> &'static str {
        match self {
            NyhaClass::ClassI => {
                "No symptomatic limitation from cardiac disease at this assessment."
            }
            NyhaClass::ClassIi => {
                "Mild symptomatic limitation; a reasonable point to review guideline-directed \
therapy and modifiable risk factors."
            }
            NyhaClass::ClassIii => {
                "Marked symptomatic limitation; consider specialist heart-failure review and \
optimisation of guideline-directed therapy."
            }
            NyhaClass::ClassIv => "Symptomatic at rest; urgent specialist assessment is indicated.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NyhaInput {
    pub nyha_class: NyhaClass,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NyhaOutcome {
    pub class: NyhaClass,
    pub interpretation: String,
}

pub fn compute(input: &NyhaInput) -> Result<NyhaOutcome, CalcError> {
    let class = input.nyha_class;
    let interpretation = format!(
        "NYHA Class {}: {} {}",
        class.slug(),
        class.definition(),
        class.guidance()
    );
    Ok(NyhaOutcome {
        class,
        interpretation,
    })
}

pub fn build_response(input: &NyhaInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("nyha_class".into(), json!(o.class.slug()));
    working.insert("definition".into(), json!(o.class.definition()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.class.slug()),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Nyha;

impl Calculator for Nyha {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "NYHA Functional Classification"
    }

    fn description(&self) -> &'static str {
        "Classifies heart-failure functional capacity (Class I-IV) by how much physical activity provokes fatigue, palpitation, dyspnoea, or anginal pain."
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
            "title": "NyhaInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["nyha_class"],
            "properties": {
                "nyha_class": {
                    "type": "string",
                    "enum": ["class_i", "class_ii", "class_iii", "class_iv"],
                    "description": "NYHA functional class: class_i=no limitation, class_ii=slight limitation, class_iii=marked limitation, class_iv=symptomatic at rest"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: NyhaInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_i_no_limitation() {
        let o = compute(&NyhaInput {
            nyha_class: NyhaClass::ClassI,
        })
        .unwrap();
        assert_eq!(o.class, NyhaClass::ClassI);
        assert!(o.interpretation.contains("No limitation"));
    }

    #[test]
    fn class_iv_symptomatic_at_rest() {
        let o = compute(&NyhaInput {
            nyha_class: NyhaClass::ClassIv,
        })
        .unwrap();
        assert_eq!(o.class, NyhaClass::ClassIv);
        assert!(o.interpretation.contains("Class IV"));
        assert!(o.interpretation.contains("urgent"));
    }

    #[test]
    fn class_ii_vs_iii_distinct() {
        let a = compute(&NyhaInput {
            nyha_class: NyhaClass::ClassIi,
        })
        .unwrap();
        let b = compute(&NyhaInput {
            nyha_class: NyhaClass::ClassIii,
        })
        .unwrap();
        assert_ne!(a.interpretation, b.interpretation);
        assert!(a.interpretation.contains("Class II:"));
        assert!(b.interpretation.contains("Class III:"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "nyha_class": "class_iii" });
        let dynamic = Nyha.calculate(&value).unwrap();
        let typed = build_response(&NyhaInput {
            nyha_class: NyhaClass::ClassIii,
        })
        .unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn rejects_unknown_class() {
        let value = json!({ "nyha_class": "class_v" });
        assert!(Nyha.calculate(&value).is_err());
    }

    #[test]
    fn rejects_unknown_fields() {
        let value = json!({ "nyha_class": "class_i", "extra": true });
        assert!(Nyha.calculate(&value).is_err());
    }
}
