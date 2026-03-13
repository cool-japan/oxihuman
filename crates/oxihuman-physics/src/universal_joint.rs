// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Universal (Hooke's) joint with two independent angular DOFs.

/// Universal joint with two orthogonal rotation axes.
#[derive(Debug, Clone)]
pub struct UniversalJoint {
    /// Angle about first axis (e.g., X).
    pub angle1: f32,
    /// Angle about second axis (e.g., Y).
    pub angle2: f32,
    pub angular_velocity1: f32,
    pub angular_velocity2: f32,
    pub max_angle1: f32,
    pub max_angle2: f32,
    pub damping: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
impl UniversalJoint {
    pub fn new(max_angle1_deg: f32, max_angle2_deg: f32) -> Self {
        UniversalJoint {
            angle1: 0.0,
            angle2: 0.0,
            angular_velocity1: 0.0,
            angular_velocity2: 0.0,
            max_angle1: max_angle1_deg.to_radians(),
            max_angle2: max_angle2_deg.to_radians(),
            damping: 0.0,
            enabled: true,
        }
    }

    pub fn apply_torque1(&mut self, torque: f32, inertia: f32, dt: f32) {
        if !self.enabled {
            return;
        }
        let damp = -self.angular_velocity1 * self.damping;
        self.angular_velocity1 += (torque + damp) / inertia * dt;
        self.angle1 += self.angular_velocity1 * dt;
        self.angle1 = self.angle1.clamp(-self.max_angle1, self.max_angle1);
    }

    pub fn apply_torque2(&mut self, torque: f32, inertia: f32, dt: f32) {
        if !self.enabled {
            return;
        }
        let damp = -self.angular_velocity2 * self.damping;
        self.angular_velocity2 += (torque + damp) / inertia * dt;
        self.angle2 += self.angular_velocity2 * dt;
        self.angle2 = self.angle2.clamp(-self.max_angle2, self.max_angle2);
    }

    /// Deflection angle: combined angular deviation from neutral.
    pub fn deflection_angle(&self) -> f32 {
        (self.angle1.powi(2) + self.angle2.powi(2)).sqrt()
    }

    /// Velocity ratio (Hooke's joint non-uniformity approximation).
    pub fn velocity_ratio(&self) -> f32 {
        let cos1 = self.angle1.cos();
        if cos1.abs() < 1e-6 {
            return 1.0;
        }
        1.0 / (cos1
            * (1.0 - (self.angle1.sin() * self.angle2.sin()).powi(2))
                .sqrt()
                .max(1e-6))
    }

    pub fn is_at_limit1(&self) -> bool {
        self.angle1.abs() >= self.max_angle1 - 1e-6
    }

    pub fn is_at_limit2(&self) -> bool {
        self.angle2.abs() >= self.max_angle2 - 1e-6
    }

    pub fn angle1_deg(&self) -> f32 {
        self.angle1.to_degrees()
    }

    pub fn angle2_deg(&self) -> f32 {
        self.angle2.to_degrees()
    }

    pub fn kinetic_energy(&self, inertia: f32) -> f32 {
        0.5 * inertia * (self.angular_velocity1.powi(2) + self.angular_velocity2.powi(2))
    }

    pub fn reset(&mut self) {
        self.angle1 = 0.0;
        self.angle2 = 0.0;
        self.angular_velocity1 = 0.0;
        self.angular_velocity2 = 0.0;
    }
}

pub fn new_universal_joint(max1_deg: f32, max2_deg: f32) -> UniversalJoint {
    UniversalJoint::new(max1_deg, max2_deg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn neutral_deflection_zero() {
        let j = new_universal_joint(45.0, 45.0);
        assert!(j.deflection_angle() < 1e-6);
    }

    #[test]
    fn torque1_rotates_axis1() {
        let mut j = new_universal_joint(90.0, 90.0);
        j.apply_torque1(10.0, 1.0, 0.1);
        assert!(j.angle1 > 0.0);
    }

    #[test]
    fn torque2_rotates_axis2() {
        let mut j = new_universal_joint(90.0, 90.0);
        j.apply_torque2(10.0, 1.0, 0.1);
        assert!(j.angle2 > 0.0);
    }

    #[test]
    fn clamp_at_max1() {
        let mut j = new_universal_joint(10.0, 90.0);
        j.apply_torque1(1000.0, 1.0, 10.0);
        assert!(j.is_at_limit1());
    }

    #[test]
    fn deflection_combined() {
        let mut j = new_universal_joint(90.0, 90.0);
        j.angle1 = 3.0f32.to_radians();
        j.angle2 = 4.0f32.to_radians();
        let deg = j.deflection_angle().to_degrees();
        assert!((deg - 5.0).abs() < 0.01);
    }

    #[test]
    fn reset_zeroes() {
        let mut j = new_universal_joint(45.0, 45.0);
        j.apply_torque1(10.0, 1.0, 0.5);
        j.reset();
        assert_eq!(j.angle1, 0.0);
    }

    #[test]
    fn angle_deg_conversion() {
        let mut j = new_universal_joint(180.0, 180.0);
        j.angle1 = PI;
        assert!((j.angle1_deg() - 180.0).abs() < 1e-4);
    }

    #[test]
    fn kinetic_energy() {
        let mut j = new_universal_joint(90.0, 90.0);
        j.angular_velocity1 = 3.0;
        j.angular_velocity2 = 4.0;
        assert!((j.kinetic_energy(2.0) - 25.0).abs() < 1e-5);
    }

    #[test]
    fn disabled_no_motion() {
        let mut j = new_universal_joint(90.0, 90.0);
        j.enabled = false;
        j.apply_torque1(100.0, 1.0, 1.0);
        assert_eq!(j.angle1, 0.0);
    }
}
