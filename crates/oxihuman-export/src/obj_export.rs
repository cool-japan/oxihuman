// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Wavefront OBJ format export.
//!
//! Provides utilities for generating `.obj` text and companion `.mtl`
//! material files from mesh data, without any external file I/O dependency.

#![allow(dead_code)]

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for OBJ export.
#[derive(Debug, Clone)]
pub struct ObjExportConfig {
    /// Whether to include normals (`vn`) lines. Default: `true`.
    pub include_normals: bool,
    /// Whether to include UV coordinates (`vt`) lines. Default: `true`.
    pub include_uvs: bool,
    /// Whether to write an `mtllib` reference. Default: `true`.
    pub include_mtllib: bool,
    /// Name of the material library file. Default: `"model.mtl"`.
    pub mtllib_name: String,
    /// Object name written to the `o` line. Default: `"mesh"`.
    pub object_name: String,
    /// Number of decimal places for floating-point values. Default: `6`.
    pub precision: usize,
}

impl Default for ObjExportConfig {
    fn default() -> Self {
        Self {
            include_normals: true,
            include_uvs: true,
            include_mtllib: true,
            mtllib_name: "model.mtl".to_string(),
            object_name: "mesh".to_string(),
            precision: 6,
        }
    }
}

/// Result of an OBJ export operation.
#[derive(Debug, Clone)]
pub struct ObjExportResult {
    /// The generated OBJ text.
    pub obj_text: String,
    /// The generated MTL text (may be empty if `include_mtllib` is false).
    pub mtl_text: String,
    /// Number of vertex position lines written.
    pub vertex_count: usize,
    /// Number of face lines written.
    pub face_count: usize,
    /// Estimated total file size in bytes.
    pub estimated_bytes: usize,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// A validated OBJ string result.
pub type ObjValidationResult = Result<(), String>;

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a default [`ObjExportConfig`].
#[allow(dead_code)]
pub fn default_obj_config() -> ObjExportConfig {
    ObjExportConfig::default()
}

/// Generate a complete OBJ text string from positions, normals, UVs, and face indices.
///
/// `positions` — flat `[x, y, z, ...]`
/// `normals`   — flat `[nx, ny, nz, ...]` (may be empty)
/// `uvs`       — flat `[u, v, ...]` (may be empty)
/// `faces`     — flat triangle indices into the position array
#[allow(dead_code)]
pub fn export_to_obj_string(
    positions: &[f32],
    normals: &[f32],
    uvs: &[f32],
    faces: &[u32],
    cfg: &ObjExportConfig,
) -> ObjExportResult {
    let mut obj = obj_header(cfg);

    let vc = obj_vertex_count(positions);
    for i in 0..vc {
        let x = positions[i * 3];
        let y = positions[i * 3 + 1];
        let z = positions[i * 3 + 2];
        obj.push_str(&obj_vertex_line(x, y, z, cfg.precision));
    }

    if cfg.include_uvs && uvs.len() >= 2 {
        let uvc = uvs.len() / 2;
        for i in 0..uvc {
            let u = uvs[i * 2];
            let v = uvs[i * 2 + 1];
            obj.push_str(&obj_uv_line(u, v, cfg.precision));
        }
    }

    if cfg.include_normals && normals.len() >= 3 {
        let nc = normals.len() / 3;
        for i in 0..nc {
            let nx = normals[i * 3];
            let ny = normals[i * 3 + 1];
            let nz = normals[i * 3 + 2];
            obj.push_str(&obj_normal_line(nx, ny, nz, cfg.precision));
        }
    }

    let fc = obj_face_count(faces);
    for i in 0..fc {
        let a = faces[i * 3] + 1;
        let b = faces[i * 3 + 1] + 1;
        let c = faces[i * 3 + 2] + 1;
        obj.push_str(&obj_face_line(a, b, c));
    }

    let mtl = if cfg.include_mtllib {
        export_mtl_string(&cfg.object_name)
    } else {
        String::new()
    };

    let est = obj_file_size_estimate(vc, fc);

    ObjExportResult {
        obj_text: obj,
        mtl_text: mtl,
        vertex_count: vc,
        face_count: fc,
        estimated_bytes: est,
    }
}

/// Generate a minimal MTL material file string for `material_name`.
#[allow(dead_code)]
pub fn export_mtl_string(material_name: &str) -> String {
    let mut mtl = String::new();
    mtl.push_str("# OxiHuman MTL export\n");
    mtl.push_str(&format!("newmtl {}\n", material_name));
    mtl.push_str("Ka 1.000000 1.000000 1.000000\n");
    mtl.push_str("Kd 0.800000 0.800000 0.800000\n");
    mtl.push_str("Ks 0.000000 0.000000 0.000000\n");
    mtl.push_str("Ns 10.0\n");
    mtl.push_str("d 1.0\n");
    mtl.push_str("illum 2\n");
    mtl
}

/// Format a single `v x y z` vertex line.
#[allow(dead_code)]
pub fn obj_vertex_line(x: f32, y: f32, z: f32, precision: usize) -> String {
    format!("v {:.prec$} {:.prec$} {:.prec$}\n", x, y, z, prec = precision)
}

/// Format a single `vn nx ny nz` normal line.
#[allow(dead_code)]
pub fn obj_normal_line(nx: f32, ny: f32, nz: f32, precision: usize) -> String {
    format!("vn {:.prec$} {:.prec$} {:.prec$}\n", nx, ny, nz, prec = precision)
}

/// Format a single `vt u v` UV line.
#[allow(dead_code)]
pub fn obj_uv_line(u: f32, v: f32, precision: usize) -> String {
    format!("vt {:.prec$} {:.prec$}\n", u, v, prec = precision)
}

/// Format a single `f a b c` face line (1-based indices).
#[allow(dead_code)]
pub fn obj_face_line(a: u32, b: u32, c: u32) -> String {
    format!("f {} {} {}\n", a, b, c)
}

/// Generate the OBJ file header block.
#[allow(dead_code)]
pub fn obj_header(cfg: &ObjExportConfig) -> String {
    let mut hdr = String::new();
    hdr.push_str("# OxiHuman OBJ export\n");
    if cfg.include_mtllib {
        hdr.push_str(&format!("mtllib {}\n", cfg.mtllib_name));
    }
    hdr.push_str(&format!("o {}\n", cfg.object_name));
    hdr
}

/// Return the number of vertex positions encoded in `positions` (length / 3).
#[allow(dead_code)]
pub fn obj_vertex_count(positions: &[f32]) -> usize {
    positions.len() / 3
}

/// Return the number of triangular faces encoded in `faces` (length / 3).
#[allow(dead_code)]
pub fn obj_face_count(faces: &[u32]) -> usize {
    faces.len() / 3
}

/// Validate that `obj_text` is non-empty and contains at least one `v` line.
///
/// Returns `Ok(())` on success or an `Err` describing the problem.
#[allow(dead_code)]
pub fn validate_obj_output(obj_text: &str) -> ObjValidationResult {
    if obj_text.is_empty() {
        return Err("OBJ text is empty".to_string());
    }
    let has_vertex = obj_text.lines().any(|l| l.starts_with("v "));
    if !has_vertex {
        return Err("OBJ text contains no vertex lines".to_string());
    }
    Ok(())
}

/// Estimate the file size in bytes for `vertex_count` vertices and `face_count` faces.
///
/// Uses a conservative heuristic: ~30 bytes per vertex, ~20 bytes per face.
#[allow(dead_code)]
pub fn obj_file_size_estimate(vertex_count: usize, face_count: usize) -> usize {
    let header_bytes = 64usize;
    header_bytes + vertex_count * 30 + face_count * 20
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_obj_config() {
        let cfg = default_obj_config();
        assert!(cfg.include_normals);
        assert!(cfg.include_uvs);
        assert!(cfg.include_mtllib);
        assert_eq!(cfg.mtllib_name, "model.mtl");
        assert_eq!(cfg.object_name, "mesh");
        assert_eq!(cfg.precision, 6);
    }

    #[test]
    fn test_obj_vertex_line() {
        let line = obj_vertex_line(1.0, 2.0, 3.0, 3);
        assert_eq!(line, "v 1.000 2.000 3.000\n");
    }

    #[test]
    fn test_obj_normal_line() {
        let line = obj_normal_line(0.0, 1.0, 0.0, 2);
        assert_eq!(line, "vn 0.00 1.00 0.00\n");
    }

    #[test]
    fn test_obj_uv_line() {
        let line = obj_uv_line(0.5, 0.75, 4);
        assert_eq!(line, "vt 0.5000 0.7500\n");
    }

    #[test]
    fn test_obj_face_line() {
        let line = obj_face_line(1, 2, 3);
        assert_eq!(line, "f 1 2 3\n");
    }

    #[test]
    fn test_obj_header_with_mtllib() {
        let cfg = default_obj_config();
        let hdr = obj_header(&cfg);
        assert!(hdr.contains("mtllib model.mtl"));
        assert!(hdr.contains("o mesh"));
    }

    #[test]
    fn test_obj_header_without_mtllib() {
        let mut cfg = default_obj_config();
        cfg.include_mtllib = false;
        let hdr = obj_header(&cfg);
        assert!(!hdr.contains("mtllib"));
    }

    #[test]
    fn test_obj_vertex_count() {
        let pos = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        assert_eq!(obj_vertex_count(&pos), 2);
    }

    #[test]
    fn test_obj_face_count() {
        let faces = vec![0u32, 1, 2, 2, 3, 0];
        assert_eq!(obj_face_count(&faces), 2);
    }

    #[test]
    fn test_export_to_obj_string_basic() {
        let pos = vec![0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let faces = vec![0u32, 1, 2];
        let cfg = default_obj_config();
        let result = export_to_obj_string(&pos, &[], &[], &faces, &cfg);
        assert_eq!(result.vertex_count, 3);
        assert_eq!(result.face_count, 1);
        assert!(result.obj_text.contains("v "));
        assert!(result.obj_text.contains("f 1 2 3"));
    }

    #[test]
    fn test_export_to_obj_string_with_normals() {
        let pos = vec![0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let normals = vec![0.0f32, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        let faces = vec![0u32, 1, 2];
        let cfg = default_obj_config();
        let result = export_to_obj_string(&pos, &normals, &[], &faces, &cfg);
        assert!(result.obj_text.contains("vn "));
    }

    #[test]
    fn test_export_to_obj_string_with_uvs() {
        let pos = vec![0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let uvs = vec![0.0f32, 0.0, 1.0, 0.0, 0.0, 1.0];
        let faces = vec![0u32, 1, 2];
        let cfg = default_obj_config();
        let result = export_to_obj_string(&pos, &[], &uvs, &faces, &cfg);
        assert!(result.obj_text.contains("vt "));
    }

    #[test]
    fn test_export_mtl_string() {
        let mtl = export_mtl_string("MyMaterial");
        assert!(mtl.contains("newmtl MyMaterial"));
        assert!(mtl.contains("Ka "));
        assert!(mtl.contains("Kd "));
        assert!(mtl.contains("Ks "));
    }

    #[test]
    fn test_validate_obj_output_ok() {
        let obj = "# test\nv 0.0 0.0 0.0\nf 1 2 3\n";
        assert!(validate_obj_output(obj).is_ok());
    }

    #[test]
    fn test_validate_obj_output_empty() {
        assert!(validate_obj_output("").is_err());
    }

    #[test]
    fn test_validate_obj_output_no_vertices() {
        let obj = "# no vertices\nf 1 2 3\n";
        assert!(validate_obj_output(obj).is_err());
    }

    #[test]
    fn test_obj_file_size_estimate() {
        let est = obj_file_size_estimate(100, 200);
        assert_eq!(est, 64 + 100 * 30 + 200 * 20);
    }

    #[test]
    fn test_obj_result_has_mtl_text() {
        let pos = vec![0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let faces = vec![0u32, 1, 2];
        let cfg = default_obj_config();
        let result = export_to_obj_string(&pos, &[], &[], &faces, &cfg);
        assert!(!result.mtl_text.is_empty());
    }

    #[test]
    fn test_obj_result_no_mtl_when_disabled() {
        let pos = vec![0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let faces = vec![0u32, 1, 2];
        let mut cfg = default_obj_config();
        cfg.include_mtllib = false;
        let result = export_to_obj_string(&pos, &[], &[], &faces, &cfg);
        assert!(result.mtl_text.is_empty());
    }
}
