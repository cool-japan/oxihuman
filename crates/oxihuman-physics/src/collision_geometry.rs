// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Collision geometry shapes for physics simulation.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum CollisionGeometry {
    Sphere { radius: f32 },
    Box { half_extents: [f32; 3] },
    Capsule { radius: f32, half_height: f32 },
    Cylinder { radius: f32, half_height: f32 },
}

#[allow(dead_code)]
impl CollisionGeometry {
    pub fn sphere(radius: f32) -> Self {
        Self::Sphere { radius }
    }

    pub fn aabb(half_extents: [f32; 3]) -> Self {
        Self::Box { half_extents }
    }

    pub fn capsule(radius: f32, half_height: f32) -> Self {
        Self::Capsule {
            radius,
            half_height,
        }
    }

    pub fn cylinder(radius: f32, half_height: f32) -> Self {
        Self::Cylinder {
            radius,
            half_height,
        }
    }

    pub fn volume(&self) -> f32 {
        match *self {
            Self::Sphere { radius } => (4.0 / 3.0) * PI * radius * radius * radius,
            Self::Box { half_extents } => {
                8.0 * half_extents[0] * half_extents[1] * half_extents[2]
            }
            Self::Capsule {
                radius,
                half_height,
            } => {
                let cylinder = PI * radius * radius * 2.0 * half_height;
                let sphere = (4.0 / 3.0) * PI * radius * radius * radius;
                cylinder + sphere
            }
            Self::Cylinder {
                radius,
                half_height,
            } => PI * radius * radius * 2.0 * half_height,
        }
    }

    pub fn surface_area(&self) -> f32 {
        match *self {
            Self::Sphere { radius } => 4.0 * PI * radius * radius,
            Self::Box { half_extents } => {
                let [x, y, z] = half_extents;
                8.0 * (x * y + y * z + x * z)
            }
            Self::Capsule {
                radius,
                half_height,
            } => {
                let cylinder_side = 2.0 * PI * radius * 2.0 * half_height;
                let sphere = 4.0 * PI * radius * radius;
                cylinder_side + sphere
            }
            Self::Cylinder {
                radius,
                half_height,
            } => {
                let side = 2.0 * PI * radius * 2.0 * half_height;
                let caps = 2.0 * PI * radius * radius;
                side + caps
            }
        }
    }

    pub fn bounding_radius(&self) -> f32 {
        match *self {
            Self::Sphere { radius } => radius,
            Self::Box { half_extents } => {
                let [x, y, z] = half_extents;
                (x * x + y * y + z * z).sqrt()
            }
            Self::Capsule {
                radius,
                half_height,
            } => radius + half_height,
            Self::Cylinder {
                radius,
                half_height,
            } => (radius * radius + half_height * half_height).sqrt(),
        }
    }

    pub fn contains_point(&self, point: [f32; 3]) -> bool {
        match *self {
            Self::Sphere { radius } => {
                let d2: f32 = point.iter().map(|&p| p * p).sum();
                d2 <= radius * radius
            }
            Self::Box { half_extents } => {
                point[0].abs() <= half_extents[0]
                    && point[1].abs() <= half_extents[1]
                    && point[2].abs() <= half_extents[2]
            }
            Self::Capsule {
                radius,
                half_height,
            } => {
                let y_clamped = point[1].clamp(-half_height, half_height);
                let dx = point[0];
                let dy = point[1] - y_clamped;
                let dz = point[2];
                dx * dx + dy * dy + dz * dz <= radius * radius
            }
            Self::Cylinder {
                radius,
                half_height,
            } => {
                let r2 = point[0] * point[0] + point[2] * point[2];
                r2 <= radius * radius && point[1].abs() <= half_height
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_volume() {
        let s = CollisionGeometry::sphere(1.0);
        let expected = (4.0 / 3.0) * PI;
        assert!((s.volume() - expected).abs() < 1e-5);
    }

    #[test]
    fn test_box_volume() {
        let b = CollisionGeometry::aabb([1.0, 2.0, 3.0]);
        assert!((b.volume() - 48.0).abs() < 1e-5);
    }

    #[test]
    fn test_sphere_surface_area() {
        let s = CollisionGeometry::sphere(1.0);
        assert!((s.surface_area() - 4.0 * PI).abs() < 1e-5);
    }

    #[test]
    fn test_bounding_radius_sphere() {
        let s = CollisionGeometry::sphere(5.0);
        assert!((s.bounding_radius() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_bounding_radius_box() {
        let b = CollisionGeometry::aabb([1.0, 1.0, 1.0]);
        let expected = 3.0f32.sqrt();
        assert!((b.bounding_radius() - expected).abs() < 1e-5);
    }

    #[test]
    fn test_contains_point_sphere() {
        let s = CollisionGeometry::sphere(2.0);
        assert!(s.contains_point([1.0, 0.0, 0.0]));
        assert!(!s.contains_point([3.0, 0.0, 0.0]));
    }

    #[test]
    fn test_contains_point_box() {
        let b = CollisionGeometry::aabb([1.0, 1.0, 1.0]);
        assert!(b.contains_point([0.5, 0.5, 0.5]));
        assert!(!b.contains_point([2.0, 0.0, 0.0]));
    }

    #[test]
    fn test_capsule_volume() {
        let c = CollisionGeometry::capsule(1.0, 2.0);
        assert!(c.volume() > 0.0);
    }

    #[test]
    fn test_cylinder_contains() {
        let c = CollisionGeometry::cylinder(1.0, 2.0);
        assert!(c.contains_point([0.5, 1.0, 0.0]));
        assert!(!c.contains_point([0.0, 3.0, 0.0]));
    }

    #[test]
    fn test_capsule_contains() {
        let c = CollisionGeometry::capsule(1.0, 2.0);
        assert!(c.contains_point([0.0, 2.5, 0.0]));
        assert!(!c.contains_point([0.0, 5.0, 0.0]));
    }
}
