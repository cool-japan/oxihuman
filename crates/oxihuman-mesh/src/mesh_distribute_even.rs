// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Distribute vertices evenly along a path (arc-length parameterisation).

/// Result of even distribution.
#[derive(Debug, Clone, Default)]
pub struct DistributeEvenResult {
    pub vertices_moved: usize,
    pub total_path_length: f32,
}

/// Computes the Euclidean distance between two 3-D points.
pub fn dist3_de(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Computes cumulative arc lengths along an ordered list of control points.
pub fn arc_lengths(control: &[[f32; 3]]) -> Vec<f32> {
    let mut lens = vec![0.0f32; control.len()];
    for i in 1..control.len() {
        lens[i] = lens[i - 1] + dist3_de(control[i - 1], control[i]);
    }
    lens
}

/// Samples a position at a given arc-length parameter `t ∈ [0, total_length]`
/// along a polyline defined by `control` points.
pub fn sample_polyline(control: &[[f32; 3]], arc_lens: &[f32], t: f32) -> [f32; 3] {
    let total = *arc_lens.last().unwrap_or(&0.0);
    if control.is_empty() {
        return [0.0; 3];
    }
    if t <= 0.0 {
        return control[0];
    }
    if t >= total {
        return control[control.len() - 1];
    }
    /* find segment */
    for i in 1..arc_lens.len() {
        if arc_lens[i] >= t {
            let seg_len = arc_lens[i] - arc_lens[i - 1];
            if seg_len < 1e-10 {
                return control[i];
            }
            let local_t = (t - arc_lens[i - 1]) / seg_len;
            let a = control[i - 1];
            let b = control[i];
            return [
                a[0] + (b[0] - a[0]) * local_t,
                a[1] + (b[1] - a[1]) * local_t,
                a[2] + (b[2] - a[2]) * local_t,
            ];
        }
    }
    control[control.len() - 1]
}

/// Distributes `vertex_ids` evenly along a path defined by `control` points.
/// The first and last vertex are kept at the path endpoints.
pub fn distribute_even(
    positions: &mut [[f32; 3]],
    vertex_ids: &[u32],
    control: &[[f32; 3]],
) -> DistributeEvenResult {
    let n = vertex_ids.len();
    if n < 2 || control.len() < 2 {
        return DistributeEvenResult::default();
    }
    let arc_lens = arc_lengths(control);
    let total = *arc_lens.last().unwrap_or(&0.0);
    let mut moved = 0usize;
    for (i, &vi) in vertex_ids.iter().enumerate() {
        let vi = vi as usize;
        if vi >= positions.len() {
            continue;
        }
        let t = total * (i as f32) / (n - 1) as f32;
        positions[vi] = sample_polyline(control, &arc_lens, t);
        moved += 1;
    }
    DistributeEvenResult {
        vertices_moved: moved,
        total_path_length: total,
    }
}

/// Returns the spacing between consecutive vertices after even distribution.
pub fn even_spacing(total_length: f32, count: usize) -> f32 {
    if count < 2 {
        return 0.0;
    }
    total_length / (count - 1) as f32
}

/// Returns the maximum deviation from even spacing in the current positions.
pub fn max_spacing_deviation(positions: &[[f32; 3]], vertex_ids: &[u32]) -> f32 {
    if vertex_ids.len() < 2 {
        return 0.0;
    }
    let mut dists = Vec::new();
    for i in 0..vertex_ids.len().saturating_sub(1) {
        let vi_a = vertex_ids[i] as usize;
        let vi_b = vertex_ids[i + 1] as usize;
        if vi_a < positions.len() && vi_b < positions.len() {
            dists.push(dist3_de(positions[vi_a], positions[vi_b]));
        }
    }
    if dists.is_empty() {
        return 0.0;
    }
    let avg = dists.iter().sum::<f32>() / dists.len() as f32;
    dists
        .iter()
        .map(|&d| (d - avg).abs())
        .fold(0.0f32, f32::max)
}

/// Total length of a chain of positions given by vertex_ids.
pub fn chain_length(positions: &[[f32; 3]], vertex_ids: &[u32]) -> f32 {
    let mut total = 0.0f32;
    for i in 0..vertex_ids.len().saturating_sub(1) {
        let va = vertex_ids[i] as usize;
        let vb = vertex_ids[i + 1] as usize;
        if va < positions.len() && vb < positions.len() {
            total += dist3_de(positions[va], positions[vb]);
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dist3_correct() {
        let d = dist3_de([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((d - 5.0).abs() < 1e-5);
    }

    #[test]
    fn arc_lengths_monotone() {
        let ctrl = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let lens = arc_lengths(&ctrl);
        assert!(lens[0] < lens[1] && lens[1] < lens[2]);
    }

    #[test]
    fn sample_polyline_midpoint() {
        let ctrl = vec![[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let lens = arc_lengths(&ctrl);
        let p = sample_polyline(&ctrl, &lens, 1.0);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn distribute_even_moves_all() {
        let ctrl = vec![[0.0f32, 0.0, 0.0], [4.0, 0.0, 0.0]];
        let mut pos = vec![[0.0f32; 3]; 3];
        let vids = [0u32, 1, 2];
        let res = distribute_even(&mut pos, &vids, &ctrl);
        assert_eq!(res.vertices_moved, 3);
    }

    #[test]
    fn even_spacing_two_verts() {
        let s = even_spacing(4.0, 5);
        assert!((s - 1.0).abs() < 1e-5);
    }

    #[test]
    fn even_spacing_one_vert_zero() {
        assert_eq!(even_spacing(4.0, 1), 0.0);
    }

    #[test]
    fn max_spacing_deviation_zero_for_even() {
        let pos = vec![[0.0f32; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let vids = [0u32, 1, 2];
        let dev = max_spacing_deviation(&pos, &vids);
        assert!(dev < 1e-5);
    }

    #[test]
    fn chain_length_correct() {
        let pos = vec![[0.0f32; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let vids = [0u32, 1, 2];
        assert!((chain_length(&pos, &vids) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn distribute_too_few_verts_returns_default() {
        let ctrl = vec![[0.0f32; 3], [1.0, 0.0, 0.0]];
        let mut pos = vec![[0.0f32; 3]];
        let vids = [0u32];
        let res = distribute_even(&mut pos, &vids, &ctrl);
        assert_eq!(res.vertices_moved, 0);
    }
}
