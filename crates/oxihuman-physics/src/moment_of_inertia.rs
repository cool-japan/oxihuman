// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Moment-of-inertia computations for common primitives and composite bodies.

/// Solid sphere: I = 2/5 * m * r^2
#[allow(dead_code)]
pub fn sphere_solid(mass: f32, radius: f32) -> f32 {
    0.4 * mass * radius * radius
}

/// Hollow sphere (thin shell): I = 2/3 * m * r^2
#[allow(dead_code)]
pub fn sphere_shell(mass: f32, radius: f32) -> f32 {
    (2.0 / 3.0) * mass * radius * radius
}

/// Solid cylinder about its axis: I = 1/2 * m * r^2
#[allow(dead_code)]
pub fn cylinder_axial(mass: f32, radius: f32) -> f32 {
    0.5 * mass * radius * radius
}

/// Solid cylinder about diameter: I = m*(3r^2 + h^2)/12
#[allow(dead_code)]
pub fn cylinder_diameter(mass: f32, radius: f32, height: f32) -> f32 {
    mass * (3.0 * radius * radius + height * height) / 12.0
}

/// Thin rod about center: I = m*L^2/12
#[allow(dead_code)]
pub fn rod_center(mass: f32, length: f32) -> f32 {
    mass * length * length / 12.0
}

/// Thin rod about end: I = m*L^2/3
#[allow(dead_code)]
pub fn rod_end(mass: f32, length: f32) -> f32 {
    mass * length * length / 3.0
}

/// Rectangular plate about center (perpendicular axis): I = m*(a^2 + b^2)/12
#[allow(dead_code)]
pub fn plate_center(mass: f32, width: f32, height: f32) -> f32 {
    mass * (width * width + height * height) / 12.0
}

/// Parallel axis theorem: I' = I_cm + m*d^2
#[allow(dead_code)]
pub fn parallel_axis(i_cm: f32, mass: f32, distance: f32) -> f32 {
    i_cm + mass * distance * distance
}

/// Point mass: I = m*r^2
#[allow(dead_code)]
pub fn point_mass(mass: f32, distance: f32) -> f32 {
    mass * distance * distance
}

/// Thin disk: I = 1/2 * m * r^2
#[allow(dead_code)]
pub fn disk(mass: f32, radius: f32) -> f32 {
    0.5 * mass * radius * radius
}

/// Annular ring: I = 1/2 * m * (r1^2 + r2^2)
#[allow(dead_code)]
pub fn annular_ring(mass: f32, inner_r: f32, outer_r: f32) -> f32 {
    0.5 * mass * (inner_r * inner_r + outer_r * outer_r)
}

/// Radius of gyration: k = sqrt(I/m)
#[allow(dead_code)]
pub fn radius_of_gyration(inertia: f32, mass: f32) -> f32 {
    if mass > 1e-12 { (inertia / mass).sqrt() } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_solid() {
        let i = sphere_solid(10.0, 2.0);
        assert!((i - 16.0).abs() < 1e-4);
    }

    #[test]
    fn test_sphere_shell() {
        let i = sphere_shell(3.0, 1.0);
        assert!((i - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_cylinder_axial() {
        let i = cylinder_axial(4.0, 3.0);
        assert!((i - 18.0).abs() < 1e-4);
    }

    #[test]
    fn test_rod_center() {
        let i = rod_center(12.0, 2.0);
        assert!((i - 4.0).abs() < 1e-4);
    }

    #[test]
    fn test_rod_end() {
        let i = rod_end(12.0, 2.0);
        assert!((i - 16.0).abs() < 1e-4);
    }

    #[test]
    fn test_parallel_axis() {
        let i_cm = rod_center(10.0, 2.0);
        let i_end = parallel_axis(i_cm, 10.0, 1.0);
        let expected = rod_end(10.0, 2.0);
        assert!((i_end - expected).abs() < 0.1);
    }

    #[test]
    fn test_point_mass() {
        let i = point_mass(5.0, 3.0);
        assert!((i - 45.0).abs() < 1e-4);
    }

    #[test]
    fn test_disk() {
        let i = disk(2.0, 4.0);
        assert!((i - 16.0).abs() < 1e-4);
    }

    #[test]
    fn test_radius_of_gyration() {
        let i = sphere_solid(10.0, 2.0);
        let k = radius_of_gyration(i, 10.0);
        assert!((k * k * 10.0 - i).abs() < 1e-3);
    }

    #[test]
    fn test_annular_ring() {
        let i = annular_ring(2.0, 1.0, 2.0);
        assert!((i - 5.0).abs() < 1e-4);
    }
}
