#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Array modifier: repeat a mesh N times along an offset.

#[allow(dead_code)]
pub struct ArrayResult {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
}

#[allow(dead_code)]
pub fn array_mesh(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    count: u32,
    offset: [f32; 3],
) -> ArrayResult {
    let nv = verts.len();
    let mut out_verts = Vec::with_capacity(array_vert_count(nv, count));
    let mut out_tris = Vec::with_capacity(array_tri_count(tris.len(), count));
    for i in 0..count {
        let dx = offset[0] * i as f32;
        let dy = offset[1] * i as f32;
        let dz = offset[2] * i as f32;
        let base = (i as usize * nv) as u32;
        for v in verts {
            out_verts.push([v[0] + dx, v[1] + dy, v[2] + dz]);
        }
        for t in tris {
            out_tris.push([t[0] + base, t[1] + base, t[2] + base]);
        }
    }
    ArrayResult {
        verts: out_verts,
        tris: out_tris,
    }
}

#[allow(dead_code)]
pub fn array_vert_count(orig: usize, count: u32) -> usize {
    orig * count as usize
}

#[allow(dead_code)]
pub fn array_tri_count(orig: usize, count: u32) -> usize {
    orig * count as usize
}

// ---- New API required by lib.rs ----

/// Array modifier parameters (new API).
pub struct ArrayParams {
    pub count: usize,
    pub offset: [f32; 3],
    pub scale: [f32; 3],
}

pub fn new_array_params(count: usize, offset: [f32; 3]) -> ArrayParams {
    ArrayParams {
        count: count.max(1),
        offset,
        scale: [1.0, 1.0, 1.0],
    }
}

pub fn array_instance_transform(params: &ArrayParams, instance: usize) -> ([f32; 3], [f32; 3]) {
    let i = instance as f32;
    let translation = [
        params.offset[0] * i,
        params.offset[1] * i,
        params.offset[2] * i,
    ];
    (translation, params.scale)
}

pub fn array_total_size(params: &ArrayParams) -> [f32; 3] {
    let n = params.count as f32;
    [
        params.offset[0] * n,
        params.offset[1] * n,
        params.offset[2] * n,
    ]
}

pub fn array_vertex_count(base_count: usize, params: &ArrayParams) -> usize {
    base_count * params.count
}

pub fn array_face_count(base_count: usize, params: &ArrayParams) -> usize {
    base_count * params.count
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tri_verts() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }
    fn unit_tri_tris() -> Vec<[u32; 3]> {
        vec![[0, 1, 2]]
    }

    #[test]
    fn vert_count_formula() {
        assert_eq!(array_vert_count(3, 4), 12);
    }

    #[test]
    fn tri_count_formula() {
        assert_eq!(array_tri_count(2, 3), 6);
    }

    #[test]
    fn single_copy_unchanged() {
        let r = array_mesh(&unit_tri_verts(), &unit_tri_tris(), 1, [1.0, 0.0, 0.0]);
        assert_eq!(r.verts.len(), 3);
        assert_eq!(r.tris.len(), 1);
        assert_eq!(r.tris[0], [0, 1, 2]);
    }

    #[test]
    fn two_copies_vert_count() {
        let r = array_mesh(&unit_tri_verts(), &unit_tri_tris(), 2, [2.0, 0.0, 0.0]);
        assert_eq!(r.verts.len(), 6);
    }

    #[test]
    fn two_copies_offset_applied() {
        let r = array_mesh(&unit_tri_verts(), &unit_tri_tris(), 2, [2.0, 0.0, 0.0]);
        assert!((r.verts[3][0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn two_copies_tri_indices_offset() {
        let r = array_mesh(&unit_tri_verts(), &unit_tri_tris(), 2, [1.0, 0.0, 0.0]);
        assert_eq!(r.tris[1], [3, 4, 5]);
    }

    #[test]
    fn zero_count_empty() {
        let r = array_mesh(&unit_tri_verts(), &unit_tri_tris(), 0, [1.0, 0.0, 0.0]);
        assert!(r.verts.is_empty());
        assert!(r.tris.is_empty());
    }

    #[test]
    fn y_offset_applied() {
        let r = array_mesh(&unit_tri_verts(), &unit_tri_tris(), 3, [0.0, 1.0, 0.0]);
        assert!((r.verts[6][1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn z_offset_applied() {
        let r = array_mesh(&unit_tri_verts(), &unit_tri_tris(), 2, [0.0, 0.0, 5.0]);
        assert!((r.verts[3][2] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn first_copy_at_origin() {
        let r = array_mesh(&unit_tri_verts(), &unit_tri_tris(), 3, [1.0, 1.0, 1.0]);
        assert!((r.verts[0][0]).abs() < 1e-6);
    }
}
