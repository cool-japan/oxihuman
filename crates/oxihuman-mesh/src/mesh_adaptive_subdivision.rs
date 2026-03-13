// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Adaptive subdivision based on curvature threshold.

/// Config for adaptive subdivision.
#[derive(Clone, Debug)]
pub struct AdaptSubdivConfig {
    /// Dihedral-angle threshold in radians; faces with larger dihedral get subdivided.
    pub angle_threshold_rad: f32,
    pub max_iterations: usize,
}

impl Default for AdaptSubdivConfig {
    fn default() -> Self {
        Self {
            angle_threshold_rad: std::f32::consts::FRAC_PI_4,
            max_iterations: 3,
        }
    }
}

/// Result of adaptive subdivision.
#[derive(Clone, Debug, Default)]
pub struct AdaptSubdivResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub subdivided_faces: usize,
}

fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = n.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-12);
    [n[0] / len, n[1] / len, n[2] / len]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter().zip(b.iter()).map(|(u, v)| u * v).sum()
}

/// Subdivide a single triangle at its midpoints.
fn subdivide_triangle(positions: &mut Vec<[f32; 3]>, ia: u32, ib: u32, ic: u32) -> [u32; 12] {
    let a = positions[ia as usize];
    let b = positions[ib as usize];
    let c = positions[ic as usize];
    let mab = [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ];
    let mbc = [
        (b[0] + c[0]) * 0.5,
        (b[1] + c[1]) * 0.5,
        (b[2] + c[2]) * 0.5,
    ];
    let mca = [
        (c[0] + a[0]) * 0.5,
        (c[1] + a[1]) * 0.5,
        (c[2] + a[2]) * 0.5,
    ];
    let imab = positions.len() as u32;
    positions.push(mab);
    let imbc = positions.len() as u32;
    positions.push(mbc);
    let imca = positions.len() as u32;
    positions.push(mca);
    // Four sub-triangles: (ia, imab, imca), (imab, ib, imbc), (imca, imbc, ic), (imab, imbc, imca)
    [
        ia, imab, imca, imab, ib, imbc, imca, imbc, ic, imab, imbc, imca,
    ]
}

/// Run one pass of adaptive subdivision.
pub fn adapt_subdivide_step(
    positions: &mut Vec<[f32; 3]>,
    indices: &[u32],
    angle_threshold_rad: f32,
) -> (Vec<u32>, usize) {
    let tri_count = indices.len() / 3;
    let mut new_indices: Vec<u32> = Vec::with_capacity(indices.len() * 4);
    let mut subdiv_count = 0;

    // Build a simple face-normal list for adjacent-face comparison
    let mut normals: Vec<[f32; 3]> = (0..tri_count)
        .map(|t| {
            let ia = indices[t * 3] as usize;
            let ib = indices[t * 3 + 1] as usize;
            let ic = indices[t * 3 + 2] as usize;
            face_normal(positions[ia], positions[ib], positions[ic])
        })
        .collect();

    for t in 0..tri_count {
        let ia = indices[t * 3];
        let ib = indices[t * 3 + 1];
        let ic = indices[t * 3 + 2];
        let n = normals[t];

        // Compare with all other face normals (simple brute-force for correctness)
        let needs_subdiv = normals.iter().enumerate().any(|(j, &m)| {
            if j == t {
                return false;
            }
            let cos_a = dot3(n, m).clamp(-1.0, 1.0);
            let angle = cos_a.acos();
            angle > angle_threshold_rad
        });

        if needs_subdiv {
            let tris = subdivide_triangle(positions, ia, ib, ic);
            new_indices.extend_from_slice(&tris);
            subdiv_count += 1;
        } else {
            new_indices.extend_from_slice(&[ia, ib, ic]);
        }

        // Update normals for new vertices (they inherit parent normal)
        while normals.len() < positions.len() {
            normals.push(n);
        }
    }
    (new_indices, subdiv_count)
}

/// Adaptively subdivide a mesh over multiple iterations.
pub fn adaptive_subdivision(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &AdaptSubdivConfig,
) -> AdaptSubdivResult {
    let mut pos = positions.to_vec();
    let mut idx = indices.to_vec();
    let mut total_subdiv = 0;

    for _ in 0..config.max_iterations {
        let (new_idx, n) = adapt_subdivide_step(&mut pos, &idx, config.angle_threshold_rad);
        idx = new_idx;
        total_subdiv += n;
        if n == 0 {
            break;
        }
    }

    AdaptSubdivResult {
        positions: pos,
        indices: idx,
        subdivided_faces: total_subdiv,
    }
}

/// Return vertex count from result.
pub fn adapt_subdiv_vertex_count(r: &AdaptSubdivResult) -> usize {
    r.positions.len()
}

/// Return triangle count from result.
pub fn adapt_subdiv_face_count(r: &AdaptSubdivResult) -> usize {
    r.indices.len() / 3
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        (pos, idx)
    }

    #[test]
    fn config_default_sane() {
        let c = AdaptSubdivConfig::default();
        assert!(c.angle_threshold_rad > 0.0);
        assert!(c.max_iterations > 0);
    }

    #[test]
    fn adaptive_subdivision_single_tri_no_neighbor() {
        /* Single triangle has no neighbors → no subdivision needed */
        let (pos, idx) = single_tri();
        let cfg = AdaptSubdivConfig {
            max_iterations: 1,
            ..Default::default()
        };
        let r = adaptive_subdivision(&pos, &idx, &cfg);
        assert_eq!(r.positions.len(), 3);
    }

    #[test]
    fn adapt_subdiv_face_count_consistent() {
        let (pos, idx) = single_tri();
        let cfg = AdaptSubdivConfig::default();
        let r = adaptive_subdivision(&pos, &idx, &cfg);
        assert_eq!(adapt_subdiv_face_count(&r) * 3, r.indices.len());
    }

    #[test]
    fn adapt_subdiv_vertex_count_gte_input() {
        let (pos, idx) = single_tri();
        let cfg = AdaptSubdivConfig::default();
        let r = adaptive_subdivision(&pos, &idx, &cfg);
        assert!(adapt_subdiv_vertex_count(&r) >= pos.len());
    }

    #[test]
    fn subdivide_triangle_adds_three_vertices() {
        let mut pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        subdivide_triangle(&mut pos, 0, 1, 2);
        assert_eq!(pos.len(), 6);
    }

    #[test]
    fn face_normal_z_axis() {
        let n = face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((n[2].abs() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn adaptive_subdivision_indices_valid() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        let cfg = AdaptSubdivConfig {
            angle_threshold_rad: 0.01,
            max_iterations: 1,
        };
        let r = adaptive_subdivision(&pos, &idx, &cfg);
        let n = r.positions.len() as u32;
        for &i in &r.indices {
            assert!(i < n);
        }
    }

    #[test]
    fn adaptive_subdivision_zero_iterations() {
        let (pos, idx) = single_tri();
        let cfg = AdaptSubdivConfig {
            max_iterations: 0,
            ..Default::default()
        };
        let r = adaptive_subdivision(&pos, &idx, &cfg);
        assert_eq!(r.positions.len(), pos.len());
    }
}
