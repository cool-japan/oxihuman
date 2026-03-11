#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A distance joint constraining two bodies to a distance range.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DistanceJoint {
    anchor_a: [f32; 3],
    anchor_b: [f32; 3],
    min_length: f32,
    max_length: f32,
    stiffness: f32,
}

#[allow(dead_code)]
pub fn new_distance_joint(anchor_a: [f32; 3], anchor_b: [f32; 3], min_len: f32, max_len: f32) -> DistanceJoint {
    DistanceJoint {
        anchor_a,
        anchor_b,
        min_length: min_len,
        max_length: max_len,
        stiffness: 1.0,
    }
}

#[allow(dead_code)]
pub fn distance_error_dj(joint: &DistanceJoint) -> f32 {
    let d = current_distance(joint);
    if d < joint.min_length {
        joint.min_length - d
    } else if d > joint.max_length {
        d - joint.max_length
    } else {
        0.0
    }
}

fn current_distance(joint: &DistanceJoint) -> f32 {
    let dx = joint.anchor_b[0] - joint.anchor_a[0];
    let dy = joint.anchor_b[1] - joint.anchor_a[1];
    let dz = joint.anchor_b[2] - joint.anchor_a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn distance_solve(joint: &DistanceJoint) -> [f32; 3] {
    let err = distance_error_dj(joint);
    if err.abs() < 1e-6 {
        return [0.0, 0.0, 0.0];
    }
    let d = current_distance(joint);
    if d < 1e-6 {
        return [0.0, 0.0, 0.0];
    }
    let scale = err * joint.stiffness / d;
    let dx = joint.anchor_b[0] - joint.anchor_a[0];
    let dy = joint.anchor_b[1] - joint.anchor_a[1];
    let dz = joint.anchor_b[2] - joint.anchor_a[2];
    [dx * scale, dy * scale, dz * scale]
}

#[allow(dead_code)]
pub fn distance_min_length(joint: &DistanceJoint) -> f32 {
    joint.min_length
}

#[allow(dead_code)]
pub fn distance_max_length(joint: &DistanceJoint) -> f32 {
    joint.max_length
}

#[allow(dead_code)]
pub fn distance_set_range(joint: &mut DistanceJoint, min_len: f32, max_len: f32) {
    joint.min_length = min_len;
    joint.max_length = max_len;
}

#[allow(dead_code)]
pub fn distance_reset(joint: &mut DistanceJoint) {
    joint.anchor_a = [0.0; 3];
    joint.anchor_b = [0.0; 3];
}

#[allow(dead_code)]
pub fn distance_force(joint: &DistanceJoint) -> f32 {
    distance_error_dj(joint) * joint.stiffness
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_distance_joint() {
        let j = new_distance_joint([0.0; 3], [1.0, 0.0, 0.0], 0.5, 2.0);
        assert!((j.min_length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_distance_error_within_range() {
        let j = new_distance_joint([0.0; 3], [1.0, 0.0, 0.0], 0.5, 2.0);
        assert!((distance_error_dj(&j)).abs() < 1e-6);
    }

    #[test]
    fn test_distance_error_too_short() {
        let j = new_distance_joint([0.0; 3], [0.1, 0.0, 0.0], 0.5, 2.0);
        assert!(distance_error_dj(&j) > 0.0);
    }

    #[test]
    fn test_distance_error_too_long() {
        let j = new_distance_joint([0.0; 3], [3.0, 0.0, 0.0], 0.5, 2.0);
        assert!(distance_error_dj(&j) > 0.0);
    }

    #[test]
    fn test_distance_solve_no_error() {
        let j = new_distance_joint([0.0; 3], [1.0, 0.0, 0.0], 0.5, 2.0);
        let f = distance_solve(&j);
        assert!(f[0].abs() < 1e-6);
    }

    #[test]
    fn test_distance_min_max() {
        let j = new_distance_joint([0.0; 3], [1.0, 0.0, 0.0], 0.5, 2.0);
        assert!((distance_min_length(&j) - 0.5).abs() < 1e-6);
        assert!((distance_max_length(&j) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_distance_set_range() {
        let mut j = new_distance_joint([0.0; 3], [1.0, 0.0, 0.0], 0.5, 2.0);
        distance_set_range(&mut j, 1.0, 3.0);
        assert!((distance_min_length(&j) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_distance_reset() {
        let mut j = new_distance_joint([1.0; 3], [2.0; 3], 0.5, 2.0);
        distance_reset(&mut j);
        assert!((j.anchor_a[0]).abs() < 1e-6);
    }

    #[test]
    fn test_distance_force() {
        let j = new_distance_joint([0.0; 3], [0.1, 0.0, 0.0], 0.5, 2.0);
        assert!(distance_force(&j) > 0.0);
    }

    #[test]
    fn test_distance_force_zero() {
        let j = new_distance_joint([0.0; 3], [1.0, 0.0, 0.0], 0.5, 2.0);
        assert!(distance_force(&j).abs() < 1e-6);
    }
}
