// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export softbody simulation settings to JSON-compatible format.

#![allow(dead_code)]

/// Softbody simulation export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SoftbodyExport {
    pub name: String,
    pub mass: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub friction: f32,
    pub collision_margin: f32,
}

/// Create a default softbody export.
#[allow(dead_code)]
pub fn default_softbody_export(name: &str) -> SoftbodyExport {
    SoftbodyExport {
        name: name.to_string(),
        mass: 1.0,
        stiffness: 0.9,
        damping: 0.01,
        friction: 0.5,
        collision_margin: 0.04,
    }
}

/// Serialize softbody export to a JSON string.
#[allow(dead_code)]
pub fn export_softbody_to_json(exp: &SoftbodyExport) -> String {
    format!(
        r#"{{"name":"{name}","mass":{mass},"stiffness":{stiff},"damping":{damp},"friction":{fric},"collision_margin":{cm}}}"#,
        name = exp.name,
        mass = exp.mass,
        stiff = exp.stiffness,
        damp = exp.damping,
        fric = exp.friction,
        cm = exp.collision_margin,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_name() {
        let exp = default_softbody_export("jelly");
        assert_eq!(exp.name, "jelly");
    }

    #[test]
    fn test_default_mass_positive() {
        let exp = default_softbody_export("s");
        assert!(exp.mass > 0.0);
    }

    #[test]
    fn test_default_stiffness_range() {
        let exp = default_softbody_export("s");
        assert!((0.0..=1.0).contains(&exp.stiffness));
    }

    #[test]
    fn test_default_damping_positive() {
        let exp = default_softbody_export("s");
        assert!(exp.damping > 0.0);
    }

    #[test]
    fn test_default_friction_positive() {
        let exp = default_softbody_export("s");
        assert!(exp.friction > 0.0);
    }

    #[test]
    fn test_collision_margin_positive() {
        let exp = default_softbody_export("s");
        assert!(exp.collision_margin > 0.0);
    }

    #[test]
    fn test_json_contains_name() {
        let exp = default_softbody_export("rubber");
        let json = export_softbody_to_json(&exp);
        assert!(json.contains("rubber"));
    }

    #[test]
    fn test_json_contains_stiffness() {
        let exp = default_softbody_export("s");
        let json = export_softbody_to_json(&exp);
        assert!(json.contains("stiffness"));
    }

    #[test]
    fn test_json_contains_mass() {
        let exp = default_softbody_export("s");
        let json = export_softbody_to_json(&exp);
        assert!(json.contains("mass"));
    }

    #[test]
    fn test_json_nonempty() {
        let exp = default_softbody_export("s");
        let json = export_softbody_to_json(&exp);
        assert!(!json.is_empty());
    }
}
