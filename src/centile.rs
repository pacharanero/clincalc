// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The `CentileReference` trait (ENG-010.3).
//!
//! Splits a centile calculator into data and engine. A reference supplies only
//! its LMS table for a selector (typically sex and parameter), its
//! interpolation strategy, and its coverage policy (age range, prematurity
//! correction, overlaps between sub-references). The provided methods do the
//! lookup, interpolation, and LMS maths via [`crate::lms`].
//!
//! Pure and allocation-free: no I/O, clocks, or global state, and no
//! dependencies beyond `core`, so the strict leaf rule is preserved. Reference
//! data and its redistribution terms are a separate gate (ENG-010.4).

use crate::lms::{
    Interpolation, Lms, LmsPoint, interpolate, measurement_to_sds, sds_to_centile,
    sds_to_measurement,
};

/// A centile reference dataset.
pub trait CentileReference {
    /// What picks a table within the reference, for example sex and
    /// parameter (height, weight, head circumference).
    type Selector;

    /// The LMS table for `selector`, sorted by strictly increasing age, or
    /// `None` if the reference has no such table.
    fn table(&self, selector: &Self::Selector) -> Option<&[LmsPoint]>;

    /// How this reference interpolates between rows. Cubic is the default;
    /// WHO references override this with linear.
    fn interpolation(&self) -> Interpolation {
        Interpolation::Cubic
    }

    /// Map a chronological age to the age used for table lookup, or `None`
    /// when the reference does not cover it. This is the hook for coverage
    /// limits, prematurity correction, and sub-reference overlaps; the engine
    /// never extrapolates or corrects on its own.
    fn lookup_age(&self, _selector: &Self::Selector, age: f64) -> Option<f64> {
        Some(age)
    }

    /// Interpolated L, M, and S for `selector` at `age`.
    fn lms(&self, selector: &Self::Selector, age: f64) -> Option<Lms> {
        let table = self.table(selector)?;
        let x = self.lookup_age(selector, age)?;
        interpolate(table, x, self.interpolation())
    }

    /// Standard deviation score of `measurement` at `age`.
    fn sds(&self, selector: &Self::Selector, age: f64, measurement: f64) -> Option<f64> {
        let p = self.lms(selector, age)?;
        measurement_to_sds(measurement, p.l, p.m, p.s)
    }

    /// Centile (0-100) of `measurement` at `age`.
    fn centile(&self, selector: &Self::Selector, age: f64, measurement: f64) -> Option<f64> {
        self.sds(selector, age, measurement).map(sds_to_centile)
    }

    /// The measurement at `age` that corresponds to `sds`.
    fn measurement_at_sds(&self, selector: &Self::Selector, age: f64, sds: f64) -> Option<f64> {
        let p = self.lms(selector, age)?;
        sds_to_measurement(sds, p.l, p.m, p.s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Synthetic table (not clinical data): L = 1, S = 0.1, M = 10 + 2x.
    const TABLE: [LmsPoint; 3] = [
        LmsPoint {
            x: 0.0,
            l: 1.0,
            m: 10.0,
            s: 0.1,
        },
        LmsPoint {
            x: 1.0,
            l: 1.0,
            m: 12.0,
            s: 0.1,
        },
        LmsPoint {
            x: 2.0,
            l: 1.0,
            m: 14.0,
            s: 0.1,
        },
    ];

    struct Toy {
        premature_weeks: f64,
    }

    impl CentileReference for Toy {
        type Selector = bool;

        fn table(&self, selector: &bool) -> Option<&[LmsPoint]> {
            selector.then_some(&TABLE[..])
        }

        fn interpolation(&self) -> Interpolation {
            Interpolation::Linear
        }

        fn lookup_age(&self, _: &bool, age: f64) -> Option<f64> {
            // Reference-specific prematurity correction, in years.
            let corrected = age - self.premature_weeks / 52.0;
            (corrected >= 0.0).then_some(corrected)
        }
    }

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} vs {b}");
    }

    #[test]
    fn median_is_zero_sds_and_fiftieth_centile() {
        let r = Toy {
            premature_weeks: 0.0,
        };
        close(r.sds(&true, 1.0, 12.0).unwrap(), 0.0);
        assert!((r.centile(&true, 1.5, 13.0).unwrap() - 50.0).abs() < 1e-4);
    }

    #[test]
    fn sds_and_measurement_round_trip() {
        let r = Toy {
            premature_weeks: 0.0,
        };
        let x = r.measurement_at_sds(&true, 0.5, 1.5).unwrap();
        close(r.sds(&true, 0.5, x).unwrap(), 1.5);
        // M(0.5) = 11, L = 1: x = 11 * (1 + 0.1 * 1.5)
        close(x, 11.0 * 1.15);
    }

    #[test]
    fn unknown_table_and_uncovered_age_are_none() {
        let r = Toy {
            premature_weeks: 0.0,
        };
        assert!(r.lms(&false, 1.0).is_none());
        assert!(r.lms(&true, 2.5).is_none());
        assert!(r.lms(&true, -0.1).is_none());
    }

    #[test]
    fn lookup_age_hook_applies_reference_policy() {
        let r = Toy {
            premature_weeks: 26.0,
        };
        // Chronological 1.5y looks up the table at 1.0y.
        close(r.lms(&true, 1.5).unwrap().m, 12.0);
        // Younger than the correction is not covered.
        assert!(r.lms(&true, 0.4).is_none());
    }
}
