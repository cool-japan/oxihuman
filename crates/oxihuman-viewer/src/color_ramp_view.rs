// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Color ramp visualization for gradient-mapped data display.

use std::f32::consts::PI;

/// Configuration for color ramp view.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorRampConfig {
    pub min_value: f32,
    pub max_value: f32,
    pub gamma: f32,
    pub saturation: f32,
    pub opacity: f32,
}

#[allow(dead_code)]
pub fn default_color_ramp_view() -> ColorRampConfig {
    ColorRampConfig { min_value: 0.0, max_value: 1.0, gamma: 1.0, saturation: 1.0, opacity: 1.0 }
}

#[allow(dead_code)]
pub fn set_color_ramp_view_min_value(cfg: &mut ColorRampConfig, value: f32) {
    cfg.min_value = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_color_ramp_view_max_value(cfg: &mut ColorRampConfig, value: f32) {
    cfg.max_value = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_color_ramp_view_gamma(cfg: &mut ColorRampConfig, value: f32) {
    cfg.gamma = value.clamp(0.1_f32, 10.0_f32);
}

#[allow(dead_code)]
pub fn set_color_ramp_view_saturation(cfg: &mut ColorRampConfig, value: f32) {
    cfg.saturation = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_color_ramp_view_opacity(cfg: &mut ColorRampConfig, value: f32) {
    cfg.opacity = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn color_ramp_view_weight(cfg: &ColorRampConfig) -> f32 {
    (cfg.min_value * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_color_ramp_view(a: &ColorRampConfig, b: &ColorRampConfig, t: f32) -> ColorRampConfig {
    let t = t.clamp(0.0, 1.0);
    ColorRampConfig {
        min_value: a.min_value + (b.min_value - a.min_value) * t,
        max_value: a.max_value + (b.max_value - a.max_value) * t,
        gamma: a.gamma + (b.gamma - a.gamma) * t,
        saturation: a.saturation + (b.saturation - a.saturation) * t,
        opacity: a.opacity + (b.opacity - a.opacity) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_color_ramp_view();
        assert!((cfg.min_value - 0.0_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_min_value() {
        let mut cfg = default_color_ramp_view();
        set_color_ramp_view_min_value(&mut cfg, 0.7);
        assert!((cfg.min_value - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_max_value() {
        let mut cfg = default_color_ramp_view();
        set_color_ramp_view_max_value(&mut cfg, 0.8);
        assert!((cfg.max_value - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_gamma() {
        let mut cfg = default_color_ramp_view();
        set_color_ramp_view_gamma(&mut cfg, 0.6);
        assert!((cfg.gamma - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_saturation() {
        let mut cfg = default_color_ramp_view();
        set_color_ramp_view_saturation(&mut cfg, 0.5);
        assert!((cfg.saturation - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_opacity() {
        let mut cfg = default_color_ramp_view();
        set_color_ramp_view_opacity(&mut cfg, 0.4);
        assert!((cfg.opacity - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_color_ramp_view();
        let w = color_ramp_view_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_color_ramp_view();
        let mut b = default_color_ramp_view();
        b.min_value = 1.0;
        let mid = blend_color_ramp_view(&a, &b, 0.5);
        assert!((mid.min_value - 0.5_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_color_ramp_view();
        let b = default_color_ramp_view();
        let r = blend_color_ramp_view(&a, &b, 0.0);
        assert!((r.min_value - a.min_value).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_color_ramp_view();
        let b = default_color_ramp_view();
        let r = blend_color_ramp_view(&a, &b, 1.0);
        assert!((r.min_value - b.min_value).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_color_ramp_view();
        let b = default_color_ramp_view();
        let r = blend_color_ramp_view(&a, &b, 2.0);
        assert!((r.min_value - b.min_value).abs() < 1e-6);
    }
}
