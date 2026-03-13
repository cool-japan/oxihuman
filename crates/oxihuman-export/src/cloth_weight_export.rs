// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cloth simulation weight export for per-vertex pin weights.

/// Cloth weight export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothWeightExport {
    pub weights: Vec<f32>,
}

/// Create new cloth weight export.
#[allow(dead_code)]
pub fn new_cloth_weight_export(count: usize) -> ClothWeightExport {
    ClothWeightExport {
        weights: vec![1.0; count],
    }
}

/// Set weight at vertex.
#[allow(dead_code)]
pub fn cw_set(e: &mut ClothWeightExport, idx: usize, w: f32) {
    if idx < e.weights.len() {
        e.weights[idx] = w.clamp(0.0, 1.0);
    }
}

/// Get weight.
#[allow(dead_code)]
pub fn cw_get(e: &ClothWeightExport, idx: usize) -> Option<f32> {
    e.weights.get(idx).copied()
}

/// Vertex count.
#[allow(dead_code)]
pub fn cw_count(e: &ClothWeightExport) -> usize {
    e.weights.len()
}

/// Average weight.
#[allow(dead_code)]
pub fn cw_average(e: &ClothWeightExport) -> f32 {
    if e.weights.is_empty() {
        return 0.0;
    }
    e.weights.iter().sum::<f32>() / e.weights.len() as f32
}

/// Count pinned vertices (weight == 0).
#[allow(dead_code)]
pub fn cw_pinned_count(e: &ClothWeightExport) -> usize {
    e.weights.iter().filter(|&&w| w < 1e-9).count()
}

/// Invert weights (1 - w).
#[allow(dead_code)]
pub fn cw_invert(e: &mut ClothWeightExport) {
    for w in &mut e.weights {
        *w = 1.0 - *w;
    }
}

/// Validate.
#[allow(dead_code)]
pub fn cw_validate(e: &ClothWeightExport) -> bool {
    e.weights.iter().all(|w| (0.0..=1.0).contains(w))
}

/// Export to JSON.
#[allow(dead_code)]
pub fn cloth_weight_to_json(e: &ClothWeightExport) -> String {
    format!(
        "{{\"vertices\":{},\"avg\":{:.6},\"pinned\":{}}}",
        cw_count(e),
        cw_average(e),
        cw_pinned_count(e)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let e = new_cloth_weight_export(5);
        assert_eq!(cw_count(&e), 5);
    }
    #[test]
    fn test_default_value() {
        let e = new_cloth_weight_export(1);
        assert!((cw_get(&e, 0).expect("should succeed") - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_set_get() {
        let mut e = new_cloth_weight_export(3);
        cw_set(&mut e, 1, 0.5);
        assert!((cw_get(&e, 1).expect("should succeed") - 0.5).abs() < 1e-6);
    }
    #[test]
    fn test_clamp() {
        let mut e = new_cloth_weight_export(1);
        cw_set(&mut e, 0, 2.0);
        assert!((cw_get(&e, 0).expect("should succeed") - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_average() {
        let mut e = new_cloth_weight_export(2);
        cw_set(&mut e, 0, 0.0);
        cw_set(&mut e, 1, 1.0);
        assert!((cw_average(&e) - 0.5).abs() < 1e-6);
    }
    #[test]
    fn test_pinned() {
        let mut e = new_cloth_weight_export(3);
        cw_set(&mut e, 0, 0.0);
        assert_eq!(cw_pinned_count(&e), 1);
    }
    #[test]
    fn test_invert() {
        let mut e = new_cloth_weight_export(1);
        cw_set(&mut e, 0, 0.3);
        cw_invert(&mut e);
        assert!((e.weights[0] - 0.7).abs() < 1e-6);
    }
    #[test]
    fn test_validate() {
        let e = new_cloth_weight_export(2);
        assert!(cw_validate(&e));
    }
    #[test]
    fn test_to_json() {
        let e = new_cloth_weight_export(1);
        assert!(cloth_weight_to_json(&e).contains("\"vertices\":1"));
    }
    #[test]
    fn test_empty_avg() {
        let e = ClothWeightExport { weights: vec![] };
        assert!((cw_average(&e)).abs() < 1e-9);
    }
}
