// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Animation layer export: blended animation layer data serialisation.

/// Blend mode for an animation layer.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerBlendMode {
    Override,
    Additive,
    Multiply,
}

/// A single animation layer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimLayer {
    pub name: String,
    pub weight: f32,
    pub blend_mode: LayerBlendMode,
    pub enabled: bool,
    pub track_count: usize,
}

/// Export bundle for animation layers.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimLayerExport {
    pub layers: Vec<AnimLayer>,
}

/// Create a new animation layer export.
#[allow(dead_code)]
pub fn new_anim_layer_export() -> AnimLayerExport {
    AnimLayerExport { layers: Vec::new() }
}

/// Add a layer.
#[allow(dead_code)]
pub fn add_anim_layer(exp: &mut AnimLayerExport, layer: AnimLayer) {
    exp.layers.push(layer);
}

/// Layer count.
#[allow(dead_code)]
pub fn anim_layer_count(exp: &AnimLayerExport) -> usize {
    exp.layers.len()
}

/// Total weight of enabled layers.
#[allow(dead_code)]
pub fn total_enabled_weight(exp: &AnimLayerExport) -> f32 {
    exp.layers
        .iter()
        .filter(|l| l.enabled)
        .map(|l| l.weight)
        .sum()
}

/// Find layer by name.
#[allow(dead_code)]
pub fn find_layer_by_name<'a>(exp: &'a AnimLayerExport, name: &str) -> Option<&'a AnimLayer> {
    exp.layers.iter().find(|l| l.name == name)
}

/// Enabled layer count.
#[allow(dead_code)]
pub fn enabled_layer_count(exp: &AnimLayerExport) -> usize {
    exp.layers.iter().filter(|l| l.enabled).count()
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn anim_layer_to_json(exp: &AnimLayerExport) -> String {
    format!(
        "{{\"layer_count\":{},\"enabled\":{}}}",
        anim_layer_count(exp),
        enabled_layer_count(exp)
    )
}

/// Blend mode name string.
#[allow(dead_code)]
pub fn blend_mode_name(mode: LayerBlendMode) -> &'static str {
    match mode {
        LayerBlendMode::Override => "override",
        LayerBlendMode::Additive => "additive",
        LayerBlendMode::Multiply => "multiply",
    }
}

/// Validate: all weights in `[0,1]`.
#[allow(dead_code)]
pub fn validate_anim_layers(exp: &AnimLayerExport) -> bool {
    exp.layers.iter().all(|l| (0.0..=1.0).contains(&l.weight))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_layer(name: &str, w: f32) -> AnimLayer {
        AnimLayer {
            name: name.to_string(),
            weight: w,
            blend_mode: LayerBlendMode::Override,
            enabled: true,
            track_count: 3,
        }
    }

    #[test]
    fn new_export_empty() {
        let exp = new_anim_layer_export();
        assert_eq!(anim_layer_count(&exp), 0);
    }

    #[test]
    fn add_layer_increments() {
        let mut exp = new_anim_layer_export();
        add_anim_layer(&mut exp, sample_layer("base", 1.0));
        assert_eq!(anim_layer_count(&exp), 1);
    }

    #[test]
    fn total_weight_sums() {
        let mut exp = new_anim_layer_export();
        add_anim_layer(&mut exp, sample_layer("a", 0.5));
        add_anim_layer(&mut exp, sample_layer("b", 0.3));
        assert!((total_enabled_weight(&exp) - 0.8).abs() < 1e-5);
    }

    #[test]
    fn find_by_name_some() {
        let mut exp = new_anim_layer_export();
        add_anim_layer(&mut exp, sample_layer("walk", 1.0));
        assert!(find_layer_by_name(&exp, "walk").is_some());
    }

    #[test]
    fn find_by_name_none() {
        let exp = new_anim_layer_export();
        assert!(find_layer_by_name(&exp, "missing").is_none());
    }

    #[test]
    fn enabled_count() {
        let mut exp = new_anim_layer_export();
        add_anim_layer(
            &mut exp,
            AnimLayer {
                enabled: false,
                ..sample_layer("x", 0.5)
            },
        );
        add_anim_layer(&mut exp, sample_layer("y", 0.5));
        assert_eq!(enabled_layer_count(&exp), 1);
    }

    #[test]
    fn validate_valid() {
        let mut exp = new_anim_layer_export();
        add_anim_layer(&mut exp, sample_layer("a", 0.7));
        assert!(validate_anim_layers(&exp));
    }

    #[test]
    fn blend_mode_names() {
        assert_eq!(blend_mode_name(LayerBlendMode::Additive), "additive");
    }

    #[test]
    fn json_contains_layer_count() {
        let mut exp = new_anim_layer_export();
        add_anim_layer(&mut exp, sample_layer("base", 1.0));
        let j = anim_layer_to_json(&exp);
        assert!(j.contains("layer_count"));
    }

    #[test]
    fn weight_in_range() {
        let l = sample_layer("t", 0.5);
        assert!((0.0..=1.0).contains(&l.weight));
    }
}
