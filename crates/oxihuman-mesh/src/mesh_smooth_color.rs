// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Smooth vertex colors using Laplacian diffusion.

/// Smooth vertex RGBA colors by averaging neighbors, `iterations` times.
/// `factor` in [0, 1]: blend weight toward neighbor average (1 = full Laplacian).
#[allow(dead_code)]
pub fn smooth_vertex_colors(
    colors: &[[f32; 4]],
    indices: &[u32],
    iterations: usize,
    factor: f32,
) -> Vec<[f32; 4]> {
    let n = colors.len();
    if n == 0 {
        return vec![];
    }
    let adj = build_color_adjacency(n, indices);
    let mut current = colors.to_vec();
    for _ in 0..iterations {
        let mut next = current.clone();
        for vi in 0..n {
            if adj[vi].is_empty() {
                continue;
            }
            let mut avg = [0.0f32; 4];
            for &nb in &adj[vi] {
                avg[0] += current[nb][0];
                avg[1] += current[nb][1];
                avg[2] += current[nb][2];
                avg[3] += current[nb][3];
            }
            let k = adj[vi].len() as f32;
            avg = [avg[0] / k, avg[1] / k, avg[2] / k, avg[3] / k];
            let c = current[vi];
            next[vi] = [
                c[0] + factor * (avg[0] - c[0]),
                c[1] + factor * (avg[1] - c[1]),
                c[2] + factor * (avg[2] - c[2]),
                c[3] + factor * (avg[3] - c[3]),
            ];
        }
        current = next;
    }
    current
}

/// Build adjacency list (neighbours per vertex) from index buffer.
#[allow(dead_code)]
pub fn build_color_adjacency(n: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut adj = vec![std::collections::HashSet::new(); n];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        adj[a].insert(b);
        adj[a].insert(c);
        adj[b].insert(a);
        adj[b].insert(c);
        adj[c].insert(a);
        adj[c].insert(b);
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

/// Clamp all color channels to [0, 1].
#[allow(dead_code)]
pub fn clamp_colors(colors: &[[f32; 4]]) -> Vec<[f32; 4]> {
    colors
        .iter()
        .map(|&c| {
            [
                c[0].clamp(0.0, 1.0),
                c[1].clamp(0.0, 1.0),
                c[2].clamp(0.0, 1.0),
                c[3].clamp(0.0, 1.0),
            ]
        })
        .collect()
}

/// Compute average color across all vertices.
#[allow(dead_code)]
pub fn average_color(colors: &[[f32; 4]]) -> [f32; 4] {
    if colors.is_empty() {
        return [0.0; 4];
    }
    let n = colors.len() as f32;
    let s = colors.iter().fold([0.0f32; 4], |acc, &c| {
        [acc[0] + c[0], acc[1] + c[1], acc[2] + c[2], acc[3] + c[3]]
    });
    [s[0] / n, s[1] / n, s[2] / n, s[3] / n]
}

/// Check all color channels are in [0, 1].
#[allow(dead_code)]
pub fn colors_in_range(colors: &[[f32; 4]]) -> bool {
    colors
        .iter()
        .all(|&c| c.iter().all(|&v| (0.0..=1.0).contains(&v)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri_mesh() -> (Vec<[f32; 4]>, Vec<u32>) {
        let colors = vec![
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
        ];
        let idx = vec![0u32, 1, 2];
        (colors, idx)
    }

    #[test]
    fn smooth_preserves_count() {
        let (col, idx) = tri_mesh();
        let res = smooth_vertex_colors(&col, &idx, 3, 0.5);
        assert_eq!(res.len(), col.len());
    }

    #[test]
    fn smooth_zero_iterations_unchanged() {
        let (col, idx) = tri_mesh();
        let res = smooth_vertex_colors(&col, &idx, 0, 0.5);
        for (a, b) in col.iter().zip(res.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-6);
        }
    }

    #[test]
    fn smooth_zero_factor_unchanged() {
        let (col, idx) = tri_mesh();
        let res = smooth_vertex_colors(&col, &idx, 5, 0.0);
        for (a, b) in col.iter().zip(res.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-6);
        }
    }

    #[test]
    fn smooth_empty_colors() {
        let res = smooth_vertex_colors(&[], &[], 3, 0.5);
        assert!(res.is_empty());
    }

    #[test]
    fn smooth_reduces_variance() {
        let (col, idx) = tri_mesh();
        let avg_before = average_color(&col);
        let res = smooth_vertex_colors(&col, &idx, 10, 1.0);
        let avg_after = average_color(&res);
        assert!((avg_before[0] - avg_after[0]).abs() < 0.1);
    }

    #[test]
    fn adjacency_symmetric() {
        let (_, idx) = tri_mesh();
        let adj = build_color_adjacency(3, &idx);
        assert!(adj[0].contains(&1));
        assert!(adj[1].contains(&0));
    }

    #[test]
    fn clamp_colors_in_range() {
        let colors = vec![[-0.5, 0.5, 1.5, 0.5]];
        let clamped = clamp_colors(&colors);
        assert!(colors_in_range(&clamped));
    }

    #[test]
    fn average_color_uniform() {
        let colors = vec![[0.5f32, 0.5, 0.5, 0.5]; 4];
        let avg = average_color(&colors);
        assert!((avg[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn colors_in_range_true() {
        let colors = vec![[0.0, 0.5, 1.0, 0.3]];
        assert!(colors_in_range(&colors));
    }

    #[test]
    fn colors_in_range_false() {
        let colors = vec![[1.5, 0.5, 0.0, 1.0]];
        assert!(!colors_in_range(&colors));
    }
}
