// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Object bounding box computation and debug display.

use std::f32::consts::PI;

/// Configuration for object bounds.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ObjectBoundsConfig {
    pub line_width: f32,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
    pub padding: f32,
}

#[allow(dead_code)]
pub fn default_object_bounds() -> ObjectBoundsConfig {
    ObjectBoundsConfig { line_width: 1.0, color_r: 1.0, color_g: 1.0, color_b: 1.0, padding: 0.01 }
}

#[allow(dead_code)]
pub fn set_object_bounds_line_width(cfg: &mut ObjectBoundsConfig, value: f32) {
    cfg.line_width = value.clamp(0.0_f32, 10.0_f32);
}

#[allow(dead_code)]
pub fn set_object_bounds_color_r(cfg: &mut ObjectBoundsConfig, value: f32) {
    cfg.color_r = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_object_bounds_color_g(cfg: &mut ObjectBoundsConfig, value: f32) {
    cfg.color_g = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_object_bounds_color_b(cfg: &mut ObjectBoundsConfig, value: f32) {
    cfg.color_b = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_object_bounds_padding(cfg: &mut ObjectBoundsConfig, value: f32) {
    cfg.padding = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn object_bounds_weight(cfg: &ObjectBoundsConfig) -> f32 {
    (cfg.line_width * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_object_bounds(a: &ObjectBoundsConfig, b: &ObjectBoundsConfig, t: f32) -> ObjectBoundsConfig {
    let t = t.clamp(0.0, 1.0);
    ObjectBoundsConfig {
        line_width: a.line_width + (b.line_width - a.line_width) * t,
        color_r: a.color_r + (b.color_r - a.color_r) * t,
        color_g: a.color_g + (b.color_g - a.color_g) * t,
        color_b: a.color_b + (b.color_b - a.color_b) * t,
        padding: a.padding + (b.padding - a.padding) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_object_bounds();
        assert!((cfg.line_width - 1.0_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_line_width() {
        let mut cfg = default_object_bounds();
        set_object_bounds_line_width(&mut cfg, 0.7);
        assert!((cfg.line_width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_color_r() {
        let mut cfg = default_object_bounds();
        set_object_bounds_color_r(&mut cfg, 0.8);
        assert!((cfg.color_r - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_color_g() {
        let mut cfg = default_object_bounds();
        set_object_bounds_color_g(&mut cfg, 0.6);
        assert!((cfg.color_g - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_color_b() {
        let mut cfg = default_object_bounds();
        set_object_bounds_color_b(&mut cfg, 0.5);
        assert!((cfg.color_b - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_padding() {
        let mut cfg = default_object_bounds();
        set_object_bounds_padding(&mut cfg, 0.4);
        assert!((cfg.padding - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_object_bounds();
        let w = object_bounds_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_object_bounds();
        let mut b = default_object_bounds();
        b.line_width = 1.0;
        let mid = blend_object_bounds(&a, &b, 0.5);
        assert!((mid.line_width - 1.0_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_object_bounds();
        let b = default_object_bounds();
        let r = blend_object_bounds(&a, &b, 0.0);
        assert!((r.line_width - a.line_width).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_object_bounds();
        let b = default_object_bounds();
        let r = blend_object_bounds(&a, &b, 1.0);
        assert!((r.line_width - b.line_width).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_object_bounds();
        let b = default_object_bounds();
        let r = blend_object_bounds(&a, &b, 2.0);
        assert!((r.line_width - b.line_width).abs() < 1e-6);
    }
}
