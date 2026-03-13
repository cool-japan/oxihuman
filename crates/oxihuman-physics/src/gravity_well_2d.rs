// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2D gravity well/attractor stub.

#[derive(Debug, Clone)]
pub struct GravityWell2d {
    pub position: [f32; 2],
    pub mass: f32,
    pub g_const: f32,
    pub min_dist: f32,
}

impl GravityWell2d {
    pub fn new(x: f32, y: f32, mass: f32) -> Self {
        GravityWell2d {
            position: [x, y],
            mass,
            g_const: 6.674e-11,
            min_dist: 0.1,
        }
    }

    pub fn with_g(mut self, g: f32) -> Self {
        self.g_const = g;
        self
    }

    pub fn force_on(&self, pos: [f32; 2], body_mass: f32) -> [f32; 2] {
        gravity_force_2d(self, pos, body_mass)
    }

    pub fn acceleration_at(&self, pos: [f32; 2]) -> [f32; 2] {
        gravity_force_2d(self, pos, 1.0)
    }

    pub fn escape_velocity(&self, dist: f32) -> f32 {
        let dist = dist.max(self.min_dist);
        (2.0 * self.g_const * self.mass / dist).sqrt()
    }

    pub fn orbital_velocity(&self, dist: f32) -> f32 {
        let dist = dist.max(self.min_dist);
        (self.g_const * self.mass / dist).sqrt()
    }
}

pub fn gravity_force_2d(well: &GravityWell2d, pos: [f32; 2], body_mass: f32) -> [f32; 2] {
    let dx = well.position[0] - pos[0];
    let dy = well.position[1] - pos[1];
    let raw_sq = dx * dx + dy * dy;
    let dist = raw_sq.sqrt().max(well.min_dist);
    if dist < f32::EPSILON {
        return [0.0; 2];
    }
    /* Use clamped dist to avoid singularity */
    let dist_sq = dist * dist;
    let force_mag = well.g_const * well.mass * body_mass / dist_sq;
    if raw_sq < f32::EPSILON {
        /* At exact origin, return a small upward push */
        return [0.0, force_mag];
    }
    let raw_dist = raw_sq.sqrt();
    [force_mag * dx / raw_dist, force_mag * dy / raw_dist]
}

pub fn apply_gravity_wells(
    wells: &[GravityWell2d],
    positions: &[[f32; 2]],
    velocities: &mut [[f32; 2]],
    masses: &[f32],
    dt: f32,
) {
    for ((pos, vel), &mass) in positions
        .iter()
        .zip(velocities.iter_mut())
        .zip(masses.iter())
    {
        let mut ax = 0.0f32;
        let mut ay = 0.0f32;
        for well in wells {
            let f = gravity_force_2d(well, *pos, mass);
            if mass > f32::EPSILON {
                ax += f[0] / mass;
                ay += f[1] / mass;
            }
        }
        vel[0] += ax * dt;
        vel[1] += ay * dt;
    }
}

pub fn potential_energy_2d(well: &GravityWell2d, pos: [f32; 2], body_mass: f32) -> f32 {
    let dx = pos[0] - well.position[0];
    let dy = pos[1] - well.position[1];
    let dist = (dx * dx + dy * dy).sqrt().max(well.min_dist);
    -well.g_const * well.mass * body_mass / dist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_direction_toward_well() {
        let well = GravityWell2d::new(0.0, 0.0, 1e12).with_g(1.0);
        let f = well.force_on([1.0, 0.0], 1.0);
        assert!(f[0] < 0.0 /* force pulls toward well (negative x) */,);
    }

    #[test]
    fn test_force_magnitude_inverse_square() {
        let well = GravityWell2d::new(0.0, 0.0, 1e12).with_g(1.0);
        let f_near = gravity_force_2d(&well, [1.0, 0.0], 1.0);
        let f_far = gravity_force_2d(&well, [2.0, 0.0], 1.0);
        let ratio = f_near[0].abs() / f_far[0].abs();
        assert!((ratio - 4.0).abs() < 0.1, /* inverse square: 4x at 2x distance */);
    }

    #[test]
    fn test_at_minimum_distance() {
        let well = GravityWell2d::new(0.0, 0.0, 1.0).with_g(1.0);
        let f = well.force_on([0.0, 0.0], 1.0);
        assert!(f[0].is_finite() && f[1].is_finite(), /* no inf at origin */);
    }

    #[test]
    fn test_escape_velocity_positive() {
        let well = GravityWell2d::new(0.0, 0.0, 1e12).with_g(1.0);
        assert!(well.escape_velocity(1.0) > 0.0, /* positive escape velocity */);
    }

    #[test]
    fn test_escape_gt_orbital() {
        let well = GravityWell2d::new(0.0, 0.0, 1e12).with_g(1.0);
        let esc = well.escape_velocity(5.0);
        let orb = well.orbital_velocity(5.0);
        assert!(esc > orb /* escape > orbital velocity */,);
    }

    #[test]
    fn test_potential_energy_negative() {
        let well = GravityWell2d::new(0.0, 0.0, 1e12).with_g(1.0);
        let pe = potential_energy_2d(&well, [5.0, 0.0], 1.0);
        assert!(pe < 0.0 /* gravitational PE is negative */,);
    }

    #[test]
    fn test_apply_gravity_wells() {
        let wells = vec![GravityWell2d::new(0.0, 0.0, 1e14).with_g(1.0)];
        let positions = vec![[10.0f32, 0.0]];
        let mut velocities = vec![[0.0f32; 2]];
        let masses = vec![1.0f32];
        apply_gravity_wells(&wells, &positions, &mut velocities, &masses, 1.0);
        assert!(velocities[0][0] < 0.0 /* pulled toward well */,);
    }

    #[test]
    fn test_force_increases_with_mass() {
        let well = GravityWell2d::new(0.0, 0.0, 1e10).with_g(1.0);
        let f1 = gravity_force_2d(&well, [5.0, 0.0], 1.0);
        let f2 = gravity_force_2d(&well, [5.0, 0.0], 2.0);
        assert!(
            (f2[0].abs() - 2.0 * f1[0].abs()).abs() < 1e-3,
            /* force proportional to body mass */
        );
    }

    #[test]
    fn test_acceleration_independent_of_mass() {
        let well = GravityWell2d::new(0.0, 0.0, 1e10).with_g(1.0);
        let a1 = well.acceleration_at([5.0, 0.0]);
        let f2 = gravity_force_2d(&well, [5.0, 0.0], 2.0);
        let a2 = [f2[0] / 2.0, f2[1] / 2.0];
        assert!((a1[0] - a2[0]).abs() < 1e-6, /* acceleration same regardless of mass */);
    }
}
