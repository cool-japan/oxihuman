// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Apply a morph weight mask to a set of morph values.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphMask {
    pub weights: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_morph_mask(n: usize) -> MorphMask {
    MorphMask { weights: vec![1.0; n] }
}

#[allow(dead_code)]
pub fn mask_set(mask: &mut MorphMask, i: usize, w: f32) {
    mask.weights[i] = w;
}

#[allow(dead_code)]
pub fn mask_get(mask: &MorphMask, i: usize) -> f32 {
    mask.weights[i]
}

#[allow(dead_code)]
pub fn mask_apply(mask: &MorphMask, values: &[f32]) -> Vec<f32> {
    values.iter().enumerate().map(|(i, &v)| {
        if i < mask.weights.len() { v * mask.weights[i] } else { v }
    }).collect()
}

#[allow(dead_code)]
pub fn mask_invert(mask: &mut MorphMask) {
    for w in &mut mask.weights {
        *w = 1.0 - *w;
    }
}

#[allow(dead_code)]
pub fn mask_count(mask: &MorphMask) -> usize {
    mask.weights.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_all_ones() {
        let m = new_morph_mask(4);
        for i in 0..4 {
            assert!((mask_get(&m, i) - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_set_get() {
        let mut m = new_morph_mask(3);
        mask_set(&mut m, 1, 0.5);
        assert!((mask_get(&m, 1) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_scales_values() {
        let mut m = new_morph_mask(3);
        mask_set(&mut m, 0, 0.5);
        mask_set(&mut m, 1, 0.0);
        mask_set(&mut m, 2, 2.0);
        let result = mask_apply(&m, &[1.0, 1.0, 1.0]);
        assert!((result[0] - 0.5).abs() < 1e-6);
        assert!((result[1] - 0.0).abs() < 1e-6);
        assert!((result[2] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_invert() {
        let mut m = new_morph_mask(2);
        mask_set(&mut m, 0, 0.3);
        mask_invert(&mut m);
        assert!((mask_get(&m, 0) - 0.7).abs() < 1e-6);
        assert!((mask_get(&m, 1) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_count() {
        let m = new_morph_mask(5);
        assert_eq!(mask_count(&m), 5);
    }

    #[test]
    fn test_apply_identity() {
        let m = new_morph_mask(3);
        let vals = vec![0.2, 0.5, 0.8];
        let result = mask_apply(&m, &vals);
        for (a, b) in result.iter().zip(vals.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn test_apply_shorter_mask() {
        let m = new_morph_mask(2);
        let result = mask_apply(&m, &[1.0, 1.0, 1.0]);
        assert!((result[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_double_invert_identity() {
        let mut m = new_morph_mask(3);
        mask_set(&mut m, 0, 0.4);
        mask_invert(&mut m);
        mask_invert(&mut m);
        assert!((mask_get(&m, 0) - 0.4).abs() < 1e-5);
    }
}
