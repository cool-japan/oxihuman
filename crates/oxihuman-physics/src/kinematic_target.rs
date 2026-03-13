// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A kinematic target for animating a body to a desired pose.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct KinematicTarget {
    target_pos: [f32; 3],
    target_rot: [f32; 4], // quaternion (x,y,z,w)
    blend_speed: f32,
    active: bool,
}

#[allow(dead_code)]
impl KinematicTarget {
    pub fn new(pos: [f32; 3]) -> Self {
        Self {
            target_pos: pos,
            target_rot: [0.0, 0.0, 0.0, 1.0],
            blend_speed: 10.0,
            active: true,
        }
    }

    pub fn with_rotation(mut self, rot: [f32; 4]) -> Self {
        self.target_rot = rot;
        self
    }

    pub fn with_blend_speed(mut self, speed: f32) -> Self {
        self.blend_speed = speed;
        self
    }

    pub fn target_pos(&self) -> [f32; 3] {
        self.target_pos
    }

    pub fn target_rot(&self) -> [f32; 4] {
        self.target_rot
    }

    pub fn set_target_pos(&mut self, pos: [f32; 3]) {
        self.target_pos = pos;
    }

    pub fn set_target_rot(&mut self, rot: [f32; 4]) {
        self.target_rot = rot;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn interpolate_pos(&self, current: [f32; 3], dt: f32) -> [f32; 3] {
        if !self.active {
            return current;
        }
        let t = (self.blend_speed * dt).min(1.0);
        [
            current[0] + (self.target_pos[0] - current[0]) * t,
            current[1] + (self.target_pos[1] - current[1]) * t,
            current[2] + (self.target_pos[2] - current[2]) * t,
        ]
    }

    pub fn distance_to_target(&self, current: [f32; 3]) -> f32 {
        let dx = self.target_pos[0] - current[0];
        let dy = self.target_pos[1] - current[1];
        let dz = self.target_pos[2] - current[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn is_at_target(&self, current: [f32; 3], tolerance: f32) -> bool {
        self.distance_to_target(current) <= tolerance
    }

    pub fn required_velocity(&self, current: [f32; 3], dt: f32) -> [f32; 3] {
        if dt <= 0.0 || !self.active {
            return [0.0; 3];
        }
        [
            (self.target_pos[0] - current[0]) / dt,
            (self.target_pos[1] - current[1]) / dt,
            (self.target_pos[2] - current[2]) / dt,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let kt = KinematicTarget::new([1.0, 2.0, 3.0]);
        assert_eq!(kt.target_pos(), [1.0, 2.0, 3.0]);
        assert!(kt.is_active());
    }

    #[test]
    fn test_with_rotation() {
        let kt = KinematicTarget::new([0.0; 3]).with_rotation([0.0, 0.0, 0.0, 1.0]);
        assert!((kt.target_rot()[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_interpolate_pos() {
        let kt = KinematicTarget::new([10.0, 0.0, 0.0]).with_blend_speed(1.0);
        let result = kt.interpolate_pos([0.0; 3], 0.5);
        assert!(result[0] > 0.0);
        assert!(result[0] < 10.0);
    }

    #[test]
    fn test_interpolate_inactive() {
        let mut kt = KinematicTarget::new([10.0, 0.0, 0.0]);
        kt.deactivate();
        let result = kt.interpolate_pos([0.0; 3], 1.0);
        assert_eq!(result, [0.0; 3]);
    }

    #[test]
    fn test_distance_to_target() {
        let kt = KinematicTarget::new([3.0, 4.0, 0.0]);
        let d = kt.distance_to_target([0.0; 3]);
        assert!((d - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_at_target() {
        let kt = KinematicTarget::new([0.0; 3]);
        assert!(kt.is_at_target([0.01, 0.0, 0.0], 0.1));
        assert!(!kt.is_at_target([1.0, 0.0, 0.0], 0.1));
    }

    #[test]
    fn test_required_velocity() {
        let kt = KinematicTarget::new([1.0, 0.0, 0.0]);
        let vel = kt.required_velocity([0.0; 3], 0.5);
        assert!((vel[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_required_velocity_zero_dt() {
        let kt = KinematicTarget::new([1.0, 0.0, 0.0]);
        let vel = kt.required_velocity([0.0; 3], 0.0);
        assert_eq!(vel, [0.0; 3]);
    }

    #[test]
    fn test_set_target_pos() {
        let mut kt = KinematicTarget::new([0.0; 3]);
        kt.set_target_pos([5.0, 5.0, 5.0]);
        assert_eq!(kt.target_pos(), [5.0, 5.0, 5.0]);
    }

    #[test]
    fn test_activate_deactivate() {
        let mut kt = KinematicTarget::new([0.0; 3]);
        kt.deactivate();
        assert!(!kt.is_active());
        kt.activate();
        assert!(kt.is_active());
    }
}
