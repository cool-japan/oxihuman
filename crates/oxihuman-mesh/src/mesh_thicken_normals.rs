// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub fn thicken_vertex(pos: [f32; 3], normal: [f32; 3], amount: f32) -> [f32; 3] {
    [
        pos[0] + normal[0] * amount,
        pos[1] + normal[1] * amount,
        pos[2] + normal[2] * amount,
    ]
}

pub fn thicken_mesh_vertices(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    amount: f32,
) -> Vec<[f32; 3]> {
    positions
        .iter()
        .zip(normals.iter())
        .map(|(&p, &n)| thicken_vertex(p, n, amount))
        .collect()
}

pub fn thicken_average_normal(normals: &[[f32; 3]]) -> [f32; 3] {
    let n = normals.len();
    if n == 0 {
        return [0.0, 1.0, 0.0];
    }
    let sum = normals.iter().fold([0.0f32; 3], |acc, &v| {
        [acc[0] + v[0], acc[1] + v[1], acc[2] + v[2]]
    });
    let avg = [sum[0] / n as f32, sum[1] / n as f32, sum[2] / n as f32];
    let len = (avg[0] * avg[0] + avg[1] * avg[1] + avg[2] * avg[2])
        .sqrt()
        .max(1e-9);
    [avg[0] / len, avg[1] / len, avg[2] / len]
}

pub fn thicken_is_normalized(n: [f32; 3]) -> bool {
    let len_sq = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
    (len_sq - 1.0).abs() < 1e-4
}

pub fn thicken_boundary_vertex(
    pos: [f32; 3],
    normal: [f32; 3],
    tangent: [f32; 3],
    amount: f32,
) -> [f32; 3] {
    // Move along the bisector of normal and tangent
    let bx = normal[0] + tangent[0];
    let by = normal[1] + tangent[1];
    let bz = normal[2] + tangent[2];
    let len = (bx * bx + by * by + bz * bz).sqrt().max(1e-9);
    let bx = bx / len;
    let by = by / len;
    let bz = bz / len;
    [
        pos[0] + bx * amount,
        pos[1] + by * amount,
        pos[2] + bz * amount,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thicken_vertex() {
        let p = [0.0, 0.0, 0.0];
        let n = [0.0, 1.0, 0.0];
        let out = thicken_vertex(p, n, 2.0);
        assert!((out[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_thicken_mesh_vertices_count() {
        let pos = vec![[0.0; 3]; 5];
        let nrm = vec![[0.0, 1.0, 0.0]; 5];
        let out = thicken_mesh_vertices(&pos, &nrm, 1.0);
        assert_eq!(out.len(), 5);
    }

    #[test]
    fn test_thicken_average_normal_upward() {
        let normals = vec![[0.0, 1.0, 0.0]; 4];
        let avg = thicken_average_normal(&normals);
        assert!((avg[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_thicken_is_normalized_true() {
        assert!(thicken_is_normalized([0.0, 1.0, 0.0]));
    }

    #[test]
    fn test_thicken_is_normalized_false() {
        assert!(!thicken_is_normalized([1.0, 1.0, 1.0]));
    }

    #[test]
    fn test_thicken_boundary_vertex_moves() {
        let out = thicken_boundary_vertex([0.0; 3], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], 1.0);
        /* should move in the y=1 x=1 direction (normalized) */
        assert!(out[0] > 0.0 && out[1] > 0.0);
    }
}
