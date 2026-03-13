// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2D rigid body dynamics stub.

use std::f32::consts::TAU;

#[derive(Debug, Clone)]
pub struct RigidBody2d {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub angle: f32,
    pub angular_velocity: f32,
    pub mass: f32,
    pub inertia: f32,
    pub restitution: f32,
}

impl RigidBody2d {
    pub fn new(mass: f32, inertia: f32) -> Self {
        RigidBody2d {
            position: [0.0; 2],
            velocity: [0.0; 2],
            angle: 0.0,
            angular_velocity: 0.0,
            mass,
            inertia,
            restitution: 0.5,
        }
    }

    pub fn apply_force(&mut self, force: [f32; 2], dt: f32) {
        if self.mass > f32::EPSILON {
            self.velocity[0] += force[0] / self.mass * dt;
            self.velocity[1] += force[1] / self.mass * dt;
        }
    }

    pub fn apply_torque(&mut self, torque: f32, dt: f32) {
        if self.inertia > f32::EPSILON {
            self.angular_velocity += torque / self.inertia * dt;
        }
    }

    pub fn integrate(&mut self, dt: f32) {
        self.position[0] += self.velocity[0] * dt;
        self.position[1] += self.velocity[1] * dt;
        self.angle = (self.angle + self.angular_velocity * dt).rem_euclid(TAU);
    }

    pub fn kinetic_energy(&self) -> f32 {
        let v2 = self.velocity[0].powi(2) + self.velocity[1].powi(2);
        0.5 * self.mass * v2 + 0.5 * self.inertia * self.angular_velocity.powi(2)
    }

    pub fn speed(&self) -> f32 {
        (self.velocity[0].powi(2) + self.velocity[1].powi(2)).sqrt()
    }
}

pub fn apply_gravity_2d(body: &mut RigidBody2d, g: f32, dt: f32) {
    body.velocity[1] -= g * dt;
}

pub fn body_2d_momentum(body: &RigidBody2d) -> [f32; 2] {
    [body.velocity[0] * body.mass, body.velocity[1] * body.mass]
}

pub fn angular_momentum_2d(body: &RigidBody2d) -> f32 {
    body.inertia * body.angular_velocity
}

pub fn apply_impulse_2d(body: &mut RigidBody2d, impulse: [f32; 2]) {
    if body.mass > f32::EPSILON {
        body.velocity[0] += impulse[0] / body.mass;
        body.velocity[1] += impulse[1] / body.mass;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrate_position() {
        let mut b = RigidBody2d::new(1.0, 1.0);
        b.velocity = [2.0, 0.0];
        b.integrate(1.0);
        assert!((b.position[0] - 2.0).abs() < 1e-6, /* x moves by velocity */);
    }

    #[test]
    fn test_apply_force() {
        let mut b = RigidBody2d::new(2.0, 1.0);
        b.apply_force([4.0, 0.0], 1.0);
        assert!((b.velocity[0] - 2.0).abs() < 1e-6, /* F/m*dt = 4/2*1 = 2 */);
    }

    #[test]
    fn test_apply_torque() {
        let mut b = RigidBody2d::new(1.0, 2.0);
        b.apply_torque(4.0, 1.0);
        assert!((b.angular_velocity - 2.0).abs() < 1e-6, /* T/I*dt = 4/2*1 = 2 */);
    }

    #[test]
    fn test_kinetic_energy_rest() {
        let b = RigidBody2d::new(1.0, 1.0);
        assert!((b.kinetic_energy() - 0.0).abs() < 1e-6 /* at rest */,);
    }

    #[test]
    fn test_gravity() {
        let mut b = RigidBody2d::new(1.0, 1.0);
        apply_gravity_2d(&mut b, 9.81, 1.0);
        assert!(b.velocity[1] < 0.0 /* gravity pulls down */,);
    }

    #[test]
    fn test_momentum() {
        let mut b = RigidBody2d::new(3.0, 1.0);
        b.velocity = [2.0, 0.0];
        let mom = body_2d_momentum(&b);
        assert!((mom[0] - 6.0).abs() < 1e-6 /* 3*2 = 6 */,);
    }

    #[test]
    fn test_apply_impulse() {
        let mut b = RigidBody2d::new(2.0, 1.0);
        apply_impulse_2d(&mut b, [4.0, 0.0]);
        assert!((b.velocity[0] - 2.0).abs() < 1e-6 /* 4/2 = 2 */,);
    }

    #[test]
    fn test_angle_wraps() {
        let mut b = RigidBody2d::new(1.0, 1.0);
        b.angular_velocity = TAU + 0.1;
        b.integrate(1.0);
        assert!((0.0..TAU).contains(&b.angle), /* angle stays in [0, TAU) */);
    }

    #[test]
    fn test_speed() {
        let mut b = RigidBody2d::new(1.0, 1.0);
        b.velocity = [3.0, 4.0];
        assert!((b.speed() - 5.0).abs() < 1e-5 /* 3-4-5 triangle */,);
    }
}
