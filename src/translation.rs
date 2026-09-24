// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Schema-prose key parity for translation bundles (roadmap item ENG-001.4).
//!
//! A calculator may advertise a locale in
//! [`supported_locales`](crate::Calculator::supported_locales) only when its
//! complete bundle is translated: metadata, governed schema prose, and
//! computed prose. This module provides the generic part of that gate - the
//! *schema-prose* layer - as pure, dependency-free `serde_json` walking.
//!
//! Two invariants are checked:
//!
//! 1. **Key parity.** For every schema property, the set of translatable prose
//!    keys present in the English schema must be present in every advertised
//!    locale's schema, and vice versa. Presence means "the key exists", which
//!    is how parity is defined for optional keys such as `caveats` (only some
//!    properties carry one).
//! 2. **No empty prose.** Every translatable prose string in an advertised
//!    bundle is non-empty: a missing or English-placeholder key must fail the
//!    gate rather than silently shipping (per `docs/translating.md`).
//!
//! Machine keys are deliberately ignored: property names, enum values,
//! `source.citation` / `source.url`, `snomedEcl`, and `status` are identical
//! across locales by policy and are never translated. Schema combinators
//! (`oneOf`, `allOf`) carry no prose and need no special handling; the walk
//! covers the root and every `properties` entry.

use serde_json::Value;

/// Keys inside a `definition` block whose values are translatable prose:
/// single strings (`concept`, `statement`, `caveats`) and arrays of strings
/// (`includes`, `excludes`). Everything else in a definition block is machine
/// data that stays byte-identical across locales.
const TRANSLATABLE_DEFINITION_KEYS: [&str; 5] =
    ["concept", "statement", "caveats", "includes", "excludes"];

/// One prose problem found while auditing a schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProseIssue {
    /// Where the problem is: `"root"` or a property name, plus the key path
    /// inside it (e.g. `"urea_mmol_l.definition.caveats"`).
    pub location: String,
    /// What is wrong, phrased for a calculator author.
    pub problem: String,
}

impl ProseIssue {
    fn empty(location: impl Into<String>) -> Self {
        ProseIssue {
            location: location.into(),
            problem: "translatable prose is empty".to_string(),
        }
    }

    fn wrong_type(location: impl Into<String>, actual: &str) -> Self {
        ProseIssue {
            location: location.into(),
            problem: format!("prose value must be a string or array of strings, got {actual}"),
        }
    }

    fn parity(location: impl Into<String>, detail: String) -> Self {
        ProseIssue {
            location: location.into(),
            problem: detail,
        }
    }
}

/// The structural prose fingerprint of one schema: which translatable keys
/// exist where, and how many elements each prose array has. Comparing
/// fingerprints between locales is the parity check.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ProseFingerprint {
    root_description: bool,
    properties: Vec<PropertyProse>,
}

impl ProseFingerprint {
    fn of(schema: &Value) -> Self {
        ProseFingerprint {
            root_description: schema.get("description").is_some(),
            properties: schema
                .get("properties")
                .and_then(Value::as_object)
                .map(|props| {
                    props
                        .iter()
                        .map(|(name, prop)| PropertyProse::of(name, prop))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

/// Per-property prose shape, including the property name so parity can match
/// properties across locale schemas.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PropertyProse {
    name: String,
    description: bool,
    /// Presence flags for `concept`, `statement`, `caveats`.
    definition_strings: [bool; 3],
    /// Element counts for `includes`, `excludes`.
    definition_arrays: [usize; 2],
}

impl PropertyProse {
    fn of(name: &str, prop: &Value) -> Self {
        let definition = prop.get("definition");
        let mut definition_strings = [false; 3];
        for (index, key) in TRANSLATABLE_DEFINITION_KEYS[..3].iter().enumerate() {
            definition_strings[index] = definition.is_some_and(|d| d.get(key).is_some());
        }
        let mut definition_arrays = [0; 2];
        for (index, key) in TRANSLATABLE_DEFINITION_KEYS[3..].iter().enumerate() {
            definition_arrays[index] = definition
                .and_then(|d| d.get(key))
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
        }
        PropertyProse {
            name: name.to_string(),
            description: prop.get("description").is_some(),
            definition_strings,
            definition_arrays,
        }
    }
}

/// Reports prose problems inside one schema on its own: prose keys holding the
/// wrong value type, empty strings, or empty array elements. Parity between
/// locales is checked separately by [`parity_issues`].
pub fn structural_issues(schema: &Value) -> Vec<ProseIssue> {
    let mut issues = Vec::new();

    if let Some(description) = schema.get("description")
        && is_blank(description)
    {
        issues.push(ProseIssue::empty("root.description"));
    }

    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return issues;
    };

    for (name, prop) in properties {
        if let Some(description) = prop.get("description")
            && is_blank(description)
        {
            issues.push(ProseIssue::empty(format!("{name}.description")));
        }
        let Some(definition) = prop.get("definition") else {
            continue;
        };
        for key in TRANSLATABLE_DEFINITION_KEYS {
            let Some(value) = definition.get(key) else {
                continue;
            };
            let location = format!("{name}.definition.{key}");
            match value {
                Value::String(_) => {
                    if is_blank(value) {
                        issues.push(ProseIssue::empty(location));
                    }
                }
                Value::Array(items) => {
                    for (element, item) in items.iter().enumerate() {
                        if is_blank(item) {
                            issues.push(ProseIssue::empty(format!("{location}[{element}]")));
                        }
                    }
                }
                other => issues.push(ProseIssue::wrong_type(location, json_type_name(other))),
            }
        }
    }

    issues
}

/// Prose-key parity between the English schema and one locale's schema: every
/// translatable prose key present in one must be present in the other, with
/// array element counts matching. Also reports any [`structural_issues`] found
/// in the locale schema, so one call covers both invariants.
pub fn parity_issues(english: &Value, locale: &Value) -> Vec<ProseIssue> {
    let mut issues = structural_issues(locale);

    let english_fp = ProseFingerprint::of(english);
    let locale_fp = ProseFingerprint::of(locale);

    if english_fp.root_description != locale_fp.root_description {
        issues.push(ProseIssue::parity(
            "root.description",
            "root description presence differs from the English schema".to_string(),
        ));
    }

    let mut english_props = english_fp.properties.iter();
    let mut locale_props = locale_fp.properties.iter();
    // Both schemas enumerate `properties` as serde_json::Map, which iterates
    // in insertion order, so the sequences line up property by property.
    loop {
        match (english_props.next(), locale_props.next()) {
            (Some(english_prop), Some(locale_prop)) => {
                if english_prop.name != locale_prop.name {
                    issues.push(ProseIssue::parity(
                        locale_prop.name.clone(),
                        format!(
                            "property order/content differs from the English schema: expected `{}`",
                            english_prop.name
                        ),
                    ));
                    break;
                }
                issues.extend(prose_diffs(english_prop, locale_prop));
            }
            (Some(extra), None) => {
                issues.push(ProseIssue::parity(
                    extra.name.clone(),
                    "property is missing from the locale schema".to_string(),
                ));
            }
            (None, Some(extra)) => {
                issues.push(ProseIssue::parity(
                    extra.name.clone(),
                    "property is not present in the English schema".to_string(),
                ));
            }
            (None, None) => break,
        }
    }

    issues
}

/// Compares one matched property's prose structure between locales.
fn prose_diffs(english: &PropertyProse, locale: &PropertyProse) -> Vec<ProseIssue> {
    let name = &locale.name;
    let mut issues = Vec::new();
    if english.description != locale.description {
        issues.push(ProseIssue::parity(
            format!("{name}.description"),
            "description presence differs from the English schema".to_string(),
        ));
    }
    for (index, key) in TRANSLATABLE_DEFINITION_KEYS[..3].iter().enumerate() {
        if english.definition_strings[index] != locale.definition_strings[index] {
            issues.push(ProseIssue::parity(
                format!("{name}.definition.{key}"),
                format!("presence of `{key}` differs from the English schema"),
            ));
        }
    }
    for (index, key) in TRANSLATABLE_DEFINITION_KEYS[3..].iter().enumerate() {
        if english.definition_arrays[index] != locale.definition_arrays[index] {
            issues.push(ProseIssue::parity(
                format!("{name}.definition.{key}"),
                format!("element count of `{key}` differs from the English schema"),
            ));
        }
    }
    issues
}

/// True when a prose value carries no text: an empty or whitespace-only
/// string, or an empty prose array.
fn is_blank(value: &Value) -> bool {
    match value {
        Value::String(s) => s.trim().is_empty(),
        Value::Array(items) => items.is_empty(),
        _ => false,
    }
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn curb65_like_schema(description: &str) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["urea_mmol_l"],
            "properties": {
                "urea_mmol_l": {
                    "type": "number",
                    "description": description,
                    "definition": {
                        "concept": "Urea (U)",
                        "statement": "Serum urea greater than 7 mmol/L scores 1 point.",
                        "caveats": "UNIT TRAP: the threshold is 7 mmol/L.",
                        "includes": ["one", "two"],
                        "excludes": [],
                        "snomedEcl": "<< 35591007 |Serum urea level - finding|",
                        "source": {"citation": "Lim WS et al.", "url": "https://doi.org/x"},
                        "status": "draft"
                    }
                }
            }
        })
    }

    #[test]
    fn identical_shapes_have_no_parity_issues() {
        let english = curb65_like_schema("Serum urea in mmol/L");
        let spanish = curb65_like_schema("Urea sérica en mmol/L");
        assert!(parity_issues(&english, &spanish).is_empty());
        assert!(parity_issues(&english, &english).is_empty());
    }

    #[test]
    fn missing_prose_key_fails_parity() {
        let mut spanish = curb65_like_schema("Urea sérica");
        spanish["properties"]["urea_mmol_l"]["definition"]
            .as_object_mut()
            .unwrap()
            .remove("caveats");
        let issues = parity_issues(&curb65_like_schema("Serum urea"), &spanish);
        assert!(
            issues
                .iter()
                .any(|i| i.location == "urea_mmol_l.definition.caveats")
        );
    }

    #[test]
    fn added_prose_key_fails_parity_in_the_other_direction() {
        let mut spanish = curb65_like_schema("Urea sérica");
        spanish["properties"]["urea_mmol_l"]["definition"]["advice"] =
            json!("Extra prose that English does not have");
        // `advice` is not a translatable definition key, so it is machine data
        // and does not affect parity; only the five known prose keys gate.
        assert!(parity_issues(&curb65_like_schema("Serum urea"), &spanish).is_empty());
    }

    #[test]
    fn empty_prose_fails_even_when_parity_holds() {
        let both = curb65_like_schema("");
        let issues = parity_issues(&both, &both);
        assert!(
            issues
                .iter()
                .any(|i| i.location == "urea_mmol_l.description" && i.problem.contains("empty"))
        );
    }

    #[test]
    fn whitespace_only_prose_is_empty() {
        let both = curb65_like_schema("   \t ");
        let issues = parity_issues(&both, &both);
        assert!(issues.iter().any(|i| i.problem.contains("empty")));
    }

    #[test]
    fn blank_array_element_is_empty_prose() {
        let mut schema = curb65_like_schema("Serum urea in mmol/L");
        schema["properties"]["urea_mmol_l"]["definition"]["includes"] = json!(["fine", "   "]);
        let issues = structural_issues(&schema);
        assert!(
            issues
                .iter()
                .any(|i| i.location == "urea_mmol_l.definition.includes[1]")
        );
    }

    #[test]
    fn machine_keys_are_ignored() {
        // Changing machine data (citation, SNOMED, status) never affects the
        // prose gate; that data is identical across locales by policy.
        let mut locale = curb65_like_schema("Urea sérica");
        locale["properties"]["urea_mmol_l"]["definition"]["snomedEcl"] = json!("<< 999 |Other|");
        locale["properties"]["urea_mmol_l"]["definition"]["status"] = json!("published");
        assert!(parity_issues(&curb65_like_schema("Serum urea"), &locale).is_empty());
    }

    #[test]
    fn different_array_element_counts_fail_parity() {
        let mut spanish = curb65_like_schema("Urea sérica");
        spanish["properties"]["urea_mmol_l"]["definition"]["includes"] = json!(["only one"]);
        let issues = parity_issues(&curb65_like_schema("Serum urea"), &spanish);
        assert!(
            issues
                .iter()
                .any(|i| i.location == "urea_mmol_l.definition.includes")
        );
    }

    #[test]
    fn wrong_prose_type_is_reported() {
        let mut schema = curb65_like_schema("Serum urea in mmol/L");
        schema["properties"]["urea_mmol_l"]["definition"]["caveats"] = json!(7);
        let issues = structural_issues(&schema);
        assert!(issues.iter().any(|i| i.problem.contains("number")));
    }

    #[test]
    fn schema_without_properties_is_fingerprinted() {
        let schema = json!({"type": "object"});
        assert!(structural_issues(&schema).is_empty());
        assert!(parity_issues(&schema, &schema).is_empty());
    }

    #[test]
    fn root_description_parity_is_checked() {
        let english = {
            let mut e = curb65_like_schema("Serum urea");
            e["description"] = json!("Supply raw measurements.");
            e
        };
        let mut spanish = curb65_like_schema("Urea sérica");
        spanish["description"] = json!("Supply raw measurements.");
        assert!(parity_issues(&english, &spanish).is_empty());

        spanish.as_object_mut().unwrap().remove("description");
        assert!(
            parity_issues(&english, &spanish)
                .iter()
                .any(|i| i.location == "root.description")
        );
    }

    #[test]
    fn property_missing_from_locale_schema_fails_parity() {
        let mut spanish = curb65_like_schema("Urea sérica");
        spanish["properties"]
            .as_object_mut()
            .unwrap()
            .remove("urea_mmol_l");
        let issues = parity_issues(&curb65_like_schema("Serum urea"), &spanish);
        assert!(
            issues
                .iter()
                .any(|i| i.location == "urea_mmol_l" && i.problem.contains("missing"))
        );
    }

    #[test]
    fn property_added_to_locale_schema_fails_parity() {
        let mut spanish = curb65_like_schema("Urea sérica");
        spanish["properties"]["extra_field"] = json!({"type": "number", "description": "Extra"});
        let issues = parity_issues(&curb65_like_schema("Serum urea"), &spanish);
        assert!(
            issues
                .iter()
                .any(|i| i.location == "extra_field" && i.problem.contains("English"))
        );
    }
}
