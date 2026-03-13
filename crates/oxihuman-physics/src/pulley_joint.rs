// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A pulley joint constraining total rope length between two anchors.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct PulleyJoint {
    ground_anchor_a: [f32; 3],
    ground_anchor_b: [f32; 3],
    total_length: f32,
    ratio: f32,
    stiffness: f32,
    damping: f32,
    min_length: f32,
}

#[allow(dead_code)]
impl PulleyJoint {
    pub fn new(ground_anchor_a: [f32; 3], ground_anchor_b: [f32; 3], total_length: f32) -> Self {
        Self {
            ground_anchor_a,
            ground_anchor_b,
            total_length: total_length.max(0.0),
            ratio: 1.0,
            stiffness: 500.0,
            damping: 50.0,
            min_length: 0.0,
        }
    }

    pub fn with_ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio.max(0.01);
        self
    }

    pub fn with_stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }

    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    pub fn with_min_length(mut self, min_length: f32) -> Self {
        self.min_length = min_length.max(0.0);
        self
    }

    pub fn ground_anchor_a(&self) -> [f32; 3] {
        self.ground_anchor_a
    }

    pub fn ground_anchor_b(&self) -> [f32; 3] {
        self.ground_anchor_b
    }

    pub fn total_length(&self) -> f32 {
        self.total_length
    }

    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn current_lengths(&self, body_a: [f32; 3], body_b: [f32; 3]) -> (f32, f32) {
        let la = Self::distance(self.ground_anchor_a, body_a);
        let lb = Self::distance(self.ground_anchor_b, body_b);
        (la, lb)
    }

    pub fn constraint_error(&self, body_a: [f32; 3], body_b: [f32; 3]) -> f32 {
        let (la, lb) = self.current_lengths(body_a, body_b);
        la + self.ratio * lb - self.total_length
    }

    pub fn is_satisfied(&self, body_a: [f32; 3], body_b: [f32; 3], tolerance: f32) -> bool {
        self.constraint_error(body_a, body_b).abs() <= tolerance
    }

    pub fn compute_correction(&self, body_a: [f32; 3], body_b: [f32; 3]) -> f32 {
        let error = self.constraint_error(body_a, body_b);
        -self.stiffness * error
    }

    pub fn length_a_for_b(&self, length_b: f32) -> f32 {
        (self.total_length - self.ratio * length_b).max(self.min_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let pj = PulleyJoint::new([0.0, 5.0, 0.0], [4.0, 5.0, 0.0], 10.0);
        assert!((pj.total_length() - 10.0).abs() < 1e-6);
        assert!((pj.ratio() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_with_ratio() {
        let pj = PulleyJoint::new([0.0; 3], [0.0; 3], 10.0).with_ratio(2.0);
        assert!((pj.ratio() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_current_lengths() {
        let pj = PulleyJoint::new([0.0, 10.0, 0.0], [10.0, 10.0, 0.0], 20.0);
        let (la, lb) = pj.current_lengths([0.0, 5.0, 0.0], [10.0, 5.0, 0.0]);
        assert!((la - 5.0).abs() < 1e-4);
        assert!((lb - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_constraint_error_satisfied() {
        let pj = PulleyJoint::new([0.0, 10.0, 0.0], [10.0, 10.0, 0.0], 10.0);
        let err = pj.constraint_error([0.0, 5.0, 0.0], [10.0, 5.0, 0.0]);
        assert!((err).abs() < 1e-4);
    }

    #[test]
    fn test_is_satisfied() {
        let pj = PulleyJoint::new([0.0, 10.0, 0.0], [10.0, 10.0, 0.0], 10.0);
        assert!(pj.is_satisfied([0.0, 5.0, 0.0], [10.0, 5.0, 0.0], 0.1));
    }

    #[test]
    fn test_constraint_error_violated() {
        let pj = PulleyJoint::new([0.0, 10.0, 0.0], [10.0, 10.0, 0.0], 5.0);
        let err = pj.constraint_error([0.0, 5.0, 0.0], [10.0, 5.0, 0.0]);
        assert!(err > 0.0);
    }

    #[test]
    fn test_compute_correction() {
        let pj = PulleyJoint::new([0.0, 10.0, 0.0], [10.0, 10.0, 0.0], 5.0);
        let c = pj.compute_correction([0.0, 5.0, 0.0], [10.0, 5.0, 0.0]);
        assert!(c < 0.0); // Should push back
    }

    #[test]
    fn test_length_a_for_b() {
        let pj = PulleyJoint::new([0.0; 3], [0.0; 3], 10.0);
        let la = pj.length_a_for_b(3.0);
        assert!((la - 7.0).abs() < 1e-4);
    }

    #[test]
    fn test_length_a_clamped() {
        let pj = PulleyJoint::new([0.0; 3], [0.0; 3], 10.0).with_min_length(5.0);
        let la = pj.length_a_for_b(8.0);
        assert!((la - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_ground_anchors() {
        let pj = PulleyJoint::new([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], 10.0);
        assert_eq!(pj.ground_anchor_a(), [1.0, 2.0, 3.0]);
        assert_eq!(pj.ground_anchor_b(), [4.0, 5.0, 6.0]);
    }
}
