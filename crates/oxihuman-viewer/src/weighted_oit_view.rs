// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct WoitView {
    pub weight_power: f32,
    pub depth_scale: f32,
    pub alpha_correction: f32,
}

pub fn new_woit_view() -> WoitView {
    WoitView {
        weight_power: 3.0,
        depth_scale: 1.0,
        alpha_correction: 0.5,
    }
}

pub fn woit_set_weight_power(v: &mut WoitView, p: f32) {
    v.weight_power = p.clamp(0.5, 10.0);
}

pub fn woit_weight(v: &WoitView, alpha: f32, depth: f32) -> f32 {
    let a = alpha.clamp(0.0, 1.0);
    let d = depth.abs();
    a * (1.0 / (1e-5 + d / v.depth_scale)).powf(v.weight_power)
}

pub fn woit_is_high_power(v: &WoitView) -> bool {
    v.weight_power > 5.0
}

pub fn woit_blend(a: &WoitView, b: &WoitView, t: f32) -> WoitView {
    let t = t.clamp(0.0, 1.0);
    WoitView {
        weight_power: a.weight_power + (b.weight_power - a.weight_power) * t,
        depth_scale: a.depth_scale + (b.depth_scale - a.depth_scale) * t,
        alpha_correction: a.alpha_correction + (b.alpha_correction - a.alpha_correction) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default weight power */
        let v = new_woit_view();
        assert!((v.weight_power - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_weight_power_clamped() {
        /* clamped to max */
        let mut v = new_woit_view();
        woit_set_weight_power(&mut v, 20.0);
        assert!((v.weight_power - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_weight_positive() {
        /* weight is positive for valid inputs */
        let v = new_woit_view();
        let w = woit_weight(&v, 0.5, 1.0);
        assert!(w > 0.0);
    }

    #[test]
    fn test_not_high_power_by_default() {
        /* default power 3 is not high */
        let v = new_woit_view();
        assert!(!woit_is_high_power(&v));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = WoitView {
            weight_power: 0.0,
            depth_scale: 0.0,
            alpha_correction: 0.0,
        };
        let b = WoitView {
            weight_power: 2.0,
            depth_scale: 2.0,
            alpha_correction: 2.0,
        };
        let c = woit_blend(&a, &b, 0.5);
        assert!((c.weight_power - 1.0).abs() < 1e-5);
    }
}
