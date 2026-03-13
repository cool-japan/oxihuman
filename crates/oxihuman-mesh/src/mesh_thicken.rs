// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Thicken a shell mesh by offsetting along per-vertex normals.

/// Result of a thicken operation.
#[allow(dead_code)]
pub struct ThickenResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub thickness: f32,
}

/// Thicken a mesh by duplicating it and offsetting the inner shell by `thickness`.
/// The original faces form the outer shell; the duplicated (flipped) faces form the inner shell.
#[allow(dead_code)]
pub fn thicken_mesh(positions: &[[f32; 3]], indices: &[u32], thickness: f32) -> ThickenResult {
    let normals = compute_vertex_normals(positions, indices);
    let n = positions.len();
    let mut new_positions = Vec::with_capacity(n * 2);
    // outer shell (original)
    new_positions.extend_from_slice(positions);
    // inner shell (offset inward)
    for (i, &p) in positions.iter().enumerate() {
        let nrm = normals[i];
        new_positions.push([
            p[0] - nrm[0] * thickness,
            p[1] - nrm[1] * thickness,
            p[2] - nrm[2] * thickness,
        ]);
    }
    let mut new_indices: Vec<u32> = Vec::with_capacity(indices.len() * 2);
    // outer shell
    new_indices.extend_from_slice(indices);
    // inner shell (flip winding)
    let offset = n as u32;
    for tri in indices.chunks_exact(3) {
        new_indices.push(tri[2] + offset);
        new_indices.push(tri[1] + offset);
        new_indices.push(tri[0] + offset);
    }
    ThickenResult {
        positions: new_positions,
        indices: new_indices,
        thickness,
    }
}

/// Compute average per-vertex normals from face normals.
#[allow(dead_code)]
pub fn compute_vertex_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut acc = vec![[0.0f32; 3]; n];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let pa = positions[a];
        let pb = positions[b];
        let pc = positions[c];
        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let n3 = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        for &i in &[a, b, c] {
            acc[i][0] += n3[0];
            acc[i][1] += n3[1];
            acc[i][2] += n3[2];
        }
    }
    acc.iter()
        .map(|&v| {
            let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            if l < 1e-8 {
                [0.0, 1.0, 0.0]
            } else {
                [v[0] / l, v[1] / l, v[2] / l]
            }
        })
        .collect()
}

/// Vertex count after thickening.
#[allow(dead_code)]
pub fn thicken_vertex_count(original_count: usize) -> usize {
    original_count * 2
}

/// Triangle count after thickening (both shells).
#[allow(dead_code)]
pub fn thicken_triangle_count(original_triangle_count: usize) -> usize {
    original_triangle_count * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        (pos, idx)
    }

    #[test]
    fn thicken_doubles_vertices() {
        let (pos, idx) = flat_tri();
        let res = thicken_mesh(&pos, &idx, 0.1);
        assert_eq!(res.positions.len(), 6);
    }

    #[test]
    fn thicken_doubles_triangles() {
        let (pos, idx) = flat_tri();
        let res = thicken_mesh(&pos, &idx, 0.1);
        assert_eq!(res.indices.len() / 3, 2);
    }

    #[test]
    fn thickness_stored() {
        let (pos, idx) = flat_tri();
        let res = thicken_mesh(&pos, &idx, 0.5);
        assert!((res.thickness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn outer_shell_unchanged() {
        let (pos, idx) = flat_tri();
        let res = thicken_mesh(&pos, &idx, 0.2);
        for (i, &p) in pos.iter().enumerate() {
            assert_eq!(res.positions[i], p);
        }
    }

    #[test]
    fn inner_shell_offset() {
        let (pos, idx) = flat_tri();
        let thickness = 0.3;
        let res = thicken_mesh(&pos, &idx, thickness);
        let n = pos.len();
        for i in 0..n {
            let outer = res.positions[i];
            let inner = res.positions[i + n];
            let dy = outer[1] - inner[1];
            assert!(dy >= 0.0 || dy.abs() < 0.01);
        }
    }

    #[test]
    fn compute_vertex_normals_flat_mesh() {
        let (pos, idx) = flat_tri();
        let norms = compute_vertex_normals(&pos, &idx);
        assert_eq!(norms.len(), 3);
        for n in &norms {
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((l - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn vertex_count_formula() {
        assert_eq!(thicken_vertex_count(10), 20);
    }

    #[test]
    fn triangle_count_formula() {
        assert_eq!(thicken_triangle_count(4), 8);
    }

    #[test]
    fn inner_winding_flipped() {
        let (pos, idx) = flat_tri();
        let res = thicken_mesh(&pos, &idx, 0.1);
        let n = pos.len() as u32;
        let outer = [res.indices[0], res.indices[1], res.indices[2]];
        let inner = [res.indices[3], res.indices[4], res.indices[5]];
        assert_eq!(inner[0], outer[2] + n);
        assert_eq!(inner[2], outer[0] + n);
    }

    #[test]
    fn zero_thickness() {
        let (pos, idx) = flat_tri();
        let res = thicken_mesh(&pos, &idx, 0.0);
        let n = pos.len();
        for i in 0..n {
            assert_eq!(res.positions[i], res.positions[i + n]);
        }
    }
}
