// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Smooth normal computation v2 with weighted averaging.

#[allow(dead_code)]
pub fn sn2_compute_face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    sn2_normalize(n)
}

#[allow(dead_code)]
pub fn sn2_compute_vertex_normals(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let mut accum = vec![[0.0f32; 3]; positions.len()];
    for tri in indices {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];
        let ab = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let ac = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let n = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        for &idx in &[i0, i1, i2] {
            accum[idx][0] += n[0];
            accum[idx][1] += n[1];
            accum[idx][2] += n[2];
        }
    }
    accum.iter().map(|&n| sn2_normalize(n)).collect()
}

#[allow(dead_code)]
pub fn sn2_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

#[allow(dead_code)]
pub fn sn2_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
pub fn sn2_average_normal(normals: &[[f32; 3]]) -> [f32; 3] {
    if normals.is_empty() {
        return [0.0, 1.0, 0.0];
    }
    let n = normals.len() as f32;
    let sum = normals.iter().fold([0.0f32; 3], |acc, &v| {
        [acc[0] + v[0], acc[1] + v[1], acc[2] + v[2]]
    });
    sn2_normalize([sum[0] / n, sum[1] / n, sum[2] / n])
}

#[allow(dead_code)]
pub fn sn2_flip_normals(normals: &mut [[f32; 3]]) {
    for n in normals.iter_mut() {
        n[0] = -n[0];
        n[1] = -n[1];
        n[2] = -n[2];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_normal_known_triangle() {
        let n = sn2_compute_face_normal(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!((n[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_vertex_normals_count() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![[0u32, 1, 2]];
        let normals = sn2_compute_vertex_normals(&positions, &indices);
        assert_eq!(normals.len(), 3);
    }

    #[test]
    fn test_normalize() {
        let n = sn2_normalize([3.0, 4.0, 0.0]);
        assert!((sn2_dot(n, n) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_dot() {
        let a = [1.0, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        assert!((sn2_dot(a, b)).abs() < 1e-10);
    }

    #[test]
    fn test_average_normal() {
        let normals = vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let avg = sn2_average_normal(&normals);
        assert!((avg[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_flip_normals() {
        let mut normals = vec![[0.0, 1.0, 0.0]];
        sn2_flip_normals(&mut normals);
        assert!((normals[0][1] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_zero_length() {
        let n = sn2_normalize([0.0, 0.0, 0.0]);
        assert_eq!(n, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_average_normal_empty() {
        let avg = sn2_average_normal(&[]);
        assert_eq!(avg, [0.0, 1.0, 0.0]);
    }
}
