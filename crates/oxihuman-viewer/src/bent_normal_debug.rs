// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bent normal debug visualizer — renders bent normal directions as colored overlays.

use std::f32::consts::FRAC_PI_2;

/// Bent normal visualization mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BentNormalMode {
    WorldSpace,
    TangentSpace,
    AoWeighted,
}

/// Configuration for bent normal debug display.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct BentNormalConfig {
    pub mode: BentNormalMode,
    pub scale: f32,
    pub ao_threshold: f32,
    pub enabled: bool,
}

impl Default for BentNormalConfig {
    fn default() -> Self {
        Self {
            mode: BentNormalMode::WorldSpace,
            scale: 1.0,
            ao_threshold: 0.5,
            enabled: false,
        }
    }
}

/// A single bent normal entry.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub struct BentNormalEntry {
    pub position: [f32; 3],
    pub bent_normal: [f32; 3],
    pub ao: f32,
}

/// Bent normal debug buffer.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct BentNormalDebug {
    pub config: BentNormalConfig,
    pub entries: Vec<BentNormalEntry>,
}

/// Create default config.
#[allow(dead_code)]
pub fn default_bent_normal_config() -> BentNormalConfig {
    BentNormalConfig::default()
}

/// Create new debug buffer.
#[allow(dead_code)]
pub fn new_bent_normal_debug(cfg: BentNormalConfig) -> BentNormalDebug {
    BentNormalDebug {
        config: cfg,
        entries: Vec::new(),
    }
}

/// Add an entry.
#[allow(dead_code)]
pub fn add_bent_normal(d: &mut BentNormalDebug, pos: [f32; 3], bn: [f32; 3], ao: f32) {
    d.entries.push(BentNormalEntry {
        position: pos,
        bent_normal: bn,
        ao: ao.clamp(0.0, 1.0),
    });
}

/// Clear all entries.
#[allow(dead_code)]
pub fn clear_bent_normals(d: &mut BentNormalDebug) {
    d.entries.clear();
}

/// Count of entries.
#[allow(dead_code)]
pub fn bent_normal_count(d: &BentNormalDebug) -> usize {
    d.entries.len()
}

/// Enable or disable the debug view.
#[allow(dead_code)]
pub fn set_bent_normal_enabled(d: &mut BentNormalDebug, enabled: bool) {
    d.config.enabled = enabled;
}

/// Filter entries by AO threshold.
#[allow(dead_code)]
pub fn filtered_entries(d: &BentNormalDebug) -> Vec<&BentNormalEntry> {
    d.entries
        .iter()
        .filter(|e| e.ao >= d.config.ao_threshold)
        .collect()
}

/// Normalize a 3D vector.
#[allow(dead_code)]
pub fn normalize_bn(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Convert bent normal to RGB color (map [-1,1] to `[0,1]`).
#[allow(dead_code)]
pub fn bent_normal_to_color(bn: [f32; 3]) -> [f32; 3] {
    [
        (bn[0] * 0.5 + 0.5).clamp(0.0, 1.0),
        (bn[1] * 0.5 + 0.5).clamp(0.0, 1.0),
        (bn[2] * 0.5 + 0.5).clamp(0.0, 1.0),
    ]
}

/// Compute angle between bent normal and surface normal using FRAC_PI_2 as reference.
#[allow(dead_code)]
pub fn bent_normal_angle(bn: [f32; 3], surface_normal: [f32; 3]) -> f32 {
    let dot = bn[0] * surface_normal[0] + bn[1] * surface_normal[1] + bn[2] * surface_normal[2];
    let angle = dot.clamp(-1.0, 1.0).acos();
    // normalize to [0,1] using FRAC_PI_2 as half-range
    angle / FRAC_PI_2
}

/// Export debug state to JSON-like string.
#[allow(dead_code)]
pub fn bent_normal_debug_to_json(d: &BentNormalDebug) -> String {
    format!(
        r#"{{"count":{},"enabled":{}}}"#,
        d.entries.len(),
        d.config.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_disabled() {
        assert!(!BentNormalConfig::default().enabled);
    }

    #[test]
    fn add_and_count() {
        let mut d = new_bent_normal_debug(default_bent_normal_config());
        add_bent_normal(&mut d, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.8);
        assert_eq!(bent_normal_count(&d), 1);
    }

    #[test]
    fn clear_empties() {
        let mut d = new_bent_normal_debug(default_bent_normal_config());
        add_bent_normal(&mut d, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        clear_bent_normals(&mut d);
        assert_eq!(bent_normal_count(&d), 0);
    }

    #[test]
    fn enable_sets_flag() {
        let mut d = new_bent_normal_debug(default_bent_normal_config());
        set_bent_normal_enabled(&mut d, true);
        assert!(d.config.enabled);
    }

    #[test]
    fn filter_by_ao() {
        let mut d = new_bent_normal_debug(BentNormalConfig {
            ao_threshold: 0.6,
            ..Default::default()
        });
        add_bent_normal(&mut d, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.9);
        add_bent_normal(&mut d, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.3);
        assert_eq!(filtered_entries(&d).len(), 1);
    }

    #[test]
    fn normalize_unit_vector() {
        let n = normalize_bn([0.0, 0.0, 1.0]);
        assert!((n[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_zero_returns_up() {
        let n = normalize_bn([0.0, 0.0, 0.0]);
        assert!((n[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn color_up_normal() {
        let c = bent_normal_to_color([0.0, 1.0, 0.0]);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn angle_parallel_is_zero() {
        let angle = bent_normal_angle([0.0, 1.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(angle < 1e-5);
    }

    #[test]
    fn json_contains_count() {
        let d = new_bent_normal_debug(default_bent_normal_config());
        assert!(bent_normal_debug_to_json(&d).contains("count"));
    }
}
