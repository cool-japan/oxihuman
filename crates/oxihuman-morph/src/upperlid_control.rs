// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Upper eyelid position control.

#![allow(dead_code)]

/// Upper eyelid control parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct UpperLidControl {
    /// Left upper lid raise amount (0 = closed, 1 = fully open).
    pub raise_left: f32,
    /// Right upper lid raise amount (0 = closed, 1 = fully open).
    pub raise_right: f32,
    /// Flutter intensity (rapid micro-movements).
    pub flutter: f32,
}

/// Create a default `UpperLidControl` with neutral values.
#[allow(dead_code)]
pub fn default_upper_lid_control() -> UpperLidControl {
    UpperLidControl {
        raise_left: 0.5,
        raise_right: 0.5,
        flutter: 0.0,
    }
}

/// Apply upper lid control to a weight slice.
///
/// Expects at least 3 entries: [left_raise, right_raise, flutter].
#[allow(dead_code)]
pub fn apply_upper_lid(weights: &mut [f32], ul: &UpperLidControl) {
    if !weights.is_empty() {
        weights[0] = ul.raise_left;
    }
    if weights.len() >= 2 {
        weights[1] = ul.raise_right;
    }
    if weights.len() >= 3 {
        weights[2] = ul.flutter;
    }
}

/// Linearly blend two `UpperLidControl` values.
#[allow(dead_code)]
pub fn upper_lid_blend(a: &UpperLidControl, b: &UpperLidControl, t: f32) -> UpperLidControl {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    UpperLidControl {
        raise_left: lerp(a.raise_left, b.raise_left),
        raise_right: lerp(a.raise_right, b.raise_right),
        flutter: lerp(a.flutter, b.flutter),
    }
}

/// Return a [0, 1] openness value for the left lid.
#[allow(dead_code)]
pub fn lid_openness_left(ul: &UpperLidControl) -> f32 {
    ul.raise_left.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let ul = default_upper_lid_control();
        assert_eq!(ul.raise_left, 0.5);
        assert_eq!(ul.raise_right, 0.5);
        assert_eq!(ul.flutter, 0.0);
    }

    #[test]
    fn test_apply_upper_lid_full() {
        let ul = UpperLidControl { raise_left: 0.8, raise_right: 0.6, flutter: 0.1 };
        let mut w = [0.0f32; 3];
        apply_upper_lid(&mut w, &ul);
        assert_eq!(w[0], 0.8);
        assert_eq!(w[1], 0.6);
        assert_eq!(w[2], 0.1);
    }

    #[test]
    fn test_apply_upper_lid_partial() {
        let ul = UpperLidControl { raise_left: 0.3, raise_right: 0.7, flutter: 0.2 };
        let mut w = [0.0f32; 1];
        apply_upper_lid(&mut w, &ul);
        assert_eq!(w[0], 0.3);
    }

    #[test]
    fn test_apply_upper_lid_empty() {
        let ul = default_upper_lid_control();
        let mut w: [f32; 0] = [];
        apply_upper_lid(&mut w, &ul); // must not panic
    }

    #[test]
    fn test_blend_zero() {
        let a = default_upper_lid_control();
        let b = UpperLidControl { raise_left: 1.0, raise_right: 1.0, flutter: 1.0 };
        let r = upper_lid_blend(&a, &b, 0.0);
        assert_eq!(r.raise_left, a.raise_left);
        assert_eq!(r.raise_right, a.raise_right);
    }

    #[test]
    fn test_blend_one() {
        let a = default_upper_lid_control();
        let b = UpperLidControl { raise_left: 1.0, raise_right: 1.0, flutter: 1.0 };
        let r = upper_lid_blend(&a, &b, 1.0);
        assert_eq!(r.raise_left, b.raise_left);
        assert_eq!(r.flutter, b.flutter);
    }

    #[test]
    fn test_blend_midpoint() {
        let a = UpperLidControl { raise_left: 0.0, raise_right: 0.0, flutter: 0.0 };
        let b = UpperLidControl { raise_left: 1.0, raise_right: 1.0, flutter: 1.0 };
        let r = upper_lid_blend(&a, &b, 0.5);
        assert!((r.raise_left - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_lid_openness_clamped_low() {
        let ul = UpperLidControl { raise_left: -0.5, raise_right: 0.0, flutter: 0.0 };
        assert_eq!(lid_openness_left(&ul), 0.0);
    }

    #[test]
    fn test_lid_openness_clamped_high() {
        let ul = UpperLidControl { raise_left: 2.0, raise_right: 0.0, flutter: 0.0 };
        assert_eq!(lid_openness_left(&ul), 1.0);
    }

    #[test]
    fn test_lid_openness_normal() {
        let ul = UpperLidControl { raise_left: 0.75, raise_right: 0.0, flutter: 0.0 };
        assert!((lid_openness_left(&ul) - 0.75).abs() < 1e-6);
    }
}
