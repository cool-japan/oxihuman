// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! JT (Jupiter Tessellation) format export stub.

/// JT LOD level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JtLodLevel {
    High,
    Medium,
    Low,
}

/// A JT tessellation stub for one LOD.
#[derive(Debug, Clone)]
pub struct JtLod {
    pub level: JtLodLevel,
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
}

/// JT file export container.
#[derive(Debug, Clone, Default)]
pub struct JtExport {
    pub file_version: String,
    pub lods: Vec<JtLod>,
    pub product_name: String,
}

/// Create a new JT export.
pub fn new_jt_export(file_version: &str, product_name: &str) -> JtExport {
    JtExport {
        file_version: file_version.to_string(),
        lods: Vec::new(),
        product_name: product_name.to_string(),
    }
}

/// Add a LOD to the JT export.
pub fn add_jt_lod(
    export: &mut JtExport,
    level: JtLodLevel,
    verts: Vec<[f32; 3]>,
    tris: Vec<[u32; 3]>,
) {
    export.lods.push(JtLod { level, verts, tris });
}

/// Return the LOD count.
pub fn jt_lod_count(export: &JtExport) -> usize {
    export.lods.len()
}

/// Return the total vertex count across all LODs.
pub fn jt_total_vertex_count(export: &JtExport) -> usize {
    export.lods.iter().map(|l| l.verts.len()).sum()
}

/// Return the total triangle count across all LODs.
pub fn jt_total_tri_count(export: &JtExport) -> usize {
    export.lods.iter().map(|l| l.tris.len()).sum()
}

/// Render the JT file header (stub).
pub fn jt_file_header(export: &JtExport) -> String {
    format!(
        "Version {} Product:{}",
        export.file_version, export.product_name
    )
}

/// Validate that all LOD triangle indices are in range.
pub fn validate_jt_export(export: &JtExport) -> bool {
    export.lods.iter().all(|lod| {
        let n = lod.verts.len() as u32;
        lod.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
    })
}

/// Find the highest-quality LOD.
pub fn jt_high_lod(export: &JtExport) -> Option<&JtLod> {
    export.lods.iter().find(|l| l.level == JtLodLevel::High)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_lod(_level: JtLodLevel) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0, 1, 2]];
        (verts, tris)
    }

    #[test]
    fn test_new_export_empty() {
        let exp = new_jt_export("10.5", "Widget");
        assert_eq!(jt_lod_count(&exp), 0);
    }

    #[test]
    fn test_add_lod() {
        let mut exp = new_jt_export("10.5", "Widget");
        let (v, t) = triangle_lod(JtLodLevel::High);
        add_jt_lod(&mut exp, JtLodLevel::High, v, t);
        assert_eq!(jt_lod_count(&exp), 1);
    }

    #[test]
    fn test_total_vertex_count() {
        let mut exp = new_jt_export("10.5", "Widget");
        let (v, t) = triangle_lod(JtLodLevel::High);
        add_jt_lod(&mut exp, JtLodLevel::High, v, t);
        let (v2, t2) = triangle_lod(JtLodLevel::Low);
        add_jt_lod(&mut exp, JtLodLevel::Low, v2, t2);
        assert_eq!(jt_total_vertex_count(&exp), 6);
    }

    #[test]
    fn test_total_tri_count() {
        let mut exp = new_jt_export("10.5", "Widget");
        let (v, t) = triangle_lod(JtLodLevel::High);
        add_jt_lod(&mut exp, JtLodLevel::High, v, t);
        assert_eq!(jt_total_tri_count(&exp), 1);
    }

    #[test]
    fn test_header_contains_version() {
        let exp = new_jt_export("10.5", "Widget");
        assert!(jt_file_header(&exp).contains("10.5"));
    }

    #[test]
    fn test_validate_valid_lod() {
        let mut exp = new_jt_export("10.5", "Widget");
        let (v, t) = triangle_lod(JtLodLevel::High);
        add_jt_lod(&mut exp, JtLodLevel::High, v, t);
        assert!(validate_jt_export(&exp));
    }

    #[test]
    fn test_high_lod_found() {
        let mut exp = new_jt_export("10.5", "Widget");
        let (v, t) = triangle_lod(JtLodLevel::High);
        add_jt_lod(&mut exp, JtLodLevel::High, v, t);
        assert!(jt_high_lod(&exp).is_some());
    }

    #[test]
    fn test_high_lod_not_found() {
        let mut exp = new_jt_export("10.5", "Widget");
        let (v, t) = triangle_lod(JtLodLevel::Low);
        add_jt_lod(&mut exp, JtLodLevel::Low, v, t);
        assert!(jt_high_lod(&exp).is_none());
    }

    #[test]
    fn test_product_name_stored() {
        let exp = new_jt_export("10.5", "MyPart");
        assert_eq!(exp.product_name, "MyPart");
    }
}
