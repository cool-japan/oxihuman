// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Transfer vertex attributes between meshes using nearest-neighbour lookup.

/// Transfer scalar attributes from a source mesh to a target mesh via nearest-neighbour.
/// Each target vertex gets the value of the nearest source vertex.
#[allow(dead_code)]
pub fn transfer_scalar_attr(
    src_positions: &[[f32; 3]],
    src_values: &[f32],
    dst_positions: &[[f32; 3]],
) -> Vec<f32> {
    if src_positions.is_empty() {
        return vec![0.0; dst_positions.len()];
    }
    dst_positions
        .iter()
        .map(|&dp| {
            let (idx, _) = nearest_vertex(dp, src_positions);
            src_values[idx]
        })
        .collect()
}

/// Transfer Vec3 attributes from source to target via nearest-neighbour.
#[allow(dead_code)]
pub fn transfer_vec3_attr(
    src_positions: &[[f32; 3]],
    src_values: &[[f32; 3]],
    dst_positions: &[[f32; 3]],
) -> Vec<[f32; 3]> {
    if src_positions.is_empty() {
        return vec![[0.0; 3]; dst_positions.len()];
    }
    dst_positions
        .iter()
        .map(|&dp| {
            let (idx, _) = nearest_vertex(dp, src_positions);
            src_values[idx]
        })
        .collect()
}

/// Transfer RGBA color attributes via nearest-neighbour.
#[allow(dead_code)]
pub fn transfer_rgba_attr(
    src_positions: &[[f32; 3]],
    src_colors: &[[f32; 4]],
    dst_positions: &[[f32; 3]],
) -> Vec<[f32; 4]> {
    if src_positions.is_empty() {
        return vec![[0.0; 4]; dst_positions.len()];
    }
    dst_positions
        .iter()
        .map(|&dp| {
            let (idx, _) = nearest_vertex(dp, src_positions);
            src_colors[idx]
        })
        .collect()
}

/// Find the nearest vertex in `positions` to `query`. Returns (index, distance).
#[allow(dead_code)]
pub fn nearest_vertex(query: [f32; 3], positions: &[[f32; 3]]) -> (usize, f32) {
    let mut best_idx = 0;
    let mut best_d = f32::INFINITY;
    for (i, &p) in positions.iter().enumerate() {
        let d = dist3_sq(query, p);
        if d < best_d {
            best_d = d;
            best_idx = i;
        }
    }
    (best_idx, best_d.sqrt())
}

/// Maximum transfer error (distance from dst vertex to its nearest source vertex).
#[allow(dead_code)]
pub fn max_transfer_error(src_positions: &[[f32; 3]], dst_positions: &[[f32; 3]]) -> f32 {
    if src_positions.is_empty() {
        return 0.0;
    }
    dst_positions
        .iter()
        .map(|&dp| nearest_vertex(dp, src_positions).1)
        .fold(0.0f32, f32::max)
}

/// Average transfer error.
#[allow(dead_code)]
pub fn avg_transfer_error(src_positions: &[[f32; 3]], dst_positions: &[[f32; 3]]) -> f32 {
    if src_positions.is_empty() || dst_positions.is_empty() {
        return 0.0;
    }
    let sum: f32 = dst_positions
        .iter()
        .map(|&dp| nearest_vertex(dp, src_positions).1)
        .sum();
    sum / dst_positions.len() as f32
}

fn dist3_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src() -> (Vec<[f32; 3]>, Vec<f32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
            vec![0.0, 1.0, 2.0],
        )
    }

    #[test]
    fn transfer_scalar_count() {
        let (src_pos, src_val) = src();
        let dst = vec![[0.1, 0.0, 0.0], [1.9, 0.0, 0.0]];
        let res = transfer_scalar_attr(&src_pos, &src_val, &dst);
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn transfer_scalar_nearest() {
        let (src_pos, src_val) = src();
        let dst = vec![[0.1, 0.0, 0.0]];
        let res = transfer_scalar_attr(&src_pos, &src_val, &dst);
        assert!((res[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn transfer_scalar_empty_src() {
        let res = transfer_scalar_attr(&[], &[], &[[0.0, 0.0, 0.0]]);
        assert_eq!(res, vec![0.0]);
    }

    #[test]
    fn transfer_vec3_correct() {
        let src_pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let src_v = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let dst = vec![[0.1, 0.0, 0.0]];
        let res = transfer_vec3_attr(&src_pos, &src_v, &dst);
        assert!((res[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn transfer_rgba_correct() {
        let src_pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let src_col = vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]];
        let dst = vec![[0.9, 0.0, 0.0]];
        let res = transfer_rgba_attr(&src_pos, &src_col, &dst);
        assert!((res[0][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn nearest_vertex_exact_match() {
        let (src_pos, _) = src();
        let (idx, dist) = nearest_vertex([1.0, 0.0, 0.0], &src_pos);
        assert_eq!(idx, 1);
        assert!(dist < 1e-6);
    }

    #[test]
    fn max_transfer_error_zero_same_mesh() {
        let (src_pos, _) = src();
        let err = max_transfer_error(&src_pos, &src_pos);
        assert!(err < 1e-6);
    }

    #[test]
    fn avg_transfer_error_empty() {
        assert_eq!(avg_transfer_error(&[], &[]), 0.0);
    }

    #[test]
    fn transfer_empty_dst() {
        let (src_pos, src_val) = src();
        let res = transfer_scalar_attr(&src_pos, &src_val, &[]);
        assert!(res.is_empty());
    }

    #[test]
    fn max_error_positive() {
        let src_pos = vec![[0.0, 0.0, 0.0]];
        let dst = vec![[3.0, 4.0, 0.0]];
        let err = max_transfer_error(&src_pos, &dst);
        assert!((err - 5.0).abs() < 0.01);
    }
}
