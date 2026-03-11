// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Temporal region shape morph — controls temporal fossa hollowing and width.

/// Temporal region morph configuration.
#[derive(Debug, Clone)]
pub struct TemporalRegionMorph {
    pub hollowing: f32,
    pub width: f32,
    pub superior_extent: f32,
    pub muscle_fullness: f32,
    pub crest_height: f32,
}

impl TemporalRegionMorph {
    pub fn new() -> Self {
        Self {
            hollowing: 0.3,
            width: 0.5,
            superior_extent: 0.5,
            muscle_fullness: 0.5,
            crest_height: 0.4,
        }
    }
}

impl Default for TemporalRegionMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new temporal region morph.
pub fn new_temporal_region_morph() -> TemporalRegionMorph {
    TemporalRegionMorph::new()
}

/// Set temporal hollowing (0 = full, 1 = deeply hollow).
pub fn temp_set_hollowing(m: &mut TemporalRegionMorph, v: f32) {
    m.hollowing = v.clamp(0.0, 1.0);
}

/// Set temporal width.
pub fn temp_set_width(m: &mut TemporalRegionMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

/// Set superior extent toward the temporal line.
pub fn temp_set_superior_extent(m: &mut TemporalRegionMorph, v: f32) {
    m.superior_extent = v.clamp(0.0, 1.0);
}

/// Set temporal muscle fullness.
pub fn temp_set_muscle_fullness(m: &mut TemporalRegionMorph, v: f32) {
    m.muscle_fullness = v.clamp(0.0, 1.0);
}

/// Compute effective temporal volume (accounting for hollowing).
pub fn temp_effective_volume(m: &TemporalRegionMorph) -> f32 {
    let raw = m.width * m.muscle_fullness;
    raw * (1.0 - m.hollowing * 0.7)
}

/// Serialize to JSON-like string.
pub fn temporal_region_morph_to_json(m: &TemporalRegionMorph) -> String {
    format!(
        r#"{{"hollowing":{:.4},"width":{:.4},"superior_extent":{:.4},"muscle_fullness":{:.4},"crest_height":{:.4}}}"#,
        m.hollowing, m.width, m.superior_extent, m.muscle_fullness, m.crest_height
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_temporal_region_morph();
        assert!((m.hollowing - 0.3).abs() < 1e-6);
        assert!((m.muscle_fullness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_hollowing_clamp() {
        let mut m = new_temporal_region_morph();
        temp_set_hollowing(&mut m, 3.0);
        assert_eq!(m.hollowing, 1.0);
    }

    #[test]
    fn test_hollowing_set() {
        let mut m = new_temporal_region_morph();
        temp_set_hollowing(&mut m, 0.6);
        assert!((m.hollowing - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_width_clamp_low() {
        let mut m = new_temporal_region_morph();
        temp_set_width(&mut m, -1.0);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_superior_extent_set() {
        let mut m = new_temporal_region_morph();
        temp_set_superior_extent(&mut m, 0.8);
        assert!((m.superior_extent - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_muscle_fullness_set() {
        let mut m = new_temporal_region_morph();
        temp_set_muscle_fullness(&mut m, 0.9);
        assert!((m.muscle_fullness - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_effective_volume_positive() {
        let m = new_temporal_region_morph();
        assert!(temp_effective_volume(&m) > 0.0);
    }

    #[test]
    fn test_effective_volume_reduces_with_hollowing() {
        let mut m = new_temporal_region_morph();
        let v0 = temp_effective_volume(&m);
        temp_set_hollowing(&mut m, 1.0);
        let v1 = temp_effective_volume(&m);
        assert!(v1 < v0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_temporal_region_morph();
        let s = temporal_region_morph_to_json(&m);
        assert!(s.contains("crest_height"));
    }

    #[test]
    fn test_clone() {
        let m = new_temporal_region_morph();
        let m2 = m.clone();
        assert!((m2.hollowing - m.hollowing).abs() < 1e-6);
    }
}
