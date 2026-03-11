// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Rigid body definition and state: mass, inertia, position, velocity, forces.


/// Type of rigid body.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType {
    Dynamic,
    Kinematic,
    Static,
}

/// A rigid body definition with physical properties.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigidBodyDef {
    pub body_type: RigidBodyType,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub mass: f32,
    pub inertia: [f32; 3], // diagonal of inertia tensor
    pub force_accumulator: [f32; 3],
    pub torque_accumulator: [f32; 3],
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub gravity_scale: f32,
    pub is_sleeping: bool,
}

#[allow(dead_code)]
impl RigidBodyDef {
    pub fn dynamic(mass: f32) -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            position: [0.0; 3],
            velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            mass,
            inertia: [mass; 3],
            force_accumulator: [0.0; 3],
            torque_accumulator: [0.0; 3],
            linear_damping: 0.01,
            angular_damping: 0.01,
            gravity_scale: 1.0,
            is_sleeping: false,
        }
    }

    pub fn kinematic() -> Self {
        Self {
            body_type: RigidBodyType::Kinematic,
            mass: 0.0,
            ..Self::dynamic(0.0)
        }
    }

    pub fn static_body() -> Self {
        Self {
            body_type: RigidBodyType::Static,
            mass: 0.0,
            ..Self::dynamic(0.0)
        }
    }

    pub fn with_position(mut self, pos: [f32; 3]) -> Self {
        self.position = pos;
        self
    }

    pub fn with_velocity(mut self, vel: [f32; 3]) -> Self {
        self.velocity = vel;
        self
    }

    pub fn inv_mass(&self) -> f32 {
        if self.mass > 1e-10 { 1.0 / self.mass } else { 0.0 }
    }

    pub fn inv_inertia(&self) -> [f32; 3] {
        [
            if self.inertia[0] > 1e-10 { 1.0 / self.inertia[0] } else { 0.0 },
            if self.inertia[1] > 1e-10 { 1.0 / self.inertia[1] } else { 0.0 },
            if self.inertia[2] > 1e-10 { 1.0 / self.inertia[2] } else { 0.0 },
        ]
    }

    pub fn apply_force(&mut self, force: [f32; 3]) {
        self.force_accumulator[0] += force[0];
        self.force_accumulator[1] += force[1];
        self.force_accumulator[2] += force[2];
    }

    pub fn apply_torque(&mut self, torque: [f32; 3]) {
        self.torque_accumulator[0] += torque[0];
        self.torque_accumulator[1] += torque[1];
        self.torque_accumulator[2] += torque[2];
    }

    pub fn clear_forces(&mut self) {
        self.force_accumulator = [0.0; 3];
        self.torque_accumulator = [0.0; 3];
    }

    pub fn kinetic_energy(&self) -> f32 {
        let v2 = self.velocity[0]*self.velocity[0]
            + self.velocity[1]*self.velocity[1]
            + self.velocity[2]*self.velocity[2];
        0.5 * self.mass * v2
    }

    pub fn speed(&self) -> f32 {
        (self.velocity[0]*self.velocity[0]
            + self.velocity[1]*self.velocity[1]
            + self.velocity[2]*self.velocity[2]).sqrt()
    }

    pub fn momentum(&self) -> [f32; 3] {
        [self.mass * self.velocity[0], self.mass * self.velocity[1], self.mass * self.velocity[2]]
    }

    pub fn is_dynamic(&self) -> bool {
        self.body_type == RigidBodyType::Dynamic
    }

    /// Integrate velocity using Euler method.
    #[allow(clippy::needless_range_loop)]
    pub fn integrate(&mut self, dt: f32, gravity: [f32; 3]) {
        if self.body_type != RigidBodyType::Dynamic { return; }
        let inv_m = self.inv_mass();
        for i in 0..3 {
            self.velocity[i] += (self.force_accumulator[i] * inv_m + gravity[i] * self.gravity_scale) * dt;
            self.velocity[i] *= 1.0 - self.linear_damping * dt;
            self.position[i] += self.velocity[i] * dt;
        }
        let inv_i = self.inv_inertia();
        for i in 0..3 {
            self.angular_velocity[i] += self.torque_accumulator[i] * inv_i[i] * dt;
            self.angular_velocity[i] *= 1.0 - self.angular_damping * dt;
        }
        self.clear_forces();
    }

    /// Set inertia for a solid sphere of given radius.
    pub fn set_sphere_inertia(&mut self, radius: f32) {
        let i = 0.4 * self.mass * radius * radius;
        self.inertia = [i, i, i];
    }

    /// Set inertia for a solid box of given half-extents.
    pub fn set_box_inertia(&mut self, hx: f32, hy: f32, hz: f32) {
        let m = self.mass / 12.0;
        self.inertia = [
            m * (hy*hy + hz*hz) * 4.0,
            m * (hx*hx + hz*hz) * 4.0,
            m * (hx*hx + hy*hy) * 4.0,
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic() {
        let b = RigidBodyDef::dynamic(5.0);
        assert!(b.is_dynamic());
        assert!((b.mass - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inv_mass() {
        let b = RigidBodyDef::dynamic(4.0);
        assert!((b.inv_mass() - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_static_inv_mass() {
        let b = RigidBodyDef::static_body();
        assert!((b.inv_mass()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_force() {
        let mut b = RigidBodyDef::dynamic(1.0);
        b.apply_force([10.0, 0.0, 0.0]);
        assert!((b.force_accumulator[0] - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_integrate() {
        let mut b = RigidBodyDef::dynamic(1.0).with_position([0.0;3]);
        b.apply_force([10.0, 0.0, 0.0]);
        b.integrate(1.0, [0.0;3]);
        assert!(b.velocity[0] > 9.0);
        assert!(b.position[0] > 9.0);
    }

    #[test]
    fn test_kinetic_energy() {
        let b = RigidBodyDef::dynamic(2.0).with_velocity([3.0, 4.0, 0.0]);
        assert!((b.kinetic_energy() - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_speed() {
        let b = RigidBodyDef::dynamic(1.0).with_velocity([3.0, 4.0, 0.0]);
        assert!((b.speed() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_momentum() {
        let b = RigidBodyDef::dynamic(3.0).with_velocity([2.0, 0.0, 0.0]);
        let m = b.momentum();
        assert!((m[0] - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_sphere_inertia() {
        let mut b = RigidBodyDef::dynamic(10.0);
        b.set_sphere_inertia(1.0);
        assert!((b.inertia[0] - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_clear_forces() {
        let mut b = RigidBodyDef::dynamic(1.0);
        b.apply_force([5.0, 5.0, 5.0]);
        b.clear_forces();
        assert!((b.force_accumulator[0]).abs() < f32::EPSILON);
    }
}
