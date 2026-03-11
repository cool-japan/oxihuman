// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Chest (pectoral/ribcage) shape morph control.

#![allow(dead_code)]

/// Chest shape parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChestControl {
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    pub shape: f32,
}

/// Returns a default `ChestControl`.
#[allow(dead_code)]
pub fn default_chest_control() -> ChestControl {
    ChestControl {
        width: 0.5,
        depth: 0.5,
        height: 0.5,
        shape: 0.0,
    }
}

/// Applies chest control values to a weight slice.
/// Indices: [0] = width, [1] = depth, [2] = height, [3] = shape.
#[allow(dead_code)]
pub fn apply_chest_control(weights: &mut [f32], cc: &ChestControl) {
    if !weights.is_empty() {
        weights[0] = cc.width;
    }
    if weights.len() > 1 {
        weights[1] = cc.depth;
    }
    if weights.len() > 2 {
        weights[2] = cc.height;
    }
    if weights.len() > 3 {
        weights[3] = cc.shape;
    }
}

/// Linearly blends two `ChestControl` values by parameter `t` in [0, 1].
#[allow(dead_code)]
pub fn chest_control_blend(a: &ChestControl, b: &ChestControl, t: f32) -> ChestControl {
    let t = t.clamp(0.0, 1.0);
    ChestControl {
        width: a.width + (b.width - a.width) * t,
        depth: a.depth + (b.depth - a.depth) * t,
        height: a.height + (b.height - a.height) * t,
        shape: a.shape + (b.shape - a.shape) * t,
    }
}

/// Approximate chest volume as a scaled ellipsoid.
#[allow(dead_code)]
pub fn chest_volume_approx(cc: &ChestControl) -> f32 {
    use std::f32::consts::PI;
    (4.0 / 3.0) * PI * cc.width * cc.depth * cc.height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_chest_control() {
        let cc = default_chest_control();
        assert!((cc.width - 0.5).abs() < 1e-6);
        assert!((cc.shape - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_chest_full() {
        let cc = ChestControl { width: 0.6, depth: 0.4, height: 0.7, shape: 0.3 };
        let mut w = [0.0f32; 4];
        apply_chest_control(&mut w, &cc);
        assert!((w[0] - 0.6).abs() < 1e-6);
        assert!((w[3] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_apply_chest_partial() {
        let cc = default_chest_control();
        let mut w = [0.0f32; 2];
        apply_chest_control(&mut w, &cc);
        assert!((w[0] - 0.5).abs() < 1e-6);
        assert!((w[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_chest_empty() {
        let cc = default_chest_control();
        let mut w: [f32; 0] = [];
        apply_chest_control(&mut w, &cc);
    }

    #[test]
    fn test_blend_at_zero() {
        let a = default_chest_control();
        let b = ChestControl { width: 1.0, depth: 1.0, height: 1.0, shape: 1.0 };
        let r = chest_control_blend(&a, &b, 0.0);
        assert!((r.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_at_one() {
        let a = default_chest_control();
        let b = ChestControl { width: 1.0, depth: 1.0, height: 1.0, shape: 1.0 };
        let r = chest_control_blend(&a, &b, 1.0);
        assert!((r.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_mid() {
        let a = default_chest_control();
        let b = ChestControl { width: 1.0, depth: 1.0, height: 1.0, shape: 1.0 };
        let r = chest_control_blend(&a, &b, 0.5);
        assert!((r.width - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_blend_clamps_negative_t() {
        let a = default_chest_control();
        let b = ChestControl { width: 0.0, depth: 0.0, height: 0.0, shape: 0.0 };
        let r = chest_control_blend(&a, &b, -1.0);
        assert!((r.width - a.width).abs() < 1e-6);
    }

    #[test]
    fn test_chest_volume_positive() {
        let cc = default_chest_control();
        assert!(chest_volume_approx(&cc) > 0.0);
    }

    #[test]
    fn test_chest_volume_zero_dim() {
        let cc = ChestControl { width: 0.0, depth: 0.5, height: 0.5, shape: 0.0 };
        assert!((chest_volume_approx(&cc)).abs() < 1e-9);
    }
}
