// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Rigid body rotating about a fixed pivot point in 2D.
#[allow(dead_code)]
pub struct PivotBody {
    pub pivot: [f32; 2],
    pub arm_length: f32,
    pub angle: f32,
    pub angular_vel: f32,
    pub mass: f32,
    pub damping: f32,
}

#[allow(dead_code)]
impl PivotBody {
    pub fn new(pivot: [f32; 2], arm_length: f32, mass: f32, damping: f32) -> Self {
        Self {
            pivot,
            arm_length,
            angle: 0.0,
            angular_vel: 0.0,
            mass,
            damping,
        }
    }
    pub fn apply_torque(&mut self, torque: f32, dt: f32) {
        let inertia = self.mass * self.arm_length * self.arm_length;
        let alpha = torque / inertia.max(1e-10);
        self.angular_vel = self.angular_vel * (1.0 - self.damping * dt) + alpha * dt;
        self.angle += self.angular_vel * dt;
    }
    pub fn tip_position(&self) -> [f32; 2] {
        [
            self.pivot[0] + self.arm_length * self.angle.cos(),
            self.pivot[1] + self.arm_length * self.angle.sin(),
        ]
    }
    pub fn kinetic_energy(&self) -> f32 {
        let inertia = self.mass * self.arm_length * self.arm_length;
        0.5 * inertia * self.angular_vel * self.angular_vel
    }
    pub fn tip_velocity(&self) -> f32 {
        self.arm_length * self.angular_vel.abs()
    }
    pub fn angle_deg(&self) -> f32 {
        self.angle * 180.0 / PI
    }
    pub fn normalize_angle(&mut self) {
        while self.angle > PI {
            self.angle -= 2.0 * PI;
        }
        while self.angle < -PI {
            self.angle += 2.0 * PI;
        }
    }
    pub fn inertia(&self) -> f32 {
        self.mass * self.arm_length * self.arm_length
    }
    pub fn reset(&mut self) {
        self.angle = 0.0;
        self.angular_vel = 0.0;
    }
}

#[allow(dead_code)]
pub fn new_pivot_body(pivot: [f32; 2], arm: f32, mass: f32, damping: f32) -> PivotBody {
    PivotBody::new(pivot, arm, mass, damping)
}
#[allow(dead_code)]
pub fn pivb_apply_torque(b: &mut PivotBody, torque: f32, dt: f32) {
    b.apply_torque(torque, dt);
}
#[allow(dead_code)]
pub fn pivb_tip_pos(b: &PivotBody) -> [f32; 2] {
    b.tip_position()
}
#[allow(dead_code)]
pub fn pivb_kinetic_energy(b: &PivotBody) -> f32 {
    b.kinetic_energy()
}
#[allow(dead_code)]
pub fn pivb_tip_velocity(b: &PivotBody) -> f32 {
    b.tip_velocity()
}
#[allow(dead_code)]
pub fn pivb_angle_deg(b: &PivotBody) -> f32 {
    b.angle_deg()
}
#[allow(dead_code)]
pub fn pivb_inertia(b: &PivotBody) -> f32 {
    b.inertia()
}
#[allow(dead_code)]
pub fn pivb_reset(b: &mut PivotBody) {
    b.reset();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_initial_tip() {
        let b = new_pivot_body([0.0, 0.0], 1.0, 1.0, 0.0);
        let tip = pivb_tip_pos(&b);
        assert!((tip[0] - 1.0).abs() < 1e-5);
        assert!(tip[1].abs() < 1e-5);
    }
    #[test]
    fn test_torque_rotates() {
        let mut b = new_pivot_body([0.0, 0.0], 1.0, 1.0, 0.0);
        pivb_apply_torque(&mut b, 10.0, 0.1);
        assert!(b.angle > 0.0);
    }
    #[test]
    fn test_kinetic_energy_zero_at_rest() {
        let b = new_pivot_body([0.0, 0.0], 1.0, 1.0, 0.0);
        assert_eq!(pivb_kinetic_energy(&b), 0.0);
    }
    #[test]
    fn test_kinetic_energy_positive_when_spinning() {
        let mut b = new_pivot_body([0.0, 0.0], 1.0, 1.0, 0.0);
        pivb_apply_torque(&mut b, 10.0, 0.1);
        assert!(pivb_kinetic_energy(&b) > 0.0);
    }
    #[test]
    fn test_tip_velocity() {
        let mut b = new_pivot_body([0.0, 0.0], 2.0, 1.0, 0.0);
        b.angular_vel = 3.0;
        assert!((pivb_tip_velocity(&b) - 6.0).abs() < 1e-5);
    }
    #[test]
    fn test_inertia() {
        let b = new_pivot_body([0.0, 0.0], 2.0, 3.0, 0.0);
        assert!((pivb_inertia(&b) - 12.0).abs() < 1e-5);
    }
    #[test]
    fn test_damping_reduces_energy() {
        let mut b = new_pivot_body([0.0, 0.0], 1.0, 1.0, 1.0);
        b.angular_vel = 5.0;
        let e0 = pivb_kinetic_energy(&b);
        pivb_apply_torque(&mut b, 0.0, 0.1);
        let e1 = pivb_kinetic_energy(&b);
        assert!(e1 < e0);
    }
    #[test]
    fn test_reset() {
        let mut b = new_pivot_body([0.0, 0.0], 1.0, 1.0, 0.0);
        pivb_apply_torque(&mut b, 10.0, 0.5);
        pivb_reset(&mut b);
        assert_eq!(b.angle, 0.0);
        assert_eq!(b.angular_vel, 0.0);
    }
    #[test]
    fn test_pivot_offset() {
        let b = new_pivot_body([1.0, 2.0], 1.0, 1.0, 0.0);
        let tip = pivb_tip_pos(&b);
        assert!((tip[0] - 2.0).abs() < 1e-5);
    }
    #[test]
    fn test_angle_deg() {
        let mut b = new_pivot_body([0.0, 0.0], 1.0, 1.0, 0.0);
        b.angle = std::f32::consts::FRAC_PI_2;
        assert!((pivb_angle_deg(&b) - 90.0).abs() < 1e-3);
    }
}
