// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Collision sphere primitive export for physics rigs.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionSphere {
    pub name: String,
    pub center: [f32; 3],
    pub radius: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionSphereExport {
    pub spheres: Vec<CollisionSphere>,
}

#[allow(dead_code)]
pub fn new_collision_sphere_export() -> CollisionSphereExport {
    CollisionSphereExport {
        spheres: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_sphere(exp: &mut CollisionSphereExport, name: &str, center: [f32; 3], radius: f32) {
    exp.spheres.push(CollisionSphere {
        name: name.to_string(),
        center,
        radius,
    });
}

#[allow(dead_code)]
pub fn sphere_count(exp: &CollisionSphereExport) -> usize {
    exp.spheres.len()
}

#[allow(dead_code)]
pub fn sphere_volume(s: &CollisionSphere) -> f32 {
    (4.0 / 3.0) * PI * s.radius * s.radius * s.radius
}

#[allow(dead_code)]
pub fn sphere_surface_area(s: &CollisionSphere) -> f32 {
    4.0 * PI * s.radius * s.radius
}

#[allow(dead_code)]
pub fn total_sphere_volume(exp: &CollisionSphereExport) -> f32 {
    exp.spheres.iter().map(sphere_volume).sum()
}

#[allow(dead_code)]
pub fn find_sphere<'a>(exp: &'a CollisionSphereExport, name: &str) -> Option<&'a CollisionSphere> {
    exp.spheres.iter().find(|s| s.name == name)
}

#[allow(dead_code)]
pub fn point_in_sphere(s: &CollisionSphere, p: [f32; 3]) -> bool {
    let d = [p[0] - s.center[0], p[1] - s.center[1], p[2] - s.center[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= s.radius * s.radius
}

#[allow(dead_code)]
pub fn collision_sphere_to_json(exp: &CollisionSphereExport) -> String {
    format!(
        "{{\"sphere_count\":{},\"total_volume\":{}}}",
        sphere_count(exp),
        total_sphere_volume(exp)
    )
}

#[allow(dead_code)]
pub fn avg_sphere_radius(exp: &CollisionSphereExport) -> f32 {
    if exp.spheres.is_empty() {
        return 0.0;
    }
    exp.spheres.iter().map(|s| s.radius).sum::<f32>() / exp.spheres.len() as f32
}

#[allow(dead_code)]
pub fn validate_spheres(exp: &CollisionSphereExport) -> bool {
    exp.spheres.iter().all(|s| s.radius > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let exp = new_collision_sphere_export();
        assert_eq!(sphere_count(&exp), 0);
    }

    #[test]
    fn test_add_sphere() {
        let mut exp = new_collision_sphere_export();
        add_sphere(&mut exp, "head", [0.0, 1.7, 0.0], 0.1);
        assert_eq!(sphere_count(&exp), 1);
    }

    #[test]
    fn test_sphere_volume() {
        let s = CollisionSphere {
            name: "x".to_string(),
            center: [0.0; 3],
            radius: 1.0,
        };
        let v = sphere_volume(&s);
        assert!((v - 4.0 / 3.0 * PI).abs() < 1e-4);
    }

    #[test]
    fn test_point_in_sphere_true() {
        let s = CollisionSphere {
            name: "s".to_string(),
            center: [0.0; 3],
            radius: 1.0,
        };
        assert!(point_in_sphere(&s, [0.5, 0.0, 0.0]));
    }

    #[test]
    fn test_point_in_sphere_false() {
        let s = CollisionSphere {
            name: "s".to_string(),
            center: [0.0; 3],
            radius: 0.5,
        };
        assert!(!point_in_sphere(&s, [1.0, 0.0, 0.0]));
    }

    #[test]
    fn test_find_sphere() {
        let mut exp = new_collision_sphere_export();
        add_sphere(&mut exp, "knee", [0.0, 0.5, 0.0], 0.05);
        assert!(find_sphere(&exp, "knee").is_some());
    }

    #[test]
    fn test_avg_radius() {
        let mut exp = new_collision_sphere_export();
        add_sphere(&mut exp, "a", [0.0; 3], 1.0);
        add_sphere(&mut exp, "b", [0.0; 3], 3.0);
        assert!((avg_sphere_radius(&exp) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_json_output() {
        let exp = new_collision_sphere_export();
        let j = collision_sphere_to_json(&exp);
        assert!(j.contains("sphere_count"));
    }

    #[test]
    fn test_validate() {
        let mut exp = new_collision_sphere_export();
        add_sphere(&mut exp, "ok", [0.0; 3], 0.1);
        assert!(validate_spheres(&exp));
    }

    #[test]
    fn test_surface_area() {
        let s = CollisionSphere {
            name: "x".to_string(),
            center: [0.0; 3],
            radius: 1.0,
        };
        assert!((sphere_surface_area(&s) - 4.0 * PI).abs() < 1e-4);
    }
}
