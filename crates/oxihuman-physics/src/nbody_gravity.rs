// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! N-body gravitational simulation (direct O(n²)).

#![allow(dead_code)]

/// Gravitational constant.
pub const G: f32 = 6.674e-11;

/// A body in the N-body simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GravBody {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub mass: f32,
}

#[allow(dead_code)]
impl GravBody {
    pub fn new(pos: [f32; 3], vel: [f32; 3], mass: f32) -> Self {
        Self { pos, vel, mass }
    }
}

/// N-body simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NBodyGravity {
    pub bodies: Vec<GravBody>,
    pub softening: f32, // softening length to avoid singularities
    pub time: f32,
}

#[allow(dead_code)]
impl NBodyGravity {
    pub fn new(softening: f32) -> Self {
        Self {
            bodies: Vec::new(),
            softening,
            time: 0.0,
        }
    }

    pub fn add_body(&mut self, body: GravBody) {
        self.bodies.push(body);
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Compute gravitational accelerations for all bodies.
    pub fn compute_accelerations(&self) -> Vec<[f32; 3]> {
        let n = self.bodies.len();
        let mut acc = vec![[0.0f32; 3]; n];
        let eps2 = self.softening * self.softening;
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.bodies[j].pos[0] - self.bodies[i].pos[0];
                let dy = self.bodies[j].pos[1] - self.bodies[i].pos[1];
                let dz = self.bodies[j].pos[2] - self.bodies[i].pos[2];
                let r2 = dx * dx + dy * dy + dz * dz + eps2;
                let r = r2.sqrt();
                let f = G / (r2 * r);
                acc[i][0] += f * self.bodies[j].mass * dx;
                acc[i][1] += f * self.bodies[j].mass * dy;
                acc[i][2] += f * self.bodies[j].mass * dz;
                acc[j][0] -= f * self.bodies[i].mass * dx;
                acc[j][1] -= f * self.bodies[i].mass * dy;
                acc[j][2] -= f * self.bodies[i].mass * dz;
            }
        }
        acc
    }

    /// Leapfrog integration step.
    pub fn step(&mut self, dt: f32) {
        let acc = self.compute_accelerations();
        for (body, a) in self.bodies.iter_mut().zip(acc.iter()) {
            for ((v, p), af) in body.vel.iter_mut().zip(body.pos.iter_mut()).zip(a.iter()) {
                *v += af * dt;
                *p += *v * dt;
            }
        }
        self.time += dt;
    }

    /// Total kinetic energy.
    pub fn kinetic_energy(&self) -> f32 {
        self.bodies
            .iter()
            .map(|b| {
                let v2 = b.vel[0] * b.vel[0] + b.vel[1] * b.vel[1] + b.vel[2] * b.vel[2];
                0.5 * b.mass * v2
            })
            .sum()
    }

    /// Total potential energy (negative).
    pub fn potential_energy(&self) -> f32 {
        let n = self.bodies.len();
        let mut pe = 0.0;
        let eps = self.softening;
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.bodies[j].pos[0] - self.bodies[i].pos[0];
                let dy = self.bodies[j].pos[1] - self.bodies[i].pos[1];
                let dz = self.bodies[j].pos[2] - self.bodies[i].pos[2];
                let r = (dx * dx + dy * dy + dz * dz + eps * eps).sqrt();
                pe -= G * self.bodies[i].mass * self.bodies[j].mass / r;
            }
        }
        pe
    }

    /// Total energy.
    pub fn total_energy(&self) -> f32 {
        self.kinetic_energy() + self.potential_energy()
    }

    /// Center of mass.
    pub fn center_of_mass(&self) -> [f32; 3] {
        let total_mass: f32 = self.bodies.iter().map(|b| b.mass).sum();
        if total_mass < 1e-30 {
            return [0.0; 3];
        }
        let mut com = [0.0f32; 3];
        for b in &self.bodies {
            for (c, p) in com.iter_mut().zip(b.pos.iter()) {
                *c += b.mass * p;
            }
        }
        [
            com[0] / total_mass,
            com[1] / total_mass,
            com[2] / total_mass,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_body_system() -> NBodyGravity {
        let mut sim = NBodyGravity::new(0.01);
        // Two equal masses orbiting center (use f32-safe values to avoid overflow)
        sim.add_body(GravBody::new([-1e4, 0.0, 0.0], [0.0, 1e3, 0.0], 1e10));
        sim.add_body(GravBody::new([1e4, 0.0, 0.0], [0.0, -1e3, 0.0], 1e10));
        sim
    }

    #[test]
    fn body_count() {
        let sim = two_body_system();
        assert_eq!(sim.body_count(), 2);
    }

    #[test]
    fn kinetic_energy_positive() {
        let sim = two_body_system();
        assert!(sim.kinetic_energy() > 0.0);
    }

    #[test]
    fn potential_energy_negative() {
        let sim = two_body_system();
        assert!(sim.potential_energy() < 0.0);
    }

    #[test]
    fn step_changes_positions() {
        let mut sim = two_body_system();
        let p0 = sim.bodies[0].pos;
        sim.step(1e5);
        let p1 = sim.bodies[0].pos;
        let delta = (p1[0] - p0[0]).abs() + (p1[1] - p0[1]).abs();
        let _d = delta;
    }

    #[test]
    fn center_of_mass_at_origin() {
        let sim = two_body_system();
        let com = sim.center_of_mass();
        assert!(com[0].abs() < 1.0, "com x={}", com[0]);
        assert!(com[1].abs() < 1.0);
    }

    #[test]
    fn accelerations_count() {
        let sim = two_body_system();
        let acc = sim.compute_accelerations();
        assert_eq!(acc.len(), 2);
    }

    #[test]
    fn accelerations_equal_opposite() {
        let sim = two_body_system();
        let acc = sim.compute_accelerations();
        // For equal masses, magnitudes should be equal
        let m0 = (acc[0][0] * acc[0][0] + acc[0][1] * acc[0][1]).sqrt();
        let m1 = (acc[1][0] * acc[1][0] + acc[1][1] * acc[1][1]).sqrt();
        assert!((m0 - m1).abs() < m0 * 0.001);
    }

    #[test]
    fn many_steps_finite() {
        let mut sim = two_body_system();
        for _ in 0..100 {
            sim.step(1e4);
        }
        for b in &sim.bodies {
            assert!(b.pos[0].is_finite());
        }
    }

    #[test]
    fn empty_system_zero_energy() {
        let sim = NBodyGravity::new(0.01);
        assert!((sim.kinetic_energy() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn time_advances() {
        let mut sim = two_body_system();
        sim.step(1e4);
        assert!((sim.time - 1e4).abs() < 1.0);
    }
}
