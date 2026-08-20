// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Locale identifiers and dependency-free locale lookup.
//!
//! Surface boundaries accept BCP 47 language tags such as `es-MX`. The engine
//! resolves them to one of the finite translation bundles compiled into a
//! calculator. HTTP `Accept-Language` parsing and locale preferences belong to
//! their respective surfaces, not this leaf module.

use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// A locale bundle compiled into `clincalc`.
///
/// The wire representation is its canonical BCP 47 tag. This enum is
/// non-exhaustive so adding a translation does not make downstream matches
/// exhaustive by accident.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLocale {
    /// English, the source locale and final fallback.
    #[default]
    En,
    /// Spanish.
    Es,
    /// Catalan.
    Ca,
}

impl SupportedLocale {
    /// The canonical BCP 47 tag for this compiled locale bundle.
    pub const fn as_bcp47(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
            Self::Ca => "ca",
        }
    }
}

impl std::fmt::Display for SupportedLocale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_bcp47())
    }
}

/// A string did not identify an exact compiled locale bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedLocale(String);

impl UnsupportedLocale {
    /// The rejected locale identifier.
    pub fn requested(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UnsupportedLocale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unsupported locale bundle: {}", self.0)
    }
}

impl std::error::Error for UnsupportedLocale {}

impl FromStr for SupportedLocale {
    type Err = UnsupportedLocale;

    /// Parse an exact supported bundle tag.
    ///
    /// Use [`lookup_locale`] when a more specific requested tag, such as
    /// `es-MX`, should resolve to an available parent bundle such as `es`.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("en") {
            Ok(Self::En)
        } else if value.eq_ignore_ascii_case("es") {
            Ok(Self::Es)
        } else if value.eq_ignore_ascii_case("ca") {
            Ok(Self::Ca)
        } else {
            Err(UnsupportedLocale(value.to_string()))
        }
    }
}

impl Serialize for SupportedLocale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_bcp47())
    }
}

impl<'de> Deserialize<'de> for SupportedLocale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(de::Error::custom)
    }
}

/// The locale set used by calculators that have not added translations.
pub const ENGLISH_ONLY: &[SupportedLocale] = &[SupportedLocale::En];

/// Every locale bundle currently compiled into the crate.
///
/// Individual calculators expose a subset through
/// [`Calculator::supported_locales`](crate::Calculator::supported_locales).
pub const COMPILED_LOCALES: &[SupportedLocale] = &[
    SupportedLocale::En,
    SupportedLocale::Es,
    SupportedLocale::Ca,
];

/// Resolve one requested BCP 47 language range against available bundles.
///
/// This implements the truncation behaviour from RFC 4647 lookup: an exact
/// match is attempted first, then subtags are removed from the right until a
/// match is found. Comparisons are ASCII case-insensitive. HTTP quality values
/// and ordered preference lists are deliberately left to the HTTP surface,
/// which can call this function once per requested range.
///
/// The function checks the structural rules needed for lookup but does not
/// claim full BCP 47 validity or CLDR canonicalisation, both of which require
/// versioned registry data.
pub fn lookup_locale(requested: &str, available: &[SupportedLocale]) -> Option<SupportedLocale> {
    let mut candidate = requested.trim();
    if candidate == "*" || !is_structurally_valid_range(candidate) {
        return None;
    }

    loop {
        if let Some(locale) = available
            .iter()
            .copied()
            .find(|locale| locale.as_bcp47().eq_ignore_ascii_case(candidate))
        {
            return Some(locale);
        }

        let separator = candidate.rfind('-')?;
        candidate = &candidate[..separator];

        // RFC 4647 removes an extension singleton together with the subtag
        // that followed it, rather than leaving a dangling `-u`, `-x`, etc.
        if candidate
            .rsplit('-')
            .next()
            .is_some_and(|subtag| subtag.len() == 1)
        {
            candidate = candidate.rsplit_once('-')?.0;
        }
    }
}

fn is_structurally_valid_range(value: &str) -> bool {
    !value.is_empty()
        && value.split('-').all(|subtag| {
            !subtag.is_empty()
                && subtag.len() <= 8
                && subtag.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_use_canonical_bcp47_spelling() {
        assert_eq!(SupportedLocale::En.as_bcp47(), "en");
        assert_eq!(SupportedLocale::Es.to_string(), "es");
        assert_eq!(SupportedLocale::Ca.to_string(), "ca");
    }

    #[test]
    fn exact_bundle_parsing_is_case_insensitive() {
        assert_eq!("EN".parse(), Ok(SupportedLocale::En));
        assert_eq!("Es".parse(), Ok(SupportedLocale::Es));
        assert_eq!("ca".parse(), Ok(SupportedLocale::Ca));
    }

    #[test]
    fn serde_uses_bcp47_tags() {
        assert_eq!(
            serde_json::to_string(&SupportedLocale::Es).unwrap(),
            r#""es""#
        );
        assert_eq!(
            serde_json::from_str::<SupportedLocale>(r#""CA""#).unwrap(),
            SupportedLocale::Ca
        );
    }

    #[test]
    fn lookup_prefers_an_exact_bundle() {
        assert_eq!(
            lookup_locale("ca", COMPILED_LOCALES),
            Some(SupportedLocale::Ca)
        );
    }

    #[test]
    fn lookup_falls_back_from_region_to_language() {
        assert_eq!(
            lookup_locale("es-MX", COMPILED_LOCALES),
            Some(SupportedLocale::Es)
        );
        assert_eq!(
            lookup_locale("ES-mx", COMPILED_LOCALES),
            Some(SupportedLocale::Es)
        );
    }

    #[test]
    fn lookup_removes_unicode_extensions() {
        assert_eq!(
            lookup_locale("es-MX-u-nu-latn", COMPILED_LOCALES),
            Some(SupportedLocale::Es)
        );
    }

    #[test]
    fn lookup_respects_the_available_calculator_bundles() {
        assert_eq!(lookup_locale("es-MX", ENGLISH_ONLY), None);
        assert_eq!(
            lookup_locale("en-GB", ENGLISH_ONLY),
            Some(SupportedLocale::En)
        );
    }

    #[test]
    fn lookup_rejects_wildcards_and_malformed_ranges() {
        assert_eq!(lookup_locale("*", COMPILED_LOCALES), None);
        assert_eq!(lookup_locale("es_MX", COMPILED_LOCALES), None);
        assert_eq!(lookup_locale("es--MX", COMPILED_LOCALES), None);
        assert_eq!(lookup_locale("", COMPILED_LOCALES), None);
    }

    #[test]
    fn unsupported_exact_bundle_reports_the_original_value() {
        let error = "es-MX".parse::<SupportedLocale>().unwrap_err();
        assert_eq!(error.requested(), "es-MX");
    }
}
