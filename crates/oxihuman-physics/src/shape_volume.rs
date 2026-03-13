// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Volume and mass calculation for primitive shapes.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Volume of a sphere.
#[allow(dead_code)]
pub fn volume_sphere(radius: f32) -> f32 {
    (4.0 / 3.0) * PI * radius * radius * radius
}

/// Volume of an axis-aligned box with given half-extents.
#[allow(dead_code)]
pub fn volume_box(hx: f32, hy: f32, hz: f32) -> f32 {
    (2.0 * hx) * (2.0 * hy) * (2.0 * hz)
}

/// Volume of a cylinder (axis aligned, given radius and height).
#[allow(dead_code)]
pub fn volume_cylinder(radius: f32, height: f32) -> f32 {
    PI * radius * radius * height
}

/// Volume of a capsule (cylinder + two hemispheres).
#[allow(dead_code)]
pub fn volume_capsule(radius: f32, height: f32) -> f32 {
    volume_cylinder(radius, height) + volume_sphere(radius)
}

/// Volume of a cone.
#[allow(dead_code)]
pub fn volume_cone(radius: f32, height: f32) -> f32 {
    (1.0 / 3.0) * PI * radius * radius * height
}

/// Compute mass from volume and density.
#[allow(dead_code)]
pub fn mass_from_volume(volume: f32, density: f32) -> f32 {
    volume * density
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_sphere_unit() {
        let v = volume_sphere(1.0);
        let expected = 4.0 / 3.0 * PI;
        assert!((v - expected).abs() < 1e-5);
    }

    #[test]
    fn test_volume_sphere_scales_with_cube_of_radius() {
        let v1 = volume_sphere(1.0);
        let v2 = volume_sphere(2.0);
        assert!((v2 - 8.0 * v1).abs() < 1e-4);
    }

    #[test]
    fn test_volume_box_unit() {
        // half-extents (0.5, 0.5, 0.5) → cube of side 1 → volume 1
        let v = volume_box(0.5, 0.5, 0.5);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_volume_box_asymmetric() {
        let v = volume_box(1.0, 2.0, 3.0);
        assert!((v - 48.0).abs() < 1e-5);
    }

    #[test]
    fn test_volume_cylinder_positive() {
        let v = volume_cylinder(1.0, 2.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_volume_capsule_greater_than_sphere() {
        let v_cap = volume_capsule(1.0, 2.0);
        let v_sph = volume_sphere(1.0);
        assert!(v_cap > v_sph);
    }

    #[test]
    fn test_volume_cone_formula() {
        let v = volume_cone(1.0, 3.0);
        let expected = PI;
        assert!((v - expected).abs() < 1e-5);
    }

    #[test]
    fn test_mass_from_volume() {
        let m = mass_from_volume(2.0, 3.0);
        assert!((m - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_mass_from_volume_zero_density() {
        let m = mass_from_volume(100.0, 0.0);
        assert!(m.abs() < 1e-9);
    }
}
