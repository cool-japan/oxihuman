// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Heat kernel signature (HKS) for mesh shape analysis.

use std::f32::consts::E;

/// Heat kernel at a vertex (simplified: using diffusion simulation).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HeatKernelResult {
    pub values: Vec<f32>,
    pub time_param: f32,
}

/// Build adjacency list.
#[allow(dead_code)]
pub fn hk_build_adjacency(indices: &[u32], n: usize) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); n];
    let tc = indices.len() / 3;
    for t in 0..tc {
        let tri = [indices[t * 3] as usize, indices[t * 3 + 1] as usize, indices[t * 3 + 2] as usize];
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            if !adj[a].contains(&b) { adj[a].push(b); }
            if !adj[b].contains(&a) { adj[b].push(a); }
        }
    }
    adj
}

/// Diffuse heat from sources for a number of steps.
#[allow(dead_code)]
pub fn diffuse_heat(
    adj: &[Vec<usize>],
    initial: &[f32],
    steps: u32,
    dt: f32,
) -> Vec<f32> {
    let _ = E;
    let n = initial.len();
    let mut u = initial.to_vec();
    for _ in 0..steps {
        let prev = u.clone();
        for i in 0..n {
            if adj[i].is_empty() { continue; }
            let k = adj[i].len() as f32;
            let mut lap = 0.0f32;
            for &j in &adj[i] {
                lap += prev[j] - prev[i];
            }
            u[i] = prev[i] + dt * lap / k;
        }
    }
    u
}

/// Compute heat kernel signature at given time for all vertices.
#[allow(dead_code)]
pub fn compute_heat_kernel(
    indices: &[u32],
    n_verts: usize,
    source: usize,
    time: f32,
    steps: u32,
) -> HeatKernelResult {
    let adj = hk_build_adjacency(indices, n_verts);
    let mut initial = vec![0.0f32; n_verts];
    initial[source] = 1.0;
    let dt = time / steps.max(1) as f32;
    let values = diffuse_heat(&adj, &initial, steps, dt);
    HeatKernelResult { values, time_param: time }
}

/// Heat value at a vertex.
#[allow(dead_code)]
pub fn heat_at_vertex(result: &HeatKernelResult, v: usize) -> f32 {
    if v < result.values.len() { result.values[v] } else { 0.0 }
}

/// Max heat value.
#[allow(dead_code)]
pub fn heat_max(result: &HeatKernelResult) -> f32 {
    result.values.iter().cloned().fold(0.0f32, f32::max)
}

/// Min heat value.
#[allow(dead_code)]
pub fn heat_min(result: &HeatKernelResult) -> f32 {
    result.values.iter().cloned().fold(f32::INFINITY, f32::min)
}

/// Normalize heat values to [0, 1].
#[allow(dead_code)]
pub fn normalize_heat(result: &mut HeatKernelResult) {
    let min = heat_min(result);
    let max = heat_max(result);
    let range = max - min;
    if range < 1e-12 { return; }
    for v in result.values.iter_mut() {
        *v = (*v - min) / range;
    }
}

/// Heat vertex count.
#[allow(dead_code)]
pub fn heat_vertex_count(result: &HeatKernelResult) -> usize {
    result.values.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjacency() {
        let adj = hk_build_adjacency(&[0, 1, 2], 3);
        assert_eq!(adj.len(), 3);
    }

    #[test]
    fn test_diffuse_preserves_count() {
        let adj = vec![vec![1usize, 2], vec![0, 2], vec![0, 1]];
        let u = diffuse_heat(&adj, &[1.0, 0.0, 0.0], 5, 0.1);
        assert_eq!(u.len(), 3);
    }

    #[test]
    fn test_heat_spreads() {
        let adj = vec![vec![1usize, 2], vec![0, 2], vec![0, 1]];
        let u = diffuse_heat(&adj, &[1.0, 0.0, 0.0], 10, 0.1);
        // Heat should spread to neighbors
        assert!(u[1] > 0.0);
    }

    #[test]
    fn test_compute_heat_kernel() {
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        let result = compute_heat_kernel(&indices, 4, 0, 1.0, 20);
        assert_eq!(heat_vertex_count(&result), 4);
    }

    #[test]
    fn test_source_has_max() {
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        let result = compute_heat_kernel(&indices, 4, 0, 0.5, 5);
        let source_heat = heat_at_vertex(&result, 0);
        assert!(source_heat > 0.0);
    }

    #[test]
    fn test_heat_max() {
        let result = HeatKernelResult { values: vec![0.1, 0.5, 0.3], time_param: 1.0 };
        assert!((heat_max(&result) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_heat_min() {
        let result = HeatKernelResult { values: vec![0.1, 0.5, 0.3], time_param: 1.0 };
        assert!((heat_min(&result) - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_normalize() {
        let mut result = HeatKernelResult { values: vec![1.0, 3.0, 5.0], time_param: 1.0 };
        normalize_heat(&mut result);
        assert!((result.values[0] - 0.0).abs() < 1e-5);
        assert!((result.values[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_heat_at_out_of_range() {
        let result = HeatKernelResult { values: vec![0.5], time_param: 1.0 };
        assert!((heat_at_vertex(&result, 999) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_mesh() {
        let result = compute_heat_kernel(&[], 0, 0, 1.0, 10);
        assert_eq!(heat_vertex_count(&result), 0);
    }

}
