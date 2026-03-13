// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Screw joint: couples linear and angular DOFs via a lead.

use std::f32::consts::PI;

/// Screw joint coupling rotation to translation.
#[derive(Debug, Clone)]
pub struct ScrewJoint {
    /// Advance per revolution (metres/rev).
    pub lead: f32,
    pub angle: f32,
    pub position: f32,
    pub min_pos: f32,
    pub max_pos: f32,
    pub angular_velocity: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
impl ScrewJoint {
    pub fn new(lead: f32, min_pos: f32, max_pos: f32) -> Self {
        ScrewJoint {
            lead,
            angle: 0.0,
            position: 0.0,
            min_pos,
            max_pos,
            angular_velocity: 0.0,
            enabled: true,
        }
    }

    /// Rotate by `dtheta` radians and update linear position.
    pub fn rotate(&mut self, dtheta: f32) {
        if !self.enabled {
            return;
        }
        self.angle += dtheta;
        self.position += dtheta / (2.0 * PI) * self.lead;
        self.clamp_position();
    }

    /// Set linear position and derive angle.
    pub fn set_position(&mut self, pos: f32) {
        self.position = pos.clamp(self.min_pos, self.max_pos);
        self.angle = self.position / self.lead * 2.0 * PI;
    }

    pub fn clamp_position(&mut self) {
        if self.position < self.min_pos {
            self.position = self.min_pos;
        } else if self.position > self.max_pos {
            self.position = self.max_pos;
        }
    }

    pub fn is_at_min(&self) -> bool {
        self.position <= self.min_pos + 1e-6
    }

    pub fn is_at_max(&self) -> bool {
        self.position >= self.max_pos - 1e-6
    }

    pub fn linear_speed(&self) -> f32 {
        self.angular_velocity / (2.0 * PI) * self.lead
    }

    pub fn travel_range(&self) -> f32 {
        self.max_pos - self.min_pos
    }

    pub fn angle_deg(&self) -> f32 {
        self.angle.to_degrees()
    }

    pub fn normalized_position(&self) -> f32 {
        let range = self.travel_range();
        if range < 1e-9 {
            return 0.0;
        }
        (self.position - self.min_pos) / range
    }

    pub fn reset(&mut self) {
        self.angle = 0.0;
        self.position = self.min_pos;
    }
}

pub fn new_screw_joint(lead: f32, min_pos: f32, max_pos: f32) -> ScrewJoint {
    ScrewJoint::new(lead, min_pos, max_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_advances_position() {
        let mut j = new_screw_joint(0.01, 0.0, 1.0);
        j.rotate(2.0 * PI);
        assert!((j.position - 0.01).abs() < 1e-5);
    }

    #[test]
    fn set_position_derives_angle() {
        let mut j = new_screw_joint(0.01, 0.0, 1.0);
        j.set_position(0.005);
        assert!((j.angle - PI).abs() < 1e-4);
    }

    #[test]
    fn clamp_to_max() {
        let mut j = new_screw_joint(0.01, 0.0, 0.05);
        j.rotate(100.0 * 2.0 * PI);
        assert!(j.is_at_max());
    }

    #[test]
    fn clamp_to_min() {
        let mut j = new_screw_joint(0.01, -0.05, 0.05);
        j.rotate(-100.0 * 2.0 * PI);
        assert!(j.is_at_min());
    }

    #[test]
    fn linear_speed_from_angular() {
        let mut j = new_screw_joint(0.01, 0.0, 1.0);
        j.angular_velocity = 2.0 * PI;
        assert!((j.linear_speed() - 0.01).abs() < 1e-6);
    }

    #[test]
    fn travel_range() {
        let j = new_screw_joint(0.01, -0.05, 0.05);
        assert!((j.travel_range() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn normalized_position_mid() {
        let mut j = new_screw_joint(0.01, 0.0, 0.1);
        j.set_position(0.05);
        assert!((j.normalized_position() - 0.5).abs() < 1e-4);
    }

    #[test]
    fn reset_to_min() {
        let mut j = new_screw_joint(0.01, 0.0, 1.0);
        j.rotate(2.0 * PI);
        j.reset();
        assert!(j.is_at_min());
    }

    #[test]
    fn disabled_no_motion() {
        let mut j = new_screw_joint(0.01, 0.0, 1.0);
        j.enabled = false;
        j.rotate(100.0);
        assert_eq!(j.position, 0.0);
    }
}
