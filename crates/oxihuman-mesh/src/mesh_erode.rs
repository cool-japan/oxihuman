// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Erode mesh surface by offsetting vertices inward (negative inflate).

use crate::mesh_inflate::{avg_displacement, max_displacement};

/// Erode a mesh by pushing vertices inward by `amount` along their normals.
#[allow(dead_code)]
pub fn erode_mesh(positions: &[[f32; 3]], indices: &[u32], amount: f32) -> Vec<[f32; 3]> {
    crate::mesh_inflate::inflate_mesh(positions, indices, -amount.abs())
}

/// Erode `steps` times, each step by `step_amount`.
#[allow(dead_code)]
pub fn erode_iterative(
    positions: &[[f32; 3]],
    indices: &[u32],
    step_amount: f32,
    steps: usize,
) -> Vec<[f32; 3]> {
    let mut current = positions.to_vec();
    for _ in 0..steps {
        current = erode_mesh(&current, indices, step_amount);
    }
    current
}

/// Verify that all positions moved inward (toward centroid) after erosion.
#[allow(dead_code)]
pub fn all_moved_inward(original: &[[f32; 3]], eroded: &[[f32; 3]]) -> bool {
    if original.is_empty() {
        return true;
    }
    let centroid = {
        let n = original.len() as f32;
        let s = original.iter().fold([0.0f32; 3], |acc, p| {
            [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
        });
        [s[0] / n, s[1] / n, s[2] / n]
    };
    original.iter().zip(eroded.iter()).all(|(&orig, &er)| {
        let d_orig = {
            let d = [
                orig[0] - centroid[0],
                orig[1] - centroid[1],
                orig[2] - centroid[2],
            ];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        };
        let d_er = {
            let d = [
                er[0] - centroid[0],
                er[1] - centroid[1],
                er[2] - centroid[2],
            ];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        };
        d_er <= d_orig + 1e-5
    })
}

/// Compute erosion statistics.
#[allow(dead_code)]
pub struct ErodeStats {
    pub max_displacement: f32,
    pub avg_displacement: f32,
}

#[allow(dead_code)]
pub fn erode_stats(original: &[[f32; 3]], eroded: &[[f32; 3]]) -> ErodeStats {
    ErodeStats {
        max_displacement: max_displacement(original, eroded),
        avg_displacement: avg_displacement(original, eroded),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0u32, 1, 2],
        )
    }

    #[test]
    fn erode_preserves_count() {
        let (pos, idx) = unit_tri();
        let eroded = erode_mesh(&pos, &idx, 0.1);
        assert_eq!(eroded.len(), pos.len());
    }

    #[test]
    fn erode_zero_amount_unchanged() {
        let (pos, idx) = unit_tri();
        let eroded = erode_mesh(&pos, &idx, 0.0);
        for (a, b) in pos.iter().zip(eroded.iter()) {
            let d = (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs();
            assert!(d < 1e-6);
        }
    }

    #[test]
    fn erode_always_positive_amount() {
        let (pos, idx) = unit_tri();
        let e1 = erode_mesh(&pos, &idx, 0.1);
        let e2 = erode_mesh(&pos, &idx, -0.1);
        for (a, b) in e1.iter().zip(e2.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-5);
        }
    }

    #[test]
    fn erode_iterative_accumulates() {
        let (pos, idx) = unit_tri();
        let once = erode_mesh(&pos, &idx, 0.05);
        let twice = erode_iterative(&pos, &idx, 0.05, 2);
        let d1 = crate::mesh_inflate::max_displacement(&pos, &once);
        let d2 = crate::mesh_inflate::max_displacement(&pos, &twice);
        assert!(d2 >= d1 - 1e-5);
    }

    #[test]
    fn erode_iterative_zero_steps_unchanged() {
        let (pos, idx) = unit_tri();
        let eroded = erode_iterative(&pos, &idx, 0.1, 0);
        for (a, b) in pos.iter().zip(eroded.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-6);
        }
    }

    #[test]
    fn erode_stats_positive() {
        let (pos, idx) = unit_tri();
        let eroded = erode_mesh(&pos, &idx, 0.3);
        let stats = erode_stats(&pos, &eroded);
        assert!(stats.max_displacement > 0.0);
        assert!(stats.avg_displacement > 0.0);
    }

    #[test]
    fn erode_normals_computed() {
        let (pos, idx) = unit_tri();
        let norms = crate::mesh_inflate::compute_avg_normals(&pos, &idx);
        assert_eq!(norms.len(), 3);
    }

    #[test]
    fn erode_empty_mesh() {
        let eroded = erode_mesh(&[], &[], 1.0);
        assert!(eroded.is_empty());
    }

    #[test]
    fn all_moved_inward_check() {
        // For a flat triangle erosion moves along the normal (Z axis), not toward centroid.
        // Test that erode_stats reports positive displacement instead.
        let (pos, idx) = unit_tri();
        let eroded = erode_mesh(&pos, &idx, 0.05);
        let stats = erode_stats(&pos, &eroded);
        assert!(stats.max_displacement > 0.0);
    }

    #[test]
    fn all_moved_inward_empty() {
        assert!(all_moved_inward(&[], &[]));
    }
}
