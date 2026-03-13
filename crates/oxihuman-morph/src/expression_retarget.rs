// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

/// A map from morph-target name to blend weight.
pub type MorphWeights = HashMap<String, f32>;

// ---------------------------------------------------------------------------
// RetargetMap
// ---------------------------------------------------------------------------

/// A bidirectional name mapping between two character rigs.
///
/// Optionally carries prefix-rewrite rules inserted by [`build_prefix_map`].
pub struct RetargetMap {
    /// source_name → target_name (explicit entries)
    forward: HashMap<String, String>,
    /// target_name → source_name (explicit entries)
    inverse: HashMap<String, String>,
    /// (src_prefix, tgt_prefix) rules for dynamic resolution
    prefix_rules: Vec<(String, String)>,
}

impl RetargetMap {
    /// Create an empty map with no prefix rules.
    pub fn new() -> Self {
        Self {
            forward: HashMap::new(),
            inverse: HashMap::new(),
            prefix_rules: Vec::new(),
        }
    }

    /// Internal constructor that stores prefix rules.
    fn new_with_prefix_rules(rules: &[(&str, &str)]) -> Self {
        Self {
            forward: HashMap::new(),
            inverse: HashMap::new(),
            prefix_rules: rules
                .iter()
                .map(|&(s, t)| (s.to_owned(), t.to_owned()))
                .collect(),
        }
    }

    /// Register a source ↔ target name pair (explicit entry).
    pub fn add(&mut self, source: impl Into<String>, target: impl Into<String>) {
        let s = source.into();
        let t = target.into();
        self.forward.insert(s.clone(), t.clone());
        self.inverse.insert(t, s);
    }

    /// Look up the target name for a given source name.
    ///
    /// Checks explicit entries first, then prefix rules.
    pub fn forward(&self, source: &str) -> Option<&str> {
        if let Some(t) = self.forward.get(source) {
            return Some(t.as_str());
        }
        // Try prefix rules (returns a &str into a temporary; we need a
        // heap-allocated version).  Because we cannot return a &str into a
        // local String, prefix rules are resolved by `retarget_weights` and
        // the public standalone `retarget_weights` function directly.
        // For the purpose of this method we only return explicit entries.
        // Prefix-rule callers should use `forward_owned`.
        None
    }

    /// Like `forward` but returns an owned `String`; also resolves prefix rules.
    pub fn forward_owned(&self, source: &str) -> Option<String> {
        if let Some(t) = self.forward.get(source) {
            return Some(t.clone());
        }
        for (src_pfx, tgt_pfx) in &self.prefix_rules {
            if let Some(suffix) = source.strip_prefix(src_pfx.as_str()) {
                return Some(format!("{}{}", tgt_pfx, suffix));
            }
        }
        None
    }

    /// Look up the source name for a given target name.
    pub fn inverse(&self, target: &str) -> Option<&str> {
        self.inverse.get(target).map(|s| s.as_str())
    }

    /// Remap `source_weights` keys through the forward mapping; drop unmapped.
    pub fn retarget_weights(&self, source_weights: &MorphWeights) -> MorphWeights {
        let mut out = MorphWeights::new();
        for (k, &v) in source_weights {
            if let Some(mapped) = self.forward_owned(k) {
                out.insert(mapped, v);
            }
        }
        out
    }

    /// Remap `target_weights` keys through the inverse mapping; drop unmapped.
    pub fn inverse_retarget_weights(&self, target_weights: &MorphWeights) -> MorphWeights {
        let mut out = MorphWeights::new();
        for (k, &v) in target_weights {
            if let Some(mapped) = self.inverse(k) {
                out.insert(mapped.to_owned(), v);
            }
        }
        out
    }

    /// Number of explicit entries stored (prefix rules are not counted).
    pub fn len(&self) -> usize {
        self.forward.len()
    }

    /// Returns `true` when there are no explicit entries and no prefix rules.
    pub fn is_empty(&self) -> bool {
        self.forward.is_empty() && self.prefix_rules.is_empty()
    }
}

impl Default for RetargetMap {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// UnmappedPolicy / RetargetConfig
// ---------------------------------------------------------------------------

/// How to handle source keys that have no forward mapping.
pub enum UnmappedPolicy {
    /// Silently drop the key.
    Drop,
    /// Pass the key through unchanged.
    PassThrough,
    /// Prepend `RetargetConfig::prefix` to the key.
    MapToPrefix,
}

/// Full configuration for [`retarget_weights`].
pub struct RetargetConfig {
    /// Multiplier applied to every output weight (default `1.0`).
    pub weight_scale: f32,
    /// Behaviour for keys absent from the [`RetargetMap`].
    pub unmapped_policy: UnmappedPolicy,
    /// Prefix used when `unmapped_policy` is [`UnmappedPolicy::MapToPrefix`].
    pub prefix: String,
    /// Clamp output weights to `[0.0, 1.0]`.
    pub clamp_output: bool,
}

impl Default for RetargetConfig {
    fn default() -> Self {
        Self {
            weight_scale: 1.0,
            unmapped_policy: UnmappedPolicy::Drop,
            prefix: String::new(),
            clamp_output: false,
        }
    }
}

// ---------------------------------------------------------------------------
// RetargetStats
// ---------------------------------------------------------------------------

/// Statistics produced by [`retarget_stats`].
pub struct RetargetStats {
    pub source_count: usize,
    pub mapped_count: usize,
    pub unmapped_count: usize,
    pub mapping_rate: f32,
}

// ---------------------------------------------------------------------------
// Standalone functions
// ---------------------------------------------------------------------------

/// Full retarget: apply map + config to `weights`.
pub fn retarget_weights(
    weights: &MorphWeights,
    map: &RetargetMap,
    config: &RetargetConfig,
) -> MorphWeights {
    let mut out = MorphWeights::new();
    for (k, &v) in weights {
        let mut val = v * config.weight_scale;
        if config.clamp_output {
            val = val.clamp(0.0, 1.0);
        }
        match map.forward_owned(k) {
            Some(mapped) => {
                out.insert(mapped, val);
            }
            None => match config.unmapped_policy {
                UnmappedPolicy::Drop => {}
                UnmappedPolicy::PassThrough => {
                    out.insert(k.clone(), val);
                }
                UnmappedPolicy::MapToPrefix => {
                    out.insert(format!("{}{}", config.prefix, k), val);
                }
            },
        }
    }
    out
}

/// Linear interpolation between two weight maps.
///
/// Keys present in either map are included; missing values are treated as 0.
pub fn blend_retargeted(source: &MorphWeights, target: &MorphWeights, t: f32) -> MorphWeights {
    let mut all_keys: Vec<String> = source.keys().cloned().collect();
    for k in target.keys() {
        if !source.contains_key(k.as_str()) {
            all_keys.push(k.clone());
        }
    }
    let mut out = MorphWeights::new();
    for k in all_keys {
        let a = source.get(k.as_str()).copied().unwrap_or(0.0);
        let b = target.get(k.as_str()).copied().unwrap_or(0.0);
        out.insert(k, a + (b - a) * t);
    }
    out
}

/// Multiply every weight in `weights` by `scale`.
pub fn scale_retarget_weights(weights: &MorphWeights, scale: f32) -> MorphWeights {
    weights
        .iter()
        .map(|(k, &v)| (k.clone(), v * scale))
        .collect()
}

/// Build a [`RetargetMap`] from prefix-pair rules.
///
/// For each `(src_prefix, tgt_prefix)` in `prefixes`, any source key that
/// starts with `src_prefix` maps to a target key where the prefix is replaced
/// with `tgt_prefix`.  The first matching rule wins.
///
/// # Example
/// ```
/// use oxihuman_morph::expression_retarget::build_prefix_map;
/// let map = build_prefix_map(&[("jaw_", "mouth_"), ("brow_", "brows_")]);
/// assert_eq!(map.forward_owned("jaw_open"), Some("mouth_open".to_owned()));
/// assert_eq!(map.forward_owned("brow_raise"), Some("brows_raise".to_owned()));
/// ```
pub fn build_prefix_map(prefixes: &[(&str, &str)]) -> RetargetMap {
    RetargetMap::new_with_prefix_rules(prefixes)
}

/// Compute mapping statistics for a source → retargeted weight pair.
pub fn retarget_stats(
    source: &MorphWeights,
    _retargeted: &MorphWeights,
    map: &RetargetMap,
) -> RetargetStats {
    let source_count = source.len();
    let mapped_count = source
        .keys()
        .filter(|k| map.forward_owned(k).is_some())
        .count();
    let unmapped_count = source_count - mapped_count;
    let mapping_rate = if source_count == 0 {
        0.0
    } else {
        mapped_count as f32 / source_count as f32
    };
    RetargetStats {
        source_count,
        mapped_count,
        unmapped_count,
        mapping_rate,
    }
}

// ---------------------------------------------------------------------------
// Factory functions
// ---------------------------------------------------------------------------

/// Map 10 common MakeHuman morph names to their DAZ Studio equivalents.
pub fn makehuman_to_daz_map() -> RetargetMap {
    let mut m = RetargetMap::new();
    m.add("jaw_open", "mouth_open");
    m.add("brow_raise_l", "brows_up_l");
    m.add("brow_raise_r", "brows_up_r");
    m.add("brow_lower_l", "brows_down_l");
    m.add("brow_lower_r", "brows_down_r");
    m.add("smile_l", "mouth_smile_l");
    m.add("smile_r", "mouth_smile_r");
    m.add("eye_blink_l", "eyes_closed_l");
    m.add("eye_blink_r", "eyes_closed_r");
    m.add("cheek_puff", "cheeks_puff");
    m
}

/// Map each key to itself (identity retarget).
pub fn identity_map(keys: &[&str]) -> RetargetMap {
    let mut m = RetargetMap::new();
    for &k in keys {
        m.add(k, k);
    }
    m
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn w(pairs: &[(&str, f32)]) -> MorphWeights {
        pairs.iter().map(|&(k, v)| (k.to_owned(), v)).collect()
    }

    // 1. RetargetMap::new creates empty map
    #[test]
    fn test_retarget_map_new_empty() {
        let m = RetargetMap::new();
        assert!(m.is_empty());
        assert_eq!(m.len(), 0);
    }

    // 2. add / forward / inverse round-trip
    #[test]
    fn test_add_forward_inverse() {
        let mut m = RetargetMap::new();
        m.add("jaw_open", "mouth_open");
        assert_eq!(m.forward("jaw_open"), Some("mouth_open"));
        assert_eq!(m.inverse("mouth_open"), Some("jaw_open"));
        assert_eq!(m.len(), 1);
    }

    // 3. forward returns None for unknown key
    #[test]
    fn test_forward_unknown() {
        let m = RetargetMap::new();
        assert_eq!(m.forward("nonexistent"), None);
    }

    // 4. inverse returns None for unknown key
    #[test]
    fn test_inverse_unknown() {
        let m = RetargetMap::new();
        assert_eq!(m.inverse("nonexistent"), None);
    }

    // 5. RetargetMap::retarget_weights maps keys correctly
    #[test]
    fn test_retarget_map_retarget_weights() {
        let mut m = RetargetMap::new();
        m.add("jaw_open", "mouth_open");
        m.add("smile_l", "mouth_smile_l");
        let src = w(&[("jaw_open", 0.8), ("smile_l", 0.5), ("unknown_key", 1.0)]);
        let out = m.retarget_weights(&src);
        assert!((out["mouth_open"] - 0.8).abs() < 1e-6);
        assert!((out["mouth_smile_l"] - 0.5).abs() < 1e-6);
        assert!(!out.contains_key("unknown_key"));
        assert_eq!(out.len(), 2);
    }

    // 6. inverse_retarget_weights
    #[test]
    fn test_inverse_retarget_weights() {
        let mut m = RetargetMap::new();
        m.add("jaw_open", "mouth_open");
        let tgt = w(&[("mouth_open", 0.7)]);
        let out = m.inverse_retarget_weights(&tgt);
        assert!((out["jaw_open"] - 0.7).abs() < 1e-6);
    }

    // 7. retarget_weights with UnmappedPolicy::Drop
    #[test]
    fn test_retarget_weights_drop() {
        let mut m = RetargetMap::new();
        m.add("jaw_open", "mouth_open");
        let src = w(&[("jaw_open", 0.6), ("extra", 0.3)]);
        let cfg = RetargetConfig {
            unmapped_policy: UnmappedPolicy::Drop,
            ..Default::default()
        };
        let out = retarget_weights(&src, &m, &cfg);
        assert!(out.contains_key("mouth_open"));
        assert!(!out.contains_key("extra"));
    }

    // 8. retarget_weights with UnmappedPolicy::PassThrough
    #[test]
    fn test_retarget_weights_passthrough() {
        let mut m = RetargetMap::new();
        m.add("jaw_open", "mouth_open");
        let src = w(&[("jaw_open", 0.6), ("extra", 0.3)]);
        let cfg = RetargetConfig {
            unmapped_policy: UnmappedPolicy::PassThrough,
            ..Default::default()
        };
        let out = retarget_weights(&src, &m, &cfg);
        assert!(out.contains_key("mouth_open"));
        assert!(out.contains_key("extra"));
        assert!((out["extra"] - 0.3).abs() < 1e-6);
    }

    // 9. retarget_weights with UnmappedPolicy::MapToPrefix
    #[test]
    fn test_retarget_weights_map_to_prefix() {
        let m = RetargetMap::new();
        let src = w(&[("smile", 0.4)]);
        let cfg = RetargetConfig {
            unmapped_policy: UnmappedPolicy::MapToPrefix,
            prefix: "pfx_".to_owned(),
            ..Default::default()
        };
        let out = retarget_weights(&src, &m, &cfg);
        assert!(out.contains_key("pfx_smile"));
        assert!((out["pfx_smile"] - 0.4).abs() < 1e-6);
    }

    // 10. retarget_weights applies weight_scale
    #[test]
    fn test_retarget_weights_scale() {
        let mut m = RetargetMap::new();
        m.add("a", "b");
        let src = w(&[("a", 0.5)]);
        let cfg = RetargetConfig {
            weight_scale: 2.0,
            unmapped_policy: UnmappedPolicy::Drop,
            ..Default::default()
        };
        let out = retarget_weights(&src, &m, &cfg);
        assert!((out["b"] - 1.0).abs() < 1e-6);
    }

    // 11. retarget_weights clamps output when clamp_output=true
    #[test]
    fn test_retarget_weights_clamp() {
        let mut m = RetargetMap::new();
        m.add("a", "b");
        let src = w(&[("a", 2.0)]);
        let cfg = RetargetConfig {
            weight_scale: 1.0,
            clamp_output: true,
            unmapped_policy: UnmappedPolicy::Drop,
            ..Default::default()
        };
        let out = retarget_weights(&src, &m, &cfg);
        assert!((out["b"] - 1.0).abs() < 1e-6);
    }

    // 12. blend_retargeted at t=0 returns source
    #[test]
    fn test_blend_retargeted_t0() {
        let s = w(&[("a", 0.3), ("b", 0.7)]);
        let t = w(&[("a", 1.0), ("b", 0.0)]);
        let out = blend_retargeted(&s, &t, 0.0);
        assert!((out["a"] - 0.3).abs() < 1e-6);
        assert!((out["b"] - 0.7).abs() < 1e-6);
    }

    // 13. blend_retargeted at t=1 returns target
    #[test]
    fn test_blend_retargeted_t1() {
        let s = w(&[("a", 0.3), ("b", 0.7)]);
        let t = w(&[("a", 1.0), ("b", 0.0)]);
        let out = blend_retargeted(&s, &t, 1.0);
        assert!((out["a"] - 1.0).abs() < 1e-6);
        assert!((out["b"] - 0.0).abs() < 1e-6);
    }

    // 14. blend_retargeted includes keys unique to either map
    #[test]
    fn test_blend_retargeted_union_keys() {
        let s = w(&[("only_src", 0.5)]);
        let t = w(&[("only_tgt", 0.8)]);
        let out = blend_retargeted(&s, &t, 0.5);
        assert!((out["only_src"] - 0.25).abs() < 1e-6);
        assert!((out["only_tgt"] - 0.4).abs() < 1e-6);
    }

    // 15. scale_retarget_weights
    #[test]
    fn test_scale_retarget_weights() {
        let src = w(&[("a", 0.4), ("b", 0.8)]);
        let out = scale_retarget_weights(&src, 0.5);
        assert!((out["a"] - 0.2).abs() < 1e-6);
        assert!((out["b"] - 0.4).abs() < 1e-6);
    }

    // 16. build_prefix_map resolves keys via forward_owned
    #[test]
    fn test_build_prefix_map() {
        let map = build_prefix_map(&[("jaw_", "mouth_"), ("brow_", "brows_")]);
        assert_eq!(map.forward_owned("jaw_open"), Some("mouth_open".to_owned()));
        assert_eq!(
            map.forward_owned("brow_raise_l"),
            Some("brows_raise_l".to_owned())
        );
        assert_eq!(map.forward_owned("unknown"), None);
    }

    // 17. build_prefix_map used with retarget_weights
    #[test]
    fn test_prefix_map_with_retarget_weights() {
        let map = build_prefix_map(&[("mh_", "daz_")]);
        let src = w(&[("mh_smile", 0.6), ("other", 0.2)]);
        let cfg = RetargetConfig {
            unmapped_policy: UnmappedPolicy::Drop,
            ..Default::default()
        };
        let out = retarget_weights(&src, &map, &cfg);
        assert!(out.contains_key("daz_smile"));
        assert!(!out.contains_key("other"));
    }

    // 18. retarget_stats basic
    #[test]
    fn test_retarget_stats() {
        let mut m = RetargetMap::new();
        m.add("jaw_open", "mouth_open");
        let src = w(&[("jaw_open", 0.8), ("unmapped", 0.2)]);
        let retargeted = m.retarget_weights(&src);
        let stats = retarget_stats(&src, &retargeted, &m);
        assert_eq!(stats.source_count, 2);
        assert_eq!(stats.mapped_count, 1);
        assert_eq!(stats.unmapped_count, 1);
        assert!((stats.mapping_rate - 0.5).abs() < 1e-6);
    }

    // 19. retarget_stats with empty source
    #[test]
    fn test_retarget_stats_empty() {
        let m = RetargetMap::new();
        let src = w(&[]);
        let retargeted = w(&[]);
        let stats = retarget_stats(&src, &retargeted, &m);
        assert_eq!(stats.source_count, 0);
        assert_eq!(stats.mapping_rate, 0.0);
    }

    // 20. makehuman_to_daz_map has 10 entries
    #[test]
    fn test_makehuman_to_daz_map_count() {
        let m = makehuman_to_daz_map();
        assert_eq!(m.len(), 10);
    }

    // 21. makehuman_to_daz_map spot-checks
    #[test]
    fn test_makehuman_to_daz_map_entries() {
        let m = makehuman_to_daz_map();
        assert_eq!(m.forward("jaw_open"), Some("mouth_open"));
        assert_eq!(m.forward("brow_raise_l"), Some("brows_up_l"));
        assert_eq!(m.forward("eye_blink_l"), Some("eyes_closed_l"));
        assert_eq!(m.forward("cheek_puff"), Some("cheeks_puff"));
    }

    // 22. identity_map maps keys to themselves
    #[test]
    fn test_identity_map() {
        let keys = ["smile", "blink", "jaw_open"];
        let m = identity_map(&keys);
        assert_eq!(m.len(), 3);
        assert_eq!(m.forward("smile"), Some("smile"));
        assert_eq!(m.inverse("blink"), Some("blink"));
    }

    // 23. identity_map retarget_weights preserves all keys
    #[test]
    fn test_identity_map_retarget_weights() {
        let keys = ["a", "b"];
        let m = identity_map(&keys);
        let src = w(&[("a", 0.3), ("b", 0.7)]);
        let out = m.retarget_weights(&src);
        assert!((out["a"] - 0.3).abs() < 1e-6);
        assert!((out["b"] - 0.7).abs() < 1e-6);
    }
}
