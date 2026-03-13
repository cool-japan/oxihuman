// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Inflate / erode mesh by pushing vertices along averaged normals.

/// Push every vertex outward along its averaged vertex normal by `amount`.
/// Positive `amount` inflates; negative erodes.
#[allow(dead_code)]
pub fn inflate_mesh(positions: &[[f32; 3]], indices: &[u32], amount: f32) -> Vec<[f32; 3]> {
    let normals = compute_avg_normals(positions, indices);
    positions
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            let n = normals[i];
            [
                p[0] + n[0] * amount,
                p[1] + n[1] * amount,
                p[2] + n[2] * amount,
            ]
        })
        .collect()
}

/// Maximum outward displacement magnitude (for inflation validation).
#[allow(dead_code)]
pub fn max_displacement(original: &[[f32; 3]], inflated: &[[f32; 3]]) -> f32 {
    original
        .iter()
        .zip(inflated.iter())
        .map(|(&a, &b)| {
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// Average displacement magnitude.
#[allow(dead_code)]
pub fn avg_displacement(original: &[[f32; 3]], inflated: &[[f32; 3]]) -> f32 {
    if original.is_empty() {
        return 0.0;
    }
    let sum: f32 = original
        .iter()
        .zip(inflated.iter())
        .map(|(&a, &b)| {
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .sum();
    sum / original.len() as f32
}

/// Compute smooth per-vertex normals (area-weighted average of face normals).
#[allow(dead_code)]
pub fn compute_avg_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut acc = vec![[0.0f32; 3]; n];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let pa = positions[a];
        let pb = positions[b];
        let pc = positions[c];
        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let n3 = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        for &i in &[a, b, c] {
            acc[i][0] += n3[0];
            acc[i][1] += n3[1];
            acc[i][2] += n3[2];
        }
    }
    acc.iter()
        .map(|&v| {
            let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            if l < 1e-8 {
                [0.0, 1.0, 0.0]
            } else {
                [v[0] / l, v[1] / l, v[2] / l]
            }
        })
        .collect()
}

/// Check that all inflated positions moved outward (positive amount).
#[allow(dead_code)]
pub fn all_moved_outward(
    original: &[[f32; 3]],
    inflated: &[[f32; 3]],
    normals: &[[f32; 3]],
) -> bool {
    original
        .iter()
        .zip(inflated.iter())
        .zip(normals.iter())
        .all(|((&orig, &inf), &n)| {
            let d = [inf[0] - orig[0], inf[1] - orig[1], inf[2] - orig[2]];
            d[0] * n[0] + d[1] * n[1] + d[2] * n[2] >= -1e-6
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0u32, 1, 2],
        )
    }

    #[test]
    fn inflate_preserves_count() {
        let (pos, idx) = flat_tri();
        let inflated = inflate_mesh(&pos, &idx, 0.1);
        assert_eq!(inflated.len(), pos.len());
    }

    #[test]
    fn inflate_zero_amount_unchanged() {
        let (pos, idx) = flat_tri();
        let inflated = inflate_mesh(&pos, &idx, 0.0);
        for (a, b) in pos.iter().zip(inflated.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-6);
            assert!((a[1] - b[1]).abs() < 1e-6);
            assert!((a[2] - b[2]).abs() < 1e-6);
        }
    }

    #[test]
    fn max_displacement_approx_amount() {
        let (pos, idx) = flat_tri();
        let inflated = inflate_mesh(&pos, &idx, 0.5);
        let d = max_displacement(&pos, &inflated);
        assert!((d - 0.5).abs() < 0.01);
    }

    #[test]
    fn avg_displacement_nonzero() {
        let (pos, idx) = flat_tri();
        let inflated = inflate_mesh(&pos, &idx, 0.3);
        let d = avg_displacement(&pos, &inflated);
        assert!(d > 0.0);
    }

    #[test]
    fn avg_displacement_empty() {
        assert_eq!(avg_displacement(&[], &[]), 0.0);
    }

    #[test]
    fn normals_unit_length() {
        let (pos, idx) = flat_tri();
        let norms = compute_avg_normals(&pos, &idx);
        for n in &norms {
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((l - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn inflate_outward() {
        let (pos, idx) = flat_tri();
        let normals = compute_avg_normals(&pos, &idx);
        let inflated = inflate_mesh(&pos, &idx, 0.2);
        assert!(all_moved_outward(&pos, &inflated, &normals));
    }

    #[test]
    fn deflate_inward() {
        let (pos, idx) = flat_tri();
        let normals = compute_avg_normals(&pos, &idx);
        let neg_normals: Vec<[f32; 3]> = normals.iter().map(|&n| [-n[0], -n[1], -n[2]]).collect();
        let deflated = inflate_mesh(&pos, &idx, -0.2);
        assert!(all_moved_outward(&pos, &deflated, &neg_normals));
    }

    #[test]
    fn inflate_empty_mesh() {
        let inflated = inflate_mesh(&[], &[], 1.0);
        assert!(inflated.is_empty());
    }

    #[test]
    fn max_displacement_zero() {
        let pos = vec![[0.0, 0.0, 0.0]];
        assert!((max_displacement(&pos, &pos) - 0.0).abs() < 1e-6);
    }
}
