// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! STL (stereolithography) format export — ASCII and binary variants.
//!
//! Generates `.stl` text or raw bytes from triangle soup data.

#![allow(dead_code)]

// ── Enums ─────────────────────────────────────────────────────────────────────

/// STL output mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StlMode {
    /// Human-readable ASCII STL.
    Ascii,
    /// Compact 80-byte-header binary STL.
    Binary,
}

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for STL export.
#[derive(Debug, Clone)]
pub struct StlExportConfig {
    /// Output mode (ASCII or Binary). Default: `StlMode::Ascii`.
    pub mode: StlMode,
    /// Solid name written in the header. Default: `"oxihuman"`.
    pub solid_name: String,
    /// Decimal precision for ASCII output. Default: `6`.
    pub precision: usize,
}

impl Default for StlExportConfig {
    fn default() -> Self {
        Self {
            mode: StlMode::Ascii,
            solid_name: "oxihuman".to_string(),
            precision: 6,
        }
    }
}

/// A single STL triangle with a face normal and three vertex positions.
#[derive(Debug, Clone, PartialEq)]
pub struct StlTriangle {
    /// Face normal `[nx, ny, nz]`.
    pub normal: [f32; 3],
    /// Vertex A `[x, y, z]`.
    pub v0: [f32; 3],
    /// Vertex B `[x, y, z]`.
    pub v1: [f32; 3],
    /// Vertex C `[x, y, z]`.
    pub v2: [f32; 3],
    /// Attribute byte count (usually 0).
    pub attribute: u16,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// Result type for STL validation.
pub type StlValidationResult = Result<(), String>;

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a default [`StlExportConfig`].
#[allow(dead_code)]
pub fn default_stl_config() -> StlExportConfig {
    StlExportConfig::default()
}

/// Export `triangles` as an ASCII STL string.
#[allow(dead_code)]
pub fn export_to_stl_ascii(triangles: &[StlTriangle], cfg: &StlExportConfig) -> String {
    let name = stl_solid_name(cfg);
    let mut out = format!("solid {}\n", name);
    for tri in triangles {
        let n = tri.normal;
        out.push_str(&format!(
            "  facet normal {:.prec$} {:.prec$} {:.prec$}\n",
            n[0],
            n[1],
            n[2],
            prec = cfg.precision
        ));
        out.push_str("    outer loop\n");
        for v in &[tri.v0, tri.v1, tri.v2] {
            out.push_str(&stl_ascii_line(v[0], v[1], v[2], cfg.precision));
        }
        out.push_str("    endloop\n");
        out.push_str("  endfacet\n");
    }
    out.push_str(&format!("endsolid {}\n", name));
    out
}

/// Export `triangles` as binary STL bytes (little-endian).
///
/// Binary layout: 80-byte header + 4-byte count + (50 bytes × N triangles).
#[allow(dead_code)]
pub fn export_to_stl_binary(triangles: &[StlTriangle], cfg: &StlExportConfig) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::with_capacity(stl_file_size(triangles.len()));

    // 80-byte header
    let header = stl_header_bytes(&cfg.solid_name);
    bytes.extend_from_slice(&header);

    // 4-byte triangle count
    let count = triangles.len() as u32;
    bytes.extend_from_slice(&count.to_le_bytes());

    // 50 bytes per triangle
    for tri in triangles {
        for &val in tri.normal.iter().chain(tri.v0.iter()).chain(tri.v1.iter()).chain(tri.v2.iter()) {
            bytes.extend_from_slice(&val.to_le_bytes());
        }
        bytes.extend_from_slice(&tri.attribute.to_le_bytes());
    }

    bytes
}

/// Construct a [`StlTriangle`] from three vertex positions, computing the face normal.
#[allow(dead_code)]
pub fn stl_triangle_from_positions(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> StlTriangle {
    let normal = stl_normal_from_vertices(v0, v1, v2);
    StlTriangle { normal, v0, v1, v2, attribute: 0 }
}

/// Produce the 80-byte binary header from `solid_name` (truncated / padded as needed).
#[allow(dead_code)]
pub fn stl_header_bytes(solid_name: &str) -> [u8; 80] {
    let mut header = [0u8; 80];
    let prefix = format!("OxiHuman STL: {}", solid_name);
    let bytes = prefix.as_bytes();
    let len = bytes.len().min(80);
    header[..len].copy_from_slice(&bytes[..len]);
    header
}

/// Return the number of triangles in `triangles`.
#[allow(dead_code)]
pub fn stl_face_count(triangles: &[StlTriangle]) -> usize {
    triangles.len()
}

/// Calculate the exact binary STL file size for `n` triangles in bytes.
///
/// Formula: `80 + 4 + n * 50`.
#[allow(dead_code)]
pub fn stl_file_size(n: usize) -> usize {
    84 + n * 50
}

/// Validate a slice of [`StlTriangle`]s.
///
/// Returns `Err` if the slice is empty or any normal has zero length.
#[allow(dead_code)]
pub fn validate_stl_triangles(triangles: &[StlTriangle]) -> StlValidationResult {
    if triangles.is_empty() {
        return Err("triangle list is empty".to_string());
    }
    for (i, tri) in triangles.iter().enumerate() {
        let n = tri.normal;
        let len_sq = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
        if len_sq < 1e-10 {
            return Err(format!("triangle {} has a zero-length normal", i));
        }
    }
    Ok(())
}

/// Compute a normalized face normal from three vertex positions.
#[allow(dead_code)]
pub fn stl_normal_from_vertices(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    let ab = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let ac = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 0.0];
    }
    [cross[0] / len, cross[1] / len, cross[2] / len]
}

/// Format a single `vertex x y z` ASCII STL line.
#[allow(dead_code)]
pub fn stl_ascii_line(x: f32, y: f32, z: f32, precision: usize) -> String {
    format!("      vertex {:.prec$} {:.prec$} {:.prec$}\n", x, y, z, prec = precision)
}

/// Return the solid name from config (falls back to `"oxihuman"` if empty).
#[allow(dead_code)]
pub fn stl_solid_name(cfg: &StlExportConfig) -> &str {
    if cfg.solid_name.is_empty() {
        "oxihuman"
    } else {
        &cfg.solid_name
    }
}

/// Merge two triangle slices into a single `Vec<StlTriangle>`.
#[allow(dead_code)]
pub fn stl_merge_triangles(a: &[StlTriangle], b: &[StlTriangle]) -> Vec<StlTriangle> {
    let mut merged = Vec::with_capacity(a.len() + b.len());
    merged.extend_from_slice(a);
    merged.extend_from_slice(b);
    merged
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_triangle() -> StlTriangle {
        stl_triangle_from_positions(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        )
    }

    #[test]
    fn test_default_stl_config() {
        let cfg = default_stl_config();
        assert_eq!(cfg.mode, StlMode::Ascii);
        assert_eq!(cfg.solid_name, "oxihuman");
        assert_eq!(cfg.precision, 6);
    }

    #[test]
    fn test_stl_normal_from_vertices_z_up() {
        let n = stl_normal_from_vertices(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!((n[2] - 1.0).abs() < 1e-5, "expected Z-up normal");
    }

    #[test]
    fn test_stl_triangle_from_positions() {
        let tri = sample_triangle();
        assert!((tri.normal[2] - 1.0).abs() < 1e-5);
        assert_eq!(tri.attribute, 0);
    }

    #[test]
    fn test_export_to_stl_ascii_contains_solid() {
        let cfg = default_stl_config();
        let tris = vec![sample_triangle()];
        let ascii = export_to_stl_ascii(&tris, &cfg);
        assert!(ascii.starts_with("solid oxihuman"));
        assert!(ascii.ends_with("endsolid oxihuman\n"));
    }

    #[test]
    fn test_export_to_stl_ascii_contains_facet() {
        let cfg = default_stl_config();
        let tris = vec![sample_triangle()];
        let ascii = export_to_stl_ascii(&tris, &cfg);
        assert!(ascii.contains("facet normal"));
        assert!(ascii.contains("outer loop"));
        assert!(ascii.contains("endloop"));
        assert!(ascii.contains("endfacet"));
    }

    #[test]
    fn test_export_to_stl_binary_length() {
        let cfg = default_stl_config();
        let tris = vec![sample_triangle(); 3];
        let bytes = export_to_stl_binary(&tris, &cfg);
        assert_eq!(bytes.len(), stl_file_size(3));
    }

    #[test]
    fn test_stl_file_size() {
        assert_eq!(stl_file_size(0), 84);
        assert_eq!(stl_file_size(1), 134);
        assert_eq!(stl_file_size(10), 584);
    }

    #[test]
    fn test_stl_face_count() {
        let tris = vec![sample_triangle(); 5];
        assert_eq!(stl_face_count(&tris), 5);
    }

    #[test]
    fn test_stl_header_bytes_length() {
        let hdr = stl_header_bytes("test");
        assert_eq!(hdr.len(), 80);
    }

    #[test]
    fn test_stl_header_bytes_content() {
        let hdr = stl_header_bytes("mysolid");
        let text = std::str::from_utf8(&hdr[..22]).unwrap();
        assert!(text.contains("mysolid"));
    }

    #[test]
    fn test_validate_stl_triangles_ok() {
        let tris = vec![sample_triangle()];
        assert!(validate_stl_triangles(&tris).is_ok());
    }

    #[test]
    fn test_validate_stl_triangles_empty() {
        assert!(validate_stl_triangles(&[]).is_err());
    }

    #[test]
    fn test_validate_stl_triangles_zero_normal() {
        let bad = StlTriangle {
            normal: [0.0, 0.0, 0.0],
            v0: [0.0, 0.0, 0.0],
            v1: [1.0, 0.0, 0.0],
            v2: [0.0, 1.0, 0.0],
            attribute: 0,
        };
        assert!(validate_stl_triangles(&[bad]).is_err());
    }

    #[test]
    fn test_stl_merge_triangles() {
        let a = vec![sample_triangle()];
        let b = vec![sample_triangle(), sample_triangle()];
        let merged = stl_merge_triangles(&a, &b);
        assert_eq!(merged.len(), 3);
    }

    #[test]
    fn test_stl_ascii_line_format() {
        let line = stl_ascii_line(1.0, 2.0, 3.0, 3);
        assert_eq!(line, "      vertex 1.000 2.000 3.000\n");
    }

    #[test]
    fn test_stl_solid_name_fallback() {
        let mut cfg = default_stl_config();
        cfg.solid_name = String::new();
        assert_eq!(stl_solid_name(&cfg), "oxihuman");
    }

    #[test]
    fn test_stl_binary_triangle_count_field() {
        let tris = vec![sample_triangle(); 7];
        let cfg = default_stl_config();
        let bytes = export_to_stl_binary(&tris, &cfg);
        let count = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]);
        assert_eq!(count, 7);
    }
}
