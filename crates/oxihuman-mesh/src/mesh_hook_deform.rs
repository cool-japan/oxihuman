// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hook point deform modifier.

/// A single hook that pulls vertices toward/from a target point.
#[derive(Debug, Clone)]
pub struct Hook {
    pub id: u32,
    pub position: [f32; 3],
    pub falloff_radius: f32,
    pub strength: f32,
    pub vertex_indices: Vec<usize>,
}

impl Hook {
    pub fn new(id: u32, position: [f32; 3], falloff_radius: f32) -> Self {
        Self { id, position, falloff_radius, strength: 1.0, vertex_indices: Vec::new() }
    }

    pub fn with_strength(mut self, s: f32) -> Self {
        self.strength = s;
        self
    }

    pub fn add_vertex(mut self, idx: usize) -> Self {
        self.vertex_indices.push(idx);
        self
    }
}

/// Compute the influence weight of a hook on a vertex based on distance.
pub fn hook_influence(hook: &Hook, vertex_pos: [f32; 3]) -> f32 {
    let dx = vertex_pos[0] - hook.position[0];
    let dy = vertex_pos[1] - hook.position[1];
    let dz = vertex_pos[2] - hook.position[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    if hook.falloff_radius <= 0.0 {
        return 1.0;
    }
    let t = (dist / hook.falloff_radius).clamp(0.0, 1.0);
    /* smooth falloff: 1 - smoothstep */
    let s = t * t * (3.0 - 2.0 * t);
    (1.0 - s) * hook.strength
}

/// Apply a hook displacement to listed vertices.
pub fn apply_hook(positions: &mut [[f32; 3]], hook: &Hook, delta: [f32; 3]) {
    for &idx in &hook.vertex_indices {
        if idx >= positions.len() {
            continue;
        }
        let w = hook_influence(hook, positions[idx]);
        positions[idx][0] += delta[0] * w;
        positions[idx][1] += delta[1] * w;
        positions[idx][2] += delta[2] * w;
    }
}

/// Apply multiple hooks to a set of positions.
pub fn apply_hooks(positions: &mut [[f32; 3]], hooks: &[Hook], deltas: &[[f32; 3]]) {
    for (hook, delta) in hooks.iter().zip(deltas.iter()) {
        apply_hook(positions, hook, *delta);
    }
}

/// Count total vertices across all hooks.
pub fn total_hook_vertices(hooks: &[Hook]) -> usize {
    hooks.iter().map(|h| h.vertex_indices.len()).sum()
}

/// Validate that all hook vertex indices are within bounds.
pub fn validate_hooks(hooks: &[Hook], vertex_count: usize) -> bool {
    hooks.iter().all(|h| h.vertex_indices.iter().all(|&i| i < vertex_count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_new() {
        let h = Hook::new(1, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(h.id, 1);
        assert!((h.strength - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_hook_with_strength() {
        let h = Hook::new(1, [0.0; 3], 1.0).with_strength(0.5);
        assert!((h.strength - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_hook_influence_at_center() {
        let h = Hook::new(1, [0.0; 3], 1.0);
        let w = hook_influence(&h, [0.0, 0.0, 0.0]);
        assert!((w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_hook_influence_beyond_radius() {
        let h = Hook::new(1, [0.0; 3], 1.0);
        let w = hook_influence(&h, [2.0, 0.0, 0.0]);
        assert!(w.abs() < 1e-5);
    }

    #[test]
    fn test_hook_influence_zero_radius() {
        let h = Hook::new(1, [0.0; 3], 0.0);
        let w = hook_influence(&h, [100.0, 0.0, 0.0]);
        assert!((w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_hook_moves_vertex() {
        let mut pos = vec![[0.0_f32, 0.0, 0.0]];
        let h = Hook::new(1, [0.0; 3], 0.0).add_vertex(0);
        apply_hook(&mut pos, &h, [1.0, 0.0, 0.0]);
        assert!((pos[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_hook_out_of_bounds_ignored() {
        let mut pos = vec![[0.0_f32; 3]];
        let h = Hook { vertex_indices: vec![5], ..Hook::new(1, [0.0; 3], 1.0) };
        apply_hook(&mut pos, &h, [1.0, 0.0, 0.0]);
        assert!(pos[0][0].abs() < 1e-5);
    }

    #[test]
    fn test_total_hook_vertices() {
        let hooks = vec![
            Hook::new(1, [0.0; 3], 1.0).add_vertex(0).add_vertex(1),
            Hook::new(2, [1.0; 3], 1.0).add_vertex(2),
        ];
        assert_eq!(total_hook_vertices(&hooks), 3);
    }

    #[test]
    fn test_validate_hooks_valid() {
        let hooks = vec![Hook::new(1, [0.0; 3], 1.0).add_vertex(0)];
        assert!(validate_hooks(&hooks, 5));
    }

    #[test]
    fn test_validate_hooks_out_of_bounds() {
        let hooks = vec![Hook::new(1, [0.0; 3], 1.0).add_vertex(10)];
        assert!(!validate_hooks(&hooks, 5));
    }
}
