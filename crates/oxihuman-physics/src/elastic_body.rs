// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// An elastic body represented as a network of particles with spring connections.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ElasticBody {
    positions: Vec<[f32; 3]>,
    velocities: Vec<[f32; 3]>,
    rest_positions: Vec<[f32; 3]>,
    mass: f32,
    stiffness: f32,
    damping: f32,
}

#[allow(dead_code)]
impl ElasticBody {
    pub fn new(positions: Vec<[f32; 3]>, mass: f32, stiffness: f32, damping: f32) -> Self {
        let rest = positions.clone();
        let n = positions.len();
        Self {
            positions,
            velocities: vec![[0.0; 3]; n],
            rest_positions: rest,
            mass: mass.max(f32::EPSILON),
            stiffness: stiffness.max(0.0),
            damping: damping.max(0.0),
        }
    }

    pub fn particle_count(&self) -> usize {
        self.positions.len()
    }

    pub fn position(&self, idx: usize) -> Option<[f32; 3]> {
        self.positions.get(idx).copied()
    }

    pub fn set_position(&mut self, idx: usize, pos: [f32; 3]) {
        if idx < self.positions.len() {
            self.positions[idx] = pos;
        }
    }

    pub fn velocity(&self, idx: usize) -> Option<[f32; 3]> {
        self.velocities.get(idx).copied()
    }

    pub fn apply_force(&mut self, idx: usize, force: [f32; 3], dt: f32) {
        if idx < self.velocities.len() {
            let inv_m = 1.0 / self.mass;
            self.velocities[idx][0] += force[0] * inv_m * dt;
            self.velocities[idx][1] += force[1] * inv_m * dt;
            self.velocities[idx][2] += force[2] * inv_m * dt;
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, dt: f32) {
        let n = self.positions.len();
        // elastic restoration
        for i in 0..n {
            for k in 0..3 {
                let disp = self.positions[i][k] - self.rest_positions[i][k];
                let spring_f = -self.stiffness * disp;
                let damp_f = -self.damping * self.velocities[i][k];
                let accel = (spring_f + damp_f) / self.mass;
                self.velocities[i][k] += accel * dt;
            }
        }
        for i in 0..n {
            for k in 0..3 {
                self.positions[i][k] += self.velocities[i][k] * dt;
            }
        }
    }

    pub fn total_kinetic_energy(&self) -> f32 {
        self.velocities
            .iter()
            .map(|v| {
                let v2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
                0.5 * self.mass * v2
            })
            .sum()
    }

    pub fn total_potential_energy(&self) -> f32 {
        self.positions
            .iter()
            .zip(self.rest_positions.iter())
            .map(|(p, r)| {
                let dx = p[0] - r[0];
                let dy = p[1] - r[1];
                let dz = p[2] - r[2];
                0.5 * self.stiffness * (dx * dx + dy * dy + dz * dz)
            })
            .sum()
    }

    pub fn center_of_mass(&self) -> [f32; 3] {
        if self.positions.is_empty() {
            return [0.0; 3];
        }
        let n = self.positions.len() as f32;
        let mut c = [0.0_f32; 3];
        for p in &self.positions {
            c[0] += p[0];
            c[1] += p[1];
            c[2] += p[2];
        }
        [c[0] / n, c[1] / n, c[2] / n]
    }

    pub fn stiffness(&self) -> f32 {
        self.stiffness
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }

    /// Natural frequency for a single oscillator (rad/s).
    pub fn natural_frequency(&self) -> f32 {
        let _ = PI; // reference constant
        (self.stiffness / self.mass).sqrt()
    }

    pub fn reset(&mut self) {
        self.positions = self.rest_positions.clone();
        for v in &mut self.velocities {
            *v = [0.0; 3];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_body() -> ElasticBody {
        ElasticBody::new(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]], 1.0, 100.0, 5.0)
    }

    #[test]
    fn test_new() {
        let b = simple_body();
        assert_eq!(b.particle_count(), 2);
    }

    #[test]
    fn test_position() {
        let b = simple_body();
        assert_eq!(b.position(0), Some([0.0, 0.0, 0.0]));
    }

    #[test]
    fn test_set_position() {
        let mut b = simple_body();
        b.set_position(0, [5.0, 0.0, 0.0]);
        assert_eq!(b.position(0), Some([5.0, 0.0, 0.0]));
    }

    #[test]
    fn test_step_returns_to_rest() {
        let mut b = simple_body();
        b.set_position(0, [2.0, 0.0, 0.0]);
        for _ in 0..1000 {
            b.step(0.01);
        }
        let p = b.position(0).expect("should succeed");
        assert!(p[0].abs() < 0.5);
    }

    #[test]
    fn test_kinetic_energy_at_rest() {
        let b = simple_body();
        assert!((b.total_kinetic_energy() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_potential_energy_at_rest() {
        let b = simple_body();
        assert!((b.total_potential_energy() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_potential_energy_displaced() {
        let mut b = simple_body();
        b.set_position(0, [1.0, 0.0, 0.0]);
        assert!(b.total_potential_energy() > 0.0);
    }

    #[test]
    fn test_center_of_mass() {
        let b = simple_body();
        let c = b.center_of_mass();
        assert!((c[0] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_reset() {
        let mut b = simple_body();
        b.set_position(0, [10.0, 10.0, 10.0]);
        b.reset();
        assert_eq!(b.position(0), Some([0.0, 0.0, 0.0]));
    }

    #[test]
    fn test_natural_frequency() {
        let b = ElasticBody::new(vec![[0.0; 3]], 1.0, 100.0, 0.0);
        assert!((b.natural_frequency() - 10.0).abs() < 0.01);
    }
}
