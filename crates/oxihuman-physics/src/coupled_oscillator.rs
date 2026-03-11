// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! N coupled harmonic oscillators (tridiagonal spring chain).

#![allow(dead_code)]

/// N-coupled oscillator chain.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CoupledOscillators {
    pub n: usize,
    pub positions: Vec<f32>,
    pub velocities: Vec<f32>,
    pub masses: Vec<f32>,
    pub spring_k: f32, // coupling spring constant
    pub wall_k: f32,   // springs to fixed walls at endpoints
    pub damping: f32,
}

#[allow(dead_code)]
impl CoupledOscillators {
    pub fn new(n: usize, mass: f32, spring_k: f32) -> Self {
        Self {
            n,
            positions: vec![0.0; n],
            velocities: vec![0.0; n],
            masses: vec![mass; n],
            spring_k,
            wall_k: spring_k,
            damping: 0.0,
        }
    }

    /// Set initial displacement of oscillator i.
    pub fn set_displacement(&mut self, i: usize, x: f32) {
        self.positions[i] = x;
    }

    /// Compute forces on each oscillator.
    #[allow(clippy::needless_range_loop)]
    pub fn compute_forces(&self) -> Vec<f32> {
        let mut forces = vec![0.0f32; self.n];
        for i in 0..self.n {
            // Left coupling (or wall at i=0)
            let x_left = if i == 0 { 0.0 } else { self.positions[i - 1] };
            let k_left = if i == 0 { self.wall_k } else { self.spring_k };
            forces[i] += k_left * (x_left - self.positions[i]);
            // Right coupling (or wall at i=n-1)
            let x_right = if i == self.n - 1 {
                0.0
            } else {
                self.positions[i + 1]
            };
            let k_right = if i == self.n - 1 {
                self.wall_k
            } else {
                self.spring_k
            };
            forces[i] += k_right * (x_right - self.positions[i]);
            // Damping
            forces[i] -= self.damping * self.velocities[i];
        }
        forces
    }

    /// RK4 step.
    pub fn step(&mut self, dt: f32) {
        // We use symplectic Euler for simplicity (energy-conserving)
        let forces = self.compute_forces();
        for ((v, f), m) in self
            .velocities
            .iter_mut()
            .zip(forces.iter())
            .zip(self.masses.iter())
        {
            let a = f / m.max(1e-10);
            *v += a * dt;
        }
        for (p, v) in self.positions.iter_mut().zip(self.velocities.iter()) {
            *p += v * dt;
        }
    }

    /// Total kinetic energy.
    pub fn kinetic_energy(&self) -> f32 {
        self.masses
            .iter()
            .zip(self.velocities.iter())
            .map(|(m, v)| 0.5 * m * v * v)
            .sum()
    }

    /// Total potential energy.
    #[allow(clippy::needless_range_loop)]
    pub fn potential_energy(&self) -> f32 {
        let mut pe = 0.0;
        for i in 0..self.n {
            let x_left = if i == 0 { 0.0 } else { self.positions[i - 1] };
            let dx = self.positions[i] - x_left;
            pe += 0.5 * self.spring_k * dx * dx;
        }
        // Right wall
        let dx_right = self.positions[self.n - 1];
        pe += 0.5 * self.wall_k * dx_right * dx_right;
        pe
    }

    /// Total energy.
    pub fn total_energy(&self) -> f32 {
        self.kinetic_energy() + self.potential_energy()
    }

    /// Center of mass displacement.
    pub fn center_of_mass(&self) -> f32 {
        let total_m: f32 = self.masses.iter().sum();
        if total_m < 1e-10 {
            return 0.0;
        }
        self.masses
            .iter()
            .zip(self.positions.iter())
            .map(|(m, x)| m * x)
            .sum::<f32>()
            / total_m
    }

    /// Normal mode frequencies (analytic for uniform chain).
    pub fn normal_frequencies(&self) -> Vec<f32> {
        use std::f32::consts::PI;
        let m = self.masses[0];
        let k = self.spring_k;
        (1..=self.n)
            .map(|p| {
                let arg = PI * p as f32 / (2.0 * (self.n + 1) as f32);
                2.0 * (k / m).sqrt() * arg.sin()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_zero_energy() {
        let osc = CoupledOscillators::new(5, 1.0, 10.0);
        assert!((osc.kinetic_energy() - 0.0).abs() < 1e-5);
    }

    #[test]
    fn displacement_then_step() {
        let mut osc = CoupledOscillators::new(5, 1.0, 10.0);
        osc.set_displacement(2, 1.0);
        osc.step(0.01);
        assert!(osc.positions[2].is_finite());
    }

    #[test]
    fn total_energy_initially_pe_only() {
        let mut osc = CoupledOscillators::new(3, 1.0, 10.0);
        osc.set_displacement(1, 1.0);
        let ke = osc.kinetic_energy();
        assert!(ke.abs() < 1e-5);
    }

    #[test]
    fn many_steps_finite() {
        let mut osc = CoupledOscillators::new(5, 1.0, 10.0);
        osc.set_displacement(0, 1.0);
        for _ in 0..500 {
            osc.step(0.001);
        }
        for &x in &osc.positions {
            assert!(x.is_finite());
        }
    }

    #[test]
    fn forces_symmetric_chain() {
        let mut osc = CoupledOscillators::new(3, 1.0, 10.0);
        osc.set_displacement(1, 1.0); // middle oscillator displaced
        let f = osc.compute_forces();
        // Force on middle should be negative (restoring)
        assert!(f[1] < 0.0, "f[1]={}", f[1]);
    }

    #[test]
    fn normal_frequencies_count() {
        let osc = CoupledOscillators::new(5, 1.0, 10.0);
        let freqs = osc.normal_frequencies();
        assert_eq!(freqs.len(), 5);
    }

    #[test]
    fn normal_frequencies_positive() {
        let osc = CoupledOscillators::new(4, 1.0, 10.0);
        for f in osc.normal_frequencies() {
            assert!(f > 0.0);
        }
    }

    #[test]
    fn normal_frequencies_increasing() {
        let osc = CoupledOscillators::new(4, 1.0, 10.0);
        let freqs = osc.normal_frequencies();
        for i in 0..freqs.len() - 1 {
            assert!(freqs[i] < freqs[i + 1]);
        }
    }

    #[test]
    fn center_of_mass_zero_at_rest() {
        let osc = CoupledOscillators::new(5, 1.0, 10.0);
        assert!(osc.center_of_mass().abs() < 1e-5);
    }

    #[test]
    fn energy_conserved_no_damping() {
        let mut osc = CoupledOscillators::new(4, 1.0, 10.0);
        osc.set_displacement(0, 1.0);
        let e0 = osc.total_energy();
        for _ in 0..500 {
            osc.step(0.001);
        }
        let e1 = osc.total_energy();
        assert!((e1 - e0).abs() < e0 * 0.05, "energy changed: {e0} -> {e1}");
    }
}
