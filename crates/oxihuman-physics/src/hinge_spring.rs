// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hinge joint with torsional spring and angular damper.

use std::f32::consts::PI;

/// A hinge joint constrained to rotate around a fixed axis,
/// with a torsional spring driving it toward a rest angle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HingeSpring {
    pub axis: [f32; 3],        // unit hinge axis
    pub angle: f32,            // current angle (radians)
    pub angular_velocity: f32, // rad/s
    pub rest_angle: f32,       // equilibrium angle (radians)
    pub stiffness: f32,
    pub damping: f32,
    pub min_angle: f32,
    pub max_angle: f32,
}

#[allow(dead_code)]
impl HingeSpring {
    pub fn new(axis: [f32; 3], stiffness: f32, damping: f32) -> Self {
        Self {
            axis: normalize3(axis),
            angle: 0.0,
            angular_velocity: 0.0,
            rest_angle: 0.0,
            stiffness,
            damping,
            min_angle: -PI,
            max_angle: PI,
        }
    }

    pub fn with_limits(mut self, min_deg: f32, max_deg: f32) -> Self {
        self.min_angle = min_deg.to_radians();
        self.max_angle = max_deg.to_radians();
        self
    }

    pub fn with_rest(mut self, rest_deg: f32) -> Self {
        self.rest_angle = rest_deg.to_radians();
        self
    }

    /// Spring torque toward rest angle.
    pub fn spring_torque(&self) -> f32 {
        -self.stiffness * (self.angle - self.rest_angle)
    }

    /// Damping torque opposing angular velocity.
    pub fn damping_torque(&self) -> f32 {
        -self.damping * self.angular_velocity
    }

    /// Total restoring torque.
    pub fn total_torque(&self) -> f32 {
        self.spring_torque() + self.damping_torque()
    }

    pub fn integrate(&mut self, inertia: f32, dt: f32) {
        let torque = self.total_torque();
        let inv_i = if inertia > 1e-9 { 1.0 / inertia } else { 0.0 };
        self.angular_velocity += torque * inv_i * dt;
        self.angle += self.angular_velocity * dt;
        // Clamp to limits
        self.angle = self.angle.clamp(self.min_angle, self.max_angle);
    }

    pub fn is_at_limit(&self) -> bool {
        (self.angle - self.min_angle).abs() < 1e-4 || (self.angle - self.max_angle).abs() < 1e-4
    }

    pub fn angle_deg(&self) -> f32 {
        self.angle.to_degrees()
    }

    pub fn kinetic_energy(&self, inertia: f32) -> f32 {
        0.5 * inertia * self.angular_velocity * self.angular_velocity
    }

    pub fn potential_energy(&self) -> f32 {
        let d = self.angle - self.rest_angle;
        0.5 * self.stiffness * d * d
    }
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-9 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

/// Natural frequency of a torsional spring.
#[allow(dead_code)]
pub fn torsional_natural_frequency(stiffness: f32, inertia: f32) -> f32 {
    if inertia > 1e-9 {
        (stiffness / inertia).sqrt()
    } else {
        0.0
    }
}

/// Critical damping for torsional spring.
#[allow(dead_code)]
pub fn torsional_critical_damping(stiffness: f32, inertia: f32) -> f32 {
    2.0 * (stiffness * inertia).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_rest_no_spring_torque() {
        let h = HingeSpring::new([0.0, 1.0, 0.0], 10.0, 0.5);
        assert!(h.spring_torque().abs() < 1e-6);
    }

    #[test]
    fn displaced_generates_restoring_torque() {
        let mut h = HingeSpring::new([0.0, 1.0, 0.0], 10.0, 0.0);
        h.angle = 0.5;
        assert!(h.spring_torque() < 0.0); // pulls back
    }

    #[test]
    fn damping_torque_opposes_velocity() {
        let mut h = HingeSpring::new([0.0, 1.0, 0.0], 10.0, 2.0);
        h.angular_velocity = 3.0;
        assert!(h.damping_torque() < 0.0);
    }

    #[test]
    fn integrate_moves_angle() {
        let mut h = HingeSpring::new([0.0, 1.0, 0.0], 0.0, 0.0);
        h.angular_velocity = 1.0;
        h.integrate(1.0, 0.1);
        assert!((h.angle - 0.1).abs() < 1e-4);
    }

    #[test]
    fn limits_clamp_angle() {
        let mut h = HingeSpring::new([0.0, 1.0, 0.0], 0.0, 0.0).with_limits(-10.0, 10.0);
        h.angular_velocity = 1000.0;
        h.integrate(1.0, 1.0);
        assert!(h.angle <= h.max_angle + 1e-4);
    }

    #[test]
    fn potential_energy_at_rest_is_zero() {
        let h = HingeSpring::new([0.0, 1.0, 0.0], 10.0, 0.0);
        assert!(h.potential_energy().abs() < 1e-6);
    }

    #[test]
    fn potential_energy_displaced() {
        let mut h = HingeSpring::new([0.0, 1.0, 0.0], 10.0, 0.0);
        h.angle = 1.0; // PE = 0.5 * 10 * 1 = 5
        assert!((h.potential_energy() - 5.0).abs() < 1e-4);
    }

    #[test]
    fn torsional_natural_frequency_formula() {
        let f = torsional_natural_frequency(4.0, 1.0);
        assert!((f - 2.0).abs() < 1e-5);
    }

    #[test]
    fn torsional_critical_damping_formula() {
        let c = torsional_critical_damping(4.0, 1.0);
        assert!((c - 4.0).abs() < 1e-5);
    }

    #[test]
    fn with_rest_sets_equilibrium() {
        let h = HingeSpring::new([0.0, 1.0, 0.0], 10.0, 0.0).with_rest(45.0);
        assert!((h.rest_angle - PI / 4.0).abs() < 1e-4);
    }
}
