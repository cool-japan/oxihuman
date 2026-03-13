// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Depth buffer debug visualization.

use std::f32::consts::PI;

/// Configuration for depth debug.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DepthDebugConfig {
    pub near_plane: f32,
    pub far_plane: f32,
    pub linear_mode: f32,
    pub invert: f32,
    pub contrast: f32,
}

#[allow(dead_code)]
pub fn default_depth_debug() -> DepthDebugConfig {
    DepthDebugConfig { near_plane: 0.1, far_plane: 100.0, linear_mode: 1.0, invert: 0.0, contrast: 1.0 }
}

#[allow(dead_code)]
pub fn set_depth_debug_near_plane(cfg: &mut DepthDebugConfig, value: f32) {
    cfg.near_plane = value.clamp(0.001_f32, 100.0_f32);
}

#[allow(dead_code)]
pub fn set_depth_debug_far_plane(cfg: &mut DepthDebugConfig, value: f32) {
    cfg.far_plane = value.clamp(0.1_f32, 10000.0_f32);
}

#[allow(dead_code)]
pub fn set_depth_debug_linear_mode(cfg: &mut DepthDebugConfig, value: f32) {
    cfg.linear_mode = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_depth_debug_invert(cfg: &mut DepthDebugConfig, value: f32) {
    cfg.invert = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_depth_debug_contrast(cfg: &mut DepthDebugConfig, value: f32) {
    cfg.contrast = value.clamp(0.1_f32, 10.0_f32);
}

#[allow(dead_code)]
pub fn depth_debug_weight(cfg: &DepthDebugConfig) -> f32 {
    (cfg.near_plane * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_depth_debug(a: &DepthDebugConfig, b: &DepthDebugConfig, t: f32) -> DepthDebugConfig {
    let t = t.clamp(0.0, 1.0);
    DepthDebugConfig {
        near_plane: a.near_plane + (b.near_plane - a.near_plane) * t,
        far_plane: a.far_plane + (b.far_plane - a.far_plane) * t,
        linear_mode: a.linear_mode + (b.linear_mode - a.linear_mode) * t,
        invert: a.invert + (b.invert - a.invert) * t,
        contrast: a.contrast + (b.contrast - a.contrast) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_depth_debug();
        assert!((cfg.near_plane - 0.1_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_near_plane() {
        let mut cfg = default_depth_debug();
        set_depth_debug_near_plane(&mut cfg, 0.7);
        assert!((cfg.near_plane - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_far_plane() {
        let mut cfg = default_depth_debug();
        set_depth_debug_far_plane(&mut cfg, 0.8);
        assert!((cfg.far_plane - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_linear_mode() {
        let mut cfg = default_depth_debug();
        set_depth_debug_linear_mode(&mut cfg, 0.6);
        assert!((cfg.linear_mode - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_invert() {
        let mut cfg = default_depth_debug();
        set_depth_debug_invert(&mut cfg, 0.5);
        assert!((cfg.invert - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_contrast() {
        let mut cfg = default_depth_debug();
        set_depth_debug_contrast(&mut cfg, 0.4);
        assert!((cfg.contrast - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_depth_debug();
        let w = depth_debug_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_depth_debug();
        let mut b = default_depth_debug();
        b.near_plane = 1.0;
        let mid = blend_depth_debug(&a, &b, 0.5);
        assert!((mid.near_plane - 0.55_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_depth_debug();
        let b = default_depth_debug();
        let r = blend_depth_debug(&a, &b, 0.0);
        assert!((r.near_plane - a.near_plane).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_depth_debug();
        let b = default_depth_debug();
        let r = blend_depth_debug(&a, &b, 1.0);
        assert!((r.near_plane - b.near_plane).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_depth_debug();
        let b = default_depth_debug();
        let r = blend_depth_debug(&a, &b, 2.0);
        assert!((r.near_plane - b.near_plane).abs() < 1e-6);
    }
}
