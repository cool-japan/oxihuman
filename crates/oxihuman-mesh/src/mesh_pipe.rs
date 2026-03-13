// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pipe/tube mesh generation along a path.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Result of a pipe mesh operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PipeResult {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
}

/// Compute the tangent direction at path index `i`.
#[allow(dead_code)]
pub fn path_tangent(path: &[[f32; 3]], i: usize) -> [f32; 3] {
    if path.len() < 2 {
        return [0.0, 1.0, 0.0];
    }
    let (a, b) = if i == 0 {
        (path[0], path[1])
    } else if i >= path.len() - 1 {
        (path[path.len() - 2], path[path.len() - 1])
    } else {
        (path[i - 1], path[i + 1])
    };
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-8);
    [dx / len, dy / len, dz / len]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-8);
    [v[0] / len, v[1] / len, v[2] / len]
}

fn build_ring(center: [f32; 3], tangent: [f32; 3], radius: f32, segments: u32) -> Vec<[f32; 3]> {
    let up = if tangent[1].abs() < 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let right = normalize(cross(tangent, up));
    let up2 = cross(right, tangent);
    (0..segments)
        .map(|i| {
            let angle = 2.0 * PI * i as f32 / segments as f32;
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            [
                center[0] + radius * (cos_a * right[0] + sin_a * up2[0]),
                center[1] + radius * (cos_a * right[1] + sin_a * up2[1]),
                center[2] + radius * (cos_a * right[2] + sin_a * up2[2]),
            ]
        })
        .collect()
}

/// Generate a triangulated cap (fan) for a ring.
#[allow(dead_code)]
pub fn pipe_cap(_center: [f32; 3], ring: &[[f32; 3]], base_idx: u32) -> Vec<[u32; 3]> {
    let center_idx = base_idx + ring.len() as u32;
    (0..ring.len())
        .map(|i| {
            let next = (i + 1) % ring.len();
            [base_idx + i as u32, base_idx + next as u32, center_idx]
        })
        .collect()
}

/// Generate a pipe (tube) mesh along a path.
#[allow(dead_code)]
pub fn make_pipe(path: &[[f32; 3]], radius: f32, segments: u32) -> PipeResult {
    if path.len() < 2 || segments < 3 {
        return PipeResult { verts: vec![], tris: vec![] };
    }
    let segs = segments as usize;
    let mut verts: Vec<[f32; 3]> = Vec::new();
    for (i, &center) in path.iter().enumerate() {
        let tangent = path_tangent(path, i);
        let ring = build_ring(center, tangent, radius, segments);
        verts.extend_from_slice(&ring);
    }
    let n_rings = path.len();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    for r in 0..(n_rings - 1) {
        for i in 0..segs {
            let next_i = (i + 1) % segs;
            let a = (r * segs + i) as u32;
            let b = (r * segs + next_i) as u32;
            let c = ((r + 1) * segs + i) as u32;
            let d = ((r + 1) * segs + next_i) as u32;
            tris.push([a, b, c]);
            tris.push([b, d, c]);
        }
    }
    PipeResult { verts, tris }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn straight_path() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]]
    }

    #[test]
    fn test_make_pipe_basic() {
        let path = straight_path();
        let result = make_pipe(&path, 0.5, 8);
        assert_eq!(result.verts.len(), 24);
    }

    #[test]
    fn test_make_pipe_tri_count() {
        let path = straight_path();
        let result = make_pipe(&path, 0.5, 8);
        // 2 segments * 8 quads * 2 tris
        assert_eq!(result.tris.len(), 32);
    }

    #[test]
    fn test_make_pipe_empty_path() {
        let result = make_pipe(&[], 0.5, 8);
        assert!(result.verts.is_empty());
        assert!(result.tris.is_empty());
    }

    #[test]
    fn test_make_pipe_few_segments() {
        let path = straight_path();
        let result = make_pipe(&path, 1.0, 2);
        assert!(result.verts.is_empty());
    }

    #[test]
    fn test_path_tangent_middle() {
        let path = vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]];
        let t = path_tangent(&path, 1);
        assert!((t[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_path_tangent_start() {
        let path = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let t = path_tangent(&path, 0);
        assert!((t[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_path_tangent_end() {
        let path = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let t = path_tangent(&path, 1);
        assert!((t[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_pipe_cap_triangle_count() {
        let ring = vec![
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
        ];
        let caps = pipe_cap([0.0, 0.0, 0.0], &ring, 0);
        assert_eq!(caps.len(), 4);
    }

    #[test]
    fn test_pipe_indices_in_range() {
        let path = vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let result = make_pipe(&path, 0.3, 6);
        let nv = result.verts.len() as u32;
        for tri in &result.tris {
            assert!(tri[0] < nv);
            assert!(tri[1] < nv);
            assert!(tri[2] < nv);
        }
    }

    #[test]
    fn test_make_pipe_single_segment() {
        let path = vec![[0.0, 0.0, 0.0], [0.0, 5.0, 0.0]];
        let result = make_pipe(&path, 1.0, 12);
        assert_eq!(result.verts.len(), 24);
        assert_eq!(result.tris.len(), 24);
    }
}
