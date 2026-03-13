// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A per-vertex mask that controls expression influence.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionMask {
    pub weights: Vec<f32>,
}

/// Create a new expression mask with `count` vertices, all set to 1.0.
#[allow(dead_code)]
pub fn new_expression_mask(count: usize) -> ExpressionMask {
    ExpressionMask {
        weights: vec![1.0; count],
    }
}

/// Set the mask weight for a region (index range).
#[allow(dead_code)]
pub fn mask_set_region(mask: &mut ExpressionMask, start: usize, end: usize, value: f32) {
    let v = value.clamp(0.0, 1.0);
    let end = end.min(mask.weights.len());
    for w in mask.weights[start..end].iter_mut() {
        *w = v;
    }
}

/// Get the mask weight at a given index.
#[allow(dead_code)]
pub fn mask_get_region(mask: &ExpressionMask, index: usize) -> f32 {
    mask.weights.get(index).copied().unwrap_or(0.0)
}

/// Apply the mask to a set of deltas (element-wise multiply).
#[allow(dead_code)]
pub fn mask_apply(mask: &ExpressionMask, deltas: &[f32]) -> Vec<f32> {
    deltas
        .iter()
        .enumerate()
        .map(|(i, d)| d * mask.weights.get(i).copied().unwrap_or(0.0))
        .collect()
}

/// Invert the mask (1 - weight for each vertex).
#[allow(dead_code)]
pub fn mask_invert(mask: &mut ExpressionMask) {
    for w in &mut mask.weights {
        *w = 1.0 - *w;
    }
}

/// Union two masks (max of each weight).
#[allow(dead_code)]
pub fn mask_union_expr(a: &ExpressionMask, b: &ExpressionMask) -> ExpressionMask {
    let len = a.weights.len().max(b.weights.len());
    let mut weights = vec![0.0_f32; len];
    for (i, w) in weights.iter_mut().enumerate() {
        let va = a.weights.get(i).copied().unwrap_or(0.0);
        let vb = b.weights.get(i).copied().unwrap_or(0.0);
        *w = va.max(vb);
    }
    ExpressionMask { weights }
}

/// Return the vertex count.
#[allow(dead_code)]
pub fn mask_vertex_count_em(mask: &ExpressionMask) -> usize {
    mask.weights.len()
}

/// Serialize the mask to JSON.
#[allow(dead_code)]
pub fn mask_to_json(mask: &ExpressionMask) -> String {
    let vals: Vec<String> = mask.weights.iter().map(|w| format!("{:.4}", w)).collect();
    format!("{{\"weights\":[{}]}}", vals.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_mask_all_ones() {
        let m = new_expression_mask(10);
        assert_eq!(mask_vertex_count_em(&m), 10);
        assert!((mask_get_region(&m, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_region() {
        let mut m = new_expression_mask(10);
        mask_set_region(&mut m, 2, 5, 0.0);
        assert!(mask_get_region(&m, 3).abs() < 1e-6);
        assert!((mask_get_region(&m, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn get_out_of_bounds() {
        let m = new_expression_mask(5);
        assert!(mask_get_region(&m, 100).abs() < 1e-6);
    }

    #[test]
    fn apply_mask() {
        let mut m = new_expression_mask(3);
        mask_set_region(&mut m, 1, 2, 0.5);
        let deltas = vec![1.0, 1.0, 1.0];
        let result = mask_apply(&m, &deltas);
        assert!((result[0] - 1.0).abs() < 1e-6);
        assert!((result[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn invert_mask() {
        let mut m = new_expression_mask(3);
        mask_set_region(&mut m, 0, 1, 0.3);
        mask_invert(&mut m);
        assert!((mask_get_region(&m, 0) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn union_masks() {
        let mut a = new_expression_mask(3);
        mask_set_region(&mut a, 0, 3, 0.3);
        let mut b = new_expression_mask(3);
        mask_set_region(&mut b, 0, 3, 0.7);
        let u = mask_union_expr(&a, &b);
        assert!((mask_get_region(&u, 0) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn vertex_count() {
        let m = new_expression_mask(42);
        assert_eq!(mask_vertex_count_em(&m), 42);
    }

    #[test]
    fn to_json() {
        let m = new_expression_mask(2);
        let j = mask_to_json(&m);
        assert!(j.contains("\"weights\""));
    }

    #[test]
    fn apply_empty_deltas() {
        let m = new_expression_mask(3);
        let result = mask_apply(&m, &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn set_region_clamped() {
        let mut m = new_expression_mask(3);
        mask_set_region(&mut m, 0, 3, 1.5);
        assert!((mask_get_region(&m, 0) - 1.0).abs() < 1e-6);
    }
}
