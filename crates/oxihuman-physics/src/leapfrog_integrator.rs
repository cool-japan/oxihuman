// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Leapfrog/Störmer-Verlet integrator.

#![allow(dead_code)]

/// A particle for leapfrog integration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LeapfrogParticle {
    pub pos: [f32; 3],
    /// Half-step velocity (leapfrog offset).
    pub vel_half: [f32; 3],
    pub mass: f32,
    /// Last full-step velocity (for energy calculation).
    pub vel: [f32; 3],
}

#[allow(dead_code)]
impl LeapfrogParticle {
    pub fn new(pos: [f32; 3], vel: [f32; 3], mass: f32) -> Self {
        Self {
            pos,
            vel_half: vel,
            mass,
            vel,
        }
    }

    /// Kick-drift-kick leapfrog step.
    pub fn step(&mut self, force: [f32; 3], dt: f32) {
        let inv_m = 1.0 / self.mass.max(1e-10);
        // Kick: v_{n+1/2} = v_{n-1/2} + a_n * dt
        for (vh, f) in self.vel_half.iter_mut().zip(force.iter()) {
            *vh += f * inv_m * dt;
        }
        // Drift: x_{n+1} = x_n + v_{n+1/2} * dt
        for (p, vh) in self.pos.iter_mut().zip(self.vel_half.iter()) {
            *p += vh * dt;
        }
        // Store full-step velocity (avg of half-steps approximation)
        self.vel = self.vel_half;
    }

    /// Kinetic energy using full-step velocity.
    pub fn kinetic_energy(&self) -> f32 {
        let v2 = self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1] + self.vel[2] * self.vel[2];
        0.5 * self.mass * v2
    }
}

/// 1D leapfrog oscillator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LeapfrogOscillator {
    pub x: f32,
    pub v: f32, // half-step velocity
    pub mass: f32,
    pub k: f32, // spring constant
    pub initialized: bool,
}

#[allow(dead_code)]
impl LeapfrogOscillator {
    pub fn new(x0: f32, v0: f32, mass: f32, k: f32) -> Self {
        Self {
            x: x0,
            v: v0,
            mass,
            k,
            initialized: false,
        }
    }

    /// Initialize half-step velocity from full-step velocity.
    pub fn initialize(&mut self, dt: f32) {
        if !self.initialized {
            let a = -self.k * self.x / self.mass;
            self.v += 0.5 * a * dt;
            self.initialized = true;
        }
    }

    /// Leapfrog step (must call initialize first).
    pub fn step(&mut self, dt: f32) {
        if !self.initialized {
            self.initialize(dt);
        }
        // Position update
        self.x += self.v * dt;
        // Acceleration at new position
        let a = -self.k * self.x / self.mass;
        // Velocity kick
        self.v += a * dt;
    }

    /// Approximate energy (half-step velocity → slight mismatch).
    pub fn energy(&self) -> f32 {
        let ke = 0.5 * self.mass * self.v * self.v;
        let pe = 0.5 * self.k * self.x * self.x;
        ke + pe
    }

    /// Period of oscillation.
    pub fn period(&self) -> f32 {
        use std::f32::consts::PI;
        2.0 * PI * (self.mass / self.k).sqrt()
    }
}

/// Leapfrog trajectory for 1D oscillator.
#[allow(dead_code)]
pub fn leapfrog_trajectory(
    x0: f32,
    v0: f32,
    mass: f32,
    k: f32,
    dt: f32,
    steps: usize,
) -> Vec<[f32; 2]> {
    let mut osc = LeapfrogOscillator::new(x0, v0, mass, k);
    let mut traj = Vec::with_capacity(steps + 1);
    traj.push([osc.x, osc.v]);
    for _ in 0..steps {
        osc.step(dt);
        traj.push([osc.x, osc.v]);
    }
    traj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_kinetic_energy() {
        let p = LeapfrogParticle::new([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], 1.0);
        assert!((p.kinetic_energy() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn particle_step_gravity() {
        let mut p = LeapfrogParticle::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        p.step([0.0, -9.81, 0.0], 1.0);
        assert!(p.pos[1] < 0.0);
    }

    #[test]
    fn oscillator_period() {
        let osc = LeapfrogOscillator::new(1.0, 0.0, 1.0, 4.0);
        use std::f32::consts::PI;
        assert!((osc.period() - PI).abs() < 0.01);
    }

    #[test]
    fn oscillator_energy_near_conserved() {
        let mut osc = LeapfrogOscillator::new(1.0, 0.0, 1.0, 1.0);
        let e0 = osc.energy();
        for _ in 0..2000 {
            osc.step(0.001);
        }
        let e1 = osc.energy();
        assert!((e1 - e0).abs() < 0.05, "energy: {e0} -> {e1}");
    }

    #[test]
    fn trajectory_length() {
        let traj = leapfrog_trajectory(1.0, 0.0, 1.0, 1.0, 0.01, 100);
        assert_eq!(traj.len(), 101);
    }

    #[test]
    fn trajectory_finite() {
        let traj = leapfrog_trajectory(1.0, 0.0, 1.0, 1.0, 0.01, 100);
        for p in &traj {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn many_oscillator_steps() {
        let mut osc = LeapfrogOscillator::new(1.0, 0.0, 1.0, 1.0);
        for _ in 0..10000 {
            osc.step(0.001);
        }
        assert!(osc.x.is_finite());
    }

    #[test]
    fn initialized_flag() {
        let mut osc = LeapfrogOscillator::new(1.0, 0.0, 1.0, 1.0);
        assert!(!osc.initialized);
        osc.initialize(0.01);
        assert!(osc.initialized);
    }

    #[test]
    fn particle_no_force_constant_velocity() {
        let mut p = LeapfrogParticle::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0);
        p.step([0.0, 0.0, 0.0], 1.0);
        assert!((p.pos[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn oscillator_returns_near_start() {
        let osc0 = LeapfrogOscillator::new(1.0, 0.0, 1.0, 1.0);
        let period = osc0.period();
        let mut osc = LeapfrogOscillator::new(1.0, 0.0, 1.0, 1.0);
        let steps = (period / 0.001) as usize;
        for _ in 0..steps {
            osc.step(0.001);
        }
        assert!((osc.x - 1.0).abs() < 0.05, "x after one period: {}", osc.x);
    }
}
