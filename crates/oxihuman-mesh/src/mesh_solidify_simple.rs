// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Result of a solidify (thicken) operation.
#[allow(dead_code)]
pub struct SolidifySimpleResult {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
}

/// Offset each vertex by `d` along its normal.
#[allow(dead_code)]
pub fn offset_verts_simple(verts: &[[f32; 3]], normals: &[[f32; 3]], d: f32) -> Vec<[f32; 3]> {
    verts
        .iter()
        .zip(normals.iter())
        .map(|(v, n)| {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            let s = if len > 1e-9 { d / len } else { d };
            [v[0] + n[0] * s, v[1] + n[1] * s, v[2] + n[2] * s]
        })
        .collect()
}

/// Generate side-wall triangles for `n` boundary vertices starting at `base`.
#[allow(dead_code)]
pub fn solidify_side_tris_simple(n: usize, base: u32) -> Vec<[u32; 3]> {
    let mut tris = Vec::with_capacity(n * 2);
    for i in 0..n {
        let j = (i + 1) % n;
        let o = base;
        let ob = base + n as u32;
        tris.push([o + i as u32, o + j as u32, ob + i as u32]);
        tris.push([o + j as u32, ob + j as u32, ob + i as u32]);
    }
    tris
}

/// Solidify a mesh by offsetting each vertex along its normal and bridging sides.
#[allow(dead_code)]
pub fn solidify_mesh_simple(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    normals: &[[f32; 3]],
    thickness: f32,
) -> SolidifySimpleResult {
    let offset = offset_verts_simple(verts, normals, thickness);
    let n = verts.len() as u32;

    let mut all_verts: Vec<[f32; 3]> = verts.to_vec();
    all_verts.extend_from_slice(&offset);

    let mut all_tris: Vec<[u32; 3]> = tris.to_vec();
    // offset (back) triangles with reversed winding
    for tri in tris {
        all_tris.push([tri[0] + n, tri[2] + n, tri[1] + n]);
    }
    // simple side strip
    let side = solidify_side_tris_simple(verts.len(), 0);
    all_tris.extend(side);

    SolidifySimpleResult { verts: all_verts, tris: all_tris }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn flat_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn flat_normals() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 1.0]; 3]
    }

    fn flat_tris() -> Vec<[u32; 3]> {
        vec![[0, 1, 2]]
    }

    #[test]
    fn test_offset_verts_simple_count() {
        let v = flat_verts();
        let n = flat_normals();
        let out = offset_verts_simple(&v, &n, 1.0);
        assert_eq!(out.len(), v.len());
        let _ = PI;
    }

    #[test]
    fn test_offset_verts_simple_direction() {
        let v = vec![[0.0f32, 0.0, 0.0]];
        let n = vec![[0.0f32, 0.0, 1.0]];
        let out = offset_verts_simple(&v, &n, 0.5);
        assert!((out[0][2] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_offset_verts_simple_zero_normal() {
        let v = vec![[1.0f32, 0.0, 0.0]];
        let n = vec![[0.0f32, 0.0, 0.0]];
        let out = offset_verts_simple(&v, &n, 1.0);
        // with zero normal, original position + thickness * (0,0,0)
        assert!((out[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_solidify_side_tris_simple_count() {
        let sides = solidify_side_tris_simple(3, 0);
        assert_eq!(sides.len(), 6);
    }

    #[test]
    fn test_solidify_side_tris_simple_empty() {
        let sides = solidify_side_tris_simple(0, 0);
        assert!(sides.is_empty());
    }

    #[test]
    fn test_solidify_mesh_simple_vert_count() {
        let v = flat_verts();
        let t = flat_tris();
        let n = flat_normals();
        let result = solidify_mesh_simple(&v, &t, &n, 0.1);
        assert_eq!(result.verts.len(), v.len() * 2);
    }

    #[test]
    fn test_solidify_mesh_simple_tris_increased() {
        let v = flat_verts();
        let t = flat_tris();
        let n = flat_normals();
        let result = solidify_mesh_simple(&v, &t, &n, 0.1);
        assert!(result.tris.len() > t.len());
    }

    #[test]
    fn test_solidify_mesh_simple_offset_applied() {
        let v = flat_verts();
        let t = flat_tris();
        let n = flat_normals();
        let result = solidify_mesh_simple(&v, &t, &n, 1.0);
        // offset verts (indices 3..5) should have z=1.0
        assert!((result.verts[3][2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_solidify_side_tris_simple_base_offset() {
        let sides = solidify_side_tris_simple(2, 10);
        // each tri should include indices >= 10
        for tri in &sides {
            assert!(tri.iter().any(|&i| i >= 10));
        }
    }
}
