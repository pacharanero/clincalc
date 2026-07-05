// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Calculator tags - lightweight taxonomy for discovery and filtering.
//!
//! Tags categorise each calculator by **specialty** (where it is used) and
//! **status** (what kind of tool it is). They are surfaced everywhere a
//! calculator is listed - the `calc list --tag <t>` filter, `calc list --tags`,
//! the published docs catalogue, and any host that enumerates the registry.
//!
//! The taxonomy is centralised here, not split across the calculators, so the
//! whole vocabulary is reviewable in one file. The default `Calculator::tags`
//! impl looks each calculator up by its machine name in [`TAGS`]; calculators
//! that need to override (e.g. a tag added post-hoc by a host) can still
//! implement `tags()` directly.
//!
//! ## Tag vocabulary
//!
//! **Specialty** (one or more per calculator):
//! `primary-care`, `emergency`, `acute-medicine`, `intensive-care`,
//! `cardiology`, `nephrology`, `hepatology`, `respiratory`, `neurology`,
//! `mental-health`, `endocrinology`, `rheumatology`, `urology`, `oncology`,
//! `geriatrics`, `perinatal`, `paediatrics`, `infectious-diseases`,
//! `surgery`, `vascular`, `musculoskeletal`.
//!
//! **Status / kind** (zero or more):
//! `nhs-mandated`, `screening`, `severity`, `prognostic`, `risk`,
//! `proprietary` (the algorithm is licence-locked and unshippable),
//! `unavailable` (the calculator returns an explanation, not a score - always
//! paired with `proprietary` today).
//!
//! Tags are lowercase, hyphen-separated, ASCII. New tags are added here only
//! after at least two calculators want them - one-offs go in the description.

/// `(machine_name, tags)` for every calculator in the registry.
///
/// Order is not significant. The lookup is small enough (~50 entries) that a
/// linear scan is fine; if the registry grows past a few hundred this becomes
/// a `phf` map.
pub const TAGS: &[(&str, &[&str])] = &[
    // ---- Primary care / mental health / screening ----
    (
        "feverpain",
        &["primary-care", "infectious-diseases", "respiratory"],
    ),
    ("phq9", &["primary-care", "mental-health", "screening"]),
    ("gad7", &["primary-care", "mental-health", "screening"]),
    ("audit", &["primary-care", "mental-health", "screening"]),
    ("auditc", &["primary-care", "mental-health", "screening"]),
    (
        "epds",
        &["primary-care", "mental-health", "perinatal", "screening"],
    ),
    ("asrs", &["primary-care", "mental-health", "screening"]),
    (
        "amts",
        &["primary-care", "geriatrics", "neurology", "screening"],
    ),
    ("mrc_dyspnoea", &["primary-care", "respiratory"]),
    // ---- Cardiology ----
    ("cha2ds2vasc", &["cardiology", "risk"]),
    ("hasbled", &["cardiology", "risk"]),
    ("heart", &["cardiology", "emergency", "risk"]),
    ("grace", &["cardiology", "acute-medicine", "prognostic"]),
    ("timi", &["cardiology", "acute-medicine", "risk"]),
    ("qrisk3", &["primary-care", "cardiology", "risk"]),
    ("euroscore2", &["cardiology", "surgery", "prognostic"]),
    // ---- Vascular / thrombosis ----
    ("wells_dvt", &["emergency", "vascular"]),
    ("wells_pe", &["emergency", "respiratory", "vascular"]),
    ("padua", &["acute-medicine", "vascular", "risk"]),
    ("abpi", &["primary-care", "vascular"]),
    // ---- Stroke / neurology ----
    ("abcd2", &["neurology", "emergency", "risk"]),
    (
        "fourat",
        &["acute-medicine", "geriatrics", "neurology", "screening"],
    ),
    // ---- Acute illness / sepsis / ICU ----
    ("news2", &["acute-medicine", "nhs-mandated", "severity"]),
    ("qsofa", &["acute-medicine", "intensive-care", "screening"]),
    ("sofa", &["intensive-care", "severity"]),
    (
        "curb65",
        &[
            "acute-medicine",
            "respiratory",
            "infectious-diseases",
            "severity",
        ],
    ),
    // ---- Renal ----
    ("egfr", &["primary-care", "nephrology"]),
    ("uacr", &["primary-care", "nephrology"]),
    ("ckd_risk", &["primary-care", "nephrology", "risk"]),
    // ---- Hepatology ----
    ("fib4", &["primary-care", "hepatology", "screening"]),
    ("child_pugh", &["hepatology", "severity"]),
    ("meld", &["hepatology", "prognostic"]),
    ("ukeld", &["hepatology", "prognostic"]),
    // ---- Endocrinology / bone ----
    ("qfracture", &["primary-care", "endocrinology", "risk"]),
    // ---- Rheumatology ----
    ("das28", &["rheumatology", "severity"]),
    // ---- Urology / oncology ----
    ("ipss", &["urology", "severity"]),
    ("gleason", &["oncology", "urology"]),
    ("npi", &["oncology", "prognostic"]),
    // ---- Surgery / trauma / pressure injury ----
    (
        "nhfs",
        &["surgery", "geriatrics", "musculoskeletal", "prognostic"],
    ),
    ("waterlow", &["acute-medicine", "geriatrics", "screening"]),
    // ---- Respiratory / COPD ----
    ("bode", &["respiratory", "prognostic"]),
    // ---- Paediatrics ----
    ("chalice", &["paediatrics", "emergency"]),
    // ---- Proprietary / unavailable (the 10 stubs) ----
    (
        "frax",
        &[
            "endocrinology",
            "musculoskeletal",
            "risk",
            "proprietary",
            "unavailable",
        ],
    ),
    (
        "mmse",
        &[
            "geriatrics",
            "neurology",
            "mental-health",
            "screening",
            "proprietary",
            "unavailable",
        ],
    ),
    (
        "must",
        &["primary-care", "screening", "proprietary", "unavailable"],
    ),
    (
        "cat",
        &["respiratory", "severity", "proprietary", "unavailable"],
    ),
    (
        "acq",
        &["respiratory", "severity", "proprietary", "unavailable"],
    ),
    (
        "elf",
        &["hepatology", "screening", "proprietary", "unavailable"],
    ),
    (
        "cfs",
        &["geriatrics", "severity", "proprietary", "unavailable"],
    ),
    (
        "lanss",
        &["neurology", "screening", "proprietary", "unavailable"],
    ),
    (
        "ohs",
        &["surgery", "musculoskeletal", "proprietary", "unavailable"],
    ),
    (
        "oks",
        &["surgery", "musculoskeletal", "proprietary", "unavailable"],
    ),
];

/// Look up the tags for a calculator by its machine name. Returns `&[]` for an
/// unknown name (the registry test ensures every shipped calculator is here).
pub fn for_name(name: &str) -> &'static [&'static str] {
    for (n, ts) in TAGS {
        if *n == name {
            return ts;
        }
    }
    &[]
}

/// Every distinct tag in [`TAGS`], deduplicated and sorted. Useful for the
/// `calc list --tags` listing and for the docs catalogue.
pub fn all_tags() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = TAGS.iter().flat_map(|(_, ts)| ts.iter().copied()).collect();
    out.sort_unstable();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_empty_for_unknown() {
        assert!(for_name("not-a-real-calculator").is_empty());
    }

    #[test]
    fn for_name_returns_tags_for_known() {
        assert!(for_name("news2").contains(&"nhs-mandated"));
        assert!(for_name("frax").contains(&"proprietary"));
    }

    #[test]
    fn all_tags_is_sorted_and_deduped() {
        let tags = all_tags();
        let mut sorted = tags.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(tags, sorted);
        // Sanity: some known specialty tags appear
        assert!(tags.contains(&"cardiology"));
        assert!(tags.contains(&"proprietary"));
    }
}
