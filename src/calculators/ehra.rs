// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! EHRA symptom classification for atrial fibrillation.
//!
//! The European Heart Rhythm Association (EHRA) score classifies
//! AF-related symptoms to guide rhythm-control decisions. The 2014
//! modified classification adds class 2a/2b to separate patients
//! whose normal activities are unaffected (2a) from those troubled
//! by symptoms but still able to carry out normal activities (2b).
//!
//! Class 1:  No symptoms
//! Class 2a: Mild - normal daily activity not affected
//! Class 2b: Moderate - normal daily activity not affected but patient troubled
//! Class 3:  Severe - normal daily activity affected
//! Class 4:  Disabling - normal daily activity discontinued

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "ehra";

pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1093/europace/eut365",
};

pub const REFERENCE: &str = "Kirchhof P, Benussi S, Kotecha D, et al. 2016 ESC Guidelines for the management of \
atrial fibrillation. Eur Heart J. 2016;37(38):2893-2962. doi:10.1093/eurheartj/ehw210 | \
Wynn GJ et al. The European Heart Rhythm Association symptom classification for atrial fibrillation: \
validation and improvement through a simple modification. Europace. 2014;16(7):965-972. doi:10.1093/europace/eut365";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EhraClass {
    /// No symptoms attributable to AF
    Class1,
    /// Mild: normal daily activity not affected
    Class2a,
    /// Moderate: normal daily activity not affected but patient troubled by symptoms
    Class2b,
    /// Severe: normal daily activity affected
    Class3,
    /// Disabling: normal daily activity discontinued
    Class4,
}

impl EhraClass {
    fn label(self) -> &'static str {
        match self {
            EhraClass::Class1 => "Class 1 - No symptoms",
            EhraClass::Class2a => "Class 2a - Mild (normal activity unaffected)",
            EhraClass::Class2b => {
                "Class 2b - Moderate (normal activity unaffected but patient troubled)"
            }
            EhraClass::Class3 => "Class 3 - Severe (normal activity affected)",
            EhraClass::Class4 => "Class 4 - Disabling (normal activity discontinued)",
        }
    }

    fn slug(self) -> &'static str {
        match self {
            EhraClass::Class1 => "1",
            EhraClass::Class2a => "2a",
            EhraClass::Class2b => "2b",
            EhraClass::Class3 => "3",
            EhraClass::Class4 => "4",
        }
    }

    fn guidance(self) -> &'static str {
        match self {
            EhraClass::Class1 => {
                "Rhythm control may not be required for symptomatic benefit alone."
            }
            EhraClass::Class2a => {
                "Rhythm control may improve quality of life; individualise based on patient preference."
            }
            EhraClass::Class2b => {
                "Rhythm control is generally indicated to improve quality of life."
            }
            EhraClass::Class3 => {
                "Rhythm control is indicated; expedited assessment and treatment appropriate."
            }
            EhraClass::Class4 => {
                "Urgent rhythm control is indicated; consider emergency cardioversion if haemodynamically unstable."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EhraInput {
    pub ehra_class: EhraClass,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EhraOutcome {
    pub class: EhraClass,
    pub interpretation: String,
}

pub fn compute(input: &EhraInput) -> Result<EhraOutcome, CalcError> {
    let class = input.ehra_class;
    let interpretation = format!(
        "EHRA {}: {} {}",
        class.slug(),
        class.label(),
        class.guidance()
    );
    Ok(EhraOutcome {
        class,
        interpretation,
    })
}

pub fn build_response(input: &EhraInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("ehra_class".into(), json!(o.class.slug()));
    working.insert("label".into(), json!(o.class.label()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.class.slug()),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct Ehra;

impl Calculator for Ehra {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "EHRA AF Symptom Classification"
    }

    fn description(&self) -> &'static str {
        "Classifies atrial fibrillation symptom burden (Classes 1, 2a, 2b, 3, 4) to guide rhythm-control decisions."
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
            "title": "EhraInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["ehra_class"],
            "properties": {
                "ehra_class": {
                    "type": "string",
                    "enum": ["class1", "class2a", "class2b", "class3", "class4"],
                    "description": "EHRA symptom class: class1=no symptoms, class2a=mild/unaffected, class2b=moderate/troubled, class3=severe/affected, class4=disabling"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: EhraInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class1_no_symptoms() {
        let o = compute(&EhraInput {
            ehra_class: EhraClass::Class1,
        })
        .unwrap();
        assert_eq!(o.class, EhraClass::Class1);
        assert!(o.interpretation.contains("No symptoms"));
    }

    #[test]
    fn class4_disabling() {
        let o = compute(&EhraInput {
            ehra_class: EhraClass::Class4,
        })
        .unwrap();
        assert_eq!(o.class, EhraClass::Class4);
        assert!(o.interpretation.contains("Disabling"));
        assert!(o.interpretation.contains("Urgent"));
    }

    #[test]
    fn class2a_vs_2b_distinct() {
        let a = compute(&EhraInput {
            ehra_class: EhraClass::Class2a,
        })
        .unwrap();
        let b = compute(&EhraInput {
            ehra_class: EhraClass::Class2b,
        })
        .unwrap();
        assert_ne!(a.interpretation, b.interpretation);
        assert!(a.interpretation.contains("2a"));
        assert!(b.interpretation.contains("2b"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "ehra_class": "class3" });
        let dynamic = Ehra.calculate(&value).unwrap();
        let typed = build_response(&EhraInput {
            ehra_class: EhraClass::Class3,
        })
        .unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn rejects_unknown_class() {
        let value = json!({ "ehra_class": "class5" });
        assert!(Ehra.calculate(&value).is_err());
    }
}
