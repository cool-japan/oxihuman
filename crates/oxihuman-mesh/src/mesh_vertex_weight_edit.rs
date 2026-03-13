// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vertex weight edit modifier.

/// Falloff type for vertex weight editing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeightFalloff {
    None,
    Curve,
    Sharp,
    Smooth,
    Root,
    Sphere,
}

/// Configuration for vertex weight edit.
#[derive(Debug, Clone)]
pub struct VertexWeightEditConfig {
    pub global_influence: f32,
    pub falloff: WeightFalloff,
    pub add_threshold: f32,
    pub remove_threshold: f32,
}

impl Default for VertexWeightEditConfig {
    fn default() -> Self {
        Self {
            global_influence: 1.0,
            falloff: WeightFalloff::None,
            add_threshold: 0.01,
            remove_threshold: 0.0,
        }
    }
}

impl VertexWeightEditConfig {
    pub fn new(influence: f32) -> Self {
        Self { global_influence: influence, ..Self::default() }
    }
}

/// Apply falloff to a raw weight value.
pub fn apply_falloff(weight: f32, falloff: WeightFalloff) -> f32 {
    let w = weight.clamp(0.0, 1.0);
    match falloff {
        WeightFalloff::None => w,
        WeightFalloff::Sharp => w * w,
        WeightFalloff::Smooth => w * w * (3.0 - 2.0 * w),
        WeightFalloff::Root => w.sqrt(),
        WeightFalloff::Sphere => (1.0 - (1.0 - w * w).max(0.0).sqrt()).clamp(0.0, 1.0),
        WeightFalloff::Curve => w, /* placeholder */
    }
}

/// Edit weights: scale by global influence and apply falloff.
pub fn edit_weights(weights: &mut [f32], cfg: &VertexWeightEditConfig) {
    for w in weights.iter_mut() {
        *w = apply_falloff(*w * cfg.global_influence, cfg.falloff).clamp(0.0, 1.0);
    }
}

/// Remove vertices with weight below threshold.
pub fn remove_below_threshold(weights: &[f32], threshold: f32) -> Vec<usize> {
    weights
        .iter()
        .enumerate()
        .filter(|(_, &w)| w < threshold)
        .map(|(i, _)| i)
        .collect()
}

/// Normalize weights so they sum to 1.0.
pub fn normalize_weights(weights: &mut [f32]) {
    let sum: f32 = weights.iter().sum();
    if sum > 1e-8 {
        for w in weights.iter_mut() {
            *w /= sum;
        }
    }
}

/// Validate config.
pub fn validate_vertex_weight_edit_config(cfg: &VertexWeightEditConfig) -> bool {
    (0.0..=1.0).contains(&cfg.global_influence)
        && cfg.add_threshold >= 0.0
        && cfg.remove_threshold >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let cfg = VertexWeightEditConfig::default();
        assert!((cfg.global_influence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_falloff_none() {
        assert!((apply_falloff(0.5, WeightFalloff::None) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_falloff_sharp() {
        assert!((apply_falloff(0.5, WeightFalloff::Sharp) - 0.25).abs() < 1e-5);
    }

    #[test]
    fn test_falloff_smooth_endpoints() {
        assert!(apply_falloff(0.0, WeightFalloff::Smooth).abs() < 1e-5);
        assert!((apply_falloff(1.0, WeightFalloff::Smooth) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_falloff_root() {
        assert!((apply_falloff(0.25, WeightFalloff::Root) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_edit_weights() {
        let mut w = vec![0.5_f32, 0.8];
        let cfg = VertexWeightEditConfig::new(0.5);
        edit_weights(&mut w, &cfg);
        assert!(w[0] <= 1.0);
        assert!(w[0] >= 0.0);
    }

    #[test]
    fn test_remove_below_threshold() {
        let weights = vec![0.1_f32, 0.5, 0.01, 0.9];
        let removed = remove_below_threshold(&weights, 0.05);
        assert_eq!(removed, vec![2]);
    }

    #[test]
    fn test_normalize_weights() {
        let mut w = vec![1.0_f32, 1.0, 2.0];
        normalize_weights(&mut w);
        let sum: f32 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_validate_config_valid() {
        let cfg = VertexWeightEditConfig::default();
        assert!(validate_vertex_weight_edit_config(&cfg));
    }

    #[test]
    fn test_validate_config_invalid_influence() {
        let cfg = VertexWeightEditConfig { global_influence: 1.5, ..Default::default() };
        assert!(!validate_vertex_weight_edit_config(&cfg));
    }
}
