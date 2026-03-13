// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cloth simulation debug view — shows particle masses, constraints, and velocities.

/// Cloth debug view configuration.
#[derive(Debug, Clone)]
pub struct ClothDebugView {
    pub enabled: bool,
    pub show_particles: bool,
    pub show_constraints: bool,
    pub show_velocities: bool,
    pub particle_size: f32,
    pub velocity_scale: f32,
}

impl ClothDebugView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_particles: true,
            show_constraints: true,
            show_velocities: false,
            particle_size: 3.0,
            velocity_scale: 0.05,
        }
    }
}

impl Default for ClothDebugView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new cloth debug view.
pub fn new_cloth_debug_view() -> ClothDebugView {
    ClothDebugView::new()
}

/// Enable or disable cloth debug display.
pub fn cldv_set_enabled(v: &mut ClothDebugView, enabled: bool) {
    v.enabled = enabled;
}

/// Toggle particle glyph display.
pub fn cldv_set_show_particles(v: &mut ClothDebugView, show: bool) {
    v.show_particles = show;
}

/// Toggle constraint edge display.
pub fn cldv_set_show_constraints(v: &mut ClothDebugView, show: bool) {
    v.show_constraints = show;
}

/// Toggle velocity arrow display.
pub fn cldv_set_show_velocities(v: &mut ClothDebugView, show: bool) {
    v.show_velocities = show;
}

/// Set particle glyph size in pixels.
pub fn cldv_set_particle_size(v: &mut ClothDebugView, size: f32) {
    v.particle_size = size.clamp(1.0, 16.0);
}

/// Set velocity arrow scale.
pub fn cldv_set_velocity_scale(v: &mut ClothDebugView, scale: f32) {
    v.velocity_scale = scale.clamp(0.0001, 1.0);
}

/// Serialize to JSON-like string.
pub fn cloth_debug_view_to_json(v: &ClothDebugView) -> String {
    format!(
        r#"{{"enabled":{},"show_particles":{},"show_constraints":{},"show_velocities":{},"particle_size":{:.4},"velocity_scale":{:.6}}}"#,
        v.enabled,
        v.show_particles,
        v.show_constraints,
        v.show_velocities,
        v.particle_size,
        v.velocity_scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_cloth_debug_view();
        assert!(!v.enabled);
        assert!(v.show_particles);
        assert!(!v.show_velocities);
    }

    #[test]
    fn test_enable() {
        let mut v = new_cloth_debug_view();
        cldv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_particles_toggle() {
        let mut v = new_cloth_debug_view();
        cldv_set_show_particles(&mut v, false);
        assert!(!v.show_particles);
    }

    #[test]
    fn test_constraints_toggle() {
        let mut v = new_cloth_debug_view();
        cldv_set_show_constraints(&mut v, false);
        assert!(!v.show_constraints);
    }

    #[test]
    fn test_velocities_toggle() {
        let mut v = new_cloth_debug_view();
        cldv_set_show_velocities(&mut v, true);
        assert!(v.show_velocities);
    }

    #[test]
    fn test_particle_size_clamp_min() {
        let mut v = new_cloth_debug_view();
        cldv_set_particle_size(&mut v, 0.0);
        assert_eq!(v.particle_size, 1.0);
    }

    #[test]
    fn test_particle_size_set() {
        let mut v = new_cloth_debug_view();
        cldv_set_particle_size(&mut v, 8.0);
        assert!((v.particle_size - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let v = new_cloth_debug_view();
        let s = cloth_debug_view_to_json(&v);
        assert!(s.contains("velocity_scale"));
    }

    #[test]
    fn test_clone() {
        let v = new_cloth_debug_view();
        let v2 = v.clone();
        assert!((v2.velocity_scale - v.velocity_scale).abs() < 1e-6);
    }
}
