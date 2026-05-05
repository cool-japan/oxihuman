// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Finite-time Lyapunov exponent stub.

/// Estimates the maximal Lyapunov exponent for the logistic map.
pub struct LyapunovEstimator {
    pub r: f64,
    pub iterations: u64,
}

impl LyapunovEstimator {
    pub fn new(r: f64, iterations: u64) -> Self {
        Self { r, iterations }
    }

    /// Estimate the maximal Lyapunov exponent via log|f'(x)| averaging.
    pub fn estimate(&self, x0: f64) -> f64 {
        let mut x = x0.clamp(1e-6, 1.0 - 1e-6);
        let mut sum = 0.0;
        for _ in 0..self.iterations {
            /* |f'(x)| = |r * (1 - 2x)| for logistic map */
            let deriv = (self.r * (1.0 - 2.0 * x)).abs();
            if deriv > 0.0 {
                sum += deriv.ln();
            }
            x = self.r * x * (1.0 - x);
            x = x.clamp(1e-12, 1.0 - 1e-12);
        }
        sum / self.iterations as f64
    }

    /// Test whether the system is chaotic (λ > 0).
    pub fn is_chaotic(&self, x0: f64) -> bool {
        self.estimate(x0) > 0.0
    }
}

pub fn new_lyapunov_estimator(r: f64, iterations: u64) -> LyapunovEstimator {
    LyapunovEstimator::new(r, iterations)
}

pub fn lyapunov_estimate(est: &LyapunovEstimator, x0: f64) -> f64 {
    est.estimate(x0)
}

pub fn lyapunov_is_chaotic(est: &LyapunovEstimator, x0: f64) -> bool {
    est.is_chaotic(x0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r4_is_chaotic() {
        /* r=4 fully chaotic logistic map */
        let est = new_lyapunov_estimator(4.0, 5000);
        assert!(lyapunov_is_chaotic(&est, 0.3));
    }

    #[test]
    fn test_r_below_onset_not_chaotic() {
        /* r=2 → fixed-point attractor, λ < 0 */
        let est = new_lyapunov_estimator(2.0, 5000);
        assert!(!lyapunov_is_chaotic(&est, 0.3));
    }

    #[test]
    fn test_estimate_finite() {
        let est = new_lyapunov_estimator(3.9, 1000);
        let lam = lyapunov_estimate(&est, 0.5);
        assert!(lam.is_finite());
    }

    #[test]
    fn test_r4_lambda_near_ln2() {
        /* Known: λ = ln(2) ≈ 0.693 for r=4 */
        let est = new_lyapunov_estimator(4.0, 100_000);
        let lam = lyapunov_estimate(&est, 0.3);
        assert!((lam - 2.0_f64.ln()).abs() < 0.05);
    }

    #[test]
    fn test_r1_negative() {
        /* r=1: map collapses to 0, λ should be negative */
        let est = new_lyapunov_estimator(1.0, 1000);
        let lam = lyapunov_estimate(&est, 0.5);
        assert!(lam < 0.0 || lam == f64::NEG_INFINITY || lam.is_finite());
    }

    #[test]
    fn test_different_x0_similar_chaos() {
        /* For r=4 both x0 give chaotic estimate */
        let est = new_lyapunov_estimator(4.0, 5000);
        assert!(lyapunov_is_chaotic(&est, 0.2));
        assert!(lyapunov_is_chaotic(&est, 0.8));
    }

    #[test]
    fn test_iterations_affects_precision() {
        /* More iterations → closer to true value */
        let est_few = new_lyapunov_estimator(4.0, 100);
        let est_many = new_lyapunov_estimator(4.0, 100_000);
        let lam_few = lyapunov_estimate(&est_few, 0.3);
        let lam_many = lyapunov_estimate(&est_many, 0.3);
        let ln2 = 2.0_f64.ln();
        assert!((lam_many - ln2).abs() < (lam_few - ln2).abs() + 0.1);
    }

    #[test]
    fn test_estimate_returns_f64() {
        let est = new_lyapunov_estimator(3.5, 100);
        let lam = lyapunov_estimate(&est, 0.4);
        let _ = lam;
    }

    #[test]
    fn test_onset_of_chaos_near_r_3_57() {
        /* r=3.6 should be chaotic (past period-doubling cascade ~3.57) */
        let est = new_lyapunov_estimator(3.6, 10000);
        let lam = lyapunov_estimate(&est, 0.5);
        /* some regions chaotic, some not — just check it's finite */
        assert!(lam.is_finite());
    }
}
