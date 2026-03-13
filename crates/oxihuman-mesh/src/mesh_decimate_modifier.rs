// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Parameters for decimate modifier.
pub struct DecimateParams {
    pub ratio: f32,
    pub min_angle_deg: f32,
}

pub fn new_decimate_params(ratio: f32) -> DecimateParams {
    DecimateParams {
        ratio: decimate_ratio_clamp(ratio),
        min_angle_deg: 1.0,
    }
}

pub fn decimate_target_face_count(current: usize, params: &DecimateParams) -> usize {
    ((current as f32 * params.ratio).round() as usize).max(1)
}

pub fn decimate_collapse_count(current: usize, params: &DecimateParams) -> usize {
    let target = decimate_target_face_count(current, params);
    current.saturating_sub(target)
}

/// Shorter + flatter edges get higher collapse priority.
pub fn decimate_priority(edge_length: f32, dihedral_angle_deg: f32) -> f32 {
    let length_penalty = edge_length.max(1e-9).recip();
    let flatness = 1.0 - (dihedral_angle_deg / 180.0).clamp(0.0, 1.0);
    length_penalty + flatness
}

pub fn decimate_ratio_clamp(ratio: f32) -> f32 {
    ratio.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_decimate_params_clamped() {
        let p = new_decimate_params(1.5);
        assert!((p.ratio - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_decimate_target_face_count() {
        let p = new_decimate_params(0.5);
        assert_eq!(decimate_target_face_count(100, &p), 50);
    }

    #[test]
    fn test_decimate_collapse_count() {
        let p = new_decimate_params(0.5);
        assert_eq!(decimate_collapse_count(100, &p), 50);
    }

    #[test]
    fn test_decimate_priority_short_edge() {
        /* shorter edge should have higher priority than longer */
        let p_short = decimate_priority(0.1, 0.0);
        let p_long = decimate_priority(10.0, 0.0);
        assert!(p_short > p_long);
    }

    #[test]
    fn test_decimate_ratio_clamp_negative() {
        assert!((decimate_ratio_clamp(-0.5) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_decimate_target_minimum_one() {
        let p = new_decimate_params(0.0);
        assert_eq!(decimate_target_face_count(10, &p), 1);
    }
}
