// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Improved motor joint with PD control and limits.
#[allow(dead_code)]
pub struct MotorJointV2 {
    pub angle: f32,
    pub angular_vel: f32,
    pub target_angle: f32,
    pub min_angle: f32,
    pub max_angle: f32,
    pub kp: f32,
    pub kd: f32,
    pub max_torque: f32,
    pub inertia: f32,
}

#[allow(dead_code)]
impl MotorJointV2 {
    pub fn new(
        min_angle: f32,
        max_angle: f32,
        kp: f32,
        kd: f32,
        max_torque: f32,
        inertia: f32,
    ) -> Self {
        Self {
            angle: 0.0,
            angular_vel: 0.0,
            target_angle: 0.0,
            min_angle,
            max_angle,
            kp,
            kd,
            max_torque,
            inertia,
        }
    }
    pub fn set_target(&mut self, t: f32) {
        self.target_angle = t.clamp(self.min_angle, self.max_angle);
    }
    pub fn step(&mut self, dt: f32) -> f32 {
        let err = angle_diff(self.target_angle, self.angle);
        let torque =
            (self.kp * err - self.kd * self.angular_vel).clamp(-self.max_torque, self.max_torque);
        let alpha = torque / self.inertia.max(1e-10);
        self.angular_vel += alpha * dt;
        self.angle = (self.angle + self.angular_vel * dt).clamp(self.min_angle, self.max_angle);
        torque
    }
    pub fn at_target(&self, tol_rad: f32) -> bool {
        angle_diff(self.target_angle, self.angle).abs() <= tol_rad
    }
    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.inertia * self.angular_vel * self.angular_vel
    }
    pub fn range_rad(&self) -> f32 {
        self.max_angle - self.min_angle
    }
    pub fn normalized_angle(&self) -> f32 {
        let r = self.range_rad();
        if r < 1e-8 {
            0.0
        } else {
            (self.angle - self.min_angle) / r
        }
    }
    pub fn reset(&mut self) {
        self.angle = 0.0;
        self.angular_vel = 0.0;
        self.target_angle = 0.0;
    }
}

#[allow(dead_code)]
pub fn angle_diff(target: f32, current: f32) -> f32 {
    let mut d = target - current;
    while d > PI {
        d -= 2.0 * PI;
    }
    while d < -PI {
        d += 2.0 * PI;
    }
    d
}

#[allow(dead_code)]
pub fn new_motor_joint_v2(
    min_a: f32,
    max_a: f32,
    kp: f32,
    kd: f32,
    max_torque: f32,
    inertia: f32,
) -> MotorJointV2 {
    MotorJointV2::new(min_a, max_a, kp, kd, max_torque, inertia)
}
#[allow(dead_code)]
pub fn mjv2_set_target(m: &mut MotorJointV2, t: f32) {
    m.set_target(t);
}
#[allow(dead_code)]
pub fn mjv2_step(m: &mut MotorJointV2, dt: f32) -> f32 {
    m.step(dt)
}
#[allow(dead_code)]
pub fn mjv2_at_target(m: &MotorJointV2, tol: f32) -> bool {
    m.at_target(tol)
}
#[allow(dead_code)]
pub fn mjv2_kinetic_energy(m: &MotorJointV2) -> f32 {
    m.kinetic_energy()
}
#[allow(dead_code)]
pub fn mjv2_normalized_angle(m: &MotorJointV2) -> f32 {
    m.normalized_angle()
}
#[allow(dead_code)]
pub fn mjv2_range(m: &MotorJointV2) -> f32 {
    m.range_rad()
}
#[allow(dead_code)]
pub fn mjv2_reset(m: &mut MotorJointV2) {
    m.reset();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_moves_to_target() {
        let mut m = new_motor_joint_v2(-PI, PI, 100.0, 10.0, 50.0, 1.0);
        mjv2_set_target(&mut m, 1.0);
        for _ in 0..100 {
            mjv2_step(&mut m, 0.01);
        }
        assert!(m.angle > 0.5);
    }
    #[test]
    fn test_clamped_to_range() {
        let mut m = new_motor_joint_v2(-0.5, 0.5, 100.0, 5.0, 100.0, 1.0);
        mjv2_set_target(&mut m, 10.0);
        for _ in 0..100 {
            mjv2_step(&mut m, 0.01);
        }
        assert!(m.angle <= 0.5 + 1e-5);
    }
    #[test]
    fn test_at_target() {
        // Use a critically-damped-ish PD: kd^2 = 4*kp*I => kd = 2*sqrt(kp*I)
        // kp=100, I=0.1 => kd=2*sqrt(10)~6.3 => use kd=20 for overdamped
        let mut m = new_motor_joint_v2(-PI, PI, 200.0, 100.0, 20.0, 0.1);
        mjv2_set_target(&mut m, 0.5);
        for _ in 0..5000 {
            mjv2_step(&mut m, 0.001);
        }
        assert!(mjv2_at_target(&m, 0.1));
    }
    #[test]
    fn test_kinetic_energy_zero_at_rest() {
        let m = new_motor_joint_v2(-PI, PI, 10.0, 1.0, 10.0, 1.0);
        assert_eq!(mjv2_kinetic_energy(&m), 0.0);
    }
    #[test]
    fn test_reset() {
        let mut m = new_motor_joint_v2(-PI, PI, 10.0, 1.0, 10.0, 1.0);
        mjv2_set_target(&mut m, 1.0);
        mjv2_step(&mut m, 0.1);
        mjv2_reset(&mut m);
        assert_eq!(m.angle, 0.0);
        assert_eq!(m.angular_vel, 0.0);
    }
    #[test]
    fn test_range() {
        let m = new_motor_joint_v2(-1.0, 1.0, 1.0, 0.1, 10.0, 1.0);
        assert!((mjv2_range(&m) - 2.0).abs() < 1e-5);
    }
    #[test]
    fn test_normalized_angle() {
        let mut m = new_motor_joint_v2(0.0, 2.0, 1.0, 0.1, 10.0, 1.0);
        m.angle = 1.0;
        assert!((mjv2_normalized_angle(&m) - 0.5).abs() < 1e-5);
    }
    #[test]
    fn test_angle_diff_wraps() {
        let d = angle_diff(0.1, 6.2);
        assert!(d.abs() < PI + 1e-5);
    }
    #[test]
    fn test_torque_limited() {
        let mut m = new_motor_joint_v2(-PI, PI, 1000.0, 0.0, 1.0, 1.0);
        let torque = mjv2_step(&mut m, 0.01);
        assert!(torque.abs() <= 1.0 + 1e-5);
    }
    #[test]
    fn test_target_clamped() {
        let mut m = new_motor_joint_v2(-1.0, 1.0, 1.0, 0.0, 10.0, 1.0);
        mjv2_set_target(&mut m, 100.0);
        assert_eq!(m.target_angle, 1.0);
    }
}
