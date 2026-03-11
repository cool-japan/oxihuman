// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Catmull-Clark subdivision v2.

#[allow(dead_code)]
pub struct CatmullClarkV2 {
    pub positions: Vec<[f32; 3]>,
    pub quads: Vec<[u32; 4]>,
}

#[allow(dead_code)]
pub fn new_catmull_clark_v2(positions: Vec<[f32; 3]>, quads: Vec<[u32; 4]>) -> CatmullClarkV2 {
    CatmullClarkV2 { positions, quads }
}

#[allow(dead_code)]
pub fn cc2_subdivide(mesh: &CatmullClarkV2) -> CatmullClarkV2 {
    let mut new_positions = mesh.positions.clone();
    let mut new_quads: Vec<[u32; 4]> = Vec::new();

    for quad in &mesh.quads {
        let p0 = mesh.positions[quad[0] as usize];
        let p1 = mesh.positions[quad[1] as usize];
        let p2 = mesh.positions[quad[2] as usize];
        let p3 = mesh.positions[quad[3] as usize];

        // Face point
        let face_pt = [
            (p0[0] + p1[0] + p2[0] + p3[0]) * 0.25,
            (p0[1] + p1[1] + p2[1] + p3[1]) * 0.25,
            (p0[2] + p1[2] + p2[2] + p3[2]) * 0.25,
        ];
        let fi = new_positions.len() as u32;
        new_positions.push(face_pt);

        // Edge midpoints
        let e01 = midpoint(p0, p1);
        let e12 = midpoint(p1, p2);
        let e23 = midpoint(p2, p3);
        let e30 = midpoint(p3, p0);
        let e01i = new_positions.len() as u32;
        new_positions.push(e01);
        let e12i = new_positions.len() as u32;
        new_positions.push(e12);
        let e23i = new_positions.len() as u32;
        new_positions.push(e23);
        let e30i = new_positions.len() as u32;
        new_positions.push(e30);

        // 4 sub-quads
        new_quads.push([quad[0], e01i, fi, e30i]);
        new_quads.push([e01i, quad[1], e12i, fi]);
        new_quads.push([fi, e12i, quad[2], e23i]);
        new_quads.push([e30i, fi, e23i, quad[3]]);
    }

    CatmullClarkV2 { positions: new_positions, quads: new_quads }
}

fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5, (a[2] + b[2]) * 0.5]
}

#[allow(dead_code)]
pub fn cc2_vertex_count(m: &CatmullClarkV2) -> usize {
    m.positions.len()
}

#[allow(dead_code)]
pub fn cc2_quad_count(m: &CatmullClarkV2) -> usize {
    m.quads.len()
}

#[allow(dead_code)]
pub fn cc2_centroid(m: &CatmullClarkV2) -> [f32; 3] {
    if m.positions.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let n = m.positions.len() as f32;
    let sum = m.positions.iter().fold([0.0f32; 3], |acc, p| {
        [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
    });
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

#[allow(dead_code)]
pub fn cc2_bounds(m: &CatmullClarkV2) -> ([f32; 3], [f32; 3]) {
    if m.positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = m.positions[0];
    let mut mx = m.positions[0];
    for p in &m.positions {
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

    fn simple_quad() -> CatmullClarkV2 {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let quads = vec![[0, 1, 2, 3]];
        new_catmull_clark_v2(positions, quads)
    }

    #[test]
    fn test_vertex_count() {
        let m = simple_quad();
        assert_eq!(cc2_vertex_count(&m), 4);
    }

    #[test]
    fn test_quad_count() {
        let m = simple_quad();
        assert_eq!(cc2_quad_count(&m), 1);
    }

    #[test]
    fn test_subdivide_produces_more_quads() {
        let m = simple_quad();
        let sub = cc2_subdivide(&m);
        assert_eq!(cc2_quad_count(&sub), 4);
    }

    #[test]
    fn test_subdivide_vertex_count_increases() {
        let m = simple_quad();
        let sub = cc2_subdivide(&m);
        assert!(cc2_vertex_count(&sub) > cc2_vertex_count(&m));
    }

    #[test]
    fn test_centroid() {
        let m = simple_quad();
        let c = cc2_centroid(&m);
        assert!((c[0] - 0.5).abs() < 1e-5);
        assert!((c[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_bounds() {
        let m = simple_quad();
        let (lo, hi) = cc2_bounds(&m);
        assert_eq!(lo, [0.0, 0.0, 0.0]);
        assert_eq!(hi, [1.0, 1.0, 0.0]);
    }

    #[test]
    fn test_empty_centroid() {
        let m = new_catmull_clark_v2(vec![], vec![]);
        assert_eq!(cc2_centroid(&m), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_double_subdivide() {
        let m = simple_quad();
        let sub1 = cc2_subdivide(&m);
        let sub2 = cc2_subdivide(&sub1);
        assert_eq!(cc2_quad_count(&sub2), 16);
    }
}
