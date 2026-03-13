// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Sacrum tilt morph — adjusts sacral base angle and width.

/// Sacrum morph configuration.
#[derive(Debug, Clone)]
pub struct SacrumMorph {
    pub tilt: f32,
    pub width: f32,
    pub curvature: f32,
    pub promontory_depth: f32,
}

impl SacrumMorph {
    pub fn new() -> Self {
        Self {
            tilt: 0.5,
            width: 0.5,
            curvature: 0.5,
            promontory_depth: 0.5,
        }
    }
}

impl Default for SacrumMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new sacrum morph.
pub fn new_sacrum_morph() -> SacrumMorph {
    SacrumMorph::new()
}

/// Set sacral tilt (0 = anterior, 1 = posterior).
pub fn sac_set_tilt(m: &mut SacrumMorph, v: f32) {
    m.tilt = v.clamp(0.0, 1.0);
}

/// Set sacrum width.
pub fn sac_set_width(m: &mut SacrumMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

/// Set sacral curvature.
pub fn sac_set_curvature(m: &mut SacrumMorph, v: f32) {
    m.curvature = v.clamp(0.0, 1.0);
}

/// Set sacral promontory depth.
pub fn sac_set_promontory_depth(m: &mut SacrumMorph, v: f32) {
    m.promontory_depth = v.clamp(0.0, 1.0);
}

/// Compute pelvic inlet estimate based on tilt and width.
pub fn sac_pelvic_inlet(m: &SacrumMorph) -> f32 {
    m.width * (1.0 - (m.tilt - 0.5).abs() * 0.3)
}

/// Serialize to JSON-like string.
pub fn sacrum_morph_to_json(m: &SacrumMorph) -> String {
    format!(
        r#"{{"tilt":{:.4},"width":{:.4},"curvature":{:.4},"promontory_depth":{:.4}}}"#,
        m.tilt, m.width, m.curvature, m.promontory_depth
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_sacrum_morph();
        assert!((m.tilt - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_tilt_clamp() {
        let mut m = new_sacrum_morph();
        sac_set_tilt(&mut m, -1.0);
        assert_eq!(m.tilt, 0.0);
    }

    #[test]
    fn test_width_set() {
        let mut m = new_sacrum_morph();
        sac_set_width(&mut m, 0.7);
        assert!((m.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_curvature_clamp() {
        let mut m = new_sacrum_morph();
        sac_set_curvature(&mut m, 3.0);
        assert_eq!(m.curvature, 1.0);
    }

    #[test]
    fn test_promontory_depth() {
        let mut m = new_sacrum_morph();
        sac_set_promontory_depth(&mut m, 0.6);
        assert!((m.promontory_depth - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_pelvic_inlet_range() {
        let m = new_sacrum_morph();
        let pi = sac_pelvic_inlet(&m);
        assert!((0.0..=1.0).contains(&pi));
    }

    #[test]
    fn test_json_keys() {
        let m = new_sacrum_morph();
        let s = sacrum_morph_to_json(&m);
        assert!(s.contains("promontory_depth"));
    }

    #[test]
    fn test_clone() {
        let m = new_sacrum_morph();
        let m2 = m.clone();
        assert!((m2.width - m.width).abs() < 1e-6);
    }

    #[test]
    fn test_pelvic_inlet_wide() {
        let mut m = new_sacrum_morph();
        sac_set_width(&mut m, 1.0);
        sac_set_tilt(&mut m, 0.5); /* neutral tilt = maximum inlet */
        assert!(sac_pelvic_inlet(&m) > 0.9);
    }
}
