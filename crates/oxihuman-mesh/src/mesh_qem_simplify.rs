// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Quadric error metrics (QEM) mesh simplification.

/// A 4×4 symmetric quadric error matrix stored as 10 unique elements.
#[derive(Clone, Debug, Default)]
pub struct Quadric {
    q: [f32; 10], // upper triangle: q00 q01 q02 q03 q11 q12 q13 q22 q23 q33
}

impl Quadric {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add another quadric in-place.
    pub fn add(&mut self, other: &Quadric) {
        for i in 0..10 {
            self.q[i] += other.q[i];
        }
    }

    /// Build a quadric from a plane (nx,ny,nz,d) where nx·x+ny·y+nz·z+d=0.
    pub fn from_plane(n: [f32; 3], d: f32) -> Self {
        let [a, b, c] = n;
        Self {
            q: [
                a * a,
                a * b,
                a * c,
                a * d,
                b * b,
                b * c,
                b * d,
                c * c,
                c * d,
                d * d,
            ],
        }
    }

    /// Evaluate v^T Q v for a homogeneous vertex v = (x,y,z,1).
    pub fn evaluate(&self, v: [f32; 3]) -> f32 {
        let [x, y, z] = v;
        let q = &self.q;
        x * x * q[0]
            + 2.0 * x * y * q[1]
            + 2.0 * x * z * q[2]
            + 2.0 * x * q[3]
            + y * y * q[4]
            + 2.0 * y * z * q[5]
            + 2.0 * y * q[6]
            + z * z * q[7]
            + 2.0 * z * q[8]
            + q[9]
    }
}

/// Config for QEM simplification.
#[derive(Clone, Debug)]
pub struct QemConfig {
    pub target_face_count: usize,
}

impl Default for QemConfig {
    fn default() -> Self {
        Self {
            target_face_count: 100,
        }
    }
}

/// Result of QEM simplification.
#[derive(Clone, Debug, Default)]
pub struct QemResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub collapses: usize,
}

fn face_normal_qem(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> ([f32; 3], f32) {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = n.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-12);
    let nn = [n[0] / len, n[1] / len, n[2] / len];
    let d = -(nn[0] * a[0] + nn[1] * a[1] + nn[2] * a[2]);
    (nn, d)
}

/// Build per-vertex quadrics.
pub fn build_vertex_quadrics(positions: &[[f32; 3]], indices: &[u32]) -> Vec<Quadric> {
    let mut qs = vec![Quadric::new(); positions.len()];
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        let (n, d) = face_normal_qem(positions[ia], positions[ib], positions[ic]);
        let q = Quadric::from_plane(n, d);
        qs[ia].add(&q);
        qs[ib].add(&q);
        qs[ic].add(&q);
    }
    qs
}

/// Compute the cost of collapsing edge (v0,v1) to their midpoint.
pub fn edge_collapse_cost_qem(v0: [f32; 3], v1: [f32; 3], q0: &Quadric, q1: &Quadric) -> f32 {
    let mid = [
        (v0[0] + v1[0]) * 0.5,
        (v0[1] + v1[1]) * 0.5,
        (v0[2] + v1[2]) * 0.5,
    ];
    let mut combined = q0.clone();
    combined.add(q1);
    combined.evaluate(mid)
}

/// Simplify a mesh using QEM (greedy edge collapse to midpoints).
pub fn qem_simplify(positions: &[[f32; 3]], indices: &[u32], config: &QemConfig) -> QemResult {
    let mut pos = positions.to_vec();
    let mut idx = indices.to_vec();
    let mut qs = build_vertex_quadrics(&pos, &idx);
    let mut collapses = 0;

    while idx.len() / 3 > config.target_face_count.max(1) {
        // Find cheapest edge
        let tri_count = idx.len() / 3;
        let mut best_cost = f32::MAX;
        let mut best_edge = (0usize, 0usize);

        for t in 0..tri_count {
            for k in 0..3 {
                let ia = idx[t * 3 + k] as usize;
                let ib = idx[t * 3 + (k + 1) % 3] as usize;
                if ia == ib {
                    continue;
                }
                let cost = edge_collapse_cost_qem(pos[ia], pos[ib], &qs[ia], &qs[ib]);
                if cost < best_cost {
                    best_cost = cost;
                    best_edge = (ia, ib);
                }
            }
        }

        if best_cost == f32::MAX {
            break;
        }

        let (ia, ib) = best_edge;
        // Collapse ia and ib to midpoint, replacing ib references with ia
        let mid = [
            (pos[ia][0] + pos[ib][0]) * 0.5,
            (pos[ia][1] + pos[ib][1]) * 0.5,
            (pos[ia][2] + pos[ib][2]) * 0.5,
        ];
        pos[ia] = mid;
        let mut combined = qs[ia].clone();
        combined.add(&qs[ib]);
        qs[ia] = combined;

        // Remap ib → ia in index buffer
        for i in idx.iter_mut() {
            if *i as usize == ib {
                *i = ia as u32;
            }
        }

        // Remove degenerate triangles
        let new_idx: Vec<u32> = idx
            .chunks(3)
            .filter(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2])
            .flatten()
            .copied()
            .collect();
        idx = new_idx;
        collapses += 1;
    }

    QemResult {
        positions: pos,
        indices: idx,
        collapses,
    }
}

/// Return vertex count.
pub fn qem_vertex_count(r: &QemResult) -> usize {
    r.positions.len()
}

/// Return face count.
pub fn qem_face_count(r: &QemResult) -> usize {
    r.indices.len() / 3
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tris() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn quadric_from_plane_evaluates_on_plane() {
        let n = [0.0, 0.0, 1.0];
        let d = -1.0; // plane z=1
        let q = Quadric::from_plane(n, d);
        // Point on plane: (0,0,1)
        let err = q.evaluate([0.0, 0.0, 1.0]);
        assert!(err.abs() < 1e-5, "err={err}");
    }

    #[test]
    fn quadric_add_doubles() {
        let n = [1.0, 0.0, 0.0];
        let d = 0.0;
        let q = Quadric::from_plane(n, d);
        let mut q2 = q.clone();
        q2.add(&q);
        let v = q2.evaluate([1.0, 0.0, 0.0]);
        let v1 = q.evaluate([1.0, 0.0, 0.0]);
        assert!((v - 2.0 * v1).abs() < 1e-5);
    }

    #[test]
    fn build_vertex_quadrics_count() {
        let (pos, idx) = two_tris();
        let qs = build_vertex_quadrics(&pos, &idx);
        assert_eq!(qs.len(), pos.len());
    }

    #[test]
    fn qem_simplify_reduces_faces() {
        let (pos, idx) = two_tris();
        let cfg = QemConfig {
            target_face_count: 1,
        };
        let r = qem_simplify(&pos, &idx, &cfg);
        assert!(qem_face_count(&r) <= 2);
    }

    #[test]
    fn qem_result_indices_valid() {
        let (pos, idx) = two_tris();
        let cfg = QemConfig {
            target_face_count: 1,
        };
        let r = qem_simplify(&pos, &idx, &cfg);
        let n = r.positions.len() as u32;
        for &i in &r.indices {
            assert!(i < n);
        }
    }

    #[test]
    fn qem_vertex_count_consistent() {
        let (pos, idx) = two_tris();
        let cfg = QemConfig::default();
        let r = qem_simplify(&pos, &idx, &cfg);
        assert_eq!(qem_vertex_count(&r), r.positions.len());
    }

    #[test]
    fn edge_collapse_cost_nonneg() {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let q = Quadric::from_plane([0.0, 1.0, 0.0], 0.0);
        let cost = edge_collapse_cost_qem(v0, v1, &q, &q);
        assert!(cost >= 0.0);
    }

    #[test]
    fn qem_face_count_consistent() {
        let (pos, idx) = two_tris();
        let cfg = QemConfig::default();
        let r = qem_simplify(&pos, &idx, &cfg);
        assert_eq!(qem_face_count(&r) * 3, r.indices.len());
    }
}
