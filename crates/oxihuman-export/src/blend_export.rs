// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export to Blender-compatible JSON interchange format.

/// Blender export configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendExportConfig {
    pub version: u32,
    pub include_materials: bool,
    pub include_animations: bool,
}

/// Blender export result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendExportResult {
    pub json: String,
    pub object_count: usize,
    pub material_count: usize,
}

/// Blender object reference.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendObject {
    pub name: String,
    pub vertex_count: usize,
    pub face_count: usize,
    pub transform: [f32; 16],
}

#[allow(dead_code)]
pub fn default_blend_config() -> BlendExportConfig {
    BlendExportConfig { version: 4, include_materials: true, include_animations: false }
}

#[allow(dead_code)]
pub fn new_blend_object(name: &str, verts: usize, faces: usize) -> BlendObject {
    let mut transform = [0.0f32; 16];
    transform[0] = 1.0; transform[5] = 1.0; transform[10] = 1.0; transform[15] = 1.0;
    BlendObject { name: name.to_string(), vertex_count: verts, face_count: faces, transform }
}

#[allow(dead_code)]
pub fn blend_object_to_json(obj: &BlendObject) -> String {
    format!(
        r#"{{"name":"{}","vertices":{},"faces":{},"transform":[{}]}}"#,
        obj.name, obj.vertex_count, obj.face_count,
        obj.transform.iter().map(|v| format!("{:.4}", v)).collect::<Vec<_>>().join(",")
    )
}

#[allow(dead_code)]
pub fn export_blend_json(objects: &[BlendObject], config: &BlendExportConfig) -> BlendExportResult {
    let obj_jsons: Vec<String> = objects.iter().map(|o| blend_object_to_json(o)).collect();
    let json = format!(
        r#"{{"version":{},"objects":[{}],"materials_included":{}}}"#,
        config.version,
        obj_jsons.join(","),
        config.include_materials
    );
    BlendExportResult { json, object_count: objects.len(), material_count: 0 }
}

#[allow(dead_code)]
pub fn blend_result_size(result: &BlendExportResult) -> usize {
    result.json.len()
}

#[allow(dead_code)]
pub fn blend_object_count(result: &BlendExportResult) -> usize {
    result.object_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_blend_config();
        assert_eq!(c.version, 4);
    }

    #[test]
    fn test_new_object() {
        let obj = new_blend_object("Cube", 8, 12);
        assert_eq!(obj.name, "Cube");
        assert_eq!(obj.vertex_count, 8);
    }

    #[test]
    fn test_object_to_json() {
        let obj = new_blend_object("Sphere", 100, 200);
        let json = blend_object_to_json(&obj);
        assert!(json.contains("Sphere"));
    }

    #[test]
    fn test_export_empty() {
        let config = default_blend_config();
        let result = export_blend_json(&[], &config);
        assert_eq!(blend_object_count(&result), 0);
    }

    #[test]
    fn test_export_single() {
        let config = default_blend_config();
        let objs = vec![new_blend_object("Mesh", 10, 20)];
        let result = export_blend_json(&objs, &config);
        assert_eq!(blend_object_count(&result), 1);
    }

    #[test]
    fn test_export_multiple() {
        let config = default_blend_config();
        let objs = vec![new_blend_object("A", 1, 1), new_blend_object("B", 2, 2)];
        let result = export_blend_json(&objs, &config);
        assert_eq!(blend_object_count(&result), 2);
    }

    #[test]
    fn test_json_contains_version() {
        let config = default_blend_config();
        let result = export_blend_json(&[], &config);
        assert!(result.json.contains("version"));
    }

    #[test]
    fn test_result_size() {
        let config = default_blend_config();
        let result = export_blend_json(&[], &config);
        assert!(blend_result_size(&result) > 0);
    }

    #[test]
    fn test_transform_identity() {
        let obj = new_blend_object("T", 0, 0);
        assert!((obj.transform[0] - 1.0).abs() < 1e-5);
        assert!((obj.transform[15] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_config_materials_flag() {
        let mut c = default_blend_config();
        c.include_materials = false;
        let result = export_blend_json(&[], &c);
        assert!(result.json.contains("false"));
    }

}
