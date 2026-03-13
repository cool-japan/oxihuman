// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Crack propagation path stub.
//!
//! Tracks the growth of a crack front through a material using the Paris law
//! for fatigue-driven growth and maximum tangential stress criterion for direction.

use std::f64::consts::PI;

/// A 2D point on a crack path.
#[derive(Debug, Clone, PartialEq)]
pub struct CrackPoint {
    pub x: f64,
    pub y: f64,
}

impl CrackPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &CrackPoint) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// Paris law parameters: da/dN = C * ΔK^m
#[derive(Debug, Clone)]
pub struct ParisLawParams {
    /// Paris coefficient C.
    pub c: f64,
    /// Paris exponent m.
    pub m: f64,
    /// Threshold ΔK below which no growth occurs [MPa√m].
    pub delta_k_threshold: f64,
    /// Critical K_IC at which fast fracture occurs [MPa√m].
    pub k_ic: f64,
}

impl Default for ParisLawParams {
    fn default() -> Self {
        Self {
            c: 1e-12,
            m: 3.0,
            delta_k_threshold: 0.5,
            k_ic: 25.0,
        }
    }
}

/// State of a propagating crack.
#[derive(Debug, Clone)]
pub struct Crack {
    pub path: Vec<CrackPoint>,
    pub half_length: f64,
    pub angle_deg: f64,
    pub cycles: u64,
    pub propagated: bool,
}

impl Crack {
    pub fn new(tip: CrackPoint, initial_half_len: f64) -> Self {
        Self {
            path: vec![tip],
            half_length: initial_half_len,
            angle_deg: 0.0,
            cycles: 0,
            propagated: false,
        }
    }

    pub fn tip(&self) -> &CrackPoint {
        &self.path[self.path.len() - 1]
    }

    pub fn path_length(&self) -> f64 {
        self.path.windows(2).map(|w| w[0].distance_to(&w[1])).sum()
    }

    pub fn step_count(&self) -> usize {
        self.path.len()
    }
}

/// Compute crack growth per cycle using Paris law.
pub fn paris_law_da_dn(delta_k: f64, params: &ParisLawParams) -> f64 {
    if delta_k < params.delta_k_threshold {
        return 0.0;
    }
    params.c * delta_k.powf(params.m)
}

/// Compute Mode I stress intensity for the current crack.
pub fn current_ki(stress: f64, crack: &Crack) -> f64 {
    stress * (PI * crack.half_length).sqrt()
}

/// Advance crack by one load cycle given stress range.
pub fn advance_crack(crack: &mut Crack, sigma_range: f64, params: &ParisLawParams) {
    let ki = current_ki(sigma_range, crack);
    if ki >= params.k_ic {
        crack.propagated = true;
        return;
    }
    let da = paris_law_da_dn(ki, params);
    if da <= 0.0 {
        crack.cycles += 1;
        return;
    }

    crack.half_length += da;
    /* Move tip in crack direction */
    let angle_rad = crack.angle_deg * PI / 180.0;
    let prev = crack.tip().clone();
    let new_tip = CrackPoint::new(prev.x + da * angle_rad.cos(), prev.y + da * angle_rad.sin());
    crack.path.push(new_tip);
    crack.cycles += 1;
}

/// Compute the number of cycles to failure from initial to critical crack size.
pub fn cycles_to_failure(
    initial_a: f64,
    critical_a: f64,
    sigma: f64,
    params: &ParisLawParams,
) -> f64 {
    if params.m == 2.0 {
        /* Analytic solution for m=2 */
        (critical_a / initial_a).ln() / (params.c * PI * sigma * sigma)
    } else {
        let m = params.m;
        let denom = params.c * (PI * sigma * sigma).powf(m / 2.0) * (1.0 - m / 2.0);
        if denom.abs() < 1e-30 {
            return f64::INFINITY;
        }
        (critical_a.powf(1.0 - m / 2.0) - initial_a.powf(1.0 - m / 2.0)) / denom
    }
}

/// Check if fast fracture has occurred.
pub fn is_fractured(crack: &Crack, params: &ParisLawParams, stress: f64) -> bool {
    crack.propagated || current_ki(stress, crack) >= params.k_ic
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_crack() -> Crack {
        Crack::new(CrackPoint::new(0.0, 0.0), 0.001)
    }

    #[test]
    fn test_paris_law_zero_below_threshold() {
        let p = ParisLawParams::default();
        assert_eq!(paris_law_da_dn(0.1, &p), 0.0);
    }

    #[test]
    fn test_paris_law_positive_above_threshold() {
        let p = ParisLawParams {
            delta_k_threshold: 0.1,
            ..Default::default()
        };
        assert!(paris_law_da_dn(1.0, &p) > 0.0);
    }

    #[test]
    fn test_current_ki_positive() {
        let c = default_crack();
        assert!(current_ki(100.0, &c) > 0.0);
    }

    #[test]
    fn test_advance_crack_grows_half_length() {
        let mut c = default_crack();
        let p = ParisLawParams {
            delta_k_threshold: 0.0,
            c: 1e-6,
            m: 2.0,
            k_ic: 1000.0,
        };
        let old_len = c.half_length;
        advance_crack(&mut c, 200.0, &p);
        assert!(c.half_length >= old_len);
    }

    #[test]
    fn test_advance_crack_increments_cycles() {
        let mut c = default_crack();
        let p = ParisLawParams::default();
        advance_crack(&mut c, 10.0, &p);
        assert_eq!(c.cycles, 1);
    }

    #[test]
    fn test_cycles_to_failure_positive() {
        let p = ParisLawParams::default();
        let n = cycles_to_failure(0.001, 0.1, 50.0, &p);
        assert!(n > 0.0);
    }

    #[test]
    fn test_crack_path_length_zero_initially() {
        let c = Crack::new(CrackPoint::new(0.0, 0.0), 0.01);
        assert_eq!(c.path_length(), 0.0);
    }

    #[test]
    fn test_is_fractured_false_small_crack() {
        let c = default_crack();
        let p = ParisLawParams::default();
        assert!(!is_fractured(&c, &p, 1.0));
    }

    #[test]
    fn test_crack_point_distance() {
        let a = CrackPoint::new(0.0, 0.0);
        let b = CrackPoint::new(3.0, 4.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 1e-9);
    }
}
