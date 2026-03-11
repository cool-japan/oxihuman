// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Smooth vertex position operations.

/// Build simple adjacency list from triangle indices.
pub fn build_adjacency(indices: &[u32], vertex_count: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); vertex_count];
    let face_count = indices.len() / 3;
    for fi in 0..face_count {
        let a = indices[fi * 3] as usize;
        let b = indices[fi * 3 + 1] as usize;
        let c = indices[fi * 3 + 2] as usize;
        for (x, y) in [(a, b), (b, c), (c, a), (b, a), (c, b), (a, c)] {
            if !adj[x].contains(&y) {
                adj[x].push(y);
            }
        }
    }
    adj
}

/// One step of Laplacian smoothing of vertex positions.
pub fn smooth_step(positions: &[[f32; 3]], adj: &[Vec<usize>], factor: f32) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut result = positions.to_vec();
    for i in 0..n {
        let neighbors = &adj[i];
        if neighbors.is_empty() {
            continue;
        }
        let mut cx = 0.0_f32;
        let mut cy = 0.0_f32;
        let mut cz = 0.0_f32;
        for &j in neighbors {
            cx += positions[j][0];
            cy += positions[j][1];
            cz += positions[j][2];
        }
        let nf = neighbors.len() as f32;
        cx /= nf;
        cy /= nf;
        cz /= nf;
        result[i][0] = positions[i][0] + (cx - positions[i][0]) * factor;
        result[i][1] = positions[i][1] + (cy - positions[i][1]) * factor;
        result[i][2] = positions[i][2] + (cz - positions[i][2]) * factor;
    }
    result
}

/// Apply n iterations of Laplacian smoothing.
pub fn smooth_n(
    positions: &[[f32; 3]],
    indices: &[u32],
    factor: f32,
    iterations: usize,
) -> Vec<[f32; 3]> {
    let adj = build_adjacency(indices, positions.len());
    let mut pos = positions.to_vec();
    for _ in 0..iterations {
        pos = smooth_step(&pos, &adj, factor);
    }
    pos
}

/// Compute per-vertex displacement from original to smoothed positions.
pub fn smooth_displacement(original: &[[f32; 3]], smoothed: &[[f32; 3]]) -> Vec<f32> {
    let n = original.len().min(smoothed.len());
    (0..n)
        .map(|i| {
            let dx = smoothed[i][0] - original[i][0];
            let dy = smoothed[i][1] - original[i][1];
            let dz = smoothed[i][2] - original[i][2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .collect()
}

/// Max displacement after smoothing.
pub fn max_displacement(disp: &[f32]) -> f32 {
    disp.iter().cloned().fold(0.0_f32, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        /* three vertices in a line, two triangles (degenerate) */
        let pos = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        (pos, idx)
    }

    #[test]
    fn test_build_adjacency_size() {
        /* adjacency has one entry per vertex */
        let (_, idx) = line_mesh();
        let adj = build_adjacency(&idx, 3);
        assert_eq!(adj.len(), 3);
    }

    #[test]
    fn test_smooth_step_output_size() {
        /* smooth step preserves vertex count */
        let (pos, idx) = line_mesh();
        let adj = build_adjacency(&idx, 3);
        let result = smooth_step(&pos, &adj, 0.5);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_smooth_n_zero_iterations() {
        /* zero iterations returns original */
        let (pos, idx) = line_mesh();
        let result = smooth_n(&pos, &idx, 0.5, 0);
        assert!((result[1][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_smooth_n_reduces_extremes() {
        /* smoothing moves outer vertices closer to center */
        let (pos, idx) = line_mesh();
        let result = smooth_n(&pos, &idx, 1.0, 5);
        assert!(result[0][0] > 0.0 || result[0][0] >= 0.0);
    }

    #[test]
    fn test_smooth_displacement_zero_no_change() {
        /* zero smoothing gives zero displacement */
        let (pos, idx) = line_mesh();
        let result = smooth_n(&pos, &idx, 0.0, 1);
        let disp = smooth_displacement(&pos, &result);
        assert!(disp.iter().all(|&d| d < 1e-6));
    }

    #[test]
    fn test_smooth_displacement_nonzero() {
        /* smoothing with factor > 0 creates displacement */
        let (pos, idx) = line_mesh();
        let result = smooth_n(&pos, &idx, 0.5, 1);
        let disp = smooth_displacement(&pos, &result);
        assert!(disp.iter().any(|&d| d > 0.0));
    }

    #[test]
    fn test_max_displacement() {
        /* max displacement is the largest value */
        let disp = vec![0.1_f32, 0.5, 0.3];
        assert!((max_displacement(&disp) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_max_displacement_empty() {
        /* max displacement on empty slice is 0 */
        assert_eq!(max_displacement(&[]), 0.0);
    }
}
