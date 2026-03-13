// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A dynamic rigid body with position, velocity, mass and force accumulation.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DynamicBody {
    position: [f32; 3],
    velocity: [f32; 3],
    force_accum: [f32; 3],
    mass: f32,
    inv_mass: f32,
    linear_damping: f32,
    active: bool,
}

#[allow(dead_code)]
impl DynamicBody {
    pub fn new(mass: f32) -> Self {
        let inv = if mass > f32::EPSILON { 1.0 / mass } else { 0.0 };
        Self {
            position: [0.0; 3],
            velocity: [0.0; 3],
            force_accum: [0.0; 3],
            mass: mass.max(0.0),
            inv_mass: inv,
            linear_damping: 0.01,
            active: true,
        }
    }

    pub fn with_position(mut self, pos: [f32; 3]) -> Self {
        self.position = pos;
        self
    }

    pub fn apply_force(&mut self, force: [f32; 3]) {
        self.force_accum[0] += force[0];
        self.force_accum[1] += force[1];
        self.force_accum[2] += force[2];
    }

    pub fn apply_impulse(&mut self, impulse: [f32; 3]) {
        self.velocity[0] += impulse[0] * self.inv_mass;
        self.velocity[1] += impulse[1] * self.inv_mass;
        self.velocity[2] += impulse[2] * self.inv_mass;
    }

    pub fn integrate(&mut self, dt: f32) {
        if !self.active || self.inv_mass == 0.0 {
            return;
        }
        // acceleration
        let ax = self.force_accum[0] * self.inv_mass;
        let ay = self.force_accum[1] * self.inv_mass;
        let az = self.force_accum[2] * self.inv_mass;
        // velocity
        self.velocity[0] += ax * dt;
        self.velocity[1] += ay * dt;
        self.velocity[2] += az * dt;
        // damping
        let damp = (1.0 - self.linear_damping).max(0.0);
        self.velocity[0] *= damp;
        self.velocity[1] *= damp;
        self.velocity[2] *= damp;
        // position
        self.position[0] += self.velocity[0] * dt;
        self.position[1] += self.velocity[1] * dt;
        self.position[2] += self.velocity[2] * dt;
        // clear forces
        self.force_accum = [0.0; 3];
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn set_position(&mut self, pos: [f32; 3]) {
        self.position = pos;
    }

    pub fn velocity(&self) -> [f32; 3] {
        self.velocity
    }

    pub fn set_velocity(&mut self, vel: [f32; 3]) {
        self.velocity = vel;
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }

    pub fn inv_mass(&self) -> f32 {
        self.inv_mass
    }

    pub fn speed(&self) -> f32 {
        let v = self.velocity;
        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.speed() * self.speed()
    }

    pub fn momentum(&self) -> [f32; 3] {
        [
            self.mass * self.velocity[0],
            self.mass * self.velocity[1],
            self.mass * self.velocity[2],
        ]
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn set_damping(&mut self, d: f32) {
        self.linear_damping = d.clamp(0.0, 1.0);
    }

    pub fn is_static(&self) -> bool {
        self.inv_mass == 0.0
    }

    pub fn clear_forces(&mut self) {
        self.force_accum = [0.0; 3];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let b = DynamicBody::new(2.0);
        assert!((b.mass() - 2.0).abs() < f32::EPSILON);
        assert!((b.inv_mass() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_static_body() {
        let b = DynamicBody::new(0.0);
        assert!(b.is_static());
    }

    #[test]
    fn test_apply_force_and_integrate() {
        let mut b = DynamicBody::new(1.0);
        b.set_damping(0.0);
        b.apply_force([10.0, 0.0, 0.0]);
        b.integrate(1.0);
        assert!(b.velocity()[0] > 0.0);
        assert!(b.position()[0] > 0.0);
    }

    #[test]
    fn test_apply_impulse() {
        let mut b = DynamicBody::new(2.0);
        b.apply_impulse([4.0, 0.0, 0.0]);
        assert!((b.velocity()[0] - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_speed() {
        let mut b = DynamicBody::new(1.0);
        b.set_velocity([3.0, 4.0, 0.0]);
        assert!((b.speed() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_kinetic_energy() {
        let mut b = DynamicBody::new(2.0);
        b.set_velocity([1.0, 0.0, 0.0]);
        assert!((b.kinetic_energy() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_momentum() {
        let mut b = DynamicBody::new(3.0);
        b.set_velocity([2.0, 0.0, 0.0]);
        assert!((b.momentum()[0] - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_active() {
        let mut b = DynamicBody::new(1.0);
        b.set_active(false);
        b.apply_force([100.0, 0.0, 0.0]);
        b.integrate(1.0);
        assert!((b.speed() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_with_position() {
        let b = DynamicBody::new(1.0).with_position([1.0, 2.0, 3.0]);
        assert_eq!(b.position(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_clear_forces() {
        let mut b = DynamicBody::new(1.0);
        b.apply_force([5.0, 5.0, 5.0]);
        b.clear_forces();
        b.set_damping(0.0);
        b.integrate(1.0);
        assert!((b.speed() - 0.0).abs() < f32::EPSILON);
    }
}
