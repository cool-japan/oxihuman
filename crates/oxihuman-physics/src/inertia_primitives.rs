// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Inertia tensor computation for primitive shapes as [Ixx, Iyy, Izz, Ixy, Ixz, Iyz].

#![allow(dead_code)]

use std::f32::consts::PI;

/// Compute the inertia tensor diagonal for a solid sphere.
/// Returns [Ixx, Iyy, Izz, Ixy, Ixz, Iyz] (off-diagonal are 0 for primitives).
#[allow(dead_code)]
pub fn inertia_sphere_prim(mass: f32, radius: f32) -> [f32; 6] {
    let i = 2.0 / 5.0 * mass * radius * radius;
    [i, i, i, 0.0, 0.0, 0.0]
}

/// Compute the inertia tensor for a solid box (aligned with axes).
/// hx, hy, hz are half-extents.
#[allow(dead_code)]
pub fn inertia_box_prim(mass: f32, hx: f32, hy: f32, hz: f32) -> [f32; 6] {
    let lx = 2.0 * hx;
    let ly = 2.0 * hy;
    let lz = 2.0 * hz;
    let ixx = mass / 12.0 * (ly * ly + lz * lz);
    let iyy = mass / 12.0 * (lx * lx + lz * lz);
    let izz = mass / 12.0 * (lx * lx + ly * ly);
    [ixx, iyy, izz, 0.0, 0.0, 0.0]
}

/// Compute the inertia tensor for a solid cylinder (axis along Y).
#[allow(dead_code)]
pub fn inertia_cylinder_prim(mass: f32, radius: f32, height: f32) -> [f32; 6] {
    let iyy = 0.5 * mass * radius * radius;
    let ixx_izz = mass / 12.0 * (3.0 * radius * radius + height * height);
    [ixx_izz, iyy, ixx_izz, 0.0, 0.0, 0.0]
}

/// Compute the inertia tensor for a capsule (cylinder + two hemispheres, axis along Y).
#[allow(dead_code)]
pub fn inertia_capsule_prim(mass: f32, radius: f32, height: f32) -> [f32; 6] {
    // Approximate as cylinder + sphere contribution
    let cyl_vol = PI * radius * radius * height;
    let hemi_vol = (2.0 / 3.0) * PI * radius * radius * radius;
    let total_vol = cyl_vol + 2.0 * hemi_vol;
    let (m_cyl, m_hemi) = if total_vol > 1e-9 {
        (mass * cyl_vol / total_vol, mass * hemi_vol / total_vol)
    } else {
        (mass * 0.5, mass * 0.25)
    };

    let [ixx_c, iyy_c, _, _, _, _] = inertia_cylinder_prim(m_cyl, radius, height);
    let [ixx_s, iyy_s, _, _, _, _] = inertia_sphere_prim(m_hemi * 2.0, radius);
    let iyy = iyy_c + iyy_s;
    let ixx = ixx_c + ixx_s;
    [ixx, iyy, ixx, 0.0, 0.0, 0.0]
}

/// Scale an inertia tensor by a scalar factor.
#[allow(dead_code)]
pub fn scale_inertia(tensor: [f32; 6], factor: f32) -> [f32; 6] {
    [
        tensor[0] * factor,
        tensor[1] * factor,
        tensor[2] * factor,
        tensor[3] * factor,
        tensor[4] * factor,
        tensor[5] * factor,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_inertia_isotropic() {
        let t = inertia_sphere_prim(1.0, 1.0);
        // All diagonal equal, off-diagonal zero
        assert!((t[0] - t[1]).abs() < 1e-6);
        assert!((t[1] - t[2]).abs() < 1e-6);
        assert!(t[3].abs() < 1e-9);
    }

    #[test]
    fn test_sphere_inertia_formula() {
        let t = inertia_sphere_prim(5.0, 2.0);
        // 2/5 * 5 * 4 = 8
        assert!((t[0] - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_box_inertia_cube() {
        let t = inertia_box_prim(1.0, 1.0, 1.0, 1.0);
        // For a cube with side 2, Ixx = Iyy = Izz = m/6 * side^2 / 2 = 2/3
        assert!((t[0] - t[1]).abs() < 1e-5);
    }

    #[test]
    fn test_box_off_diagonal_zero() {
        let t = inertia_box_prim(2.0, 0.5, 1.0, 0.5);
        assert!(t[3].abs() < 1e-9);
        assert!(t[4].abs() < 1e-9);
        assert!(t[5].abs() < 1e-9);
    }

    #[test]
    fn test_cylinder_symmetry() {
        let t = inertia_cylinder_prim(1.0, 0.5, 2.0);
        // Ixx == Izz for cylinder aligned on Y
        assert!((t[0] - t[2]).abs() < 1e-6);
    }

    #[test]
    fn test_capsule_returns_six_components() {
        let t = inertia_capsule_prim(1.0, 0.3, 1.0);
        assert_eq!(t.len(), 6);
        // Diagonal components should be positive
        assert!(t[0] > 0.0);
        assert!(t[1] > 0.0);
    }

    #[test]
    fn test_scale_inertia() {
        let t = inertia_sphere_prim(1.0, 1.0);
        let scaled = scale_inertia(t, 2.0);
        assert!((scaled[0] - t[0] * 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale_zero() {
        let t = inertia_sphere_prim(1.0, 1.0);
        let scaled = scale_inertia(t, 0.0);
        for v in &scaled {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn test_sphere_inertia_scales_with_mass() {
        let t1 = inertia_sphere_prim(1.0, 1.0);
        let t2 = inertia_sphere_prim(2.0, 1.0);
        assert!((t2[0] - 2.0 * t1[0]).abs() < 1e-5);
    }
}
