// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sort vertices along a chosen axis and produce a remapped index buffer.

use std::f32::consts::PI;

/// Axis to sort vertices along.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SortAxis {
    X,
    Y,
    Z,
}

/// Result of a vertex-sort operation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SortVertexResult {
    pub sorted_positions: Vec<[f32; 3]>,
    pub old_to_new: Vec<usize>,
    pub new_to_old: Vec<usize>,
}

/// Sort vertices along `axis` and return remapped positions plus mapping tables.
#[allow(dead_code)]
pub fn sort_vertices(positions: &[[f32; 3]], axis: SortAxis) -> SortVertexResult {
    let n = positions.len();
    let mut order: Vec<usize> = (0..n).collect();
    let ax = match axis {
        SortAxis::X => 0,
        SortAxis::Y => 1,
        SortAxis::Z => 2,
    };
    order.sort_by(|&a, &b| {
        positions[a][ax]
            .partial_cmp(&positions[b][ax])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut old_to_new = vec![0usize; n];
    for (new_idx, &old_idx) in order.iter().enumerate() {
        old_to_new[old_idx] = new_idx;
    }
    let sorted_positions = order.iter().map(|&i| positions[i]).collect();
    SortVertexResult {
        sorted_positions,
        old_to_new,
        new_to_old: order,
    }
}

/// Remap an index buffer using an `old_to_new` table.
#[allow(dead_code)]
pub fn remap_indices(indices: &[u32], old_to_new: &[usize]) -> Vec<u32> {
    indices
        .iter()
        .map(|&i| old_to_new[i as usize] as u32)
        .collect()
}

/// Check that a remap table is a valid permutation.
#[allow(dead_code)]
pub fn is_valid_permutation(old_to_new: &[usize]) -> bool {
    let n = old_to_new.len();
    let mut seen = vec![false; n];
    for &v in old_to_new {
        if v >= n || seen[v] {
            return false;
        }
        seen[v] = true;
    }
    true
}

/// Compute the bounding box [min, max] of sorted positions along `axis`.
#[allow(dead_code)]
pub fn sorted_axis_range(sorted: &[[f32; 3]], axis: SortAxis) -> (f32, f32) {
    if sorted.is_empty() {
        return (0.0, 0.0);
    }
    let ax = match axis {
        SortAxis::X => 0,
        SortAxis::Y => 1,
        SortAxis::Z => 2,
    };
    let first = sorted[0][ax];
    let last = sorted[sorted.len() - 1][ax];
    (first, last)
}

/// Return the index of the vertex closest to `origin` along `axis`.
#[allow(dead_code)]
pub fn nearest_to_origin(positions: &[[f32; 3]], axis: SortAxis) -> Option<usize> {
    if positions.is_empty() {
        return None;
    }
    let ax = match axis {
        SortAxis::X => 0,
        SortAxis::Y => 1,
        SortAxis::Z => 2,
    };
    positions
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            a[ax]
                .abs()
                .partial_cmp(&b[ax].abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
}

/// Compute centroid of positions.
#[allow(dead_code)]
pub fn centroid(positions: &[[f32; 3]]) -> [f32; 3] {
    if positions.is_empty() {
        return [0.0; 3];
    }
    let n = positions.len() as f32;
    let s = positions.iter().fold([0.0_f32; 3], |acc, p| {
        [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
    });
    [s[0] / n, s[1] / n, s[2] / n]
}

/// Sort by radial distance from the centroid (ignores `PI`).
#[allow(dead_code)]
pub fn sort_by_radius(positions: &[[f32; 3]]) -> SortVertexResult {
    let c = centroid(positions);
    let n = positions.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        let ra = (0..3)
            .map(|i| (positions[a][i] - c[i]).powi(2))
            .sum::<f32>();
        let rb = (0..3)
            .map(|i| (positions[b][i] - c[i]).powi(2))
            .sum::<f32>();
        ra.partial_cmp(&rb).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut old_to_new = vec![0usize; n];
    for (new_idx, &old_idx) in order.iter().enumerate() {
        old_to_new[old_idx] = new_idx;
    }
    let sorted_positions = order.iter().map(|&i| positions[i]).collect();
    let _ = PI; // suppress unused-import warning
    SortVertexResult {
        sorted_positions,
        old_to_new,
        new_to_old: order,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_positions() -> Vec<[f32; 3]> {
        vec![[3.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]
    }

    #[test]
    fn sort_x_ascending() {
        let res = sort_vertices(&sample_positions(), SortAxis::X);
        assert!((res.sorted_positions[0][0] - 1.0).abs() < 1e-6);
        assert!((res.sorted_positions[2][0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn old_to_new_permutation() {
        let res = sort_vertices(&sample_positions(), SortAxis::X);
        assert!(is_valid_permutation(&res.old_to_new));
    }

    #[test]
    fn remap_indices_basic() {
        let res = sort_vertices(&sample_positions(), SortAxis::X);
        let idx = vec![0u32, 1, 2];
        let remapped = remap_indices(&idx, &res.old_to_new);
        assert_eq!(remapped.len(), 3);
    }

    #[test]
    fn sorted_axis_range_test() {
        let res = sort_vertices(&sample_positions(), SortAxis::X);
        let (lo, hi) = sorted_axis_range(&res.sorted_positions, SortAxis::X);
        assert!(lo <= hi);
    }

    #[test]
    fn nearest_to_origin_test() {
        let pos = vec![[5.0, 0.0, 0.0], [0.1, 0.0, 0.0], [3.0, 0.0, 0.0]];
        assert_eq!(nearest_to_origin(&pos, SortAxis::X), Some(1));
    }

    #[test]
    fn centroid_basic() {
        let pos = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let c = centroid(&pos);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn sort_by_radius_is_permutation() {
        let res = sort_by_radius(&sample_positions());
        assert!(is_valid_permutation(&res.old_to_new));
    }

    #[test]
    fn sort_y_axis() {
        let pos = vec![[0.0, 3.0, 0.0], [0.0, 1.0, 0.0]];
        let res = sort_vertices(&pos, SortAxis::Y);
        assert!((res.sorted_positions[0][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn empty_positions() {
        let res = sort_vertices(&[], SortAxis::Z);
        assert!(res.sorted_positions.is_empty());
    }
}
