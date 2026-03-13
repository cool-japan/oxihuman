// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mustache shape and density morph control.

/// Mustache morph configuration.
#[derive(Debug, Clone)]
pub struct MustacheMorph {
    pub density: f32,
    pub width: f32,
    pub droop: f32,
    pub thickness: f32,
}

impl MustacheMorph {
    pub fn new() -> Self {
        Self {
            density: 0.5,
            width: 0.5,
            droop: 0.0,
            thickness: 0.5,
        }
    }
}

impl Default for MustacheMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new mustache morph.
pub fn new_mustache_morph() -> MustacheMorph {
    MustacheMorph::new()
}

/// Set strand density.
pub fn mustache_set_density(morph: &mut MustacheMorph, density: f32) {
    morph.density = density.clamp(0.0, 1.0);
}

/// Set mustache width in normalized range [0, 1].
pub fn mustache_set_width(morph: &mut MustacheMorph, width: f32) {
    morph.width = width.clamp(0.0, 1.0);
}

/// Set downward droop factor in normalized range [0, 1].
pub fn mustache_set_droop(morph: &mut MustacheMorph, droop: f32) {
    morph.droop = droop.clamp(0.0, 1.0);
}

/// Compute a combined style score as a simple heuristic.
pub fn mustache_style_score(morph: &MustacheMorph) -> f32 {
    (morph.density + morph.width + morph.thickness) / 3.0
}

/// Serialize to JSON-like string.
pub fn mustache_morph_to_json(morph: &MustacheMorph) -> String {
    format!(
        r#"{{"density":{:.4},"width":{:.4},"droop":{:.4},"thickness":{:.4}}}"#,
        morph.density, morph.width, morph.droop, morph.thickness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let m = new_mustache_morph();
        assert!((m.droop - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_density_clamp() {
        let mut m = new_mustache_morph();
        mustache_set_density(&mut m, 5.0);
        assert_eq!(m.density, 1.0);
    }

    #[test]
    fn test_width_set() {
        let mut m = new_mustache_morph();
        mustache_set_width(&mut m, 0.75);
        assert!((m.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_droop_set() {
        let mut m = new_mustache_morph();
        mustache_set_droop(&mut m, 0.6);
        assert!((m.droop - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_style_score_range() {
        let m = new_mustache_morph();
        let s = mustache_style_score(&m);
        assert!((0.0..=1.0).contains(&s));
    }

    #[test]
    fn test_json() {
        let m = new_mustache_morph();
        let s = mustache_morph_to_json(&m);
        assert!(s.contains("droop"));
    }

    #[test]
    fn test_clone() {
        let m = new_mustache_morph();
        let m2 = m.clone();
        assert!((m2.density - m.density).abs() < 1e-6);
    }

    #[test]
    fn test_droop_clamp() {
        let mut m = new_mustache_morph();
        mustache_set_droop(&mut m, -2.0);
        assert_eq!(m.droop, 0.0);
    }

    #[test]
    fn test_default_trait() {
        let m: MustacheMorph = Default::default();
        assert!((m.thickness - 0.5).abs() < 1e-6);
    }
}
