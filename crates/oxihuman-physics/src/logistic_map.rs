// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Logistic map iteration.

/// State of the logistic map x_{n+1} = r * x_n * (1 - x_n).
#[derive(Debug, Clone)]
pub struct LogisticMap {
    pub x: f64,
    pub r: f64,
    pub iteration: u64,
}

impl LogisticMap {
    pub fn new(x0: f64, r: f64) -> Self {
        Self {
            x: x0.clamp(0.0, 1.0),
            r,
            iteration: 0,
        }
    }

    pub fn step(&mut self) {
        self.x = self.r * self.x * (1.0 - self.x);
        self.iteration += 1;
    }

    pub fn iterate_n(&mut self, n: u64) {
        for _ in 0..n {
            self.step();
        }
    }

    /// Collect n iterations as a time series.
    pub fn orbit(&self, n: u64) -> Vec<f64> {
        let mut x = self.x;
        let mut out = Vec::with_capacity(n as usize);
        for _ in 0..n {
            x = self.r * x * (1.0 - x);
            out.push(x);
        }
        out
    }

    pub fn is_bounded(&self) -> bool {
        (0.0..=1.0).contains(&self.x)
    }
}

pub fn new_logistic_map(x0: f64, r: f64) -> LogisticMap {
    LogisticMap::new(x0, r)
}

pub fn lm_step(lm: &mut LogisticMap) {
    lm.step();
}

pub fn lm_iterate(lm: &mut LogisticMap, n: u64) {
    lm.iterate_n(n);
}

pub fn lm_value(lm: &LogisticMap) -> f64 {
    lm.x
}

pub fn lm_orbit(lm: &LogisticMap, n: u64) -> Vec<f64> {
    lm.orbit(n)
}

pub fn lm_is_bounded(lm: &LogisticMap) -> bool {
    lm.is_bounded()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map() {
        let lm = new_logistic_map(0.5, 3.5);
        assert!((lm_value(&lm) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_step() {
        let mut lm = new_logistic_map(0.5, 3.5);
        lm_step(&mut lm);
        /* r=3.5, x=0.5 → 3.5*0.5*0.5 = 0.875 */
        assert!((lm_value(&lm) - 0.875).abs() < 1e-10);
    }

    #[test]
    fn test_bounded_after_steps() {
        let mut lm = new_logistic_map(0.3, 3.9);
        lm_iterate(&mut lm, 1000);
        assert!(lm_is_bounded(&lm));
    }

    #[test]
    fn test_fixed_point_r_1() {
        /* r=1: x* = 0 is the only fixed point */
        let mut lm = new_logistic_map(0.0, 1.0);
        lm_step(&mut lm);
        assert_eq!(lm_value(&lm), 0.0);
    }

    #[test]
    fn test_orbit_length() {
        let lm = new_logistic_map(0.5, 3.5);
        let orb = lm_orbit(&lm, 10);
        assert_eq!(orb.len(), 10);
    }

    #[test]
    fn test_orbit_bounded() {
        let lm = new_logistic_map(0.2, 3.7);
        let orb = lm_orbit(&lm, 100);
        for v in &orb {
            assert!((0.0..=1.0).contains(v));
        }
    }

    #[test]
    fn test_iteration_counter() {
        let mut lm = new_logistic_map(0.5, 2.0);
        lm_iterate(&mut lm, 5);
        assert_eq!(lm.iteration, 5);
    }

    #[test]
    fn test_clamp_initial_x() {
        /* x0 > 1 should be clamped to 1 */
        let lm = new_logistic_map(2.0, 3.5);
        assert!((lm_value(&lm) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_r_4_chaotic() {
        /* r=4 is fully chaotic, still bounded in [0,1] */
        let mut lm = new_logistic_map(0.4, 4.0);
        lm_iterate(&mut lm, 500);
        assert!(lm_is_bounded(&lm));
    }
}
