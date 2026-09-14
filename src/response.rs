// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The result schema returned by every calculator.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A completed calculation, ready to be rendered, recorded, or dispatched.
///
/// This mirrors the JSON object the web calculators send through
/// `gitehr-bridge.js` (`calculator`, `result`, `interpretation`, `working`,
/// `reference`), so a result produced here and one produced in the browser are
/// the same shape by construction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalculationResponse {
    /// Machine name of the calculator, e.g. `"feverpain"`.
    pub calculator: String,
    /// The primary computed value — a number or short string.
    pub result: Value,
    /// Human-readable clinical interpretation.
    pub interpretation: String,
    /// Step-by-step breakdown of how the result was reached. Locale-aware
    /// calculations also reserve `content_locale` for the canonical BCP 47
    /// bundle tag actually used to render the response.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub working: Map<String, Value>,
    /// Primary citation / guideline reference.
    pub reference: String,
}

impl CalculationResponse {
    /// A plain-text summary suitable for a clipboard / journal entry.
    ///
    /// `input` is the JSON object that was passed to
    /// [`Calculator::calculate`](crate::calculator::Calculator::calculate) to
    /// produce this response, so the pasted text alone is enough to
    /// reconstruct the calculation: which version of `clincalc` ran it, which
    /// calculator, what was entered, and what came out. Intentionally free of
    /// any timestamp — the recording host adds that, so this output stays
    /// deterministic for a given response and input.
    pub fn to_summary_text(&self, input: &Value) -> String {
        let mut out = format!(
            "clincalc {}\nCalculator: {}\nInputs:",
            env!("CARGO_PKG_VERSION"),
            self.calculator
        );
        push_key_value_lines(&mut out, input.as_object());
        out.push_str(&format!(
            "\nResult: {}\nInterpretation: {}",
            value_to_string(&self.result),
            self.interpretation
        ));
        if !self.working.is_empty() {
            out.push_str("\nWorking:");
            push_key_value_lines(&mut out, Some(&self.working));
        }
        out.push_str(&format!("\nReference: {}", self.reference));
        out
    }
}

fn push_key_value_lines(out: &mut String, map: Option<&Map<String, Value>>) {
    match map {
        Some(map) if !map.is_empty() => {
            for (key, value) in map {
                out.push_str(&format!("\n  {key}: {}", value_to_string(value)));
            }
        }
        _ => out.push_str(" (none)"),
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::CalculationResponse;
    use serde_json::json;

    #[test]
    fn summary_text_carries_version_inputs_and_working_for_reconstruction() {
        let response = CalculationResponse {
            calculator: "curb65".to_string(),
            result: json!(2),
            interpretation: "Moderate severity.".to_string(),
            working: serde_json::Map::from_iter([("confusion".to_string(), json!(false))]),
            reference: "Lim WS, et al. Thorax. 2003;58(5):377-382.".to_string(),
        };
        let input = json!({"age": 68, "confusion": false, "urea_mmol_l": 8.2});

        let out = response.to_summary_text(&input);

        assert!(out.starts_with(&format!("clincalc {}\n", env!("CARGO_PKG_VERSION"))));
        assert!(out.contains("Calculator: curb65"));
        assert!(out.contains("age: 68"));
        assert!(out.contains("urea_mmol_l: 8.2"));
        assert!(out.contains("Result: 2"));
        assert!(out.contains("Interpretation: Moderate severity."));
        assert!(out.contains("confusion: false"));
        assert!(out.contains("Reference: Lim WS, et al. Thorax. 2003;58(5):377-382."));
    }

    #[test]
    fn summary_text_marks_absent_inputs_and_working_explicitly() {
        let response = CalculationResponse {
            calculator: "example".to_string(),
            result: json!("n/a"),
            interpretation: "No inputs required.".to_string(),
            working: serde_json::Map::new(),
            reference: "Some Guideline.".to_string(),
        };

        let out = response.to_summary_text(&json!({}));

        assert!(out.contains("Inputs: (none)"));
        assert!(!out.contains("Working:"));
    }
}
