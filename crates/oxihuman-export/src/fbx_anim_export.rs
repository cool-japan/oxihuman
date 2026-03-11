// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! FBX animation export stub for skeletal animation curves.

/// Configuration for FBX animation export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FbxAnimConfig {
    pub frame_rate: f32,
    pub start_frame: i32,
    pub end_frame: i32,
    pub bake_transforms: bool,
}

/// A single animation curve for one node property.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FbxAnimCurve {
    pub node_name: String,
    pub property: String,
    pub keyframes: Vec<FbxKeyframe>,
}

/// A single keyframe on an animation curve.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FbxKeyframe {
    pub time: f32,
    pub value: f32,
    pub tangent_in: f32,
    pub tangent_out: f32,
}

/// An animation layer containing multiple curves.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FbxAnimLayer {
    pub name: String,
    pub curves: Vec<FbxAnimCurve>,
}

/// Result of exporting FBX animation layers.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FbxAnimExportResult {
    pub layer_count: usize,
    pub total_keyframes: usize,
    pub duration_sec: f32,
    pub success: bool,
}

/// Returns a default FBX animation configuration (24 fps, frames 0–100).
#[allow(dead_code)]
pub fn default_fbx_anim_config() -> FbxAnimConfig {
    FbxAnimConfig {
        frame_rate: 24.0,
        start_frame: 0,
        end_frame: 100,
        bake_transforms: false,
    }
}

/// Creates a new, empty FBX animation layer with the given name.
#[allow(dead_code)]
pub fn new_fbx_anim_layer(name: &str) -> FbxAnimLayer {
    FbxAnimLayer {
        name: name.to_string(),
        curves: Vec::new(),
    }
}

/// Appends a curve to an animation layer.
#[allow(dead_code)]
pub fn add_curve_to_layer(layer: &mut FbxAnimLayer, curve: FbxAnimCurve) {
    layer.curves.push(curve);
}

/// Creates a new, empty FBX animation curve for a node property.
#[allow(dead_code)]
pub fn new_fbx_anim_curve(node: &str, property: &str) -> FbxAnimCurve {
    FbxAnimCurve {
        node_name: node.to_string(),
        property: property.to_string(),
        keyframes: Vec::new(),
    }
}

/// Appends a keyframe to a curve.
#[allow(dead_code)]
pub fn add_keyframe(curve: &mut FbxAnimCurve, kf: FbxKeyframe) {
    curve.keyframes.push(kf);
}

/// Exports a slice of animation layers using the provided configuration.
#[allow(dead_code)]
pub fn export_anim_layers(layers: &[FbxAnimLayer], cfg: &FbxAnimConfig) -> FbxAnimExportResult {
    let total_kf: usize = layers.iter().map(total_keyframes_in_layer).sum();
    FbxAnimExportResult {
        layer_count: layers.len(),
        total_keyframes: total_kf,
        duration_sec: anim_duration_sec(cfg),
        success: !layers.is_empty(),
    }
}

/// Returns the total number of keyframes across all curves in a layer.
#[allow(dead_code)]
pub fn total_keyframes_in_layer(layer: &FbxAnimLayer) -> usize {
    layer.curves.iter().map(|c| c.keyframes.len()).sum()
}

/// Computes the duration in seconds from a config's frame range and frame rate.
#[allow(dead_code)]
pub fn anim_duration_sec(cfg: &FbxAnimConfig) -> f32 {
    let frames = (cfg.end_frame - cfg.start_frame).max(0) as f32;
    if cfg.frame_rate > 0.0 {
        frames / cfg.frame_rate
    } else {
        0.0
    }
}

/// Returns the number of keyframes in a single curve.
#[allow(dead_code)]
pub fn keyframe_count_for_curve(curve: &FbxAnimCurve) -> usize {
    curve.keyframes.len()
}

/// Serialises an export result to a JSON string.
#[allow(dead_code)]
pub fn fbx_anim_result_to_json(r: &FbxAnimExportResult) -> String {
    format!(
        r#"{{"layer_count":{l},"total_keyframes":{k},"duration_sec":{d:.4},"success":{s}}}"#,
        l = r.layer_count,
        k = r.total_keyframes,
        d = r.duration_sec,
        s = r.success,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_fbx_anim_config();
        assert!((cfg.frame_rate - 24.0).abs() < 1e-6);
        assert_eq!(cfg.start_frame, 0);
        assert_eq!(cfg.end_frame, 100);
        assert!(!cfg.bake_transforms);
    }

    #[test]
    fn duration_sec_correct() {
        let mut cfg = default_fbx_anim_config();
        cfg.start_frame = 0;
        cfg.end_frame = 24;
        cfg.frame_rate = 24.0;
        let dur = anim_duration_sec(&cfg);
        assert!((dur - 1.0).abs() < 1e-5);
    }

    #[test]
    fn duration_sec_zero_fps() {
        let mut cfg = default_fbx_anim_config();
        cfg.frame_rate = 0.0;
        assert_eq!(anim_duration_sec(&cfg), 0.0);
    }

    #[test]
    fn add_keyframe_increments_count() {
        let mut curve = new_fbx_anim_curve("Hips", "LocalTranslation");
        let kf = FbxKeyframe { time: 0.0, value: 1.0, tangent_in: 0.0, tangent_out: 0.0 };
        add_keyframe(&mut curve, kf);
        assert_eq!(keyframe_count_for_curve(&curve), 1);
    }

    #[test]
    fn add_curve_to_layer_increments() {
        let mut layer = new_fbx_anim_layer("Base");
        let curve = new_fbx_anim_curve("Spine", "RotationX");
        add_curve_to_layer(&mut layer, curve);
        assert_eq!(layer.curves.len(), 1);
    }

    #[test]
    fn total_keyframes_in_layer_sums_curves() {
        let mut layer = new_fbx_anim_layer("Base");
        let mut c1 = new_fbx_anim_curve("A", "X");
        add_keyframe(&mut c1, FbxKeyframe { time: 0.0, value: 0.0, tangent_in: 0.0, tangent_out: 0.0 });
        add_keyframe(&mut c1, FbxKeyframe { time: 1.0, value: 1.0, tangent_in: 0.0, tangent_out: 0.0 });
        let mut c2 = new_fbx_anim_curve("B", "Y");
        add_keyframe(&mut c2, FbxKeyframe { time: 0.0, value: 5.0, tangent_in: 0.0, tangent_out: 0.0 });
        add_curve_to_layer(&mut layer, c1);
        add_curve_to_layer(&mut layer, c2);
        assert_eq!(total_keyframes_in_layer(&layer), 3);
    }

    #[test]
    fn export_anim_layers_success_flag() {
        let cfg = default_fbx_anim_config();
        let layer = new_fbx_anim_layer("Base");
        let result = export_anim_layers(&[layer], &cfg);
        assert!(result.success);
        assert_eq!(result.layer_count, 1);
    }

    #[test]
    fn export_anim_layers_empty_not_success() {
        let cfg = default_fbx_anim_config();
        let result = export_anim_layers(&[], &cfg);
        assert!(!result.success);
    }

    #[test]
    fn result_to_json_contains_fields() {
        let r = FbxAnimExportResult {
            layer_count: 2,
            total_keyframes: 10,
            duration_sec: 1.5,
            success: true,
        };
        let json = fbx_anim_result_to_json(&r);
        assert!(json.contains("\"layer_count\":2"));
        assert!(json.contains("\"total_keyframes\":10"));
        assert!(json.contains("\"success\":true"));
    }
}
