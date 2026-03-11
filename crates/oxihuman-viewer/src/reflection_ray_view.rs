// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ray-traced reflection debug view stub.

/// Reflection ray visualization config.
#[derive(Debug, Clone)]
pub struct ReflectionRayViewConfig {
    pub max_bounces: usize,
    pub intensity: f32,
    pub enabled: bool,
    pub show_miss: bool,
}

impl Default for ReflectionRayViewConfig {
    fn default() -> Self {
        ReflectionRayViewConfig {
            max_bounces: 2,
            intensity: 1.0,
            enabled: true,
            show_miss: true,
        }
    }
}

/// Create a new reflection ray view config.
pub fn new_reflection_ray_view() -> ReflectionRayViewConfig {
    ReflectionRayViewConfig::default()
}

/// Set max bounces.
pub fn rrv_set_max_bounces(cfg: &mut ReflectionRayViewConfig, bounces: usize) {
    cfg.max_bounces = bounces;
}

/// Set intensity.
pub fn rrv_set_intensity(cfg: &mut ReflectionRayViewConfig, intensity: f32) {
    cfg.intensity = intensity.max(0.0);
}

/// Enable or disable.
pub fn rrv_set_enabled(cfg: &mut ReflectionRayViewConfig, enabled: bool) {
    cfg.enabled = enabled;
}

/// Toggle miss ray display.
pub fn rrv_toggle_miss(cfg: &mut ReflectionRayViewConfig) {
    cfg.show_miss = !cfg.show_miss;
}

/// Compute reflected direction (stub: mirrors around normal).
pub fn rrv_reflect(incident: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    let dot = incident[0] * normal[0] + incident[1] * normal[1] + incident[2] * normal[2];
    [
        incident[0] - 2.0 * dot * normal[0],
        incident[1] - 2.0 * dot * normal[1],
        incident[2] - 2.0 * dot * normal[2],
    ]
}

/// Return a JSON-like string.
pub fn rrv_to_json(cfg: &ReflectionRayViewConfig) -> String {
    format!(
        r#"{{"max_bounces":{},"intensity":{:.4},"enabled":{}}}"#,
        cfg.max_bounces, cfg.intensity, cfg.enabled
    )
}

/// Return miss color.
pub fn rrv_miss_color() -> [f32; 3] {
    [0.1, 0.1, 0.2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_max_bounces() {
        let c = new_reflection_ray_view();
        assert_eq!(c.max_bounces, 2 /* default max bounces is 2 */,);
    }

    #[test]
    fn test_set_max_bounces() {
        let mut c = new_reflection_ray_view();
        rrv_set_max_bounces(&mut c, 5);
        assert_eq!(c.max_bounces, 5 /* max bounces must match */,);
    }

    #[test]
    fn test_set_intensity() {
        let mut c = new_reflection_ray_view();
        rrv_set_intensity(&mut c, 1.5);
        assert!((c.intensity - 1.5).abs() < 1e-5, /* intensity must match */);
    }

    #[test]
    fn test_set_intensity_negative_clamps() {
        let mut c = new_reflection_ray_view();
        rrv_set_intensity(&mut c, -1.0);
        assert!((c.intensity).abs() < 1e-6, /* negative intensity clamped to 0 */);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_reflection_ray_view();
        rrv_set_enabled(&mut c, false);
        assert!(!c.enabled /* should be disabled */,);
    }

    #[test]
    fn test_toggle_miss() {
        let mut c = new_reflection_ray_view();
        let before = c.show_miss;
        rrv_toggle_miss(&mut c);
        assert_ne!(
            c.show_miss,
            before, /* miss toggle should change state */
        );
    }

    #[test]
    fn test_reflect_normal_incidence() {
        /* Incident along -Z, normal along +Z -> reflected along +Z */
        let r = rrv_reflect([0.0, 0.0, -1.0], [0.0, 0.0, 1.0]);
        assert!((r[2] - 1.0).abs() < 1e-5, /* reflected z component should be 1 */);
    }

    #[test]
    fn test_to_json_contains_bounces() {
        let c = new_reflection_ray_view();
        let j = rrv_to_json(&c);
        assert!(j.contains("max_bounces"), /* JSON must contain max_bounces */);
    }

    #[test]
    fn test_miss_color_returns_three() {
        let m = rrv_miss_color();
        assert_eq!(m.len(), 3 /* miss color must be RGB */,);
    }

    #[test]
    fn test_default_show_miss_true() {
        let c = new_reflection_ray_view();
        assert!(c.show_miss /* show_miss enabled by default */,);
    }
}
