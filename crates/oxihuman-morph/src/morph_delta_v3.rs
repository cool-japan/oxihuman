// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Morph delta v3: compressed sparse deltas (only non-zero).

#[allow(dead_code)]
pub struct SparseDelta {
    pub index: u32,
    pub delta: [f32; 3],
}

#[allow(dead_code)]
pub struct MorphDeltaV3 {
    pub sparse_deltas: Vec<SparseDelta>,
    pub full_vertex_count: usize,
}

#[allow(dead_code)]
pub fn new_morph_delta_v3(full_vertex_count: usize) -> MorphDeltaV3 {
    MorphDeltaV3 { sparse_deltas: Vec::new(), full_vertex_count }
}

#[allow(dead_code)]
pub fn md3_add_delta(d: &mut MorphDeltaV3, index: u32, delta: [f32; 3]) {
    d.sparse_deltas.push(SparseDelta { index, delta });
}

#[allow(dead_code)]
pub fn md3_apply(d: &MorphDeltaV3, positions: &mut [[f32; 3]], weight: f32) {
    for sd in &d.sparse_deltas {
        let i = sd.index as usize;
        if i < positions.len() {
            positions[i][0] += sd.delta[0] * weight;
            positions[i][1] += sd.delta[1] * weight;
            positions[i][2] += sd.delta[2] * weight;
        }
    }
}

#[allow(dead_code)]
pub fn md3_delta_count(d: &MorphDeltaV3) -> usize {
    d.sparse_deltas.len()
}

#[allow(dead_code)]
pub fn md3_compression_ratio(d: &MorphDeltaV3) -> f32 {
    let dc = d.sparse_deltas.len();
    if dc == 0 {
        return 1.0;
    }
    d.full_vertex_count as f32 / dc as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_delta() {
        let mut d = new_morph_delta_v3(100);
        md3_add_delta(&mut d, 5, [0.1, 0.0, 0.0]);
        assert_eq!(md3_delta_count(&d), 1);
    }

    #[test]
    fn test_delta_count() {
        let mut d = new_morph_delta_v3(100);
        md3_add_delta(&mut d, 0, [1.0, 0.0, 0.0]);
        md3_add_delta(&mut d, 1, [0.0, 1.0, 0.0]);
        assert_eq!(md3_delta_count(&d), 2);
    }

    #[test]
    fn test_apply() {
        let mut d = new_morph_delta_v3(3);
        md3_add_delta(&mut d, 0, [1.0, 0.0, 0.0]);
        let mut pos = vec![[0.0f32; 3]; 3];
        md3_apply(&d, &mut pos, 1.0);
        assert!((pos[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_zero_weight() {
        let mut d = new_morph_delta_v3(3);
        md3_add_delta(&mut d, 0, [1.0, 0.0, 0.0]);
        let mut pos = vec![[0.0f32; 3]; 3];
        md3_apply(&d, &mut pos, 0.0);
        assert!(pos[0][0].abs() < 1e-5);
    }

    #[test]
    fn test_apply_partial_weight() {
        let mut d = new_morph_delta_v3(3);
        md3_add_delta(&mut d, 0, [2.0, 0.0, 0.0]);
        let mut pos = vec![[0.0f32; 3]; 3];
        md3_apply(&d, &mut pos, 0.5);
        assert!((pos[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compression_ratio() {
        let mut d = new_morph_delta_v3(100);
        md3_add_delta(&mut d, 0, [1.0, 0.0, 0.0]);
        md3_add_delta(&mut d, 1, [0.0, 1.0, 0.0]);
        assert!((md3_compression_ratio(&d) - 50.0).abs() < 1e-3);
    }

    #[test]
    fn test_compression_ratio_empty() {
        let d = new_morph_delta_v3(100);
        assert!((md3_compression_ratio(&d) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_out_of_bounds_safe() {
        let mut d = new_morph_delta_v3(3);
        md3_add_delta(&mut d, 999, [1.0, 0.0, 0.0]);
        let mut pos = vec![[0.0f32; 3]; 3];
        md3_apply(&d, &mut pos, 1.0); /* should not panic */
    }
}
