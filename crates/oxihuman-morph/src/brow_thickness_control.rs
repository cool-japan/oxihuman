// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Eyebrow thickness / density control.

#![allow(dead_code)]

/// Eyebrow thickness parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BrowThickness {
    /// Overall brow thickness (0 = thin, 1 = thick).
    pub thickness: f32,
    /// Taper from inner to outer corner (0 = uniform, 1 = tapered).
    pub taper: f32,
    /// Brow length relative to the orbital width (0 = short, 1 = long).
    pub length: f32,
}

/// Create a default `BrowThickness` with moderate values.
#[allow(dead_code)]
pub fn default_brow_thickness() -> BrowThickness {
    BrowThickness {
        thickness: 0.5,
        taper: 0.5,
        length: 0.5,
    }
}

/// Apply brow thickness to a weight slice.
///
/// Expects at least 3 entries: [thickness, taper, length].
#[allow(dead_code)]
pub fn apply_brow_thickness(weights: &mut [f32], bt: &BrowThickness) {
    if !weights.is_empty() {
        weights[0] = bt.thickness;
    }
    if weights.len() >= 2 {
        weights[1] = bt.taper;
    }
    if weights.len() >= 3 {
        weights[2] = bt.length;
    }
}

/// Linearly blend two `BrowThickness` values.
#[allow(dead_code)]
pub fn brow_thickness_blend(a: &BrowThickness, b: &BrowThickness, t: f32) -> BrowThickness {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    BrowThickness {
        thickness: lerp(a.thickness, b.thickness),
        taper: lerp(a.taper, b.taper),
        length: lerp(a.length, b.length),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let bt = default_brow_thickness();
        assert_eq!(bt.thickness, 0.5);
        assert_eq!(bt.taper, 0.5);
        assert_eq!(bt.length, 0.5);
    }

    #[test]
    fn test_apply_full() {
        let bt = BrowThickness { thickness: 0.9, taper: 0.1, length: 0.7 };
        let mut w = [0.0f32; 3];
        apply_brow_thickness(&mut w, &bt);
        assert_eq!(w[0], 0.9);
        assert_eq!(w[1], 0.1);
        assert_eq!(w[2], 0.7);
    }

    #[test]
    fn test_apply_single() {
        let bt = BrowThickness { thickness: 0.6, taper: 0.2, length: 0.4 };
        let mut w = [0.0f32; 1];
        apply_brow_thickness(&mut w, &bt);
        assert_eq!(w[0], 0.6);
    }

    #[test]
    fn test_apply_empty() {
        let bt = default_brow_thickness();
        let mut w: [f32; 0] = [];
        apply_brow_thickness(&mut w, &bt);
    }

    #[test]
    fn test_blend_zero() {
        let a = BrowThickness { thickness: 0.1, taper: 0.2, length: 0.3 };
        let b = BrowThickness { thickness: 0.9, taper: 0.8, length: 0.7 };
        let r = brow_thickness_blend(&a, &b, 0.0);
        assert!((r.thickness - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = BrowThickness { thickness: 0.1, taper: 0.2, length: 0.3 };
        let b = BrowThickness { thickness: 0.9, taper: 0.8, length: 0.7 };
        let r = brow_thickness_blend(&a, &b, 1.0);
        assert!((r.length - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_blend_half() {
        let a = BrowThickness { thickness: 0.0, taper: 0.0, length: 0.0 };
        let b = BrowThickness { thickness: 1.0, taper: 1.0, length: 1.0 };
        let r = brow_thickness_blend(&a, &b, 0.5);
        assert!((r.taper - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone_eq() {
        let bt = default_brow_thickness();
        assert_eq!(bt.clone(), bt);
    }

    #[test]
    fn test_debug() {
        let bt = default_brow_thickness();
        let s = format!("{:?}", bt);
        assert!(s.contains("BrowThickness"));
    }
}
