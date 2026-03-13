// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Elastic wave propagation: 1-D wave equation on a particle chain.

/// A 1-D chain of particles for elastic wave simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ElasticChain {
    pub displacement: Vec<f32>,
    pub velocity: Vec<f32>,
    pub wave_speed: f32,
    pub damping: f32,
    pub length: f32,
}

#[allow(dead_code)]
impl ElasticChain {
    pub fn new(n: usize, wave_speed: f32, damping: f32, length: f32) -> Self {
        Self {
            displacement: vec![0.0; n],
            velocity: vec![0.0; n],
            wave_speed,
            damping,
            length,
        }
    }

    pub fn n(&self) -> usize {
        self.displacement.len()
    }

    pub fn dx(&self) -> f32 {
        if self.n() > 1 {
            self.length / (self.n() - 1) as f32
        } else {
            self.length
        }
    }

    /// Excite node `i` with displacement amplitude.
    pub fn excite(&mut self, i: usize, amplitude: f32) {
        if i < self.n() {
            self.displacement[i] += amplitude;
        }
    }

    /// Integrate one timestep (explicit finite differences, fixed boundaries).
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, dt: f32) {
        let n = self.n();
        if n < 3 {
            return;
        }
        let dx = self.dx();
        let c2 = self.wave_speed * self.wave_speed;
        let r2 = c2 * dt * dt / (dx * dx);

        let mut new_disp = self.displacement.clone();

        for i in 1..n - 1 {
            let d2u =
                self.displacement[i + 1] - 2.0 * self.displacement[i] + self.displacement[i - 1];
            self.velocity[i] += r2 * d2u / dt - self.damping * self.velocity[i] * dt;
            new_disp[i] = self.displacement[i] + self.velocity[i] * dt;
        }

        // Fixed boundary conditions
        new_disp[0] = 0.0;
        new_disp[n - 1] = 0.0;
        self.velocity[0] = 0.0;
        self.velocity[n - 1] = 0.0;

        self.displacement = new_disp;
    }

    pub fn max_displacement(&self) -> f32 {
        self.displacement.iter().cloned().fold(0.0f32, f32::max)
    }

    pub fn energy(&self) -> f32 {
        let ke: f32 = self.velocity.iter().map(|&v| 0.5 * v * v).sum();
        let dx = self.dx();
        let pe: f32 = (1..self.n())
            .map(|i| {
                let du = (self.displacement[i] - self.displacement[i - 1]) / dx;
                0.5 * self.wave_speed * self.wave_speed * du * du
            })
            .sum();
        ke + pe
    }

    pub fn reset(&mut self) {
        for v in &mut self.displacement {
            *v = 0.0;
        }
        for v in &mut self.velocity {
            *v = 0.0;
        }
    }
}

/// Theoretical phase velocity of a wave with frequency `f` and wavelength `lambda`.
#[allow(dead_code)]
pub fn phase_velocity(frequency: f32, wavelength: f32) -> f32 {
    frequency * wavelength
}

/// CFL condition: stable dt for wave speed `c` and grid spacing `dx`.
#[allow(dead_code)]
pub fn cfl_stable_dt(wave_speed: f32, dx: f32) -> f32 {
    dx / wave_speed.max(1e-9)
}

/// Standing wave mode frequency for string of length L, mode n, wave speed c.
#[allow(dead_code)]
pub fn standing_wave_frequency(mode: u32, length: f32, wave_speed: f32) -> f32 {
    mode as f32 * wave_speed / (2.0 * length)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn new_chain_zero_displacement() {
        let c = ElasticChain::new(10, 1.0, 0.0, 1.0);
        assert!(c.displacement.iter().all(|&d| d == 0.0));
    }

    #[test]
    fn excite_sets_displacement() {
        let mut c = ElasticChain::new(10, 1.0, 0.0, 1.0);
        c.excite(5, 1.0);
        assert!((c.displacement[5] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn step_propagates_wave() {
        let mut c = ElasticChain::new(20, 1.0, 0.0, 1.0);
        c.excite(10, 1.0);
        let before = c.max_displacement();
        c.step(0.01);
        // Wave should have spread - adjacent nodes now have displacement
        assert!(c.displacement.iter().any(|&d| d.abs() > 0.0 && d != before));
    }

    #[test]
    fn boundaries_fixed() {
        let mut c = ElasticChain::new(10, 1.0, 0.0, 1.0);
        c.excite(5, 1.0);
        c.step(0.01);
        assert!((c.displacement[0]).abs() < 1e-9);
        assert!((c.displacement[9]).abs() < 1e-9);
    }

    #[test]
    fn damping_reduces_energy() {
        let mut c = ElasticChain::new(20, 1.0, 1.0, 1.0);
        c.excite(10, 1.0);
        let e0 = c.energy();
        for _ in 0..50 {
            c.step(0.01);
        }
        let e1 = c.energy();
        // Damped, so energy should decrease (or be equal if already zero)
        assert!(e1 <= e0 + 1e-3);
    }

    #[test]
    fn reset_zeroes_all() {
        let mut c = ElasticChain::new(10, 1.0, 0.0, 1.0);
        c.excite(5, 1.0);
        c.step(0.01);
        c.reset();
        assert!(c.displacement.iter().all(|&d| d == 0.0));
        assert!(c.velocity.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn phase_velocity_formula() {
        let v = phase_velocity(440.0, 0.78);
        assert!((v - 440.0 * 0.78).abs() < 1e-3);
    }

    #[test]
    fn cfl_dt_formula() {
        let dt = cfl_stable_dt(340.0, 0.01);
        assert!((dt - 0.01 / 340.0).abs() < 1e-9);
    }

    #[test]
    fn standing_wave_fundamental() {
        // mode=1, L=1, c=1 → f=0.5
        let f = standing_wave_frequency(1, 1.0, 1.0);
        assert!((f - 0.5).abs() < 1e-5);
    }

    #[test]
    fn standing_wave_uses_pi_via_length() {
        // mode=1, L=PI, c=PI → f = PI / (2*PI) = 0.5
        let f = standing_wave_frequency(1, PI, PI);
        assert!((f - 0.5).abs() < 1e-5);
    }
}
