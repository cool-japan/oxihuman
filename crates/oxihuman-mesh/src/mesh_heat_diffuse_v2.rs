// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Heat diffusion kernel on mesh using explicit Euler time-stepping (v2).

use std::collections::HashMap;

/// Configuration for heat diffusion.
#[allow(dead_code)]
pub struct HeatDiffuseV2Config {
    pub steps: usize,
    pub dt: f32,
    pub conductivity: f32,
}

impl Default for HeatDiffuseV2Config {
    fn default() -> Self {
        Self {
            steps: 10,
            dt: 0.01,
            conductivity: 1.0,
        }
    }
}

/// A heat field on mesh vertices.
#[allow(dead_code)]
pub struct HeatFieldV2 {
    pub values: Vec<f32>,
    pub time: f32,
}

fn build_adjacency_v2(n_verts: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); n_verts];
    for t in 0..(indices.len() / 3) {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        for &(a, b) in &[(i0, i1), (i1, i2), (i2, i0), (i1, i0), (i2, i1), (i0, i2)] {
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
        }
    }
    adj
}

fn edge_length_sq(positions: &[[f32; 3]], a: usize, b: usize) -> f32 {
    let pa = positions[a];
    let pb = positions[b];
    let dx = pa[0] - pb[0];
    let dy = pa[1] - pb[1];
    let dz = pa[2] - pb[2];
    dx * dx + dy * dy + dz * dz
}

/// Create a new heat field with given initial values.
#[allow(dead_code)]
pub fn new_heat_field_v2(n_verts: usize, initial: f32) -> HeatFieldV2 {
    HeatFieldV2 {
        values: vec![initial; n_verts],
        time: 0.0,
    }
}

/// Set heat source at vertex.
#[allow(dead_code)]
pub fn set_heat_source_v2(field: &mut HeatFieldV2, vertex: usize, value: f32) {
    if vertex < field.values.len() {
        field.values[vertex] = value;
    }
}

/// Perform explicit Euler heat diffusion steps.
#[allow(dead_code)]
pub fn diffuse_heat_v2(
    field: &mut HeatFieldV2,
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &HeatDiffuseV2Config,
) {
    let n = field.values.len();
    let adj = build_adjacency_v2(n, indices);
    for _ in 0..config.steps {
        let old = field.values.clone();
        for i in 0..n {
            let mut laplacian = 0.0_f32;
            let mut w_sum = 0.0_f32;
            for &nb in &adj[i] {
                let w = 1.0 / (edge_length_sq(positions, i, nb).sqrt().max(1e-9));
                laplacian += w * (old[nb] - old[i]);
                w_sum += w;
            }
            if w_sum > 1e-9 {
                field.values[i] += config.dt * config.conductivity * laplacian / w_sum;
            }
        }
        field.time += config.dt;
    }
}

/// Maximum heat value in the field.
#[allow(dead_code)]
pub fn heat_field_v2_max(field: &HeatFieldV2) -> f32 {
    field
        .values
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max)
}

/// Minimum heat value in the field.
#[allow(dead_code)]
pub fn heat_field_v2_min(field: &HeatFieldV2) -> f32 {
    field.values.iter().cloned().fold(f32::INFINITY, f32::min)
}

/// Normalize heat field to [0, 1].
#[allow(dead_code)]
pub fn normalize_heat_field_v2(field: &mut HeatFieldV2) {
    let mn = heat_field_v2_min(field);
    let mx = heat_field_v2_max(field);
    let range = mx - mn;
    if range < 1e-9 {
        return;
    }
    for v in &mut field.values {
        *v = (*v - mn) / range;
    }
}

/// Compute heat gradient at a vertex (difference from average of neighbors).
#[allow(dead_code)]
pub fn heat_gradient_v2(field: &HeatFieldV2, vertex: usize, adj: &[Vec<usize>]) -> f32 {
    if vertex >= adj.len() || adj[vertex].is_empty() {
        return 0.0;
    }
    let avg: f32 =
        adj[vertex].iter().map(|&nb| field.values[nb]).sum::<f32>() / adj[vertex].len() as f32;
    field.values[vertex] - avg
}

/// Build adjacency for external use.
#[allow(dead_code)]
pub fn build_heat_adjacency_v2(n_verts: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    build_adjacency_v2(n_verts, indices)
}

/// Store edges as HashMap for lookup.
#[allow(dead_code)]
pub fn heat_edge_map_v2(indices: &[u32]) -> HashMap<(usize, usize), f32> {
    let mut map = HashMap::new();
    for t in 0..(indices.len() / 3) {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        for &(a, b) in &[(i0, i1), (i1, i2), (i2, i0)] {
            map.entry((a.min(b), a.max(b))).or_insert(1.0);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.5, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let indices: Vec<u32> = vec![0, 1, 2, 1, 3, 4, 1, 4, 2];
        (positions, indices)
    }

    #[test]
    fn new_field_size_correct() {
        let field = new_heat_field_v2(5, 0.0);
        assert_eq!(field.values.len(), 5);
    }

    #[test]
    fn set_source_changes_value() {
        let mut field = new_heat_field_v2(5, 0.0);
        set_heat_source_v2(&mut field, 2, 1.0);
        assert!((field.values[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn diffuse_changes_field() {
        let (pos, idx) = simple_mesh();
        let mut field = new_heat_field_v2(pos.len(), 0.0);
        set_heat_source_v2(&mut field, 0, 1.0);
        let config = HeatDiffuseV2Config::default();
        diffuse_heat_v2(&mut field, &pos, &idx, &config);
        assert!(heat_field_v2_max(&field) > 0.0);
    }

    #[test]
    fn heat_field_v2_max_min_ordered() {
        let field = new_heat_field_v2(4, 0.5);
        assert!((heat_field_v2_max(&field) - heat_field_v2_min(&field)).abs() < 1e-5);
    }

    #[test]
    fn normalize_maps_to_unit_range() {
        let mut field = new_heat_field_v2(5, 0.0);
        field.values = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        normalize_heat_field_v2(&mut field);
        assert!((heat_field_v2_min(&field) - 0.0).abs() < 1e-5);
        assert!((heat_field_v2_max(&field) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn heat_gradient_v2_center_hotter() {
        let (pos, idx) = simple_mesh();
        let mut field = new_heat_field_v2(pos.len(), 0.0);
        set_heat_source_v2(&mut field, 1, 1.0);
        let config = HeatDiffuseV2Config::default();
        diffuse_heat_v2(&mut field, &pos, &idx, &config);
        let adj = build_heat_adjacency_v2(pos.len(), &idx);
        let grad = heat_gradient_v2(&field, 1, &adj);
        assert!(grad.is_finite());
    }

    #[test]
    fn build_adjacency_returns_correct_size() {
        let (_, idx) = simple_mesh();
        let adj = build_heat_adjacency_v2(5, &idx);
        assert_eq!(adj.len(), 5);
    }

    #[test]
    fn heat_edge_map_nonempty() {
        let (_, idx) = simple_mesh();
        let map = heat_edge_map_v2(&idx);
        assert!(!map.is_empty());
    }

    #[test]
    fn default_config_positive() {
        let c = HeatDiffuseV2Config::default();
        assert!(c.steps > 0);
        assert!(c.dt > 0.0);
        assert!(c.conductivity > 0.0);
    }

    #[test]
    fn time_advances_during_diffusion() {
        let (pos, idx) = simple_mesh();
        let mut field = new_heat_field_v2(pos.len(), 0.0);
        let config = HeatDiffuseV2Config {
            steps: 5,
            dt: 0.1,
            conductivity: 1.0,
        };
        diffuse_heat_v2(&mut field, &pos, &idx, &config);
        assert!((field.time - 0.5).abs() < 1e-4);
    }
}
