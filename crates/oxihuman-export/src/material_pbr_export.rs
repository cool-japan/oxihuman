// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export PBR material data to JSON-compatible format.

#![allow(dead_code)]

/// PBR material export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialExport {
    pub name: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],
    pub alpha_mode: u8,
}

/// Alpha mode constants.
pub const ALPHA_OPAQUE: u8 = 0;
pub const ALPHA_BLEND: u8 = 1;
pub const ALPHA_MASK: u8 = 2;

/// Create a default opaque PBR material.
#[allow(dead_code)]
pub fn default_material_export(name: &str) -> MaterialExport {
    MaterialExport {
        name: name.to_string(),
        base_color: [1.0, 1.0, 1.0, 1.0],
        metallic: 0.0,
        roughness: 0.5,
        emissive: [0.0, 0.0, 0.0],
        alpha_mode: ALPHA_OPAQUE,
    }
}

/// Return true if the material uses alpha blending or has alpha < 1.
#[allow(dead_code)]
pub fn material_is_transparent(mat: &MaterialExport) -> bool {
    mat.alpha_mode != ALPHA_OPAQUE || mat.base_color[3] < 1.0
}

/// Serialize a material to a JSON string.
#[allow(dead_code)]
pub fn export_material_to_json(mat: &MaterialExport) -> String {
    format!(
        r#"{{"name":"{name}","base_color":[{r},{g},{b},{a}],"metallic":{metallic},"roughness":{roughness},"emissive":[{er},{eg},{eb}],"alpha_mode":{alpha_mode}}}"#,
        name = mat.name,
        r = mat.base_color[0], g = mat.base_color[1],
        b = mat.base_color[2], a = mat.base_color[3],
        metallic = mat.metallic,
        roughness = mat.roughness,
        er = mat.emissive[0], eg = mat.emissive[1], eb = mat.emissive[2],
        alpha_mode = mat.alpha_mode,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_material_name() {
        let mat = default_material_export("my_mat");
        assert_eq!(mat.name, "my_mat");
    }

    #[test]
    fn test_default_opaque() {
        let mat = default_material_export("m");
        assert!(!material_is_transparent(&mat));
    }

    #[test]
    fn test_alpha_blend_transparent() {
        let mut mat = default_material_export("m");
        mat.alpha_mode = ALPHA_BLEND;
        assert!(material_is_transparent(&mat));
    }

    #[test]
    fn test_alpha_color_transparent() {
        let mut mat = default_material_export("m");
        mat.base_color[3] = 0.5;
        assert!(material_is_transparent(&mat));
    }

    #[test]
    fn test_json_contains_name() {
        let mat = default_material_export("steel");
        let json = export_material_to_json(&mat);
        assert!(json.contains("steel"));
    }

    #[test]
    fn test_json_contains_metallic() {
        let mat = default_material_export("m");
        let json = export_material_to_json(&mat);
        assert!(json.contains("metallic"));
    }

    #[test]
    fn test_json_contains_roughness() {
        let mat = default_material_export("m");
        let json = export_material_to_json(&mat);
        assert!(json.contains("roughness"));
    }

    #[test]
    fn test_default_base_color_white() {
        let mat = default_material_export("m");
        assert!((mat.base_color[0] - 1.0).abs() < 1e-5);
        assert!((mat.base_color[3] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_default_roughness_range() {
        let mat = default_material_export("m");
        assert!((0.0..=1.0).contains(&mat.roughness));
    }

    #[test]
    fn test_default_metallic_zero() {
        let mat = default_material_export("m");
        assert!((mat.metallic).abs() < 1e-5);
    }
}
