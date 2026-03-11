// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! OpenXR scene description export.

#[allow(dead_code)]
/// An OpenXR reference space (Local, Stage, or View).
#[derive(Debug, Clone, PartialEq)]
pub struct XrReferenceSpace {
    /// "Local", "Stage", or "View"
    pub space_type: String,
    pub pose_x: f32,
    pub pose_y: f32,
    pub pose_z: f32,
}

#[allow(dead_code)]
/// A composition layer (projection or quad overlay).
#[derive(Debug, Clone, PartialEq)]
pub struct XrCompositionLayer {
    /// "Projection" or "Quad"
    pub layer_type: String,
    pub eye_index: u8,
    pub width: u32,
    pub height: u32,
    pub near: f32,
    pub far: f32,
}

#[allow(dead_code)]
/// A swapchain descriptor.
#[derive(Debug, Clone, PartialEq)]
pub struct XrSwapchain {
    pub width: u32,
    pub height: u32,
    /// e.g. "R8G8B8A8_SRGB"
    pub format: String,
    pub sample_count: u8,
}

#[allow(dead_code)]
/// A full OpenXR scene descriptor.
#[derive(Debug, Clone)]
pub struct XrScene {
    pub app_name: String,
    pub api_version: [u8; 3],
    pub reference_spaces: Vec<XrReferenceSpace>,
    pub layers: Vec<XrCompositionLayer>,
    pub swapchains: Vec<XrSwapchain>,
}

// ── Constructors ──────────────────────────────────────────────────────────────

/// Build a Stage reference space at the origin.
pub fn stage_reference_space() -> XrReferenceSpace {
    XrReferenceSpace {
        space_type: "Stage".to_string(),
        pose_x: 0.0,
        pose_y: 0.0,
        pose_z: 0.0,
    }
}

/// Build a Local reference space at the origin.
pub fn local_reference_space() -> XrReferenceSpace {
    XrReferenceSpace {
        space_type: "Local".to_string(),
        pose_x: 0.0,
        pose_y: 0.0,
        pose_z: 0.0,
    }
}

/// Build a projection layer for the given eye index (0 = left, 1 = right).
pub fn xr_projection_layer(width: u32, height: u32) -> XrCompositionLayer {
    XrCompositionLayer {
        layer_type: "Projection".to_string(),
        eye_index: 0,
        width,
        height,
        near: 0.05,
        far: 100.0,
    }
}

/// Build a 2D quad overlay composition layer.
#[allow(clippy::too_many_arguments)]
pub fn xr_quad_layer(width: u32, height: u32, _pos: [f32; 3]) -> XrCompositionLayer {
    XrCompositionLayer {
        layer_type: "Quad".to_string(),
        eye_index: 255, // both eyes
        width,
        height,
        near: 0.0,
        far: 0.0,
    }
}

/// Build a stereo swapchain with standard SRGB format.
pub fn stereo_swapchain(width: u32, height: u32) -> XrSwapchain {
    XrSwapchain {
        width,
        height,
        format: "R8G8B8A8_SRGB".to_string(),
        sample_count: 1,
    }
}

/// Create a default stereo XR scene with 2 projection layers.
pub fn default_xr_scene(app_name: &str) -> XrScene {
    let mut left = xr_projection_layer(1920, 1080);
    left.eye_index = 0;
    let mut right = xr_projection_layer(1920, 1080);
    right.eye_index = 1;

    XrScene {
        app_name: app_name.to_string(),
        api_version: [1, 0, 34],
        reference_spaces: vec![stage_reference_space(), local_reference_space()],
        layers: vec![left, right],
        swapchains: vec![stereo_swapchain(1920, 1080)],
    }
}

// ── Scene queries ─────────────────────────────────────────────────────────────

/// Return the number of composition layers.
pub fn layer_count(scene: &XrScene) -> usize {
    scene.layers.len()
}

/// Estimate the GPU frame budget in milliseconds (11.1 ms for 90 Hz).
///
/// Returns a rougher figure based on layer complexity.
pub fn estimate_frame_budget_ms(scene: &XrScene) -> f32 {
    // 90 Hz → 11.1 ms per frame baseline
    let base_ms: f32 = 1000.0 / 90.0;
    // Each additional layer beyond 2 costs ~0.5 ms (rough estimate)
    let extra = (scene.layers.len().saturating_sub(2)) as f32 * 0.5;
    base_ms + extra
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Validate the XR scene. Returns a list of error strings (empty = valid).
pub fn validate_xr_scene(scene: &XrScene) -> Vec<String> {
    let mut errors = Vec::new();
    if scene.app_name.is_empty() {
        errors.push("app_name is empty".to_string());
    }
    for (i, layer) in scene.layers.iter().enumerate() {
        if layer.width == 0 || layer.height == 0 {
            errors.push(format!("layer[{}]: zero-size dimensions", i));
        }
        let valid_types = ["Projection", "Quad"];
        if !valid_types.contains(&layer.layer_type.as_str()) {
            errors.push(format!("layer[{}]: unknown type '{}'", i, layer.layer_type));
        }
    }
    for (i, sc) in scene.swapchains.iter().enumerate() {
        if sc.width == 0 || sc.height == 0 {
            errors.push(format!("swapchain[{}]: zero-size dimensions", i));
        }
        if sc.sample_count == 0 {
            errors.push(format!("swapchain[{}]: sample_count is 0", i));
        }
    }
    errors
}

// ── JSON serialisation ────────────────────────────────────────────────────────

/// Build a JSON description of the full XR scene.
pub fn build_xr_scene_json(scene: &XrScene) -> String {
    let spaces_json: String = scene
        .reference_spaces
        .iter()
        .map(|s| {
            format!(
                r#"{{"type":"{}","pose":[{},{},{}]}}"#,
                s.space_type, s.pose_x, s.pose_y, s.pose_z
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let layers_json: String = scene
        .layers
        .iter()
        .map(|l| {
            format!(
                r#"{{"type":"{}","eye":{},"width":{},"height":{},"near":{},"far":{}}}"#,
                l.layer_type, l.eye_index, l.width, l.height, l.near, l.far
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let swapchains_json: String = scene
        .swapchains
        .iter()
        .map(|sc| {
            format!(
                r#"{{"width":{},"height":{},"format":"{}","sampleCount":{}}}"#,
                sc.width, sc.height, sc.format, sc.sample_count
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    format!(
        r#"{{"appName":"{}","apiVersion":[{},{},{}],"referenceSpaces":[{}],"layers":[{}],"swapchains":[{}]}}"#,
        scene.app_name,
        scene.api_version[0],
        scene.api_version[1],
        scene.api_version[2],
        spaces_json,
        layers_json,
        swapchains_json,
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_xr_scene_has_two_layers() {
        let scene = default_xr_scene("TestApp");
        assert!(
            scene.layers.len() >= 2,
            "default scene should have at least 2 layers"
        );
    }

    #[test]
    fn default_xr_scene_app_name() {
        let scene = default_xr_scene("MyVrApp");
        assert_eq!(scene.app_name, "MyVrApp");
    }

    #[test]
    fn build_xr_scene_json_contains_app_name() {
        let scene = default_xr_scene("OxiViewer");
        let json = build_xr_scene_json(&scene);
        assert!(json.contains("OxiViewer"), "JSON should contain app name");
    }

    #[test]
    fn build_xr_scene_json_contains_layers_key() {
        let scene = default_xr_scene("A");
        let json = build_xr_scene_json(&scene);
        assert!(json.contains("layers"));
    }

    #[test]
    fn validate_empty_scene_no_errors() {
        let scene = XrScene {
            app_name: "Test".to_string(),
            api_version: [1, 0, 0],
            reference_spaces: vec![],
            layers: vec![],
            swapchains: vec![],
        };
        let errs = validate_xr_scene(&scene);
        assert!(errs.is_empty(), "empty-layers scene: {:?}", errs);
    }

    #[test]
    fn validate_empty_app_name_errors() {
        let scene = XrScene {
            app_name: String::new(),
            api_version: [1, 0, 0],
            reference_spaces: vec![],
            layers: vec![],
            swapchains: vec![],
        };
        let errs = validate_xr_scene(&scene);
        assert!(!errs.is_empty());
    }

    #[test]
    fn xr_projection_layer_eye_index() {
        let layer = xr_projection_layer(1024, 1024);
        assert_eq!(layer.eye_index, 0);
        assert_eq!(layer.layer_type, "Projection");
    }

    #[test]
    fn xr_quad_layer_type() {
        let layer = xr_quad_layer(512, 512, [0.0, 1.0, -1.5]);
        assert_eq!(layer.layer_type, "Quad");
    }

    #[test]
    fn layer_count_correct() {
        let scene = default_xr_scene("X");
        assert_eq!(layer_count(&scene), 2);
    }

    #[test]
    fn estimate_frame_budget_near_11ms() {
        let scene = default_xr_scene("X");
        let budget = estimate_frame_budget_ms(&scene);
        assert!(
            (budget - 11.11).abs() < 0.5,
            "2-layer scene budget should be ~11.1ms, got {}",
            budget
        );
    }

    #[test]
    fn estimate_frame_budget_extra_layers() {
        let mut scene = default_xr_scene("X");
        scene.layers.push(xr_quad_layer(512, 512, [0.0; 3]));
        let budget = estimate_frame_budget_ms(&scene);
        // Should be > base
        assert!(budget > 1000.0 / 90.0);
    }

    #[test]
    fn stage_reference_space_type() {
        let s = stage_reference_space();
        assert_eq!(s.space_type, "Stage");
    }

    #[test]
    fn local_reference_space_type() {
        let s = local_reference_space();
        assert_eq!(s.space_type, "Local");
    }

    #[test]
    fn stereo_swapchain_format() {
        let sc = stereo_swapchain(1920, 1080);
        assert_eq!(sc.format, "R8G8B8A8_SRGB");
        assert_eq!(sc.width, 1920);
        assert_eq!(sc.height, 1080);
    }

    #[test]
    fn validate_zero_size_layer_errors() {
        let mut scene = default_xr_scene("T");
        scene.layers[0].width = 0;
        let errs = validate_xr_scene(&scene);
        assert!(!errs.is_empty());
    }
}
