// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Accumulates and combines multiple forces into a single net force.

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
#[derive(Debug, Clone)]
pub struct ForceEntry {
    pub force: [f32; 3],
    pub label: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForceCombiner {
    entries: Vec<ForceEntry>,
}

#[allow(dead_code)]
impl ForceCombiner {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, force: [f32; 3], label: &str) {
        self.entries.push(ForceEntry { force, label: label.to_string() });
    }

    pub fn add_gravity(&mut self, mass: f32, g: f32) {
        self.entries.push(ForceEntry {
            force: [0.0, -mass * g, 0.0],
            label: "gravity".to_string(),
        });
    }

    pub fn net_force(&self) -> [f32; 3] {
        let mut sum = [0.0f32; 3];
        for e in &self.entries {
            sum = vec3_add(sum, e.force);
        }
        sum
    }

    pub fn net_magnitude(&self) -> f32 {
        vec3_len(self.net_force())
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn get_by_label(&self, label: &str) -> Option<[f32; 3]> {
        self.entries.iter().find(|e| e.label == label).map(|e| e.force)
    }

    pub fn scale_all(&mut self, factor: f32) {
        for e in &mut self.entries {
            e.force = vec3_scale(e.force, factor);
        }
    }

    pub fn clamp_magnitude(&self, max_mag: f32) -> [f32; 3] {
        let net = self.net_force();
        let mag = vec3_len(net);
        if mag > max_mag && mag > 1e-12 {
            vec3_scale(net, max_mag / mag)
        } else {
            net
        }
    }

    pub fn entries(&self) -> &[ForceEntry] {
        &self.entries
    }
}

impl Default for ForceCombiner {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_force_empty() {
        let c = ForceCombiner::new();
        let f = c.net_force();
        assert!((f[0]).abs() < 1e-10);
    }

    #[test]
    fn test_add_and_net() {
        let mut c = ForceCombiner::new();
        c.add([1.0, 0.0, 0.0], "f1");
        c.add([0.0, 2.0, 0.0], "f2");
        let net = c.net_force();
        assert!((net[0] - 1.0).abs() < 1e-5);
        assert!((net[1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_gravity() {
        let mut c = ForceCombiner::new();
        c.add_gravity(2.0, 9.81);
        let net = c.net_force();
        assert!((net[1] + 19.62).abs() < 0.01);
    }

    #[test]
    fn test_count() {
        let mut c = ForceCombiner::new();
        c.add([1.0, 0.0, 0.0], "a");
        c.add([0.0, 1.0, 0.0], "b");
        assert_eq!(c.count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut c = ForceCombiner::new();
        c.add([1.0, 0.0, 0.0], "a");
        c.clear();
        assert_eq!(c.count(), 0);
    }

    #[test]
    fn test_get_by_label() {
        let mut c = ForceCombiner::new();
        c.add([5.0, 0.0, 0.0], "wind");
        let f = c.get_by_label("wind").expect("should succeed");
        assert!((f[0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_get_by_label_missing() {
        let c = ForceCombiner::new();
        assert!(c.get_by_label("none").is_none());
    }

    #[test]
    fn test_scale_all() {
        let mut c = ForceCombiner::new();
        c.add([2.0, 0.0, 0.0], "a");
        c.scale_all(3.0);
        let f = c.get_by_label("a").expect("should succeed");
        assert!((f[0] - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_clamp_magnitude() {
        let mut c = ForceCombiner::new();
        c.add([100.0, 0.0, 0.0], "big");
        let clamped = c.clamp_magnitude(5.0);
        assert!((vec3_len(clamped) - 5.0).abs() < 1e-3);
    }

    #[test]
    fn test_net_magnitude() {
        let mut c = ForceCombiner::new();
        c.add([3.0, 4.0, 0.0], "a");
        assert!((c.net_magnitude() - 5.0).abs() < 1e-4);
    }
}
