// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A distance joint maintains a fixed distance between two anchor points.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistanceJoint {
    pub body_a: u32,
    pub body_b: u32,
    pub anchor_a: [f32; 3],
    pub anchor_b: [f32; 3],
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub min_distance: f32,
    pub max_distance: f32,
}

#[allow(dead_code)]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    vec3_dot(v, v).sqrt()
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
impl DistanceJoint {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        body_a: u32,
        body_b: u32,
        anchor_a: [f32; 3],
        anchor_b: [f32; 3],
        rest_length: f32,
    ) -> Self {
        Self {
            body_a,
            body_b,
            anchor_a,
            anchor_b,
            rest_length,
            stiffness: 1.0,
            damping: 0.1,
            min_distance: 0.0,
            max_distance: f32::MAX,
        }
    }

    pub fn current_distance(&self) -> f32 {
        vec3_len(vec3_sub(self.anchor_b, self.anchor_a))
    }

    pub fn stretch_ratio(&self) -> f32 {
        if self.rest_length <= 0.0 {
            return 1.0;
        }
        self.current_distance() / self.rest_length
    }

    pub fn violation(&self) -> f32 {
        self.current_distance() - self.rest_length
    }

    pub fn correction_force(&self) -> [f32; 3] {
        let delta = vec3_sub(self.anchor_b, self.anchor_a);
        let dist = vec3_len(delta);
        if dist < 1e-10 {
            return [0.0, 0.0, 0.0];
        }
        let error = dist - self.rest_length;
        let clamped_dist = dist.clamp(self.min_distance, self.max_distance);
        let actual_error = clamped_dist - self.rest_length;
        let _ = error; // use clamped
        let dir = vec3_scale(delta, 1.0 / dist);
        vec3_scale(dir, actual_error * self.stiffness)
    }

    pub fn is_stretched(&self) -> bool {
        self.current_distance() > self.rest_length
    }

    pub fn is_compressed(&self) -> bool {
        self.current_distance() < self.rest_length
    }

    pub fn with_stiffness(mut self, s: f32) -> Self {
        self.stiffness = s;
        self
    }

    pub fn with_damping(mut self, d: f32) -> Self {
        self.damping = d;
        self
    }

    pub fn with_limits(mut self, min: f32, max: f32) -> Self {
        self.min_distance = min;
        self.max_distance = max;
        self
    }

    pub fn energy(&self) -> f32 {
        let v = self.violation();
        0.5 * self.stiffness * v * v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_joint(ax: f32, bx: f32, rest: f32) -> DistanceJoint {
        DistanceJoint::new(0, 1, [ax, 0.0, 0.0], [bx, 0.0, 0.0], rest)
    }

    #[test]
    fn test_current_distance() {
        let j = make_joint(0.0, 3.0, 3.0);
        assert!((j.current_distance() - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_violation_zero() {
        let j = make_joint(0.0, 3.0, 3.0);
        assert!(j.violation().abs() < 1e-6);
    }

    #[test]
    fn test_violation_stretched() {
        let j = make_joint(0.0, 5.0, 3.0);
        assert!((j.violation() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_stretched() {
        let j = make_joint(0.0, 5.0, 3.0);
        assert!(j.is_stretched());
    }

    #[test]
    fn test_is_compressed() {
        let j = make_joint(0.0, 1.0, 3.0);
        assert!(j.is_compressed());
    }

    #[test]
    fn test_stretch_ratio() {
        let j = make_joint(0.0, 6.0, 3.0);
        assert!((j.stretch_ratio() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_energy_at_rest() {
        let j = make_joint(0.0, 3.0, 3.0);
        assert!(j.energy() < 1e-10);
    }

    #[test]
    fn test_energy_stretched() {
        let j = make_joint(0.0, 5.0, 3.0);
        assert!(j.energy() > 0.0);
    }

    #[test]
    fn test_with_stiffness() {
        let j = make_joint(0.0, 3.0, 3.0).with_stiffness(10.0);
        assert!((j.stiffness - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_correction_force_at_rest() {
        let j = make_joint(0.0, 3.0, 3.0);
        let f = j.correction_force();
        assert!(f[0].abs() < 1e-6);
    }
}
