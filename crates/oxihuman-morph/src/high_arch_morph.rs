// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! High arch (pes cavus) foot morph.

/// High arch configuration.
#[derive(Debug, Clone)]
pub struct HighArchMorphConfig {
    pub arch_rise: f32,
    pub supination: f32,
    pub claw_toe: f32,
}

impl Default for HighArchMorphConfig {
    fn default() -> Self {
        Self {
            arch_rise: 0.0,
            supination: 0.0,
            claw_toe: 0.0,
        }
    }
}

/// High arch morph state.
#[derive(Debug, Clone)]
pub struct HighArchMorph {
    pub config: HighArchMorphConfig,
    pub intensity: f32,
    pub enabled: bool,
}

impl HighArchMorph {
    pub fn new() -> Self {
        Self {
            config: HighArchMorphConfig::default(),
            intensity: 0.0,
            enabled: true,
        }
    }
}

impl Default for HighArchMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new HighArchMorph.
pub fn new_high_arch_morph() -> HighArchMorph {
    HighArchMorph::new()
}

/// Set arch rise factor (0.0–1.0).
pub fn high_arch_set_rise(morph: &mut HighArchMorph, v: f32) {
    morph.config.arch_rise = v.clamp(0.0, 1.0);
}

/// Set supination factor (0.0–1.0).
pub fn high_arch_set_supination(morph: &mut HighArchMorph, v: f32) {
    morph.config.supination = v.clamp(0.0, 1.0);
}

/// Set claw toe deformation factor.
pub fn high_arch_set_claw_toe(morph: &mut HighArchMorph, v: f32) {
    morph.config.claw_toe = v.clamp(0.0, 1.0);
}

/// Compute effective arch height (1.0 = normal, >1.0 = raised).
pub fn high_arch_height(morph: &HighArchMorph) -> f32 {
    1.0 + morph.intensity * morph.config.arch_rise
}

/// Serialize to JSON.
pub fn high_arch_to_json(morph: &HighArchMorph) -> String {
    format!(
        r#"{{"intensity":{},"arch_rise":{},"supination":{},"claw_toe":{}}}"#,
        morph.intensity, morph.config.arch_rise, morph.config.supination, morph.config.claw_toe,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_high_arch_morph();
        assert!((m.config.arch_rise - 0.0).abs() < 1e-6 /* default */);
    }

    #[test]
    fn test_rise_clamp() {
        let mut m = new_high_arch_morph();
        high_arch_set_rise(&mut m, 3.0);
        assert!((m.config.arch_rise - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_supination() {
        let mut m = new_high_arch_morph();
        high_arch_set_supination(&mut m, 0.5);
        assert!((m.config.supination - 0.5).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_claw_toe() {
        let mut m = new_high_arch_morph();
        high_arch_set_claw_toe(&mut m, 0.2);
        assert!((m.config.claw_toe - 0.2).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_height_normal() {
        let m = new_high_arch_morph();
        assert!((high_arch_height(&m) - 1.0).abs() < 1e-6 /* baseline */);
    }

    #[test]
    fn test_height_raised() {
        let mut m = new_high_arch_morph();
        high_arch_set_rise(&mut m, 1.0);
        m.intensity = 1.0;
        assert!(high_arch_height(&m) > 1.0 /* raised */);
    }

    #[test]
    fn test_json_keys() {
        let m = new_high_arch_morph();
        let j = high_arch_to_json(&m);
        assert!(j.contains("arch_rise") /* key */);
    }

    #[test]
    fn test_default_enabled() {
        let m = HighArchMorph::default();
        assert!(m.enabled /* enabled */);
    }

    #[test]
    fn test_clone() {
        let m = new_high_arch_morph();
        let c = m.clone();
        assert!((c.config.claw_toe - m.config.claw_toe).abs() < 1e-6 /* equal */);
    }
}
