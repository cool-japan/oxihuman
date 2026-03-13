// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! P² quantile estimator (Jain & Chlamtac algorithm).

/// P² algorithm marker positions and heights.
#[derive(Debug, Clone)]
pub struct P2Quantile {
    /// Target quantile (0..=1).
    pub p: f64,
    /// Marker positions (n values, not necessarily integer).
    n: [f64; 5],
    /// Desired marker positions.
    n_prime: [f64; 5],
    /// Increments for desired positions.
    dn: [f64; 5],
    /// Marker heights (observed values at positions).
    q: [f64; 5],
    count: usize,
}

impl P2Quantile {
    pub fn new(p: f64) -> Self {
        let p = p.clamp(0.0, 1.0);
        P2Quantile {
            p,
            n: [1.0, 2.0, 3.0, 4.0, 5.0],
            n_prime: [1.0, 1.0 + 2.0 * p, 1.0 + 4.0 * p, 3.0 + 2.0 * p, 5.0],
            dn: [0.0, p / 2.0, p, (1.0 + p) / 2.0, 1.0],
            q: [0.0; 5],
            count: 0,
        }
    }

    pub fn add(&mut self, x: f64) {
        self.count += 1;
        if self.count <= 5 {
            self.q[self.count - 1] = x;
            if self.count == 5 {
                self.q
                    .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            }
            return;
        }
        /* Find cell k where q[k] <= x < q[k+1] */
        let k = if x < self.q[0] {
            self.q[0] = x;
            0
        } else if x < self.q[1] {
            0
        } else if x < self.q[2] {
            1
        } else if x < self.q[3] {
            2
        } else if x <= self.q[4] {
            3
        } else {
            self.q[4] = x;
            3
        };
        for i in (k + 1)..5 {
            self.n[i] += 1.0;
        }
        for i in 0..5 {
            self.n_prime[i] += self.dn[i];
        }
        for i in 1..4 {
            let d = self.n_prime[i] - self.n[i];
            if (d >= 1.0 && self.n[i + 1] - self.n[i] > 1.0)
                || (d <= -1.0 && self.n[i - 1] - self.n[i] < -1.0)
            {
                let sign = if d > 0.0 { 1.0 } else { -1.0 };
                let q_new = self.parabolic(i, sign);
                if q_new > self.q[i - 1] && q_new < self.q[i + 1] {
                    self.q[i] = q_new;
                } else {
                    self.q[i] = self.linear(i, sign);
                }
                self.n[i] += sign;
            }
        }
    }

    fn parabolic(&self, i: usize, d: f64) -> f64 {
        let qi = self.q[i];
        let qi1 = if d > 0.0 {
            self.q[i + 1]
        } else {
            self.q[i - 1]
        };
        let qi_1 = if d > 0.0 {
            self.q[i - 1]
        } else {
            self.q[i + 1]
        };
        let ni = self.n[i];
        let ni1 = if d > 0.0 {
            self.n[i + 1]
        } else {
            self.n[i - 1]
        };
        let ni_1 = if d > 0.0 {
            self.n[i - 1]
        } else {
            self.n[i + 1]
        };
        qi + d / (ni1 - ni_1)
            * ((ni - ni_1 + d) * (qi1 - qi) / (ni1 - ni)
                + (ni1 - ni - d) * (qi - qi_1) / (ni - ni_1))
    }

    fn linear(&self, i: usize, d: f64) -> f64 {
        let j = if d > 0.0 { i + 1 } else { i - 1 };
        self.q[i] + d * (self.q[j] - self.q[i]) / (self.n[j] - self.n[i])
    }

    /// Return current estimate of the quantile.
    pub fn estimate(&self) -> Option<f64> {
        if self.count < 5 {
            return None;
        }
        Some(self.q[2])
    }

    pub fn sample_count(&self) -> usize {
        self.count
    }
}

pub fn quantile_batch(data: &[f64], p: f64) -> Option<f64> {
    if data.is_empty() {
        return None;
    }
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = p * (sorted.len() as f64 - 1.0);
    let lo = idx.floor() as usize;
    let hi = (lo + 1).min(sorted.len() - 1);
    let frac = idx - lo as f64;
    Some(sorted[lo] + frac * (sorted[hi] - sorted[lo]))
}

pub fn median_batch(data: &[f64]) -> Option<f64> {
    quantile_batch(data, 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p2_needs_five_samples() {
        let mut q = P2Quantile::new(0.5);
        for i in 0..4 {
            q.add(i as f64);
        }
        assert!(q.estimate().is_none() /* not enough samples yet */,);
    }

    #[test]
    fn test_p2_after_five() {
        let mut q = P2Quantile::new(0.5);
        for i in 0..5 {
            q.add(i as f64);
        }
        assert!(q.estimate().is_some(), /* 5 samples = estimate available */);
    }

    #[test]
    fn test_p2_median_uniform() {
        let mut q = P2Quantile::new(0.5);
        for i in 0..=100 {
            q.add(i as f64);
        }
        let est = q.estimate().expect("should succeed");
        assert!((est - 50.0).abs() < 5.0 /* median of 0..100 ≈ 50 */,);
    }

    #[test]
    fn test_quantile_batch_median() {
        let data: Vec<f64> = (1..=11).map(|x| x as f64).collect();
        let m = quantile_batch(&data, 0.5).expect("should succeed");
        assert!((m - 6.0).abs() < 1e-10 /* median of 1..11 = 6 */,);
    }

    #[test]
    fn test_quantile_batch_min() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let m = quantile_batch(&data, 0.0).expect("should succeed");
        assert!((m - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_quantile_batch_max() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let m = quantile_batch(&data, 1.0).expect("should succeed");
        assert!((m - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_median_batch() {
        let data = vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0];
        let m = median_batch(&data).expect("should succeed");
        assert!(m > 0.0 /* median is positive */,);
    }

    #[test]
    fn test_empty_returns_none() {
        assert!(quantile_batch(&[], 0.5).is_none() /* empty input */,);
    }

    #[test]
    fn test_sample_count() {
        let mut q = P2Quantile::new(0.5);
        for i in 0..10 {
            q.add(i as f64);
        }
        assert_eq!(q.sample_count(), 10);
    }
}
