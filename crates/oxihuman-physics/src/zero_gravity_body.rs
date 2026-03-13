// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A body simulation in zero-gravity (space) — pure Newtonian inertia.

/// A rigid body in zero gravity.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ZeroGravityBody {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub orientation: [f32; 4],
    pub angular_vel: [f32; 3],
    pub mass: f32,
    pub inv_mass: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub time: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl ZeroGravityBody {
    pub fn new(mass: f32) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        Self {
            pos: [0.0; 3],
            vel: [0.0; 3],
            // Unit quaternion [x, y, z, w].
            orientation: [0.0, 0.0, 0.0, 1.0],
            angular_vel: [0.0; 3],
            mass: mass.max(1e-6),
            inv_mass,
            linear_damping: 0.0,
            angular_damping: 0.0,
            time: 0.0,
            steps: 0,
        }
    }

    pub fn with_pos(mut self, pos: [f32; 3]) -> Self {
        self.pos = pos;
        self
    }

    pub fn with_vel(mut self, vel: [f32; 3]) -> Self {
        self.vel = vel;
        self
    }

    pub fn with_angular_vel(mut self, av: [f32; 3]) -> Self {
        self.angular_vel = av;
        self
    }

    pub fn set_damping(&mut self, linear: f32, angular: f32) {
        self.linear_damping = linear.clamp(0.0, 1.0);
        self.angular_damping = angular.clamp(0.0, 1.0);
    }

    pub fn apply_force(&mut self, force: [f32; 3], dt: f32) {
        let acc = [
            force[0] * self.inv_mass,
            force[1] * self.inv_mass,
            force[2] * self.inv_mass,
        ];
        self.vel[0] += acc[0] * dt;
        self.vel[1] += acc[1] * dt;
        self.vel[2] += acc[2] * dt;
    }

    pub fn apply_torque(&mut self, torque: [f32; 3], dt: f32) {
        // Simplified: no inertia tensor, just scale by inv_mass.
        self.angular_vel[0] += torque[0] * self.inv_mass * dt;
        self.angular_vel[1] += torque[1] * self.inv_mass * dt;
        self.angular_vel[2] += torque[2] * self.inv_mass * dt;
    }

    pub fn apply_impulse(&mut self, impulse: [f32; 3]) {
        self.vel[0] += impulse[0] * self.inv_mass;
        self.vel[1] += impulse[1] * self.inv_mass;
        self.vel[2] += impulse[2] * self.inv_mass;
    }

    pub fn step(&mut self, dt: f32) {
        // Linear motion.
        let d = 1.0 - self.linear_damping * dt;
        self.vel[0] *= d;
        self.vel[1] *= d;
        self.vel[2] *= d;
        self.pos[0] += self.vel[0] * dt;
        self.pos[1] += self.vel[1] * dt;
        self.pos[2] += self.vel[2] * dt;
        // Angular motion (simplified quaternion integration).
        let da = 1.0 - self.angular_damping * dt;
        self.angular_vel[0] *= da;
        self.angular_vel[1] *= da;
        self.angular_vel[2] *= da;
        let half_dt = 0.5 * dt;
        let [ax, ay, az] = self.angular_vel;
        let [qx, qy, qz, qw] = self.orientation;
        self.orientation[0] = qx + (qw * ax - qz * ay + qy * az) * half_dt;
        self.orientation[1] = qy + (qw * ay + qz * ax - qx * az) * half_dt;
        self.orientation[2] = qz + (qw * az - qy * ax + qx * ay) * half_dt;
        self.orientation[3] = qw - (qx * ax + qy * ay + qz * az) * half_dt;
        // Normalize quaternion.
        let len = (self.orientation[0] * self.orientation[0]
            + self.orientation[1] * self.orientation[1]
            + self.orientation[2] * self.orientation[2]
            + self.orientation[3] * self.orientation[3])
            .sqrt()
            .max(1e-9);
        for q in &mut self.orientation {
            *q /= len;
        }
        self.time += dt;
        self.steps += 1;
    }

    pub fn speed(&self) -> f32 {
        (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1] + self.vel[2] * self.vel[2]).sqrt()
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.speed() * self.speed()
    }

    pub fn reset(&mut self) {
        self.pos = [0.0; 3];
        self.vel = [0.0; 3];
        self.orientation = [0.0, 0.0, 0.0, 1.0];
        self.angular_vel = [0.0; 3];
        self.time = 0.0;
        self.steps = 0;
    }
}

impl Default for ZeroGravityBody {
    fn default() -> Self {
        Self::new(1.0)
    }
}

pub fn new_zero_gravity_body(mass: f32) -> ZeroGravityBody {
    ZeroGravityBody::new(mass)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_velocity_in_zero_grav() {
        let mut b = new_zero_gravity_body(1.0).with_vel([3.0, 0.0, 0.0]);
        b.step(1.0);
        assert!((b.pos[0] - 3.0).abs() < 1e-4);
    }

    #[test]
    fn force_accelerates() {
        let mut b = new_zero_gravity_body(1.0);
        b.apply_force([10.0, 0.0, 0.0], 1.0);
        b.step(1.0);
        assert!(b.vel[0] > 0.0);
    }

    #[test]
    fn impulse_changes_vel() {
        let mut b = new_zero_gravity_body(2.0);
        b.apply_impulse([4.0, 0.0, 0.0]);
        assert!((b.vel[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn damping_reduces_speed() {
        let mut b = new_zero_gravity_body(1.0).with_vel([10.0, 0.0, 0.0]);
        b.set_damping(0.5, 0.0);
        b.step(0.1);
        assert!(b.speed() < 10.0);
    }

    #[test]
    fn quaternion_stays_normalized() {
        let mut b = new_zero_gravity_body(1.0).with_angular_vel([1.0, 2.0, 3.0]);
        for _ in 0..100 {
            b.step(0.016);
        }
        let len = (b.orientation[0] * b.orientation[0]
            + b.orientation[1] * b.orientation[1]
            + b.orientation[2] * b.orientation[2]
            + b.orientation[3] * b.orientation[3])
            .sqrt();
        assert!((len - 1.0).abs() < 1e-4);
    }

    #[test]
    fn step_count() {
        let mut b = new_zero_gravity_body(1.0);
        b.step(0.016);
        b.step(0.016);
        assert_eq!(b.steps, 2);
    }

    #[test]
    fn time_advances() {
        let mut b = new_zero_gravity_body(1.0);
        b.step(0.5);
        assert!((b.time - 0.5).abs() < 1e-5);
    }

    #[test]
    fn kinetic_energy_positive_when_moving() {
        let b = new_zero_gravity_body(2.0).with_vel([3.0, 0.0, 0.0]);
        assert!(b.kinetic_energy() > 0.0);
    }

    #[test]
    fn reset_zeroes_all() {
        let mut b = new_zero_gravity_body(1.0).with_vel([5.0, 0.0, 0.0]);
        b.step(1.0);
        b.reset();
        assert!(b.speed() < 1e-5);
        assert_eq!(b.steps, 0);
    }

    #[test]
    fn torque_changes_angular_vel() {
        let mut b = new_zero_gravity_body(1.0);
        b.apply_torque([0.0, 10.0, 0.0], 1.0);
        assert!(b.angular_vel[1].abs() > 0.0);
    }
}
