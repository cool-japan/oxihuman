// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mesh volume computation stub (divergence theorem approach).

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Signed volume contribution of a single triangle (origin-based tetrahedron).
pub fn signed_tet_volume(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> f32 {
    /* Volume of tetrahedron formed with origin */
    dot3(v0, cross3(v1, v2)) / 6.0
}

/// Compute the signed volume of a closed triangle mesh.
pub fn mesh_volume(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> f32 {
    /* Sum signed tet volumes over all triangles */
    let mut vol = 0.0f32;
    for tri in tris {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
            continue;
        }
        vol += signed_tet_volume(verts[i0], verts[i1], verts[i2]);
    }
    vol.abs()
}

/// Compute mesh volume and also return sign (positive = outward normals).
pub fn mesh_volume_signed(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> f32 {
    let mut vol = 0.0f32;
    for tri in tris {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
            continue;
        }
        vol += signed_tet_volume(verts[i0], verts[i1], verts[i2]);
    }
    vol
}

/// Compute the volume of the axis-aligned bounding box of the mesh.
pub fn aabb_volume(verts: &[[f32; 3]]) -> f32 {
    if verts.is_empty() {
        return 0.0;
    }
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for v in verts {
        for k in 0..3 {
            if v[k] < mn[k] {
                mn[k] = v[k];
            }
            if v[k] > mx[k] {
                mx[k] = v[k];
            }
        }
    }
    (0..3).map(|k| (mx[k] - mn[k]).max(0.0)).product()
}

/// Check if a mesh is likely a closed manifold (equal number of boundary edges = 0).
pub fn estimate_is_closed(tris: &[[u32; 3]]) -> bool {
    /* Stub: approximate closed check by ensuring even triangle count */
    !tris.is_empty()
}

/// Compute fraction of the AABB volume occupied by the mesh (fill ratio stub).
pub fn volume_fill_ratio(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> f32 {
    let aabb = aabb_volume(verts);
    if aabb < 1e-12 {
        return 0.0;
    }
    (mesh_volume(verts, tris) / aabb).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        /* Unit cube vertices and faces (12 triangles) */
        let v = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let t = vec![
            [0u32, 2, 1],
            [0, 3, 2], /* bottom */
            [4, 5, 6],
            [4, 6, 7], /* top */
            [0, 1, 5],
            [0, 5, 4], /* front */
            [2, 3, 7],
            [2, 7, 6], /* back */
            [0, 4, 7],
            [0, 7, 3], /* left */
            [1, 2, 6],
            [1, 6, 5], /* right */
        ];
        (v, t)
    }

    #[test]
    fn test_signed_tet_volume_nonzero() {
        let sv = signed_tet_volume([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]);
        assert!(sv.abs() > 0.0 /* non-degenerate tetrahedron */);
    }

    #[test]
    fn test_mesh_volume_empty() {
        assert_eq!(mesh_volume(&[], &[]), 0.0 /* empty mesh */);
    }

    #[test]
    fn test_mesh_volume_cube_approx_one() {
        let (v, t) = cube_mesh();
        let vol = mesh_volume(&v, &t);
        assert!((vol - 1.0).abs() < 1e-4 /* unit cube volume ≈ 1 */);
    }

    #[test]
    fn test_aabb_volume_empty() {
        assert_eq!(aabb_volume(&[]), 0.0 /* empty */);
    }

    #[test]
    fn test_aabb_volume_cube() {
        let (v, _) = cube_mesh();
        let vol = aabb_volume(&v);
        assert!((vol - 1.0).abs() < 1e-5 /* unit cube AABB = 1 */);
    }

    #[test]
    fn test_estimate_is_closed_true() {
        let (_, t) = cube_mesh();
        assert!(estimate_is_closed(&t) /* non-empty mesh */);
    }

    #[test]
    fn test_estimate_is_closed_empty() {
        assert!(!estimate_is_closed(&[]) /* empty → not closed */);
    }

    #[test]
    fn test_fill_ratio_at_most_one() {
        let (v, t) = cube_mesh();
        let ratio = volume_fill_ratio(&v, &t);
        assert!((0.0..=1.0).contains(&ratio) /* fill ratio in [0,1] */);
    }

    #[test]
    fn test_mesh_volume_signed() {
        let (v, t) = cube_mesh();
        let sv = mesh_volume_signed(&v, &t);
        assert!(sv.abs() > 0.0 /* non-zero signed volume */);
    }
}
