// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Gestational age and estimated due date (EDD) from the last menstrual
//! period (LMP), by Naegele's rule with an optional cycle-length adjustment.
//!
//! Like every calculator in this crate, this is a pure function: it takes an
//! explicit `reference_date` rather than reading the system clock, so the
//! caller supplies "today" (or any date of interest) and the result is
//! deterministic. Date arithmetic is done with a compact proleptic-Gregorian
//! day-count algorithm (Howard Hinnant's `days_from_civil`/`civil_from_days`,
//! public domain) rather than adding a date/time dependency to this
//! serde-only leaf crate.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "gestational_age";
pub const REFERENCE: &str = "Naegele's rule: Naegele FC. Lehrbuch der Geburtshilfe. 1812 (public domain). Current clinical confirmation and cycle-length adjustment per American College of Obstetricians and Gynecologists. Committee Opinion No. 700: Methods for Estimating the Due Date. Obstet Gynecol. 2017;129(5):e150-e154.";
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from standard obstetric practice",
    "https://doi.org/10.1097/AOG.0000000000002046",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CivilDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GestationalAgeInput {
    /// First day of the last menstrual period.
    pub last_menstrual_period: CivilDate,
    /// The date gestational age is calculated at (the caller's "today").
    pub reference_date: CivilDate,
    /// Optional usual menstrual cycle length in days, for the ACOG-endorsed
    /// adjustment to Naegele's rule (which assumes a 28-day cycle). Omit if
    /// unknown.
    #[serde(default)]
    pub cycle_length_days: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GestationalAgeOutcome {
    pub estimated_due_date: CivilDate,
    pub gestational_age_weeks: u32,
    pub gestational_age_days: u32,
    pub trimester: u8,
}

/// Days since 1970-01-01 for a proleptic-Gregorian civil date. Howard
/// Hinnant's `days_from_civil` algorithm (public domain); valid for any
/// representable year.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Inverse of [`days_from_civil`]: the proleptic-Gregorian civil date for a
/// day count since 1970-01-01.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Validate a `CivilDate` by round-tripping it through the day-count
/// conversion: an invalid date (e.g. 31 February) will not round-trip.
fn validate_date(date: CivilDate, name: &str) -> Result<i64, CalcError> {
    if !(1900..=2200).contains(&date.year) {
        return Err(CalcError::InvalidInput(format!(
            "{name}.year must be between 1900 and 2200"
        )));
    }
    if !(1..=12).contains(&date.month) {
        return Err(CalcError::InvalidInput(format!(
            "{name}.month must be between 1 and 12"
        )));
    }
    if !(1..=31).contains(&date.day) {
        return Err(CalcError::InvalidInput(format!(
            "{name}.day must be between 1 and 31"
        )));
    }
    let days = days_from_civil(date.year as i64, date.month as i64, date.day as i64);
    let (y, m, d) = civil_from_days(days);
    if y != date.year as i64 || m != date.month as u32 || d != date.day as u32 {
        return Err(CalcError::InvalidInput(format!(
            "{name} is not a valid calendar date"
        )));
    }
    Ok(days)
}

pub fn compute(input: &GestationalAgeInput) -> Result<GestationalAgeOutcome, CalcError> {
    let lmp_days = validate_date(input.last_menstrual_period, "last_menstrual_period")?;
    let reference_days = validate_date(input.reference_date, "reference_date")?;

    if let Some(cycle) = input.cycle_length_days
        && !(21..=45).contains(&cycle)
    {
        return Err(CalcError::InvalidInput(
            "cycle_length_days must be between 21 and 45".to_string(),
        ));
    }

    let gestation_days_at_reference = reference_days - lmp_days;
    if !(0..=320).contains(&gestation_days_at_reference) {
        return Err(CalcError::InvalidInput(
            "reference_date must be on or after last_menstrual_period, and no more than 320 days after it".to_string(),
        ));
    }

    let cycle_adjustment_days = input.cycle_length_days.map(|c| c as i64 - 28).unwrap_or(0);
    let edd_days = lmp_days + 280 + cycle_adjustment_days;
    let (edd_y, edd_m, edd_d) = civil_from_days(edd_days);

    let gestational_age_weeks = (gestation_days_at_reference / 7) as u32;
    let gestational_age_days = (gestation_days_at_reference % 7) as u32;
    let trimester = if gestational_age_weeks < 14 {
        1
    } else if gestational_age_weeks < 28 {
        2
    } else {
        3
    };

    Ok(GestationalAgeOutcome {
        estimated_due_date: CivilDate {
            year: edd_y as i32,
            month: edd_m as u8,
            day: edd_d as u8,
        },
        gestational_age_weeks,
        gestational_age_days,
        trimester,
    })
}

pub fn build_response(input: &GestationalAgeInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "estimated_due_date".into(),
        json!(format!(
            "{:04}-{:02}-{:02}",
            o.estimated_due_date.year, o.estimated_due_date.month, o.estimated_due_date.day
        )),
    );
    working.insert("trimester".into(), json!(o.trimester));
    working.insert(
        "cycle_adjusted".into(),
        json!(input.cycle_length_days.is_some()),
    );

    let interpretation = format!(
        "Gestational age {} weeks {} days (trimester {}); estimated due date {:04}-{:02}-{:02}{}. Naegele's rule assumes a regular 28-day cycle with ovulation on day 14; it is a population estimate and is routinely superseded by first-trimester ultrasound dating when available.",
        o.gestational_age_weeks,
        o.gestational_age_days,
        o.trimester,
        o.estimated_due_date.year,
        o.estimated_due_date.month,
        o.estimated_due_date.day,
        if input.cycle_length_days.is_some() {
            " (adjusted for the supplied cycle length)"
        } else {
            ""
        }
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(format!(
            "{}w{}d",
            o.gestational_age_weeks, o.gestational_age_days
        )),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

pub struct GestationalAge;
impl Calculator for GestationalAge {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Gestational Age / Estimated Due Date"
    }
    fn description(&self) -> &'static str {
        "Gestational age and estimated due date from the last menstrual period, by Naegele's rule with an optional cycle-length adjustment."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "GestationalAgeInput", "type": "object", "additionalProperties": false,
            "required": ["last_menstrual_period", "reference_date"],
            "properties": {
                "last_menstrual_period": {
                    "type": "object", "additionalProperties": false, "required": ["year", "month", "day"],
                    "properties": {
                        "year": { "type": "integer", "minimum": 1900, "maximum": 2200 },
                        "month": { "type": "integer", "minimum": 1, "maximum": 12 },
                        "day": { "type": "integer", "minimum": 1, "maximum": 31 }
                    },
                    "description": "First day of the last menstrual period"
                },
                "reference_date": {
                    "type": "object", "additionalProperties": false, "required": ["year", "month", "day"],
                    "properties": {
                        "year": { "type": "integer", "minimum": 1900, "maximum": 2200 },
                        "month": { "type": "integer", "minimum": 1, "maximum": 12 },
                        "day": { "type": "integer", "minimum": 1, "maximum": 31 }
                    },
                    "description": "The date to calculate gestational age at (the caller's \"today\"); this calculator does not read the system clock"
                },
                "cycle_length_days": { "type": "integer", "minimum": 21, "maximum": 45, "description": "Optional usual menstrual cycle length, for the ACOG-endorsed adjustment to Naegele's rule" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: GestationalAgeInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u8, day: u8) -> CivilDate {
        CivilDate { year, month, day }
    }

    #[test]
    fn day_count_round_trips() {
        for (y, m, d) in [
            (2024, 1, 1),
            (2024, 2, 29),
            (2000, 2, 29),
            (1900, 3, 1),
            (2026, 12, 31),
        ] {
            let days = days_from_civil(y, m, d);
            assert_eq!(civil_from_days(days), (y, m as u32, d as u32));
        }
    }

    #[test]
    fn naegele_adds_280_days() {
        // LMP 1 Jan 2026 -> EDD 8 Oct 2026 (280 days later).
        let out = compute(&GestationalAgeInput {
            last_menstrual_period: date(2026, 1, 1),
            reference_date: date(2026, 1, 1),
            cycle_length_days: None,
        })
        .unwrap();
        assert_eq!(out.estimated_due_date, date(2026, 10, 8));
        assert_eq!(out.gestational_age_weeks, 0);
        assert_eq!(out.gestational_age_days, 0);
    }

    #[test]
    fn gestational_age_at_later_reference_date() {
        // LMP 1 Jan 2026, reference 15 Mar 2026 = 73 days later = 10w3d.
        let out = compute(&GestationalAgeInput {
            last_menstrual_period: date(2026, 1, 1),
            reference_date: date(2026, 3, 15),
            cycle_length_days: None,
        })
        .unwrap();
        assert_eq!(out.gestational_age_weeks, 10);
        assert_eq!(out.gestational_age_days, 3);
        assert_eq!(out.trimester, 1);
    }

    #[test]
    fn cycle_length_adjusts_edd() {
        // A 35-day cycle shifts EDD 7 days later than the default 28-day assumption.
        let default_edd = compute(&GestationalAgeInput {
            last_menstrual_period: date(2026, 1, 1),
            reference_date: date(2026, 1, 1),
            cycle_length_days: None,
        })
        .unwrap()
        .estimated_due_date;
        let adjusted_edd = compute(&GestationalAgeInput {
            last_menstrual_period: date(2026, 1, 1),
            reference_date: date(2026, 1, 1),
            cycle_length_days: Some(35),
        })
        .unwrap()
        .estimated_due_date;
        let delta = days_from_civil(
            adjusted_edd.year as i64,
            adjusted_edd.month as i64,
            adjusted_edd.day as i64,
        ) - days_from_civil(
            default_edd.year as i64,
            default_edd.month as i64,
            default_edd.day as i64,
        );
        assert_eq!(delta, 7);
    }

    #[test]
    fn rejects_invalid_calendar_date() {
        assert!(
            compute(&GestationalAgeInput {
                last_menstrual_period: date(2026, 2, 30),
                reference_date: date(2026, 2, 30),
                cycle_length_days: None,
            })
            .is_err()
        );
    }

    #[test]
    fn rejects_reference_before_lmp() {
        assert!(
            compute(&GestationalAgeInput {
                last_menstrual_period: date(2026, 3, 1),
                reference_date: date(2026, 1, 1),
                cycle_length_days: None,
            })
            .is_err()
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "last_menstrual_period": {"year": 2026, "month": 1, "day": 1},
            "reference_date": {"year": 2026, "month": 3, "day": 15}
        });
        let typed = GestationalAgeInput {
            last_menstrual_period: date(2026, 1, 1),
            reference_date: date(2026, 3, 15),
            cycle_length_days: None,
        };
        let dynamic = GestationalAge.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            GestationalAge
                .calculate(&json!({"last_menstrual_period": {"year": 2026, "month": 1, "day": 1}}))
                .is_err()
        );
    }
}
