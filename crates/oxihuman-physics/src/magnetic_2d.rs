// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2D magnetic field simulation stub.

/// A 2D magnetic field source (dipole approximation).
#[derive(Debug, Clone)]
pub struct MagneticSource2d {
    pub position: [f32; 2],
    pub strength: f32,
    pub polarity: f32,
}

impl MagneticSource2d {
    pub fn new(x: f32, y: f32, strength: f32) -> Self {
        MagneticSource2d {
            position: [x, y],
            strength,
            polarity: 1.0,
        }
    }

    pub fn north(x: f32, y: f32, strength: f32) -> Self {
        MagneticSource2d::new(x, y, strength)
    }

    pub fn south(x: f32, y: f32, strength: f32) -> Self {
        MagneticSource2d {
            position: [x, y],
            strength,
            polarity: -1.0,
        }
    }

    pub fn field_at(&self, pos: [f32; 2]) -> [f32; 2] {
        magnetic_field_2d(self, pos)
    }
}

/// Compute the 2D magnetic field vector at a point (inverse-square dipole).
pub fn magnetic_field_2d(source: &MagneticSource2d, pos: [f32; 2]) -> [f32; 2] {
    let dx = pos[0] - source.position[0];
    let dy = pos[1] - source.position[1];
    let dist_sq = dx * dx + dy * dy;
    if dist_sq < f32::EPSILON {
        return [0.0; 2];
    }
    let dist = dist_sq.sqrt();
    let mag = source.strength * source.polarity / dist_sq;
    [mag * dx / dist, mag * dy / dist]
}

/// Compute force on a charged particle moving in the field.
pub fn lorentz_force_2d(charge: f32, velocity: [f32; 2], field_z: f32) -> [f32; 2] {
    [
        charge * velocity[1] * field_z,
        -charge * velocity[0] * field_z,
    ]
}

/// Superpose fields from multiple sources.
pub fn total_field_2d(sources: &[MagneticSource2d], pos: [f32; 2]) -> [f32; 2] {
    sources.iter().fold([0.0f32; 2], |acc, s| {
        let f = s.field_at(pos);
        [acc[0] + f[0], acc[1] + f[1]]
    })
}

/// Field magnitude at a point from a source.
pub fn field_magnitude_2d(source: &MagneticSource2d, pos: [f32; 2]) -> f32 {
    let f = magnetic_field_2d(source, pos);
    (f[0] * f[0] + f[1] * f[1]).sqrt()
}

/// Apply magnetic force to a body's velocity.
pub fn apply_magnetic_force(
    position: [f32; 2],
    velocity: &mut [f32; 2],
    charge: f32,
    mass: f32,
    sources: &[MagneticSource2d],
    dt: f32,
) {
    let field = total_field_2d(sources, position);
    let field_z = field[0] + field[1];
    let force = lorentz_force_2d(charge, *velocity, field_z);
    if mass > f32::EPSILON {
        velocity[0] += force[0] / mass * dt;
        velocity[1] += force[1] / mass * dt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_at_source_is_zero() {
        let src = MagneticSource2d::new(0.0, 0.0, 1.0);
        let f = src.field_at([0.0, 0.0]);
        assert!(f[0].abs() < 1e-5 && f[1].abs() < 1e-5, /* field at source = 0 */);
    }

    #[test]
    fn test_field_decays_with_distance() {
        let src = MagneticSource2d::new(0.0, 0.0, 100.0);
        let f_near = field_magnitude_2d(&src, [1.0, 0.0]);
        let f_far = field_magnitude_2d(&src, [10.0, 0.0]);
        assert!(f_near > f_far /* field decays with distance */,);
    }

    #[test]
    fn test_south_polarity_reverses_field() {
        let north = MagneticSource2d::north(0.0, 0.0, 100.0);
        let south = MagneticSource2d::south(0.0, 0.0, 100.0);
        let f_n = magnetic_field_2d(&north, [1.0, 0.0]);
        let f_s = magnetic_field_2d(&south, [1.0, 0.0]);
        assert!((f_n[0] + f_s[0]).abs() < 1e-5, /* north + south cancel */);
    }

    #[test]
    fn test_lorentz_force_perpendicular() {
        let force = lorentz_force_2d(1.0, [1.0, 0.0], 1.0);
        assert!(force[1].abs() > 0.0, /* lorentz force is perpendicular to velocity */);
    }

    #[test]
    fn test_lorentz_zero_velocity() {
        let force = lorentz_force_2d(1.0, [0.0, 0.0], 1.0);
        assert!(
            force[0].abs() < 1e-6 && force[1].abs() < 1e-6,
            /* no velocity = no lorentz force */
        );
    }

    #[test]
    fn test_total_field_superposition() {
        let sources = vec![
            MagneticSource2d::north(0.0, 0.0, 100.0),
            MagneticSource2d::north(0.0, 0.0, 100.0),
        ];
        let f_single = MagneticSource2d::north(0.0, 0.0, 100.0).field_at([1.0, 0.0]);
        let f_total = total_field_2d(&sources, [1.0, 0.0]);
        assert!(
            (f_total[0] - 2.0 * f_single[0]).abs() < 1e-4,
            /* two identical sources = double field */
        );
    }

    #[test]
    fn test_apply_magnetic_force() {
        let src = MagneticSource2d::new(0.0, 0.0, 1000.0);
        let mut vel = [1.0f32, 0.0];
        apply_magnetic_force([5.0, 0.0], &mut vel, 1.0, 1.0, &[src], 0.1);
        assert!(vel[0].is_finite() && vel[1].is_finite(), /* velocity stays finite */);
    }

    #[test]
    fn test_field_magnitude_positive() {
        let src = MagneticSource2d::new(0.0, 0.0, 100.0);
        let mag = field_magnitude_2d(&src, [2.0, 0.0]);
        assert!(mag > 0.0 /* non-zero field at non-zero distance */,);
    }
}
