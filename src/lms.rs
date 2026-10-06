// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! LMS (Box-Cox power, median, coefficient of variation) transforms for centile
//! references (ENG-010.1).
//!
//! Pure functions only: no I/O, clocks, or global state, and no dependency beyond
//! `core`, so the strict leaf rule is preserved. The normal CDF uses the
//! Abramowitz and Stegun 7.1.26 error-function approximation (absolute error
//! below 1.5e-7 on erf), which is ample for reporting centiles to one decimal
//! place.
//!
//! Interpolation of L, M, and S between tabulated ages (ENG-010.2) is here too.
//! Reference data and coverage policy belong to each centile reference
//! (ENG-010.3 onwards), not to this module.

/// Below this magnitude `L` is treated as zero (the logarithmic limit of the
/// Box-Cox transform), avoiding catastrophic cancellation in `(x/M)^L - 1`.
const L_ZERO_EPSILON: f64 = 1e-12;

/// Convert a measurement to a standard deviation score (z-score) using LMS
/// parameters: `z = ((x/M)^L - 1) / (L*S)`, or `ln(x/M) / S` when `L == 0`.
///
/// Returns `None` for non-positive or non-finite `x`, `m`, or `s`, or a
/// non-finite `l`.
pub fn measurement_to_sds(x: f64, l: f64, m: f64, s: f64) -> Option<f64> {
    if !valid_params(l, m, s) || !x.is_finite() || x <= 0.0 {
        return None;
    }
    let ratio = x / m;
    let z = if l.abs() < L_ZERO_EPSILON {
        ratio.ln() / s
    } else {
        (ratio.powf(l) - 1.0) / (l * s)
    };
    z.is_finite().then_some(z)
}

/// Convert a standard deviation score back to a measurement:
/// `x = M * (1 + L*S*z)^(1/L)`, or `M * exp(S*z)` when `L == 0`.
///
/// Returns `None` for invalid parameters, a non-finite `z`, or a `z` outside
/// the transform's domain (`1 + L*S*z <= 0`).
pub fn sds_to_measurement(z: f64, l: f64, m: f64, s: f64) -> Option<f64> {
    if !valid_params(l, m, s) || !z.is_finite() {
        return None;
    }
    let x = if l.abs() < L_ZERO_EPSILON {
        m * (s * z).exp()
    } else {
        let base = 1.0 + l * s * z;
        if base <= 0.0 {
            return None;
        }
        m * base.powf(1.0 / l)
    };
    x.is_finite().then_some(x)
}

/// Standard normal cumulative distribution function.
pub fn normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / core::f64::consts::SQRT_2))
}

/// Convert a z-score to a centile (0-100) on the standard normal distribution.
pub fn sds_to_centile(z: f64) -> f64 {
    100.0 * normal_cdf(z)
}

/// One row of an LMS reference table: the L, M, and S values at `x` (usually
/// decimal age in years, but any monotonic axis works).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LmsPoint {
    pub x: f64,
    pub l: f64,
    pub m: f64,
    pub s: f64,
}

/// Interpolated L, M, and S at a requested position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lms {
    pub l: f64,
    pub m: f64,
    pub s: f64,
}

/// How a reference interpolates between table rows. Each reference declares its
/// own strategy: cubic is the default for LMSGrowth-style tables, linear is
/// what the WHO references use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interpolation {
    Linear,
    Cubic,
}

/// Interpolate L, M, and S independently at `x` over a table sorted by strictly
/// increasing `x`.
///
/// `Cubic` is Lagrange interpolation through the four rows nearest `x` (two
/// either side, shifted inwards at the ends of the table), falling back to
/// linear when the table has fewer than four rows. An exact row match returns
/// that row unchanged.
///
/// Returns `None` if `x` is non-finite or outside the table's range, the table
/// is empty or not strictly increasing, or any value is non-finite. Callers
/// own coverage policy, so there is no extrapolation.
pub fn interpolate(table: &[LmsPoint], x: f64, strategy: Interpolation) -> Option<Lms> {
    if !x.is_finite() || table.is_empty() {
        return None;
    }
    let all_finite = table
        .iter()
        .all(|p| p.x.is_finite() && p.l.is_finite() && p.m.is_finite() && p.s.is_finite());
    if !all_finite || table.windows(2).any(|w| w[1].x <= w[0].x) {
        return None;
    }
    let first = table[0].x;
    let last = table[table.len() - 1].x;
    if x < first || x > last {
        return None;
    }
    // Index of the last row with p.x <= x.
    let lo = table.partition_point(|p| p.x <= x) - 1;
    if table[lo].x == x {
        let p = table[lo];
        return Some(Lms {
            l: p.l,
            m: p.m,
            s: p.s,
        });
    }
    // x is strictly inside (table[lo].x, table[lo + 1].x).
    let window: &[LmsPoint] = match strategy {
        Interpolation::Cubic if table.len() >= 4 => {
            let start = lo.saturating_sub(1).min(table.len() - 4);
            &table[start..start + 4]
        }
        _ => &table[lo..lo + 2],
    };
    Some(Lms {
        l: lagrange(window, x, |p| p.l),
        m: lagrange(window, x, |p| p.m),
        s: lagrange(window, x, |p| p.s),
    })
}

fn lagrange(points: &[LmsPoint], x: f64, value: impl Fn(&LmsPoint) -> f64) -> f64 {
    let mut total = 0.0;
    for (i, pi) in points.iter().enumerate() {
        let mut weight = 1.0;
        for (j, pj) in points.iter().enumerate() {
            if i != j {
                weight *= (x - pj.x) / (pi.x - pj.x);
            }
        }
        total += weight * value(pi);
    }
    total
}

fn valid_params(l: f64, m: f64, s: f64) -> bool {
    l.is_finite() && m.is_finite() && s.is_finite() && m > 0.0 && s > 0.0
}

/// Error function, Abramowitz and Stegun 7.1.26 (|error| < 1.5e-7).
fn erf(x: f64) -> f64 {
    const P: f64 = 0.327_591_1;
    const A: [f64; 5] = [
        0.254_829_592,
        -0.284_496_736,
        1.421_413_741,
        -1.453_152_027,
        1.061_405_429,
    ];
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + P * x);
    let poly = t * (A[0] + t * (A[1] + t * (A[2] + t * (A[3] + t * A[4]))));
    sign * (1.0 - poly * (-x * x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() <= tol, "{a} vs {b} (tol {tol})");
    }

    #[test]
    fn median_is_zero_sds() {
        for l in [-2.0, -0.5, 0.0, 0.7, 1.0, 2.0] {
            close(measurement_to_sds(50.0, l, 50.0, 0.1).unwrap(), 0.0, 1e-12);
        }
    }

    #[test]
    fn l_one_is_linear() {
        // z = (x/M - 1) / S = (12/10 - 1) / 0.1 = 2
        close(
            measurement_to_sds(12.0, 1.0, 10.0, 0.1).unwrap(),
            2.0,
            1e-12,
        );
    }

    #[test]
    fn l_zero_is_log_normal() {
        // z = ln(x/M) / S
        close(
            measurement_to_sds(20.0, 0.0, 10.0, 0.25).unwrap(),
            2.0_f64.ln() / 0.25,
            1e-12,
        );
    }

    #[test]
    fn small_l_approaches_log_limit() {
        let log = measurement_to_sds(13.0, 0.0, 10.0, 0.12).unwrap();
        let near = measurement_to_sds(13.0, 1e-6, 10.0, 0.12).unwrap();
        close(log, near, 1e-5);
    }

    #[test]
    fn round_trip() {
        for (l, m, s) in [(1.0, 10.0, 0.1), (0.0, 80.0, 0.05), (-1.5, 15.0, 0.12)] {
            for z in [-3.0, -1.0, 0.0, 0.5, 2.0] {
                let x = sds_to_measurement(z, l, m, s).unwrap();
                close(measurement_to_sds(x, l, m, s).unwrap(), z, 1e-9);
            }
        }
    }

    #[test]
    fn out_of_domain_and_invalid_inputs_are_none() {
        // 1 + L*S*z = 1 + 2*0.5*(-2) = -1 <= 0
        assert!(sds_to_measurement(-2.0, 2.0, 10.0, 0.5).is_none());
        assert!(measurement_to_sds(0.0, 1.0, 10.0, 0.1).is_none());
        assert!(measurement_to_sds(-1.0, 1.0, 10.0, 0.1).is_none());
        assert!(measurement_to_sds(5.0, 1.0, 0.0, 0.1).is_none());
        assert!(measurement_to_sds(5.0, 1.0, 10.0, 0.0).is_none());
        assert!(measurement_to_sds(f64::NAN, 1.0, 10.0, 0.1).is_none());
        assert!(sds_to_measurement(f64::INFINITY, 1.0, 10.0, 0.1).is_none());
    }

    #[test]
    fn normal_cdf_matches_tables() {
        close(normal_cdf(0.0), 0.5, 1e-7);
        close(normal_cdf(1.0), 0.841_344_746, 1e-7);
        close(normal_cdf(-1.0), 0.158_655_254, 1e-7);
        close(normal_cdf(1.959_964), 0.975, 1e-7);
        close(normal_cdf(-3.0), 0.001_349_898, 1e-7);
        close(normal_cdf(3.0), 0.998_650_102, 1e-7);
    }

    #[test]
    fn centile_is_percentage() {
        close(sds_to_centile(0.0), 50.0, 1e-5);
        close(sds_to_centile(-1.645), 5.0, 1e-2);
    }

    fn pt(x: f64, l: f64, m: f64, s: f64) -> LmsPoint {
        LmsPoint { x, l, m, s }
    }

    // Rows sampled from m = x^3 - 2x, l = x^2, s = 0.1 + 0.01x.
    fn cubic_table() -> Vec<LmsPoint> {
        [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]
            .iter()
            .map(|&x| pt(x, x * x, x * x * x - 2.0 * x, 0.1 + 0.01 * x))
            .collect()
    }

    #[test]
    fn exact_row_is_returned_unchanged() {
        let t = cubic_table();
        for strategy in [Interpolation::Linear, Interpolation::Cubic] {
            let got = interpolate(&t, 3.0, strategy).unwrap();
            assert_eq!(
                got,
                Lms {
                    l: 9.0,
                    m: 21.0,
                    s: 0.13
                }
            );
        }
    }

    #[test]
    fn linear_interpolates_midpoint() {
        let t = [pt(1.0, 1.0, 10.0, 0.1), pt(2.0, 0.0, 20.0, 0.2)];
        let got = interpolate(&t, 1.25, Interpolation::Linear).unwrap();
        close(got.l, 0.75, 1e-12);
        close(got.m, 12.5, 1e-12);
        close(got.s, 0.125, 1e-12);
    }

    #[test]
    fn cubic_reproduces_a_cubic_exactly() {
        let t = cubic_table();
        for x in [0.4, 1.5, 2.5, 3.75, 4.9] {
            let got = interpolate(&t, x, Interpolation::Cubic).unwrap();
            close(got.m, x * x * x - 2.0 * x, 1e-9);
            close(got.l, x * x, 1e-9);
            close(got.s, 0.1 + 0.01 * x, 1e-12);
        }
    }

    #[test]
    fn cubic_beats_linear_on_curved_data() {
        let t = cubic_table();
        let exact = 2.5_f64.powi(3) - 5.0;
        let cubic = interpolate(&t, 2.5, Interpolation::Cubic).unwrap().m;
        let linear = interpolate(&t, 2.5, Interpolation::Linear).unwrap().m;
        assert!((cubic - exact).abs() < (linear - exact).abs());
    }

    #[test]
    fn cubic_falls_back_to_linear_on_short_tables() {
        let t = [
            pt(0.0, 1.0, 10.0, 0.1),
            pt(1.0, 1.0, 20.0, 0.1),
            pt(2.0, 1.0, 50.0, 0.1),
        ];
        let cubic = interpolate(&t, 0.5, Interpolation::Cubic).unwrap();
        let linear = interpolate(&t, 0.5, Interpolation::Linear).unwrap();
        assert_eq!(cubic, linear);
    }

    #[test]
    fn out_of_range_and_bad_tables_are_none() {
        let t = cubic_table();
        assert!(interpolate(&t, -0.1, Interpolation::Cubic).is_none());
        assert!(interpolate(&t, 5.1, Interpolation::Linear).is_none());
        assert!(interpolate(&t, f64::NAN, Interpolation::Linear).is_none());
        assert!(interpolate(&[], 1.0, Interpolation::Linear).is_none());
        let unsorted = [pt(1.0, 1.0, 1.0, 0.1), pt(1.0, 1.0, 2.0, 0.1)];
        assert!(interpolate(&unsorted, 1.0, Interpolation::Linear).is_none());
        let nan = [pt(0.0, 1.0, f64::NAN, 0.1), pt(1.0, 1.0, 2.0, 0.1)];
        assert!(interpolate(&nan, 0.5, Interpolation::Linear).is_none());
    }
}
