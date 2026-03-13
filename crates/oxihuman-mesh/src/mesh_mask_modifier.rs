// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Parameters for mask modifier.
pub struct MaskParams {
    pub threshold: f32,
    pub invert: bool,
}

pub fn new_mask_params(threshold: f32) -> MaskParams {
    MaskParams {
        threshold: threshold.clamp(0.0, 1.0),
        invert: false,
    }
}

pub fn mask_keep_vertex(weight: f32, params: &MaskParams) -> bool {
    let keep = weight >= params.threshold;
    if params.invert {
        !keep
    } else {
        keep
    }
}

pub fn mask_apply(weights: &[f32], params: &MaskParams) -> Vec<bool> {
    weights
        .iter()
        .map(|&w| mask_keep_vertex(w, params))
        .collect()
}

pub fn mask_count_visible(weights: &[f32], params: &MaskParams) -> usize {
    weights
        .iter()
        .filter(|&&w| mask_keep_vertex(w, params))
        .count()
}

pub fn mask_invert(params: &mut MaskParams) {
    params.invert = !params.invert;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mask_params() {
        let p = new_mask_params(0.5);
        assert!((p.threshold - 0.5).abs() < 1e-6);
        assert!(!p.invert);
    }

    #[test]
    fn test_mask_keep_vertex_above_threshold() {
        let p = new_mask_params(0.5);
        assert!(mask_keep_vertex(0.6, &p));
    }

    #[test]
    fn test_mask_keep_vertex_below_threshold() {
        let p = new_mask_params(0.5);
        assert!(!mask_keep_vertex(0.3, &p));
    }

    #[test]
    fn test_mask_apply_counts() {
        let p = new_mask_params(0.5);
        let weights = vec![0.1, 0.6, 0.9, 0.4];
        let result = mask_apply(&weights, &p);
        assert_eq!(result.iter().filter(|&&b| b).count(), 2);
    }

    #[test]
    fn test_mask_count_visible() {
        let p = new_mask_params(0.5);
        let weights = vec![0.0, 0.5, 1.0];
        assert_eq!(mask_count_visible(&weights, &p), 2);
    }

    #[test]
    fn test_mask_invert() {
        let mut p = new_mask_params(0.5);
        mask_invert(&mut p);
        assert!(p.invert);
        /* inverted: low weight vertex is kept */
        assert!(mask_keep_vertex(0.3, &p));
    }
}
