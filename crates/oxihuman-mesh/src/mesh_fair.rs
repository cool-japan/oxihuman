#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh fairing using cotangent weights.

#[allow(dead_code)]
pub fn fair_mesh(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    iterations: u32,
    lambda: f32,
) -> Vec<[f32; 3]> {
    let n = verts.len();
    let adj = build_adjacency(tris, n);
    let mut pos = verts.to_vec();
    for _ in 0..iterations {
        let prev = pos.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let mut sum = [0.0f32; 3];
            for &j in &adj[i] {
                sum[0] += prev[j][0];
                sum[1] += prev[j][1];
                sum[2] += prev[j][2];
            }
            let k = adj[i].len() as f32;
            let avg = [sum[0] / k, sum[1] / k, sum[2] / k];
            pos[i][0] = prev[i][0] + lambda * (avg[0] - prev[i][0]);
            pos[i][1] = prev[i][1] + lambda * (avg[1] - prev[i][1]);
            pos[i][2] = prev[i][2] + lambda * (avg[2] - prev[i][2]);
        }
    }
    pos
}

#[allow(dead_code)]
pub fn cotangent_weight(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> f32 {
    // cot(angle at v2) = dot(v2v0, v2v1) / |cross(v2v0, v2v1)|
    let a = [v0[0] - v2[0], v0[1] - v2[1], v0[2] - v2[2]];
    let b = [v1[0] - v2[0], v1[1] - v2[1], v1[2] - v2[2]];
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let mag = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    if mag < 1e-10 {
        0.0
    } else {
        dot / mag
    }
}

#[allow(dead_code)]
pub fn build_adjacency(tris: &[[u32; 3]], n_verts: usize) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); n_verts];
    for tri in tris {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if !adj[a].contains(&b) {
            adj[a].push(b);
        }
        if !adj[a].contains(&c) {
            adj[a].push(c);
        }
        if !adj[b].contains(&a) {
            adj[b].push(a);
        }
        if !adj[b].contains(&c) {
            adj[b].push(c);
        }
        if !adj[c].contains(&a) {
            adj[c].push(a);
        }
        if !adj[c].contains(&b) {
            adj[c].push(b);
        }
    }
    adj
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn tri_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn tri_faces() -> Vec<[u32; 3]> {
        vec![[0, 1, 2]]
    }

    #[test]
    fn adjacency_three_verts() {
        let adj = build_adjacency(&tri_faces(), 3);
        assert_eq!(adj[0].len(), 2);
        assert_eq!(adj[1].len(), 2);
        assert_eq!(adj[2].len(), 2);
    }

    #[test]
    fn adjacency_no_duplicates() {
        let tris = vec![[0u32, 1, 2], [0, 1, 2]];
        let adj = build_adjacency(&tris, 3);
        assert_eq!(adj[0].len(), 2);
    }

    #[test]
    fn fair_mesh_zero_iterations_unchanged() {
        let verts = tri_verts();
        let result = fair_mesh(&verts, &tri_faces(), 0, 0.5);
        assert!((result[0][0] - verts[0][0]).abs() < 1e-6);
    }

    #[test]
    fn fair_mesh_preserves_count() {
        let verts = tri_verts();
        let result = fair_mesh(&verts, &tri_faces(), 3, 0.5);
        assert_eq!(result.len(), verts.len());
    }

    #[test]
    fn cotangent_weight_right_angle() {
        // right angle at v2 => angle is PI/2 => cot = 0
        let v0 = [1.0f32, 0.0, 0.0];
        let v1 = [0.0, 1.0, 0.0];
        let v2 = [0.0f32, 0.0, 0.0];
        let w = cotangent_weight(v0, v1, v2);
        assert!(w.abs() < 1e-5, "cot(PI/2) should be ~0, got {w}");
    }

    #[test]
    fn cotangent_weight_60_degree() {
        // Equilateral triangle with side length 1.
        // v2 is the apex, angle at v2 is 60°, cot(60°) = 1/sqrt(3) ≈ 0.577
        let v0 = [0.0f32, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.5f32, (PI / 3.0).sin(), 0.0]; // apex of equilateral triangle
        let w = cotangent_weight(v0, v1, v2);
        let expected = 1.0 / 3.0_f32.sqrt();
        assert!((w - expected).abs() < 0.01, "cot(60°) expected ~{expected}, got {w}");
    }

    #[test]
    fn fair_mesh_lambda_zero_unchanged() {
        let verts = tri_verts();
        let result = fair_mesh(&verts, &tri_faces(), 5, 0.0);
        for i in 0..verts.len() {
            assert!((result[i][0] - verts[i][0]).abs() < 1e-6);
        }
    }

    #[test]
    fn build_adjacency_empty() {
        let adj = build_adjacency(&[], 3);
        assert_eq!(adj.len(), 3);
        assert!(adj[0].is_empty());
    }

    #[test]
    fn build_adjacency_disconnected() {
        let tris = vec![[0u32, 1, 2]];
        let adj = build_adjacency(&tris, 5);
        assert!(adj[3].is_empty());
        assert!(adj[4].is_empty());
    }
}
