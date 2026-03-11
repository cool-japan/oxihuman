// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Rigid body properties panel view.

/// Rigid body type.
#[derive(Debug, Clone, PartialEq)]
pub enum RigidBodyTypeView {
    Active,
    Passive,
}

/// Rigid body properties panel state.
#[derive(Debug, Clone)]
pub struct RigidBodyPropsView {
    pub rb_type: RigidBodyTypeView,
    pub mass: f32,
    pub friction: f32,
    pub restitution: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
}

impl Default for RigidBodyPropsView {
    fn default() -> Self {
        Self {
            rb_type: RigidBodyTypeView::Active,
            mass: 1.0,
            friction: 0.5,
            restitution: 0.0,
            linear_damping: 0.04,
            angular_damping: 0.1,
        }
    }
}

/// Create a new RigidBodyPropsView.
pub fn new_rigid_body_props_view() -> RigidBodyPropsView {
    RigidBodyPropsView::default()
}

/// Set rigid body mass.
pub fn rbpv_set_mass(view: &mut RigidBodyPropsView, mass: f32) {
    view.mass = mass.clamp(0.001, 1e6);
}

/// Set friction coefficient.
pub fn rbpv_set_friction(view: &mut RigidBodyPropsView, v: f32) {
    view.friction = v.clamp(0.0, 1.0);
}

/// Set restitution (bounciness).
pub fn rbpv_set_restitution(view: &mut RigidBodyPropsView, v: f32) {
    view.restitution = v.clamp(0.0, 1.0);
}

/// Set linear damping.
pub fn rbpv_set_linear_damping(view: &mut RigidBodyPropsView, v: f32) {
    view.linear_damping = v.clamp(0.0, 1.0);
}

/// Compute kinetic energy proxy (0.5 * mass * 1.0 normalized).
pub fn rbpv_kinetic_energy_proxy(view: &RigidBodyPropsView) -> f32 {
    0.5 * view.mass
}

/// Serialize to JSON.
pub fn rigid_body_props_to_json(view: &RigidBodyPropsView) -> String {
    format!(
        r#"{{"mass":{},"friction":{},"restitution":{},"lin_damping":{}}}"#,
        view.mass, view.friction, view.restitution, view.linear_damping,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = new_rigid_body_props_view();
        assert!((v.mass - 1.0).abs() < 1e-6 /* default mass */);
    }

    #[test]
    fn test_mass_clamp_low() {
        let mut v = new_rigid_body_props_view();
        rbpv_set_mass(&mut v, 0.0);
        assert!(v.mass > 0.0 /* above zero */);
    }

    #[test]
    fn test_friction_clamp() {
        let mut v = new_rigid_body_props_view();
        rbpv_set_friction(&mut v, 5.0);
        assert!((v.friction - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_restitution() {
        let mut v = new_rigid_body_props_view();
        rbpv_set_restitution(&mut v, 0.8);
        assert!((v.restitution - 0.8).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_linear_damping() {
        let mut v = new_rigid_body_props_view();
        rbpv_set_linear_damping(&mut v, 0.5);
        assert!((v.linear_damping - 0.5).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_type_default_active() {
        let v = new_rigid_body_props_view();
        assert_eq!(v.rb_type, RigidBodyTypeView::Active /* active */);
    }

    #[test]
    fn test_kinetic_energy_proxy() {
        let mut v = new_rigid_body_props_view();
        rbpv_set_mass(&mut v, 4.0);
        assert!((rbpv_kinetic_energy_proxy(&v) - 2.0).abs() < 1e-6 /* 0.5 * 4 */);
    }

    #[test]
    fn test_json_keys() {
        let v = new_rigid_body_props_view();
        let j = rigid_body_props_to_json(&v);
        assert!(j.contains("mass") /* key */);
    }

    #[test]
    fn test_clone() {
        let v = new_rigid_body_props_view();
        let c = v.clone();
        assert!((c.mass - v.mass).abs() < 1e-6 /* equal */);
    }
}
