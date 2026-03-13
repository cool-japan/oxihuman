// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Euler column buckling model.
#[derive(Debug, Clone)]
pub struct BucklingModel {
    pub length: f32,
    pub youngs_modulus: f32,
    pub moment_of_inertia: f32,
    pub k_factor: f32,
}

/// Create a new BucklingModel.
pub fn new_buckling_model(l: f32, e: f32, i: f32, k: f32) -> BucklingModel {
    BucklingModel {
        length: l,
        youngs_modulus: e,
        moment_of_inertia: i,
        k_factor: k,
    }
}

/// Euler critical load: P_cr = π²·E·I / (K·L)².
pub fn euler_critical_load(b: &BucklingModel) -> f32 {
    let eff_len = b.k_factor * b.length;
    if eff_len.abs() < 1e-12 {
        return f32::INFINITY;
    }
    PI * PI * b.youngs_modulus * b.moment_of_inertia / (eff_len * eff_len)
}

/// Slenderness ratio: lambda = K·L / sqrt(I/A) = K·L·sqrt(A) / sqrt(I).
/// Provided area for calculation: lambda = K*L / r, where r = sqrt(I/A).
pub fn slenderness_ratio(b: &BucklingModel, area: f32) -> f32 {
    if area < 1e-12 || b.moment_of_inertia < 1e-12 {
        return f32::INFINITY;
    }
    let r = (b.moment_of_inertia / area).sqrt();
    if r < 1e-12 {
        return f32::INFINITY;
    }
    b.k_factor * b.length / r
}

/// Returns true when the column is slender (slenderness ratio > 120).
pub fn is_slender(b: &BucklingModel, area: f32) -> bool {
    slenderness_ratio(b, area) > 120.0
}

/// Critical stress: sigma_cr = P_cr / A.
pub fn critical_stress(b: &BucklingModel, area: f32) -> f32 {
    if area < 1e-12 {
        return 0.0;
    }
    euler_critical_load(b) / area
}

/// Safety factor against buckling: SF = P_cr / P_applied.
pub fn buckling_safety_factor(b: &BucklingModel, p_applied: f32) -> f32 {
    if p_applied.abs() < 1e-12 {
        return f32::INFINITY;
    }
    euler_critical_load(b) / p_applied
}

/// Effective length of the column: L_eff = K * L.
pub fn effective_length(b: &BucklingModel) -> f32 {
    b.k_factor * b.length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buckling_model() {
        /* constructor */
        let b = new_buckling_model(3.0, 200e9, 1e-6, 1.0);
        assert!((b.length - 3.0).abs() < 1e-9);
        assert!((b.k_factor - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_euler_critical_load_positive() {
        let b = new_buckling_model(1.0, 200e9, 1e-6, 1.0);
        let p = euler_critical_load(&b);
        assert!(p > 0.0);
    }

    #[test]
    fn test_euler_critical_load_pinned_fixed() {
        /* K=0.7 reduces load capacity vs K=1.0 (wait, less eff length -> more capacity) */
        let b1 = new_buckling_model(1.0, 200e9, 1e-6, 1.0);
        let b2 = new_buckling_model(1.0, 200e9, 1e-6, 0.5);
        assert!(euler_critical_load(&b2) > euler_critical_load(&b1));
    }

    #[test]
    fn test_slenderness_ratio_positive() {
        let b = new_buckling_model(3.0, 200e9, 1e-6, 1.0);
        let sr = slenderness_ratio(&b, 1e-4);
        assert!(sr > 0.0);
    }

    #[test]
    fn test_is_slender_true() {
        /* very long, thin column */
        let b = new_buckling_model(10.0, 200e9, 1e-9, 1.0);
        assert!(is_slender(&b, 1e-5));
    }

    #[test]
    fn test_is_slender_false() {
        /* very short, thick column */
        let b = new_buckling_model(0.01, 200e9, 1e-3, 1.0);
        assert!(!is_slender(&b, 1.0));
    }

    #[test]
    fn test_critical_stress() {
        let b = new_buckling_model(1.0, 200e9, 1e-6, 1.0);
        let sigma = critical_stress(&b, 1e-4);
        assert!(sigma > 0.0);
    }

    #[test]
    fn test_buckling_safety_factor() {
        let b = new_buckling_model(1.0, 200e9, 1e-6, 1.0);
        let p_cr = euler_critical_load(&b);
        let sf = buckling_safety_factor(&b, p_cr / 2.0);
        assert!((sf - 2.0).abs() < 1e-3);
    }

    #[test]
    fn test_effective_length() {
        let b = new_buckling_model(4.0, 200e9, 1e-6, 0.7);
        let le = effective_length(&b);
        assert!((le - 2.8).abs() < 1e-5);
    }
}
