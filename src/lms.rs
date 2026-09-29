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
//! Reference data, interpolation, and coverage policy belong to each centile
//! reference (ENG-010.2 onwards), not to this module.

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
}
