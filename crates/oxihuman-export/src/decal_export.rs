#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export decal projector settings.

#[allow(dead_code)]
pub struct DecalExport {
    pub name: String,
    pub texture: String,
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub size: [f32; 2],
    pub depth: f32,
    pub opacity: f32,
}

#[allow(dead_code)]
pub fn default_decal_export(name: &str) -> DecalExport {
    DecalExport {
        name: name.to_string(),
        texture: "decal_default.png".to_string(),
        position: [0.0, 0.0, 0.0],
        normal: [0.0, 1.0, 0.0],
        size: [1.0, 1.0],
        depth: 0.5,
        opacity: 1.0,
    }
}

#[allow(dead_code)]
pub fn export_decal_to_json(d: &DecalExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"texture\":\"{}\",\"position\":[{},{},{}],\"normal\":[{},{},{}],\"size\":[{},{}],\"depth\":{},\"opacity\":{}}}",
        d.name, d.texture,
        d.position[0], d.position[1], d.position[2],
        d.normal[0], d.normal[1], d.normal[2],
        d.size[0], d.size[1],
        d.depth, d.opacity
    )
}

/// Returns a 4x4 world matrix for the decal projector (identity stub).
#[allow(dead_code)]
pub fn decal_world_matrix(d: &DecalExport) -> [[f32; 4]; 4] {
    let _ = d;
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_decal_name() {
        let d = default_decal_export("wound");
        assert_eq!(d.name, "wound");
    }

    #[test]
    fn default_opacity_one() {
        let d = default_decal_export("x");
        assert!((d.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn default_depth_half() {
        let d = default_decal_export("x");
        assert!((d.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn default_normal_is_up() {
        let d = default_decal_export("x");
        assert!((d.normal[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn json_contains_name() {
        let d = default_decal_export("tattoo");
        let json = export_decal_to_json(&d);
        assert!(json.contains("tattoo"));
    }

    #[test]
    fn json_contains_texture() {
        let d = default_decal_export("x");
        let json = export_decal_to_json(&d);
        assert!(json.contains("texture"));
    }

    #[test]
    fn world_matrix_is_4x4() {
        let d = default_decal_export("x");
        let m = decal_world_matrix(&d);
        assert_eq!(m.len(), 4);
        assert_eq!(m[0].len(), 4);
    }

    #[test]
    fn world_matrix_diagonal_one() {
        let d = default_decal_export("x");
        let m = decal_world_matrix(&d);
        for (i, row) in m.iter().enumerate() {
            assert!((row[i] - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn json_valid_brackets() {
        let d = default_decal_export("scar");
        let json = export_decal_to_json(&d);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }

    #[test]
    fn default_size_one_by_one() {
        let d = default_decal_export("x");
        assert!((d.size[0] - 1.0).abs() < 1e-6);
        assert!((d.size[1] - 1.0).abs() < 1e-6);
    }
}
