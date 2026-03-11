// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Hip width and shape morph control.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Hip width and shape parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HipWidth {
    pub width: f32,
    pub depth: f32,
    pub height: f32,
}

/// Returns a default `HipWidth`.
#[allow(dead_code)]
pub fn default_hip_width() -> HipWidth {
    HipWidth {
        width: 0.5,
        depth: 0.5,
        height: 0.5,
    }
}

/// Applies hip width values to a weight slice.
/// Indices: `[0]` = width, `[1]` = depth, `[2]` = height.
#[allow(dead_code)]
pub fn apply_hip_width(weights: &mut [f32], hw: &HipWidth) {
    if !weights.is_empty() {
        weights[0] = hw.width;
    }
    if weights.len() > 1 {
        weights[1] = hw.depth;
    }
    if weights.len() > 2 {
        weights[2] = hw.height;
    }
}

/// Linearly blends two `HipWidth` values by parameter `t` in [0, 1].
#[allow(dead_code)]
pub fn hip_width_blend(a: &HipWidth, b: &HipWidth, t: f32) -> HipWidth {
    let t = t.clamp(0.0, 1.0);
    HipWidth {
        width: a.width + (b.width - a.width) * t,
        depth: a.depth + (b.depth - a.depth) * t,
        height: a.height + (b.height - a.height) * t,
    }
}

/// Computes the width-to-depth ratio of the hip.
#[allow(dead_code)]
pub fn hip_ratio(hw: &HipWidth) -> f32 {
    if hw.depth.abs() < f32::EPSILON {
        return 1.0;
    }
    hw.width / hw.depth
}

/// Approximate hip circumference using ellipse perimeter (Ramanujan approximation).
#[allow(dead_code)]
pub fn hip_circumference_approx(hw: &HipWidth, scale_m: f32) -> f32 {
    let a = hw.width * scale_m;
    let b = hw.depth * scale_m;
    let h = (a - b).powi(2) / ((a + b).powi(2) + f32::EPSILON);
    PI * (a + b) * (1.0 + (3.0 * h) / (10.0 + (4.0 - 3.0 * h).sqrt()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_hip_width() {
        let hw = default_hip_width();
        assert!((hw.width - 0.5).abs() < 1e-6);
        assert!((hw.depth - 0.5).abs() < 1e-6);
        assert!((hw.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_hip_width_full() {
        let hw = HipWidth {
            width: 0.6,
            depth: 0.4,
            height: 0.7,
        };
        let mut w = [0.0f32; 3];
        apply_hip_width(&mut w, &hw);
        assert!((w[0] - 0.6).abs() < 1e-6);
        assert!((w[1] - 0.4).abs() < 1e-6);
        assert!((w[2] - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_apply_hip_width_short_slice() {
        let hw = default_hip_width();
        let mut w = [0.0f32; 1];
        apply_hip_width(&mut w, &hw);
        assert!((w[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_hip_width_empty() {
        let hw = default_hip_width();
        let mut w: [f32; 0] = [];
        apply_hip_width(&mut w, &hw); // must not panic
    }

    #[test]
    fn test_blend_at_zero() {
        let a = default_hip_width();
        let b = HipWidth {
            width: 1.0,
            depth: 1.0,
            height: 1.0,
        };
        let r = hip_width_blend(&a, &b, 0.0);
        assert!((r.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_at_one() {
        let a = default_hip_width();
        let b = HipWidth {
            width: 1.0,
            depth: 1.0,
            height: 1.0,
        };
        let r = hip_width_blend(&a, &b, 1.0);
        assert!((r.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamps_t() {
        let a = default_hip_width();
        let b = HipWidth {
            width: 1.0,
            depth: 0.8,
            height: 0.9,
        };
        let r = hip_width_blend(&a, &b, -1.0);
        assert!((r.width - a.width).abs() < 1e-6);
    }

    #[test]
    fn test_hip_ratio_equal_axes() {
        let hw = HipWidth {
            width: 0.5,
            depth: 0.5,
            height: 0.5,
        };
        assert!((hip_ratio(&hw) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_hip_ratio_wide() {
        let hw = HipWidth {
            width: 1.0,
            depth: 0.5,
            height: 0.5,
        };
        assert!((hip_ratio(&hw) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_hip_circumference_positive() {
        let hw = default_hip_width();
        let c = hip_circumference_approx(&hw, 0.5);
        assert!(c > 0.0);
    }
}
