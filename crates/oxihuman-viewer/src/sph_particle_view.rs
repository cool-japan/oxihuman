// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! SPH particle visualization — renders smoothed-particle hydrodynamics particles.

/// SPH particle view configuration.
#[derive(Debug, Clone)]
pub struct SphParticleView {
    pub enabled: bool,
    pub particle_radius: f32,
    pub color_by_speed: bool,
    pub max_speed: f32,
    pub opacity: f32,
}

impl SphParticleView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            particle_radius: 2.0,
            color_by_speed: true,
            max_speed: 5.0,
            opacity: 0.8,
        }
    }
}

impl Default for SphParticleView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new SPH particle view.
pub fn new_sph_particle_view() -> SphParticleView {
    SphParticleView::new()
}

/// Enable or disable SPH particle display.
pub fn sph_set_enabled(v: &mut SphParticleView, enabled: bool) {
    v.enabled = enabled;
}

/// Set particle glyph radius in pixels.
pub fn sph_set_particle_radius(v: &mut SphParticleView, r: f32) {
    v.particle_radius = r.clamp(0.5, 20.0);
}

/// Enable speed-based color mapping.
pub fn sph_set_color_by_speed(v: &mut SphParticleView, enabled: bool) {
    v.color_by_speed = enabled;
}

/// Set maximum speed for color normalization.
pub fn sph_set_max_speed(v: &mut SphParticleView, s: f32) {
    v.max_speed = s.max(0.001);
}

/// Set particle opacity.
pub fn sph_set_opacity(v: &mut SphParticleView, o: f32) {
    v.opacity = o.clamp(0.0, 1.0);
}

/// Normalize a speed value to 0-1 for color mapping.
pub fn sph_normalize_speed(v: &SphParticleView, speed: f32) -> f32 {
    (speed / v.max_speed).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn sph_particle_view_to_json(v: &SphParticleView) -> String {
    format!(
        r#"{{"enabled":{},"particle_radius":{:.4},"color_by_speed":{},"max_speed":{:.4},"opacity":{:.4}}}"#,
        v.enabled, v.particle_radius, v.color_by_speed, v.max_speed, v.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_sph_particle_view();
        assert!(!v.enabled);
        assert!(v.color_by_speed);
    }

    #[test]
    fn test_enable() {
        let mut v = new_sph_particle_view();
        sph_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_radius_clamp_low() {
        let mut v = new_sph_particle_view();
        sph_set_particle_radius(&mut v, 0.0);
        assert_eq!(v.particle_radius, 0.5);
    }

    #[test]
    fn test_radius_set() {
        let mut v = new_sph_particle_view();
        sph_set_particle_radius(&mut v, 5.0);
        assert!((v.particle_radius - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_color_by_speed_off() {
        let mut v = new_sph_particle_view();
        sph_set_color_by_speed(&mut v, false);
        assert!(!v.color_by_speed);
    }

    #[test]
    fn test_max_speed_set() {
        let mut v = new_sph_particle_view();
        sph_set_max_speed(&mut v, 10.0);
        assert!((v.max_speed - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_speed_half() {
        let mut v = new_sph_particle_view();
        sph_set_max_speed(&mut v, 10.0);
        assert!((sph_normalize_speed(&v, 5.0) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_json_keys() {
        let v = new_sph_particle_view();
        let s = sph_particle_view_to_json(&v);
        assert!(s.contains("color_by_speed"));
    }

    #[test]
    fn test_clone() {
        let v = new_sph_particle_view();
        let v2 = v.clone();
        assert!((v2.opacity - v.opacity).abs() < 1e-6);
    }
}
