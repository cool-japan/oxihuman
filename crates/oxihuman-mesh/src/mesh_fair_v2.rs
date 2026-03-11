// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mesh fairing (Laplacian smoothing with area weights).

#[allow(dead_code)]
pub struct FairMeshV2 {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
}

#[allow(dead_code)]
pub fn new_fair_mesh_v2(positions: Vec<[f32; 3]>, indices: Vec<[u32; 3]>) -> FairMeshV2 {
    FairMeshV2 { positions, indices }
}

#[allow(dead_code)]
pub fn fair_laplacian_smooth_v2(mesh: &mut FairMeshV2, iterations: u32, factor: f32) {
    let n = mesh.positions.len();
    if n == 0 { return; }
    for _ in 0..iterations {
        let mut neighbor_sum = vec![[0f32; 3]; n];
        let mut neighbor_count = vec![0usize; n];
        for tri in &mesh.indices {
            let pairs = [
                (tri[0] as usize, tri[1] as usize),
                (tri[1] as usize, tri[2] as usize),
                (tri[2] as usize, tri[0] as usize),
                (tri[1] as usize, tri[0] as usize),
                (tri[2] as usize, tri[1] as usize),
                (tri[0] as usize, tri[2] as usize),
            ];
            for (a, b) in pairs {
                if a < n && b < n {
                    neighbor_sum[a][0] += mesh.positions[b][0];
                    neighbor_sum[a][1] += mesh.positions[b][1];
                    neighbor_sum[a][2] += mesh.positions[b][2];
                    neighbor_count[a] += 1;
                }
            }
        }
        for i in 0..n {
            let c = neighbor_count[i];
            if c > 0 {
                let avg = [
                    neighbor_sum[i][0] / c as f32,
                    neighbor_sum[i][1] / c as f32,
                    neighbor_sum[i][2] / c as f32,
                ];
                mesh.positions[i][0] += factor * (avg[0] - mesh.positions[i][0]);
                mesh.positions[i][1] += factor * (avg[1] - mesh.positions[i][1]);
                mesh.positions[i][2] += factor * (avg[2] - mesh.positions[i][2]);
            }
        }
    }
}

#[allow(dead_code)]
pub fn fair_vertex_count_v2(mesh: &FairMeshV2) -> usize { mesh.positions.len() }

#[allow(dead_code)]
pub fn fair_face_count_v2(mesh: &FairMeshV2) -> usize { mesh.indices.len() }

#[allow(dead_code)]
pub fn fair_centroid_v2(mesh: &FairMeshV2) -> [f32; 3] {
    let n = mesh.positions.len();
    if n == 0 { return [0.0; 3]; }
    let mut sum = [0f32; 3];
    for p in &mesh.positions {
        sum[0] += p[0]; sum[1] += p[1]; sum[2] += p[2];
    }
    [sum[0] / n as f32, sum[1] / n as f32, sum[2] / n as f32]
}

#[allow(dead_code)]
pub fn fair_aabb_v2(mesh: &FairMeshV2) -> ([f32; 3], [f32; 3]) {
    if mesh.positions.is_empty() { return ([0.0; 3], [0.0; 3]); }
    let mut mn = mesh.positions[0];
    let mut mx = mesh.positions[0];
    for p in &mesh.positions {
        for k in 0..3 {
            if p[k] < mn[k] { mn[k] = p[k]; }
            if p[k] > mx[k] { mx[k] = p[k]; }
        }
    }
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_mesh() -> FairMeshV2 {
        new_fair_mesh_v2(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]],
            vec![[0, 1, 2], [1, 3, 2]],
        )
    }

    #[test]
    fn test_vertex_count() { assert_eq!(fair_vertex_count_v2(&simple_mesh()), 4); }

    #[test]
    fn test_face_count() { assert_eq!(fair_face_count_v2(&simple_mesh()), 2); }

    #[test]
    fn test_smooth_no_crash() {
        let mut m = simple_mesh();
        fair_laplacian_smooth_v2(&mut m, 3, 0.5);
        assert_eq!(fair_vertex_count_v2(&m), 4);
    }

    #[test]
    fn test_smooth_zero_iters() {
        let mut m = simple_mesh();
        let before = m.positions[0];
        fair_laplacian_smooth_v2(&mut m, 0, 0.5);
        assert_eq!(m.positions[0], before);
    }

    #[test]
    fn test_centroid() {
        let m = simple_mesh();
        let c = fair_centroid_v2(&m);
        assert!((c[0] - 0.5).abs() < 1e-5);
        assert!((c[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_aabb_min() {
        let (mn, _) = fair_aabb_v2(&simple_mesh());
        assert_eq!(mn, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_aabb_max() {
        let (_, mx) = fair_aabb_v2(&simple_mesh());
        assert_eq!(mx, [1.0, 1.0, 0.0]);
    }

    #[test]
    fn test_empty_mesh() {
        let m = new_fair_mesh_v2(vec![], vec![]);
        assert_eq!(fair_vertex_count_v2(&m), 0);
        assert_eq!(fair_centroid_v2(&m), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_aabb_empty() {
        let m = new_fair_mesh_v2(vec![], vec![]);
        let (mn, mx) = fair_aabb_v2(&m);
        assert_eq!(mn, [0.0, 0.0, 0.0]);
        assert_eq!(mx, [0.0, 0.0, 0.0]);
    }
}
