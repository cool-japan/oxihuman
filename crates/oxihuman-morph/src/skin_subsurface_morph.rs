// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Subsurface scattering depth morph control.

/// Skin subsurface morph configuration.
#[derive(Debug, Clone)]
pub struct SkinSubsurfaceMorph {
    pub scatter_depth: f32,
    pub red_depth: f32,
    pub green_depth: f32,
    pub blue_depth: f32,
}

impl SkinSubsurfaceMorph {
    pub fn new() -> Self {
        Self {
            scatter_depth: 0.5,
            red_depth: 0.8,
            green_depth: 0.4,
            blue_depth: 0.2,
        }
    }
}

impl Default for SkinSubsurfaceMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new skin subsurface morph.
pub fn new_skin_subsurface_morph() -> SkinSubsurfaceMorph {
    SkinSubsurfaceMorph::new()
}

/// Set global scatter depth.
pub fn skin_sss_set_depth(morph: &mut SkinSubsurfaceMorph, depth: f32) {
    morph.scatter_depth = depth.clamp(0.0, 1.0);
}

/// Set per-channel depths for spectral scattering.
pub fn skin_sss_set_rgb_depths(morph: &mut SkinSubsurfaceMorph, r: f32, g: f32, b: f32) {
    morph.red_depth = r.clamp(0.0, 1.0);
    morph.green_depth = g.clamp(0.0, 1.0);
    morph.blue_depth = b.clamp(0.0, 1.0);
}

/// Set red channel depth.
pub fn skin_sss_set_red_depth(morph: &mut SkinSubsurfaceMorph, depth: f32) {
    morph.red_depth = depth.clamp(0.0, 1.0);
}

/// Compute mean chromatic depth across channels.
pub fn skin_sss_mean_depth(morph: &SkinSubsurfaceMorph) -> f32 {
    (morph.red_depth + morph.green_depth + morph.blue_depth) / 3.0
}

/// Serialize to JSON-like string.
pub fn skin_subsurface_morph_to_json(morph: &SkinSubsurfaceMorph) -> String {
    format!(
        r#"{{"scatter_depth":{:.4},"red_depth":{:.4},"green_depth":{:.4},"blue_depth":{:.4}}}"#,
        morph.scatter_depth, morph.red_depth, morph.green_depth, morph.blue_depth
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_skin_subsurface_morph();
        assert!((m.scatter_depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_depth_clamp() {
        let mut m = new_skin_subsurface_morph();
        skin_sss_set_depth(&mut m, 2.0);
        assert_eq!(m.scatter_depth, 1.0);
    }

    #[test]
    fn test_rgb_depths_set() {
        let mut m = new_skin_subsurface_morph();
        skin_sss_set_rgb_depths(&mut m, 0.9, 0.5, 0.1);
        assert!((m.red_depth - 0.9).abs() < 1e-6);
        assert!((m.blue_depth - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_red_depth_clamp() {
        let mut m = new_skin_subsurface_morph();
        skin_sss_set_red_depth(&mut m, -1.0);
        assert_eq!(m.red_depth, 0.0);
    }

    #[test]
    fn test_mean_depth() {
        let mut m = new_skin_subsurface_morph();
        skin_sss_set_rgb_depths(&mut m, 0.9, 0.6, 0.3);
        let mean = skin_sss_mean_depth(&m);
        assert!((mean - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_json() {
        let m = new_skin_subsurface_morph();
        let s = skin_subsurface_morph_to_json(&m);
        assert!(s.contains("scatter_depth"));
        assert!(s.contains("red_depth"));
    }

    #[test]
    fn test_clone() {
        let m = new_skin_subsurface_morph();
        let m2 = m.clone();
        assert!((m2.blue_depth - m.blue_depth).abs() < 1e-6);
    }

    #[test]
    fn test_default_trait() {
        let m: SkinSubsurfaceMorph = Default::default();
        assert!((m.green_depth - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_json_blue() {
        let m = new_skin_subsurface_morph();
        let s = skin_subsurface_morph_to_json(&m);
        assert!(s.contains("blue_depth"));
    }
}
