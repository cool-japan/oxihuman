// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ragdoll body debug view — visualises per-bone rigid bodies and joints.

/// Ragdoll debug view configuration.
#[derive(Debug, Clone)]
pub struct RagdollDebugView {
    pub enabled: bool,
    pub show_bodies: bool,
    pub show_joints: bool,
    pub body_color: [f32; 4],
    pub joint_color: [f32; 4],
    pub joint_radius: f32,
}

impl RagdollDebugView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_bodies: true,
            show_joints: true,
            body_color: [0.2, 0.6, 1.0, 0.5],
            joint_color: [1.0, 0.8, 0.0, 0.9],
            joint_radius: 0.015,
        }
    }
}

impl Default for RagdollDebugView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new ragdoll debug view.
pub fn new_ragdoll_debug_view() -> RagdollDebugView {
    RagdollDebugView::new()
}

/// Enable or disable ragdoll debug view.
pub fn rdv_set_enabled(v: &mut RagdollDebugView, enabled: bool) {
    v.enabled = enabled;
}

/// Toggle rigid body box display.
pub fn rdv_set_show_bodies(v: &mut RagdollDebugView, show: bool) {
    v.show_bodies = show;
}

/// Toggle joint pivot display.
pub fn rdv_set_show_joints(v: &mut RagdollDebugView, show: bool) {
    v.show_joints = show;
}

/// Set joint sphere radius.
pub fn rdv_set_joint_radius(v: &mut RagdollDebugView, r: f32) {
    v.joint_radius = r.clamp(0.001, 0.5);
}

/// Set body wireframe colour.
pub fn rdv_set_body_color(v: &mut RagdollDebugView, color: [f32; 4]) {
    v.body_color = color;
}

/// Serialize to JSON-like string.
pub fn ragdoll_debug_view_to_json(v: &RagdollDebugView) -> String {
    format!(
        r#"{{"enabled":{},"show_bodies":{},"show_joints":{},"joint_radius":{:.4}}}"#,
        v.enabled, v.show_bodies, v.show_joints, v.joint_radius
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_ragdoll_debug_view();
        assert!(!v.enabled);
        assert!(v.show_bodies);
        assert!(v.show_joints);
    }

    #[test]
    fn test_enable() {
        let mut v = new_ragdoll_debug_view();
        rdv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_bodies_toggle() {
        let mut v = new_ragdoll_debug_view();
        rdv_set_show_bodies(&mut v, false);
        assert!(!v.show_bodies);
    }

    #[test]
    fn test_joints_toggle() {
        let mut v = new_ragdoll_debug_view();
        rdv_set_show_joints(&mut v, false);
        assert!(!v.show_joints);
    }

    #[test]
    fn test_joint_radius_clamp() {
        let mut v = new_ragdoll_debug_view();
        rdv_set_joint_radius(&mut v, 0.0);
        assert_eq!(v.joint_radius, 0.001);
    }

    #[test]
    fn test_joint_radius_set() {
        let mut v = new_ragdoll_debug_view();
        rdv_set_joint_radius(&mut v, 0.05);
        assert!((v.joint_radius - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_body_color() {
        let mut v = new_ragdoll_debug_view();
        rdv_set_body_color(&mut v, [1.0, 0.0, 0.0, 1.0]);
        assert!((v.body_color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let v = new_ragdoll_debug_view();
        let s = ragdoll_debug_view_to_json(&v);
        assert!(s.contains("joint_radius"));
    }

    #[test]
    fn test_clone() {
        let v = new_ragdoll_debug_view();
        let v2 = v.clone();
        assert!((v2.joint_radius - v.joint_radius).abs() < 1e-6);
    }
}
