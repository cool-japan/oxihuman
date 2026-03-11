// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Exported representation of an empty/null object.
#[allow(dead_code)]
pub struct EmptyObjectExport {
    pub name: String,
    pub display_type: u8,
    pub size: f32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

/// Create a default empty object export (plain axes display).
#[allow(dead_code)]
pub fn default_empty_object_export(name: &str) -> EmptyObjectExport {
    EmptyObjectExport {
        name: name.to_string(),
        display_type: 0,
        size: 1.0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 0.0, 1.0],
    }
}

/// Export an empty object to a JSON string.
#[allow(dead_code)]
pub fn export_empty_to_json(e: &EmptyObjectExport) -> String {
    format!(
        r#"{{"name":"{}","display_type":{},"size":{},"position":[{},{},{}],"rotation":[{},{},{},{}]}}"#,
        e.name,
        e.display_type,
        e.size,
        e.position[0], e.position[1], e.position[2],
        e.rotation[0], e.rotation[1], e.rotation[2], e.rotation[3]
    )
}

/// Check if the empty object is at the origin.
#[allow(dead_code)]
pub fn empty_is_at_origin(e: &EmptyObjectExport) -> bool {
    e.position[0].abs() < 1e-6 && e.position[1].abs() < 1e-6 && e.position[2].abs() < 1e-6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_empty_object_export_name() {
        let e = default_empty_object_export("Empty");
        assert_eq!(e.name, "Empty");
    }

    #[test]
    fn test_default_empty_object_export_size() {
        let e = default_empty_object_export("E");
        assert!((e.size - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_default_empty_object_export_position_origin() {
        let e = default_empty_object_export("E");
        assert!(empty_is_at_origin(&e));
    }

    #[test]
    fn test_default_empty_object_export_identity_quat() {
        let e = default_empty_object_export("E");
        // identity quaternion w=1
        assert!((e.rotation[3] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_export_empty_to_json_name() {
        let e = default_empty_object_export("NullObj");
        let json = export_empty_to_json(&e);
        assert!(json.contains("NullObj"));
    }

    #[test]
    fn test_export_empty_to_json_display_type() {
        let e = default_empty_object_export("E");
        let json = export_empty_to_json(&e);
        assert!(json.contains("display_type"));
    }

    #[test]
    fn test_export_empty_to_json_structure() {
        let e = default_empty_object_export("E");
        let json = export_empty_to_json(&e);
        assert!(json.starts_with('{') && json.ends_with('}'));
    }

    #[test]
    fn test_empty_is_at_origin_not_origin() {
        let mut e = default_empty_object_export("E");
        e.position = [1.0, 0.0, 0.0];
        assert!(!empty_is_at_origin(&e));
    }

    #[test]
    fn test_export_empty_to_json_position() {
        let e = default_empty_object_export("E");
        let json = export_empty_to_json(&e);
        assert!(json.contains("position"));
    }

    #[test]
    fn test_export_empty_to_json_rotation() {
        let e = default_empty_object_export("E");
        let json = export_empty_to_json(&e);
        assert!(json.contains("rotation"));
    }
}
