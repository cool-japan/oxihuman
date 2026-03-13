#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mirror modifier: mirror mesh across an axis.

#[allow(dead_code)]
pub struct MirrorResult {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
}

#[allow(dead_code)]
pub fn mirror_vert(v: [f32; 3], axis: u8) -> [f32; 3] {
    match axis {
        0 => [-v[0], v[1], v[2]],
        1 => [v[0], -v[1], v[2]],
        2 => [v[0], v[1], -v[2]],
        _ => v,
    }
}

fn do_mirror(verts: &[[f32; 3]], tris: &[[u32; 3]], axis: u8) -> MirrorResult {
    let nv = verts.len() as u32;
    let mut out_verts: Vec<[f32; 3]> = verts.to_vec();
    let mut out_tris: Vec<[u32; 3]> = tris.to_vec();
    for v in verts {
        out_verts.push(mirror_vert(*v, axis));
    }
    for t in tris {
        // flip winding on mirror side
        out_tris.push([t[0] + nv, t[2] + nv, t[1] + nv]);
    }
    MirrorResult {
        verts: out_verts,
        tris: out_tris,
    }
}

#[allow(dead_code)]
pub fn mirror_x(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> MirrorResult {
    do_mirror(verts, tris, 0)
}

#[allow(dead_code)]
pub fn mirror_y(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> MirrorResult {
    do_mirror(verts, tris, 1)
}

#[allow(dead_code)]
pub fn mirror_z(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> MirrorResult {
    do_mirror(verts, tris, 2)
}

// ---- New API required by lib.rs ----

/// Mirror modifier parameters (new API).
pub struct MirrorParams {
    pub axis_x: bool,
    pub axis_y: bool,
    pub axis_z: bool,
    pub merge_threshold: f32,
}

pub fn new_mirror_params() -> MirrorParams {
    MirrorParams {
        axis_x: true,
        axis_y: false,
        axis_z: false,
        merge_threshold: 0.001,
    }
}

pub fn mirror_vertex(p: [f32; 3], params: &MirrorParams) -> Vec<[f32; 3]> {
    let mut result = Vec::new();
    if params.axis_x {
        result.push([-p[0], p[1], p[2]]);
    }
    if params.axis_y {
        result.push([p[0], -p[1], p[2]]);
    }
    if params.axis_z {
        result.push([p[0], p[1], -p[2]]);
    }
    result
}

pub fn mirror_should_merge(a: [f32; 3], b: [f32; 3], threshold: f32) -> bool {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt() < threshold
}

pub fn mirror_copy_count(params: &MirrorParams) -> usize {
    let mut n = 1usize;
    if params.axis_x {
        n += 1;
    }
    if params.axis_y {
        n += 1;
    }
    if params.axis_z {
        n += 1;
    }
    n
}

pub fn mirror_flip_normal(n: [f32; 3], params: &MirrorParams) -> [f32; 3] {
    let mut r = n;
    if params.axis_x {
        r[0] = -r[0];
    }
    if params.axis_y {
        r[1] = -r[1];
    }
    if params.axis_z {
        r[2] = -r[2];
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_verts() -> Vec<[f32; 3]> {
        vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]
    }
    fn sample_tris() -> Vec<[u32; 3]> {
        vec![[0, 1, 2]]
    }

    #[test]
    fn mirror_vert_x_negates_x() {
        let v = mirror_vert([3.0, 4.0, 5.0], 0);
        assert!((v[0] + 3.0).abs() < 1e-6);
        assert!((v[1] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn mirror_vert_y_negates_y() {
        let v = mirror_vert([1.0, 2.0, 3.0], 1);
        assert!((v[1] + 2.0).abs() < 1e-6);
    }

    #[test]
    fn mirror_vert_z_negates_z() {
        let v = mirror_vert([1.0, 2.0, 3.0], 2);
        assert!((v[2] + 3.0).abs() < 1e-6);
    }

    #[test]
    fn mirror_x_doubles_vert_count() {
        let r = mirror_x(&sample_verts(), &sample_tris());
        assert_eq!(r.verts.len(), 6);
    }

    #[test]
    fn mirror_x_doubles_tri_count() {
        let r = mirror_x(&sample_verts(), &sample_tris());
        assert_eq!(r.tris.len(), 2);
    }

    #[test]
    fn mirror_y_doubles_vert_count() {
        let r = mirror_y(&sample_verts(), &sample_tris());
        assert_eq!(r.verts.len(), 6);
    }

    #[test]
    fn mirror_z_doubles_vert_count() {
        let r = mirror_z(&sample_verts(), &sample_tris());
        assert_eq!(r.verts.len(), 6);
    }

    #[test]
    fn mirror_x_mirrored_vert_x_negated() {
        let r = mirror_x(&sample_verts(), &sample_tris());
        assert!((r.verts[3][0] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn mirror_winding_flipped() {
        let r = mirror_x(&sample_verts(), &sample_tris());
        // second tri should have indices [0+3, 2+3, 1+3]
        assert_eq!(r.tris[1], [3, 5, 4]);
    }
}
