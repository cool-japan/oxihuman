// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Result of a bevel operation.
#[allow(dead_code)]
pub struct BevelEdgeResult {
    pub new_verts: Vec<[f32; 3]>,
    pub new_tris: Vec<[u32; 3]>,
    pub bevel_width: f32,
}

/// Bevel a single edge defined by two vertices, returning 4 new positions.
#[allow(dead_code)]
pub fn bevel_edge_simple(
    v0: [f32; 3],
    v1: [f32; 3],
    width: f32,
) -> ([f32; 3], [f32; 3], [f32; 3], [f32; 3]) {
    let dir = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    let t = if len > 1e-9 { width / len } else { 0.0 };
    let a0 = [v0[0] + dir[0] * t, v0[1] + dir[1] * t, v0[2] + dir[2] * t];
    let a1 = [v1[0] - dir[0] * t, v1[1] - dir[1] * t, v1[2] - dir[2] * t];
    // perpendicular offset using a simple cross with up
    let up = [0.0f32, 0.0, 1.0];
    let perp = cross3(dir, up);
    let plen = (perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2]).sqrt();
    let pn = if plen > 1e-9 {
        [perp[0] / plen * width, perp[1] / plen * width, perp[2] / plen * width]
    } else {
        [width, 0.0, 0.0]
    };
    let b0 = [a0[0] + pn[0], a0[1] + pn[1], a0[2] + pn[2]];
    let b1 = [a1[0] + pn[0], a1[1] + pn[1], a1[2] + pn[2]];
    (a0, a1, b0, b1)
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Bevel a vertex by offsetting along each incident edge.
#[allow(dead_code)]
pub fn bevel_vertex_simple(
    pos: [f32; 3],
    edges: &[([f32; 3], [f32; 3])],
    width: f32,
) -> Vec<[f32; 3]> {
    edges
        .iter()
        .map(|(_, far)| {
            let dir = [far[0] - pos[0], far[1] - pos[1], far[2] - pos[2]];
            let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
            let t = if len > 1e-9 { width / len } else { 0.0 };
            [pos[0] + dir[0] * t, pos[1] + dir[1] * t, pos[2] + dir[2] * t]
        })
        .collect()
}

/// Compute the number of vertices after beveling.
#[allow(dead_code)]
pub fn bevel_vert_count_simple(original: usize, bevel_segs: u32) -> usize {
    original + original * bevel_segs as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_bevel_edge_simple_returns_4_points() {
        let (a0, a1, b0, b1) = bevel_edge_simple([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.1);
        // a0 should be offset inward from v0
        assert!(a0[0] > 0.0 - 1e-5);
        let _ = (a1, b0, b1);
    }

    #[test]
    fn test_bevel_edge_simple_width_zero() {
        let (a0, _a1, _b0, _b1) = bevel_edge_simple([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.0);
        assert!((a0[0]).abs() < 1e-5);
    }

    #[test]
    fn test_bevel_vertex_simple_count() {
        let pos = [0.5f32, 0.5, 0.0];
        let edges = vec![
            ([0.5, 0.5, 0.0], [1.0, 0.5, 0.0]),
            ([0.5, 0.5, 0.0], [0.5, 1.0, 0.0]),
            ([0.5, 0.5, 0.0], [0.0, 0.5, 0.0]),
        ];
        let pts = bevel_vertex_simple(pos, &edges, 0.1);
        assert_eq!(pts.len(), 3);
    }

    #[test]
    fn test_bevel_vertex_simple_offsets_toward_edge() {
        let pos = [0.0f32, 0.0, 0.0];
        let edges = vec![([0.0, 0.0, 0.0], [1.0, 0.0, 0.0])];
        let pts = bevel_vertex_simple(pos, &edges, 0.2);
        assert!(pts[0][0] > 0.0);
    }

    #[test]
    fn test_bevel_vert_count_simple_no_segs() {
        assert_eq!(bevel_vert_count_simple(4, 0), 4);
    }

    #[test]
    fn test_bevel_vert_count_simple_one_seg() {
        assert_eq!(bevel_vert_count_simple(4, 1), 8);
    }

    #[test]
    fn test_bevel_vert_count_simple_two_segs() {
        assert_eq!(bevel_vert_count_simple(3, 2), 9);
    }

    #[test]
    fn test_bevel_edge_simple_distinct_pairs() {
        let (a0, a1, b0, b1) = bevel_edge_simple([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], 0.5);
        // a0 and a1 should differ in x
        assert!((a0[0] - a1[0]).abs() > 0.5);
        let _ = (b0, b1);
    }

    #[test]
    fn test_bevel_vertex_simple_empty_edges() {
        let pts = bevel_vertex_simple([0.0, 0.0, 0.0], &[], 0.5);
        assert!(pts.is_empty());
    }

    #[test]
    fn test_cross3_perpendicular() {
        let x = [1.0f32, 0.0, 0.0];
        let y = [0.0f32, 1.0, 0.0];
        let z = cross3(x, y);
        // cross product of x and y is z
        assert!((z[2] - 1.0).abs() < 1e-5);
        let _ = PI;
    }
}
