// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Collision shape export for physics engines.

use std::f32::consts::PI;

/// Type of collision primitive.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CollisionShapeType {
    Sphere,
    Box,
    Capsule,
    ConvexHull,
    Mesh,
}

/// A collision shape descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionShapeExport {
    pub shape_type: CollisionShapeType,
    pub name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub params: [f32; 4],
}

/// Collection of collision shapes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionShapeBundle {
    pub shapes: Vec<CollisionShapeExport>,
}

/// Create a sphere collision shape.
#[allow(dead_code)]
pub fn sphere_collision(name: &str, center: [f32; 3], radius: f32) -> CollisionShapeExport {
    CollisionShapeExport {
        shape_type: CollisionShapeType::Sphere,
        name: name.to_string(),
        position: center,
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0; 3],
        params: [radius, 0.0, 0.0, 0.0],
    }
}

/// Create a box collision shape (half-extents).
#[allow(dead_code)]
pub fn box_collision(name: &str, center: [f32; 3], half_extents: [f32; 3]) -> CollisionShapeExport {
    CollisionShapeExport {
        shape_type: CollisionShapeType::Box,
        name: name.to_string(),
        position: center,
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0; 3],
        params: [half_extents[0], half_extents[1], half_extents[2], 0.0],
    }
}

/// New empty bundle.
#[allow(dead_code)]
pub fn new_collision_bundle() -> CollisionShapeBundle {
    CollisionShapeBundle { shapes: Vec::new() }
}

/// Add a shape to bundle.
#[allow(dead_code)]
pub fn add_collision_shape(bundle: &mut CollisionShapeBundle, shape: CollisionShapeExport) {
    bundle.shapes.push(shape);
}

/// Shape count.
#[allow(dead_code)]
pub fn collision_shape_count(bundle: &CollisionShapeBundle) -> usize {
    bundle.shapes.len()
}

/// Volume estimate for a shape (sphere or box).
#[allow(dead_code)]
pub fn shape_volume(shape: &CollisionShapeExport) -> f32 {
    match shape.shape_type {
        CollisionShapeType::Sphere => (4.0 / 3.0) * PI * shape.params[0].powi(3),
        CollisionShapeType::Box => 8.0 * shape.params[0] * shape.params[1] * shape.params[2],
        _ => 0.0,
    }
}

/// Validate bundle: all shape names non-empty.
#[allow(dead_code)]
pub fn validate_collision_bundle(bundle: &CollisionShapeBundle) -> bool {
    bundle.shapes.iter().all(|s| !s.name.is_empty())
}

/// Export to JSON.
#[allow(dead_code)]
pub fn collision_bundle_to_json(bundle: &CollisionShapeBundle) -> String {
    format!("{{\"shape_count\":{}}}", collision_shape_count(bundle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_collision() {
        let s = sphere_collision("sphere0", [0.0; 3], 1.0);
        assert_eq!(s.shape_type, CollisionShapeType::Sphere);
    }

    #[test]
    fn test_box_collision() {
        let b = box_collision("box0", [0.0; 3], [1.0, 1.0, 1.0]);
        assert_eq!(b.shape_type, CollisionShapeType::Box);
    }

    #[test]
    fn test_add_collision_shape() {
        let mut bundle = new_collision_bundle();
        add_collision_shape(&mut bundle, sphere_collision("s", [0.0; 3], 1.0));
        assert_eq!(collision_shape_count(&bundle), 1);
    }

    #[test]
    fn test_sphere_volume() {
        let s = sphere_collision("s", [0.0; 3], 1.0);
        let vol = shape_volume(&s);
        assert!((vol - 4.0 * PI / 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_box_volume() {
        let b = box_collision("b", [0.0; 3], [1.0, 2.0, 3.0]);
        let vol = shape_volume(&b);
        assert!((vol - 48.0).abs() < 1e-4);
    }

    #[test]
    fn test_validate_bundle_valid() {
        let mut bundle = new_collision_bundle();
        add_collision_shape(&mut bundle, sphere_collision("s", [0.0; 3], 1.0));
        assert!(validate_collision_bundle(&bundle));
    }

    #[test]
    fn test_validate_bundle_empty_name() {
        let mut bundle = new_collision_bundle();
        add_collision_shape(&mut bundle, sphere_collision("", [0.0; 3], 1.0));
        assert!(!validate_collision_bundle(&bundle));
    }

    #[test]
    fn test_collision_bundle_to_json() {
        let bundle = new_collision_bundle();
        let j = collision_bundle_to_json(&bundle);
        assert!(j.contains("\"shape_count\":0"));
    }

    #[test]
    fn test_empty_bundle() {
        let bundle = new_collision_bundle();
        assert_eq!(collision_shape_count(&bundle), 0);
    }

    #[test]
    fn test_pi_usage() {
        // Verify we use std::f32::consts::PI
        let s = sphere_collision("s", [0.0; 3], 1.0);
        let vol = shape_volume(&s);
        assert!(vol > 4.0);
    }
}
