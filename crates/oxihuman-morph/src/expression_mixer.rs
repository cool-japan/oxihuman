// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

pub type MorphWeightMap = HashMap<String, f32>;

/// A single mixer layer that contributes to the final output.
pub struct MixLayer {
    pub name: String,
    pub weights: MorphWeightMap,
    /// Overall blend factor for this layer [0, 1].
    pub blend: f32,
    /// Whether this layer is additive (true) or override (false).
    pub additive: bool,
}

/// The expression mixer: stacks layers and produces a final morph weight map.
pub struct ExpressionMixer {
    layers: Vec<MixLayer>,
}

impl ExpressionMixer {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn add_layer(&mut self, layer: MixLayer) {
        self.layers.push(layer);
    }

    /// Remove layer by name; returns true if found and removed.
    pub fn remove_layer(&mut self, name: &str) -> bool {
        if let Some(pos) = self.layers.iter().position(|l| l.name == name) {
            self.layers.remove(pos);
            true
        } else {
            false
        }
    }

    /// Set blend factor for the named layer; returns true if found.
    pub fn set_blend(&mut self, name: &str, blend: f32) -> bool {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.name == name) {
            layer.blend = blend;
            true
        } else {
            false
        }
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Evaluate all layers in order to produce a final morph weight map.
    ///
    /// - Additive layers: add `weight * layer.blend` to each key.
    /// - Override layers: lerp current value toward layer's weight by `layer.blend`.
    pub fn evaluate(&self) -> MorphWeightMap {
        let mut result: MorphWeightMap = HashMap::new();

        for layer in &self.layers {
            if layer.additive {
                for (key, &val) in &layer.weights {
                    let current = result.entry(key.clone()).or_insert(0.0);
                    *current += val * layer.blend;
                }
            } else {
                // Override: lerp current toward layer value by blend.
                // First, collect all keys from result and layer.
                let all_keys: Vec<String> = result
                    .keys()
                    .chain(layer.weights.keys())
                    .cloned()
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                for key in all_keys {
                    let current = result.get(&key).copied().unwrap_or(0.0);
                    let target = layer.weights.get(&key).copied().unwrap_or(0.0);
                    let blended = current + (target - current) * layer.blend;
                    result.insert(key, blended);
                }
            }
        }

        result
    }

    pub fn clear(&mut self) {
        self.layers.clear();
    }
}

impl Default for ExpressionMixer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Standalone utility functions
// ---------------------------------------------------------------------------

/// Lerp between `a` (t=0) and `b` (t=1) over the union of all keys.
pub fn merge_weight_maps(a: &MorphWeightMap, b: &MorphWeightMap, t: f32) -> MorphWeightMap {
    let all_keys: std::collections::HashSet<&String> = a.keys().chain(b.keys()).collect();
    let mut result = MorphWeightMap::new();
    for key in all_keys {
        let av = a.get(key).copied().unwrap_or(0.0);
        let bv = b.get(key).copied().unwrap_or(0.0);
        result.insert(key.clone(), av + (bv - av) * t);
    }
    result
}

/// Add `scale * additive[k]` to `base[k]` over the union of all keys.
pub fn add_weight_maps(
    base: &MorphWeightMap,
    additive: &MorphWeightMap,
    scale: f32,
) -> MorphWeightMap {
    let all_keys: std::collections::HashSet<&String> = base.keys().chain(additive.keys()).collect();
    let mut result = MorphWeightMap::new();
    for key in all_keys {
        let bv = base.get(key).copied().unwrap_or(0.0);
        let av = additive.get(key).copied().unwrap_or(0.0);
        result.insert(key.clone(), bv + scale * av);
    }
    result
}

/// Clamp every value in the map to [min, max].
pub fn clamp_weight_map(map: &MorphWeightMap, min: f32, max: f32) -> MorphWeightMap {
    map.iter()
        .map(|(k, &v)| (k.clone(), v.clamp(min, max)))
        .collect()
}

/// Multiply every value in the map by `scale`.
pub fn scale_weight_map(map: &MorphWeightMap, scale: f32) -> MorphWeightMap {
    map.iter().map(|(k, &v)| (k.clone(), v * scale)).collect()
}

/// Compute the L2 magnitude (sqrt of sum of squares) of the weight map.
pub fn weight_map_magnitude(map: &MorphWeightMap) -> f32 {
    map.values().map(|&v| v * v).sum::<f32>().sqrt()
}

/// Return the top `n` entries by absolute value, sorted descending.
pub fn top_n_weights(map: &MorphWeightMap, n: usize) -> Vec<(String, f32)> {
    let mut entries: Vec<(String, f32)> = map.iter().map(|(k, &v)| (k.clone(), v)).collect();
    entries.sort_by(|a, b| {
        b.1.abs()
            .partial_cmp(&a.1.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    entries.truncate(n);
    entries
}

/// Keep only entries whose absolute value is >= threshold.
pub fn threshold_weight_map(map: &MorphWeightMap, threshold: f32) -> MorphWeightMap {
    map.iter()
        .filter(|(_, &v)| v.abs() >= threshold)
        .map(|(k, &v)| (k.clone(), v))
        .collect()
}

// ---------------------------------------------------------------------------
// Factory functions
// ---------------------------------------------------------------------------

/// Build a lip-sync override layer.
pub fn lip_sync_layer(viseme_weights: MorphWeightMap, blend: f32) -> MixLayer {
    MixLayer {
        name: "lip_sync".to_string(),
        weights: viseme_weights,
        blend,
        additive: false,
    }
}

/// Build an emotion override layer.
pub fn emotion_layer(emotion_weights: MorphWeightMap, blend: f32) -> MixLayer {
    MixLayer {
        name: "emotion".to_string(),
        weights: emotion_weights,
        blend,
        additive: false,
    }
}

/// Build a micro-expression additive layer.
pub fn micro_expression_layer(weights: MorphWeightMap, blend: f32) -> MixLayer {
    MixLayer {
        name: "micro_expression".to_string(),
        weights,
        blend,
        additive: true,
    }
}

/// Build a corrective additive layer.
pub fn corrective_layer(weights: MorphWeightMap, blend: f32) -> MixLayer {
    MixLayer {
        name: "corrective".to_string(),
        weights,
        blend,
        additive: true,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, f32)]) -> MorphWeightMap {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    // --- ExpressionMixer basics ---

    #[test]
    fn test_empty_mixer_evaluates_to_empty_map() {
        let mixer = ExpressionMixer::new();
        let result = mixer.evaluate();
        assert!(result.is_empty());
    }

    #[test]
    fn test_add_layer_increases_count() {
        let mut mixer = ExpressionMixer::new();
        assert_eq!(mixer.layer_count(), 0);
        mixer.add_layer(emotion_layer(map(&[("smile", 1.0)]), 1.0));
        assert_eq!(mixer.layer_count(), 1);
    }

    #[test]
    fn test_remove_layer_found() {
        let mut mixer = ExpressionMixer::new();
        mixer.add_layer(emotion_layer(map(&[("smile", 1.0)]), 1.0));
        let removed = mixer.remove_layer("emotion");
        assert!(removed);
        assert_eq!(mixer.layer_count(), 0);
    }

    #[test]
    fn test_remove_layer_not_found() {
        let mut mixer = ExpressionMixer::new();
        let removed = mixer.remove_layer("nonexistent");
        assert!(!removed);
    }

    #[test]
    fn test_set_blend_found() {
        let mut mixer = ExpressionMixer::new();
        mixer.add_layer(emotion_layer(map(&[("smile", 1.0)]), 0.5));
        let ok = mixer.set_blend("emotion", 0.8);
        assert!(ok);
        let result = mixer.evaluate();
        let val = result["smile"];
        assert!((val - 0.8).abs() < 1e-5, "expected 0.8, got {val}");
    }

    #[test]
    fn test_set_blend_not_found() {
        let mut mixer = ExpressionMixer::new();
        let ok = mixer.set_blend("absent", 0.5);
        assert!(!ok);
    }

    #[test]
    fn test_clear() {
        let mut mixer = ExpressionMixer::new();
        mixer.add_layer(emotion_layer(map(&[("smile", 1.0)]), 1.0));
        mixer.clear();
        assert_eq!(mixer.layer_count(), 0);
        assert!(mixer.evaluate().is_empty());
    }

    // --- Override layer behaviour ---

    #[test]
    fn test_override_layer_full_blend() {
        let mut mixer = ExpressionMixer::new();
        mixer.add_layer(MixLayer {
            name: "base".to_string(),
            weights: map(&[("a", 0.0)]),
            blend: 1.0,
            additive: false,
        });
        mixer.add_layer(MixLayer {
            name: "override".to_string(),
            weights: map(&[("a", 1.0)]),
            blend: 1.0,
            additive: false,
        });
        let result = mixer.evaluate();
        assert!((result["a"] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_override_layer_half_blend() {
        let mut mixer = ExpressionMixer::new();
        mixer.add_layer(MixLayer {
            name: "base".to_string(),
            weights: map(&[("a", 0.0)]),
            blend: 1.0,
            additive: false,
        });
        mixer.add_layer(MixLayer {
            name: "override".to_string(),
            weights: map(&[("a", 1.0)]),
            blend: 0.5,
            additive: false,
        });
        let result = mixer.evaluate();
        assert!((result["a"] - 0.5).abs() < 1e-5);
    }

    // --- Additive layer behaviour ---

    #[test]
    fn test_additive_layer() {
        let mut mixer = ExpressionMixer::new();
        mixer.add_layer(MixLayer {
            name: "base".to_string(),
            weights: map(&[("a", 0.3)]),
            blend: 1.0,
            additive: false,
        });
        mixer.add_layer(MixLayer {
            name: "add".to_string(),
            weights: map(&[("a", 0.5)]),
            blend: 1.0,
            additive: true,
        });
        let result = mixer.evaluate();
        // override sets a=0.3, then additive adds 0.5*1.0=0.5 → 0.8
        assert!((result["a"] - 0.8).abs() < 1e-5, "got {}", result["a"]);
    }

    #[test]
    fn test_additive_layer_with_scale() {
        let mut mixer = ExpressionMixer::new();
        mixer.add_layer(micro_expression_layer(map(&[("twitch", 0.4)]), 0.5));
        let result = mixer.evaluate();
        // additive: 0.4 * 0.5 = 0.2
        assert!((result["twitch"] - 0.2).abs() < 1e-5);
    }

    // --- Standalone utilities ---

    #[test]
    fn test_merge_weight_maps_midpoint() {
        let a = map(&[("x", 0.0), ("y", 1.0)]);
        let b = map(&[("x", 1.0), ("z", 1.0)]);
        let m = merge_weight_maps(&a, &b, 0.5);
        assert!((m["x"] - 0.5).abs() < 1e-5);
        assert!((m["y"] - 0.5).abs() < 1e-5);
        assert!((m["z"] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_merge_weight_maps_t0_equals_a() {
        let a = map(&[("x", 0.3)]);
        let b = map(&[("x", 0.9)]);
        let m = merge_weight_maps(&a, &b, 0.0);
        assert!((m["x"] - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_add_weight_maps() {
        let base = map(&[("a", 0.5)]);
        let add = map(&[("a", 0.2), ("b", 0.4)]);
        let result = add_weight_maps(&base, &add, 2.0);
        assert!((result["a"] - 0.9).abs() < 1e-5); // 0.5 + 2*0.2
        assert!((result["b"] - 0.8).abs() < 1e-5); // 0.0 + 2*0.4
    }

    #[test]
    fn test_clamp_weight_map() {
        let m = map(&[("a", -0.5), ("b", 1.5), ("c", 0.5)]);
        let c = clamp_weight_map(&m, 0.0, 1.0);
        assert!((c["a"] - 0.0).abs() < 1e-5);
        assert!((c["b"] - 1.0).abs() < 1e-5);
        assert!((c["c"] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_scale_weight_map() {
        let m = map(&[("a", 0.4), ("b", 0.8)]);
        let s = scale_weight_map(&m, 0.5);
        assert!((s["a"] - 0.2).abs() < 1e-5);
        assert!((s["b"] - 0.4).abs() < 1e-5);
    }

    #[test]
    fn test_weight_map_magnitude() {
        let m = map(&[("a", 3.0), ("b", 4.0)]);
        let mag = weight_map_magnitude(&m);
        assert!((mag - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_top_n_weights() {
        let m = map(&[("a", 0.1), ("b", 0.9), ("c", 0.5), ("d", -0.8)]);
        let top = top_n_weights(&m, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "b");
        assert_eq!(top[1].0, "d");
    }

    #[test]
    fn test_top_n_weights_fewer_than_n() {
        let m = map(&[("x", 0.3)]);
        let top = top_n_weights(&m, 5);
        assert_eq!(top.len(), 1);
    }

    #[test]
    fn test_threshold_weight_map() {
        let m = map(&[("a", 0.05), ("b", 0.5), ("c", -0.3)]);
        let t = threshold_weight_map(&m, 0.1);
        assert!(!t.contains_key("a"));
        assert!(t.contains_key("b"));
        assert!(t.contains_key("c"));
    }

    // --- Factory functions ---

    #[test]
    fn test_lip_sync_layer_factory() {
        let layer = lip_sync_layer(map(&[("vowel_a", 1.0)]), 0.7);
        assert_eq!(layer.name, "lip_sync");
        assert!(!layer.additive);
        assert!((layer.blend - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_emotion_layer_factory() {
        let layer = emotion_layer(map(&[("smile", 0.8)]), 1.0);
        assert_eq!(layer.name, "emotion");
        assert!(!layer.additive);
    }

    #[test]
    fn test_micro_expression_layer_factory() {
        let layer = micro_expression_layer(map(&[("brow_raise", 0.3)]), 0.5);
        assert_eq!(layer.name, "micro_expression");
        assert!(layer.additive);
    }

    #[test]
    fn test_corrective_layer_factory() {
        let layer = corrective_layer(map(&[("jaw_fix", 0.1)]), 1.0);
        assert_eq!(layer.name, "corrective");
        assert!(layer.additive);
    }
}
