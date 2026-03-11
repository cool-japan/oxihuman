// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Bounding Volume Hierarchy for ray-triangle intersection (v2).

#[allow(dead_code)]
pub struct BVHNodeV2 {
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
    pub left: usize,
    pub right: usize,
    pub face_idx: i32,
}

#[allow(dead_code)]
pub struct BVHv2 {
    pub nodes: Vec<BVHNodeV2>,
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
}

fn triangle_aabb(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let mn = [
        p0[0].min(p1[0]).min(p2[0]),
        p0[1].min(p1[1]).min(p2[1]),
        p0[2].min(p1[2]).min(p2[2]),
    ];
    let mx = [
        p0[0].max(p1[0]).max(p2[0]),
        p0[1].max(p1[1]).max(p2[1]),
        p0[2].max(p1[2]).max(p2[2]),
    ];
    (mn, mx)
}

#[allow(dead_code)]
pub fn new_bvh_v2(positions: Vec<[f32; 3]>, indices: Vec<[u32; 3]>) -> BVHv2 {
    let mut nodes = Vec::new();
    for (fi, tri) in indices.iter().enumerate() {
        let p0 = positions[tri[0] as usize];
        let p1 = positions[tri[1] as usize];
        let p2 = positions[tri[2] as usize];
        let (mn, mx) = triangle_aabb(p0, p1, p2);
        nodes.push(BVHNodeV2 {
            aabb_min: mn,
            aabb_max: mx,
            left: 0,
            right: 0,
            face_idx: fi as i32,
        });
    }
    BVHv2 { nodes, positions, indices }
}

#[allow(dead_code)]
pub fn bvh_v2_node_count(b: &BVHv2) -> usize {
    b.nodes.len()
}

#[allow(dead_code)]
pub fn bvh_v2_face_count(b: &BVHv2) -> usize {
    b.indices.len()
}

fn ray_aabb(origin: [f32; 3], dir: [f32; 3], mn: [f32; 3], mx: [f32; 3]) -> bool {
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;
    for i in 0..3 {
        let inv = if dir[i].abs() < 1e-10 { f32::INFINITY } else { 1.0 / dir[i] };
        let t0 = (mn[i] - origin[i]) * inv;
        let t1 = (mx[i] - origin[i]) * inv;
        let (lo, hi) = if t0 < t1 { (t0, t1) } else { (t1, t0) };
        tmin = tmin.max(lo);
        tmax = tmax.min(hi);
        if tmax < tmin { return false; }
    }
    tmax >= 0.0
}

fn ray_triangle(origin: [f32; 3], dir: [f32; 3], p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> bool {
    let e1 = [p1[0]-p0[0], p1[1]-p0[1], p1[2]-p0[2]];
    let e2 = [p2[0]-p0[0], p2[1]-p0[1], p2[2]-p0[2]];
    let h = [dir[1]*e2[2]-dir[2]*e2[1], dir[2]*e2[0]-dir[0]*e2[2], dir[0]*e2[1]-dir[1]*e2[0]];
    let a = e1[0]*h[0] + e1[1]*h[1] + e1[2]*h[2];
    if a.abs() < 1e-10 { return false; }
    let f = 1.0 / a;
    let s = [origin[0]-p0[0], origin[1]-p0[1], origin[2]-p0[2]];
    let u = f * (s[0]*h[0] + s[1]*h[1] + s[2]*h[2]);
    if !(0.0..=1.0).contains(&u) { return false; }
    let q = [s[1]*e1[2]-s[2]*e1[1], s[2]*e1[0]-s[0]*e1[2], s[0]*e1[1]-s[1]*e1[0]];
    let v = f * (dir[0]*q[0] + dir[1]*q[1] + dir[2]*q[2]);
    if v < 0.0 || u + v > 1.0 { return false; }
    let t = f * (e2[0]*q[0] + e2[1]*q[1] + e2[2]*q[2]);
    t > 1e-10
}

#[allow(dead_code)]
pub fn bvh_v2_ray_test(b: &BVHv2, origin: [f32; 3], dir: [f32; 3]) -> Option<usize> {
    for (fi, tri) in b.indices.iter().enumerate() {
        let p0 = b.positions[tri[0] as usize];
        let p1 = b.positions[tri[1] as usize];
        let p2 = b.positions[tri[2] as usize];
        if ray_triangle(origin, dir, p0, p1, p2) {
            return Some(fi);
        }
    }
    None
}

#[allow(dead_code)]
pub fn bvh_v2_bounds(b: &BVHv2) -> ([f32; 3], [f32; 3]) {
    if b.positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = b.positions[0];
    let mut mx = b.positions[0];
    for p in &b.positions {
        for i in 0..3 {
            if p[i] < mn[i] { mn[i] = p[i]; }
            if p[i] > mx[i] { mx[i] = p[i]; }
        }
    }
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_bvh() -> BVHv2 {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ];
        let indices = vec![[0u32, 1, 2]];
        new_bvh_v2(positions, indices)
    }

    #[test]
    fn test_node_count() {
        let b = simple_bvh();
        assert_eq!(bvh_v2_node_count(&b), 1);
    }

    #[test]
    fn test_face_count() {
        let b = simple_bvh();
        assert_eq!(bvh_v2_face_count(&b), 1);
    }

    #[test]
    fn test_bounds() {
        let b = simple_bvh();
        let (lo, hi) = bvh_v2_bounds(&b);
        assert!(lo[0] <= 0.0);
        assert!(hi[0] >= 1.0);
    }

    #[test]
    fn test_ray_test_no_crash() {
        let b = simple_bvh();
        let _ = bvh_v2_ray_test(&b, [0.5, 0.3, -1.0], [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_ray_test_hit() {
        let b = simple_bvh();
        let result = bvh_v2_ray_test(&b, [0.5, 0.3, -1.0], [0.0, 0.0, 1.0]);
        assert!(result.is_some());
    }

    #[test]
    fn test_ray_test_miss() {
        let b = simple_bvh();
        let result = bvh_v2_ray_test(&b, [10.0, 10.0, -1.0], [0.0, 0.0, 1.0]);
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_bvh() {
        let b = new_bvh_v2(vec![], vec![]);
        assert_eq!(bvh_v2_node_count(&b), 0);
        assert_eq!(bvh_v2_face_count(&b), 0);
    }

    #[test]
    fn test_bounds_empty() {
        let b = new_bvh_v2(vec![], vec![]);
        let (lo, hi) = bvh_v2_bounds(&b);
        assert_eq!(lo, [0.0; 3]);
        assert_eq!(hi, [0.0; 3]);
    }
}
