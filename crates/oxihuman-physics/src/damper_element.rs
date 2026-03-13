// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A viscous damper element that opposes velocity.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DamperElement {
    coefficient: f32,
    max_force: f32,
    velocity: f32,
}

#[allow(dead_code)]
impl DamperElement {
    pub fn new(coefficient: f32) -> Self {
        Self {
            coefficient: coefficient.max(0.0),
            max_force: f32::MAX,
            velocity: 0.0,
        }
    }

    pub fn with_max_force(mut self, max_force: f32) -> Self {
        self.max_force = max_force.max(0.0);
        self
    }

    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity;
    }

    pub fn force(&self) -> f32 {
        let raw = -self.coefficient * self.velocity;
        raw.clamp(-self.max_force, self.max_force)
    }

    pub fn force_for_velocity(&self, velocity: f32) -> f32 {
        let raw = -self.coefficient * velocity;
        raw.clamp(-self.max_force, self.max_force)
    }

    pub fn power_dissipated(&self) -> f32 {
        self.coefficient * self.velocity * self.velocity
    }

    pub fn coefficient(&self) -> f32 {
        self.coefficient
    }

    pub fn set_coefficient(&mut self, c: f32) {
        self.coefficient = c.max(0.0);
    }

    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    pub fn max_force(&self) -> f32 {
        self.max_force
    }

    pub fn is_active(&self) -> bool {
        self.velocity.abs() > f32::EPSILON
    }

    pub fn critical_damping_coefficient(mass: f32, stiffness: f32) -> f32 {
        2.0 * (mass * stiffness).sqrt()
    }

    pub fn energy_dissipated(&self, dt: f32) -> f32 {
        self.power_dissipated() * dt
    }
}

/// 3D damper element.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DamperElement3d {
    coefficient: f32,
    velocity: [f32; 3],
}

#[allow(dead_code)]
impl DamperElement3d {
    pub fn new(coefficient: f32) -> Self {
        Self {
            coefficient: coefficient.max(0.0),
            velocity: [0.0; 3],
        }
    }

    pub fn set_velocity(&mut self, v: [f32; 3]) {
        self.velocity = v;
    }

    pub fn force(&self) -> [f32; 3] {
        [
            -self.coefficient * self.velocity[0],
            -self.coefficient * self.velocity[1],
            -self.coefficient * self.velocity[2],
        ]
    }

    pub fn power_dissipated(&self) -> f32 {
        let v2 = self.velocity[0] * self.velocity[0]
            + self.velocity[1] * self.velocity[1]
            + self.velocity[2] * self.velocity[2];
        self.coefficient * v2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let d = DamperElement::new(10.0);
        assert!((d.coefficient() - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_force_zero_velocity() {
        let d = DamperElement::new(10.0);
        assert!((d.force() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_force_positive_velocity() {
        let mut d = DamperElement::new(10.0);
        d.set_velocity(2.0);
        assert!((d.force() + 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_max_force_clamp() {
        let mut d = DamperElement::new(100.0).with_max_force(50.0);
        d.set_velocity(10.0);
        assert!((d.force() + 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_power_dissipated() {
        let mut d = DamperElement::new(10.0);
        d.set_velocity(3.0);
        assert!((d.power_dissipated() - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_critical_damping() {
        let c = DamperElement::critical_damping_coefficient(1.0, 100.0);
        assert!((c - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_is_active() {
        let mut d = DamperElement::new(10.0);
        assert!(!d.is_active());
        d.set_velocity(1.0);
        assert!(d.is_active());
    }

    #[test]
    fn test_energy_dissipated() {
        let mut d = DamperElement::new(10.0);
        d.set_velocity(2.0);
        assert!((d.energy_dissipated(0.5) - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_3d_force() {
        let mut d = DamperElement3d::new(5.0);
        d.set_velocity([1.0, 2.0, 3.0]);
        let f = d.force();
        assert!((f[0] + 5.0).abs() < f32::EPSILON);
        assert!((f[1] + 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_3d_power() {
        let mut d = DamperElement3d::new(2.0);
        d.set_velocity([1.0, 0.0, 0.0]);
        assert!((d.power_dissipated() - 2.0).abs() < f32::EPSILON);
    }
}
