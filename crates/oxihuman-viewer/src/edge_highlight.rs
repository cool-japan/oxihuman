// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Edge detection and highlighting post-process.

use std::f32::consts::PI;

/// Configuration for edge highlight.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeHighlightConfig {
    pub thickness: f32,
    pub threshold: f32,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
}

#[allow(dead_code)]
pub fn default_edge_highlight() -> EdgeHighlightConfig {
    EdgeHighlightConfig { thickness: 1.0, threshold: 0.1, color_r: 1.0, color_g: 1.0, color_b: 1.0 }
}

#[allow(dead_code)]
pub fn set_edge_highlight_thickness(cfg: &mut EdgeHighlightConfig, value: f32) {
    cfg.thickness = value.clamp(0.0_f32, 10.0_f32);
}

#[allow(dead_code)]
pub fn set_edge_highlight_threshold(cfg: &mut EdgeHighlightConfig, value: f32) {
    cfg.threshold = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_edge_highlight_color_r(cfg: &mut EdgeHighlightConfig, value: f32) {
    cfg.color_r = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_edge_highlight_color_g(cfg: &mut EdgeHighlightConfig, value: f32) {
    cfg.color_g = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_edge_highlight_color_b(cfg: &mut EdgeHighlightConfig, value: f32) {
    cfg.color_b = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn edge_highlight_weight(cfg: &EdgeHighlightConfig) -> f32 {
    (cfg.thickness * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_edge_highlight(a: &EdgeHighlightConfig, b: &EdgeHighlightConfig, t: f32) -> EdgeHighlightConfig {
    let t = t.clamp(0.0, 1.0);
    EdgeHighlightConfig {
        thickness: a.thickness + (b.thickness - a.thickness) * t,
        threshold: a.threshold + (b.threshold - a.threshold) * t,
        color_r: a.color_r + (b.color_r - a.color_r) * t,
        color_g: a.color_g + (b.color_g - a.color_g) * t,
        color_b: a.color_b + (b.color_b - a.color_b) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_edge_highlight();
        assert!((cfg.thickness - 1.0_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_thickness() {
        let mut cfg = default_edge_highlight();
        set_edge_highlight_thickness(&mut cfg, 0.7);
        assert!((cfg.thickness - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_threshold() {
        let mut cfg = default_edge_highlight();
        set_edge_highlight_threshold(&mut cfg, 0.8);
        assert!((cfg.threshold - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_color_r() {
        let mut cfg = default_edge_highlight();
        set_edge_highlight_color_r(&mut cfg, 0.6);
        assert!((cfg.color_r - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_color_g() {
        let mut cfg = default_edge_highlight();
        set_edge_highlight_color_g(&mut cfg, 0.5);
        assert!((cfg.color_g - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_color_b() {
        let mut cfg = default_edge_highlight();
        set_edge_highlight_color_b(&mut cfg, 0.4);
        assert!((cfg.color_b - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_edge_highlight();
        let w = edge_highlight_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_edge_highlight();
        let mut b = default_edge_highlight();
        b.thickness = 1.0;
        let mid = blend_edge_highlight(&a, &b, 0.5);
        assert!((mid.thickness - 1.0_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_edge_highlight();
        let b = default_edge_highlight();
        let r = blend_edge_highlight(&a, &b, 0.0);
        assert!((r.thickness - a.thickness).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_edge_highlight();
        let b = default_edge_highlight();
        let r = blend_edge_highlight(&a, &b, 1.0);
        assert!((r.thickness - b.thickness).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_edge_highlight();
        let b = default_edge_highlight();
        let r = blend_edge_highlight(&a, &b, 2.0);
        assert!((r.thickness - b.thickness).abs() < 1e-6);
    }
}
