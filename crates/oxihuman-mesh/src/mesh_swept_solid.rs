// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Swept solid mesh: a profile polygon extruded along a polyline path.

/// A 2D profile polygon (vertices in XY).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Profile2D {
    pub verts: Vec<[f32; 2]>,
}

/// A swept solid mesh result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SweptMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub segment_count: usize,
}

/// Build a swept mesh by extruding `profile` along `path`.
/// Each path point defines a cross-section plane normal = forward direction.
#[allow(dead_code)]
pub fn sweep_profile(profile: &Profile2D, path: &[[f32; 3]]) -> SweptMesh {
    let n_profile = profile.verts.len();
    let n_path = path.len();
    if n_profile == 0 || n_path < 2 {
        return SweptMesh {
            positions: Vec::new(),
            indices: Vec::new(),
            segment_count: 0,
        };
    }

    let mut positions = Vec::with_capacity(n_profile * n_path);
    for (seg, &p) in path.iter().enumerate() {
        let fwd = if seg + 1 < n_path {
            let next = path[seg + 1];
            normalize3([next[0] - p[0], next[1] - p[1], next[2] - p[2]])
        } else {
            let prev = path[seg - 1];
            normalize3([p[0] - prev[0], p[1] - prev[1], p[2] - prev[2]])
        };
        let (right, up) = frame_from_forward(fwd);
        for &v in &profile.verts {
            positions.push([
                p[0] + right[0] * v[0] + up[0] * v[1],
                p[1] + right[1] * v[0] + up[1] * v[1],
                p[2] + right[2] * v[0] + up[2] * v[1],
            ]);
        }
    }

    let mut indices = Vec::new();
    for seg in 0..(n_path - 1) {
        let base0 = (seg * n_profile) as u32;
        let base1 = ((seg + 1) * n_profile) as u32;
        for vi in 0..n_profile {
            let a = base0 + vi as u32;
            let b = base0 + ((vi + 1) % n_profile) as u32;
            let c = base1 + vi as u32;
            let d = base1 + ((vi + 1) % n_profile) as u32;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }

    let segment_count = n_path - 1;
    SweptMesh {
        positions,
        indices,
        segment_count,
    }
}

/// Vertex count of a swept mesh.
#[allow(dead_code)]
pub fn swept_vertex_count(m: &SweptMesh) -> usize {
    m.positions.len()
}

/// Triangle count of a swept mesh.
#[allow(dead_code)]
pub fn swept_triangle_count(m: &SweptMesh) -> usize {
    m.indices.len() / 3
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn frame_from_forward(fwd: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let up_hint = if fwd[1].abs() < 0.9 {
        [0.0_f32, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let right = cross3(fwd, up_hint);
    let right = normalize3(right);
    let up = cross3(right, fwd);
    (right, normalize3(up))
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_profile() -> Profile2D {
        Profile2D {
            verts: vec![[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0]],
        }
    }

    fn straight_path() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, 2.0]]
    }

    #[test]
    fn vertex_count_correct() {
        let m = sweep_profile(&square_profile(), &straight_path());
        assert_eq!(m.positions.len(), 3 * 3);
    }

    #[test]
    fn triangle_count_correct() {
        let m = sweep_profile(&square_profile(), &straight_path());
        assert_eq!(swept_triangle_count(&m), 2 * 3 * 2);
    }

    #[test]
    fn segment_count() {
        let m = sweep_profile(&square_profile(), &straight_path());
        assert_eq!(m.segment_count, 2);
    }

    #[test]
    fn empty_path_returns_empty() {
        let m = sweep_profile(&square_profile(), &[]);
        assert!(m.positions.is_empty());
    }

    #[test]
    fn single_path_returns_empty() {
        let m = sweep_profile(&square_profile(), &[[0.0, 0.0, 0.0]]);
        assert!(m.positions.is_empty());
    }

    #[test]
    fn empty_profile_returns_empty() {
        let p = Profile2D { verts: vec![] };
        let m = sweep_profile(&p, &straight_path());
        assert!(m.positions.is_empty());
    }

    #[test]
    fn indices_multiple_of_three() {
        let m = sweep_profile(&square_profile(), &straight_path());
        assert_eq!(m.indices.len() % 3, 0);
    }

    #[test]
    fn normalize3_unit() {
        let n = normalize3([3.0, 0.0, 0.0]);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn normalize3_zero_safe() {
        let n = normalize3([0.0, 0.0, 0.0]);
        assert_eq!(n, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn swept_vertex_count_helper() {
        let m = sweep_profile(&square_profile(), &straight_path());
        assert_eq!(swept_vertex_count(&m), m.positions.len());
    }
}
