// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vertex bevel tool.

/// Result of a vertex bevel operation.
#[derive(Debug, Clone)]
pub struct VertexBevelResult {
    pub new_positions: Vec<[f32; 3]>,
    pub new_indices: Vec<u32>,
    pub beveled_vertex_count: usize,
    pub new_vertex_count: usize,
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-10 {
        [0.0; 3]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Compute bevel positions around a vertex given its connected neighbours.
pub fn bevel_vertex_positions(
    vertex: [f32; 3],
    neighbours: &[[f32; 3]],
    amount: f32,
) -> Vec<[f32; 3]> {
    neighbours
        .iter()
        .map(|&nb| {
            let dir = normalize3([nb[0] - vertex[0], nb[1] - vertex[1], nb[2] - vertex[2]]);
            [
                vertex[0] + dir[0] * amount,
                vertex[1] + dir[1] * amount,
                vertex[2] + dir[2] * amount,
            ]
        })
        .collect()
}

/// Apply vertex bevel to selected vertices.
pub fn bevel_vertices(
    positions: &[[f32; 3]],
    indices: &[u32],
    vertex_indices: &[usize],
    amount: f32,
) -> VertexBevelResult {
    let mut new_positions = positions.to_vec();
    let mut new_indices = indices.to_vec();
    let mut beveled_vertex_count = 0usize;
    let mut total_new = 0usize;
    for &vi in vertex_indices {
        if vi >= positions.len() {
            continue;
        }
        let vertex = positions[vi];
        let mut neighbours: Vec<usize> = Vec::new();
        for tri in indices.chunks(3) {
            if tri.len() < 3 {
                continue;
            }
            let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
            if a == vi {
                if !neighbours.contains(&b) {
                    neighbours.push(b);
                }
                if !neighbours.contains(&c) {
                    neighbours.push(c);
                }
            }
            if b == vi {
                if !neighbours.contains(&a) {
                    neighbours.push(a);
                }
                if !neighbours.contains(&c) {
                    neighbours.push(c);
                }
            }
            if c == vi {
                if !neighbours.contains(&a) {
                    neighbours.push(a);
                }
                if !neighbours.contains(&b) {
                    neighbours.push(b);
                }
            }
        }
        if neighbours.is_empty() {
            continue;
        }
        let nb_positions: Vec<[f32; 3]> = neighbours.iter().map(|&ni| positions[ni]).collect();
        let bevel_pts = bevel_vertex_positions(vertex, &nb_positions, amount);
        let base = new_positions.len() as u32;
        total_new += bevel_pts.len();
        new_positions.extend_from_slice(&bevel_pts);
        if bevel_pts.len() >= 3 {
            for i in 1..bevel_pts.len() - 1 {
                new_indices.extend_from_slice(&[base, base + i as u32, base + i as u32 + 1]);
            }
        }
        beveled_vertex_count += 1;
    }
    VertexBevelResult {
        new_positions,
        new_indices,
        beveled_vertex_count,
        new_vertex_count: total_new,
    }
}

pub fn bevel_centroid(pts: &[[f32; 3]]) -> [f32; 3] {
    if pts.is_empty() {
        return [0.0; 3];
    }
    let n = pts.len() as f32;
    let s = pts
        .iter()
        .fold([0.0f32; 3], |a, &p| [a[0] + p[0], a[1] + p[1], a[2] + p[2]]);
    [s[0] / n, s[1] / n, s[2] / n]
}

pub fn bevel_vertex_estimate(n: usize, valence: usize) -> usize {
    n * valence
}
pub fn validate_bevel_amount(amount: f32, min_edge_len: f32) -> bool {
    amount > 0.0 && amount < min_edge_len * 0.5
}
pub fn default_bevel_amount() -> f32 {
    0.05
}

pub fn bevel_polygon_area_2d(pts: &[[f32; 3]]) -> f32 {
    let n = pts.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        area += pts[i][0] * pts[j][1];
        area -= pts[j][0] * pts[i][1];
    }
    (area * 0.5).abs()
}

pub fn bevel_at_factor(vertex: [f32; 3], bevel_pts: &[[f32; 3]], factor: f32) -> Vec<[f32; 3]> {
    bevel_pts
        .iter()
        .map(|&bp| lerp3(vertex, bp, factor))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bevel_vertex_positions() {
        let v = [0.0, 0.0, 0.0];
        let nb = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let pts = bevel_vertex_positions(v, &nb, 0.2);
        assert_eq!(pts.len(), 2);
        assert!((pts[0][0] - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_bevel_centroid() {
        let pts = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]];
        let c = bevel_centroid(&pts);
        assert!((c[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_bevel_vertex_estimate() {
        assert_eq!(bevel_vertex_estimate(3, 4), 12);
    }

    #[test]
    fn test_validate_bevel_amount() {
        assert!(validate_bevel_amount(0.04, 0.1));
        assert!(!validate_bevel_amount(0.1, 0.1));
        assert!(!validate_bevel_amount(0.0, 1.0));
    }

    #[test]
    fn test_default_bevel_amount() {
        assert!(default_bevel_amount() > 0.0);
    }

    #[test]
    fn test_bevel_polygon_area_2d() {
        let pts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let area = bevel_polygon_area_2d(&pts);
        assert!((area - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_bevel_at_factor() {
        let v = [0.0; 3];
        let pts = vec![[1.0, 0.0, 0.0]];
        let interp = bevel_at_factor(v, &pts, 0.5);
        assert!((interp[0][0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_bevel_vertices_runs() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let res = bevel_vertices(&pos, &idx, &[0], 0.1);
        assert!(res.beveled_vertex_count > 0);
    }

    #[test]
    fn test_bevel_vertices_oob() {
        let pos = vec![[0.0; 3]];
        let idx = vec![];
        let res = bevel_vertices(&pos, &idx, &[99], 0.1);
        assert_eq!(res.beveled_vertex_count, 0);
    }
}
