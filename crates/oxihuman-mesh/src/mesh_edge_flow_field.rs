// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
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

/// Discrete curl magnitude at a 1-ring via Stokes' theorem.
///
/// `ring` is the ordered boundary loop as `(vertex_index, position)` pairs, where
/// `vertex_index` indexes into `f.flow_vectors`. The circulation
/// `Γ = ∮ F·dl` is evaluated with the trapezoidal rule over the closed loop,
/// the enclosed area `A` via Newell's method, and the curl component normal to
/// the loop's plane is `Γ / A`. Returns `|Γ / A|` (0 for degenerate rings).
pub fn edge_flow_field_curl_magnitude(
    f: &EdgeFlowField,
    center: [f32; 3],
    ring: &[(usize, [f32; 3])],
) -> f32 {
    let _ = center; // reserved for future area-weighting; circulation is center-independent
    let k = ring.len();
    if k < 3 {
        return 0.0;
    }

    // Enclosed area via Newell's method (|Newell normal| = 2 * area).
    let mut nx = 0.0f32;
    let mut ny = 0.0f32;
    let mut nz = 0.0f32;
    #[allow(clippy::needless_range_loop)]
    for i in 0..k {
        let a = ring[i].1;
        let b = ring[(i + 1) % k].1;
        nx += (a[1] - b[1]) * (a[2] + b[2]);
        ny += (a[2] - b[2]) * (a[0] + b[0]);
        nz += (a[0] - b[0]) * (a[1] + b[1]);
    }
    let area = 0.5 * (nx * nx + ny * ny + nz * nz).sqrt();
    if area < 1e-12 {
        return 0.0;
    }

    // Circulation Γ = Σ (F_i + F_{i+1})/2 · (p_{i+1} − p_i), closing the loop.
    let mut circ = 0.0f32;
    for i in 0..k {
        let (idx_i, p_i) = ring[i];
        let (idx_j, p_j) = ring[(i + 1) % k];
        let fi = f.flow_vectors.get(idx_i).copied().unwrap_or([0.0; 3]);
        let fj = f.flow_vectors.get(idx_j).copied().unwrap_or([0.0; 3]);
        let favg = [
            0.5 * (fi[0] + fj[0]),
            0.5 * (fi[1] + fj[1]),
            0.5 * (fi[2] + fj[2]),
        ];
        let edge = [p_j[0] - p_i[0], p_j[1] - p_i[1], p_j[2] - p_i[2]];
        circ += favg[0] * edge[0] + favg[1] * edge[1] + favg[2] * edge[2];
    }

    (circ / area).abs()
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
    fn test_curl_empty_ring() {
        /* empty ring => 0 */
        let f = new_edge_flow_field(2);
        assert!((edge_flow_field_curl_magnitude(&f, [0.0; 3], &[])).abs() < 1e-6);
    }

    #[test]
    fn test_curl_rotational_field() {
        // Square corners CCW (normal +z): area = 4.
        let corners = [
            [1.0f32, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
        ];
        let mut f = new_edge_flow_field(4);
        // Rotational field F(x,y) = (-y, x, 0).
        for (i, c) in corners.iter().enumerate() {
            edge_flow_field_set(&mut f, i, [-c[1], c[0], 0.0]);
        }
        let ring: Vec<(usize, [f32; 3])> =
            corners.iter().enumerate().map(|(i, &p)| (i, p)).collect();
        let curl = edge_flow_field_curl_magnitude(&f, [0.0; 3], &ring);
        assert!((curl - 2.0).abs() < 1e-4, "expected curl 2.0, got {curl}");
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
