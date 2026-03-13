// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Rigid body with angular velocity integration.

use std::f32::consts::PI;

/// A body with angular velocity and orientation tracking.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AngularVelocityBody {
    pub angular_velocity: [f32; 3],
    pub orientation: [f32; 4], // quaternion [x, y, z, w]
    pub inertia: f32,
    pub torque_accum: [f32; 3],
    pub damping: f32,
}

#[allow(dead_code)]
impl AngularVelocityBody {
    pub fn new(inertia: f32, damping: f32) -> Self {
        Self {
            angular_velocity: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
            inertia,
            torque_accum: [0.0; 3],
            damping: damping.clamp(0.0, 1.0),
        }
    }

    pub fn apply_torque(&mut self, torque: [f32; 3]) {
        self.torque_accum[0] += torque[0];
        self.torque_accum[1] += torque[1];
        self.torque_accum[2] += torque[2];
    }

    pub fn integrate(&mut self, dt: f32) {
        let inv_i = if self.inertia > 1e-9 {
            1.0 / self.inertia
        } else {
            0.0
        };
        self.angular_velocity[0] += self.torque_accum[0] * inv_i * dt;
        self.angular_velocity[1] += self.torque_accum[1] * inv_i * dt;
        self.angular_velocity[2] += self.torque_accum[2] * inv_i * dt;

        // Apply damping
        let damp = (1.0 - self.damping * dt).max(0.0);
        self.angular_velocity[0] *= damp;
        self.angular_velocity[1] *= damp;
        self.angular_velocity[2] *= damp;

        // Integrate orientation via quaternion derivative
        integrate_orientation(&mut self.orientation, self.angular_velocity, dt);

        self.torque_accum = [0.0; 3];
    }

    pub fn angular_speed(&self) -> f32 {
        let [wx, wy, wz] = self.angular_velocity;
        (wx * wx + wy * wy + wz * wz).sqrt()
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.inertia * self.angular_speed().powi(2)
    }

    pub fn reset_velocity(&mut self) {
        self.angular_velocity = [0.0; 3];
    }

    pub fn set_angular_velocity(&mut self, v: [f32; 3]) {
        self.angular_velocity = v;
    }
}

/// Integrate quaternion orientation by angular velocity * dt.
#[allow(dead_code)]
pub fn integrate_orientation(q: &mut [f32; 4], omega: [f32; 3], dt: f32) {
    let [ox, oy, oz] = omega;
    let [qx, qy, qz, qw] = *q;
    let dqx = 0.5 * (qw * ox - qz * oy + qy * oz);
    let dqy = 0.5 * (qz * ox + qw * oy - qx * oz);
    let dqz = 0.5 * (-qy * ox + qx * oy + qw * oz);
    let dqw = 0.5 * (-qx * ox - qy * oy - qz * oz);

    q[0] = qx + dqx * dt;
    q[1] = qy + dqy * dt;
    q[2] = qz + dqz * dt;
    q[3] = qw + dqw * dt;
    normalize_quaternion(q);
}

/// Normalize a quaternion in-place.
#[allow(dead_code)]
pub fn normalize_quaternion(q: &mut [f32; 4]) {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len > 1e-9 {
        q[0] /= len;
        q[1] /= len;
        q[2] /= len;
        q[3] /= len;
    } else {
        *q = [0.0, 0.0, 0.0, 1.0];
    }
}

/// Clamp angular velocity magnitude to `max_speed` (rad/s).
#[allow(dead_code)]
pub fn clamp_angular_velocity(v: [f32; 3], max_speed: f32) -> [f32; 3] {
    let speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if speed > max_speed && speed > 1e-9 {
        let s = max_speed / speed;
        [v[0] * s, v[1] * s, v[2] * s]
    } else {
        v
    }
}

/// Angular velocity from axis-angle (axis must be unit length).
#[allow(dead_code)]
pub fn axis_angle_to_angular_velocity(axis: [f32; 3], radians_per_sec: f32) -> [f32; 3] {
    [
        axis[0] * radians_per_sec,
        axis[1] * radians_per_sec,
        axis[2] * radians_per_sec,
    ]
}

/// Full rotation period in seconds given angular speed (rad/s).
#[allow(dead_code)]
pub fn rotation_period(angular_speed: f32) -> Option<f32> {
    if angular_speed.abs() < 1e-9 {
        None
    } else {
        Some(2.0 * PI / angular_speed.abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_body_zero_velocity() {
        let b = AngularVelocityBody::new(1.0, 0.0);
        assert_eq!(b.angular_speed(), 0.0);
    }

    #[test]
    fn apply_torque_and_integrate_increases_speed() {
        let mut b = AngularVelocityBody::new(1.0, 0.0);
        b.apply_torque([0.0, 10.0, 0.0]);
        b.integrate(0.1);
        assert!(b.angular_speed() > 0.0);
    }

    #[test]
    fn damping_reduces_velocity() {
        let mut b = AngularVelocityBody::new(1.0, 0.5);
        b.set_angular_velocity([0.0, 10.0, 0.0]);
        b.integrate(0.1);
        assert!(b.angular_speed() < 10.0);
    }

    #[test]
    fn kinetic_energy_formula() {
        let mut b = AngularVelocityBody::new(2.0, 0.0);
        b.set_angular_velocity([0.0, 4.0, 0.0]);
        // KE = 0.5 * 2.0 * 16.0 = 16.0
        assert!((b.kinetic_energy() - 16.0).abs() < 1e-4);
    }

    #[test]
    fn normalize_quaternion_unit_length() {
        let mut q = [2.0f32, 0.0, 0.0, 0.0];
        normalize_quaternion(&mut q);
        let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }

    #[test]
    fn clamp_angular_velocity_no_change_within_limit() {
        let v = [1.0f32, 0.0, 0.0];
        let clamped = clamp_angular_velocity(v, 10.0);
        assert!((clamped[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn clamp_angular_velocity_reduces_excess() {
        let v = [0.0f32, 100.0, 0.0];
        let clamped = clamp_angular_velocity(v, 5.0);
        let speed =
            (clamped[0] * clamped[0] + clamped[1] * clamped[1] + clamped[2] * clamped[2]).sqrt();
        assert!((speed - 5.0).abs() < 1e-5);
    }

    #[test]
    fn axis_angle_to_velocity() {
        let v = axis_angle_to_angular_velocity([0.0, 1.0, 0.0], 3.0);
        assert!((v[1] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn rotation_period_computation() {
        let period = rotation_period(PI);
        assert!(period.is_some_and(|p| (p - 2.0).abs() < 1e-5));
    }

    #[test]
    fn rotation_period_zero_returns_none() {
        assert!(rotation_period(0.0).is_none());
    }
}
