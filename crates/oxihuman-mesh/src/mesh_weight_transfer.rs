// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Transfer blend weights from source mesh to target mesh.

#[allow(dead_code)]
pub struct WeightTransfer {
    pub source_weights: Vec<Vec<f32>>,
    pub source_positions: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn new_weight_transfer(
    source_positions: Vec<[f32; 3]>,
    source_weights: Vec<Vec<f32>>,
) -> WeightTransfer {
    WeightTransfer { source_weights, source_positions }
}

fn sq_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

#[allow(dead_code)]
pub fn wt_transfer_point(wt: &WeightTransfer, pos: [f32; 3]) -> Vec<f32> {
    if wt.source_positions.is_empty() {
        return Vec::new();
    }
    let mut best_idx = 0;
    let mut best_dist = f32::MAX;
    for (i, &sp) in wt.source_positions.iter().enumerate() {
        let d = sq_dist(pos, sp);
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }
    if best_idx < wt.source_weights.len() {
        wt.source_weights[best_idx].clone()
    } else {
        Vec::new()
    }
}

#[allow(dead_code)]
pub fn wt_transfer_mesh(wt: &WeightTransfer, target_positions: &[[f32; 3]]) -> Vec<Vec<f32>> {
    target_positions.iter().map(|&pos| wt_transfer_point(wt, pos)).collect()
}

#[allow(dead_code)]
pub fn wt_source_count(wt: &WeightTransfer) -> usize {
    wt.source_positions.len()
}

#[allow(dead_code)]
pub fn wt_weight_dim(wt: &WeightTransfer) -> usize {
    wt.source_weights.first().map_or(0, |w| w.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_wt() -> WeightTransfer {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let weights = vec![
            vec![0.6f32, 0.3, 0.1],
            vec![0.1f32, 0.8, 0.1],
            vec![0.2f32, 0.2, 0.6],
        ];
        new_weight_transfer(positions, weights)
    }

    #[test]
    fn test_source_count() {
        let wt = make_wt();
        assert_eq!(wt_source_count(&wt), 3);
    }

    #[test]
    fn test_weight_dim() {
        let wt = make_wt();
        assert_eq!(wt_weight_dim(&wt), 3);
    }

    #[test]
    fn test_transfer_point_nearest() {
        let wt = make_wt();
        let w = wt_transfer_point(&wt, [0.05, 0.0, 0.0]);
        assert_eq!(w.len(), 3);
        assert!((w[0] - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_transfer_mesh_count() {
        let wt = make_wt();
        let targets = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let result = wt_transfer_mesh(&wt, &targets);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_weights_sum_near_one() {
        let wt = make_wt();
        let w = wt_transfer_point(&wt, [0.0, 0.0, 0.0]);
        let sum: f32 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_source() {
        let wt = new_weight_transfer(vec![], vec![]);
        let w = wt_transfer_point(&wt, [0.0, 0.0, 0.0]);
        assert!(w.is_empty());
    }

    #[test]
    fn test_transfer_mesh_empty_targets() {
        let wt = make_wt();
        let result = wt_transfer_mesh(&wt, &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_weight_dim_empty() {
        let wt = new_weight_transfer(vec![], vec![]);
        assert_eq!(wt_weight_dim(&wt), 0);
    }
}
