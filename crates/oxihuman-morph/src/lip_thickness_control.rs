// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Lip thickness (upper/lower volume) control.

#![allow(dead_code)]

/// Lip thickness parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct LipThickness {
    /// Upper lip volume (0 = thin, 1 = full).
    pub upper_vol: f32,
    /// Lower lip volume (0 = thin, 1 = full).
    pub lower_vol: f32,
    /// Corner volume (0 = tapered, 1 = full corners).
    pub corner_vol: f32,
}

/// Create a default `LipThickness` with neutral values.
#[allow(dead_code)]
pub fn default_lip_thickness() -> LipThickness {
    LipThickness {
        upper_vol: 0.5,
        lower_vol: 0.5,
        corner_vol: 0.5,
    }
}

/// Apply lip thickness to a weight slice.
///
/// Expects at least 3 entries: [upper_vol, lower_vol, corner_vol].
#[allow(dead_code)]
pub fn apply_lip_thickness(weights: &mut [f32], lt: &LipThickness) {
    if !weights.is_empty() {
        weights[0] = lt.upper_vol;
    }
    if weights.len() >= 2 {
        weights[1] = lt.lower_vol;
    }
    if weights.len() >= 3 {
        weights[2] = lt.corner_vol;
    }
}

/// Linearly blend two `LipThickness` values.
#[allow(dead_code)]
pub fn lip_thickness_blend(a: &LipThickness, b: &LipThickness, t: f32) -> LipThickness {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    LipThickness {
        upper_vol: lerp(a.upper_vol, b.upper_vol),
        lower_vol: lerp(a.lower_vol, b.lower_vol),
        corner_vol: lerp(a.corner_vol, b.corner_vol),
    }
}

/// Return the sum of all lip volumes as a total volume estimate.
#[allow(dead_code)]
pub fn total_lip_volume(lt: &LipThickness) -> f32 {
    lt.upper_vol + lt.lower_vol + lt.corner_vol
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let lt = default_lip_thickness();
        assert_eq!(lt.upper_vol, 0.5);
        assert_eq!(lt.lower_vol, 0.5);
        assert_eq!(lt.corner_vol, 0.5);
    }

    #[test]
    fn test_apply_full() {
        let lt = LipThickness {
            upper_vol: 0.8,
            lower_vol: 0.6,
            corner_vol: 0.4,
        };
        let mut w = [0.0f32; 3];
        apply_lip_thickness(&mut w, &lt);
        assert_eq!(w[0], 0.8);
        assert_eq!(w[1], 0.6);
        assert_eq!(w[2], 0.4);
    }

    #[test]
    fn test_apply_partial() {
        let lt = LipThickness {
            upper_vol: 0.9,
            lower_vol: 0.3,
            corner_vol: 0.1,
        };
        let mut w = [0.0f32; 2];
        apply_lip_thickness(&mut w, &lt);
        assert_eq!(w[0], 0.9);
        assert_eq!(w[1], 0.3);
    }

    #[test]
    fn test_apply_empty() {
        let lt = default_lip_thickness();
        let mut w: [f32; 0] = [];
        apply_lip_thickness(&mut w, &lt);
    }

    #[test]
    fn test_blend_zero() {
        let a = LipThickness {
            upper_vol: 0.1,
            lower_vol: 0.2,
            corner_vol: 0.3,
        };
        let b = LipThickness {
            upper_vol: 0.9,
            lower_vol: 0.8,
            corner_vol: 0.7,
        };
        let r = lip_thickness_blend(&a, &b, 0.0);
        assert!((r.upper_vol - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = LipThickness {
            upper_vol: 0.1,
            lower_vol: 0.2,
            corner_vol: 0.3,
        };
        let b = LipThickness {
            upper_vol: 0.9,
            lower_vol: 0.8,
            corner_vol: 0.7,
        };
        let r = lip_thickness_blend(&a, &b, 1.0);
        assert!((r.corner_vol - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_total_lip_volume() {
        let lt = LipThickness {
            upper_vol: 0.4,
            lower_vol: 0.5,
            corner_vol: 0.1,
        };
        assert!((total_lip_volume(&lt) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_total_default_volume() {
        let lt = default_lip_thickness();
        assert!((total_lip_volume(&lt) - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_clone_eq() {
        let lt = default_lip_thickness();
        assert_eq!(lt.clone(), lt);
    }
}
