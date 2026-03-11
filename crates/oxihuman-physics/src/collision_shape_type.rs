#![allow(dead_code)]

use std::f32::consts::PI;

/// Enumeration of collision shape types.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionShapeType {
    Sphere,
    Box,
    Capsule,
    Cylinder,
    Cone,
    ConvexHull,
    TriangleMesh,
}

/// Returns the name of the shape type.
#[allow(dead_code)]
pub fn shape_type_name(shape: CollisionShapeType) -> &'static str {
    match shape {
        CollisionShapeType::Sphere => "sphere",
        CollisionShapeType::Box => "box",
        CollisionShapeType::Capsule => "capsule",
        CollisionShapeType::Cylinder => "cylinder",
        CollisionShapeType::Cone => "cone",
        CollisionShapeType::ConvexHull => "convex_hull",
        CollisionShapeType::TriangleMesh => "triangle_mesh",
    }
}

/// Returns whether the shape type is convex.
#[allow(dead_code)]
pub fn shape_is_convex(shape: CollisionShapeType) -> bool {
    !matches!(shape, CollisionShapeType::TriangleMesh)
}

/// Returns the support function value along a direction (simplified, for unit shapes).
#[allow(dead_code)]
pub fn shape_support_fn(shape: CollisionShapeType, direction: [f32; 3], radius: f32) -> [f32; 3] {
    let len = (direction[0] * direction[0]
        + direction[1] * direction[1]
        + direction[2] * direction[2])
    .sqrt();
    if len < f32::EPSILON {
        return [0.0; 3];
    }
    let inv = radius / len;
    match shape {
        CollisionShapeType::Sphere => {
            [direction[0] * inv, direction[1] * inv, direction[2] * inv]
        }
        _ => {
            [direction[0] * inv, direction[1] * inv, direction[2] * inv]
        }
    }
}

/// Returns the AABB half-extents for a unit shape.
#[allow(dead_code)]
pub fn shape_aabb(shape: CollisionShapeType, radius: f32) -> ([f32; 3], [f32; 3]) {
    match shape {
        CollisionShapeType::Sphere | CollisionShapeType::Capsule => {
            ([-radius; 3], [radius; 3])
        }
        CollisionShapeType::Box => {
            ([-radius; 3], [radius; 3])
        }
        _ => ([-radius; 3], [radius; 3]),
    }
}

/// Returns approximate volume for a unit shape.
#[allow(dead_code)]
pub fn shape_volume_approx(shape: CollisionShapeType, radius: f32) -> f32 {
    match shape {
        CollisionShapeType::Sphere => (4.0 / 3.0) * PI * radius * radius * radius,
        CollisionShapeType::Box => (2.0 * radius).powi(3),
        CollisionShapeType::Cylinder => PI * radius * radius * (2.0 * radius),
        CollisionShapeType::Cone => PI * radius * radius * (2.0 * radius) / 3.0,
        CollisionShapeType::Capsule => {
            (4.0 / 3.0) * PI * radius * radius * radius + PI * radius * radius * (2.0 * radius)
        }
        _ => (2.0 * radius).powi(3),
    }
}

/// Returns approximate inertia for a unit shape (scalar, about center).
#[allow(dead_code)]
pub fn shape_inertia_approx(shape: CollisionShapeType, mass: f32, radius: f32) -> f32 {
    match shape {
        CollisionShapeType::Sphere => 0.4 * mass * radius * radius,
        CollisionShapeType::Box => mass * (2.0 * radius * 2.0 * radius) / 6.0,
        _ => 0.4 * mass * radius * radius,
    }
}

/// Returns the center of the shape (always origin for primitive shapes).
#[allow(dead_code)]
pub fn shape_center(_shape: CollisionShapeType) -> [f32; 3] {
    [0.0; 3]
}

/// Serializes shape type to JSON.
#[allow(dead_code)]
pub fn shape_to_json(shape: CollisionShapeType) -> String {
    format!("{{\"type\":\"{}\",\"convex\":{}}}", shape_type_name(shape), shape_is_convex(shape))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_name() {
        assert_eq!(shape_type_name(CollisionShapeType::Sphere), "sphere");
        assert_eq!(shape_type_name(CollisionShapeType::Box), "box");
    }

    #[test]
    fn test_is_convex() {
        assert!(shape_is_convex(CollisionShapeType::Sphere));
        assert!(!shape_is_convex(CollisionShapeType::TriangleMesh));
    }

    #[test]
    fn test_support_fn() {
        let s = shape_support_fn(CollisionShapeType::Sphere, [1.0, 0.0, 0.0], 1.0);
        assert!((s[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_aabb() {
        let (min, max) = shape_aabb(CollisionShapeType::Sphere, 1.0);
        assert!((min[0] - (-1.0)).abs() < f32::EPSILON);
        assert!((max[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_volume_sphere() {
        let v = shape_volume_approx(CollisionShapeType::Sphere, 1.0);
        assert!((v - 4.0 * PI / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_volume_box() {
        let v = shape_volume_approx(CollisionShapeType::Box, 1.0);
        assert!((v - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inertia() {
        let i = shape_inertia_approx(CollisionShapeType::Sphere, 1.0, 1.0);
        assert!((i - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_center() {
        let c = shape_center(CollisionShapeType::Sphere);
        assert!((c[0]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_to_json() {
        let json = shape_to_json(CollisionShapeType::Sphere);
        assert!(json.contains("\"type\":\"sphere\""));
    }

    #[test]
    fn test_support_zero_dir() {
        let s = shape_support_fn(CollisionShapeType::Sphere, [0.0; 3], 1.0);
        assert!((s[0]).abs() < f32::EPSILON);
    }
}
