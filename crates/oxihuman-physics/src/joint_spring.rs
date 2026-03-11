// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A spring constraint between two anchor points (joint spring).

#[allow(dead_code)]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointSpring {
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub max_force: f32,
}

#[allow(dead_code)]
impl JointSpring {
    pub fn new(rest_length: f32, stiffness: f32, damping: f32) -> Self {
        Self { rest_length, stiffness, damping, max_force: f32::MAX }
    }

    pub fn with_max_force(mut self, max_force: f32) -> Self {
        self.max_force = max_force;
        self
    }

    pub fn compute_force(
        &self,
        pos_a: [f32; 3],
        pos_b: [f32; 3],
        vel_a: [f32; 3],
        vel_b: [f32; 3],
    ) -> [f32; 3] {
        let delta = vec3_sub(pos_b, pos_a);
        let dist = vec3_len(delta);
        if dist < 1e-10 {
            return [0.0; 3];
        }
        let dir = vec3_scale(delta, 1.0 / dist);
        let stretch = dist - self.rest_length;
        let rel_vel = vec3_sub(vel_b, vel_a);
        let vel_along = vec3_dot(rel_vel, dir);
        let force_mag = (self.stiffness * stretch + self.damping * vel_along)
            .clamp(-self.max_force, self.max_force);
        vec3_scale(dir, force_mag)
    }

    pub fn potential_energy(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
        let dist = vec3_len(vec3_sub(pos_b, pos_a));
        let stretch = dist - self.rest_length;
        0.5 * self.stiffness * stretch * stretch
    }

    pub fn current_length(pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
        vec3_len(vec3_sub(pos_b, pos_a))
    }

    pub fn stretch_ratio(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
        if self.rest_length.abs() < 1e-12 {
            return 1.0;
        }
        vec3_len(vec3_sub(pos_b, pos_a)) / self.rest_length
    }

    pub fn is_compressed(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> bool {
        vec3_len(vec3_sub(pos_b, pos_a)) < self.rest_length
    }

    pub fn is_stretched(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> bool {
        vec3_len(vec3_sub(pos_b, pos_a)) > self.rest_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_stretch() {
        let s = JointSpring::new(1.0, 100.0, 0.0);
        let f = s.compute_force([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert!(vec3_len(f) < 1e-5);
    }

    #[test]
    fn test_stretched() {
        let s = JointSpring::new(1.0, 100.0, 0.0);
        let f = s.compute_force([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert!(f[0] > 0.0); // pulls toward b
        assert!((f[0] - 100.0).abs() < 1e-3);
    }

    #[test]
    fn test_compressed() {
        let s = JointSpring::new(2.0, 100.0, 0.0);
        let f = s.compute_force([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert!(f[0] < 0.0); // pushes apart
    }

    #[test]
    fn test_damping() {
        let s = JointSpring::new(1.0, 0.0, 10.0);
        let f = s.compute_force(
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            [0.0; 3], [1.0, 0.0, 0.0],
        );
        assert!(f[0] > 0.0);
    }

    #[test]
    fn test_max_force() {
        let s = JointSpring::new(1.0, 10000.0, 0.0).with_max_force(5.0);
        let f = s.compute_force([0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert!((vec3_len(f) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_potential_energy() {
        let s = JointSpring::new(1.0, 100.0, 0.0);
        let e = s.potential_energy([0.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert!((e - 50.0).abs() < 1e-3);
    }

    #[test]
    fn test_stretch_ratio() {
        let s = JointSpring::new(2.0, 100.0, 0.0);
        let r = s.stretch_ratio([0.0, 0.0, 0.0], [4.0, 0.0, 0.0]);
        assert!((r - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_compressed_stretched() {
        let s = JointSpring::new(2.0, 100.0, 0.0);
        assert!(s.is_compressed([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]));
        assert!(s.is_stretched([0.0, 0.0, 0.0], [3.0, 0.0, 0.0]));
    }

    #[test]
    fn test_coincident_points() {
        let s = JointSpring::new(1.0, 100.0, 0.0);
        let f = s.compute_force([0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3]);
        assert!(vec3_len(f) < 1e-5);
    }

    #[test]
    fn test_current_length() {
        let l = JointSpring::current_length([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((l - 5.0).abs() < 1e-4);
    }
}
