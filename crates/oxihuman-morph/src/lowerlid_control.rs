// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Lower eyelid position control.

#![allow(dead_code)]

/// Lower eyelid control parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct LowerLidControl {
    /// Left lower lid lowering amount (0 = neutral, 1 = fully lowered).
    pub lower_left: f32,
    /// Right lower lid lowering amount (0 = neutral, 1 = fully lowered).
    pub lower_right: f32,
    /// Puffiness under the eye.
    pub puff: f32,
}

/// Create a default `LowerLidControl` with neutral values.
#[allow(dead_code)]
pub fn default_lower_lid_control() -> LowerLidControl {
    LowerLidControl {
        lower_left: 0.0,
        lower_right: 0.0,
        puff: 0.0,
    }
}

/// Apply lower lid control to a weight slice.
///
/// Expects at least 3 entries: [lower_left, lower_right, puff].
#[allow(dead_code)]
pub fn apply_lower_lid(weights: &mut [f32], ll: &LowerLidControl) {
    if !weights.is_empty() {
        weights[0] = ll.lower_left;
    }
    if weights.len() >= 2 {
        weights[1] = ll.lower_right;
    }
    if weights.len() >= 3 {
        weights[2] = ll.puff;
    }
}

/// Linearly blend two `LowerLidControl` values.
#[allow(dead_code)]
pub fn lower_lid_blend(a: &LowerLidControl, b: &LowerLidControl, t: f32) -> LowerLidControl {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    LowerLidControl {
        lower_left: lerp(a.lower_left, b.lower_left),
        lower_right: lerp(a.lower_right, b.lower_right),
        puff: lerp(a.puff, b.puff),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let ll = default_lower_lid_control();
        assert_eq!(ll.lower_left, 0.0);
        assert_eq!(ll.lower_right, 0.0);
        assert_eq!(ll.puff, 0.0);
    }

    #[test]
    fn test_apply_full() {
        let ll = LowerLidControl { lower_left: 0.4, lower_right: 0.6, puff: 0.2 };
        let mut w = [0.0f32; 3];
        apply_lower_lid(&mut w, &ll);
        assert_eq!(w[0], 0.4);
        assert_eq!(w[1], 0.6);
        assert_eq!(w[2], 0.2);
    }

    #[test]
    fn test_apply_partial() {
        let ll = LowerLidControl { lower_left: 0.9, lower_right: 0.1, puff: 0.5 };
        let mut w = [0.0f32; 2];
        apply_lower_lid(&mut w, &ll);
        assert_eq!(w[0], 0.9);
        assert_eq!(w[1], 0.1);
    }

    #[test]
    fn test_apply_empty() {
        let ll = default_lower_lid_control();
        let mut w: [f32; 0] = [];
        apply_lower_lid(&mut w, &ll);
    }

    #[test]
    fn test_blend_zero() {
        let a = LowerLidControl { lower_left: 0.2, lower_right: 0.3, puff: 0.1 };
        let b = LowerLidControl { lower_left: 0.8, lower_right: 0.9, puff: 0.7 };
        let r = lower_lid_blend(&a, &b, 0.0);
        assert!((r.lower_left - a.lower_left).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = LowerLidControl { lower_left: 0.2, lower_right: 0.3, puff: 0.1 };
        let b = LowerLidControl { lower_left: 0.8, lower_right: 0.9, puff: 0.7 };
        let r = lower_lid_blend(&a, &b, 1.0);
        assert!((r.lower_left - b.lower_left).abs() < 1e-6);
    }

    #[test]
    fn test_blend_half() {
        let a = LowerLidControl { lower_left: 0.0, lower_right: 0.0, puff: 0.0 };
        let b = LowerLidControl { lower_left: 1.0, lower_right: 1.0, puff: 1.0 };
        let r = lower_lid_blend(&a, &b, 0.5);
        assert!((r.puff - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        let ll = default_lower_lid_control();
        let ll2 = ll.clone();
        assert_eq!(ll, ll2);
    }

    #[test]
    fn test_debug_format() {
        let ll = default_lower_lid_control();
        let s = format!("{:?}", ll);
        assert!(s.contains("LowerLidControl"));
    }
}
