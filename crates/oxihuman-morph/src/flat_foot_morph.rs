// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Flat foot (pes planus / fallen arch) morph.

/// Flat foot configuration.
#[derive(Debug, Clone)]
pub struct FlatFootMorphConfig {
    pub arch_collapse: f32,
    pub pronation: f32,
    pub toe_splay: f32,
}

impl Default for FlatFootMorphConfig {
    fn default() -> Self {
        Self {
            arch_collapse: 0.0,
            pronation: 0.0,
            toe_splay: 0.0,
        }
    }
}

/// Flat foot morph state.
#[derive(Debug, Clone)]
pub struct FlatFootMorph {
    pub config: FlatFootMorphConfig,
    pub intensity: f32,
    pub left_foot: bool,
    pub right_foot: bool,
}

impl FlatFootMorph {
    pub fn new() -> Self {
        Self {
            config: FlatFootMorphConfig::default(),
            intensity: 0.0,
            left_foot: true,
            right_foot: true,
        }
    }
}

impl Default for FlatFootMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new FlatFootMorph.
pub fn new_flat_foot_morph() -> FlatFootMorph {
    FlatFootMorph::new()
}

/// Set arch collapse factor (0.0 = normal, 1.0 = fully flat).
pub fn flat_foot_set_arch_collapse(morph: &mut FlatFootMorph, v: f32) {
    morph.config.arch_collapse = v.clamp(0.0, 1.0);
}

/// Set pronation angle factor.
pub fn flat_foot_set_pronation(morph: &mut FlatFootMorph, v: f32) {
    morph.config.pronation = v.clamp(0.0, 1.0);
}

/// Set toe splay factor.
pub fn flat_foot_set_toe_splay(morph: &mut FlatFootMorph, v: f32) {
    morph.config.toe_splay = v.clamp(0.0, 1.0);
}

/// Compute effective arch height reduction.
pub fn flat_foot_arch_height(morph: &FlatFootMorph) -> f32 {
    let base_height = 1.0f32;
    base_height - morph.intensity * morph.config.arch_collapse
}

/// Serialize to JSON.
pub fn flat_foot_to_json(morph: &FlatFootMorph) -> String {
    format!(
        r#"{{"intensity":{},"arch_collapse":{},"pronation":{},"toe_splay":{}}}"#,
        morph.intensity, morph.config.arch_collapse, morph.config.pronation, morph.config.toe_splay,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_flat_foot_morph();
        assert!(m.left_foot /* left enabled by default */);
    }

    #[test]
    fn test_arch_collapse_clamp() {
        let mut m = new_flat_foot_morph();
        flat_foot_set_arch_collapse(&mut m, 2.0);
        assert!((m.config.arch_collapse - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_pronation() {
        let mut m = new_flat_foot_morph();
        flat_foot_set_pronation(&mut m, 0.6);
        assert!((m.config.pronation - 0.6).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_toe_splay() {
        let mut m = new_flat_foot_morph();
        flat_foot_set_toe_splay(&mut m, 0.3);
        assert!((m.config.toe_splay - 0.3).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_arch_height_normal() {
        let m = new_flat_foot_morph();
        assert!((flat_foot_arch_height(&m) - 1.0).abs() < 1e-6 /* full height */);
    }

    #[test]
    fn test_arch_height_flat() {
        let mut m = new_flat_foot_morph();
        flat_foot_set_arch_collapse(&mut m, 1.0);
        m.intensity = 1.0;
        assert!((flat_foot_arch_height(&m) - 0.0).abs() < 1e-6 /* fully flat */);
    }

    #[test]
    fn test_json_keys() {
        let m = new_flat_foot_morph();
        let j = flat_foot_to_json(&m);
        assert!(j.contains("arch_collapse") /* key */);
    }

    #[test]
    fn test_default_both_feet() {
        let m = FlatFootMorph::default();
        assert!(m.right_foot && m.left_foot /* both enabled */);
    }

    #[test]
    fn test_clone() {
        let m = new_flat_foot_morph();
        let c = m.clone();
        assert!((c.intensity - m.intensity).abs() < 1e-6 /* clone */);
    }
}
