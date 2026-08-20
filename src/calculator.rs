// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The [`Calculator`] trait and its error type.

use serde_json::Value;

use crate::license::CalculatorLicense;
use crate::locale::{ENGLISH_ONLY, SupportedLocale};
use crate::response::CalculationResponse;

/// Something went wrong turning inputs into a result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    /// The supplied inputs were malformed, out of range, or the wrong shape.
    InvalidInput(String),
}

impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalcError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
        }
    }
}

impl std::error::Error for CalcError {}

/// A clinical calculator: metadata plus a dynamic JSON entrypoint.
///
/// Each implementation also exposes a strongly-typed `Input`/`compute` pair in
/// its own module; the trait is the uniform surface the CLI, MCP server, and
/// GUI dispatch through without knowing about any specific calculator.
pub trait Calculator {
    /// Machine name, e.g. `"feverpain"`. Stable; used as a CLI subcommand and
    /// MCP tool name.
    fn name(&self) -> &'static str;

    /// Human-readable title, e.g. `"FeverPAIN Score"`.
    ///
    /// This compatibility method always returns the English source text. New
    /// surfaces should resolve a locale and call [`title_for`](Self::title_for).
    fn title(&self) -> &'static str;

    /// Human-readable title in a resolved locale.
    ///
    /// Calculators without a complete translation inherit the English title.
    fn title_for(&self, _locale: SupportedLocale) -> &'static str {
        self.title()
    }

    /// One-line description of what the calculator does.
    ///
    /// This compatibility method always returns the English source text.
    fn description(&self) -> &'static str;

    /// One-line description in a resolved locale.
    fn description_for(&self, _locale: SupportedLocale) -> &'static str {
        self.description()
    }

    /// Primary citation / guideline reference.
    fn reference(&self) -> &'static str;

    /// The licence the calculator's clinical algorithm/content is distributed
    /// under, with a URL evidencing it.
    ///
    /// This is the algorithm's provenance (distinct from the AGPL code licence).
    /// It is a required method so the basis for shipping every calculator is
    /// always on record and can be re-evidenced from the cited source.
    fn license(&self) -> CalculatorLicense;

    /// JSON Schema describing the accepted inputs.
    ///
    /// Powers `clincalc calc <name> --schema` and MCP tool definitions, and lets an LLM
    /// work out the required inputs without parsing prose help.
    fn input_schema(&self) -> Value;

    /// JSON Schema with human-readable annotations in a resolved locale.
    fn input_schema_for(&self, _locale: SupportedLocale) -> Value {
        self.input_schema()
    }

    /// Compute a result from JSON inputs.
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError>;

    /// Compute a result whose human-readable prose uses a resolved locale.
    ///
    /// The default falls back the complete response to English. Implement this
    /// only when the calculator declares that locale in
    /// [`supported_locales`](Self::supported_locales).
    fn calculate_for(
        &self,
        input: &Value,
        _locale: SupportedLocale,
    ) -> Result<CalculationResponse, CalcError> {
        let mut response = self.calculate(input)?;
        response.working.insert(
            "content_locale".into(),
            Value::String(SupportedLocale::En.as_bcp47().into()),
        );
        Ok(response)
    }

    /// A fillable input template derived from [`input_schema`](Self::input_schema).
    ///
    /// Every input key is present with a placeholder describing the expected
    /// value, so a caller can fill it in and pass it straight back to
    /// [`calculate`](Self::calculate). Generated from the schema, so it cannot
    /// drift from the real contract.
    fn input_template(&self) -> Value {
        crate::template::template_from_schema(&self.input_schema())
    }

    /// A fillable input template with placeholders in a resolved locale.
    fn input_template_for(&self, locale: SupportedLocale) -> Value {
        let resolved = if self.supported_locales().contains(&locale) {
            locale
        } else {
            SupportedLocale::En
        };
        crate::template::template_from_schema_for(&self.input_schema_for(resolved), resolved)
    }

    /// Complete locale bundles available for this calculator.
    ///
    /// A locale is listed only when all of the calculator's metadata, schema
    /// prose, and computed prose have been translated and reviewed. This avoids
    /// silently mixing languages within one clinical response.
    fn supported_locales(&self) -> &'static [SupportedLocale] {
        ENGLISH_ONLY
    }

    /// Tags categorising this calculator: specialty (where it is used) and
    /// status (proprietary / unavailable / nhs-mandated / risk / ...).
    ///
    /// Used for filtering and grouping in `clincalc list --tag <t>`, the docs
    /// catalogue, and any host that enumerates the registry. The default
    /// implementation looks the calculator up by its machine name in the
    /// central [`tags::TAGS`](crate::tags::TAGS) table, so the whole taxonomy
    /// is reviewable in one file; calculators that need to override (e.g. a
    /// host adds a tag to a calculator it embeds) can implement this directly.
    fn tags(&self) -> &'static [&'static str] {
        crate::tags::for_name(self.name())
    }
}
