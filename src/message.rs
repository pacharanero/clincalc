// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Structured descriptors for localised clinical messages.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A stable semantic message ID plus named, JSON-typed arguments.
///
/// Renderers translate the complete message identified by [`id`](Self::id).
/// Arguments remain structured until rendering so locale never changes their
/// clinical value or type. IDs and argument names are stable English ASCII
/// slugs and are not themselves translated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClinicalMessage {
    /// Stable semantic identifier, e.g. `curb65.interpretation.high`.
    pub id: String,
    /// Named values interpolated or selected by the translated message.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub arguments: Map<String, Value>,
}

impl ClinicalMessage {
    /// Start a descriptor with no arguments.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            arguments: Map::new(),
        }
    }

    /// Add one named argument.
    #[must_use]
    pub fn with_argument(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.arguments.insert(name.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialises_stable_id_and_typed_named_arguments() {
        let message = ClinicalMessage::new("curb65.interpretation.high")
            .with_argument("score", 4)
            .with_argument("mortality_percent", 41.5);

        assert_eq!(
            serde_json::to_value(message).unwrap(),
            serde_json::json!({
                "id": "curb65.interpretation.high",
                "arguments": {
                    "score": 4,
                    "mortality_percent": 41.5
                }
            })
        );
    }
}
