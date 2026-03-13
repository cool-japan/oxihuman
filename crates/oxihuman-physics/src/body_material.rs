#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Physics body material properties.

/// Material properties for a physics body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyMaterial {
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
    pub name: String,
}

#[allow(dead_code)]
pub fn new_body_material(friction: f32, restitution: f32, density: f32, name: &str) -> BodyMaterial {
    BodyMaterial {
        friction,
        restitution,
        density,
        name: name.to_string(),
    }
}

#[allow(dead_code)]
pub fn default_body_material() -> BodyMaterial {
    new_body_material(0.5, 0.3, 1000.0, "default")
}

#[allow(dead_code)]
pub fn steel_material() -> BodyMaterial {
    new_body_material(0.4, 0.1, 7800.0, "steel")
}

#[allow(dead_code)]
pub fn rubber_body_material() -> BodyMaterial {
    new_body_material(0.9, 0.8, 1100.0, "rubber")
}

#[allow(dead_code)]
pub fn ice_body_material() -> BodyMaterial {
    new_body_material(0.05, 0.1, 917.0, "ice")
}

#[allow(dead_code)]
pub fn combine_friction(a: &BodyMaterial, b: &BodyMaterial) -> f32 {
    (a.friction * b.friction).sqrt()
}

#[allow(dead_code)]
pub fn combine_restitution_bodies(a: &BodyMaterial, b: &BodyMaterial) -> f32 {
    a.restitution.max(b.restitution)
}

#[allow(dead_code)]
pub fn material_density(m: &BodyMaterial) -> f32 {
    m.density
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let m = default_body_material();
        assert!((m.friction - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_steel() {
        let m = steel_material();
        assert!((m.density - 7800.0).abs() < 1e-6);
    }

    #[test]
    fn test_rubber() {
        let m = rubber_body_material();
        assert!(m.friction > 0.8);
    }

    #[test]
    fn test_ice() {
        let m = ice_body_material();
        assert!(m.friction < 0.1);
    }

    #[test]
    fn test_combine_friction() {
        let a = new_body_material(0.4, 0.0, 1000.0, "a");
        let b = new_body_material(0.9, 0.0, 1000.0, "b");
        let f = combine_friction(&a, &b);
        assert!((f - (0.36f32).sqrt()).abs() < 1e-4);
    }

    #[test]
    fn test_combine_restitution() {
        let a = new_body_material(0.0, 0.2, 1000.0, "a");
        let b = new_body_material(0.0, 0.8, 1000.0, "b");
        assert!((combine_restitution_bodies(&a, &b) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_density() {
        let m = new_body_material(0.5, 0.5, 2000.0, "heavy");
        assert!((material_density(&m) - 2000.0).abs() < 1e-6);
    }

    #[test]
    fn test_name() {
        let m = new_body_material(0.5, 0.5, 1000.0, "test_mat");
        assert_eq!(m.name, "test_mat");
    }

    #[test]
    fn test_custom() {
        let m = new_body_material(0.1, 0.9, 500.0, "custom");
        assert!((m.restitution - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_combine_same_friction() {
        let m = default_body_material();
        let f = combine_friction(&m, &m);
        assert!((f - m.friction).abs() < 1e-4);
    }
}
