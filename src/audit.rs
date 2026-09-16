// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Reference-link auditing for calculator licences (roadmap item ENG-008).
//!
//! [`run`] HEAD-requests every distinct licence `source_url` in the registry,
//! reports non-2xx responses and redirects, and separately flags every
//! calculator whose `last_verified` date is missing or older than a
//! configurable threshold. This is a maintenance command invoked explicitly
//! via `clincalc audit-references`; it never runs as part of scoring.

use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ureq::Agent;

use crate::Calculator;

/// A calculator's licence evidence has never been reverified, or was
/// reverified longer ago than the requested threshold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleLicense {
    pub calculator: &'static str,
    pub source_url: &'static str,
    pub last_verified: Option<&'static str>,
}

/// The outcome of HEAD-requesting one licence `source_url`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlStatus {
    /// A 2xx response.
    Ok(u16),
    /// A 3xx response, with the `Location` header when the server sent one.
    Redirect {
        status: u16,
        location: Option<String>,
    },
    /// A 4xx or 5xx response.
    HttpError(u16),
    /// The request could not be completed at all (DNS, TLS, timeout, ...).
    RequestFailed(String),
}

/// One distinct licence `source_url` and every calculator that cites it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlCheck {
    pub source_url: &'static str,
    pub calculators: Vec<&'static str>,
    pub status: UrlStatus,
}

/// The result of auditing the whole registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditReport {
    pub url_checks: Vec<UrlCheck>,
    pub stale_licenses: Vec<StaleLicense>,
}

impl AuditReport {
    /// Any `source_url` that did not resolve to a 2xx response.
    pub fn has_url_problems(&self) -> bool {
        self.url_checks
            .iter()
            .any(|check| !matches!(check.status, UrlStatus::Ok(_)))
    }
}

/// HEAD-requests every distinct licence `source_url` in [`crate::all`] and
/// flags licences whose evidence has never been reverified, or was reverified
/// more than `max_age_days` ago. Calculators that cite the same `source_url`
/// (for example a shared guideline) are checked once.
pub fn run(max_age_days: u32, timeout: Duration) -> AuditReport {
    let calculators = crate::all();
    let cutoff = cutoff_date(max_age_days, SystemTime::now());
    let (by_url, stale_licenses) = group_licenses(&calculators, &cutoff);

    let agent = build_agent(timeout);
    let url_checks = by_url
        .into_iter()
        .map(|(source_url, calculators)| UrlCheck {
            status: check_url(&agent, source_url),
            source_url,
            calculators,
        })
        .collect();

    AuditReport {
        url_checks,
        stale_licenses,
    }
}

/// Groups every calculator's licence by `source_url` and separately collects
/// every calculator whose licence is stale as of `cutoff`. Pure and
/// network-free, so it is unit-testable against the real registry.
fn group_licenses(
    calculators: &[Box<dyn Calculator>],
    cutoff: &str,
) -> (BTreeMap<&'static str, Vec<&'static str>>, Vec<StaleLicense>) {
    let mut by_url: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();
    let mut stale_licenses = Vec::new();
    for calculator in calculators {
        let license = calculator.license();
        by_url
            .entry(license.source_url)
            .or_default()
            .push(calculator.name());
        if is_stale(license.last_verified, cutoff) {
            stale_licenses.push(StaleLicense {
                calculator: calculator.name(),
                source_url: license.source_url,
                last_verified: license.last_verified,
            });
        }
    }
    (by_url, stale_licenses)
}

fn build_agent(timeout: Duration) -> Agent {
    let config = Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .max_redirects_will_error(false)
        .timeout_global(Some(timeout))
        .build();
    Agent::new_with_config(config)
}

fn check_url(agent: &Agent, url: &str) -> UrlStatus {
    match agent.head(url).call() {
        Ok(response) => {
            let status = response.status().as_u16();
            if (300..400).contains(&status) {
                let location = response
                    .headers()
                    .get("location")
                    .and_then(|value| value.to_str().ok())
                    .map(str::to_string);
                UrlStatus::Redirect { status, location }
            } else if status >= 400 {
                UrlStatus::HttpError(status)
            } else {
                UrlStatus::Ok(status)
            }
        }
        Err(err) => UrlStatus::RequestFailed(err.to_string()),
    }
}

/// A licence is stale when it has never been reverified, or was reverified
/// before `cutoff` (an ISO `YYYY-MM-DD` date, compared lexicographically -
/// valid because zero-padded ISO dates sort the same way lexically and
/// chronologically).
fn is_stale(last_verified: Option<&str>, cutoff: &str) -> bool {
    match last_verified {
        None => true,
        Some(date) => date < cutoff,
    }
}

/// The `YYYY-MM-DD` date `max_age_days` before `now`.
fn cutoff_date(max_age_days: u32, now: SystemTime) -> String {
    let cutoff_days = days_since_epoch(now) - i64::from(max_age_days);
    let (year, month, day) = civil_from_days(cutoff_days);
    format!("{year:04}-{month:02}-{day:02}")
}

fn days_since_epoch(now: SystemTime) -> i64 {
    let elapsed = now.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
    (elapsed.as_secs() / 86_400) as i64
}

/// Converts a day count since 1970-01-01 to a proleptic Gregorian
/// `(year, month, day)`, using Howard Hinnant's public-domain
/// `civil_from_days` algorithm:
/// <http://howardhinnant.github.io/date_algorithms.html#civil_from_days>.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097); // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let year = if month <= 2 { y + 1 } else { y };
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_from_days_matches_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
        assert_eq!(civil_from_days(11_017), (2000, 3, 1));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29)); // leap day
    }

    #[test]
    fn cutoff_date_subtracts_whole_days_across_a_leap_year() {
        let now = UNIX_EPOCH + Duration::from_secs(11_017 * 86_400); // 2000-03-01
        assert_eq!(cutoff_date(1, now), "2000-02-29");
        assert_eq!(cutoff_date(0, now), "2000-03-01");
        assert_eq!(cutoff_date(366, now), "1999-03-01");
    }

    #[test]
    fn is_stale_treats_missing_verification_as_stale() {
        assert!(is_stale(None, "2026-01-01"));
    }

    #[test]
    fn is_stale_compares_iso_dates_lexicographically() {
        assert!(is_stale(Some("2024-12-31"), "2025-01-01"));
        assert!(!is_stale(Some("2025-01-01"), "2025-01-01"));
        assert!(!is_stale(Some("2025-06-01"), "2025-01-01"));
    }

    #[test]
    fn group_licenses_collapses_a_shared_source_url() {
        let calculators = crate::all();
        let (by_url, _stale) = group_licenses(&calculators, "2000-01-01");
        assert!(
            by_url.values().any(|names| names.len() > 1),
            "expected at least one source_url shared by multiple calculators"
        );
    }

    #[test]
    fn group_licenses_flags_every_calculator_before_any_reverification_pass() {
        // ENG-008.1 added `last_verified`, but no calculator has recorded a
        // reverification yet, so a far-future cutoff should flag them all.
        let calculators = crate::all();
        let (_by_url, stale) = group_licenses(&calculators, "9999-12-31");
        assert_eq!(stale.len(), calculators.len());
    }

    #[test]
    fn has_url_problems_is_false_only_when_every_check_is_ok() {
        let ok_report = AuditReport {
            url_checks: vec![UrlCheck {
                source_url: "https://example.org",
                calculators: vec!["example"],
                status: UrlStatus::Ok(200),
            }],
            stale_licenses: Vec::new(),
        };
        assert!(!ok_report.has_url_problems());

        let broken_report = AuditReport {
            url_checks: vec![UrlCheck {
                source_url: "https://example.org",
                calculators: vec!["example"],
                status: UrlStatus::HttpError(404),
            }],
            stale_licenses: Vec::new(),
        };
        assert!(broken_report.has_url_problems());
    }
}
