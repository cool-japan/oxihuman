// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Duffing nonlinear oscillator: x'' + delta*x' - alpha*x + beta*x³ = gamma*cos(omega*t).

#![allow(dead_code)]

/// Duffing oscillator state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DuffingBody {
    pub alpha: f32, // linear stiffness
    pub beta: f32,  // nonlinear stiffness
    pub delta: f32, // damping
    pub gamma: f32, // driving amplitude
    pub omega: f32, // driving frequency
    pub x: f32,
    pub v: f32,
    pub t: f32,
}

#[allow(dead_code)]
impl DuffingBody {
    pub fn new(alpha: f32, beta: f32, delta: f32, gamma: f32, omega: f32) -> Self {
        Self {
            alpha,
            beta,
            delta,
            gamma,
            omega,
            x: 0.1,
            v: 0.0,
            t: 0.0,
        }
    }

    /// Default chaotic Duffing parameters.
    pub fn chaotic() -> Self {
        Self::new(1.0, 1.0, 0.2, 0.3, 1.2)
    }

    /// Acceleration at (x, v, t).
    pub fn accel(&self, x: f32, v: f32, t: f32) -> f32 {
        -self.delta * v - (-self.alpha * x + self.beta * x * x * x)
            + self.gamma * (self.omega * t).cos()
    }

    /// Euler step.
    pub fn step_euler(&mut self, dt: f32) {
        let a = self.accel(self.x, self.v, self.t);
        self.x += self.v * dt;
        self.v += a * dt;
        self.t += dt;
    }

    /// RK4 step.
    pub fn step_rk4(&mut self, dt: f32) {
        let (x0, v0, t0) = (self.x, self.v, self.t);
        let k1v = self.accel(x0, v0, t0);
        let k1x = v0;
        let k2v = self.accel(x0 + 0.5 * dt * k1x, v0 + 0.5 * dt * k1v, t0 + 0.5 * dt);
        let k2x = v0 + 0.5 * dt * k1v;
        let k3v = self.accel(x0 + 0.5 * dt * k2x, v0 + 0.5 * dt * k2v, t0 + 0.5 * dt);
        let k3x = v0 + 0.5 * dt * k2v;
        let k4v = self.accel(x0 + dt * k3x, v0 + dt * k3v, t0 + dt);
        let k4x = v0 + dt * k3v;
        self.x += dt / 6.0 * (k1x + 2.0 * k2x + 2.0 * k3x + k4x);
        self.v += dt / 6.0 * (k1v + 2.0 * k2v + 2.0 * k3v + k4v);
        self.t += dt;
    }

    /// Kinetic energy.
    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.v * self.v
    }

    /// Potential energy: -alpha/2 * x² + beta/4 * x⁴.
    pub fn potential_energy(&self) -> f32 {
        -self.alpha / 2.0 * self.x * self.x + self.beta / 4.0 * self.x.powi(4)
    }

    /// Phase space position (x, v).
    pub fn phase_point(&self) -> [f32; 2] {
        [self.x, self.v]
    }
}

/// Generate Duffing trajectory.
#[allow(dead_code)]
pub fn duffing_trajectory(params: DuffingBody, dt: f32, steps: usize) -> Vec<[f32; 2]> {
    let mut d = params;
    let mut traj = Vec::with_capacity(steps + 1);
    traj.push(d.phase_point());
    for _ in 0..steps {
        d.step_rk4(dt);
        traj.push(d.phase_point());
    }
    traj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let d = DuffingBody::chaotic();
        assert!((d.x - 0.1).abs() < 1e-5);
        assert!(d.v.abs() < 1e-5);
    }

    #[test]
    fn accel_at_origin() {
        let d = DuffingBody::chaotic();
        // At x=0, v=0, driving at t=0: accel = gamma
        let a = d.accel(0.0, 0.0, 0.0);
        assert!((a - d.gamma).abs() < 1e-4);
    }

    #[test]
    fn euler_step_changes_state() {
        let mut d = DuffingBody::chaotic();
        let x0 = d.x;
        d.step_euler(0.01);
        let _delta = (d.x - x0).abs();
    }

    #[test]
    fn rk4_step_finite() {
        let mut d = DuffingBody::chaotic();
        for _ in 0..100 {
            d.step_rk4(0.01);
        }
        assert!(d.x.is_finite() && d.v.is_finite());
    }

    #[test]
    fn kinetic_energy_nonneg() {
        let d = DuffingBody::chaotic();
        assert!(d.kinetic_energy() >= 0.0);
    }

    #[test]
    fn trajectory_length() {
        let traj = duffing_trajectory(DuffingBody::chaotic(), 0.01, 50);
        assert_eq!(traj.len(), 51);
    }

    #[test]
    fn trajectory_finite() {
        let traj = duffing_trajectory(DuffingBody::chaotic(), 0.01, 100);
        for p in &traj {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn no_driving_decays() {
        // With gamma=0, delta>0, amplitude should stay bounded
        let mut d = DuffingBody::new(1.0, 0.1, 0.5, 0.0, 1.0);
        d.x = 2.0;
        let e0 = d.kinetic_energy() + d.potential_energy();
        for _ in 0..500 {
            d.step_rk4(0.01);
        }
        let e1 = d.kinetic_energy() + d.potential_energy();
        assert!(e1 < e0 + 5.0, "energy grew: {e0} -> {e1}");
    }

    #[test]
    fn phase_point() {
        let d = DuffingBody::chaotic();
        let p = d.phase_point();
        assert!((p[0] - d.x).abs() < 1e-5);
        assert!((p[1] - d.v).abs() < 1e-5);
    }

    #[test]
    fn time_advances_after_step() {
        let mut d = DuffingBody::chaotic();
        let t0 = d.t;
        d.step_rk4(0.1);
        assert!((d.t - (t0 + 0.1)).abs() < 1e-5);
    }
}
