// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ray-traced shadow debug view stub.

/// Shadow ray visualization config.
#[derive(Debug, Clone)]
pub struct ShadowRayViewConfig {
    pub max_rays: usize,
    pub shadow_tint: [f32; 3],
    pub lit_tint: [f32; 3],
    pub enabled: bool,
}

impl Default for ShadowRayViewConfig {
    fn default() -> Self {
        ShadowRayViewConfig {
            max_rays: 64,
            shadow_tint: [0.2, 0.2, 0.4],
            lit_tint: [1.0, 1.0, 0.8],
            enabled: true,
        }
    }
}

/// Create a new shadow ray view config.
pub fn new_shadow_ray_view() -> ShadowRayViewConfig {
    ShadowRayViewConfig::default()
}

/// Set max rays.
pub fn srv_set_max_rays(cfg: &mut ShadowRayViewConfig, max: usize) {
    cfg.max_rays = max.max(1);
}

/// Set shadow tint.
pub fn srv_set_shadow_tint(cfg: &mut ShadowRayViewConfig, tint: [f32; 3]) {
    cfg.shadow_tint = tint;
}

/// Enable or disable.
pub fn srv_set_enabled(cfg: &mut ShadowRayViewConfig, enabled: bool) {
    cfg.enabled = enabled;
}

/// Compute the pixel color for a shadowed/lit pixel.
pub fn srv_pixel_color(cfg: &ShadowRayViewConfig, in_shadow: bool) -> [f32; 3] {
    if in_shadow {
        cfg.shadow_tint
    } else {
        cfg.lit_tint
    }
}

/// Return a JSON-like string.
pub fn srv_to_json(cfg: &ShadowRayViewConfig) -> String {
    format!(
        r#"{{"max_rays":{},"enabled":{}}}"#,
        cfg.max_rays, cfg.enabled
    )
}

/// Return a color mixing factor based on shadow coverage (0-1).
pub fn srv_coverage_factor(coverage: f32) -> f32 {
    coverage.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_max_rays() {
        let c = new_shadow_ray_view();
        assert_eq!(c.max_rays, 64 /* default max rays is 64 */,);
    }

    #[test]
    fn test_set_max_rays() {
        let mut c = new_shadow_ray_view();
        srv_set_max_rays(&mut c, 128);
        assert_eq!(c.max_rays, 128 /* max rays must match */,);
    }

    #[test]
    fn test_set_max_rays_minimum() {
        let mut c = new_shadow_ray_view();
        srv_set_max_rays(&mut c, 0);
        assert!(c.max_rays >= 1 /* max rays must be at least 1 */,);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_shadow_ray_view();
        srv_set_enabled(&mut c, false);
        assert!(!c.enabled /* should be disabled */,);
    }

    #[test]
    fn test_pixel_color_shadow() {
        let c = new_shadow_ray_view();
        let color = srv_pixel_color(&c, true);
        assert!((color[0] - 0.2).abs() < 1e-5, /* shadow tint r component */);
    }

    #[test]
    fn test_pixel_color_lit() {
        let c = new_shadow_ray_view();
        let color = srv_pixel_color(&c, false);
        assert!((color[0] - 1.0).abs() < 1e-5, /* lit tint r component */);
    }

    #[test]
    fn test_coverage_factor_clamps() {
        assert!((srv_coverage_factor(2.0) - 1.0).abs() < 1e-5, /* clamped to 1 */);
        assert!((srv_coverage_factor(-1.0)).abs() < 1e-6, /* clamped to 0 */);
    }

    #[test]
    fn test_to_json_contains_max_rays() {
        let c = new_shadow_ray_view();
        let j = srv_to_json(&c);
        assert!(j.contains("max_rays") /* JSON must contain max_rays */,);
    }

    #[test]
    fn test_set_shadow_tint() {
        let mut c = new_shadow_ray_view();
        srv_set_shadow_tint(&mut c, [0.5, 0.5, 0.5]);
        assert!((c.shadow_tint[0] - 0.5).abs() < 1e-5, /* tint must be updated */);
    }

    #[test]
    fn test_default_enabled() {
        let c = new_shadow_ray_view();
        assert!(c.enabled /* enabled by default */,);
    }
}
