// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Flat grid plane mesh generation.

#![allow(dead_code)]

/// Return the number of vertices for a grid plane.
#[allow(dead_code)]
pub fn grid_vert_count(sx: u32, sz: u32) -> usize {
    ((sx + 1) * (sz + 1)) as usize
}

/// Return the number of triangles for a grid plane.
#[allow(dead_code)]
pub fn grid_tri_count(sx: u32, sz: u32) -> usize {
    (sx * sz * 2) as usize
}

/// Compute UV coordinates for grid plane vertices.
#[allow(dead_code)]
pub fn grid_uv(verts: &[[f32; 3]], width: f32, depth: f32) -> Vec<[f32; 2]> {
    let half_w = width * 0.5;
    let half_d = depth * 0.5;
    verts
        .iter()
        .map(|v| {
            let u = (v[0] + half_w) / width.max(1e-8);
            let v_coord = (v[2] + half_d) / depth.max(1e-8);
            [u.clamp(0.0, 1.0), v_coord.clamp(0.0, 1.0)]
        })
        .collect()
}

/// Generate a flat grid plane mesh.
/// Returns (vertices, triangles).
#[allow(dead_code)]
pub fn make_grid_plane(
    width: f32,
    depth: f32,
    segs_x: u32,
    segs_z: u32,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    if segs_x == 0 || segs_z == 0 {
        return (vec![], vec![]);
    }
    let mut verts: Vec<[f32; 3]> = Vec::new();
    for iz in 0..=(segs_z) {
        for ix in 0..=(segs_x) {
            let x = -width * 0.5 + width * ix as f32 / segs_x as f32;
            let z = -depth * 0.5 + depth * iz as f32 / segs_z as f32;
            verts.push([x, 0.0, z]);
        }
    }
    let row_len = segs_x + 1;
    let mut tris: Vec<[u32; 3]> = Vec::new();
    for iz in 0..segs_z {
        for ix in 0..segs_x {
            let a = iz * row_len + ix;
            let b = iz * row_len + ix + 1;
            let c = (iz + 1) * row_len + ix;
            let d = (iz + 1) * row_len + ix + 1;
            tris.push([a, b, c]);
            tris.push([b, d, c]);
        }
    }
    (verts, tris)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_vert_count() {
        assert_eq!(grid_vert_count(4, 4), 25);
    }

    #[test]
    fn test_grid_tri_count() {
        assert_eq!(grid_tri_count(4, 4), 32);
    }

    #[test]
    fn test_make_grid_vert_count() {
        let (verts, _) = make_grid_plane(2.0, 2.0, 4, 4);
        assert_eq!(verts.len(), 25);
    }

    #[test]
    fn test_make_grid_tri_count() {
        let (_, tris) = make_grid_plane(2.0, 2.0, 4, 4);
        assert_eq!(tris.len(), 32);
    }

    #[test]
    fn test_make_grid_zero_segs() {
        let (verts, tris) = make_grid_plane(2.0, 2.0, 0, 4);
        assert!(verts.is_empty());
        assert!(tris.is_empty());
    }

    #[test]
    fn test_grid_uv_range() {
        let (verts, _) = make_grid_plane(2.0, 2.0, 4, 4);
        let uvs = grid_uv(&verts, 2.0, 2.0);
        for uv in &uvs {
            assert!((0.0..=1.0).contains(&uv[0]));
            assert!((0.0..=1.0).contains(&uv[1]));
        }
    }

    #[test]
    fn test_grid_uv_length() {
        let (verts, _) = make_grid_plane(2.0, 2.0, 3, 3);
        let uvs = grid_uv(&verts, 2.0, 2.0);
        assert_eq!(uvs.len(), verts.len());
    }

    #[test]
    fn test_all_y_zero() {
        let (verts, _) = make_grid_plane(2.0, 2.0, 4, 4);
        for v in &verts {
            assert!((v[1]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_indices_in_range() {
        let (verts, tris) = make_grid_plane(2.0, 2.0, 5, 5);
        let nv = verts.len() as u32;
        for tri in &tris {
            assert!(tri[0] < nv);
            assert!(tri[1] < nv);
            assert!(tri[2] < nv);
        }
    }

    #[test]
    fn test_grid_1x1() {
        let (verts, tris) = make_grid_plane(1.0, 1.0, 1, 1);
        assert_eq!(verts.len(), 4);
        assert_eq!(tris.len(), 2);
    }
}
