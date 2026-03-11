// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Physics simulation debug overlay — master toggle for all physics debug layers.

/// Physics debug view configuration.
#[derive(Debug, Clone)]
pub struct PhysicsDebugView {
    pub enabled: bool,
    pub show_colliders: bool,
    pub show_forces: bool,
    pub show_joints: bool,
    pub opacity: f32,
}

impl PhysicsDebugView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_colliders: true,
            show_forces: true,
            show_joints: true,
            opacity: 0.8,
        }
    }
}

impl Default for PhysicsDebugView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new physics debug view.
pub fn new_physics_debug_view() -> PhysicsDebugView {
    PhysicsDebugView::new()
}

/// Enable or disable the physics debug overlay.
pub fn pdv_set_enabled(v: &mut PhysicsDebugView, enabled: bool) {
    v.enabled = enabled;
}

/// Toggle collider wireframes.
pub fn pdv_set_show_colliders(v: &mut PhysicsDebugView, show: bool) {
    v.show_colliders = show;
}

/// Toggle force vector display.
pub fn pdv_set_show_forces(v: &mut PhysicsDebugView, show: bool) {
    v.show_forces = show;
}

/// Set overlay opacity.
pub fn pdv_set_opacity(v: &mut PhysicsDebugView, opacity: f32) {
    v.opacity = opacity.clamp(0.0, 1.0);
}

/// Serialize to JSON-like string.
pub fn physics_debug_view_to_json(v: &PhysicsDebugView) -> String {
    format!(
        r#"{{"enabled":{},"show_colliders":{},"show_forces":{},"opacity":{:.4}}}"#,
        v.enabled, v.show_colliders, v.show_forces, v.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_physics_debug_view();
        assert!(!v.enabled);
        assert!(v.show_colliders);
    }

    #[test]
    fn test_enable() {
        let mut v = new_physics_debug_view();
        pdv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_disable_colliders() {
        let mut v = new_physics_debug_view();
        pdv_set_show_colliders(&mut v, false);
        assert!(!v.show_colliders);
    }

    #[test]
    fn test_forces_toggle() {
        let mut v = new_physics_debug_view();
        pdv_set_show_forces(&mut v, false);
        assert!(!v.show_forces);
    }

    #[test]
    fn test_opacity_clamp() {
        let mut v = new_physics_debug_view();
        pdv_set_opacity(&mut v, 2.0);
        assert_eq!(v.opacity, 1.0);
    }

    #[test]
    fn test_opacity_set() {
        let mut v = new_physics_debug_view();
        pdv_set_opacity(&mut v, 0.5);
        assert!((v.opacity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let v = new_physics_debug_view();
        let s = physics_debug_view_to_json(&v);
        assert!(s.contains("show_colliders"));
    }

    #[test]
    fn test_clone() {
        let v = new_physics_debug_view();
        let v2 = v.clone();
        assert_eq!(v2.enabled, v.enabled);
    }

    #[test]
    fn test_default_opacity() {
        let v = new_physics_debug_view();
        assert!((v.opacity - 0.8).abs() < 1e-6);
    }
}
