// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Procedural wrinkle generator stub.

/// Wrinkle pattern type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WrinklePattern {
    Linear,
    Radial,
    Noise,
}

/// A single procedural wrinkle region.
#[derive(Debug, Clone)]
pub struct WrinkleRegion {
    pub pattern: WrinklePattern,
    pub center: [f32; 3],
    pub radius: f32,
    pub amplitude: f32,
    pub frequency: f32,
    pub driver_weight: f32,
}

/// Procedural wrinkle generator.
#[derive(Debug, Clone)]
pub struct ProceduralWrinkle {
    pub regions: Vec<WrinkleRegion>,
    pub vertex_count: usize,
    pub enabled: bool,
    pub global_scale: f32,
}

impl ProceduralWrinkle {
    pub fn new(vertex_count: usize) -> Self {
        ProceduralWrinkle {
            regions: Vec::new(),
            vertex_count,
            enabled: true,
            global_scale: 1.0,
        }
    }
}

/// Create a new procedural wrinkle generator.
pub fn new_procedural_wrinkle(vertex_count: usize) -> ProceduralWrinkle {
    ProceduralWrinkle::new(vertex_count)
}

/// Add a wrinkle region.
pub fn pw_add_region(pw: &mut ProceduralWrinkle, region: WrinkleRegion) {
    pw.regions.push(region);
}

/// Evaluate wrinkle normals/offsets (stub: zeroed).
pub fn pw_evaluate(pw: &ProceduralWrinkle) -> Vec<[f32; 3]> {
    /* Stub: returns zeroed offset array */
    vec![[0.0; 3]; pw.vertex_count]
}

/// Set global scale.
pub fn pw_set_global_scale(pw: &mut ProceduralWrinkle, scale: f32) {
    pw.global_scale = scale.max(0.0);
}

/// Enable or disable.
pub fn pw_set_enabled(pw: &mut ProceduralWrinkle, enabled: bool) {
    pw.enabled = enabled;
}

/// Return region count.
pub fn pw_region_count(pw: &ProceduralWrinkle) -> usize {
    pw.regions.len()
}

/// Serialize to JSON-like string.
pub fn pw_to_json(pw: &ProceduralWrinkle) -> String {
    format!(
        r#"{{"vertex_count":{},"region_count":{},"global_scale":{},"enabled":{}}}"#,
        pw.vertex_count,
        pw.regions.len(),
        pw.global_scale,
        pw.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vertex_count() {
        let pw = new_procedural_wrinkle(200);
        assert_eq!(pw.vertex_count, 200 /* vertex count must match */,);
    }

    #[test]
    fn test_no_regions_initially() {
        let pw = new_procedural_wrinkle(100);
        assert_eq!(pw_region_count(&pw), 0 /* no regions initially */,);
    }

    #[test]
    fn test_add_region() {
        let mut pw = new_procedural_wrinkle(100);
        pw_add_region(
            &mut pw,
            WrinkleRegion {
                pattern: WrinklePattern::Linear,
                center: [0.0; 3],
                radius: 1.0,
                amplitude: 0.5,
                frequency: 2.0,
                driver_weight: 1.0,
            },
        );
        assert_eq!(pw_region_count(&pw), 1 /* one region after add */,);
    }

    #[test]
    fn test_evaluate_length() {
        let pw = new_procedural_wrinkle(50);
        let out = pw_evaluate(&pw);
        assert_eq!(
            out.len(),
            50, /* output length must match vertex count */
        );
    }

    #[test]
    fn test_evaluate_zeroed() {
        let pw = new_procedural_wrinkle(4);
        let out = pw_evaluate(&pw);
        assert!((out[0][0]).abs() < 1e-6 /* stub must return zeros */,);
    }

    #[test]
    fn test_set_global_scale() {
        let mut pw = new_procedural_wrinkle(10);
        pw_set_global_scale(&mut pw, 2.5);
        assert!((pw.global_scale - 2.5).abs() < 1e-5, /* scale must be set */);
    }

    #[test]
    fn test_global_scale_clamped_negative() {
        let mut pw = new_procedural_wrinkle(10);
        pw_set_global_scale(&mut pw, -1.0);
        assert!((pw.global_scale).abs() < 1e-6, /* negative scale clamped to 0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut pw = new_procedural_wrinkle(10);
        pw_set_enabled(&mut pw, false);
        assert!(!pw.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_vertex_count() {
        let pw = new_procedural_wrinkle(30);
        let j = pw_to_json(&pw);
        assert!(j.contains("\"vertex_count\""), /* json must contain vertex_count */);
    }

    #[test]
    fn test_enabled_default() {
        let pw = new_procedural_wrinkle(1);
        assert!(pw.enabled /* must be enabled by default */,);
    }
}
