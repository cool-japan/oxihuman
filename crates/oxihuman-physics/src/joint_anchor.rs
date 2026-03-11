// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Joint anchor: a world-space or local-space attachment point for constraints.

/// Coordinate space of the anchor.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorSpace {
    World,
    Local,
}

/// An anchor point for a joint constraint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointAnchor {
    pub body_id: u32,
    pub local_offset: [f32; 3],
    pub space: AnchorSpace,
}

#[allow(dead_code)]
impl JointAnchor {
    pub fn world(body_id: u32, position: [f32; 3]) -> Self {
        Self {
            body_id,
            local_offset: position,
            space: AnchorSpace::World,
        }
    }

    pub fn local(body_id: u32, offset: [f32; 3]) -> Self {
        Self {
            body_id,
            local_offset: offset,
            space: AnchorSpace::Local,
        }
    }

    /// Transform local offset to world space given body position and rotation (quaternion).
    pub fn world_position(&self, body_pos: [f32; 3], body_rot: [f32; 4]) -> [f32; 3] {
        if self.space == AnchorSpace::World {
            return self.local_offset;
        }
        let rotated = rotate_by_quaternion(self.local_offset, body_rot);
        [
            body_pos[0] + rotated[0],
            body_pos[1] + rotated[1],
            body_pos[2] + rotated[2],
        ]
    }
}

/// Rotate a vector by a unit quaternion [x, y, z, w].
#[allow(dead_code)]
pub fn rotate_by_quaternion(v: [f32; 3], q: [f32; 4]) -> [f32; 3] {
    let [qx, qy, qz, qw] = q;
    let [vx, vy, vz] = v;
    // t = 2 * cross(q.xyz, v)
    let tx = 2.0 * (qy * vz - qz * vy);
    let ty = 2.0 * (qz * vx - qx * vz);
    let tz = 2.0 * (qx * vy - qy * vx);
    [
        vx + qw * tx + qy * tz - qz * ty,
        vy + qw * ty + qz * tx - qx * tz,
        vz + qw * tz + qx * ty - qy * tx,
    ]
}

/// A pair of anchors defining a joint between two bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointAnchorPair {
    pub anchor_a: JointAnchor,
    pub anchor_b: JointAnchor,
    pub compliance: f32,
}

#[allow(dead_code)]
impl JointAnchorPair {
    pub fn new(anchor_a: JointAnchor, anchor_b: JointAnchor, compliance: f32) -> Self {
        Self {
            anchor_a,
            anchor_b,
            compliance,
        }
    }

    /// Separation vector from anchor_a world pos to anchor_b world pos.
    pub fn separation(
        &self,
        pos_a: [f32; 3],
        rot_a: [f32; 4],
        pos_b: [f32; 3],
        rot_b: [f32; 4],
    ) -> [f32; 3] {
        let wa = self.anchor_a.world_position(pos_a, rot_a);
        let wb = self.anchor_b.world_position(pos_b, rot_b);
        [wb[0] - wa[0], wb[1] - wa[1], wb[2] - wa[2]]
    }

    pub fn separation_magnitude(
        &self,
        pos_a: [f32; 3],
        rot_a: [f32; 4],
        pos_b: [f32; 3],
        rot_b: [f32; 4],
    ) -> f32 {
        let s = self.separation(pos_a, rot_a, pos_b, rot_b);
        (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt()
    }

    pub fn is_satisfied(
        &self,
        pos_a: [f32; 3],
        rot_a: [f32; 4],
        pos_b: [f32; 3],
        rot_b: [f32; 4],
        tolerance: f32,
    ) -> bool {
        self.separation_magnitude(pos_a, rot_a, pos_b, rot_b) <= tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_anchor_returns_offset_as_world_pos() {
        let a = JointAnchor::world(0, [1.0, 2.0, 3.0]);
        let pos = a.world_position([0.0; 3], [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(pos, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn local_anchor_identity_rotation() {
        let a = JointAnchor::local(0, [1.0, 0.0, 0.0]);
        let pos = a.world_position([0.0; 3], [0.0, 0.0, 0.0, 1.0]);
        assert!((pos[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn local_anchor_with_body_translation() {
        let a = JointAnchor::local(0, [1.0, 0.0, 0.0]);
        let pos = a.world_position([5.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0]);
        assert!((pos[0] - 6.0).abs() < 1e-5);
    }

    #[test]
    fn rotate_by_identity_quaternion_unchanged() {
        let v = [1.0f32, 2.0, 3.0];
        let rotated = rotate_by_quaternion(v, [0.0, 0.0, 0.0, 1.0]);
        assert!((rotated[0] - 1.0).abs() < 1e-5);
        assert!((rotated[1] - 2.0).abs() < 1e-5);
        assert!((rotated[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn separation_zero_when_coincident() {
        let id_rot = [0.0, 0.0, 0.0, 1.0];
        let pair = JointAnchorPair::new(
            JointAnchor::world(0, [1.0, 0.0, 0.0]),
            JointAnchor::world(1, [1.0, 0.0, 0.0]),
            0.001,
        );
        let mag = pair.separation_magnitude([0.0; 3], id_rot, [0.0; 3], id_rot);
        assert!(mag < 1e-5);
    }

    #[test]
    fn separation_nonzero_when_apart() {
        let id_rot = [0.0, 0.0, 0.0, 1.0];
        let pair = JointAnchorPair::new(
            JointAnchor::world(0, [0.0; 3]),
            JointAnchor::world(1, [3.0, 4.0, 0.0]),
            0.001,
        );
        let mag = pair.separation_magnitude([0.0; 3], id_rot, [0.0; 3], id_rot);
        assert!((mag - 5.0).abs() < 1e-4);
    }

    #[test]
    fn is_satisfied_when_close() {
        let id_rot = [0.0, 0.0, 0.0, 1.0];
        let pair = JointAnchorPair::new(
            JointAnchor::world(0, [0.0; 3]),
            JointAnchor::world(1, [0.001, 0.0, 0.0]),
            0.001,
        );
        assert!(pair.is_satisfied([0.0; 3], id_rot, [0.0; 3], id_rot, 0.01));
    }

    #[test]
    fn anchor_space_local_flag() {
        let a = JointAnchor::local(1, [0.0; 3]);
        assert_eq!(a.space, AnchorSpace::Local);
    }

    #[test]
    fn anchor_space_world_flag() {
        let a = JointAnchor::world(1, [0.0; 3]);
        assert_eq!(a.space, AnchorSpace::World);
    }
}
