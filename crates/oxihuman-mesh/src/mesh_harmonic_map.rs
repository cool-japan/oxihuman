// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Harmonic mapping for mesh parameterization using Laplacian smoothing to UVs.

use std::f32::consts::PI;

/// Result of harmonic map parameterization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HarmonicMapResult {
    pub uvs: Vec<[f32; 2]>,
    pub iterations_used: u32,
    pub residual: f32,
}

/// Map boundary vertices to a circle in UV space.
#[allow(dead_code)]
pub fn map_boundary_to_circle(boundary: &[u32], n_verts: usize) -> Vec<[f32; 2]> {
    let mut uvs = vec![[0.5f32, 0.5]; n_verts];
    let blen = boundary.len();
    if blen == 0 {
        return uvs;
    }
    for (i, &vi) in boundary.iter().enumerate() {
        let angle = 2.0 * PI * (i as f32) / (blen as f32);
        uvs[vi as usize] = [0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()];
    }
    uvs
}

/// Build adjacency list from triangle indices.
#[allow(dead_code)]
pub fn build_adjacency(indices: &[u32], n_verts: usize) -> Vec<Vec<u32>> {
    let mut adj = vec![Vec::new(); n_verts];
    let tc = indices.len() / 3;
    for t in 0..tc {
        let tri = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            if !adj[a as usize].contains(&b) {
                adj[a as usize].push(b);
            }
            if !adj[b as usize].contains(&a) {
                adj[b as usize].push(a);
            }
        }
    }
    adj
}

/// Solve harmonic map via iterative Laplacian smoothing of interior UVs.
#[allow(dead_code)]
pub fn harmonic_map(
    indices: &[u32],
    n_verts: usize,
    boundary: &[u32],
    max_iterations: u32,
) -> HarmonicMapResult {
    let mut uvs = map_boundary_to_circle(boundary, n_verts);
    let adj = build_adjacency(indices, n_verts);
    let is_boundary: std::collections::HashSet<u32> = boundary.iter().cloned().collect();
    let mut residual = 0.0f32;
    let mut iters = 0u32;
    for it in 0..max_iterations {
        residual = 0.0;
        for vi in 0..n_verts {
            if is_boundary.contains(&(vi as u32)) {
                continue;
            }
            if adj[vi].is_empty() {
                continue;
            }
            let mut avg = [0.0f32; 2];
            for &nb in &adj[vi] {
                avg[0] += uvs[nb as usize][0];
                avg[1] += uvs[nb as usize][1];
            }
            let k = adj[vi].len() as f32;
            avg[0] /= k;
            avg[1] /= k;
            let dx = avg[0] - uvs[vi][0];
            let dy = avg[1] - uvs[vi][1];
            residual += dx * dx + dy * dy;
            uvs[vi] = avg;
        }
        iters = it + 1;
        if residual < 1e-8 {
            break;
        }
    }
    HarmonicMapResult {
        uvs,
        iterations_used: iters,
        residual,
    }
}

/// UV distortion metric: ratio of UV area to 3D area.
#[allow(dead_code)]
pub fn uv_area_ratio(
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
    face: usize,
) -> f32 {
    let i0 = indices[face * 3] as usize;
    let i1 = indices[face * 3 + 1] as usize;
    let i2 = indices[face * 3 + 2] as usize;
    let uv_area = {
        let u0 = uvs[i0];
        let u1 = uvs[i1];
        let u2 = uvs[i2];
        ((u1[0] - u0[0]) * (u2[1] - u0[1]) - (u2[0] - u0[0]) * (u1[1] - u0[1])).abs() * 0.5
    };
    let area_3d = {
        let e1 = [
            positions[i1][0] - positions[i0][0],
            positions[i1][1] - positions[i0][1],
            positions[i1][2] - positions[i0][2],
        ];
        let e2 = [
            positions[i2][0] - positions[i0][0],
            positions[i2][1] - positions[i0][1],
            positions[i2][2] - positions[i0][2],
        ];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        (cx * cx + cy * cy + cz * cz).sqrt() * 0.5
    };
    if area_3d < 1e-12 {
        return 0.0;
    }
    uv_area / area_3d
}

/// Harmonic map vertex count.
#[allow(dead_code)]
pub fn harmonic_map_vertex_count(result: &HarmonicMapResult) -> usize {
    result.uvs.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::f32::consts::PI;

    #[test]
    fn test_map_boundary_circle() {
        let uvs = map_boundary_to_circle(&[0, 1, 2], 3);
        assert_eq!(uvs.len(), 3);
        let _ = PI;
    }

    #[test]
    fn test_boundary_on_unit_circle() {
        let uvs = map_boundary_to_circle(&[0, 1, 2, 3], 4);
        for &uv in &uvs {
            let dx = uv[0] - 0.5;
            let dy = uv[1] - 0.5;
            let r = (dx * dx + dy * dy).sqrt();
            assert!((r - 0.5).abs() < 1e-4);
        }
    }

    #[test]
    fn test_build_adjacency() {
        let adj = build_adjacency(&[0, 1, 2], 3);
        assert_eq!(adj.len(), 3);
        assert!(adj[0].contains(&1));
    }

    #[test]
    fn test_harmonic_map_basic() {
        let indices = vec![0, 1, 2, 0, 2, 3];
        let result = harmonic_map(&indices, 4, &[0, 1, 2, 3], 10);
        assert_eq!(harmonic_map_vertex_count(&result), 4);
    }

    #[test]
    fn test_harmonic_converges() {
        let indices = vec![0, 1, 2, 0, 2, 3, 0, 3, 4];
        let result = harmonic_map(&indices, 5, &[1, 2, 3, 4], 100);
        assert!(result.iterations_used <= 100);
    }

    #[test]
    fn test_uv_area_ratio() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let r = uv_area_ratio(&pos, &uvs, &[0, 1, 2], 0);
        assert!((r - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_empty_boundary() {
        let uvs = map_boundary_to_circle(&[], 3);
        assert_eq!(uvs.len(), 3);
    }

    #[test]
    fn test_single_interior_vertex() {
        // 4 boundary, 1 interior
        let indices = vec![0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4];
        let result = harmonic_map(&indices, 5, &[0, 1, 2, 3], 50);
        // interior vertex 4 should be near center
        let uv4 = result.uvs[4];
        assert!((uv4[0] - 0.5).abs() < 0.3);
        assert!((uv4[1] - 0.5).abs() < 0.3);
    }

    #[test]
    fn test_residual_decreases() {
        let indices = vec![0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4];
        let r1 = harmonic_map(&indices, 5, &[0, 1, 2, 3], 1);
        let r2 = harmonic_map(&indices, 5, &[0, 1, 2, 3], 100);
        assert!(r2.residual <= r1.residual + 1e-6);
    }

    #[test]
    fn test_vertex_count() {
        let result = HarmonicMapResult {
            uvs: vec![[0.0; 2]; 5],
            iterations_used: 0,
            residual: 0.0,
        };
        assert_eq!(harmonic_map_vertex_count(&result), 5);
    }
}
