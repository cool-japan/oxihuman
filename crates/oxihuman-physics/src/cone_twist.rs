// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A cone-twist constraint limiting rotation to a cone plus twist angle.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ConeTwist {
    anchor: [f32; 3],
    axis: [f32; 3],
    swing_limit: f32,
    twist_limit: f32,
    softness: f32,
    damping: f32,
    current_swing: f32,
    current_twist: f32,
}

#[allow(dead_code)]
impl ConeTwist {
    pub fn new(anchor: [f32; 3], axis: [f32; 3]) -> Self {
        Self {
            anchor,
            axis,
            swing_limit: PI * 0.5,
            twist_limit: PI * 0.25,
            softness: 1.0,
            damping: 0.5,
            current_swing: 0.0,
            current_twist: 0.0,
        }
    }

    pub fn with_swing_limit(mut self, limit: f32) -> Self {
        self.swing_limit = limit.clamp(0.0, PI);
        self
    }

    pub fn with_twist_limit(mut self, limit: f32) -> Self {
        self.twist_limit = limit.clamp(0.0, PI);
        self
    }

    pub fn with_softness(mut self, s: f32) -> Self {
        self.softness = s.clamp(0.0, 1.0);
        self
    }

    pub fn with_damping(mut self, d: f32) -> Self {
        self.damping = d.clamp(0.0, 1.0);
        self
    }

    pub fn set_swing(&mut self, angle: f32) {
        self.current_swing = angle;
    }

    pub fn set_twist(&mut self, angle: f32) {
        self.current_twist = angle;
    }

    pub fn is_swing_violated(&self) -> bool {
        self.current_swing.abs() > self.swing_limit
    }

    pub fn is_twist_violated(&self) -> bool {
        self.current_twist.abs() > self.twist_limit
    }

    pub fn is_violated(&self) -> bool {
        self.is_swing_violated() || self.is_twist_violated()
    }

    pub fn swing_correction(&self) -> f32 {
        if self.is_swing_violated() {
            let excess = self.current_swing.abs() - self.swing_limit;
            excess * self.softness * self.current_swing.signum()
        } else {
            0.0
        }
    }

    pub fn twist_correction(&self) -> f32 {
        if self.is_twist_violated() {
            let excess = self.current_twist.abs() - self.twist_limit;
            excess * self.softness * self.current_twist.signum()
        } else {
            0.0
        }
    }

    pub fn anchor(&self) -> [f32; 3] {
        self.anchor
    }

    pub fn axis(&self) -> [f32; 3] {
        self.axis
    }

    pub fn swing_limit(&self) -> f32 {
        self.swing_limit
    }

    pub fn twist_limit(&self) -> f32 {
        self.twist_limit
    }

    pub fn swing_fraction(&self) -> f32 {
        if self.swing_limit < f32::EPSILON {
            return 0.0;
        }
        (self.current_swing.abs() / self.swing_limit).min(1.0)
    }

    pub fn twist_fraction(&self) -> f32 {
        if self.twist_limit < f32::EPSILON {
            return 0.0;
        }
        (self.current_twist.abs() / self.twist_limit).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ct = ConeTwist::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(!ct.is_violated());
    }

    #[test]
    fn test_swing_violation() {
        let mut ct = ConeTwist::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]).with_swing_limit(0.5);
        ct.set_swing(1.0);
        assert!(ct.is_swing_violated());
    }

    #[test]
    fn test_twist_violation() {
        let mut ct = ConeTwist::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]).with_twist_limit(0.3);
        ct.set_twist(0.5);
        assert!(ct.is_twist_violated());
    }

    #[test]
    fn test_no_violation() {
        let mut ct = ConeTwist::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
            .with_swing_limit(1.0)
            .with_twist_limit(1.0);
        ct.set_swing(0.5);
        ct.set_twist(0.5);
        assert!(!ct.is_violated());
    }

    #[test]
    fn test_swing_correction() {
        let mut ct = ConeTwist::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
            .with_swing_limit(0.5)
            .with_softness(1.0);
        ct.set_swing(0.8);
        let corr = ct.swing_correction();
        assert!(corr > 0.0);
    }

    #[test]
    fn test_no_correction_within_limits() {
        let mut ct = ConeTwist::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]).with_swing_limit(1.0);
        ct.set_swing(0.5);
        assert!((ct.swing_correction() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_swing_fraction() {
        let mut ct = ConeTwist::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]).with_swing_limit(1.0);
        ct.set_swing(0.5);
        assert!((ct.swing_fraction() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_twist_fraction() {
        let mut ct = ConeTwist::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]).with_twist_limit(PI * 0.25);
        ct.set_twist(PI * 0.125);
        assert!((ct.twist_fraction() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_anchor_axis() {
        let ct = ConeTwist::new([1.0, 2.0, 3.0], [0.0, 0.0, 1.0]);
        assert_eq!(ct.anchor(), [1.0, 2.0, 3.0]);
        assert_eq!(ct.axis(), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_clamp_limits() {
        let ct = ConeTwist::new([0.0; 3], [0.0, 1.0, 0.0]).with_swing_limit(10.0);
        assert!((ct.swing_limit() - PI).abs() < f32::EPSILON);
    }
}
