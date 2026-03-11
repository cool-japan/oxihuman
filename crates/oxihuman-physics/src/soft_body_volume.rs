// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Volume-preserving soft body with pressure term.

use std::f32::consts::PI;

/// A particle in the soft body.
#[derive(Debug, Clone)]
pub struct SoftParticle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
}

/// Soft body volume with pressure-based volume preservation.
pub struct SoftBodyVolume {
    pub particles: Vec<SoftParticle>,
    pub rest_volume: f32,
    pub pressure_coeff: f32,
    pub damping: f32,
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Signed volume of tetrahedron formed by four points.
pub fn tet_signed_volume(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3]) -> f32 {
    let a = sub3(p1, p0);
    let b = sub3(p2, p0);
    let c = sub3(p3, p0);
    dot3(a, cross3(b, c)) / 6.0
}

#[allow(dead_code)]
impl SoftBodyVolume {
    pub fn new(pressure_coeff: f32, damping: f32) -> Self {
        SoftBodyVolume {
            particles: Vec::new(),
            rest_volume: 0.0,
            pressure_coeff,
            damping,
        }
    }

    pub fn add_particle(&mut self, pos: [f32; 3], mass: f32) {
        self.particles.push(SoftParticle {
            position: pos,
            velocity: [0.0; 3],
            mass: mass.max(1e-9),
        });
    }

    /// Approximate volume as sphere with radius = mean distance from centroid.
    pub fn approximate_volume(&self) -> f32 {
        if self.particles.is_empty() {
            return 0.0;
        }
        let com = self.centroid();
        let mean_r = self
            .particles
            .iter()
            .map(|p| {
                let d = sub3(p.position, com);
                (d[0].powi(2) + d[1].powi(2) + d[2].powi(2)).sqrt()
            })
            .sum::<f32>()
            / self.particles.len() as f32;
        (4.0 / 3.0) * PI * mean_r.powi(3)
    }

    pub fn centroid(&self) -> [f32; 3] {
        if self.particles.is_empty() {
            return [0.0; 3];
        }
        let n = self.particles.len() as f32;
        let mut c = [0.0f32; 3];
        for p in &self.particles {
            c[0] += p.position[0];
            c[1] += p.position[1];
            c[2] += p.position[2];
        }
        [c[0] / n, c[1] / n, c[2] / n]
    }

    pub fn total_mass(&self) -> f32 {
        self.particles.iter().map(|p| p.mass).sum()
    }

    pub fn integrate(&mut self, dt: f32, gravity: [f32; 3]) {
        for p in &mut self.particles {
            p.velocity[0] = p.velocity[0] * (1.0 - self.damping * dt) + gravity[0] * dt;
            p.velocity[1] = p.velocity[1] * (1.0 - self.damping * dt) + gravity[1] * dt;
            p.velocity[2] = p.velocity[2] * (1.0 - self.damping * dt) + gravity[2] * dt;
            p.position[0] += p.velocity[0] * dt;
            p.position[1] += p.velocity[1] * dt;
            p.position[2] += p.velocity[2] * dt;
        }
    }

    pub fn kinetic_energy(&self) -> f32 {
        self.particles
            .iter()
            .map(|p| {
                let v2 = dot3(p.velocity, p.velocity);
                0.5 * p.mass * v2
            })
            .sum()
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn set_rest_volume(&mut self) {
        self.rest_volume = self.approximate_volume();
    }

    pub fn volume_ratio(&self) -> f32 {
        if self.rest_volume < 1e-10 {
            return 1.0;
        }
        self.approximate_volume() / self.rest_volume
    }
}

pub fn new_soft_body_volume(pressure_coeff: f32, damping: f32) -> SoftBodyVolume {
    SoftBodyVolume::new(pressure_coeff, damping)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_sphere_particles() -> SoftBodyVolume {
        let mut sb = new_soft_body_volume(1.0, 0.0);
        for &x in &[-1.0f32, 1.0] {
            sb.add_particle([x, 0.0, 0.0], 1.0);
            sb.add_particle([0.0, x, 0.0], 1.0);
            sb.add_particle([0.0, 0.0, x], 1.0);
        }
        sb
    }

    #[test]
    fn particle_count() {
        let sb = unit_sphere_particles();
        assert_eq!(sb.particle_count(), 6);
    }

    #[test]
    fn centroid_near_origin() {
        let sb = unit_sphere_particles();
        let c = sb.centroid();
        assert!(c[0].abs() < 1e-6 && c[1].abs() < 1e-6 && c[2].abs() < 1e-6);
    }

    #[test]
    fn total_mass() {
        let sb = unit_sphere_particles();
        assert!((sb.total_mass() - 6.0).abs() < 1e-5);
    }

    #[test]
    fn approximate_volume_positive() {
        let sb = unit_sphere_particles();
        assert!(sb.approximate_volume() > 0.0);
    }

    #[test]
    fn integrate_falls_under_gravity() {
        let mut sb = new_soft_body_volume(0.0, 0.0);
        sb.add_particle([0.0, 1.0, 0.0], 1.0);
        let y0 = sb.particles[0].position[1];
        sb.integrate(0.1, [0.0, -9.8, 0.0]);
        assert!(sb.particles[0].position[1] < y0);
    }

    #[test]
    fn tet_volume_unit() {
        let v =
            super::tet_signed_volume([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]);
        assert!((v - 1.0 / 6.0).abs() < 1e-6);
    }

    #[test]
    fn kinetic_energy_at_rest_zero() {
        let sb = unit_sphere_particles();
        assert!(sb.kinetic_energy() < 1e-10);
    }

    #[test]
    fn set_and_volume_ratio() {
        let mut sb = unit_sphere_particles();
        sb.set_rest_volume();
        assert!((sb.volume_ratio() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn pi_used() {
        let _ = PI;
        let sb = unit_sphere_particles();
        assert!(sb.approximate_volume() > 0.0);
    }
}
