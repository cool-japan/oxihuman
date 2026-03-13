// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Motion vector debug visualization for temporal effects.

use std::f32::consts::PI;

/// Configuration for motion vector view.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MotionVectorConfig {
    pub scale: f32,
    pub threshold: f32,
    pub color_mode: f32,
    pub overlay_opacity: f32,
    pub arrow_size: f32,
}

#[allow(dead_code)]
pub fn default_motion_vector_view() -> MotionVectorConfig {
    MotionVectorConfig { scale: 1.0, threshold: 0.1, color_mode: 0.0, overlay_opacity: 0.5, arrow_size: 2.0 }
}

#[allow(dead_code)]
pub fn set_motion_vector_view_scale(cfg: &mut MotionVectorConfig, value: f32) {
    cfg.scale = value.clamp(0.0_f32, 10.0_f32);
}

#[allow(dead_code)]
pub fn set_motion_vector_view_threshold(cfg: &mut MotionVectorConfig, value: f32) {
    cfg.threshold = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_motion_vector_view_color_mode(cfg: &mut MotionVectorConfig, value: f32) {
    cfg.color_mode = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_motion_vector_view_overlay_opacity(cfg: &mut MotionVectorConfig, value: f32) {
    cfg.overlay_opacity = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_motion_vector_view_arrow_size(cfg: &mut MotionVectorConfig, value: f32) {
    cfg.arrow_size = value.clamp(0.0_f32, 10.0_f32);
}

#[allow(dead_code)]
pub fn motion_vector_view_weight(cfg: &MotionVectorConfig) -> f32 {
    (cfg.scale * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_motion_vector_view(a: &MotionVectorConfig, b: &MotionVectorConfig, t: f32) -> MotionVectorConfig {
    let t = t.clamp(0.0, 1.0);
    MotionVectorConfig {
        scale: a.scale + (b.scale - a.scale) * t,
        threshold: a.threshold + (b.threshold - a.threshold) * t,
        color_mode: a.color_mode + (b.color_mode - a.color_mode) * t,
        overlay_opacity: a.overlay_opacity + (b.overlay_opacity - a.overlay_opacity) * t,
        arrow_size: a.arrow_size + (b.arrow_size - a.arrow_size) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_motion_vector_view();
        assert!((cfg.scale - 1.0_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_scale() {
        let mut cfg = default_motion_vector_view();
        set_motion_vector_view_scale(&mut cfg, 0.7);
        assert!((cfg.scale - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_threshold() {
        let mut cfg = default_motion_vector_view();
        set_motion_vector_view_threshold(&mut cfg, 0.8);
        assert!((cfg.threshold - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_color_mode() {
        let mut cfg = default_motion_vector_view();
        set_motion_vector_view_color_mode(&mut cfg, 0.6);
        assert!((cfg.color_mode - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_overlay_opacity() {
        let mut cfg = default_motion_vector_view();
        set_motion_vector_view_overlay_opacity(&mut cfg, 0.5);
        assert!((cfg.overlay_opacity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_arrow_size() {
        let mut cfg = default_motion_vector_view();
        set_motion_vector_view_arrow_size(&mut cfg, 0.4);
        assert!((cfg.arrow_size - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_motion_vector_view();
        let w = motion_vector_view_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_motion_vector_view();
        let mut b = default_motion_vector_view();
        b.scale = 1.0;
        let mid = blend_motion_vector_view(&a, &b, 0.5);
        assert!((mid.scale - 1.0_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_motion_vector_view();
        let b = default_motion_vector_view();
        let r = blend_motion_vector_view(&a, &b, 0.0);
        assert!((r.scale - a.scale).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_motion_vector_view();
        let b = default_motion_vector_view();
        let r = blend_motion_vector_view(&a, &b, 1.0);
        assert!((r.scale - b.scale).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_motion_vector_view();
        let b = default_motion_vector_view();
        let r = blend_motion_vector_view(&a, &b, 2.0);
        assert!((r.scale - b.scale).abs() < 1e-6);
    }
}
