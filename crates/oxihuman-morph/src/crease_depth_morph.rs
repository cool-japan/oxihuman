// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Wrinkle/crease depth morph stub.

/// Crease region.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreaseRegion {
    Forehead,
    GlabellaLines,
    NasolabialFold,
    Marionette,
    CrowsFeet,
    NeckLines,
}

/// A crease entry with depth.
#[derive(Debug, Clone)]
pub struct CreaseEntry {
    pub region: CreaseRegion,
    pub depth: f32,
    pub sharpness: f32,
}

/// Crease depth morph controller.
#[derive(Debug, Clone)]
pub struct CreaseDepthMorph {
    pub creases: Vec<CreaseEntry>,
    pub global_scale: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl CreaseDepthMorph {
    pub fn new(morph_count: usize) -> Self {
        CreaseDepthMorph {
            creases: Vec::new(),
            global_scale: 1.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new crease depth morph.
pub fn new_crease_depth_morph(morph_count: usize) -> CreaseDepthMorph {
    CreaseDepthMorph::new(morph_count)
}

/// Add a crease entry.
pub fn cdm_add_crease(morph: &mut CreaseDepthMorph, entry: CreaseEntry) {
    morph.creases.push(entry);
}

/// Set global scale for all creases.
pub fn cdm_set_global_scale(morph: &mut CreaseDepthMorph, scale: f32) {
    morph.global_scale = scale.clamp(0.0, 2.0);
}

/// Clear all creases.
pub fn cdm_clear(morph: &mut CreaseDepthMorph) {
    morph.creases.clear();
}

/// Evaluate morph weights (stub: avg depth × global_scale).
pub fn cdm_evaluate(morph: &CreaseDepthMorph) -> Vec<f32> {
    /* Stub: average crease depth scaled by global_scale */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    let avg = if morph.creases.is_empty() {
        0.0
    } else {
        morph.creases.iter().map(|c| c.depth).sum::<f32>() / morph.creases.len() as f32
    };
    let w = (avg * morph.global_scale).clamp(0.0, 1.0);
    vec![w; morph.morph_count]
}

/// Enable or disable.
pub fn cdm_set_enabled(morph: &mut CreaseDepthMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Return crease count.
pub fn cdm_crease_count(morph: &CreaseDepthMorph) -> usize {
    morph.creases.len()
}

/// Serialize to JSON-like string.
pub fn cdm_to_json(morph: &CreaseDepthMorph) -> String {
    format!(
        r#"{{"crease_count":{},"global_scale":{},"morph_count":{},"enabled":{}}}"#,
        morph.creases.len(),
        morph.global_scale,
        morph.morph_count,
        morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_crease(region: CreaseRegion, depth: f32) -> CreaseEntry {
        CreaseEntry {
            region,
            depth,
            sharpness: 0.5,
        }
    }

    #[test]
    fn test_initial_empty() {
        let m = new_crease_depth_morph(4);
        assert_eq!(cdm_crease_count(&m), 0 /* no creases initially */);
    }

    #[test]
    fn test_add_crease() {
        let mut m = new_crease_depth_morph(4);
        cdm_add_crease(&mut m, make_crease(CreaseRegion::Forehead, 0.5));
        assert_eq!(cdm_crease_count(&m), 1 /* one crease added */);
    }

    #[test]
    fn test_clear() {
        let mut m = new_crease_depth_morph(4);
        cdm_add_crease(&mut m, make_crease(CreaseRegion::CrowsFeet, 0.8));
        cdm_clear(&mut m);
        assert_eq!(cdm_crease_count(&m), 0 /* cleared */);
    }

    #[test]
    fn test_global_scale_clamped() {
        let mut m = new_crease_depth_morph(4);
        cdm_set_global_scale(&mut m, 5.0);
        assert!((m.global_scale - 2.0).abs() < 1e-6 /* global_scale clamped to 2.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_crease_depth_morph(5);
        assert_eq!(
            cdm_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_empty_creases() {
        let m = new_crease_depth_morph(4);
        let out = cdm_evaluate(&m);
        assert!((out[0]).abs() < 1e-6 /* empty creases must give zero weight */);
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_crease_depth_morph(4);
        cdm_set_enabled(&mut m, false);
        assert!(cdm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_crease_count() {
        let m = new_crease_depth_morph(4);
        let j = cdm_to_json(&m);
        assert!(j.contains("\"crease_count\"") /* JSON must have crease_count */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_crease_depth_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_avg_depth() {
        let mut m = new_crease_depth_morph(2);
        cdm_add_crease(&mut m, make_crease(CreaseRegion::Marionette, 0.4));
        cdm_add_crease(&mut m, make_crease(CreaseRegion::NeckLines, 0.6));
        let out = cdm_evaluate(&m);
        assert!((out[0] - 0.5).abs() < 1e-5 /* avg depth must be 0.5 */);
    }
}
