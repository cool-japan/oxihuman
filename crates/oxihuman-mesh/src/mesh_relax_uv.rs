// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! UV island relaxation.

/// Build UV adjacency from triangle UV indices.
pub fn build_uv_adjacency(uv_indices: &[u32], uv_count: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); uv_count];
    let face_count = uv_indices.len() / 3;
    for fi in 0..face_count {
        let a = uv_indices[fi * 3] as usize;
        let b = uv_indices[fi * 3 + 1] as usize;
        let c = uv_indices[fi * 3 + 2] as usize;
        for (x, y) in [(a, b), (b, c), (c, a), (b, a), (c, b), (a, c)] {
            if x < uv_count && y < uv_count && !adj[x].contains(&y) {
                adj[x].push(y);
            }
        }
    }
    adj
}

/// One step of Laplacian UV relaxation.
pub fn relax_step(
    uvs: &[[f32; 2]],
    adj: &[Vec<usize>],
    boundary: &[bool],
    factor: f32,
) -> Vec<[f32; 2]> {
    let n = uvs.len();
    let mut result = uvs.to_vec();
    for i in 0..n {
        if boundary[i] {
            continue;
        }
        let neighbors = &adj[i];
        if neighbors.is_empty() {
            continue;
        }
        let mut cu = 0.0_f32;
        let mut cv = 0.0_f32;
        for &j in neighbors {
            cu += uvs[j][0];
            cv += uvs[j][1];
        }
        let nf = neighbors.len() as f32;
        cu /= nf;
        cv /= nf;
        result[i][0] = uvs[i][0] + (cu - uvs[i][0]) * factor;
        result[i][1] = uvs[i][1] + (cv - uvs[i][1]) * factor;
    }
    result
}

/// Relax UVs for n iterations.
pub fn relax_uvs(
    uvs: &[[f32; 2]],
    uv_indices: &[u32],
    boundary: &[bool],
    factor: f32,
    iterations: usize,
) -> Vec<[f32; 2]> {
    let adj = build_uv_adjacency(uv_indices, uvs.len());
    let mut current = uvs.to_vec();
    for _ in 0..iterations {
        current = relax_step(&current, &adj, boundary, factor);
    }
    current
}

/// Compute max UV displacement from relaxation.
pub fn max_uv_displacement(before: &[[f32; 2]], after: &[[f32; 2]]) -> f32 {
    let n = before.len().min(after.len());
    (0..n)
        .map(|i| {
            let du = after[i][0] - before[i][0];
            let dv = after[i][1] - before[i][1];
            (du * du + dv * dv).sqrt()
        })
        .fold(0.0_f32, f32::max)
}

/// Check that all UVs are within [0, 1] range.
pub fn uvs_in_range(uvs: &[[f32; 2]]) -> bool {
    uvs.iter()
        .all(|uv| (0.0..=1.0).contains(&uv[0]) && (0.0..=1.0).contains(&uv[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_uvs() -> (Vec<[f32; 2]>, Vec<u32>) {
        let uvs = vec![[0.0_f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let idx = vec![0u32, 1, 2];
        (uvs, idx)
    }

    #[test]
    fn test_build_uv_adjacency_size() {
        /* adjacency size matches uv count */
        let (_, idx) = triangle_uvs();
        let adj = build_uv_adjacency(&idx, 3);
        assert_eq!(adj.len(), 3);
    }

    #[test]
    fn test_relax_step_output_size() {
        /* relax step preserves UV count */
        let (uvs, idx) = triangle_uvs();
        let adj = build_uv_adjacency(&idx, 3);
        let boundary = vec![false; 3];
        let result = relax_step(&uvs, &adj, &boundary, 0.5);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_boundary_vertex_unchanged() {
        /* boundary UVs are not moved */
        let (uvs, idx) = triangle_uvs();
        let adj = build_uv_adjacency(&idx, 3);
        let boundary = vec![true, true, true];
        let result = relax_step(&uvs, &adj, &boundary, 1.0);
        assert!((result[0][0] - uvs[0][0]).abs() < 1e-6);
    }

    #[test]
    fn test_relax_zero_iterations() {
        /* zero iterations returns original */
        let (uvs, idx) = triangle_uvs();
        let boundary = vec![false; 3];
        let result = relax_uvs(&uvs, &idx, &boundary, 0.5, 0);
        assert!((result[1][0] - uvs[1][0]).abs() < 1e-6);
    }

    #[test]
    fn test_max_uv_displacement_identity() {
        /* no relaxation = zero displacement */
        let (uvs, _) = triangle_uvs();
        let d = max_uv_displacement(&uvs, &uvs);
        assert!(d < 1e-6);
    }

    #[test]
    fn test_uvs_in_range_true() {
        /* valid UVs pass range check */
        let uvs = vec![[0.0_f32, 0.0], [0.5, 0.5], [1.0, 1.0]];
        assert!(uvs_in_range(&uvs));
    }

    #[test]
    fn test_uvs_in_range_false() {
        /* out-of-range UVs fail check */
        let uvs = vec![[0.0_f32, 0.0], [1.5, 0.5]];
        assert!(!uvs_in_range(&uvs));
    }

    #[test]
    fn test_relax_moves_interior() {
        /* relaxation with non-zero factor moves interior vertex */
        let (uvs, idx) = triangle_uvs();
        let boundary = vec![true, true, false];
        let result = relax_uvs(&uvs, &idx, &boundary, 1.0, 1);
        let disp = max_uv_displacement(&uvs, &result);
        assert!(disp > 0.0);
    }
}
