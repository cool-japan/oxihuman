#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A pulley joint connecting two bodies through anchors.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PulleyJoint {
    anchor_a: [f32; 3],
    anchor_b: [f32; 3],
    ground_a: [f32; 3],
    ground_b: [f32; 3],
    ratio: f32,
}

#[allow(dead_code)]
pub fn new_pulley_joint(
    anchor_a: [f32; 3], anchor_b: [f32; 3],
    ground_a: [f32; 3], ground_b: [f32; 3],
    ratio: f32,
) -> PulleyJoint {
    PulleyJoint { anchor_a, anchor_b, ground_a, ground_b, ratio }
}

fn segment_length(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn pulley_ratio(joint: &PulleyJoint) -> f32 {
    joint.ratio
}

#[allow(dead_code)]
pub fn pulley_length_a(joint: &PulleyJoint) -> f32 {
    segment_length(joint.ground_a, joint.anchor_a)
}

#[allow(dead_code)]
pub fn pulley_length_b(joint: &PulleyJoint) -> f32 {
    segment_length(joint.ground_b, joint.anchor_b)
}

#[allow(dead_code)]
pub fn pulley_solve(joint: &PulleyJoint) -> f32 {
    let la = pulley_length_a(joint);
    let lb = pulley_length_b(joint);
    la + joint.ratio * lb
}

#[allow(dead_code)]
pub fn pulley_total_length(joint: &PulleyJoint) -> f32 {
    pulley_length_a(joint) + pulley_length_b(joint)
}

#[allow(dead_code)]
pub fn pulley_reset(joint: &mut PulleyJoint) {
    joint.anchor_a = joint.ground_a;
    joint.anchor_b = joint.ground_b;
}

#[allow(dead_code)]
pub fn pulley_force(joint: &PulleyJoint) -> f32 {
    let constraint = pulley_solve(joint);
    constraint * joint.ratio
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pulley_joint() {
        let j = new_pulley_joint([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0], 2.0);
        assert!((pulley_ratio(&j) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_pulley_ratio() {
        let j = new_pulley_joint([0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], 1.5);
        assert!((pulley_ratio(&j) - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_pulley_length_a() {
        let j = new_pulley_joint([0.0, 1.0, 0.0], [0.0; 3], [0.0; 3], [0.0; 3], 1.0);
        assert!((pulley_length_a(&j) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_pulley_length_b() {
        let j = new_pulley_joint([0.0; 3], [0.0, 2.0, 0.0], [0.0; 3], [0.0; 3], 1.0);
        assert!((pulley_length_b(&j) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_pulley_solve() {
        let j = new_pulley_joint([0.0, 1.0, 0.0], [0.0, 2.0, 0.0], [0.0; 3], [0.0; 3], 1.0);
        let s = pulley_solve(&j);
        assert!(s > 0.0);
    }

    #[test]
    fn test_pulley_total_length() {
        let j = new_pulley_joint([0.0, 1.0, 0.0], [0.0, 2.0, 0.0], [0.0; 3], [0.0; 3], 1.0);
        assert!((pulley_total_length(&j) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_pulley_reset() {
        let mut j = new_pulley_joint([0.0, 1.0, 0.0], [0.0, 2.0, 0.0], [0.0; 3], [0.0; 3], 1.0);
        pulley_reset(&mut j);
        assert!((pulley_length_a(&j)).abs() < 1e-6);
    }

    #[test]
    fn test_pulley_force() {
        let j = new_pulley_joint([0.0, 1.0, 0.0], [0.0, 2.0, 0.0], [0.0; 3], [0.0; 3], 1.0);
        assert!(pulley_force(&j) > 0.0);
    }

    #[test]
    fn test_zero_length() {
        let j = new_pulley_joint([0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], 1.0);
        assert!((pulley_total_length(&j)).abs() < 1e-6);
    }

    #[test]
    fn test_ratio_effect() {
        let j1 = new_pulley_joint([0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0; 3], [0.0; 3], 1.0);
        let j2 = new_pulley_joint([0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0; 3], [0.0; 3], 2.0);
        assert!(pulley_force(&j2) > pulley_force(&j1));
    }
}
