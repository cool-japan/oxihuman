// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Ethnic morphology variation presets and blend system.
//!
//! Provides ethnic group presets and blending utilities for morphological
//! variation across different human ethnic backgrounds.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum EthnicGroup {
    EastAsian,
    SouthAsian,
    African,
    European,
    MiddleEastern,
    Latino,
    Other,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EthnicPreset {
    pub group: EthnicGroup,
    pub name: String,
    pub eye_shape: f32,
    pub nose_width: f32,
    pub nose_bridge_height: f32,
    pub lip_fullness: f32,
    pub cheekbone_prominence: f32,
    pub jaw_width: f32,
    pub brow_height: f32,
    pub face_width: f32,
    pub chin_projection: f32,
    pub eye_spacing: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EthnicMorphConfig {
    pub weights: Vec<f32>,
    pub presets: Vec<EthnicPreset>,
    pub normalize_on_set: bool,
    pub smooth_blend: bool,
}

/// Returns the canonical name string for an ethnic group.
#[allow(dead_code)]
pub fn ethnic_group_name(group: &EthnicGroup) -> &'static str {
    match group {
        EthnicGroup::EastAsian => "East Asian",
        EthnicGroup::SouthAsian => "South Asian",
        EthnicGroup::African => "African",
        EthnicGroup::European => "European",
        EthnicGroup::MiddleEastern => "Middle Eastern",
        EthnicGroup::Latino => "Latino",
        EthnicGroup::Other => "Other",
    }
}

/// Returns the number of ethnic groups (excluding `Other`).
#[allow(dead_code)]
pub fn ethnic_weight_count() -> usize {
    6
}

/// Constructs a default `EthnicMorphConfig` with equal weights.
#[allow(dead_code)]
pub fn default_ethnic_config() -> EthnicMorphConfig {
    let n = ethnic_weight_count() + 1; // include Other
    EthnicMorphConfig {
        weights: vec![1.0 / n as f32; n],
        presets: vec![
            new_ethnic_preset(EthnicGroup::EastAsian),
            new_ethnic_preset(EthnicGroup::SouthAsian),
            new_ethnic_preset(EthnicGroup::African),
            new_ethnic_preset(EthnicGroup::European),
            new_ethnic_preset(EthnicGroup::MiddleEastern),
            new_ethnic_preset(EthnicGroup::Latino),
            new_ethnic_preset(EthnicGroup::Other),
        ],
        normalize_on_set: true,
        smooth_blend: true,
    }
}

/// Creates a morphological preset for a given ethnic group.
#[allow(dead_code)]
pub fn new_ethnic_preset(group: EthnicGroup) -> EthnicPreset {
    let name = ethnic_group_name(&group).to_string();
    match group {
        EthnicGroup::EastAsian => EthnicPreset {
            group: EthnicGroup::EastAsian,
            name,
            eye_shape: 0.7,
            nose_width: 0.45,
            nose_bridge_height: 0.3,
            lip_fullness: 0.5,
            cheekbone_prominence: 0.6,
            jaw_width: 0.55,
            brow_height: 0.45,
            face_width: 0.55,
            chin_projection: 0.35,
            eye_spacing: 0.5,
        },
        EthnicGroup::SouthAsian => EthnicPreset {
            group: EthnicGroup::SouthAsian,
            name,
            eye_shape: 0.55,
            nose_width: 0.55,
            nose_bridge_height: 0.4,
            lip_fullness: 0.6,
            cheekbone_prominence: 0.5,
            jaw_width: 0.5,
            brow_height: 0.5,
            face_width: 0.5,
            chin_projection: 0.4,
            eye_spacing: 0.5,
        },
        EthnicGroup::African => EthnicPreset {
            group: EthnicGroup::African,
            name,
            eye_shape: 0.45,
            nose_width: 0.7,
            nose_bridge_height: 0.25,
            lip_fullness: 0.8,
            cheekbone_prominence: 0.55,
            jaw_width: 0.55,
            brow_height: 0.55,
            face_width: 0.55,
            chin_projection: 0.35,
            eye_spacing: 0.52,
        },
        EthnicGroup::European => EthnicPreset {
            group: EthnicGroup::European,
            name,
            eye_shape: 0.4,
            nose_width: 0.4,
            nose_bridge_height: 0.6,
            lip_fullness: 0.45,
            cheekbone_prominence: 0.45,
            jaw_width: 0.5,
            brow_height: 0.5,
            face_width: 0.5,
            chin_projection: 0.5,
            eye_spacing: 0.5,
        },
        EthnicGroup::MiddleEastern => EthnicPreset {
            group: EthnicGroup::MiddleEastern,
            name,
            eye_shape: 0.5,
            nose_width: 0.5,
            nose_bridge_height: 0.55,
            lip_fullness: 0.55,
            cheekbone_prominence: 0.5,
            jaw_width: 0.5,
            brow_height: 0.48,
            face_width: 0.5,
            chin_projection: 0.45,
            eye_spacing: 0.5,
        },
        EthnicGroup::Latino => EthnicPreset {
            group: EthnicGroup::Latino,
            name,
            eye_shape: 0.5,
            nose_width: 0.55,
            nose_bridge_height: 0.45,
            lip_fullness: 0.6,
            cheekbone_prominence: 0.55,
            jaw_width: 0.52,
            brow_height: 0.5,
            face_width: 0.52,
            chin_projection: 0.42,
            eye_spacing: 0.5,
        },
        EthnicGroup::Other => EthnicPreset {
            group: EthnicGroup::Other,
            name,
            eye_shape: 0.5,
            nose_width: 0.5,
            nose_bridge_height: 0.5,
            lip_fullness: 0.5,
            cheekbone_prominence: 0.5,
            jaw_width: 0.5,
            brow_height: 0.5,
            face_width: 0.5,
            chin_projection: 0.5,
            eye_spacing: 0.5,
        },
    }
}

/// Blends two ethnic presets by weight `t` in [0,1].
#[allow(dead_code)]
pub fn blend_ethnic_presets(a: &EthnicPreset, b: &EthnicPreset, t: f32) -> EthnicPreset {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    EthnicPreset {
        group: if t < 0.5 { a.group.clone() } else { b.group.clone() },
        name: format!("{}/{}", a.name, b.name),
        eye_shape: lerp(a.eye_shape, b.eye_shape),
        nose_width: lerp(a.nose_width, b.nose_width),
        nose_bridge_height: lerp(a.nose_bridge_height, b.nose_bridge_height),
        lip_fullness: lerp(a.lip_fullness, b.lip_fullness),
        cheekbone_prominence: lerp(a.cheekbone_prominence, b.cheekbone_prominence),
        jaw_width: lerp(a.jaw_width, b.jaw_width),
        brow_height: lerp(a.brow_height, b.brow_height),
        face_width: lerp(a.face_width, b.face_width),
        chin_projection: lerp(a.chin_projection, b.chin_projection),
        eye_spacing: lerp(a.eye_spacing, b.eye_spacing),
    }
}

/// Returns a flat weight map representing this preset's feature values.
#[allow(dead_code)]
pub fn ethnic_to_morph_weights(preset: &EthnicPreset) -> Vec<(String, f32)> {
    vec![
        ("ethnic_eye_shape".to_string(), preset.eye_shape),
        ("ethnic_nose_width".to_string(), preset.nose_width),
        ("ethnic_nose_bridge_height".to_string(), preset.nose_bridge_height),
        ("ethnic_lip_fullness".to_string(), preset.lip_fullness),
        ("ethnic_cheekbone_prominence".to_string(), preset.cheekbone_prominence),
        ("ethnic_jaw_width".to_string(), preset.jaw_width),
        ("ethnic_brow_height".to_string(), preset.brow_height),
        ("ethnic_face_width".to_string(), preset.face_width),
        ("ethnic_chin_projection".to_string(), preset.chin_projection),
        ("ethnic_eye_spacing".to_string(), preset.eye_spacing),
    ]
}

/// Sets a specific ethnic group's blend weight in the config.
#[allow(dead_code)]
pub fn set_ethnic_blend(config: &mut EthnicMorphConfig, group_index: usize, weight: f32) {
    if group_index < config.weights.len() {
        config.weights[group_index] = weight.max(0.0);
        if config.normalize_on_set {
            normalize_ethnic_weights(config);
        }
    }
}

/// Normalizes the ethnic weights so they sum to 1.0.
#[allow(dead_code)]
pub fn normalize_ethnic_weights(config: &mut EthnicMorphConfig) {
    let total: f32 = config.weights.iter().sum();
    if total > 1e-6 {
        for w in &mut config.weights {
            *w /= total;
        }
    }
}

/// Serializes a preset to a simple JSON-like string.
#[allow(dead_code)]
pub fn preset_to_json(preset: &EthnicPreset) -> String {
    format!(
        r#"{{"group":"{}","eye_shape":{:.3},"nose_width":{:.3},"nose_bridge_height":{:.3},"lip_fullness":{:.3},"cheekbone_prominence":{:.3},"jaw_width":{:.3},"brow_height":{:.3},"face_width":{:.3},"chin_projection":{:.3},"eye_spacing":{:.3}}}"#,
        preset.name,
        preset.eye_shape,
        preset.nose_width,
        preset.nose_bridge_height,
        preset.lip_fullness,
        preset.cheekbone_prominence,
        preset.jaw_width,
        preset.brow_height,
        preset.face_width,
        preset.chin_projection,
        preset.eye_spacing,
    )
}

/// Returns a compact float vector representation of a preset.
#[allow(dead_code)]
pub fn ethnic_feature_vector(preset: &EthnicPreset) -> [f32; 10] {
    [
        preset.eye_shape,
        preset.nose_width,
        preset.nose_bridge_height,
        preset.lip_fullness,
        preset.cheekbone_prominence,
        preset.jaw_width,
        preset.brow_height,
        preset.face_width,
        preset.chin_projection,
        preset.eye_spacing,
    ]
}

/// Determines the dominant ethnic group based on blend weights.
#[allow(dead_code)]
pub fn dominant_ethnic_group(config: &EthnicMorphConfig) -> Option<usize> {
    if config.weights.is_empty() {
        return None;
    }
    let mut max_idx = 0;
    let mut max_val = config.weights[0];
    for (i, &w) in config.weights.iter().enumerate().skip(1) {
        if w > max_val {
            max_val = w;
            max_idx = i;
        }
    }
    Some(max_idx)
}

/// Applies ethnic morphology to face parameters (returns a weight map).
#[allow(dead_code)]
pub fn apply_ethnic_to_face(config: &EthnicMorphConfig) -> Vec<(String, f32)> {
    // Weighted blend across all presets
    let n = config.presets.len().min(config.weights.len());
    if n == 0 {
        return vec![];
    }
    let mut eye_shape = 0.0f32;
    let mut nose_width = 0.0f32;
    let mut nose_bridge_height = 0.0f32;
    let mut lip_fullness = 0.0f32;
    let mut cheekbone_prominence = 0.0f32;
    let mut jaw_width = 0.0f32;
    let mut brow_height = 0.0f32;
    let mut face_width = 0.0f32;
    let mut chin_projection = 0.0f32;
    let mut eye_spacing = 0.0f32;

    for i in 0..n {
        let w = config.weights[i];
        let p = &config.presets[i];
        eye_shape += w * p.eye_shape;
        nose_width += w * p.nose_width;
        nose_bridge_height += w * p.nose_bridge_height;
        lip_fullness += w * p.lip_fullness;
        cheekbone_prominence += w * p.cheekbone_prominence;
        jaw_width += w * p.jaw_width;
        brow_height += w * p.brow_height;
        face_width += w * p.face_width;
        chin_projection += w * p.chin_projection;
        eye_spacing += w * p.eye_spacing;
    }

    vec![
        ("ethnic_eye_shape".to_string(), eye_shape),
        ("ethnic_nose_width".to_string(), nose_width),
        ("ethnic_nose_bridge_height".to_string(), nose_bridge_height),
        ("ethnic_lip_fullness".to_string(), lip_fullness),
        ("ethnic_cheekbone_prominence".to_string(), cheekbone_prominence),
        ("ethnic_jaw_width".to_string(), jaw_width),
        ("ethnic_brow_height".to_string(), brow_height),
        ("ethnic_face_width".to_string(), face_width),
        ("ethnic_chin_projection".to_string(), chin_projection),
        ("ethnic_eye_spacing".to_string(), eye_spacing),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethnic_group_name() {
        assert_eq!(ethnic_group_name(&EthnicGroup::EastAsian), "East Asian");
        assert_eq!(ethnic_group_name(&EthnicGroup::African), "African");
        assert_eq!(ethnic_group_name(&EthnicGroup::European), "European");
        assert_eq!(ethnic_group_name(&EthnicGroup::Latino), "Latino");
        assert_eq!(ethnic_group_name(&EthnicGroup::Other), "Other");
    }

    #[test]
    fn test_ethnic_weight_count() {
        assert_eq!(ethnic_weight_count(), 6);
    }

    #[test]
    fn test_default_ethnic_config() {
        let config = default_ethnic_config();
        assert_eq!(config.presets.len(), 7);
        assert_eq!(config.weights.len(), 7);
        let sum: f32 = config.weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_new_ethnic_preset_east_asian() {
        let p = new_ethnic_preset(EthnicGroup::EastAsian);
        assert_eq!(p.name, "East Asian");
        assert!(p.eye_shape > 0.0 && p.eye_shape <= 1.0);
    }

    #[test]
    fn test_new_ethnic_preset_african() {
        let p = new_ethnic_preset(EthnicGroup::African);
        assert_eq!(p.name, "African");
        assert!(p.lip_fullness > 0.5); // African preset should have fuller lips
    }

    #[test]
    fn test_blend_ethnic_presets_t0() {
        let a = new_ethnic_preset(EthnicGroup::European);
        let b = new_ethnic_preset(EthnicGroup::EastAsian);
        let blended = blend_ethnic_presets(&a, &b, 0.0);
        assert!((blended.eye_shape - a.eye_shape).abs() < 1e-5);
    }

    #[test]
    fn test_blend_ethnic_presets_t1() {
        let a = new_ethnic_preset(EthnicGroup::European);
        let b = new_ethnic_preset(EthnicGroup::EastAsian);
        let blended = blend_ethnic_presets(&a, &b, 1.0);
        assert!((blended.eye_shape - b.eye_shape).abs() < 1e-5);
    }

    #[test]
    fn test_blend_ethnic_presets_midpoint() {
        let a = new_ethnic_preset(EthnicGroup::European);
        let b = new_ethnic_preset(EthnicGroup::African);
        let blended = blend_ethnic_presets(&a, &b, 0.5);
        let expected_lip = (a.lip_fullness + b.lip_fullness) * 0.5;
        assert!((blended.lip_fullness - expected_lip).abs() < 1e-5);
    }

    #[test]
    fn test_ethnic_to_morph_weights() {
        let p = new_ethnic_preset(EthnicGroup::European);
        let weights = ethnic_to_morph_weights(&p);
        assert_eq!(weights.len(), 10);
        assert!(weights.iter().any(|(k, _)| k == "ethnic_eye_shape"));
    }

    #[test]
    fn test_set_ethnic_blend() {
        let mut config = default_ethnic_config();
        set_ethnic_blend(&mut config, 0, 1.0);
        // After normalize, weight[0] should be highest or near 1
        assert!(config.weights[0] > 0.0);
    }

    #[test]
    fn test_normalize_ethnic_weights() {
        let mut config = default_ethnic_config();
        config.weights = vec![2.0, 3.0, 5.0, 0.0, 0.0, 0.0, 0.0];
        normalize_ethnic_weights(&mut config);
        let sum: f32 = config.weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
        assert!((config.weights[2] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_preset_to_json() {
        let p = new_ethnic_preset(EthnicGroup::European);
        let json = preset_to_json(&p);
        assert!(json.contains("European"));
        assert!(json.contains("eye_shape"));
    }

    #[test]
    fn test_ethnic_feature_vector() {
        let p = new_ethnic_preset(EthnicGroup::MiddleEastern);
        let v = ethnic_feature_vector(&p);
        assert_eq!(v.len(), 10);
        assert!((v[0] - p.eye_shape).abs() < 1e-6);
    }

    #[test]
    fn test_dominant_ethnic_group() {
        let mut config = default_ethnic_config();
        config.weights = vec![0.0, 0.0, 0.9, 0.1, 0.0, 0.0, 0.0];
        let dom = dominant_ethnic_group(&config);
        assert_eq!(dom, Some(2));
    }

    #[test]
    fn test_dominant_ethnic_group_empty() {
        let config = EthnicMorphConfig {
            weights: vec![],
            presets: vec![],
            normalize_on_set: false,
            smooth_blend: false,
        };
        assert_eq!(dominant_ethnic_group(&config), None);
    }

    #[test]
    fn test_apply_ethnic_to_face() {
        let mut config = default_ethnic_config();
        config.weights = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let weights = apply_ethnic_to_face(&config);
        assert_eq!(weights.len(), 10);
        let ea = new_ethnic_preset(EthnicGroup::EastAsian);
        let eye = weights.iter().find(|(k, _)| k == "ethnic_eye_shape").expect("should succeed");
        assert!((eye.1 - ea.eye_shape).abs() < 1e-5);
    }

    #[test]
    fn test_apply_ethnic_to_face_empty() {
        let config = EthnicMorphConfig {
            weights: vec![],
            presets: vec![],
            normalize_on_set: false,
            smooth_blend: false,
        };
        let result = apply_ethnic_to_face(&config);
        assert!(result.is_empty());
    }

    #[test]
    fn test_set_ethnic_blend_normalize_false() {
        let mut config = EthnicMorphConfig {
            weights: vec![0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
            presets: vec![],
            normalize_on_set: false,
            smooth_blend: false,
        };
        set_ethnic_blend(&mut config, 2, 0.3);
        assert!((config.weights[2] - 0.3).abs() < 1e-6);
        assert!((config.weights[0] - 0.5).abs() < 1e-6); // unchanged
    }

    #[test]
    fn test_preset_group_field() {
        let p = new_ethnic_preset(EthnicGroup::SouthAsian);
        assert_eq!(p.group, EthnicGroup::SouthAsian);
    }
}
