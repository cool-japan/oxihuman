// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Rack-and-pinion mechanical transmission.

use std::f32::consts::PI;

/// Rack-and-pinion coupling: converts linear rack displacement to pinion angle.
#[derive(Debug, Clone)]
pub struct RackPinion {
    pub pitch_radius: f32,
    pub rack_position: f32,
    pub pinion_angle: f32,
    pub gear_ratio: f32,
    pub efficiency: f32,
}

#[allow(dead_code)]
impl RackPinion {
    pub fn new(pitch_radius: f32) -> Self {
        RackPinion {
            pitch_radius: pitch_radius.max(1e-6),
            rack_position: 0.0,
            pinion_angle: 0.0,
            gear_ratio: 1.0,
            efficiency: 1.0,
        }
    }

    /// Advance the rack by `delta` and update the pinion angle.
    pub fn advance_rack(&mut self, delta: f32) {
        self.rack_position += delta;
        self.pinion_angle += delta / self.pitch_radius * self.gear_ratio;
    }

    /// Rotate the pinion by `dtheta` and update the rack position.
    pub fn rotate_pinion(&mut self, dtheta: f32) {
        self.pinion_angle += dtheta;
        self.rack_position += dtheta * self.pitch_radius / self.gear_ratio * self.efficiency;
    }

    pub fn pinion_rpm(&self, rack_speed: f32) -> f32 {
        let angular_speed = rack_speed / self.pitch_radius * self.gear_ratio;
        angular_speed * 60.0 / (2.0 * PI)
    }

    pub fn rack_speed_from_rpm(&self, rpm: f32) -> f32 {
        let omega = rpm * 2.0 * PI / 60.0;
        omega * self.pitch_radius / self.gear_ratio
    }

    pub fn circumference(&self) -> f32 {
        2.0 * PI * self.pitch_radius
    }

    pub fn reset(&mut self) {
        self.rack_position = 0.0;
        self.pinion_angle = 0.0;
    }

    pub fn pinion_angle_deg(&self) -> f32 {
        self.pinion_angle.to_degrees()
    }

    pub fn set_efficiency(&mut self, e: f32) {
        self.efficiency = e.clamp(0.0, 1.0);
    }
}

pub fn new_rack_pinion(pitch_radius: f32) -> RackPinion {
    RackPinion::new(pitch_radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_rack_updates_angle() {
        let mut rp = new_rack_pinion(1.0);
        rp.advance_rack(PI);
        assert!((rp.pinion_angle - PI).abs() < 1e-5);
    }

    #[test]
    fn rotate_pinion_updates_rack() {
        let mut rp = new_rack_pinion(1.0);
        rp.rotate_pinion(PI);
        assert!((rp.rack_position - PI).abs() < 1e-5);
    }

    #[test]
    fn circumference() {
        let rp = new_rack_pinion(1.0);
        assert!((rp.circumference() - 2.0 * PI).abs() < 1e-5);
    }

    #[test]
    fn rpm_conversion() {
        let rp = new_rack_pinion(1.0);
        let rpm = rp.pinion_rpm(2.0 * PI);
        assert!((rpm - 60.0).abs() < 1e-3);
    }

    #[test]
    fn rack_speed_from_rpm() {
        let rp = new_rack_pinion(1.0);
        let speed = rp.rack_speed_from_rpm(60.0);
        assert!((speed - 2.0 * PI).abs() < 1e-3);
    }

    #[test]
    fn reset_zeroes() {
        let mut rp = new_rack_pinion(1.0);
        rp.advance_rack(10.0);
        rp.reset();
        assert_eq!(rp.rack_position, 0.0);
        assert_eq!(rp.pinion_angle, 0.0);
    }

    #[test]
    fn efficiency_clamps() {
        let mut rp = new_rack_pinion(1.0);
        rp.set_efficiency(1.5);
        assert!((0.0..=1.0).contains(&rp.efficiency));
    }

    #[test]
    fn pinion_angle_deg() {
        let mut rp = new_rack_pinion(1.0);
        rp.pinion_angle = PI;
        assert!((rp.pinion_angle_deg() - 180.0).abs() < 1e-4);
    }

    #[test]
    fn gear_ratio_scales_angle() {
        let mut rp = new_rack_pinion(1.0);
        rp.gear_ratio = 2.0;
        rp.advance_rack(1.0);
        assert!((rp.pinion_angle - 2.0).abs() < 1e-5);
    }
}
