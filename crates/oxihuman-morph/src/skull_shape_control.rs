// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Skull shape morphing (cranial width, depth, height).

/// Skull shape parameters controlling cranial dimensions.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SkullShape {
    /// Cranial width (lateral dimension), normalised 0..1.
    pub cranial_width: f32,
    /// Cranial depth (front-to-back), normalised 0..1.
    pub cranial_depth: f32,
    /// Cranial height (vertical), normalised 0..1.
    pub cranial_height: f32,
    /// Temporal width (side-to-side at the temples), normalised 0..1.
    pub temporal_width: f32,
}

/// Return a default (neutral) skull shape.
#[allow(dead_code)]
pub fn default_skull_shape() -> SkullShape {
    SkullShape {
        cranial_width: 0.5,
        cranial_depth: 0.5,
        cranial_height: 0.5,
        temporal_width: 0.5,
    }
}

/// Apply skull shape to a morph-weight slice.
///
/// Expects `weights` to have at least 4 elements;
/// elements beyond that are left unchanged.
#[allow(dead_code)]
pub fn apply_skull_shape(weights: &mut [f32], s: &SkullShape) {
    if !weights.is_empty() { weights[0] = s.cranial_width; }
    if weights.len() > 1 { weights[1] = s.cranial_depth; }
    if weights.len() > 2 { weights[2] = s.cranial_height; }
    if weights.len() > 3 { weights[3] = s.temporal_width; }
}

/// Linear blend between two skull shapes.
#[allow(dead_code)]
pub fn skull_shape_blend(a: &SkullShape, b: &SkullShape, t: f32) -> SkullShape {
    let t = t.clamp(0.0, 1.0);
    SkullShape {
        cranial_width:  a.cranial_width  + (b.cranial_width  - a.cranial_width)  * t,
        cranial_depth:  a.cranial_depth  + (b.cranial_depth  - a.cranial_depth)  * t,
        cranial_height: a.cranial_height + (b.cranial_height - a.cranial_height) * t,
        temporal_width: a.temporal_width + (b.temporal_width - a.temporal_width) * t,
    }
}

/// Estimate cranial volume as an ellipsoid approximation (cm³).
///
/// Assumes each parameter maps to a radius in the range [7.0, 10.0] cm.
#[allow(dead_code)]
pub fn skull_volume_estimate(s: &SkullShape) -> f32 {
    let scale = |v: f32| 7.0 + v * 3.0_f32;
    let a = scale(s.cranial_width);
    let b = scale(s.cranial_depth);
    let c = scale(s.cranial_height);
    (4.0 / 3.0) * std::f32::consts::PI * a * b * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_are_half() {
        let s = default_skull_shape();
        assert_eq!(s.cranial_width, 0.5);
        assert_eq!(s.cranial_depth, 0.5);
        assert_eq!(s.cranial_height, 0.5);
        assert_eq!(s.temporal_width, 0.5);
    }

    #[test]
    fn apply_writes_four_weights() {
        let s = SkullShape {
            cranial_width: 0.2,
            cranial_depth: 0.4,
            cranial_height: 0.6,
            temporal_width: 0.8,
        };
        let mut w = [0.0_f32; 4];
        apply_skull_shape(&mut w, &s);
        assert_eq!(w[0], 0.2);
        assert_eq!(w[1], 0.4);
        assert_eq!(w[2], 0.6);
        assert_eq!(w[3], 0.8);
    }

    #[test]
    fn apply_handles_short_slice() {
        let s = default_skull_shape();
        let mut w = [0.0_f32; 2];
        apply_skull_shape(&mut w, &s);
        assert_eq!(w[0], 0.5);
        assert_eq!(w[1], 0.5);
    }

    #[test]
    fn blend_at_zero_returns_a() {
        let a = default_skull_shape();
        let b = SkullShape { cranial_width: 1.0, cranial_depth: 1.0, cranial_height: 1.0, temporal_width: 1.0 };
        let c = skull_shape_blend(&a, &b, 0.0);
        assert_eq!(c, a);
    }

    #[test]
    fn blend_at_one_returns_b() {
        let a = default_skull_shape();
        let b = SkullShape { cranial_width: 1.0, cranial_depth: 1.0, cranial_height: 1.0, temporal_width: 1.0 };
        let c = skull_shape_blend(&a, &b, 1.0);
        assert_eq!(c, b);
    }

    #[test]
    fn blend_midpoint() {
        let a = SkullShape { cranial_width: 0.0, cranial_depth: 0.0, cranial_height: 0.0, temporal_width: 0.0 };
        let b = SkullShape { cranial_width: 1.0, cranial_depth: 1.0, cranial_height: 1.0, temporal_width: 1.0 };
        let c = skull_shape_blend(&a, &b, 0.5);
        assert!((c.cranial_width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn blend_clamps_t_below_zero() {
        let a = default_skull_shape();
        let b = SkullShape { cranial_width: 1.0, cranial_depth: 1.0, cranial_height: 1.0, temporal_width: 1.0 };
        let c = skull_shape_blend(&a, &b, -1.0);
        assert_eq!(c, a);
    }

    #[test]
    fn blend_clamps_t_above_one() {
        let a = default_skull_shape();
        let b = SkullShape { cranial_width: 1.0, cranial_depth: 1.0, cranial_height: 1.0, temporal_width: 1.0 };
        let c = skull_shape_blend(&a, &b, 2.0);
        assert_eq!(c, b);
    }

    #[test]
    fn volume_estimate_positive() {
        let s = default_skull_shape();
        assert!(skull_volume_estimate(&s) > 0.0);
    }

    #[test]
    fn volume_increases_with_size() {
        let small = SkullShape { cranial_width: 0.0, cranial_depth: 0.0, cranial_height: 0.0, temporal_width: 0.0 };
        let large = SkullShape { cranial_width: 1.0, cranial_depth: 1.0, cranial_height: 1.0, temporal_width: 1.0 };
        assert!(skull_volume_estimate(&large) > skull_volume_estimate(&small));
    }

    #[test]
    fn apply_empty_slice_no_panic() {
        let s = default_skull_shape();
        let mut w: [f32; 0] = [];
        apply_skull_shape(&mut w, &s);
    }
}
