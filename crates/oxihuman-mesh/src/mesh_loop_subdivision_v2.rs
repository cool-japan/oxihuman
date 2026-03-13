// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Loop subdivision v2 for triangles.

#[allow(dead_code)]
pub struct LoopSubdivV2 {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

#[allow(dead_code)]
pub fn new_loop_subdiv_v2(positions: Vec<[f32; 3]>, triangles: Vec<[u32; 3]>) -> LoopSubdivV2 {
    LoopSubdivV2 { positions, triangles }
}

#[allow(dead_code)]
pub fn ls2_subdivide(mesh: &LoopSubdivV2) -> LoopSubdivV2 {
    let mut new_positions = mesh.positions.clone();
    let mut new_triangles: Vec<[u32; 3]> = Vec::new();

    for tri in &mesh.triangles {
        let p0 = mesh.positions[tri[0] as usize];
        let p1 = mesh.positions[tri[1] as usize];
        let p2 = mesh.positions[tri[2] as usize];

        let m01 = midpoint(p0, p1);
        let m12 = midpoint(p1, p2);
        let m20 = midpoint(p2, p0);

        let m01i = new_positions.len() as u32;
        new_positions.push(m01);
        let m12i = new_positions.len() as u32;
        new_positions.push(m12);
        let m20i = new_positions.len() as u32;
        new_positions.push(m20);

        new_triangles.push([tri[0], m01i, m20i]);
        new_triangles.push([m01i, tri[1], m12i]);
        new_triangles.push([m20i, m12i, tri[2]]);
        new_triangles.push([m01i, m12i, m20i]);
    }

    LoopSubdivV2 { positions: new_positions, triangles: new_triangles }
}

fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5, (a[2] + b[2]) * 0.5]
}

#[allow(dead_code)]
pub fn ls2_vertex_count(m: &LoopSubdivV2) -> usize {
    m.positions.len()
}

#[allow(dead_code)]
pub fn ls2_triangle_count(m: &LoopSubdivV2) -> usize {
    m.triangles.len()
}

#[allow(dead_code)]
pub fn ls2_centroid(m: &LoopSubdivV2) -> [f32; 3] {
    if m.positions.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let n = m.positions.len() as f32;
    let sum = m.positions.iter().fold([0.0f32; 3], |acc, p| {
        [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
    });
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_tri() -> LoopSubdivV2 {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ];
        let triangles = vec![[0, 1, 2]];
        new_loop_subdiv_v2(positions, triangles)
    }

    #[test]
    fn test_vertex_count() {
        let m = simple_tri();
        assert_eq!(ls2_vertex_count(&m), 3);
    }

    #[test]
    fn test_triangle_count() {
        let m = simple_tri();
        assert_eq!(ls2_triangle_count(&m), 1);
    }

    #[test]
    fn test_subdivide_multiplies_tris_by_4() {
        let m = simple_tri();
        let sub = ls2_subdivide(&m);
        assert_eq!(ls2_triangle_count(&sub), 4);
    }

    #[test]
    fn test_subdivide_twice() {
        let m = simple_tri();
        let sub1 = ls2_subdivide(&m);
        let sub2 = ls2_subdivide(&sub1);
        assert_eq!(ls2_triangle_count(&sub2), 16);
    }

    #[test]
    fn test_centroid() {
        let m = simple_tri();
        let c = ls2_centroid(&m);
        assert!((c[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_centroid_empty() {
        let m = new_loop_subdiv_v2(vec![], vec![]);
        assert_eq!(ls2_centroid(&m), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_subdivide_vertex_count() {
        let m = simple_tri();
        let sub = ls2_subdivide(&m);
        assert_eq!(ls2_vertex_count(&sub), 6); /* 3 original + 3 edge midpoints */
    }

    #[test]
    fn test_new() {
        let m = new_loop_subdiv_v2(vec![[0.0, 0.0, 0.0]], vec![]);
        assert_eq!(ls2_vertex_count(&m), 1);
    }
}
