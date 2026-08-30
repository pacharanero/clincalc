// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # clincalc
//!
//! Open, auditable clinical calculators: the pure scoring engine and the `clincalc`
//! command-line surface in one crate.
//!
//! ## Feature flags
//!
//! - `default = ["cli", "rest-api"]` builds the standalone `clincalc` binary,
//!   exposes the reusable [`cli`] module, and enables `clincalc api`.
//! - `mcp` adds the optional local stdio MCP server used by `clincalc mcp`. It
//!   pulls in the MCP SDK and async runtime behind the feature gate.
//! - `default-features = false` builds only the leaf engine: calculators,
//!   registry, schemas, tags, licences, and [`CalculationResponse`]. This mode
//!   depends only on `serde` and `serde_json`.
//!
//! ```toml
//! clincalc = { version = "0.2", default-features = false }
//! ```
//!
//! ## One registry, many surfaces
//!
//! With `default-features = false` this crate is a strict **leaf**: it never
//! depends on a host application and never on an async runtime. That is what lets
//! the same logic drive every surface without divergence:
//!
//! - the standalone `clincalc` binary (the default `cli` feature)
//! - host CLIs that embed the `cli` module (for example GitEHR's `gitehr calc`)
//! - the optional MCP server (each calculator exposed as a tool)
//! - a native desktop GUI (called natively over a Tauri command)
//! - the single-file web calculators
//!
//! Adding a calculator to [`all()`] surfaces it everywhere.
//!
//! ```rust
//! let calc = clincalc::get("feverpain").expect("calculator exists");
//! let input = serde_json::json!({
//!     "fever": true,
//!     "purulence": true,
//!     "attend_rapidly": true,
//!     "inflamed_tonsils": false,
//!     "absence_of_cough": false
//! });
//! let response = calc.calculate(&input).expect("valid input");
//! assert_eq!(response.calculator, "feverpain");
//! ```
//!
//! Every calculator implements the [`Calculator`] trait and returns a
//! [`CalculationResponse`] - the Rust counterpart of the JSON schema the web
//! calculators dispatch via `gitehr-bridge.js`.
//!
//! Scoring functions are pure: no clock, no I/O, no global state. A host that
//! needs a timestamp stamps it when recording the result, so the core stays
//! deterministic and trivially testable.

// Some calculators (e.g. QRISK3, QFracture) declare large JSON Schemas via the
// `json!` macro, which recurses once per property; raise the limit so they
// compile without per-file workarounds.
#![recursion_limit = "256"]

pub mod calculator;
pub mod calculators;
pub mod license;
pub mod locale;
pub mod message;
pub mod proprietary;
pub mod response;
pub mod tags;
pub mod template;

/// The command-line surface (`CalcCommand`, `run`), behind the `cli` feature.
/// Embeddable by host CLIs such as GitEHR's `gitehr calc`.
#[cfg(feature = "cli")]
pub mod cli;

/// The Model Context Protocol server surface, behind the optional `mcp` feature.
#[cfg(feature = "mcp")]
pub mod mcp;

/// The HTTP REST API surface, behind the optional `rest-api` feature.
#[cfg(feature = "rest-api")]
pub mod api;

pub use calculator::{CalcError, Calculator};
pub use license::CalculatorLicense;
pub use locale::{
    COMPILED_LOCALES, ENGLISH_ONLY, SupportedLocale, UnsupportedLocale, lookup_locale,
};
pub use message::ClinicalMessage;
pub use proprietary::ProprietaryCalculator;
pub use response::CalculationResponse;
pub use tags::{all_tags, for_name as tags_for_name};
pub use template::template_from_schema;

/// Every calculator known to the engine, in display order.
///
/// This is the single registry the CLI, MCP server, and GUI all enumerate, so
/// adding a calculator in one place surfaces it everywhere.
pub fn all() -> Vec<Box<dyn Calculator>> {
    let mut list: Vec<Box<dyn Calculator>> = vec![
        Box::new(calculators::feverpain::FeverPain),
        Box::new(calculators::centor::Centor),
        Box::new(calculators::alvarado::Alvarado),
        Box::new(calculators::apgar::Apgar),
        Box::new(calculators::alcohol_units::AlcoholUnits),
        Box::new(calculators::asrs::Asrs),
        Box::new(calculators::phq9::Phq9),
        Box::new(calculators::gad7::Gad7),
        Box::new(calculators::egfr::Egfr),
        Box::new(calculators::cockcroft_gault::CockcroftGault),
        Box::new(calculators::fena::Fena),
        Box::new(calculators::free_water_deficit::FreeWaterDeficit),
        Box::new(calculators::corrected_calcium::CorrectedCalcium),
        Box::new(calculators::corrected_sodium::CorrectedSodium),
        Box::new(calculators::anion_gap::AnionGap),
        Box::new(calculators::energy_requirement::EnergyRequirement),
        Box::new(calculators::bmi::Bmi),
        Box::new(calculators::body_fat_circumference::BodyFatCircumference),
        Box::new(calculators::body_surface_area::BodySurfaceArea),
        Box::new(calculators::waist_to_height_ratio::WaistToHeightRatio),
        Box::new(calculators::waist_to_hip_ratio::WaistToHipRatio),
        Box::new(calculators::relative_fat_mass::RelativeFatMass),
        Box::new(calculators::max_heart_rate::MaxHeartRate),
        Box::new(calculators::one_rep_max::OneRepMax),
        Box::new(calculators::wilks::Wilks),
        Box::new(calculators::fib4::Fib4),
        Box::new(calculators::cha2ds2vasc::Cha2ds2Vasc),
        Box::new(calculators::cha2ds2_va::Cha2ds2Va),
        Box::new(calculators::ehra::Ehra),
        Box::new(calculators::ascvd::Ascvd),
        Box::new(calculators::familial_hypercholesterolaemia::FamilialHypercholesterolaemia),
        Box::new(calculators::ldl_cholesterol::LdlCholesterol),
        Box::new(calculators::findrisc::Findrisc),
        Box::new(calculators::homa_ir::HomaIr),
        Box::new(calculators::auditc::AuditC),
        Box::new(calculators::audit::Audit),
        Box::new(calculators::ciwa_ar::CiwaAr),
        Box::new(calculators::cows::Cows),
        Box::new(calculators::epds::Epds),
        Box::new(calculators::ipss::Ipss),
        Box::new(calculators::isth_overt_dic::IsthOvertDic),
        Box::new(calculators::amts::Amts),
        Box::new(calculators::barthel::Barthel),
        Box::new(calculators::mrc_dyspnoea::MrcDyspnoea),
        Box::new(calculators::news2::News2),
        Box::new(calculators::curb65::Curb65),
        Box::new(calculators::wells_dvt::WellsDvt),
        Box::new(calculators::wells_pe::WellsPe),
        Box::new(calculators::perc::Perc),
        Box::new(calculators::hasbled::HasBled),
        Box::new(calculators::abcd2::Abcd2),
        Box::new(calculators::qsofa::Qsofa),
        Box::new(calculators::fourat::FourAt),
        Box::new(calculators::four_ts::FourTs),
        Box::new(calculators::das28::Das28),
        Box::new(calculators::basdai::Basdai),
        Box::new(calculators::uacr::Uacr),
        Box::new(calculators::sofa::Sofa),
        Box::new(calculators::apache2::Apache2),
        Box::new(calculators::charlson::Charlson),
        Box::new(calculators::heart::Heart),
        Box::new(calculators::nyha::Nyha),
        Box::new(calculators::lrinec::Lrinec),
        Box::new(calculators::glasgow_blatchford::GlasgowBlatchford),
        Box::new(calculators::hinchey::Hinchey),
        Box::new(calculators::timi::Timi),
        Box::new(calculators::child_pugh::ChildPugh),
        Box::new(calculators::meld::Meld),
        Box::new(calculators::meld_3::Meld3),
        Box::new(calculators::padua::Padua),
        Box::new(calculators::caprini::Caprini),
        Box::new(calculators::asa_physical_status::AsaPhysicalStatus),
        Box::new(calculators::ukeld::Ukeld),
        Box::new(calculators::nhfs::Nhfs),
        Box::new(calculators::bode::Bode),
        Box::new(calculators::abpi::Abpi),
        Box::new(calculators::waterlow::Waterlow),
        Box::new(calculators::braden::Braden),
        Box::new(calculators::gcs::Gcs),
        Box::new(calculators::gleason::Gleason),
        Box::new(calculators::psa_density::PsaDensity),
        Box::new(calculators::npi::Npi),
        Box::new(calculators::binet::Binet),
        Box::new(calculators::khorana::Khorana),
        Box::new(calculators::chalice::Chalice),
        Box::new(calculators::ckd_risk::CkdRisk),
        Box::new(calculators::grace::Grace),
        Box::new(calculators::euroscore2::EuroScore2),
        Box::new(calculators::rcri::Rcri),
        Box::new(calculators::qrisk3::Qrisk3),
        Box::new(calculators::qfracture::Qfracture),
    ];
    // Proprietary / licence-locked tools: registered so a clinician learns why
    // they are absent and where to turn, rather than finding silence.
    for p in proprietary::PROPRIETARY {
        list.push(Box::new(*p));
    }
    list
}

/// Look up a single calculator by its machine name (e.g. `"feverpain"`).
pub fn get(name: &str) -> Option<Box<dyn Calculator>> {
    all().into_iter().find(|c| c.name() == name)
}

#[cfg(test)]
mod registry_tests {
    use super::*;

    /// Policy: every calculator must declare a non-empty distribution licence
    /// with a reverifiable source URL. This guards the requirement at CI time,
    /// so a new calculator cannot ship without recording the basis on which we
    /// distribute it.
    #[test]
    fn every_calculator_records_its_license() {
        for calc in all() {
            let lic = calc.license();
            assert!(
                !lic.license.trim().is_empty(),
                "{}: license must not be empty",
                calc.name()
            );
            assert!(
                lic.source_url.starts_with("http"),
                "{}: license source_url must be a URL, got {:?}",
                calc.name(),
                lic.source_url
            );
        }
    }

    /// Policy: every calculator must declare at least one tag, so it is
    /// discoverable via `clincalc list --tag <t>` and groupable in the docs.
    /// The central [`tags::TAGS`](crate::tags::TAGS) table is the canonical
    /// place to add new entries.
    #[test]
    fn every_calculator_has_at_least_one_tag() {
        for calc in all() {
            assert!(
                !calc.tags().is_empty(),
                "{}: tags() must return at least one tag - add an entry to clincalc::tags::TAGS",
                calc.name()
            );
        }
    }

    /// Policy: a schema that advertises a closed object must reject unknown
    /// fields at the typed deserialization boundary used by every surface.
    #[test]
    fn every_closed_schema_rejects_unknown_fields() {
        const SENTINEL: &str = "__unexpected_clincalc_field";
        let input = serde_json::json!({ (SENTINEL): null });

        for calc in all() {
            if calc.input_schema()["additionalProperties"] != false {
                continue;
            }

            let error = calc.calculate(&input).unwrap_err();
            assert!(
                matches!(&error, CalcError::InvalidInput(message) if message.contains(SENTINEL)),
                "{}: closed schema did not reject {SENTINEL}: {error}",
                calc.name()
            );
        }
    }

    /// Translation availability is a stable, duplicate-free subset of the
    /// bundles compiled into the crate, and always includes source English.
    #[test]
    fn every_calculator_has_valid_locale_coverage() {
        for calc in all() {
            let locales = calc.supported_locales();
            assert!(
                locales.contains(&SupportedLocale::En),
                "{}: supported locales must include source English",
                calc.name()
            );
            for (index, locale) in locales.iter().enumerate() {
                assert!(
                    COMPILED_LOCALES.contains(locale),
                    "{}: locale {locale} is not compiled into clincalc",
                    calc.name()
                );
                assert!(
                    !locales[..index].contains(locale),
                    "{}: locale {locale} is listed more than once",
                    calc.name()
                );
            }
        }
    }

    /// Locale-aware companion methods must preserve the complete English
    /// contract until a calculator opts into a reviewed translation bundle.
    #[test]
    fn locale_defaults_preserve_the_english_contract() {
        let calc = get("feverpain").unwrap();
        let input = serde_json::json!({
            "fever": true,
            "purulence": true,
            "attend_rapidly": true,
            "inflamed_tonsils": false,
            "absence_of_cough": false
        });

        assert_eq!(calc.supported_locales(), ENGLISH_ONLY);
        assert_eq!(calc.title_for(SupportedLocale::En), calc.title());
        assert_eq!(
            calc.description_for(SupportedLocale::En),
            calc.description()
        );
        assert_eq!(
            calc.input_schema_for(SupportedLocale::En),
            calc.input_schema()
        );
        assert_eq!(
            calc.input_template_for(SupportedLocale::En),
            calc.input_template()
        );
        let english = calc.calculate_for(&input, SupportedLocale::En).unwrap();
        assert_eq!(english.working["content_locale"], "en");
        let mut english_without_locale = english;
        english_without_locale.working.remove("content_locale");
        assert_eq!(english_without_locale, calc.calculate(&input).unwrap());

        // Passing an undeclared bundle to a default implementation falls back
        // the whole representation to English rather than mixing languages.
        assert_eq!(calc.title_for(SupportedLocale::Es), calc.title());
        assert_eq!(
            calc.input_template_for(SupportedLocale::Es),
            calc.input_template()
        );
        let spanish_fallback = calc.calculate_for(&input, SupportedLocale::Es).unwrap();
        assert_eq!(spanish_fallback.working["content_locale"], "en");
        let mut fallback_without_locale = spanish_fallback;
        fallback_without_locale.working.remove("content_locale");
        assert_eq!(fallback_without_locale, calc.calculate(&input).unwrap());
    }
}
