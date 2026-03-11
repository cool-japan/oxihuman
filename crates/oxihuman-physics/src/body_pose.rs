// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Represents a rigid body pose (position + orientation as quaternion).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BodyPose {
    pub position: [f32; 3],
    pub rotation: [f32; 4], // quaternion [x, y, z, w]
}

#[allow(dead_code)]
impl BodyPose {
    pub fn identity() -> Self {
        Self {
            position: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn from_position(pos: [f32; 3]) -> Self {
        Self {
            position: pos,
            rotation: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn from_position_rotation(pos: [f32; 3], rot: [f32; 4]) -> Self {
        let mut pose = Self {
            position: pos,
            rotation: rot,
        };
        pose.normalize_rotation();
        pose
    }

    pub fn normalize_rotation(&mut self) {
        let mag_sq: f32 = self.rotation.iter().map(|&r| r * r).sum();
        let mag = mag_sq.sqrt();
        if mag > 1e-9 {
            for r in &mut self.rotation {
                *r /= mag;
            }
        }
    }

    pub fn distance_to(&self, other: &BodyPose) -> f32 {
        let dx = self.position[0] - other.position[0];
        let dy = self.position[1] - other.position[1];
        let dz = self.position[2] - other.position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn lerp(&self, other: &BodyPose, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let mut pos = [0.0f32; 3];
        for (i, p) in pos.iter_mut().enumerate() {
            *p = self.position[i] + (other.position[i] - self.position[i]) * t;
        }
        let mut rot = [0.0f32; 4];
        for (i, r) in rot.iter_mut().enumerate() {
            *r = self.rotation[i] + (other.rotation[i] - self.rotation[i]) * t;
        }
        let mut result = Self {
            position: pos,
            rotation: rot,
        };
        result.normalize_rotation();
        result
    }

    pub fn translate(&mut self, offset: [f32; 3]) {
        for (p, o) in self.position.iter_mut().zip(offset.iter()) {
            *p += o;
        }
    }

    pub fn forward(&self) -> [f32; 3] {
        let [x, y, z, w] = self.rotation;
        [
            2.0 * (x * z + w * y),
            2.0 * (y * z - w * x),
            1.0 - 2.0 * (x * x + y * y),
        ]
    }

    pub fn inverse(&self) -> Self {
        let neg_pos = [-self.position[0], -self.position[1], -self.position[2]];
        let inv_rot = [-self.rotation[0], -self.rotation[1], -self.rotation[2], self.rotation[3]];
        Self {
            position: neg_pos,
            rotation: inv_rot,
        }
    }
}

impl Default for BodyPose {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let p = BodyPose::identity();
        assert_eq!(p.position, [0.0; 3]);
        assert!((p.rotation[3] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_from_position() {
        let p = BodyPose::from_position([1.0, 2.0, 3.0]);
        assert_eq!(p.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_distance_to() {
        let a = BodyPose::from_position([0.0, 0.0, 0.0]);
        let b = BodyPose::from_position([3.0, 4.0, 0.0]);
        assert!((a.distance_to(&b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_lerp_at_zero() {
        let a = BodyPose::from_position([0.0; 3]);
        let b = BodyPose::from_position([10.0, 0.0, 0.0]);
        let result = a.lerp(&b, 0.0);
        assert!((result.position[0]).abs() < 1e-6);
    }

    #[test]
    fn test_lerp_at_one() {
        let a = BodyPose::from_position([0.0; 3]);
        let b = BodyPose::from_position([10.0, 0.0, 0.0]);
        let result = a.lerp(&b, 1.0);
        assert!((result.position[0] - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_lerp_midpoint() {
        let a = BodyPose::from_position([0.0; 3]);
        let b = BodyPose::from_position([10.0, 0.0, 0.0]);
        let result = a.lerp(&b, 0.5);
        assert!((result.position[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_translate() {
        let mut p = BodyPose::from_position([1.0, 2.0, 3.0]);
        p.translate([1.0, 1.0, 1.0]);
        assert_eq!(p.position, [2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_normalize_rotation() {
        let mut p = BodyPose::identity();
        p.rotation = [0.0, 0.0, 0.0, 2.0];
        p.normalize_rotation();
        assert!((p.rotation[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_inverse() {
        let p = BodyPose::from_position([1.0, 2.0, 3.0]);
        let inv = p.inverse();
        assert_eq!(inv.position, [-1.0, -2.0, -3.0]);
    }

    #[test]
    fn test_forward_identity() {
        let p = BodyPose::identity();
        let fwd = p.forward();
        assert!((fwd[2] - 1.0).abs() < 1e-6);
    }
}
