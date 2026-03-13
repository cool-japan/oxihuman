// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Skin texture scale morph — controls UV tiling and detail scale.

/// Skin texture scale morph configuration.
#[derive(Debug, Clone)]
pub struct SkinTextureScaleMorph {
    pub scale_u: f32,
    pub scale_v: f32,
    pub detail_scale: f32,
    pub micro_scale: f32,
}

impl SkinTextureScaleMorph {
    pub fn new() -> Self {
        Self {
            scale_u: 1.0,
            scale_v: 1.0,
            detail_scale: 1.0,
            micro_scale: 1.0,
        }
    }
}

impl Default for SkinTextureScaleMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new skin texture scale morph.
pub fn new_skin_texture_scale_morph() -> SkinTextureScaleMorph {
    SkinTextureScaleMorph::new()
}

/// Set uniform UV scale.
pub fn skin_tex_scale_set_uniform(morph: &mut SkinTextureScaleMorph, scale: f32) {
    let s = scale.clamp(0.01, 10.0);
    morph.scale_u = s;
    morph.scale_v = s;
}

/// Set U-axis scale independently.
pub fn skin_tex_scale_set_u(morph: &mut SkinTextureScaleMorph, scale: f32) {
    morph.scale_u = scale.clamp(0.01, 10.0);
}

/// Set V-axis scale independently.
pub fn skin_tex_scale_set_v(morph: &mut SkinTextureScaleMorph, scale: f32) {
    morph.scale_v = scale.clamp(0.01, 10.0);
}

/// Compute aspect ratio of the UV scale.
pub fn skin_tex_scale_aspect_ratio(morph: &SkinTextureScaleMorph) -> f32 {
    if morph.scale_v.abs() < 1e-6 {
        1.0
    } else {
        morph.scale_u / morph.scale_v
    }
}

/// Serialize to JSON-like string.
pub fn skin_texture_scale_morph_to_json(morph: &SkinTextureScaleMorph) -> String {
    format!(
        r#"{{"scale_u":{:.4},"scale_v":{:.4},"detail_scale":{:.4},"micro_scale":{:.4}}}"#,
        morph.scale_u, morph.scale_v, morph.detail_scale, morph.micro_scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_skin_texture_scale_morph();
        assert!((m.scale_u - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_uniform_scale() {
        let mut m = new_skin_texture_scale_morph();
        skin_tex_scale_set_uniform(&mut m, 2.0);
        assert!((m.scale_u - 2.0).abs() < 1e-6);
        assert!((m.scale_v - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_u_scale_clamp() {
        let mut m = new_skin_texture_scale_morph();
        skin_tex_scale_set_u(&mut m, 15.0);
        assert!((m.scale_u - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_v_scale_set() {
        let mut m = new_skin_texture_scale_morph();
        skin_tex_scale_set_v(&mut m, 0.5);
        assert!((m.scale_v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_aspect_ratio_default() {
        let m = new_skin_texture_scale_morph();
        assert!((skin_tex_scale_aspect_ratio(&m) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_aspect_ratio_nonuniform() {
        let mut m = new_skin_texture_scale_morph();
        skin_tex_scale_set_u(&mut m, 2.0);
        skin_tex_scale_set_v(&mut m, 1.0);
        assert!((skin_tex_scale_aspect_ratio(&m) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_json() {
        let m = new_skin_texture_scale_morph();
        let s = skin_texture_scale_morph_to_json(&m);
        assert!(s.contains("detail_scale"));
    }

    #[test]
    fn test_clone() {
        let m = new_skin_texture_scale_morph();
        let m2 = m.clone();
        assert!((m2.micro_scale - m.micro_scale).abs() < 1e-6);
    }

    #[test]
    fn test_default_trait() {
        let m: SkinTextureScaleMorph = Default::default();
        assert!((m.detail_scale - 1.0).abs() < 1e-6);
    }
}
