// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// View layer settings for export.
#[allow(dead_code)]
pub struct ViewLayerExport {
    pub name: String,
    pub use_ao: bool,
    pub use_shadow: bool,
    pub use_motion_blur: bool,
    pub samples: u32,
    pub pass_names: Vec<String>,
}

/// Create a default view layer export.
#[allow(dead_code)]
pub fn default_view_layer_export(name: &str) -> ViewLayerExport {
    ViewLayerExport {
        name: name.to_string(),
        use_ao: false,
        use_shadow: true,
        use_motion_blur: false,
        samples: 128,
        pass_names: Vec::new(),
    }
}

/// Add a render pass to the view layer.
#[allow(dead_code)]
pub fn add_pass(vl: &mut ViewLayerExport, pass: &str) {
    vl.pass_names.push(pass.to_string());
}

/// Export the view layer to JSON.
#[allow(dead_code)]
pub fn export_view_layer_to_json(vl: &ViewLayerExport) -> String {
    let passes: String = vl
        .pass_names
        .iter()
        .map(|p| format!(r#""{}""#, p))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"name":"{}","use_ao":{},"use_shadow":{},"use_motion_blur":{},"samples":{},"passes":[{}]}}"#,
        vl.name, vl.use_ao, vl.use_shadow, vl.use_motion_blur, vl.samples, passes
    )
}

/// Count render passes.
#[allow(dead_code)]
pub fn pass_count(vl: &ViewLayerExport) -> usize {
    vl.pass_names.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_view_layer_export_name() {
        let vl = default_view_layer_export("ViewLayer");
        assert_eq!(vl.name, "ViewLayer");
    }

    #[test]
    fn test_default_view_layer_export_samples() {
        let vl = default_view_layer_export("VL");
        assert_eq!(vl.samples, 128);
    }

    #[test]
    fn test_default_view_layer_export_no_passes() {
        let vl = default_view_layer_export("VL");
        assert_eq!(pass_count(&vl), 0);
    }

    #[test]
    fn test_add_pass_count() {
        let mut vl = default_view_layer_export("VL");
        add_pass(&mut vl, "Combined");
        add_pass(&mut vl, "Depth");
        assert_eq!(pass_count(&vl), 2);
    }

    #[test]
    fn test_add_pass_name() {
        let mut vl = default_view_layer_export("VL");
        add_pass(&mut vl, "Normal");
        assert_eq!(vl.pass_names[0], "Normal");
    }

    #[test]
    fn test_export_view_layer_to_json_name() {
        let vl = default_view_layer_export("MainLayer");
        let json = export_view_layer_to_json(&vl);
        assert!(json.contains("MainLayer"));
    }

    #[test]
    fn test_export_view_layer_to_json_samples() {
        let vl = default_view_layer_export("VL");
        let json = export_view_layer_to_json(&vl);
        assert!(json.contains("128"));
    }

    #[test]
    fn test_export_view_layer_to_json_with_passes() {
        let mut vl = default_view_layer_export("VL");
        add_pass(&mut vl, "AO");
        let json = export_view_layer_to_json(&vl);
        assert!(json.contains("AO"));
    }

    #[test]
    fn test_export_view_layer_json_structure() {
        let vl = default_view_layer_export("VL");
        let json = export_view_layer_to_json(&vl);
        assert!(json.starts_with('{') && json.ends_with('}'));
    }

    #[test]
    fn test_view_layer_shadow_default_true() {
        let vl = default_view_layer_export("VL");
        assert!(vl.use_shadow);
    }
}
