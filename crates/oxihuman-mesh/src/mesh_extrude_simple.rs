// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Result of an extrude operation.
#[allow(dead_code)]
pub struct ExtrudeSimpleResult {
    pub new_verts: Vec<[f32; 3]>,
    pub new_tris: Vec<[u32; 3]>,
    pub extruded_vert_map: Vec<(u32, u32)>,
}

/// Extrude selected face indices by `amount` along their average normal.
#[allow(dead_code)]
pub fn extrude_faces_simple(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    face_indices: &[usize],
    amount: f32,
) -> ExtrudeSimpleResult {
    let mut new_verts: Vec<[f32; 3]> = verts.to_vec();
    let mut new_tris: Vec<[u32; 3]> = tris.to_vec();
    let mut extruded_vert_map: Vec<(u32, u32)> = Vec::new();

    for &fi in face_indices {
        if fi >= tris.len() {
            continue;
        }
        let tri = tris[fi];
        let n = face_normal_simple(verts, tri);
        for &vi in &tri {
            let orig = verts[vi as usize];
            let extruded = [
                orig[0] + n[0] * amount,
                orig[1] + n[1] * amount,
                orig[2] + n[2] * amount,
            ];
            let new_idx = new_verts.len() as u32;
            new_verts.push(extruded);
            extruded_vert_map.push((vi, new_idx));
            new_tris.push([vi, new_idx, new_idx]);
        }
    }

    ExtrudeSimpleResult { new_verts, new_tris, extruded_vert_map }
}

/// Extrude a set of vertices along a given direction by `dist`.
#[allow(dead_code)]
pub fn extrude_vertices_along_simple(
    verts: &[[f32; 3]],
    indices: &[u32],
    dir: [f32; 3],
    dist: f32,
) -> Vec<[f32; 3]> {
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    let d = if len > 1e-9 { len } else { 1.0 };
    let nd = [dir[0] / d * dist, dir[1] / d * dist, dir[2] / d * dist];

    indices
        .iter()
        .filter_map(|&vi| {
            let v = verts.get(vi as usize)?;
            Some([v[0] + nd[0], v[1] + nd[1], v[2] + nd[2]])
        })
        .collect()
}

/// Build cap triangles bridging original loop to extruded loop.
#[allow(dead_code)]
pub fn cap_extrusion_simple(original: &[u32], extruded: &[u32]) -> Vec<[u32; 3]> {
    let n = original.len().min(extruded.len());
    let mut caps = Vec::with_capacity(n * 2);
    for i in 0..n {
        let j = (i + 1) % n;
        caps.push([original[i], extruded[i], extruded[j]]);
        caps.push([original[i], extruded[j], original[j]]);
    }
    caps
}

fn face_normal_simple(verts: &[[f32; 3]], tri: [u32; 3]) -> [f32; 3] {
    let a = verts[tri[0] as usize];
    let b = verts[tri[1] as usize];
    let c = verts[tri[2] as usize];
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-9 { [n[0] / len, n[1] / len, n[2] / len] } else { [0.0, 1.0, 0.0] }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_quad_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn simple_quad_tris() -> Vec<[u32; 3]> {
        vec![[0, 1, 2], [0, 2, 3]]
    }

    #[test]
    fn test_extrude_faces_simple_produces_new_verts() {
        let verts = simple_quad_verts();
        let tris = simple_quad_tris();
        let result = extrude_faces_simple(&verts, &tris, &[0], 0.5);
        assert!(result.new_verts.len() > verts.len());
    }

    #[test]
    fn test_extrude_faces_simple_map_not_empty() {
        let verts = simple_quad_verts();
        let tris = simple_quad_tris();
        let result = extrude_faces_simple(&verts, &tris, &[0], 1.0);
        assert!(!result.extruded_vert_map.is_empty());
    }

    #[test]
    fn test_extrude_faces_simple_out_of_range_ignored() {
        let verts = simple_quad_verts();
        let tris = simple_quad_tris();
        let result = extrude_faces_simple(&verts, &tris, &[99], 1.0);
        assert_eq!(result.new_verts.len(), verts.len());
    }

    #[test]
    fn test_extrude_vertices_along_simple_count() {
        let verts = simple_quad_verts();
        let out = extrude_vertices_along_simple(&verts, &[0, 1, 2], [0.0, 0.0, 1.0], 0.5);
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn test_extrude_vertices_along_simple_values() {
        let verts = vec![[0.0f32, 0.0, 0.0]];
        let out = extrude_vertices_along_simple(&verts, &[0], [0.0, 1.0, 0.0], 2.0);
        assert!((out[0][1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_cap_extrusion_simple_count() {
        let orig = [0u32, 1, 2, 3];
        let ext = [4u32, 5, 6, 7];
        let caps = cap_extrusion_simple(&orig, &ext);
        assert_eq!(caps.len(), 8);
    }

    #[test]
    fn test_cap_extrusion_simple_empty() {
        let caps = cap_extrusion_simple(&[], &[]);
        assert!(caps.is_empty());
    }

    #[test]
    fn test_extrude_all_faces() {
        let verts = simple_quad_verts();
        let tris = simple_quad_tris();
        let result = extrude_faces_simple(&verts, &tris, &[0, 1], 0.25);
        assert!(result.new_tris.len() > tris.len());
    }

    #[test]
    fn test_extrude_zero_amount() {
        let verts = simple_quad_verts();
        let tris = simple_quad_tris();
        let result = extrude_faces_simple(&verts, &tris, &[0], 0.0);
        // new verts are added but at same position as original
        let n_orig = verts.len();
        let n_new = result.new_verts.len();
        assert!(n_new >= n_orig);
    }

    #[test]
    fn test_face_normal_simple_z_up() {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let n = face_normal_simple(&verts, [0, 1, 2]);
        assert!(n[2] > 0.5);
    }
}
