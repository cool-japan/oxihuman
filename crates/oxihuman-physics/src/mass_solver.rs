// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Utilities for computing mass properties from geometry.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MassProperties {
    pub mass: f32,
    pub center_of_mass: [f32; 3],
    pub inertia_diagonal: [f32; 3],
}

#[allow(dead_code)]
impl MassProperties {
    pub fn zero() -> Self {
        Self { mass: 0.0, center_of_mass: [0.0; 3], inertia_diagonal: [0.0; 3] }
    }

    pub fn inv_mass(&self) -> f32 {
        if self.mass > 1e-12 { 1.0 / self.mass } else { 0.0 }
    }

    pub fn inv_inertia(&self) -> [f32; 3] {
        [
            if self.inertia_diagonal[0] > 1e-12 { 1.0 / self.inertia_diagonal[0] } else { 0.0 },
            if self.inertia_diagonal[1] > 1e-12 { 1.0 / self.inertia_diagonal[1] } else { 0.0 },
            if self.inertia_diagonal[2] > 1e-12 { 1.0 / self.inertia_diagonal[2] } else { 0.0 },
        ]
    }
}

/// Mass properties of a solid sphere.
#[allow(dead_code)]
pub fn sphere_mass(density: f32, radius: f32) -> MassProperties {
    let vol = (4.0 / 3.0) * PI * radius * radius * radius;
    let mass = density * vol;
    let i = 0.4 * mass * radius * radius;
    MassProperties { mass, center_of_mass: [0.0; 3], inertia_diagonal: [i, i, i] }
}

/// Mass properties of a solid box (half-extents).
#[allow(dead_code)]
pub fn box_mass(density: f32, half_extents: [f32; 3]) -> MassProperties {
    let (hx, hy, hz) = (half_extents[0], half_extents[1], half_extents[2]);
    let vol = 8.0 * hx * hy * hz;
    let mass = density * vol;
    let (sx, sy, sz) = (4.0 * hx * hx, 4.0 * hy * hy, 4.0 * hz * hz);
    MassProperties {
        mass,
        center_of_mass: [0.0; 3],
        inertia_diagonal: [
            mass / 12.0 * (sy + sz),
            mass / 12.0 * (sx + sz),
            mass / 12.0 * (sx + sy),
        ],
    }
}

/// Mass properties of a solid cylinder (axis along Y).
#[allow(dead_code)]
pub fn cylinder_mass(density: f32, radius: f32, half_height: f32) -> MassProperties {
    let vol = PI * radius * radius * 2.0 * half_height;
    let mass = density * vol;
    let h = 2.0 * half_height;
    let iy = 0.5 * mass * radius * radius;
    let ix = mass * (3.0 * radius * radius + h * h) / 12.0;
    MassProperties {
        mass,
        center_of_mass: [0.0; 3],
        inertia_diagonal: [ix, iy, ix],
    }
}

/// Combine two mass properties (parallel axis theorem simplified).
#[allow(dead_code)]
pub fn combine_mass(a: &MassProperties, b: &MassProperties) -> MassProperties {
    let total = a.mass + b.mass;
    if total < 1e-12 {
        return MassProperties::zero();
    }
    let com = [
        (a.center_of_mass[0] * a.mass + b.center_of_mass[0] * b.mass) / total,
        (a.center_of_mass[1] * a.mass + b.center_of_mass[1] * b.mass) / total,
        (a.center_of_mass[2] * a.mass + b.center_of_mass[2] * b.mass) / total,
    ];
    let inertia = [
        a.inertia_diagonal[0] + b.inertia_diagonal[0],
        a.inertia_diagonal[1] + b.inertia_diagonal[1],
        a.inertia_diagonal[2] + b.inertia_diagonal[2],
    ];
    MassProperties { mass: total, center_of_mass: com, inertia_diagonal: inertia }
}

/// Scale mass uniformly.
#[allow(dead_code)]
pub fn scale_mass(props: &MassProperties, factor: f32) -> MassProperties {
    MassProperties {
        mass: props.mass * factor,
        center_of_mass: props.center_of_mass,
        inertia_diagonal: [
            props.inertia_diagonal[0] * factor,
            props.inertia_diagonal[1] * factor,
            props.inertia_diagonal[2] * factor,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_mass() {
        let p = sphere_mass(1.0, 1.0);
        let expected_vol = (4.0 / 3.0) * PI;
        assert!((p.mass - expected_vol).abs() < 0.01);
    }

    #[test]
    fn test_sphere_inertia() {
        let p = sphere_mass(1.0, 1.0);
        let i = 0.4 * p.mass;
        assert!((p.inertia_diagonal[0] - i).abs() < 0.01);
    }

    #[test]
    fn test_box_mass() {
        let p = box_mass(1.0, [1.0, 1.0, 1.0]);
        assert!((p.mass - 8.0).abs() < 1e-3);
    }

    #[test]
    fn test_cylinder_mass() {
        let p = cylinder_mass(1.0, 1.0, 1.0);
        let expected = PI * 2.0;
        assert!((p.mass - expected).abs() < 0.1);
    }

    #[test]
    fn test_combine() {
        let a = sphere_mass(1.0, 1.0);
        let b = sphere_mass(1.0, 1.0);
        let c = combine_mass(&a, &b);
        assert!((c.mass - 2.0 * a.mass).abs() < 0.01);
    }

    #[test]
    fn test_inv_mass() {
        let p = sphere_mass(1.0, 1.0);
        assert!((p.inv_mass() * p.mass - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_inv_mass_zero() {
        let p = MassProperties::zero();
        assert!((p.inv_mass()).abs() < 1e-10);
    }

    #[test]
    fn test_scale() {
        let p = sphere_mass(1.0, 1.0);
        let s = scale_mass(&p, 2.0);
        assert!((s.mass - 2.0 * p.mass).abs() < 0.01);
    }

    #[test]
    fn test_inv_inertia() {
        let p = sphere_mass(1.0, 1.0);
        let inv = p.inv_inertia();
        assert!((inv[0] * p.inertia_diagonal[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_box_symmetric() {
        let p = box_mass(1.0, [1.0, 1.0, 1.0]);
        assert!((p.inertia_diagonal[0] - p.inertia_diagonal[1]).abs() < 1e-5);
    }
}
