// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A single elastic strand (hair/thread) modeled as a chain of particles.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ElasticStrand {
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
    pub rest_lengths: Vec<f32>,
    pub stiffness: f32,
    pub damping: f32,
    pub mass_per_node: f32,
    pub pinned_root: bool,
}

#[allow(dead_code)]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
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
impl ElasticStrand {
    pub fn new_straight(root: [f32; 3], direction: [f32; 3], num_nodes: usize, segment_length: f32) -> Self {
        assert!(num_nodes >= 2);
        let mut positions = Vec::with_capacity(num_nodes);
        let dir_len = vec3_len(direction);
        let norm_dir = if dir_len > 1e-10 {
            vec3_scale(direction, 1.0 / dir_len)
        } else {
            [0.0, -1.0, 0.0]
        };
        for i in 0..num_nodes {
            let offset = vec3_scale(norm_dir, i as f32 * segment_length);
            positions.push(vec3_add(root, offset));
        }
        let rest_lengths = vec![segment_length; num_nodes - 1];
        Self {
            positions,
            velocities: vec![[0.0; 3]; num_nodes],
            rest_lengths,
            stiffness: 100.0,
            damping: 1.0,
            mass_per_node: 0.01,
            pinned_root: true,
        }
    }

    pub fn node_count(&self) -> usize {
        self.positions.len()
    }

    pub fn segment_count(&self) -> usize {
        self.positions.len().saturating_sub(1)
    }

    pub fn total_length(&self) -> f32 {
        let mut len = 0.0;
        for i in 1..self.positions.len() {
            len += vec3_len(vec3_sub(self.positions[i], self.positions[i - 1]));
        }
        len
    }

    pub fn rest_total_length(&self) -> f32 {
        self.rest_lengths.iter().sum()
    }

    pub fn stretch_ratio(&self) -> f32 {
        let rest = self.rest_total_length();
        if rest < 1e-10 {
            return 1.0;
        }
        self.total_length() / rest
    }

    /// Apply gravity to all non-pinned nodes.
    pub fn apply_gravity(&mut self, gravity: [f32; 3], dt: f32) {
        let start = if self.pinned_root { 1 } else { 0 };
        for i in start..self.velocities.len() {
            self.velocities[i] = vec3_add(self.velocities[i], vec3_scale(gravity, dt));
        }
    }

    /// Simple Verlet-like position update.
    pub fn integrate(&mut self, dt: f32) {
        let start = if self.pinned_root { 1 } else { 0 };
        for i in start..self.positions.len() {
            self.positions[i] = vec3_add(self.positions[i], vec3_scale(self.velocities[i], dt));
        }
    }

    /// Enforce distance constraints between adjacent nodes.
    pub fn solve_constraints(&mut self, iterations: usize) {
        for _ in 0..iterations {
            for i in 0..self.segment_count() {
                let delta = vec3_sub(self.positions[i + 1], self.positions[i]);
                let dist = vec3_len(delta);
                if dist < 1e-10 {
                    continue;
                }
                let rest = self.rest_lengths[i];
                let correction = (dist - rest) / dist;
                let half = vec3_scale(delta, correction * 0.5);
                if !self.pinned_root || i > 0 {
                    self.positions[i] = vec3_add(self.positions[i], half);
                }
                self.positions[i + 1] = vec3_sub(self.positions[i + 1], half);
            }
        }
    }

    pub fn tip(&self) -> [f32; 3] {
        *self.positions.last().unwrap_or(&[0.0; 3])
    }

    pub fn root(&self) -> [f32; 3] {
        self.positions[0]
    }

    pub fn total_mass(&self) -> f32 {
        self.mass_per_node * self.node_count() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_straight() {
        let s = ElasticStrand::new_straight([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], 5, 0.1);
        assert_eq!(s.node_count(), 5);
        assert_eq!(s.segment_count(), 4);
    }

    #[test]
    fn test_total_length() {
        let s = ElasticStrand::new_straight([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], 5, 0.1);
        assert!((s.total_length() - 0.4).abs() < 1e-5);
    }

    #[test]
    fn test_rest_total_length() {
        let s = ElasticStrand::new_straight([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], 5, 0.1);
        assert!((s.rest_total_length() - 0.4).abs() < 1e-5);
    }

    #[test]
    fn test_stretch_ratio_at_rest() {
        let s = ElasticStrand::new_straight([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], 5, 0.1);
        assert!((s.stretch_ratio() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_root_and_tip() {
        let s = ElasticStrand::new_straight([1.0, 2.0, 3.0], [1.0, 0.0, 0.0], 3, 1.0);
        assert!((s.root()[0] - 1.0).abs() < 1e-5);
        assert!((s.tip()[0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_gravity() {
        let mut s = ElasticStrand::new_straight([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], 3, 0.1);
        s.apply_gravity([0.0, -9.81, 0.0], 0.01);
        assert!(s.velocities[1][1] < 0.0);
        // root should be unaffected when pinned
        assert!((s.velocities[0][1]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_integrate() {
        let mut s = ElasticStrand::new_straight([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], 3, 0.1);
        s.velocities[2] = [1.0, 0.0, 0.0];
        let old_tip = s.tip();
        s.integrate(0.01);
        assert!(s.tip()[0] > old_tip[0]);
    }

    #[test]
    fn test_solve_constraints() {
        let mut s = ElasticStrand::new_straight([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], 3, 0.1);
        s.positions[2] = [0.0, -0.5, 0.0]; // stretch it
        s.solve_constraints(5);
        let len = s.total_length();
        assert!((len - s.rest_total_length()).abs() < 0.05);
    }

    #[test]
    fn test_total_mass() {
        let s = ElasticStrand::new_straight([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], 10, 0.1);
        assert!((s.total_mass() - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_pinned_root_stays() {
        let mut s = ElasticStrand::new_straight([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], 3, 0.1);
        s.apply_gravity([0.0, -9.81, 0.0], 0.1);
        s.integrate(0.1);
        assert!((s.root()[0]).abs() < f32::EPSILON);
        assert!((s.root()[1]).abs() < f32::EPSILON);
    }
}
