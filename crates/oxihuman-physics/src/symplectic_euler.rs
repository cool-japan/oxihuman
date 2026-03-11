// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Symplectic Euler integrator for Hamiltonian systems.

#![allow(dead_code)]

/// A particle for symplectic Euler integration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SympParticle {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub mass: f32,
}

#[allow(dead_code)]
impl SympParticle {
    pub fn new(pos: [f32; 3], vel: [f32; 3], mass: f32) -> Self {
        Self { pos, vel, mass }
    }

    /// Symplectic Euler step: update velocity first, then position.
    /// force: force vector at current state.
    pub fn step(&mut self, force: [f32; 3], dt: f32) {
        let inv_m = 1.0 / self.mass.max(1e-10);
        // Update velocity first (symplectic)
        for (v, f) in self.vel.iter_mut().zip(force.iter()) {
            *v += f * inv_m * dt;
        }
        // Then update position using new velocity
        for (p, v) in self.pos.iter_mut().zip(self.vel.iter()) {
            *p += v * dt;
        }
    }

    /// Kinetic energy.
    pub fn kinetic_energy(&self) -> f32 {
        let v2 = self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1] + self.vel[2] * self.vel[2];
        0.5 * self.mass * v2
    }
}

/// Symplectic Euler for a 1D harmonic oscillator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SympOscillator1D {
    pub q: f32, // position
    pub p: f32, // momentum
    pub m: f32, // mass
    pub k: f32, // spring constant
}

#[allow(dead_code)]
impl SympOscillator1D {
    pub fn new(q0: f32, p0: f32, m: f32, k: f32) -> Self {
        Self { q: q0, p: p0, m, k }
    }

    /// Symplectic Euler step.
    pub fn step(&mut self, dt: f32) {
        // p_{n+1} = p_n - k*q_n*dt  (update momentum first)
        self.p -= self.k * self.q * dt;
        // q_{n+1} = q_n + (p_{n+1}/m)*dt  (then position)
        self.q += self.p / self.m * dt;
    }

    /// Total energy H = p²/(2m) + k*q²/2.
    pub fn energy(&self) -> f32 {
        self.p * self.p / (2.0 * self.m) + self.k * self.q * self.q / 2.0
    }

    /// Angular frequency.
    pub fn omega(&self) -> f32 {
        (self.k / self.m).sqrt()
    }

    /// Period.
    pub fn period(&self) -> f32 {
        use std::f32::consts::PI;
        2.0 * PI / self.omega()
    }
}

/// Integrate a 1D harmonic oscillator for `steps`, return (q, p) trajectory.
#[allow(dead_code)]
pub fn symp_oscillator_trajectory(
    q0: f32,
    p0: f32,
    m: f32,
    k: f32,
    dt: f32,
    steps: usize,
) -> Vec<[f32; 2]> {
    let mut osc = SympOscillator1D::new(q0, p0, m, k);
    let mut traj = Vec::with_capacity(steps + 1);
    traj.push([osc.q, osc.p]);
    for _ in 0..steps {
        osc.step(dt);
        traj.push([osc.q, osc.p]);
    }
    traj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symp_particle_kinetic_energy() {
        let p = SympParticle::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 2.0);
        assert!((p.kinetic_energy() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn symp_particle_step_no_force() {
        let mut p = SympParticle::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0);
        p.step([0.0, 0.0, 0.0], 1.0);
        assert!((p.pos[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn symp_particle_step_with_force() {
        let mut p = SympParticle::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        p.step([1.0, 0.0, 0.0], 1.0);
        assert!(p.pos[0] > 0.0);
    }

    #[test]
    fn oscillator_energy_near_conserved() {
        let mut osc = SympOscillator1D::new(1.0, 0.0, 1.0, 1.0);
        let e0 = osc.energy();
        for _ in 0..1000 {
            osc.step(0.01);
        }
        let e1 = osc.energy();
        assert!((e1 - e0).abs() < 0.05, "energy drift: {e0} -> {e1}");
    }

    #[test]
    fn oscillator_period() {
        let osc = SympOscillator1D::new(1.0, 0.0, 1.0, 4.0);
        let period = osc.period();
        use std::f32::consts::PI;
        assert!((period - PI).abs() < 0.01);
    }

    #[test]
    fn trajectory_length() {
        let traj = symp_oscillator_trajectory(1.0, 0.0, 1.0, 1.0, 0.01, 100);
        assert_eq!(traj.len(), 101);
    }

    #[test]
    fn trajectory_finite() {
        let traj = symp_oscillator_trajectory(1.0, 0.0, 1.0, 1.0, 0.01, 100);
        for p in &traj {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn symp_particle_velocity_changes_with_force() {
        let mut p = SympParticle::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        p.step([5.0, 0.0, 0.0], 1.0);
        assert!((p.vel[0] - 5.0).abs() < 1e-4);
    }

    #[test]
    fn omega_correct() {
        let osc = SympOscillator1D::new(1.0, 0.0, 1.0, 4.0);
        assert!((osc.omega() - 2.0).abs() < 1e-4);
    }

    #[test]
    fn symp_oscillation_returns_to_start() {
        let osc = SympOscillator1D::new(1.0, 0.0, 1.0, 1.0);
        let period = osc.period();
        let mut osc2 = osc.clone();
        let steps = (period / 0.001) as usize;
        for _ in 0..steps {
            osc2.step(0.001);
        }
        assert!((osc2.q - 1.0).abs() < 0.05, "q after period: {}", osc2.q);
    }
}
