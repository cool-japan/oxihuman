// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Post-processing effects chain (bloom, sharpen, SSAO stub, tone map, etc.).
//!
//! Pure data — no GPU calls.  Effects are stored as a sequence; calling
//! `chain_to_json` serialises the chain to a JSON string that a renderer
//! can consume.

#![allow(dead_code)]

use std::collections::HashMap;

// ── PostProcessEffect ──────────────────────────────────────────────────────────

/// A discrete post-processing effect variant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PostProcessEffect {
    /// Screen-space bloom / glow.
    Bloom,
    /// Unsharp-mask sharpening filter.
    Sharpen,
    /// Screen-space ambient occlusion (stub).
    Ssao,
    /// Tone mapping (Reinhard / ACES / filmic).
    ToneMap,
    /// Lateral chromatic aberration.
    ChromAberration,
    /// Screen-edge vignette darkening.
    Vignette,
}

impl PostProcessEffect {
    /// Return the canonical display name of the effect.
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            PostProcessEffect::Bloom => "Bloom",
            PostProcessEffect::Sharpen => "Sharpen",
            PostProcessEffect::Ssao => "SSAO",
            PostProcessEffect::ToneMap => "ToneMap",
            PostProcessEffect::ChromAberration => "ChromAberration",
            PostProcessEffect::Vignette => "Vignette",
        }
    }
}

// ── EffectState ────────────────────────────────────────────────────────────────

/// Runtime state for a single effect slot in the chain.
#[derive(Debug, Clone)]
pub struct EffectState {
    /// Whether this effect is active.
    pub enabled: bool,
    /// Named float parameters (e.g. `"intensity"`, `"threshold"`).
    pub params: HashMap<String, f32>,
}

impl Default for EffectState {
    fn default() -> Self {
        EffectState {
            enabled: true,
            params: HashMap::new(),
        }
    }
}

// ── PostProcessConfig ──────────────────────────────────────────────────────────

/// Configuration defaults for a new `PostProcessChain`.
#[derive(Debug, Clone)]
pub struct PostProcessConfig {
    /// JSON indentation width.  Default: `2`.
    pub json_indent: usize,
    /// Whether newly added effects default to enabled.  Default: `true`.
    pub default_enabled: bool,
}

impl Default for PostProcessConfig {
    fn default() -> Self {
        PostProcessConfig {
            json_indent: 2,
            default_enabled: true,
        }
    }
}

// ── PostProcessChain ───────────────────────────────────────────────────────────

/// An ordered list of post-processing effects with per-effect state.
#[derive(Debug, Clone)]
pub struct PostProcessChain {
    /// Ordered effect variants.
    pub effects: Vec<PostProcessEffect>,
    /// Per-effect state keyed by position in `effects`.
    pub states: Vec<EffectState>,
    /// Chain-level configuration.
    pub config: PostProcessConfig,
}

// ── Type aliases ───────────────────────────────────────────────────────────────

/// A parameter key-value pair.
pub type EffectParam = (String, f32);

/// Result of `apply_chain_stub`.
pub type ChainApplyResult = String;

// ── Constructor functions ──────────────────────────────────────────────────────

/// Return a default `PostProcessConfig`.
#[allow(dead_code)]
pub fn default_post_process_config() -> PostProcessConfig {
    PostProcessConfig::default()
}

/// Create an empty `PostProcessChain` with the given configuration.
#[allow(dead_code)]
pub fn new_post_process_chain(config: PostProcessConfig) -> PostProcessChain {
    PostProcessChain {
        effects: Vec::new(),
        states: Vec::new(),
        config,
    }
}

// ── Chain manipulation ─────────────────────────────────────────────────────────

/// Append an effect to the chain.  Returns the new effect's index.
#[allow(dead_code)]
pub fn add_effect(chain: &mut PostProcessChain, effect: PostProcessEffect) -> usize {
    let enabled = chain.config.default_enabled;
    chain.effects.push(effect);
    chain.states.push(EffectState {
        enabled,
        params: HashMap::new(),
    });
    chain.effects.len() - 1
}

/// Remove the effect at `idx`.  Returns `true` on success.
#[allow(dead_code)]
pub fn remove_effect(chain: &mut PostProcessChain, idx: usize) -> bool {
    if idx >= chain.effects.len() {
        return false;
    }
    chain.effects.remove(idx);
    chain.states.remove(idx);
    true
}

/// Return the number of effects in the chain.
#[allow(dead_code)]
pub fn effect_count(chain: &PostProcessChain) -> usize {
    chain.effects.len()
}

/// Reset the chain to empty, discarding all effects and state.
#[allow(dead_code)]
pub fn reset_chain(chain: &mut PostProcessChain) {
    chain.effects.clear();
    chain.states.clear();
}

// ── Per-effect parameter helpers ───────────────────────────────────────────────

/// Set a named float parameter on the effect at `idx`.
///
/// Returns `false` if `idx` is out of range.
#[allow(dead_code)]
pub fn set_effect_param(chain: &mut PostProcessChain, idx: usize, key: &str, value: f32) -> bool {
    if idx >= chain.states.len() {
        return false;
    }
    chain.states[idx].params.insert(key.to_string(), value);
    true
}

/// Get a named float parameter from the effect at `idx`.
///
/// Returns `None` if `idx` is out of range or the parameter is not set.
#[allow(dead_code)]
pub fn get_effect_param(chain: &PostProcessChain, idx: usize, key: &str) -> Option<f32> {
    chain.states.get(idx)?.params.get(key).copied()
}

// ── Enable / disable helpers ───────────────────────────────────────────────────

/// Return `true` if the effect at `idx` is enabled.
///
/// Returns `false` for out-of-range indices.
#[allow(dead_code)]
pub fn effect_enabled(chain: &PostProcessChain, idx: usize) -> bool {
    chain.states.get(idx).is_some_and(|s| s.enabled)
}

/// Toggle the enabled state of the effect at `idx`.
///
/// Returns the new enabled state, or `false` if `idx` is out of range.
#[allow(dead_code)]
pub fn toggle_effect(chain: &mut PostProcessChain, idx: usize) -> bool {
    if idx >= chain.states.len() {
        return false;
    }
    chain.states[idx].enabled = !chain.states[idx].enabled;
    chain.states[idx].enabled
}

// ── Effect name helper ─────────────────────────────────────────────────────────

/// Return the display name of the effect at `idx`, or `None` if out of range.
#[allow(dead_code)]
pub fn effect_name(chain: &PostProcessChain, idx: usize) -> Option<&'static str> {
    chain.effects.get(idx).map(PostProcessEffect::display_name)
}

// ── Serialisation ──────────────────────────────────────────────────────────────

/// Serialise an `EffectState` to a JSON object fragment (no outer braces).
fn state_to_json_inner(state: &EffectState) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("\"enabled\":{}", state.enabled));
    if !state.params.is_empty() {
        let mut sorted: Vec<(&String, &f32)> = state.params.iter().collect();
        sorted.sort_by_key(|(k, _)| k.as_str());
        let param_strs: Vec<String> = sorted
            .iter()
            .map(|(k, v)| format!("\"{}\":{:.6}", k, v))
            .collect();
        parts.push(format!("\"params\":{{{}}}", param_strs.join(",")));
    } else {
        parts.push("\"params\":{}".to_string());
    }
    parts.join(",")
}

/// Produce a JSON representation of the full chain.
#[allow(dead_code)]
pub fn chain_to_json(chain: &PostProcessChain) -> String {
    let mut effect_entries: Vec<String> = Vec::new();
    for (i, effect) in chain.effects.iter().enumerate() {
        let state_json = state_to_json_inner(&chain.states[i]);
        effect_entries.push(format!(
            "{{\"effect\":\"{}\",{}}}",
            effect.display_name(),
            state_json
        ));
    }
    format!("{{\"effects\":[{}]}}", effect_entries.join(","))
}

/// Stub: apply the chain to a framebuffer description.
///
/// In this data-only implementation the function just serialises the chain
/// and returns the JSON string (a real renderer would iterate effects).
#[allow(dead_code)]
pub fn apply_chain_stub(chain: &PostProcessChain) -> ChainApplyResult {
    chain_to_json(chain)
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chain() -> PostProcessChain {
        new_post_process_chain(default_post_process_config())
    }

    // 1 – default_post_process_config
    #[test]
    fn test_default_post_process_config() {
        let cfg = default_post_process_config();
        assert_eq!(cfg.json_indent, 2);
        assert!(cfg.default_enabled);
    }

    // 2 – new_post_process_chain is empty
    #[test]
    fn test_new_post_process_chain_empty() {
        let chain = make_chain();
        assert_eq!(effect_count(&chain), 0);
    }

    // 3 – add_effect returns index
    #[test]
    fn test_add_effect_returns_index() {
        let mut chain = make_chain();
        let idx = add_effect(&mut chain, PostProcessEffect::Bloom);
        assert_eq!(idx, 0);
        let idx2 = add_effect(&mut chain, PostProcessEffect::ToneMap);
        assert_eq!(idx2, 1);
    }

    // 4 – effect_count after adds
    #[test]
    fn test_effect_count() {
        let mut chain = make_chain();
        add_effect(&mut chain, PostProcessEffect::Bloom);
        add_effect(&mut chain, PostProcessEffect::Sharpen);
        assert_eq!(effect_count(&chain), 2);
    }

    // 5 – remove_effect success
    #[test]
    fn test_remove_effect_success() {
        let mut chain = make_chain();
        add_effect(&mut chain, PostProcessEffect::Bloom);
        add_effect(&mut chain, PostProcessEffect::Vignette);
        assert!(remove_effect(&mut chain, 0));
        assert_eq!(effect_count(&chain), 1);
    }

    // 6 – remove_effect out of range
    #[test]
    fn test_remove_effect_out_of_range() {
        let mut chain = make_chain();
        assert!(!remove_effect(&mut chain, 5));
    }

    // 7 – set_effect_param / get_effect_param
    #[test]
    fn test_set_get_effect_param() {
        let mut chain = make_chain();
        add_effect(&mut chain, PostProcessEffect::Bloom);
        assert!(set_effect_param(&mut chain, 0, "intensity", 0.5));
        let v = get_effect_param(&chain, 0, "intensity");
        assert!(v.is_some());
        assert!((v.unwrap() - 0.5).abs() < 1e-6);
    }

    // 8 – get_effect_param missing key
    #[test]
    fn test_get_effect_param_missing() {
        let mut chain = make_chain();
        add_effect(&mut chain, PostProcessEffect::Bloom);
        assert!(get_effect_param(&chain, 0, "nonexistent").is_none());
    }

    // 9 – effect_enabled default true
    #[test]
    fn test_effect_enabled_default_true() {
        let mut chain = make_chain();
        add_effect(&mut chain, PostProcessEffect::Ssao);
        assert!(effect_enabled(&chain, 0));
    }

    // 10 – toggle_effect
    #[test]
    fn test_toggle_effect() {
        let mut chain = make_chain();
        add_effect(&mut chain, PostProcessEffect::Bloom);
        assert!(effect_enabled(&chain, 0));
        let new_state = toggle_effect(&mut chain, 0);
        assert!(!new_state);
        assert!(!effect_enabled(&chain, 0));
        toggle_effect(&mut chain, 0);
        assert!(effect_enabled(&chain, 0));
    }

    // 11 – effect_name
    #[test]
    fn test_effect_name() {
        let mut chain = make_chain();
        add_effect(&mut chain, PostProcessEffect::ToneMap);
        assert_eq!(effect_name(&chain, 0), Some("ToneMap"));
    }

    // 12 – effect_name out of range
    #[test]
    fn test_effect_name_out_of_range() {
        let chain = make_chain();
        assert!(effect_name(&chain, 99).is_none());
    }

    // 13 – chain_to_json non-empty
    #[test]
    fn test_chain_to_json_non_empty() {
        let mut chain = make_chain();
        add_effect(&mut chain, PostProcessEffect::Bloom);
        let json = chain_to_json(&chain);
        assert!(!json.is_empty());
        assert!(json.contains("Bloom"));
    }

    // 14 – chain_to_json empty chain
    #[test]
    fn test_chain_to_json_empty() {
        let chain = make_chain();
        let json = chain_to_json(&chain);
        assert!(json.contains("\"effects\":[]"), "empty chain JSON wrong: {json}");
    }

    // 15 – chain_to_json contains params
    #[test]
    fn test_chain_to_json_with_params() {
        let mut chain = make_chain();
        let idx = add_effect(&mut chain, PostProcessEffect::Bloom);
        set_effect_param(&mut chain, idx, "threshold", 0.9);
        let json = chain_to_json(&chain);
        assert!(json.contains("threshold"), "param 'threshold' not in JSON");
    }

    // 16 – apply_chain_stub returns JSON
    #[test]
    fn test_apply_chain_stub_returns_json() {
        let mut chain = make_chain();
        add_effect(&mut chain, PostProcessEffect::ToneMap);
        let result = apply_chain_stub(&chain);
        assert!(result.contains("ToneMap"));
    }

    // 17 – reset_chain clears all
    #[test]
    fn test_reset_chain() {
        let mut chain = make_chain();
        add_effect(&mut chain, PostProcessEffect::Bloom);
        add_effect(&mut chain, PostProcessEffect::Sharpen);
        reset_chain(&mut chain);
        assert_eq!(effect_count(&chain), 0);
    }

    // 18 – PostProcessEffect display_name
    #[test]
    fn test_display_name_all_variants() {
        assert_eq!(PostProcessEffect::Bloom.display_name(), "Bloom");
        assert_eq!(PostProcessEffect::Sharpen.display_name(), "Sharpen");
        assert_eq!(PostProcessEffect::Ssao.display_name(), "SSAO");
        assert_eq!(PostProcessEffect::ToneMap.display_name(), "ToneMap");
        assert_eq!(PostProcessEffect::ChromAberration.display_name(), "ChromAberration");
        assert_eq!(PostProcessEffect::Vignette.display_name(), "Vignette");
    }

    // 19 – set_effect_param out of range returns false
    #[test]
    fn test_set_effect_param_out_of_range() {
        let mut chain = make_chain();
        assert!(!set_effect_param(&mut chain, 5, "k", 1.0));
    }

    // 20 – toggle_effect out of range returns false
    #[test]
    fn test_toggle_effect_out_of_range() {
        let mut chain = make_chain();
        assert!(!toggle_effect(&mut chain, 0));
    }

    // 21 – multiple effects preserve order
    #[test]
    fn test_effects_preserve_order() {
        let mut chain = make_chain();
        add_effect(&mut chain, PostProcessEffect::Bloom);
        add_effect(&mut chain, PostProcessEffect::Ssao);
        add_effect(&mut chain, PostProcessEffect::ToneMap);
        assert_eq!(chain.effects[0], PostProcessEffect::Bloom);
        assert_eq!(chain.effects[1], PostProcessEffect::Ssao);
        assert_eq!(chain.effects[2], PostProcessEffect::ToneMap);
    }

    // 22 – chain with disabled effect in JSON
    #[test]
    fn test_disabled_effect_in_json() {
        let mut chain = make_chain();
        add_effect(&mut chain, PostProcessEffect::Bloom);
        toggle_effect(&mut chain, 0);
        let json = chain_to_json(&chain);
        assert!(json.contains("\"enabled\":false"), "disabled state not in JSON: {json}");
    }

    // 23 – config default_enabled=false makes effects start disabled
    #[test]
    fn test_default_enabled_false() {
        let cfg = PostProcessConfig {
            default_enabled: false,
            ..PostProcessConfig::default()
        };
        let mut chain = new_post_process_chain(cfg);
        add_effect(&mut chain, PostProcessEffect::Vignette);
        assert!(!effect_enabled(&chain, 0));
    }
}
