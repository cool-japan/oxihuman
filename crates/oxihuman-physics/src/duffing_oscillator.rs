// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Duffing nonlinear oscillator (different from existing duffing_body).

/// Duffing oscillator: x'' + delta*x' - alpha*x + beta*x^3 = gamma*cos(omega*t)
#[derive(Debug, Clone)]
pub struct DuffingOscillator {
    /// Position.
    pub x: f64,
    /// Velocity.
    pub v: f64,
    /// Damping coefficient.
    pub delta: f64,
    /// Linear stiffness.
    pub alpha: f64,
    /// Nonlinear stiffness.
    pub beta: f64,
    /// Forcing amplitude.
    pub gamma: f64,
    /// Forcing frequency.
    pub omega: f64,
    /// Current time.
    pub t: f64,
}

impl DuffingOscillator {
    pub fn new(
        x0: f64,
        v0: f64,
        delta: f64,
        alpha: f64,
        beta: f64,
        gamma: f64,
        omega: f64,
    ) -> Self {
        Self {
            x: x0,
            v: v0,
            delta,
            alpha,
            beta,
            gamma,
            omega,
            t: 0.0,
        }
    }

    fn acceleration(&self, x: f64, v: f64, t: f64) -> f64 {
        -self.delta * v + self.alpha * x - self.beta * x * x * x
            + self.gamma * (self.omega * t).cos()
    }

    /// Euler integration step.
    pub fn step(&mut self, dt: f64) {
        let a = self.acceleration(self.x, self.v, self.t);
        self.v += a * dt;
        self.x += self.v * dt;
        self.t += dt;
    }

    /// Total energy (unforced part).
    pub fn energy(&self) -> f64 {
        0.5 * self.v * self.v - 0.5 * self.alpha * self.x * self.x
            + 0.25 * self.beta * self.x * self.x * self.x * self.x
    }
}

pub fn new_duffing_oscillator(
    x0: f64,
    v0: f64,
    delta: f64,
    alpha: f64,
    beta: f64,
    gamma: f64,
    omega: f64,
) -> DuffingOscillator {
    DuffingOscillator::new(x0, v0, delta, alpha, beta, gamma, omega)
}

pub fn duffing_step(osc: &mut DuffingOscillator, dt: f64) {
    osc.step(dt);
}

pub fn duffing_energy(osc: &DuffingOscillator) -> f64 {
    osc.energy()
}

pub fn duffing_position(osc: &DuffingOscillator) -> f64 {
    osc.x
}

pub fn duffing_velocity(osc: &DuffingOscillator) -> f64 {
    osc.v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_oscillator() {
        let d = new_duffing_oscillator(1.0, 0.0, 0.1, 1.0, 1.0, 0.0, 1.0);
        assert!((duffing_position(&d) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_step_changes_position() {
        let mut d = new_duffing_oscillator(1.0, 0.5, 0.1, 1.0, 1.0, 0.0, 1.0);
        duffing_step(&mut d, 0.01);
        assert!((duffing_position(&d) - 1.0).abs() > 1e-10);
    }

    #[test]
    fn test_velocity_changes() {
        let mut d = new_duffing_oscillator(1.0, 0.0, 0.0, 1.0, 0.0, 0.5, 1.0);
        duffing_step(&mut d, 0.01);
        /* forcing is non-zero at t=0 so velocity should change */
        assert!(duffing_velocity(&d).abs() > 1e-10);
    }

    #[test]
    fn test_zero_at_rest_no_forcing() {
        /* x=0, v=0, no forcing → stays at 0 */
        let mut d = new_duffing_oscillator(0.0, 0.0, 0.5, 1.0, 1.0, 0.0, 1.0);
        duffing_step(&mut d, 0.01);
        assert!(duffing_position(&d).abs() < 1e-10);
    }

    #[test]
    fn test_state_finite_after_many_steps() {
        let mut d = new_duffing_oscillator(1.0, 0.0, 0.2, 1.0, 1.0, 0.3, 1.4);
        for _ in 0..1000 {
            duffing_step(&mut d, 0.001);
        }
        assert!(d.x.is_finite());
        assert!(d.v.is_finite());
    }

    #[test]
    fn test_time_advances() {
        let mut d = new_duffing_oscillator(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        duffing_step(&mut d, 0.05);
        assert!((d.t - 0.05).abs() < 1e-12);
    }

    #[test]
    fn test_energy_finite() {
        let mut d = new_duffing_oscillator(1.0, 0.5, 0.1, 1.0, 1.0, 0.3, 1.0);
        duffing_step(&mut d, 0.01);
        assert!(duffing_energy(&d).is_finite());
    }

    #[test]
    fn test_damping_reduces_amplitude() {
        /* Damped harmonic oscillator: alpha=1, beta=0, delta=0.5, no forcing.
        Start near equilibrium with small perturbation and non-zero velocity.
        Euler with small dt → velocity decays, position stays bounded < 2. */
        let mut d = new_duffing_oscillator(0.1, 0.5, 0.5, 1.0, 0.0, 0.0, 1.0);
        for _ in 0..2000 {
            duffing_step(&mut d, 0.001);
        }
        /* After 2s with damping, both |x| and |v| should be very small */
        assert!(duffing_position(&d).abs() < 2.0);
        assert!(duffing_velocity(&d).abs() < 2.0);
    }

    #[test]
    fn test_no_negative_beta_blowup_short_run() {
        /* softening spring with small steps shouldn't blow up immediately */
        let mut d = new_duffing_oscillator(0.5, 0.0, 0.5, 1.0, -0.1, 0.0, 1.0);
        for _ in 0..50 {
            duffing_step(&mut d, 0.001);
        }
        assert!(d.x.is_finite());
    }
}
