// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! PLY (Polygon File Format / Stanford Triangle Format) ASCII and binary export.
//!
//! Supports three encoding modes: ASCII, binary little-endian, and binary big-endian.

#![allow(dead_code)]

// ── Enums ─────────────────────────────────────────────────────────────────────

/// PLY output encoding mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlyMode {
    /// Human-readable ASCII PLY.
    Ascii,
    /// Binary little-endian PLY.
    BinaryLe,
    /// Binary big-endian PLY.
    BinaryBe,
}

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for PLY export.
#[derive(Debug, Clone)]
pub struct PlyExportConfig {
    /// Output encoding mode. Default: `PlyMode::Ascii`.
    pub mode: PlyMode,
    /// Comment string written after the format line. Default: `"exported by OxiHuman"`.
    pub comment: String,
    /// Decimal precision for ASCII float values. Default: `6`.
    pub precision: usize,
}

impl Default for PlyExportConfig {
    fn default() -> Self {
        Self {
            mode: PlyMode::Ascii,
            comment: "exported by OxiHuman".to_string(),
            precision: 6,
        }
    }
}

/// A single PLY element (e.g., `vertex` or `face`) with its properties.
#[derive(Debug, Clone)]
pub struct PlyElement {
    /// Element name (e.g., `"vertex"`, `"face"`).
    pub name: String,
    /// Number of instances of this element.
    pub count: usize,
    /// Property names in declaration order.
    pub properties: Vec<String>,
    /// Row data: each inner `Vec<f32>` holds one element's property values.
    pub data: Vec<Vec<f32>>,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// Result type for PLY validation.
pub type PlyValidationResult = Result<(), String>;

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a default [`PlyExportConfig`].
#[allow(dead_code)]
pub fn default_ply_config() -> PlyExportConfig {
    PlyExportConfig::default()
}

/// Construct a new [`PlyElement`] with a given name and property list.
#[allow(dead_code)]
pub fn new_ply_element(name: &str, properties: &[&str]) -> PlyElement {
    PlyElement {
        name: name.to_string(),
        count: 0,
        properties: properties.iter().map(|s| s.to_string()).collect(),
        data: Vec::new(),
    }
}

/// Append `element` to the `elements` vector and return the new length.
#[allow(dead_code)]
pub fn add_ply_element(elements: &mut Vec<PlyElement>, element: PlyElement) -> usize {
    elements.push(element);
    elements.len()
}

/// Return the number of elements in the `elements` slice.
#[allow(dead_code)]
pub fn element_count(elements: &[PlyElement]) -> usize {
    elements.len()
}

/// Build a PLY header string from `elements` and `cfg`.
#[allow(dead_code)]
pub fn ply_header_string(elements: &[PlyElement], cfg: &PlyExportConfig) -> String {
    let format_str = match cfg.mode {
        PlyMode::Ascii => "ascii 1.0",
        PlyMode::BinaryLe => "binary_little_endian 1.0",
        PlyMode::BinaryBe => "binary_big_endian 1.0",
    };
    let mut hdr = String::from("ply\n");
    hdr.push_str(&format!("format {} \n", format_str));
    if !cfg.comment.is_empty() {
        hdr.push_str(&format!("comment {}\n", cfg.comment));
    }
    for elem in elements {
        hdr.push_str(&format!("element {} {}\n", elem.name, elem.count));
        for prop in &elem.properties {
            hdr.push_str(&format!("property float {}\n", prop));
        }
    }
    hdr.push_str("end_header\n");
    hdr
}

/// Serialize `elements` to a PLY ASCII string.
#[allow(dead_code)]
pub fn ply_to_ascii_string(elements: &[PlyElement], cfg: &PlyExportConfig) -> String {
    let mut out = ply_header_string(elements, cfg);
    let prec = cfg.precision;
    for elem in elements {
        for row in &elem.data {
            let row_str: Vec<String> = row.iter().map(|v| format!("{:.prec$}", v, prec = prec)).collect();
            out.push_str(&row_str.join(" "));
            out.push('\n');
        }
    }
    out
}

/// Serialize `elements` to binary little-endian PLY bytes.
#[allow(dead_code)]
pub fn ply_to_binary_le(elements: &[PlyElement], cfg: &PlyExportConfig) -> Vec<u8> {
    let header = ply_header_string(elements, cfg);
    let mut bytes: Vec<u8> = header.into_bytes();
    for elem in elements {
        for row in &elem.data {
            for &v in row {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
    }
    bytes
}

/// Return the vertex count from the first element named `"vertex"`, or `0`.
#[allow(dead_code)]
pub fn ply_vertex_count(elements: &[PlyElement]) -> usize {
    elements.iter().find(|e| e.name == "vertex").map_or(0, |e| e.count)
}

/// Return the face count from the first element named `"face"`, or `0`.
#[allow(dead_code)]
pub fn ply_face_count(elements: &[PlyElement]) -> usize {
    elements.iter().find(|e| e.name == "face").map_or(0, |e| e.count)
}

/// Validate a slice of [`PlyElement`]s.
///
/// Returns `Err` if elements is empty or any element's `data.len()` mismatches `count`.
#[allow(dead_code)]
pub fn validate_ply(elements: &[PlyElement]) -> PlyValidationResult {
    if elements.is_empty() {
        return Err("no PLY elements defined".to_string());
    }
    for elem in elements {
        if elem.data.len() != elem.count {
            return Err(format!(
                "element '{}': count={} but data rows={}",
                elem.name,
                elem.count,
                elem.data.len()
            ));
        }
    }
    Ok(())
}

/// Estimate the binary file size in bytes for the given elements (little-endian floats).
#[allow(dead_code)]
pub fn ply_file_size(elements: &[PlyElement], cfg: &PlyExportConfig) -> usize {
    let header_len = ply_header_string(elements, cfg).len();
    let data_len: usize = elements
        .iter()
        .map(|e| e.count * e.properties.len() * 4)
        .sum();
    header_len + data_len
}

/// Return the property names of the first element with the given `name`, or empty slice wrapper.
#[allow(dead_code)]
pub fn ply_property_list(elements: &[PlyElement], name: &str) -> Vec<String> {
    elements
        .iter()
        .find(|e| e.name == name)
        .map(|e| e.properties.clone())
        .unwrap_or_default()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vertex_element() -> PlyElement {
        let mut elem = new_ply_element("vertex", &["x", "y", "z"]);
        elem.count = 2;
        elem.data = vec![vec![0.0, 1.0, 2.0], vec![3.0, 4.0, 5.0]];
        elem
    }

    fn make_face_element() -> PlyElement {
        let mut elem = new_ply_element("face", &["v0", "v1", "v2"]);
        elem.count = 1;
        elem.data = vec![vec![0.0, 1.0, 2.0]];
        elem
    }

    #[test]
    fn test_default_ply_config() {
        let cfg = default_ply_config();
        assert_eq!(cfg.mode, PlyMode::Ascii);
        assert_eq!(cfg.precision, 6);
        assert!(!cfg.comment.is_empty());
    }

    #[test]
    fn test_new_ply_element_fields() {
        let elem = new_ply_element("vertex", &["x", "y", "z"]);
        assert_eq!(elem.name, "vertex");
        assert_eq!(elem.properties.len(), 3);
        assert_eq!(elem.count, 0);
        assert!(elem.data.is_empty());
    }

    #[test]
    fn test_add_ply_element_returns_length() {
        let mut elements: Vec<PlyElement> = Vec::new();
        let n = add_ply_element(&mut elements, make_vertex_element());
        assert_eq!(n, 1);
        let n2 = add_ply_element(&mut elements, make_face_element());
        assert_eq!(n2, 2);
    }

    #[test]
    fn test_element_count() {
        let elements = vec![make_vertex_element(), make_face_element()];
        assert_eq!(element_count(&elements), 2);
    }

    #[test]
    fn test_ply_header_contains_ply() {
        let cfg = default_ply_config();
        let elements = vec![make_vertex_element()];
        let hdr = ply_header_string(&elements, &cfg);
        assert!(hdr.starts_with("ply\n"));
        assert!(hdr.contains("end_header"));
        assert!(hdr.contains("element vertex 2"));
    }

    #[test]
    fn test_ply_header_binary_le_format() {
        let mut cfg = default_ply_config();
        cfg.mode = PlyMode::BinaryLe;
        let elements = vec![make_vertex_element()];
        let hdr = ply_header_string(&elements, &cfg);
        assert!(hdr.contains("binary_little_endian"));
    }

    #[test]
    fn test_ply_header_binary_be_format() {
        let mut cfg = default_ply_config();
        cfg.mode = PlyMode::BinaryBe;
        let elements = vec![make_vertex_element()];
        let hdr = ply_header_string(&elements, &cfg);
        assert!(hdr.contains("binary_big_endian"));
    }

    #[test]
    fn test_ply_to_ascii_string_contains_values() {
        let cfg = default_ply_config();
        let elements = vec![make_vertex_element()];
        let ascii = ply_to_ascii_string(&elements, &cfg);
        assert!(ascii.contains("0.000000"));
        assert!(ascii.contains("3.000000"));
    }

    #[test]
    fn test_ply_to_binary_le_starts_with_header() {
        let mut cfg = default_ply_config();
        cfg.mode = PlyMode::BinaryLe;
        let elements = vec![make_vertex_element()];
        let bytes = ply_to_binary_le(&elements, &cfg);
        assert!(bytes.starts_with(b"ply\n"));
    }

    #[test]
    fn test_ply_vertex_count() {
        let elements = vec![make_vertex_element(), make_face_element()];
        assert_eq!(ply_vertex_count(&elements), 2);
    }

    #[test]
    fn test_ply_face_count() {
        let elements = vec![make_vertex_element(), make_face_element()];
        assert_eq!(ply_face_count(&elements), 1);
    }

    #[test]
    fn test_ply_vertex_count_missing() {
        let elements = vec![make_face_element()];
        assert_eq!(ply_vertex_count(&elements), 0);
    }

    #[test]
    fn test_validate_ply_empty() {
        assert!(validate_ply(&[]).is_err());
    }

    #[test]
    fn test_validate_ply_mismatch() {
        let mut elem = make_vertex_element();
        elem.count = 99; // mismatch with data.len() == 2
        assert!(validate_ply(&[elem]).is_err());
    }

    #[test]
    fn test_validate_ply_ok() {
        let elements = vec![make_vertex_element(), make_face_element()];
        assert!(validate_ply(&elements).is_ok());
    }

    #[test]
    fn test_ply_file_size_positive() {
        let cfg = default_ply_config();
        let mut cfg_le = cfg.clone();
        cfg_le.mode = PlyMode::BinaryLe;
        let elements = vec![make_vertex_element()];
        let size = ply_file_size(&elements, &cfg_le);
        assert!(size > 0);
    }

    #[test]
    fn test_ply_property_list_found() {
        let elements = vec![make_vertex_element()];
        let props = ply_property_list(&elements, "vertex");
        assert_eq!(props, vec!["x", "y", "z"]);
    }

    #[test]
    fn test_ply_property_list_not_found() {
        let elements = vec![make_vertex_element()];
        let props = ply_property_list(&elements, "edge");
        assert!(props.is_empty());
    }

    #[test]
    fn test_ply_mode_debug() {
        assert_eq!(format!("{:?}", PlyMode::Ascii), "Ascii");
        assert_eq!(format!("{:?}", PlyMode::BinaryLe), "BinaryLe");
        assert_eq!(format!("{:?}", PlyMode::BinaryBe), "BinaryBe");
    }
}
