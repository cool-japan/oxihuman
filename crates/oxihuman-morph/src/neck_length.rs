// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! NeckLength — neck length and volume control.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Neck length and radius parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckLength {
    pub length: f32,
    pub radius: f32,
}

impl Default for NeckLength {
    fn default() -> Self {
        NeckLength { length: 1.0, radius: 0.1 }
    }
}

/// Create a default `NeckLength`.
#[allow(dead_code)]
pub fn new_neck_length() -> NeckLength {
    NeckLength::default()
}

/// Set the neck length.
#[allow(dead_code)]
pub fn set_neck_length(nl: &mut NeckLength, l: f32) {
    nl.length = l;
}

/// Return the current neck length.
#[allow(dead_code)]
pub fn get_neck_length(nl: &NeckLength) -> f32 {
    nl.length
}

/// Return the neck radius.
#[allow(dead_code)]
pub fn neck_radius(nl: &NeckLength) -> f32 {
    nl.radius
}

/// Approximate cylindrical neck volume: π·r²·h.
#[allow(dead_code)]
pub fn neck_volume_approx(nl: &NeckLength) -> f32 {
    PI * nl.radius * nl.radius * nl.length
}

/// Write the neck length into a weight array (index 0).
#[allow(dead_code)]
pub fn apply_neck_length(nl: &NeckLength, weights: &mut [f32]) {
    if !weights.is_empty() {
        weights[0] = nl.length;
    }
}

/// Convert `NeckLength` to a scalar parameter in [0, 1].
#[allow(dead_code)]
pub fn neck_to_param(nl: &NeckLength, min_l: f32, max_l: f32) -> f32 {
    let range = (max_l - min_l).max(f32::EPSILON);
    ((nl.length - min_l) / range).clamp(0.0, 1.0)
}

/// Reconstruct a `NeckLength` from a scalar parameter in [0, 1].
#[allow(dead_code)]
pub fn neck_from_param(param: f32, min_l: f32, max_l: f32) -> NeckLength {
    NeckLength { length: min_l + param.clamp(0.0, 1.0) * (max_l - min_l), radius: 0.1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_neck_length() {
        let nl = new_neck_length();
        assert!((nl.length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_get_neck_length() {
        let mut nl = new_neck_length();
        set_neck_length(&mut nl, 1.4);
        assert!((get_neck_length(&nl) - 1.4).abs() < 1e-6);
    }

    #[test]
    fn test_neck_radius() {
        let nl = new_neck_length();
        assert!(neck_radius(&nl) > 0.0);
    }

    #[test]
    fn test_neck_volume_positive() {
        let nl = new_neck_length();
        assert!(neck_volume_approx(&nl) > 0.0);
    }

    #[test]
    fn test_apply_neck_length() {
        let nl = NeckLength { length: 1.2, radius: 0.1 };
        let mut w = vec![0.0_f32];
        apply_neck_length(&nl, &mut w);
        assert!((w[0] - 1.2).abs() < 1e-6);
    }

    #[test]
    fn test_neck_to_param_midpoint() {
        let nl = NeckLength { length: 1.5, radius: 0.1 };
        let p = neck_to_param(&nl, 1.0, 2.0);
        assert!((p - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_neck_from_param_midpoint() {
        let nl = neck_from_param(0.5, 1.0, 2.0);
        assert!((nl.length - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_neck_roundtrip() {
        let nl = NeckLength { length: 1.7, radius: 0.1 };
        let p = neck_to_param(&nl, 1.0, 2.0);
        let nl2 = neck_from_param(p, 1.0, 2.0);
        assert!((nl2.length - nl.length).abs() < 1e-4);
    }
}
