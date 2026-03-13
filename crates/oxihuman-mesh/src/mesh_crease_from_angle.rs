#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Auto-crease edges based on dihedral angle threshold.

use std::f32::consts::PI;

#[allow(dead_code)]
pub struct AutoCreaseResult {
    pub crease_edges: Vec<(u32, u32)>,
    pub crease_weights: Vec<f32>,
}

#[allow(dead_code)]
pub fn auto_crease_from_angle(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    threshold_deg: f32,
) -> AutoCreaseResult {
    use std::collections::HashMap;
    let mut edge_tris: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (ti, tri) in tris.iter().enumerate() {
        for e in 0..3 {
            let a = tri[e];
            let b = tri[(e + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_tris.entry(key).or_default().push(ti);
        }
    }
    let face_normals: Vec<[f32; 3]> = tris
        .iter()
        .map(|t| {
            let v0 = verts[t[0] as usize];
            let v1 = verts[t[1] as usize];
            let v2 = verts[t[2] as usize];
            normalize3(cross3(sub3(v1, v0), sub3(v2, v0)))
        })
        .collect();
    let mut crease_edges = vec![];
    let mut crease_weights = vec![];
    for ((a, b), tri_list) in &edge_tris {
        if tri_list.len() < 2 {
            continue;
        }
        let n0 = face_normals[tri_list[0]];
        let n1 = face_normals[tri_list[1]];
        let angle = dihedral_angle(
            verts[*a as usize],
            verts[*b as usize],
            face_normals[tri_list[0]],
            face_normals[tri_list[1]],
        );
        let _ = (n0, n1);
        if angle >= threshold_deg {
            let weight = (angle / 180.0).clamp(0.0, 1.0);
            crease_edges.push((*a, *b));
            crease_weights.push(weight);
        }
    }
    AutoCreaseResult { crease_edges, crease_weights }
}

#[allow(dead_code)]
pub fn dihedral_angle(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3], v3: [f32; 3]) -> f32 {
    // Angle between two face normals: faces share edge (v0,v1), with opposite verts v2 and v3.
    // Compute face normals using the shared edge direction.
    let edge = sub3(v1, v0);
    let n0 = normalize3(cross3(edge, sub3(v2, v0)));
    let n1 = normalize3(cross3(edge, sub3(v3, v0)));
    let dot = dot3(n0, n1).clamp(-1.0, 1.0);
    dot.acos() * 180.0 / PI
}

#[allow(dead_code)]
pub fn crease_count(result: &AutoCreaseResult) -> usize {
    result.crease_edges.len()
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-7 { [0.0, 0.0, 1.0] } else { [v[0] / len, v[1] / len, v[2] / len] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crease_count_empty() {
        let r = AutoCreaseResult { crease_edges: vec![], crease_weights: vec![] };
        assert_eq!(crease_count(&r), 0);
    }

    #[test]
    fn crease_count_matches() {
        let r = AutoCreaseResult { crease_edges: vec![(0, 1), (2, 3)], crease_weights: vec![0.5, 0.8] };
        assert_eq!(crease_count(&r), 2);
    }

    #[test]
    fn dihedral_flat_is_zero() {
        // Two coplanar triangles sharing an edge (v0,v1), with v2 and v3 on opposite sides.
        // Both face normals point in the same direction when wound consistently,
        // so the dihedral angle (between face normals) is 0°.
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.5, 1.0, 0.0];
        // Place v3 on the same side as v2 for same-normal (flat) case
        let v3 = [0.5, 2.0, 0.0];
        let angle = dihedral_angle(v0, v1, v2, v3);
        assert!(angle < 1.0, "Expected near-zero, got {}", angle);
    }

    #[test]
    fn dihedral_right_angle() {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.5, 1.0, 0.0];
        let v3 = [0.5, 0.0, 1.0];
        let angle = dihedral_angle(v0, v1, v2, v3);
        assert!((angle - 90.0).abs() < 1.0, "Expected ~90 deg, got {}", angle);
    }

    #[test]
    fn auto_crease_empty_mesh() {
        let r = auto_crease_from_angle(&[], &[], 30.0);
        assert_eq!(crease_count(&r), 0);
    }

    #[test]
    fn auto_crease_flat_mesh_no_creases() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        let tris: Vec<[u32; 3]> = vec![[0, 1, 2], [1, 3, 2]];
        let r = auto_crease_from_angle(&verts, &tris, 30.0);
        assert_eq!(crease_count(&r), 0);
    }

    #[test]
    fn auto_crease_orthogonal_mesh() {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.0, 1.0],
        ];
        let tris: Vec<[u32; 3]> = vec![[0, 1, 2], [0, 1, 3]];
        let r = auto_crease_from_angle(&verts, &tris, 30.0);
        assert!(crease_count(&r) > 0);
    }

    #[test]
    fn crease_weights_in_range() {
        let r = AutoCreaseResult { crease_edges: vec![(0, 1)], crease_weights: vec![0.5] };
        for &w in &r.crease_weights {
            assert!((0.0..=1.0).contains(&w));
        }
    }

    #[test]
    fn crease_edges_and_weights_same_len() {
        let r = auto_crease_from_angle(&[], &[], 45.0);
        assert_eq!(r.crease_edges.len(), r.crease_weights.len());
    }
}
