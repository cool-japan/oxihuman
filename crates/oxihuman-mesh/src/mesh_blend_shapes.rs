// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh-level blend shape (morph target) evaluation and interpolation.

// ── Types ─────────────────────────────────────────────────────────────────────

/// How blend-shape deltas are accumulated across multiple targets.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendAccumMode {
    /// Each weighted delta is added on top of the base.
    Additive,
    /// The highest-weight target completely overwrites the base.
    Overwrite,
    /// Weighted deltas are averaged by the sum of weights.
    Average,
}

/// Configuration for blend shape evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendShapeConfig {
    /// Normalize weights so they sum to 1.0 before evaluation.
    pub normalize_weights: bool,
    /// How multiple targets are combined.
    pub accumulation: BlendAccumMode,
}

/// A single blend shape (morph target): name + per-vertex position deltas.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendTarget {
    /// Human-readable name (e.g. "smile", "brow_raise_l").
    pub name: String,
    /// Per-vertex position delta (same length as base mesh).
    pub deltas: Vec<[f32; 3]>,
}

/// Output of [`evaluate_blend_shapes`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendShapeResult {
    /// Deformed vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Number of targets whose weight exceeded 0.0.
    pub active_targets: usize,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a default [`BlendShapeConfig`].
#[allow(dead_code)]
pub fn default_blend_shape_config() -> BlendShapeConfig {
    BlendShapeConfig {
        normalize_weights: false,
        accumulation: BlendAccumMode::Additive,
    }
}

/// Construct a new blend target.
#[allow(dead_code)]
pub fn new_blend_target(name: &str, deltas: Vec<[f32; 3]>) -> BlendTarget {
    BlendTarget {
        name: name.to_string(),
        deltas,
    }
}

/// Evaluate blend shapes and return deformed positions.
///
/// `base`, `targets`, and `weights` are zipped; extra entries are ignored.
#[allow(dead_code)]
pub fn evaluate_blend_shapes(
    base: &[[f32; 3]],
    targets: &[BlendTarget],
    weights: &[f32],
    cfg: &BlendShapeConfig,
) -> BlendShapeResult {
    let n = base.len();
    let mut positions: Vec<[f32; 3]> = base.to_vec();
    let mut active_targets = 0usize;

    let effective_weights: Vec<f32> = if cfg.normalize_weights {
        normalize_blend_weights(weights)
    } else {
        weights.to_vec()
    };

    let n_targets = targets.len().min(effective_weights.len());

    match cfg.accumulation {
        BlendAccumMode::Additive => {
            for (target, &w) in targets[..n_targets].iter().zip(effective_weights.iter()) {
                if w == 0.0 {
                    continue;
                }
                active_targets += 1;
                let m = n.min(target.deltas.len());
                for (pos, &delta) in positions[..m].iter_mut().zip(target.deltas[..m].iter()) {
                    add_weighted_delta(pos, delta, w);
                }
            }
        }
        BlendAccumMode::Overwrite => {
            // Find target with maximum weight.
            if let Some((best_idx, _)) = effective_weights[..n_targets]
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            {
                let w = effective_weights[best_idx];
                if w > 0.0 {
                    active_targets = 1;
                    let target = &targets[best_idx];
                    let m = n.min(target.deltas.len());
                    for (pos, &delta) in positions[..m].iter_mut().zip(target.deltas[..m].iter()) {
                        add_weighted_delta(pos, delta, w);
                    }
                }
            }
        }
        BlendAccumMode::Average => {
            let weight_sum: f32 = effective_weights[..n_targets]
                .iter()
                .filter(|&&w| w > 0.0)
                .sum();
            if weight_sum > 1e-12 {
                let mut acc: Vec<[f32; 3]> = vec![[0.0; 3]; n];
                for (target, &w) in targets[..n_targets].iter().zip(effective_weights.iter()) {
                    if w <= 0.0 {
                        continue;
                    }
                    active_targets += 1;
                    let m = n.min(target.deltas.len());
                    for (a, &delta) in acc[..m].iter_mut().zip(target.deltas[..m].iter()) {
                        a[0] += delta[0] * w;
                        a[1] += delta[1] * w;
                        a[2] += delta[2] * w;
                    }
                }
                for (pos, a) in positions.iter_mut().zip(acc.iter()) {
                    pos[0] += a[0] / weight_sum;
                    pos[1] += a[1] / weight_sum;
                    pos[2] += a[2] / weight_sum;
                }
            }
        }
    }

    BlendShapeResult {
        positions,
        active_targets,
    }
}

/// Number of per-vertex deltas in a blend target.
#[allow(dead_code)]
#[inline]
pub fn blend_target_delta_count(t: &BlendTarget) -> usize {
    t.deltas.len()
}

/// Normalize a weight slice so all values sum to 1.0.
///
/// Returns a zeroed vector if the sum is ≤ 0.
#[allow(dead_code)]
pub fn normalize_blend_weights(weights: &[f32]) -> Vec<f32> {
    let s: f32 = weights.iter().sum();
    if s <= 0.0 {
        vec![0.0; weights.len()]
    } else {
        weights.iter().map(|&w| w / s).collect()
    }
}

/// Serialize a [`BlendShapeResult`] to a compact JSON string.
#[allow(dead_code)]
pub fn blend_shape_result_to_json(r: &BlendShapeResult) -> String {
    format!(
        "{{\"active_targets\":{},\"vertex_count\":{}}}",
        r.active_targets,
        r.positions.len()
    )
}

/// Add a weighted delta to a mutable position.
#[allow(dead_code)]
#[inline]
pub fn add_weighted_delta(pos: &mut [f32; 3], delta: [f32; 3], weight: f32) {
    pos[0] += delta[0] * weight;
    pos[1] += delta[1] * weight;
    pos[2] += delta[2] * weight;
}

/// Serialize a [`BlendTarget`] to a compact JSON string.
#[allow(dead_code)]
pub fn blend_target_to_json(t: &BlendTarget) -> String {
    format!(
        "{{\"name\":\"{}\",\"delta_count\":{}}}",
        t.name,
        t.deltas.len()
    )
}

/// Maximum per-vertex displacement magnitude after blending.
#[allow(dead_code)]
pub fn max_displacement(r: &BlendShapeResult, base: &[[f32; 3]]) -> f32 {
    r.positions
        .iter()
        .zip(base.iter())
        .map(|(p, b)| {
            let dx = p[0] - b[0];
            let dy = p[1] - b[1];
            let dz = p[2] - b[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .fold(0.0_f32, f32::max)
}

/// Return the name of the accumulation mode.
#[allow(dead_code)]
pub fn accum_mode_name(cfg: &BlendShapeConfig) -> &'static str {
    match cfg.accumulation {
        BlendAccumMode::Additive => "additive",
        BlendAccumMode::Overwrite => "overwrite",
        BlendAccumMode::Average => "average",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_base() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }

    fn smile_target() -> BlendTarget {
        new_blend_target(
            "smile",
            vec![[0.0, 0.1, 0.0], [0.0, 0.1, 0.0], [0.0, 0.2, 0.0]],
        )
    }

    #[test]
    fn default_config_is_additive() {
        let cfg = default_blend_shape_config();
        assert_eq!(cfg.accumulation, BlendAccumMode::Additive);
        assert!(!cfg.normalize_weights);
    }

    #[test]
    fn additive_blend_at_full_weight() {
        let base = simple_base();
        let target = smile_target();
        let cfg = BlendShapeConfig {
            normalize_weights: false,
            accumulation: BlendAccumMode::Additive,
        };
        let result = evaluate_blend_shapes(&base, &[target], &[1.0], &cfg);
        assert!((result.positions[0][1] - 0.1).abs() < 1e-6);
        assert_eq!(result.active_targets, 1);
    }

    #[test]
    fn zero_weight_produces_no_change() {
        let base = simple_base();
        let target = smile_target();
        let cfg = default_blend_shape_config();
        let result = evaluate_blend_shapes(&base, &[target], &[0.0], &cfg);
        for (p, b) in result.positions.iter().zip(base.iter()) {
            assert!((p[0] - b[0]).abs() < 1e-6);
            assert!((p[1] - b[1]).abs() < 1e-6);
        }
        assert_eq!(result.active_targets, 0);
    }

    #[test]
    fn normalize_weights_sums_to_one() {
        let weights = vec![2.0_f32, 2.0, 2.0];
        let normalized = normalize_blend_weights(&weights);
        let sum: f32 = normalized.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn blend_target_to_json_has_name() {
        let t = smile_target();
        let json = blend_target_to_json(&t);
        assert!(json.contains("smile"));
        assert!(json.contains("delta_count"));
    }

    #[test]
    fn max_displacement_full_weight() {
        let base = simple_base();
        let target = smile_target();
        let cfg = default_blend_shape_config();
        let result = evaluate_blend_shapes(&base, &[target], &[1.0], &cfg);
        let disp = max_displacement(&result, &base);
        assert!(disp > 0.0);
    }

    #[test]
    fn accum_mode_name_strings() {
        assert_eq!(accum_mode_name(&BlendShapeConfig { normalize_weights: false, accumulation: BlendAccumMode::Additive }), "additive");
        assert_eq!(accum_mode_name(&BlendShapeConfig { normalize_weights: false, accumulation: BlendAccumMode::Overwrite }), "overwrite");
        assert_eq!(accum_mode_name(&BlendShapeConfig { normalize_weights: false, accumulation: BlendAccumMode::Average }), "average");
    }

    #[test]
    fn overwrite_mode_applies_best_weight() {
        let base = simple_base();
        let t1 = new_blend_target("low", vec![[0.0, 0.1, 0.0]; 3]);
        let t2 = new_blend_target("high", vec![[0.0, 0.5, 0.0]; 3]);
        let cfg = BlendShapeConfig {
            normalize_weights: false,
            accumulation: BlendAccumMode::Overwrite,
        };
        let result = evaluate_blend_shapes(&base, &[t1, t2], &[0.2, 0.8], &cfg);
        // highest weight is t2 at 0.8 → delta 0.5 * 0.8 = 0.4
        assert!((result.positions[0][1] - 0.4).abs() < 1e-5);
    }
}
