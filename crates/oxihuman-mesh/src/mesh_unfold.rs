// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Unfold a triangle mesh to a flat 2D net using a spanning tree approach.

use std::collections::HashMap;

/// Result of unfolding.
#[allow(dead_code)]
pub struct UnfoldResult {
    pub uvs: Vec<[f32; 2]>,
    pub face_visited: Vec<bool>,
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-9 {
        [1.0, 0.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

/// Place a triangle in 2D given one edge already placed.
#[allow(dead_code)]
pub fn place_triangle_2d(
    p0_3d: [f32; 3],
    p1_3d: [f32; 3],
    p2_3d: [f32; 3],
    uv0: [f32; 2],
    uv1: [f32; 2],
) -> [f32; 2] {
    let edge3d = sub3(p1_3d, p0_3d);
    let edge_len = len3(edge3d);
    let edge2d = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
    let edge2d_len = (edge2d[0] * edge2d[0] + edge2d[1] * edge2d[1]).sqrt();
    if edge2d_len < 1e-9 || edge_len < 1e-9 {
        return [uv0[0], uv0[1]];
    }
    let u_hat = [edge2d[0] / edge2d_len, edge2d[1] / edge2d_len];
    let v_hat = [-u_hat[1], u_hat[0]];
    let edge3d_n = normalize3(edge3d);
    let up3d = normalize3(cross3(edge3d, cross3(edge3d, sub3(p2_3d, p0_3d))));
    let v2 = sub3(p2_3d, p0_3d);
    let proj_u = dot3(v2, edge3d_n);
    let perp = sub3(
        v2,
        [
            edge3d_n[0] * proj_u,
            edge3d_n[1] * proj_u,
            edge3d_n[2] * proj_u,
        ],
    );
    let _ = up3d;
    let proj_v = len3(perp);
    let scale = edge2d_len / edge_len;
    [
        uv0[0] + u_hat[0] * proj_u * scale + v_hat[0] * proj_v * scale,
        uv0[1] + u_hat[1] * proj_u * scale + v_hat[1] * proj_v * scale,
    ]
}

/// Unfold mesh to 2D using BFS spanning tree over shared edges.
#[allow(dead_code)]
pub fn unfold_mesh(positions: &[[f32; 3]], indices: &[u32]) -> UnfoldResult {
    let n_verts = positions.len();
    let n_tri = indices.len() / 3;
    let mut uvs = vec![[f32::NAN, f32::NAN]; n_verts];
    let mut face_visited = vec![false; n_tri];
    if n_tri == 0 {
        return UnfoldResult { uvs, face_visited };
    }
    let mut edge_to_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for t in 0..n_tri {
        for e in 0..3 {
            let a = indices[t * 3 + e];
            let b = indices[t * 3 + (e + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_to_faces.entry(key).or_default().push(t);
        }
    }
    let i0 = indices[0] as usize;
    let i1 = indices[1] as usize;
    let i2 = indices[2] as usize;
    let p0 = positions[i0];
    let p1 = positions[i1];
    let edge_len = len3(sub3(p1, p0));
    uvs[i0] = [0.0, 0.0];
    uvs[i1] = [edge_len, 0.0];
    uvs[i2] = place_triangle_2d(p0, p1, positions[i2], uvs[i0], uvs[i1]);
    face_visited[0] = true;
    let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    queue.push_back(0);
    while let Some(t) = queue.pop_front() {
        for e in 0..3 {
            let a = indices[t * 3 + e];
            let b = indices[t * 3 + (e + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            if let Some(faces) = edge_to_faces.get(&key) {
                for &nb in faces {
                    if face_visited[nb] {
                        continue;
                    }
                    face_visited[nb] = true;
                    let uva = uvs[a as usize];
                    let uvb = uvs[b as usize];
                    if uva[0].is_nan() || uvb[0].is_nan() {
                        continue;
                    }
                    for v_local in 0..3 {
                        let vi = indices[nb * 3 + v_local] as usize;
                        if uvs[vi][0].is_nan() {
                            let pa = positions[a as usize];
                            let pb = positions[b as usize];
                            let pv = positions[vi];
                            uvs[vi] = place_triangle_2d(pa, pb, pv, uva, uvb);
                        }
                    }
                    queue.push_back(nb);
                }
            }
        }
    }
    UnfoldResult { uvs, face_visited }
}

/// Fraction of vertices assigned UV coordinates.
#[allow(dead_code)]
pub fn unfold_coverage(result: &UnfoldResult) -> f32 {
    let total = result.uvs.len();
    if total == 0 {
        return 0.0;
    }
    let assigned = result.uvs.iter().filter(|uv| !uv[0].is_nan()).count();
    assigned as f32 / total as f32
}

/// Number of faces visited during unfolding.
#[allow(dead_code)]
pub fn unfold_visited_count(result: &UnfoldResult) -> usize {
    result.face_visited.iter().filter(|&&v| v).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tri_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.5, 1.0, 0.0],
        ];
        let indices: Vec<u32> = vec![0, 1, 2, 1, 3, 2];
        (positions, indices)
    }

    #[test]
    fn unfold_visits_all_faces() {
        let (pos, idx) = two_tri_mesh();
        let result = unfold_mesh(&pos, &idx);
        assert_eq!(unfold_visited_count(&result), 2);
    }

    #[test]
    fn unfold_coverage_connected_mesh() {
        let (pos, idx) = two_tri_mesh();
        let result = unfold_mesh(&pos, &idx);
        let cov = unfold_coverage(&result);
        assert!(cov > 0.5);
    }

    #[test]
    fn unfold_first_vertex_at_origin() {
        let (pos, idx) = two_tri_mesh();
        let result = unfold_mesh(&pos, &idx);
        let uv0 = result.uvs[0];
        assert!(!uv0[0].is_nan());
        assert!((uv0[0] - 0.0).abs() < 1e-5 && (uv0[1] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn unfold_second_vertex_on_x_axis() {
        let (pos, idx) = two_tri_mesh();
        let result = unfold_mesh(&pos, &idx);
        let uv1 = result.uvs[1];
        assert!(!uv1[0].is_nan());
        assert!(uv1[1].abs() < 1e-5);
    }

    #[test]
    fn unfold_coverage_full_planar() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices: Vec<u32> = vec![0, 1, 2];
        let result = unfold_mesh(&positions, &indices);
        assert!((unfold_coverage(&result) - 1.0).abs() < 0.01);
    }

    #[test]
    fn unfold_empty_mesh() {
        let result = unfold_mesh(&[], &[]);
        assert_eq!(unfold_visited_count(&result), 0);
    }

    #[test]
    fn place_triangle_2d_nonoverlap() {
        let p0 = [0.0, 0.0, 0.0];
        let p1 = [1.0, 0.0, 0.0];
        let p2 = [0.0, 1.0, 0.0];
        let uv2 = place_triangle_2d(p0, p1, p2, [0.0, 0.0], [1.0, 0.0]);
        assert!(!uv2[0].is_nan() && !uv2[1].is_nan());
    }

    #[test]
    fn unfold_visited_count_single_tri() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let indices: Vec<u32> = vec![0, 1, 2];
        let result = unfold_mesh(&positions, &indices);
        assert_eq!(unfold_visited_count(&result), 1);
    }

    #[test]
    fn unfold_coverage_nonnegative() {
        let result = UnfoldResult {
            uvs: vec![[f32::NAN, f32::NAN]; 4],
            face_visited: vec![false; 2],
        };
        assert_eq!(unfold_coverage(&result), 0.0);
    }
}
