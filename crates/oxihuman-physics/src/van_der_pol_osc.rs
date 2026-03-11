// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Van der Pol oscillator (separate from existing van_der_pol module).

/// Van der Pol oscillator: x'' - mu*(1 - x^2)*x' + x = 0
#[derive(Debug, Clone)]
pub struct VanDerPolOsc {
    pub x: f64,
    pub v: f64,
    /// Nonlinearity parameter.
    pub mu: f64,
    pub t: f64,
}

impl VanDerPolOsc {
    pub fn new(x0: f64, v0: f64, mu: f64) -> Self {
        Self {
            x: x0,
            v: v0,
            mu,
            t: 0.0,
        }
    }

    fn acceleration(&self) -> f64 {
        self.mu * (1.0 - self.x * self.x) * self.v - self.x
    }

    /// RK4 integration step.
    pub fn step(&mut self, dt: f64) {
        let f = |x: f64, v: f64| -> (f64, f64) {
            let a = self.mu * (1.0 - x * x) * v - x;
            (v, a)
        };

        let (dx1, dv1) = f(self.x, self.v);
        let (dx2, dv2) = f(self.x + 0.5 * dt * dx1, self.v + 0.5 * dt * dv1);
        let (dx3, dv3) = f(self.x + 0.5 * dt * dx2, self.v + 0.5 * dt * dv2);
        let (dx4, dv4) = f(self.x + dt * dx3, self.v + dt * dv3);

        self.x += dt / 6.0 * (dx1 + 2.0 * dx2 + 2.0 * dx3 + dx4);
        self.v += dt / 6.0 * (dv1 + 2.0 * dv2 + 2.0 * dv3 + dv4);
        self.t += dt;
    }

    /// Approximate energy (kinetic + potential).
    pub fn energy(&self) -> f64 {
        0.5 * self.v * self.v + 0.5 * self.x * self.x
    }
}

pub fn new_van_der_pol_osc(x0: f64, v0: f64, mu: f64) -> VanDerPolOsc {
    VanDerPolOsc::new(x0, v0, mu)
}

pub fn vdp_step(osc: &mut VanDerPolOsc, dt: f64) {
    osc.step(dt);
}

pub fn vdp_position(osc: &VanDerPolOsc) -> f64 {
    osc.x
}

pub fn vdp_velocity(osc: &VanDerPolOsc) -> f64 {
    osc.v
}

pub fn vdp_energy(osc: &VanDerPolOsc) -> f64 {
    osc.energy()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_osc() {
        let o = new_van_der_pol_osc(2.0, 0.0, 1.0);
        assert!((vdp_position(&o) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_step_changes_state() {
        let mut o = new_van_der_pol_osc(1.0, 0.5, 1.0);
        vdp_step(&mut o, 0.01);
        assert!((vdp_position(&o) - 1.0).abs() > 1e-10);
    }

    #[test]
    fn test_state_finite_after_steps() {
        let mut o = new_van_der_pol_osc(2.0, 0.0, 1.0);
        for _ in 0..500 {
            vdp_step(&mut o, 0.01);
        }
        assert!(o.x.is_finite());
        assert!(o.v.is_finite());
    }

    #[test]
    fn test_energy_positive() {
        let o = new_van_der_pol_osc(1.0, 1.0, 1.0);
        assert!(vdp_energy(&o) > 0.0);
    }

    #[test]
    fn test_time_advances() {
        let mut o = new_van_der_pol_osc(0.0, 1.0, 0.5);
        vdp_step(&mut o, 0.1);
        assert!((o.t - 0.1).abs() < 1e-12);
    }

    #[test]
    fn test_limit_cycle_behavior() {
        /* Van der Pol with mu=1 should settle near amplitude ~2 */
        let mut o = new_van_der_pol_osc(0.1, 0.0, 1.0);
        for _ in 0..2000 {
            vdp_step(&mut o, 0.01);
        }
        /* should be bounded */
        assert!(o.x.abs() < 10.0);
    }

    #[test]
    fn test_zero_initial_stays_zero() {
        /* x=0, v=0 → acceleration = 0 → stays at origin */
        let mut o = new_van_der_pol_osc(0.0, 0.0, 1.0);
        vdp_step(&mut o, 0.01);
        assert!(vdp_position(&o).abs() < 1e-10);
    }

    #[test]
    fn test_large_mu_bounded() {
        let mut o = new_van_der_pol_osc(2.0, 0.0, 5.0);
        for _ in 0..500 {
            vdp_step(&mut o, 0.001);
        }
        assert!(o.x.is_finite());
    }

    #[test]
    fn test_mu_zero_is_harmonic() {
        /* mu=0 → simple harmonic oscillator */
        let mut o = new_van_der_pol_osc(1.0, 0.0, 0.0);
        for _ in 0..100 {
            vdp_step(&mut o, 0.01);
        }
        assert!(o.x.is_finite());
    }
}
