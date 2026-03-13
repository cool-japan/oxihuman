// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Render pass management and ordering.

use std::f32::consts::PI;

/// Configuration for pass manager.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PassManagerConfig {
    pub max_passes: f32,
    pub depth_prepass: f32,
    pub shadow_pass: f32,
    pub post_pass: f32,
    pub overlay_pass: f32,
}

#[allow(dead_code)]
pub fn default_pass_manager() -> PassManagerConfig {
    PassManagerConfig { max_passes: 8.0, depth_prepass: 1.0, shadow_pass: 1.0, post_pass: 1.0, overlay_pass: 1.0 }
}

#[allow(dead_code)]
pub fn set_pass_manager_max_passes(cfg: &mut PassManagerConfig, value: f32) {
    cfg.max_passes = value.clamp(1.0_f32, 128.0_f32);
}

#[allow(dead_code)]
pub fn set_pass_manager_depth_prepass(cfg: &mut PassManagerConfig, value: f32) {
    cfg.depth_prepass = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_pass_manager_shadow_pass(cfg: &mut PassManagerConfig, value: f32) {
    cfg.shadow_pass = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_pass_manager_post_pass(cfg: &mut PassManagerConfig, value: f32) {
    cfg.post_pass = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_pass_manager_overlay_pass(cfg: &mut PassManagerConfig, value: f32) {
    cfg.overlay_pass = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn pass_manager_weight(cfg: &PassManagerConfig) -> f32 {
    (cfg.max_passes * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_pass_manager(a: &PassManagerConfig, b: &PassManagerConfig, t: f32) -> PassManagerConfig {
    let t = t.clamp(0.0, 1.0);
    PassManagerConfig {
        max_passes: a.max_passes + (b.max_passes - a.max_passes) * t,
        depth_prepass: a.depth_prepass + (b.depth_prepass - a.depth_prepass) * t,
        shadow_pass: a.shadow_pass + (b.shadow_pass - a.shadow_pass) * t,
        post_pass: a.post_pass + (b.post_pass - a.post_pass) * t,
        overlay_pass: a.overlay_pass + (b.overlay_pass - a.overlay_pass) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_pass_manager();
        assert!((cfg.max_passes - 8.0_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_max_passes() {
        let mut cfg = default_pass_manager();
        set_pass_manager_max_passes(&mut cfg, 0.7);
        assert!((cfg.max_passes - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_prepass() {
        let mut cfg = default_pass_manager();
        set_pass_manager_depth_prepass(&mut cfg, 0.8);
        assert!((cfg.depth_prepass - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_shadow_pass() {
        let mut cfg = default_pass_manager();
        set_pass_manager_shadow_pass(&mut cfg, 0.6);
        assert!((cfg.shadow_pass - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_post_pass() {
        let mut cfg = default_pass_manager();
        set_pass_manager_post_pass(&mut cfg, 0.5);
        assert!((cfg.post_pass - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_overlay_pass() {
        let mut cfg = default_pass_manager();
        set_pass_manager_overlay_pass(&mut cfg, 0.4);
        assert!((cfg.overlay_pass - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_pass_manager();
        let w = pass_manager_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_pass_manager();
        let mut b = default_pass_manager();
        b.max_passes = 1.0;
        let mid = blend_pass_manager(&a, &b, 0.5);
        assert!((mid.max_passes - 4.5_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_pass_manager();
        let b = default_pass_manager();
        let r = blend_pass_manager(&a, &b, 0.0);
        assert!((r.max_passes - a.max_passes).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_pass_manager();
        let b = default_pass_manager();
        let r = blend_pass_manager(&a, &b, 1.0);
        assert!((r.max_passes - b.max_passes).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_pass_manager();
        let b = default_pass_manager();
        let r = blend_pass_manager(&a, &b, 2.0);
        assert!((r.max_passes - b.max_passes).abs() < 1e-6);
    }
}
