// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Describes mass distribution properties: mass, center of mass, and inertia.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct MassDistribution {
    mass: f32,
    center_of_mass: [f32; 3],
    inertia: [f32; 3], // diagonal inertia tensor (Ixx, Iyy, Izz)
}

#[allow(dead_code)]
impl MassDistribution {
    pub fn new(mass: f32, center_of_mass: [f32; 3], inertia: [f32; 3]) -> Self {
        Self {
            mass: mass.max(0.0),
            center_of_mass,
            inertia,
        }
    }

    pub fn point_mass(mass: f32, position: [f32; 3]) -> Self {
        Self::new(mass, position, [0.0; 3])
    }

    pub fn solid_sphere(mass: f32, radius: f32) -> Self {
        let i = 0.4 * mass * radius * radius;
        Self::new(mass, [0.0; 3], [i, i, i])
    }

    pub fn solid_box(mass: f32, half_extents: [f32; 3]) -> Self {
        let hx2 = half_extents[0] * half_extents[0];
        let hy2 = half_extents[1] * half_extents[1];
        let hz2 = half_extents[2] * half_extents[2];
        let factor = mass / 3.0;
        Self::new(
            mass,
            [0.0; 3],
            [
                factor * (hy2 + hz2),
                factor * (hx2 + hz2),
                factor * (hx2 + hy2),
            ],
        )
    }

    pub fn solid_cylinder(mass: f32, radius: f32, half_height: f32) -> Self {
        let r2 = radius * radius;
        let h2 = half_height * half_height;
        let iyy = mass * r2 * 0.5;
        let ixx = mass * (3.0 * r2 + 4.0 * h2) / 12.0;
        Self::new(mass, [0.0; 3], [ixx, iyy, ixx])
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }

    pub fn inv_mass(&self) -> f32 {
        if self.mass <= 0.0 {
            0.0
        } else {
            1.0 / self.mass
        }
    }

    pub fn center_of_mass(&self) -> [f32; 3] {
        self.center_of_mass
    }

    pub fn inertia(&self) -> [f32; 3] {
        self.inertia
    }

    pub fn inv_inertia(&self) -> [f32; 3] {
        [
            if self.inertia[0] > 0.0 {
                1.0 / self.inertia[0]
            } else {
                0.0
            },
            if self.inertia[1] > 0.0 {
                1.0 / self.inertia[1]
            } else {
                0.0
            },
            if self.inertia[2] > 0.0 {
                1.0 / self.inertia[2]
            } else {
                0.0
            },
        ]
    }

    pub fn is_static(&self) -> bool {
        self.mass <= 0.0
    }

    pub fn translate_inertia(&self, offset: [f32; 3]) -> [f32; 3] {
        let d2 = offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2];
        [
            self.inertia[0] + self.mass * (d2 - offset[0] * offset[0]),
            self.inertia[1] + self.mass * (d2 - offset[1] * offset[1]),
            self.inertia[2] + self.mass * (d2 - offset[2] * offset[2]),
        ]
    }

    pub fn density_sphere(radius: f32) -> f32 {
        let volume = (4.0 / 3.0) * PI * radius * radius * radius;
        if volume > 0.0 {
            1.0 / volume
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_mass() {
        let m = MassDistribution::point_mass(5.0, [1.0, 0.0, 0.0]);
        assert!((m.mass() - 5.0).abs() < 1e-6);
        assert_eq!(m.center_of_mass(), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_solid_sphere() {
        let m = MassDistribution::solid_sphere(10.0, 1.0);
        let i = m.inertia();
        assert!((i[0] - 4.0).abs() < 1e-4);
        assert!((i[0] - i[1]).abs() < 1e-6);
    }

    #[test]
    fn test_solid_box() {
        let m = MassDistribution::solid_box(12.0, [1.0, 1.0, 1.0]);
        let i = m.inertia();
        assert!((i[0] - i[1]).abs() < 1e-6);
    }

    #[test]
    fn test_inv_mass() {
        let m = MassDistribution::point_mass(4.0, [0.0; 3]);
        assert!((m.inv_mass() - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_inv_mass_static() {
        let m = MassDistribution::new(0.0, [0.0; 3], [0.0; 3]);
        assert!((m.inv_mass()).abs() < 1e-6);
        assert!(m.is_static());
    }

    #[test]
    fn test_inv_inertia() {
        let m = MassDistribution::new(1.0, [0.0; 3], [2.0, 4.0, 8.0]);
        let ii = m.inv_inertia();
        assert!((ii[0] - 0.5).abs() < 1e-6);
        assert!((ii[1] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_translate_inertia() {
        let m = MassDistribution::solid_sphere(1.0, 1.0);
        let translated = m.translate_inertia([1.0, 0.0, 0.0]);
        assert!(translated[0] >= m.inertia()[0]);
    }

    #[test]
    fn test_solid_cylinder() {
        let m = MassDistribution::solid_cylinder(10.0, 1.0, 1.0);
        assert!(m.inertia()[0] > 0.0);
        assert!(m.inertia()[1] > 0.0);
    }

    #[test]
    fn test_density_sphere() {
        let d = MassDistribution::density_sphere(1.0);
        assert!(d > 0.0);
    }

    #[test]
    fn test_negative_mass_clamped() {
        let m = MassDistribution::new(-5.0, [0.0; 3], [0.0; 3]);
        assert!((m.mass()).abs() < 1e-6);
    }
}
