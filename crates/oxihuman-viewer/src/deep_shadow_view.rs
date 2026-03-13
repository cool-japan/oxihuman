// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct DeepShadowView {
    pub layer_count: u32,
    pub min_transmittance: f32,
    pub bias: f32,
}

pub fn new_deep_shadow_view() -> DeepShadowView {
    DeepShadowView {
        layer_count: 8,
        min_transmittance: 0.001,
        bias: 0.0005,
    }
}

pub fn ds_set_layer_count(v: &mut DeepShadowView, n: u32) {
    v.layer_count = n.clamp(2, 32);
}

pub fn ds_transmittance(v: &DeepShadowView, alpha: f32) -> f32 {
    let a = alpha.clamp(0.0, 1.0);
    (1.0 - a)
        .powf(v.layer_count as f32)
        .max(v.min_transmittance)
}

pub fn ds_is_high_depth(v: &DeepShadowView) -> bool {
    v.layer_count >= 16
}

pub fn ds_blend(a: &DeepShadowView, b: &DeepShadowView, t: f32) -> DeepShadowView {
    let t = t.clamp(0.0, 1.0);
    let lc =
        (a.layer_count as f32 + (b.layer_count as f32 - a.layer_count as f32) * t).round() as u32;
    DeepShadowView {
        layer_count: lc.clamp(2, 32),
        min_transmittance: a.min_transmittance + (b.min_transmittance - a.min_transmittance) * t,
        bias: a.bias + (b.bias - a.bias) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default layer count */
        let v = new_deep_shadow_view();
        assert_eq!(v.layer_count, 8);
    }

    #[test]
    fn test_set_layer_count_clamped() {
        /* clamped to 32 */
        let mut v = new_deep_shadow_view();
        ds_set_layer_count(&mut v, 100);
        assert_eq!(v.layer_count, 32);
    }

    #[test]
    fn test_transmittance_opaque() {
        /* fully opaque surface yields min transmittance */
        let v = new_deep_shadow_view();
        let t = ds_transmittance(&v, 1.0);
        assert!((t - v.min_transmittance).abs() < 1e-6);
    }

    #[test]
    fn test_not_high_depth_by_default() {
        /* 8 layers is not high depth */
        let v = new_deep_shadow_view();
        assert!(!ds_is_high_depth(&v));
    }

    #[test]
    fn test_blend() {
        /* midpoint bias */
        let a = DeepShadowView {
            layer_count: 8,
            min_transmittance: 0.001,
            bias: 0.0,
        };
        let b = DeepShadowView {
            layer_count: 8,
            min_transmittance: 0.001,
            bias: 0.002,
        };
        let c = ds_blend(&a, &b, 0.5);
        assert!((c.bias - 0.001).abs() < 1e-6);
    }
}
