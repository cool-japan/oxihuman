// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cage-based deformation warp.

#[allow(dead_code)]
pub struct CageWarp {
    pub cage_positions: Vec<[f32; 3]>,
    pub target_positions: Vec<[f32; 3]>,
    pub weights: Vec<Vec<f32>>,
}

#[allow(dead_code)]
pub fn new_cage_warp(cage: Vec<[f32; 3]>) -> CageWarp {
    let target = cage.clone();
    CageWarp { cage_positions: cage, target_positions: target, weights: Vec::new() }
}

#[allow(dead_code)]
pub fn cw_set_weights(warp: &mut CageWarp, weights: Vec<Vec<f32>>) {
    warp.weights = weights;
}

#[allow(dead_code)]
pub fn cw_move_cage_vertex(warp: &mut CageWarp, i: usize, delta: [f32; 3]) {
    if i < warp.target_positions.len() {
        warp.target_positions[i][0] += delta[0];
        warp.target_positions[i][1] += delta[1];
        warp.target_positions[i][2] += delta[2];
    }
}

#[allow(dead_code)]
pub fn cw_cage_count(warp: &CageWarp) -> usize {
    warp.cage_positions.len()
}

#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn cw_deform_point(warp: &CageWarp, weights_for_point: &[f32]) -> [f32; 3] {
    let mut result = [0f32; 3];
    let n = weights_for_point.len().min(warp.target_positions.len());
    for i in 0..n {
        result[0] += weights_for_point[i] * warp.target_positions[i][0];
        result[1] += weights_for_point[i] * warp.target_positions[i][1];
        result[2] += weights_for_point[i] * warp.target_positions[i][2];
    }
    result
}

#[allow(dead_code)]
pub fn cw_reset_cage(warp: &mut CageWarp) {
    warp.target_positions = warp.cage_positions.clone();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cage() -> CageWarp {
        new_cage_warp(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
    }

    #[test]
    fn test_cage_count() {
        assert_eq!(cw_cage_count(&make_cage()), 3);
    }

    #[test]
    fn test_move_vertex_changes_target() {
        let mut warp = make_cage();
        cw_move_cage_vertex(&mut warp, 0, [5.0, 0.0, 0.0]);
        assert!((warp.target_positions[0][0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_move_vertex_does_not_change_cage() {
        let mut warp = make_cage();
        cw_move_cage_vertex(&mut warp, 0, [5.0, 0.0, 0.0]);
        assert!((warp.cage_positions[0][0]).abs() < 1e-5);
    }

    #[test]
    fn test_deform_point_weighted() {
        let warp = make_cage();
        let weights = vec![0.5, 0.5, 0.0];
        let p = cw_deform_point(&warp, &weights);
        assert!((p[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_deform_point_first_vertex() {
        let warp = make_cage();
        let weights = vec![1.0, 0.0, 0.0];
        let p = cw_deform_point(&warp, &weights);
        assert_eq!(p, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_reset_cage() {
        let mut warp = make_cage();
        cw_move_cage_vertex(&mut warp, 0, [5.0, 0.0, 0.0]);
        cw_reset_cage(&mut warp);
        assert!((warp.target_positions[0][0]).abs() < 1e-5);
    }

    #[test]
    fn test_set_weights() {
        let mut warp = make_cage();
        let w = vec![vec![0.5, 0.5, 0.0], vec![0.0, 1.0, 0.0]];
        cw_set_weights(&mut warp, w.clone());
        assert_eq!(warp.weights.len(), 2);
    }

    #[test]
    fn test_cage_count_empty() {
        let warp = new_cage_warp(vec![]);
        assert_eq!(cw_cage_count(&warp), 0);
    }
}
