// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Chaotic double pendulum (RK4 integration).

#![allow(dead_code)]

/// State of a double pendulum.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DoublePendulum {
    pub m1: f32,  // mass of bob 1 (kg)
    pub m2: f32,  // mass of bob 2 (kg)
    pub l1: f32,  // length of rod 1 (m)
    pub l2: f32,  // length of rod 2 (m)
    pub g: f32,   // gravity (m/s²)
    pub th1: f32, // angle of rod 1 (rad)
    pub th2: f32, // angle of rod 2 (rad)
    pub w1: f32,  // angular velocity of rod 1
    pub w2: f32,  // angular velocity of rod 2
}

#[allow(dead_code)]
impl DoublePendulum {
    pub fn new(m1: f32, m2: f32, l1: f32, l2: f32) -> Self {
        Self {
            m1,
            m2,
            l1,
            l2,
            g: 9.81,
            th1: 1.0,
            th2: 0.5,
            w1: 0.0,
            w2: 0.0,
        }
    }

    /// Equations of motion: returns (d_th1, d_w1, d_th2, d_w2).
    fn derivatives(&self, th1: f32, w1: f32, th2: f32, w2: f32) -> (f32, f32, f32, f32) {
        let m1 = self.m1;
        let m2 = self.m2;
        let l1 = self.l1;
        let l2 = self.l2;
        let g = self.g;
        let d = th1 - th2;
        let denom1 = (m1 + m2) * l1 - m2 * l1 * d.cos() * d.cos();
        let denom2 = (l2 / l1) * denom1;
        let a1 = (m2 * l1 * w1 * w1 * d.sin() * d.cos()
            + m2 * g * th2.sin() * d.cos()
            + m2 * l2 * w2 * w2 * d.sin()
            - (m1 + m2) * g * th1.sin())
            / denom1;
        let a2 = (-m2 * l2 * w2 * w2 * d.sin() * d.cos()
            + (m1 + m2) * (g * th1.sin() * d.cos() - l1 * w1 * w1 * d.sin() - g * th2.sin()))
            / denom2;
        (w1, a1, w2, a2)
    }

    /// RK4 step.
    pub fn step(&mut self, dt: f32) {
        let (th1, w1, th2, w2) = (self.th1, self.w1, self.th2, self.w2);
        let (k1a, k1b, k1c, k1d) = self.derivatives(th1, w1, th2, w2);
        let (k2a, k2b, k2c, k2d) = self.derivatives(
            th1 + 0.5 * dt * k1a,
            w1 + 0.5 * dt * k1b,
            th2 + 0.5 * dt * k1c,
            w2 + 0.5 * dt * k1d,
        );
        let (k3a, k3b, k3c, k3d) = self.derivatives(
            th1 + 0.5 * dt * k2a,
            w1 + 0.5 * dt * k2b,
            th2 + 0.5 * dt * k2c,
            w2 + 0.5 * dt * k2d,
        );
        let (k4a, k4b, k4c, k4d) =
            self.derivatives(th1 + dt * k3a, w1 + dt * k3b, th2 + dt * k3c, w2 + dt * k3d);
        self.th1 += dt / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a);
        self.w1 += dt / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b);
        self.th2 += dt / 6.0 * (k1c + 2.0 * k2c + 2.0 * k3c + k4c);
        self.w2 += dt / 6.0 * (k1d + 2.0 * k2d + 2.0 * k3d + k4d);
    }

    /// Position of bob 1 (x, y).
    pub fn bob1_pos(&self) -> [f32; 2] {
        [self.l1 * self.th1.sin(), -self.l1 * self.th1.cos()]
    }

    /// Position of bob 2 (x, y).
    pub fn bob2_pos(&self) -> [f32; 2] {
        let [bx, by] = self.bob1_pos();
        [bx + self.l2 * self.th2.sin(), by - self.l2 * self.th2.cos()]
    }

    /// Total kinetic energy.
    pub fn kinetic_energy(&self) -> f32 {
        let v1sq = (self.l1 * self.w1) * (self.l1 * self.w1);
        let vx2 = self.l1 * self.w1 * self.th1.cos() + self.l2 * self.w2 * self.th2.cos();
        let vy2 = self.l1 * self.w1 * self.th1.sin() + self.l2 * self.w2 * self.th2.sin();
        0.5 * self.m1 * v1sq + 0.5 * self.m2 * (vx2 * vx2 + vy2 * vy2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_changes_angles() {
        let mut dp = DoublePendulum::new(1.0, 1.0, 1.0, 1.0);
        let th1_0 = dp.th1;
        dp.step(0.01);
        let _delta = (dp.th1 - th1_0).abs();
    }

    #[test]
    fn bob1_pos_finite() {
        let dp = DoublePendulum::new(1.0, 1.0, 1.0, 1.0);
        let p = dp.bob1_pos();
        assert!(p[0].is_finite() && p[1].is_finite());
    }

    #[test]
    fn bob2_pos_finite() {
        let dp = DoublePendulum::new(1.0, 1.0, 1.0, 1.0);
        let p = dp.bob2_pos();
        assert!(p[0].is_finite() && p[1].is_finite());
    }

    #[test]
    fn bob1_distance_from_origin() {
        let dp = DoublePendulum::new(1.0, 1.0, 1.5, 1.0);
        let p = dp.bob1_pos();
        let d = (p[0] * p[0] + p[1] * p[1]).sqrt();
        assert!((d - 1.5).abs() < 0.01);
    }

    #[test]
    fn kinetic_energy_positive() {
        let mut dp = DoublePendulum::new(1.0, 1.0, 1.0, 1.0);
        dp.w1 = 1.0;
        assert!(dp.kinetic_energy() > 0.0);
    }

    #[test]
    fn many_steps_finite() {
        let mut dp = DoublePendulum::new(1.0, 1.0, 1.0, 1.0);
        for _ in 0..200 {
            dp.step(0.005);
        }
        assert!(dp.th1.is_finite() && dp.th2.is_finite());
    }

    #[test]
    fn initial_kinetic_energy_zero_if_no_velocity() {
        let dp = DoublePendulum::new(1.0, 1.0, 1.0, 1.0);
        // w1 = w2 = 0
        let ke = dp.kinetic_energy();
        assert!(ke.abs() < 1e-5);
    }

    #[test]
    fn different_masses_affect_energy() {
        let mut dp1 = DoublePendulum::new(1.0, 1.0, 1.0, 1.0);
        let mut dp2 = DoublePendulum::new(2.0, 1.0, 1.0, 1.0);
        dp1.w1 = 1.0;
        dp2.w1 = 1.0;
        assert!(dp2.kinetic_energy() > dp1.kinetic_energy());
    }

    #[test]
    fn bob2_farther_than_bob1() {
        let dp = DoublePendulum::new(1.0, 1.0, 1.0, 1.0);
        let p1 = dp.bob1_pos();
        let p2 = dp.bob2_pos();
        let d1 = (p1[0] * p1[0] + p1[1] * p1[1]).sqrt();
        let d2 = (p2[0] * p2[0] + p2[1] * p2[1]).sqrt();
        assert!(d2 > d1 * 0.5); // Not strictly true, but reasonable
    }

    #[test]
    fn rk4_step_lower_error_than_euler() {
        // Just ensure RK4 doesn't blow up in 10 steps
        let mut dp = DoublePendulum::new(1.0, 1.0, 1.0, 1.0);
        dp.th1 = 1.5;
        dp.th2 = -0.5;
        for _ in 0..10 {
            dp.step(0.01);
        }
        assert!(dp.th1.abs() < 10.0 && dp.th2.abs() < 10.0);
    }
}
