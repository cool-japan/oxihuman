// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Full body height scaling morph control.

#![allow(dead_code)]

/// Full body height parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyHeight {
    pub total_height: f32,
    pub leg_ratio: f32,
    pub torso_ratio: f32,
    pub head_ratio: f32,
}

/// Returns a default `BodyHeight` with average proportions.
#[allow(dead_code)]
pub fn default_body_height() -> BodyHeight {
    BodyHeight {
        total_height: 1.75,
        leg_ratio: 0.47,
        torso_ratio: 0.43,
        head_ratio: 0.10,
    }
}

/// Applies body height scaling to a weight slice.
/// Weights are indexed as: [0] = leg, [1] = torso, [2] = head height scale.
#[allow(dead_code)]
pub fn apply_body_height(weights: &mut [f32], bh: &BodyHeight) {
    if !weights.is_empty() {
        weights[0] = bh.leg_ratio;
    }
    if weights.len() > 1 {
        weights[1] = bh.torso_ratio;
    }
    if weights.len() > 2 {
        weights[2] = bh.head_ratio;
    }
}

/// Linearly blends two `BodyHeight` values by parameter `t` in [0, 1].
#[allow(dead_code)]
pub fn body_height_blend(a: &BodyHeight, b: &BodyHeight, t: f32) -> BodyHeight {
    let t = t.clamp(0.0, 1.0);
    BodyHeight {
        total_height: a.total_height + (b.total_height - a.total_height) * t,
        leg_ratio: a.leg_ratio + (b.leg_ratio - a.leg_ratio) * t,
        torso_ratio: a.torso_ratio + (b.torso_ratio - a.torso_ratio) * t,
        head_ratio: a.head_ratio + (b.head_ratio - a.head_ratio) * t,
    }
}

/// Computes the effective height from ratios and total height.
#[allow(dead_code)]
pub fn height_from_ratios(bh: &BodyHeight) -> f32 {
    bh.total_height * (bh.leg_ratio + bh.torso_ratio + bh.head_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_body_height() {
        let bh = default_body_height();
        assert!((bh.total_height - 1.75).abs() < 1e-6);
        assert!((bh.leg_ratio - 0.47).abs() < 1e-6);
    }

    #[test]
    fn test_apply_body_height_full() {
        let bh = default_body_height();
        let mut w = [0.0f32; 3];
        apply_body_height(&mut w, &bh);
        assert!((w[0] - 0.47).abs() < 1e-6);
        assert!((w[1] - 0.43).abs() < 1e-6);
        assert!((w[2] - 0.10).abs() < 1e-6);
    }

    #[test]
    fn test_apply_body_height_partial() {
        let bh = default_body_height();
        let mut w = [0.0f32; 2];
        apply_body_height(&mut w, &bh);
        assert!((w[0] - 0.47).abs() < 1e-6);
        assert!((w[1] - 0.43).abs() < 1e-6);
    }

    #[test]
    fn test_apply_body_height_empty() {
        let bh = default_body_height();
        let mut w: [f32; 0] = [];
        apply_body_height(&mut w, &bh); // must not panic
    }

    #[test]
    fn test_blend_at_zero() {
        let a = default_body_height();
        let mut b = default_body_height();
        b.total_height = 2.0;
        let result = body_height_blend(&a, &b, 0.0);
        assert!((result.total_height - a.total_height).abs() < 1e-6);
    }

    #[test]
    fn test_blend_at_one() {
        let a = default_body_height();
        let mut b = default_body_height();
        b.total_height = 2.0;
        let result = body_height_blend(&a, &b, 1.0);
        assert!((result.total_height - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_at_half() {
        let a = default_body_height();
        let mut b = default_body_height();
        b.total_height = 2.0;
        let result = body_height_blend(&a, &b, 0.5);
        let expected = (a.total_height + 2.0) / 2.0;
        assert!((result.total_height - expected).abs() < 1e-5);
    }

    #[test]
    fn test_blend_clamps_t() {
        let a = default_body_height();
        let b = default_body_height();
        let result = body_height_blend(&a, &b, 2.0);
        assert!((result.total_height - b.total_height).abs() < 1e-6);
    }

    #[test]
    fn test_height_from_ratios_approx_one() {
        let bh = default_body_height();
        let h = height_from_ratios(&bh);
        // ratios sum to 1.0, so h == total_height
        assert!((h - bh.total_height).abs() < 1e-5);
    }

    #[test]
    fn test_height_from_ratios_custom() {
        let bh = BodyHeight {
            total_height: 2.0,
            leg_ratio: 0.5,
            torso_ratio: 0.4,
            head_ratio: 0.1,
        };
        let h = height_from_ratios(&bh);
        assert!((h - 2.0).abs() < 1e-5);
    }
}
