// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct DepthPeelingView {
    pub layer_count: u32,
    pub min_depth_separation: f32,
    pub alpha_threshold: f32,
}

pub fn new_depth_peeling_view() -> DepthPeelingView {
    DepthPeelingView {
        layer_count: 4,
        min_depth_separation: 0.001,
        alpha_threshold: 0.01,
    }
}

pub fn dp_set_layer_count(v: &mut DepthPeelingView, n: u32) {
    v.layer_count = n.clamp(1, 16);
}

pub fn dp_is_single_pass(v: &DepthPeelingView) -> bool {
    v.layer_count == 1
}

pub fn dp_coverage(v: &DepthPeelingView) -> f32 {
    1.0 - (1.0 - v.alpha_threshold).powi(v.layer_count as i32)
}

pub fn dp_blend(a: &DepthPeelingView, b: &DepthPeelingView, t: f32) -> DepthPeelingView {
    let t = t.clamp(0.0, 1.0);
    let lc =
        (a.layer_count as f32 + (b.layer_count as f32 - a.layer_count as f32) * t).round() as u32;
    DepthPeelingView {
        layer_count: lc.clamp(1, 16),
        min_depth_separation: a.min_depth_separation
            + (b.min_depth_separation - a.min_depth_separation) * t,
        alpha_threshold: a.alpha_threshold + (b.alpha_threshold - a.alpha_threshold) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default layer count */
        let v = new_depth_peeling_view();
        assert_eq!(v.layer_count, 4);
    }

    #[test]
    fn test_set_layer_count() {
        /* clamped to 16 */
        let mut v = new_depth_peeling_view();
        dp_set_layer_count(&mut v, 20);
        assert_eq!(v.layer_count, 16);
    }

    #[test]
    fn test_not_single_pass_by_default() {
        /* 4 layers is not single pass */
        let v = new_depth_peeling_view();
        assert!(!dp_is_single_pass(&v));
    }

    #[test]
    fn test_single_pass() {
        /* 1 layer is single pass */
        let mut v = new_depth_peeling_view();
        dp_set_layer_count(&mut v, 1);
        assert!(dp_is_single_pass(&v));
    }

    #[test]
    fn test_blend() {
        /* midpoint layer count */
        let a = DepthPeelingView {
            layer_count: 2,
            min_depth_separation: 0.001,
            alpha_threshold: 0.01,
        };
        let b = DepthPeelingView {
            layer_count: 8,
            min_depth_separation: 0.001,
            alpha_threshold: 0.01,
        };
        let c = dp_blend(&a, &b, 0.5);
        assert!(c.layer_count >= 4 && c.layer_count <= 6);
    }
}
