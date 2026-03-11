// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A friction joint that limits relative motion between two bodies.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FrictionJoint {
    anchor_a: [f32; 3],
    anchor_b: [f32; 3],
    max_force: f32,
    max_torque: f32,
    static_friction: f32,
    dynamic_friction: f32,
    accumulated_force: f32,
}

#[allow(dead_code)]
impl FrictionJoint {
    pub fn new(anchor_a: [f32; 3], anchor_b: [f32; 3]) -> Self {
        Self {
            anchor_a,
            anchor_b,
            max_force: 100.0,
            max_torque: 50.0,
            static_friction: 0.6,
            dynamic_friction: 0.4,
            accumulated_force: 0.0,
        }
    }

    pub fn with_max_force(mut self, f: f32) -> Self {
        self.max_force = f.max(0.0);
        self
    }

    pub fn with_max_torque(mut self, t: f32) -> Self {
        self.max_torque = t.max(0.0);
        self
    }

    pub fn with_coefficients(mut self, static_f: f32, dynamic_f: f32) -> Self {
        self.static_friction = static_f.clamp(0.0, 1.0);
        self.dynamic_friction = dynamic_f.clamp(0.0, 1.0);
        self
    }

    pub fn compute_friction_force(&self, relative_velocity: f32, normal_force: f32) -> f32 {
        let coeff = if relative_velocity.abs() < 0.01 {
            self.static_friction
        } else {
            self.dynamic_friction
        };
        let raw = coeff * normal_force.abs();
        raw.min(self.max_force)
    }

    pub fn compute_friction_torque(&self, angular_velocity: f32, normal_force: f32) -> f32 {
        let raw = self.dynamic_friction * normal_force.abs() * angular_velocity.abs();
        raw.min(self.max_torque)
    }

    pub fn anchor_a(&self) -> [f32; 3] {
        self.anchor_a
    }

    pub fn anchor_b(&self) -> [f32; 3] {
        self.anchor_b
    }

    pub fn max_force(&self) -> f32 {
        self.max_force
    }

    pub fn max_torque(&self) -> f32 {
        self.max_torque
    }

    pub fn static_friction(&self) -> f32 {
        self.static_friction
    }

    pub fn dynamic_friction(&self) -> f32 {
        self.dynamic_friction
    }

    pub fn anchor_distance(&self) -> f32 {
        let dx = self.anchor_b[0] - self.anchor_a[0];
        let dy = self.anchor_b[1] - self.anchor_a[1];
        let dz = self.anchor_b[2] - self.anchor_a[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn accumulate_force(&mut self, f: f32) {
        self.accumulated_force += f;
    }

    pub fn reset_accumulated(&mut self) {
        self.accumulated_force = 0.0;
    }

    pub fn accumulated_force(&self) -> f32 {
        self.accumulated_force
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let fj = FrictionJoint::new([0.0; 3], [1.0, 0.0, 0.0]);
        assert!((fj.static_friction() - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_static_friction_low_velocity() {
        let fj = FrictionJoint::new([0.0; 3], [0.0; 3]).with_coefficients(0.5, 0.3);
        let f = fj.compute_friction_force(0.001, 100.0);
        assert!((f - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_dynamic_friction_high_velocity() {
        let fj = FrictionJoint::new([0.0; 3], [0.0; 3]).with_coefficients(0.5, 0.3);
        let f = fj.compute_friction_force(1.0, 100.0);
        assert!((f - 30.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_force_clamp() {
        let fj = FrictionJoint::new([0.0; 3], [0.0; 3])
            .with_max_force(10.0)
            .with_coefficients(0.9, 0.9);
        let f = fj.compute_friction_force(1.0, 1000.0);
        assert!((f - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_anchor_distance() {
        let fj = FrictionJoint::new([0.0; 3], [3.0, 4.0, 0.0]);
        assert!((fj.anchor_distance() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_friction_torque() {
        let fj = FrictionJoint::new([0.0; 3], [0.0; 3]).with_coefficients(0.5, 0.4);
        let t = fj.compute_friction_torque(2.0, 10.0);
        assert!((t - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_with_max_torque() {
        let fj = FrictionJoint::new([0.0; 3], [0.0; 3])
            .with_max_torque(5.0)
            .with_coefficients(0.5, 0.4);
        let t = fj.compute_friction_torque(100.0, 100.0);
        assert!((t - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_accumulate_force() {
        let mut fj = FrictionJoint::new([0.0; 3], [0.0; 3]);
        fj.accumulate_force(5.0);
        fj.accumulate_force(3.0);
        assert!((fj.accumulated_force() - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_reset_accumulated() {
        let mut fj = FrictionJoint::new([0.0; 3], [0.0; 3]);
        fj.accumulate_force(10.0);
        fj.reset_accumulated();
        assert!((fj.accumulated_force() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_coefficients_clamp() {
        let fj = FrictionJoint::new([0.0; 3], [0.0; 3]).with_coefficients(2.0, -1.0);
        assert!((fj.static_friction() - 1.0).abs() < f32::EPSILON);
        assert!((fj.dynamic_friction() - 0.0).abs() < f32::EPSILON);
    }
}
