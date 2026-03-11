// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Per-vertex and per-face normal computation utilities.

#[allow(unused_imports)]
use std::f32::consts::PI;

/// Compute flat (per-face) normals for a triangle mesh.
#[allow(dead_code)]
pub fn compute_flat_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let nf = indices.len() / 3;
    let mut normals = Vec::with_capacity(nf);
    for tri in indices.chunks_exact(3) {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = cross3(ab, ac);
        normals.push(normalize3(n));
    }
    normals
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0].powi(2) + v[1].powi(2) + v[2].powi(2)).sqrt();
    if l < 1e-9 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Compute smooth (area-weighted per-vertex) normals.
#[allow(dead_code)]
pub fn compute_smooth_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut acc = vec![[0.0_f32; 3]; n];
    for tri in indices.chunks_exact(3) {
        let (ia, ib, ic) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let a = positions[ia];
        let b = positions[ib];
        let c = positions[ic];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let face_n = cross3(ab, ac);
        for &i in &[ia, ib, ic] {
            acc[i][0] += face_n[0];
            acc[i][1] += face_n[1];
            acc[i][2] += face_n[2];
        }
    }
    acc.into_iter().map(normalize3).collect()
}

/// Flip all normals (reverse direction).
#[allow(dead_code)]
pub fn flip_normals(normals: &mut [[f32; 3]]) {
    for n in normals.iter_mut() {
        n[0] = -n[0];
        n[1] = -n[1];
        n[2] = -n[2];
    }
}

/// Check if normals are approximately unit length.
#[allow(dead_code)]
pub fn normals_are_unit(normals: &[[f32; 3]], tol: f32) -> bool {
    normals.iter().all(|n| {
        let l = (n[0].powi(2) + n[1].powi(2) + n[2].powi(2)).sqrt();
        (l - 1.0).abs() < tol
    })
}

/// Compute the average normal direction.
#[allow(dead_code)]
pub fn average_normal(normals: &[[f32; 3]]) -> [f32; 3] {
    if normals.is_empty() {
        return [0.0, 0.0, 1.0];
    }
    let n = normals.len() as f32;
    let s = normals
        .iter()
        .fold([0.0_f32; 3], |a, v| [a[0] + v[0], a[1] + v[1], a[2] + v[2]]);
    normalize3([s[0] / n, s[1] / n, s[2] / n])
}

/// Count normals pointing away from a given view direction.
#[allow(dead_code)]
pub fn count_back_facing(normals: &[[f32; 3]], view: [f32; 3]) -> usize {
    let view = normalize3(view);
    normals.iter().filter(|&&n| dot3(n, view) < 0.0).count()
}

/// Compute angle (radians) between two normals.
#[allow(dead_code)]
pub fn normal_angle(a: [f32; 3], b: [f32; 3]) -> f32 {
    dot3(a, b).clamp(-1.0, 1.0).acos()
}

/// Detect degenerate normals (near-zero length before normalization).
#[allow(dead_code)]
pub fn degenerate_normal_count(positions: &[[f32; 3]], indices: &[u32]) -> usize {
    indices
        .chunks_exact(3)
        .filter(|tri| {
            let a = positions[tri[0] as usize];
            let b = positions[tri[1] as usize];
            let c = positions[tri[2] as usize];
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = cross3(ab, ac);
            (n[0].powi(2) + n[1].powi(2) + n[2].powi(2)).sqrt() < 1e-9
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0u32, 1, 2],
        )
    }

    #[test]
    fn flat_normal_points_up() {
        let (p, i) = single_tri();
        let n = compute_flat_normals(&p, &i);
        assert!((n[0][2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn smooth_normals_unit_length() {
        let (p, i) = single_tri();
        let n = compute_smooth_normals(&p, &i);
        assert!(normals_are_unit(&n, 1e-5));
    }

    #[test]
    fn flip_reverses_direction() {
        let (p, i) = single_tri();
        let mut n = compute_flat_normals(&p, &i);
        flip_normals(&mut n);
        assert!(n[0][2] < 0.0);
    }

    #[test]
    fn average_normal_unit() {
        let n = vec![[0.0f32, 0.0, 1.0], [0.0, 0.0, 1.0]];
        let avg = average_normal(&n);
        let l = (avg[0].powi(2) + avg[1].powi(2) + avg[2].powi(2)).sqrt();
        assert!((l - 1.0).abs() < 1e-5);
    }

    #[test]
    fn count_back_facing_test() {
        let n = vec![[0.0f32, 0.0, 1.0], [0.0, 0.0, -1.0]];
        let cnt = count_back_facing(&n, [0.0, 0.0, 1.0]);
        assert_eq!(cnt, 1);
    }

    #[test]
    fn normal_angle_orthogonal() {
        let a = [1.0f32, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        let ang = normal_angle(a, b);
        assert!((ang - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn degenerate_count_zero_normal() {
        let (p, i) = single_tri();
        assert_eq!(degenerate_normal_count(&p, &i), 0);
    }

    #[test]
    fn flat_and_smooth_agree_on_plane() {
        let (p, i) = single_tri();
        let flat = compute_flat_normals(&p, &i);
        let smooth = compute_smooth_normals(&p, &i);
        // All vertices on same plane, both should point same direction
        for s in &smooth {
            assert!((s[2] - flat[0][2]).abs() < 1e-5);
        }
    }

    #[test]
    fn pi_constant_used() {
        // Verify the PI constant is accessible and has expected magnitude
        assert!((PI - std::f32::consts::PI).abs() < 1e-6);
    }

    #[test]
    fn empty_positions() {
        let n = compute_smooth_normals(&[], &[]);
        assert!(n.is_empty());
    }
}
