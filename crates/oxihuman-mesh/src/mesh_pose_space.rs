// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::TAU;

pub struct PoseKey {
    pub pose_angle_rad: f32,
    pub delta_positions: Vec<[f32; 3]>,
}

pub fn new_pose_key(angle: f32, deltas: Vec<[f32; 3]>) -> PoseKey {
    PoseKey {
        pose_angle_rad: angle,
        delta_positions: deltas,
    }
}

/// Gaussian weight based on angular distance.
pub fn pose_weight(key: &PoseKey, current_angle: f32, falloff: f32) -> f32 {
    let diff = angle_diff(key.pose_angle_rad, current_angle);
    let sigma = falloff.max(1e-8);
    (-0.5 * (diff / sigma).powi(2)).exp()
}

fn angle_diff(a: f32, b: f32) -> f32 {
    /* shortest angular distance on circle */
    let mut d = (a - b) % TAU;
    if d > std::f32::consts::PI {
        d -= TAU;
    } else if d < -std::f32::consts::PI {
        d += TAU;
    }
    d.abs()
}

pub fn pose_apply(
    keys: &[PoseKey],
    base: &[[f32; 3]],
    current_angle: f32,
    falloff: f32,
) -> Vec<[f32; 3]> {
    let n = base.len();
    let mut result: Vec<[f32; 3]> = base.to_vec();
    let weights: Vec<f32> = keys
        .iter()
        .map(|k| pose_weight(k, current_angle, falloff))
        .collect();
    let total: f32 = weights.iter().sum();
    if total < 1e-8 {
        return result;
    }
    for k in 0..keys.len() {
        let w = weights[k] / total;
        let deltas = &keys[k].delta_positions;
        for i in 0..n.min(deltas.len()) {
            result[i][0] += w * deltas[i][0];
            result[i][1] += w * deltas[i][1];
            result[i][2] += w * deltas[i][2];
        }
    }
    result
}

pub fn pose_key_count(keys: &[PoseKey]) -> usize {
    keys.len()
}

pub fn pose_best_key(keys: &[PoseKey], angle: f32) -> usize {
    let mut best = 0;
    let mut best_diff = f32::MAX;
    for (i, k) in keys.iter().enumerate() {
        let d = angle_diff(k.pose_angle_rad, angle);
        if d < best_diff {
            best_diff = d;
            best = i;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new_pose_key() {
        /* construction */
        let k = new_pose_key(PI, vec![[0.1, 0.0, 0.0]]);
        assert!((k.pose_angle_rad - PI).abs() < 1e-6);
        assert_eq!(k.delta_positions.len(), 1);
    }

    #[test]
    fn test_pose_weight_at_key() {
        /* weight is 1 when current matches key */
        let k = new_pose_key(0.0, vec![]);
        let w = pose_weight(&k, 0.0, 0.5);
        assert!((w - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_pose_weight_falloff() {
        /* weight decreases with distance */
        let k = new_pose_key(0.0, vec![]);
        let w0 = pose_weight(&k, 0.0, 0.5);
        let w1 = pose_weight(&k, 1.0, 0.5);
        assert!(w0 > w1);
    }

    #[test]
    fn test_pose_apply_no_keys() {
        /* no keys => base unchanged */
        let base = vec![[1.0, 0.0, 0.0_f32]];
        let result = pose_apply(&[], &base, 0.0, 1.0);
        assert_eq!(result, base);
    }

    #[test]
    fn test_pose_best_key() {
        /* picks closest key */
        let keys = vec![new_pose_key(0.0, vec![]), new_pose_key(PI, vec![])];
        let best = pose_best_key(&keys, 0.1);
        assert_eq!(best, 0);
    }

    #[test]
    fn test_pose_key_count() {
        /* count */
        let keys = vec![new_pose_key(0.0, vec![]), new_pose_key(1.0, vec![])];
        assert_eq!(pose_key_count(&keys), 2);
    }
}
