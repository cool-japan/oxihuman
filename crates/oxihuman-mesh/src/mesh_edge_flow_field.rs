// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Per-vertex edge flow field for mesh analysis.
pub struct EdgeFlowField {
    pub flow_vectors: Vec<[f32; 3]>,
    pub vertex_count: usize,
}

pub fn new_edge_flow_field(n: usize) -> EdgeFlowField {
    EdgeFlowField {
        flow_vectors: vec![[0.0; 3]; n],
        vertex_count: n,
    }
}

pub fn edge_flow_field_set(f: &mut EdgeFlowField, i: usize, v: [f32; 3]) {
    f.flow_vectors[i] = v;
}

pub fn edge_flow_field_get(f: &EdgeFlowField, i: usize) -> [f32; 3] {
    f.flow_vectors[i]
}

pub fn edge_flow_field_divergence(f: &EdgeFlowField, i: usize, neighbors: &[usize]) -> f32 {
    let fi = f.flow_vectors[i];
    let mut div = 0.0_f32;
    for &j in neighbors {
        let fj = f.flow_vectors[j];
        div += fi[0] - fj[0] + fi[1] - fj[1] + fi[2] - fj[2];
    }
    div
}

/// Stub: returns 0.
pub fn edge_flow_field_curl_magnitude(
    _f: &EdgeFlowField,
    _center: [f32; 3],
    _ring: &[[f32; 3]],
) -> f32 {
    0.0
}

pub fn edge_flow_field_mean_speed(f: &EdgeFlowField) -> f32 {
    if f.flow_vectors.is_empty() {
        return 0.0;
    }
    let total: f32 = f
        .flow_vectors
        .iter()
        .map(|v| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt())
        .sum();
    total / f.flow_vectors.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_edge_flow_field() {
        /* zero vectors */
        let f = new_edge_flow_field(4);
        assert_eq!(f.vertex_count, 4);
        assert_eq!(edge_flow_field_get(&f, 0), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_set_get() {
        /* round-trip */
        let mut f = new_edge_flow_field(3);
        edge_flow_field_set(&mut f, 1, [1.0, 0.0, 0.0]);
        assert_eq!(edge_flow_field_get(&f, 1), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_mean_speed_uniform() {
        /* all unit x => mean = 1 */
        let mut f = new_edge_flow_field(3);
        for i in 0..3 {
            edge_flow_field_set(&mut f, i, [1.0, 0.0, 0.0]);
        }
        assert!((edge_flow_field_mean_speed(&f) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_curl_stub() {
        /* stub always 0 */
        let f = new_edge_flow_field(2);
        assert!((edge_flow_field_curl_magnitude(&f, [0.0; 3], &[])).abs() < 1e-6);
    }

    #[test]
    fn test_divergence_no_neighbors() {
        /* no neighbors => 0 */
        let f = new_edge_flow_field(3);
        assert!((edge_flow_field_divergence(&f, 0, &[])).abs() < 1e-6);
    }

    #[test]
    fn test_mean_speed_empty() {
        /* empty => 0 */
        let f = new_edge_flow_field(0);
        assert!((edge_flow_field_mean_speed(&f)).abs() < 1e-6);
    }
}
