// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Result of a knife-project cut.
#[allow(dead_code)]
pub struct KnifeResult {
    pub cut_verts: Vec<[f32; 3]>,
    pub cut_edges: Vec<(u32, u32)>,
}

/// Count of cut vertices.
#[allow(dead_code)]
pub fn cut_vert_count(result: &KnifeResult) -> usize {
    result.cut_verts.len()
}

/// Check if a segment intersects a triangle; return intersection point if so.
#[allow(dead_code)]
pub fn segment_tri_intersect(
    seg_a: [f32; 3],
    seg_b: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<[f32; 3]> {
    // Möller–Trumbore against the segment
    let dir = [seg_b[0] - seg_a[0], seg_b[1] - seg_a[1], seg_b[2] - seg_a[2]];
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let h = cross3(dir, e2);
    let a = dot3(e1, h);
    if a.abs() < 1e-9 {
        return None;
    }
    let f = 1.0 / a;
    let s = [seg_a[0] - v0[0], seg_a[1] - v0[1], seg_a[2] - v0[2]];
    let u = f * dot3(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross3(s, e1);
    let v = f * dot3(dir, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * dot3(e2, q);
    if !(0.0..=1.0).contains(&t) {
        return None;
    }
    Some([
        seg_a[0] + t * dir[0],
        seg_a[1] + t * dir[1],
        seg_a[2] + t * dir[2],
    ])
}

/// Project a cut line onto a mesh; gather all intersection points.
#[allow(dead_code)]
pub fn knife_project(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    cut_line: ([f32; 3], [f32; 3]),
) -> KnifeResult {
    let mut cut_verts = Vec::new();
    let mut cut_edges = Vec::new();

    for tri in tris {
        let v0 = verts[tri[0] as usize];
        let v1 = verts[tri[1] as usize];
        let v2 = verts[tri[2] as usize];
        if let Some(pt) = segment_tri_intersect(cut_line.0, cut_line.1, v0, v1, v2) {
            let idx = cut_verts.len() as u32;
            cut_verts.push(pt);
            if idx > 0 {
                cut_edges.push((idx - 1, idx));
            }
        }
    }

    KnifeResult { cut_verts, cut_edges }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn unit_tri_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn unit_tri_indices() -> Vec<[u32; 3]> {
        vec![[0, 1, 2]]
    }

    #[test]
    fn test_segment_tri_intersect_hits() {
        let v0 = [0.0f32, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        // ray from above center down
        let pt = segment_tri_intersect(
            [0.25, 0.25, 1.0],
            [0.25, 0.25, -1.0],
            v0, v1, v2,
        );
        assert!(pt.is_some());
        let _ = PI;
    }

    #[test]
    fn test_segment_tri_intersect_miss() {
        let v0 = [0.0f32, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        let pt = segment_tri_intersect(
            [5.0, 5.0, 1.0],
            [5.0, 5.0, -1.0],
            v0, v1, v2,
        );
        assert!(pt.is_none());
    }

    #[test]
    fn test_segment_tri_intersect_parallel() {
        let v0 = [0.0f32, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        // segment parallel to plane
        let pt = segment_tri_intersect(
            [0.25, 0.25, 0.5],
            [0.75, 0.25, 0.5],
            v0, v1, v2,
        );
        assert!(pt.is_none());
    }

    #[test]
    fn test_knife_project_no_tris() {
        let verts = unit_tri_verts();
        let result = knife_project(&verts, &[], ([0.5, 0.0, 1.0], [0.5, 0.0, -1.0]));
        assert!(result.cut_verts.is_empty());
    }

    #[test]
    fn test_knife_project_hit() {
        let verts = unit_tri_verts();
        let tris = unit_tri_indices();
        let result = knife_project(&verts, &tris, ([0.2, 0.2, 1.0], [0.2, 0.2, -1.0]));
        assert!(!result.cut_verts.is_empty());
    }

    #[test]
    fn test_cut_vert_count_helper() {
        let r = KnifeResult { cut_verts: vec![[0.0; 3]; 3], cut_edges: Vec::new() };
        assert_eq!(cut_vert_count(&r), 3);
    }

    #[test]
    fn test_knife_edges_count() {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [1.0, 2.0, 0.0],
            [3.0, 2.0, 0.0],
        ];
        let tris = vec![[0u32, 1, 2], [1, 3, 2]];
        let result = knife_project(&verts, &tris, ([1.5, 1.0, 2.0], [1.5, 1.0, -2.0]));
        // edges connect consecutive cut verts
        if result.cut_verts.len() > 1 {
            assert!(!result.cut_edges.is_empty());
        }
    }

    #[test]
    fn test_dot3_basic() {
        let a = [1.0f32, 0.0, 0.0];
        let b = [0.0f32, 1.0, 0.0];
        assert!((dot3(a, b)).abs() < 1e-6);
    }

    #[test]
    fn test_cross3_basic() {
        let x = [1.0f32, 0.0, 0.0];
        let y = [0.0f32, 1.0, 0.0];
        let z = cross3(x, y);
        assert!((z[2] - 1.0).abs() < 1e-5);
    }
}
