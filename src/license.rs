// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Distribution licence / provenance for a calculator's clinical algorithm.

use serde::{Deserialize, Serialize};

/// The terms under which a calculator's clinical algorithm or content is
/// distributed, with a URL evidencing them so the basis can be reverified.
///
/// This is the licence of the *algorithm and clinical content*, which is
/// distinct from the licence of the original clincalc source code
/// (AGPL-3.0-or-later). Third-party-derived modules retain their own licences;
/// in particular, the QRISK3 and QFracture ports are LGPL-3.0-or-later. Pure
/// scoring algorithms are generally not subject to copyright and are
/// implemented here from the primary literature; some instruments (for example
/// questionnaires) carry an explicit licence or public-domain grant from their
/// owner. Either way, every [`Calculator`](crate::Calculator) must declare one,
/// so the basis for distributing each calculator is always on record and can be
/// re-evidenced from the cited source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalculatorLicense {
    /// The licence or terms the algorithm/content is available under: an SPDX
    /// identifier where one applies, otherwise a short description (for example
    /// "Public domain - no permission required").
    pub license: &'static str,
    /// A URL evidencing the licence or terms, for reverification.
    pub source_url: &'static str,
    /// The date (`YYYY-MM-DD`) `source_url` was last confirmed to still
    /// evidence `license`, or `None` if it has never been reverified since
    /// this field was introduced. Distinct from the citation/guideline
    /// currency of the calculator itself - this tracks only the licence
    /// evidence link. See roadmap item ENG-008.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_verified: Option<&'static str>,
    /// The URL actually requested for the `last_verified` check, if it
    /// differs from `source_url` (for example a redirect target). `None`
    /// means `source_url` itself was checked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_url: Option<&'static str>,
}

impl CalculatorLicense {
    /// Construct a licence with no recorded reverification yet.
    ///
    /// This is the normal way to build a `CalculatorLicense`: almost every
    /// calculator has never been through the (not yet built) reverification
    /// pass, so `last_verified` and `verification_url` start `None`. A
    /// calculator whose licence evidence has been reverified can override
    /// them with struct-update syntax:
    ///
    /// ```
    /// # use clincalc::CalculatorLicense;
    /// const LICENSE: CalculatorLicense = CalculatorLicense {
    ///     last_verified: Some("2026-09-15"),
    ///     ..CalculatorLicense::new("Public domain", "https://example.org")
    /// };
    /// ```
    pub const fn new(license: &'static str, source_url: &'static str) -> Self {
        Self {
            license,
            source_url,
            last_verified: None,
            verification_url: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_leaves_verification_unset() {
        let lic = CalculatorLicense::new("MIT", "https://example.org");
        assert_eq!(lic.last_verified, None);
        assert_eq!(lic.verification_url, None);
    }

    #[test]
    fn unset_verification_fields_are_omitted_from_json() {
        let lic = CalculatorLicense::new("MIT", "https://example.org");
        let json = serde_json::to_value(lic).unwrap();
        assert!(!json.as_object().unwrap().contains_key("last_verified"));
        assert!(!json.as_object().unwrap().contains_key("verification_url"));
    }

    #[test]
    fn recorded_verification_serializes_alongside_the_licence() {
        let lic = CalculatorLicense {
            last_verified: Some("2026-09-15"),
            verification_url: Some("https://example.org/evidence"),
            ..CalculatorLicense::new("MIT", "https://example.org")
        };
        let json = serde_json::to_value(lic).unwrap();
        assert_eq!(json["last_verified"], "2026-09-15");
        assert_eq!(json["verification_url"], "https://example.org/evidence");
    }
}
