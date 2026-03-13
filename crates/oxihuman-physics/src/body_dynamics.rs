// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Basic rigid body dynamics: position, velocity, force accumulation, integration.

#[allow(dead_code)]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyDynamics {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub force: [f32; 3],
    pub mass: f32,
    pub inv_mass: f32,
    pub linear_damping: f32,
}

#[allow(dead_code)]
impl BodyDynamics {
    pub fn new(mass: f32) -> Self {
        let inv = if mass > 1e-12 { 1.0 / mass } else { 0.0 };
        Self {
            position: [0.0; 3],
            velocity: [0.0; 3],
            force: [0.0; 3],
            mass,
            inv_mass: inv,
            linear_damping: 0.0,
        }
    }

    pub fn set_position(&mut self, pos: [f32; 3]) {
        self.position = pos;
    }

    pub fn set_velocity(&mut self, vel: [f32; 3]) {
        self.velocity = vel;
    }

    pub fn apply_force(&mut self, f: [f32; 3]) {
        self.force = vec3_add(self.force, f);
    }

    pub fn apply_impulse(&mut self, impulse: [f32; 3]) {
        self.velocity = vec3_add(self.velocity, vec3_scale(impulse, self.inv_mass));
    }

    pub fn clear_forces(&mut self) {
        self.force = [0.0; 3];
    }

    pub fn integrate(&mut self, dt: f32) {
        let accel = vec3_scale(self.force, self.inv_mass);
        self.velocity = vec3_add(self.velocity, vec3_scale(accel, dt));
        // Apply damping
        let factor = (1.0 - self.linear_damping * dt).max(0.0);
        self.velocity = vec3_scale(self.velocity, factor);
        self.position = vec3_add(self.position, vec3_scale(self.velocity, dt));
        self.clear_forces();
    }

    pub fn speed(&self) -> f32 {
        vec3_len(self.velocity)
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.speed() * self.speed()
    }

    pub fn momentum(&self) -> [f32; 3] {
        vec3_scale(self.velocity, self.mass)
    }

    pub fn is_static(&self) -> bool {
        self.inv_mass < 1e-12
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let b = BodyDynamics::new(2.0);
        assert!((b.mass - 2.0).abs() < 1e-5);
        assert!((b.inv_mass - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_apply_force_and_integrate() {
        let mut b = BodyDynamics::new(1.0);
        b.apply_force([10.0, 0.0, 0.0]);
        b.integrate(1.0);
        assert!((b.velocity[0] - 10.0).abs() < 1e-3);
    }

    #[test]
    fn test_apply_impulse() {
        let mut b = BodyDynamics::new(2.0);
        b.apply_impulse([4.0, 0.0, 0.0]);
        assert!((b.velocity[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_static_body() {
        let b = BodyDynamics::new(0.0);
        assert!(b.is_static());
    }

    #[test]
    fn test_kinetic_energy() {
        let mut b = BodyDynamics::new(2.0);
        b.set_velocity([3.0, 0.0, 0.0]);
        assert!((b.kinetic_energy() - 9.0).abs() < 1e-3);
    }

    #[test]
    fn test_momentum() {
        let mut b = BodyDynamics::new(3.0);
        b.set_velocity([2.0, 0.0, 0.0]);
        let p = b.momentum();
        assert!((p[0] - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_clear_forces() {
        let mut b = BodyDynamics::new(1.0);
        b.apply_force([5.0, 5.0, 5.0]);
        b.clear_forces();
        assert!((b.force[0]).abs() < 1e-10);
    }

    #[test]
    fn test_set_position() {
        let mut b = BodyDynamics::new(1.0);
        b.set_position([1.0, 2.0, 3.0]);
        assert!((b.position[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_speed() {
        let mut b = BodyDynamics::new(1.0);
        b.set_velocity([3.0, 4.0, 0.0]);
        assert!((b.speed() - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_damping() {
        let mut b = BodyDynamics::new(1.0);
        b.linear_damping = 0.1;
        b.set_velocity([10.0, 0.0, 0.0]);
        b.integrate(1.0);
        assert!(b.velocity[0] < 10.0);
    }
}
