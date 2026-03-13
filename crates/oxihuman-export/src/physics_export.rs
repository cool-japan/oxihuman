// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export physics simulation data to JSON-compatible format.

#![allow(dead_code)]

/// Physics object export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsObjectExport {
    pub name: String,
    pub object_type: u8,
    pub mass: f32,
    pub restitution: f32,
    pub friction: f32,
    pub is_static: bool,
}

/// Full physics scene export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsExport {
    pub objects: Vec<PhysicsObjectExport>,
    pub gravity: [f32; 3],
}

/// Create a new physics export with default gravity.
#[allow(dead_code)]
pub fn new_physics_export(gravity: [f32; 3]) -> PhysicsExport {
    PhysicsExport {
        objects: Vec::new(),
        gravity,
    }
}

/// Add a physics object to the export.
#[allow(dead_code)]
pub fn add_physics_object(exp: &mut PhysicsExport, name: &str, type_: u8, mass: f32) {
    exp.objects.push(PhysicsObjectExport {
        name: name.to_string(),
        object_type: type_,
        mass,
        restitution: 0.3,
        friction: 0.5,
        is_static: mass <= 0.0,
    });
}

/// Serialize physics export to a JSON string.
#[allow(dead_code)]
pub fn export_physics_to_json(exp: &PhysicsExport) -> String {
    let objs: Vec<String> = exp
        .objects
        .iter()
        .map(|o| {
            format!(
                r#"{{"name":"{n}","object_type":{t},"mass":{m},"restitution":{r},"friction":{f},"is_static":{s}}}"#,
                n = o.name,
                t = o.object_type,
                m = o.mass,
                r = o.restitution,
                f = o.friction,
                s = o.is_static,
            )
        })
        .collect();
    format!(
        r#"{{"gravity":[{gx},{gy},{gz}],"objects":[{objects}]}}"#,
        gx = exp.gravity[0],
        gy = exp.gravity[1],
        gz = exp.gravity[2],
        objects = objs.join(","),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_physics_gravity() {
        let exp = new_physics_export([0.0, -9.81, 0.0]);
        assert!((exp.gravity[1] + 9.81).abs() < 1e-4);
    }

    #[test]
    fn test_new_physics_empty() {
        let exp = new_physics_export([0.0, -9.81, 0.0]);
        assert!(exp.objects.is_empty());
    }

    #[test]
    fn test_add_physics_object() {
        let mut exp = new_physics_export([0.0, -9.81, 0.0]);
        add_physics_object(&mut exp, "box", 0, 1.0);
        assert_eq!(exp.objects.len(), 1);
    }

    #[test]
    fn test_zero_mass_is_static() {
        let mut exp = new_physics_export([0.0, -9.81, 0.0]);
        add_physics_object(&mut exp, "floor", 0, 0.0);
        assert!(exp.objects[0].is_static);
    }

    #[test]
    fn test_nonzero_mass_not_static() {
        let mut exp = new_physics_export([0.0, -9.81, 0.0]);
        add_physics_object(&mut exp, "ball", 0, 1.5);
        assert!(!exp.objects[0].is_static);
    }

    #[test]
    fn test_json_contains_gravity() {
        let exp = new_physics_export([0.0, -9.81, 0.0]);
        let json = export_physics_to_json(&exp);
        assert!(json.contains("gravity"));
    }

    #[test]
    fn test_json_contains_object_name() {
        let mut exp = new_physics_export([0.0, -9.81, 0.0]);
        add_physics_object(&mut exp, "my_rigidbody", 0, 2.0);
        let json = export_physics_to_json(&exp);
        assert!(json.contains("my_rigidbody"));
    }

    #[test]
    fn test_add_multiple_objects() {
        let mut exp = new_physics_export([0.0, -9.81, 0.0]);
        add_physics_object(&mut exp, "obj1", 0, 1.0);
        add_physics_object(&mut exp, "obj2", 1, 2.0);
        assert_eq!(exp.objects.len(), 2);
    }

    #[test]
    fn test_default_restitution() {
        let mut exp = new_physics_export([0.0, -9.81, 0.0]);
        add_physics_object(&mut exp, "o", 0, 1.0);
        assert!((exp.objects[0].restitution - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_default_friction() {
        let mut exp = new_physics_export([0.0, -9.81, 0.0]);
        add_physics_object(&mut exp, "o", 0, 1.0);
        assert!((exp.objects[0].friction - 0.5).abs() < 1e-5);
    }
}
