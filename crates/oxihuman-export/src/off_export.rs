//! OFF (Object File Format) mesh export — simple ASCII format with vertex/face lists.
//!
//! Supports optional per-face colours via the COFF variant.

use std::fmt::Write as FmtWrite;

// ── Structs ──────────────────────────────────────────────────────────────────

/// Configuration for OFF export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OffExportConfig {
    /// Write per-face colours (COFF header instead of OFF).
    pub color_mode: bool,
    /// Number of decimal places for vertex coordinates.
    pub precision: usize,
}

/// In-memory OFF document.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OffDocument {
    /// Export configuration.
    pub config: OffExportConfig,
    /// Vertex positions.
    pub vertices: Vec<[f32; 3]>,
    /// Triangle face indices.
    pub faces: Vec<[u32; 3]>,
    /// Per-face colours (RGB in [0,1]).  Empty = no colour.
    pub face_colors: Vec<(usize, [f32; 3])>,
}

// ── Public functions ──────────────────────────────────────────────────────────

/// Returns a sensible default [`OffExportConfig`].
#[allow(dead_code)]
pub fn default_off_config() -> OffExportConfig {
    OffExportConfig {
        color_mode: false,
        precision: 6,
    }
}

/// Creates a new, empty [`OffDocument`] with the given config.
#[allow(dead_code)]
pub fn new_off_document(cfg: &OffExportConfig) -> OffDocument {
    OffDocument {
        config: cfg.clone(),
        vertices: Vec::new(),
        faces: Vec::new(),
        face_colors: Vec::new(),
    }
}

/// Replaces the mesh data in the document.
#[allow(dead_code)]
pub fn off_set_mesh(doc: &mut OffDocument, verts: &[[f32; 3]], faces: &[[u32; 3]]) {
    doc.vertices = verts.to_vec();
    doc.faces = faces.to_vec();
    doc.face_colors.clear();
}

/// Serialises the document to an OFF ASCII string.
#[allow(dead_code)]
pub fn off_to_string(doc: &OffDocument) -> String {
    let mut out = String::new();
    let header = if doc.config.color_mode { "COFF" } else { "OFF" };
    let _ = writeln!(out, "{}", header);
    let _ = writeln!(
        out,
        "{} {} 0",
        doc.vertices.len(),
        doc.faces.len()
    );
    let prec = doc.config.precision;
    for v in &doc.vertices {
        let _ = writeln!(out, "{:.prec$} {:.prec$} {:.prec$}", v[0], v[1], v[2]);
    }
    // Build a colour lookup by face index
    let mut color_map: std::collections::HashMap<usize, [f32; 3]> =
        std::collections::HashMap::new();
    for &(idx, col) in &doc.face_colors {
        color_map.insert(idx, col);
    }
    for (fi, f) in doc.faces.iter().enumerate() {
        if doc.config.color_mode {
            if let Some(col) = color_map.get(&fi) {
                let _ = writeln!(
                    out,
                    "3 {} {} {} {:.4} {:.4} {:.4}",
                    f[0], f[1], f[2], col[0], col[1], col[2]
                );
            } else {
                let _ = writeln!(out, "3 {} {} {} 0.5 0.5 0.5", f[0], f[1], f[2]);
            }
        } else {
            let _ = writeln!(out, "3 {} {} {}", f[0], f[1], f[2]);
        }
    }
    out
}

/// Writes the OFF document to a file at `path`.
#[allow(dead_code)]
pub fn off_write_to_file(doc: &OffDocument, path: &str) -> Result<(), String> {
    let content = off_to_string(doc);
    std::fs::write(path, content).map_err(|e| e.to_string())
}

/// Returns the number of vertices in the document.
#[allow(dead_code)]
pub fn off_vertex_count(doc: &OffDocument) -> usize {
    doc.vertices.len()
}

/// Returns the number of faces in the document.
#[allow(dead_code)]
pub fn off_face_count(doc: &OffDocument) -> usize {
    doc.faces.len()
}

/// Enables or disables per-face colour output.
#[allow(dead_code)]
pub fn off_set_color_mode(cfg: &mut OffExportConfig, enabled: bool) {
    cfg.color_mode = enabled;
}

/// Adds a per-face colour override for `face_idx`.
#[allow(dead_code)]
pub fn off_add_color(doc: &mut OffDocument, face_idx: usize, r: f32, g: f32, b: f32) {
    doc.face_colors.push((face_idx, [r, g, b]));
}

/// Clears all mesh data and colours from the document.
#[allow(dead_code)]
pub fn off_document_clear(doc: &mut OffDocument) {
    doc.vertices.clear();
    doc.faces.clear();
    doc.face_colors.clear();
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_off_config();
        assert!(!cfg.color_mode);
        assert_eq!(cfg.precision, 6);
    }

    #[test]
    fn test_new_document_empty() {
        let cfg = default_off_config();
        let doc = new_off_document(&cfg);
        assert_eq!(off_vertex_count(&doc), 0);
        assert_eq!(off_face_count(&doc), 0);
    }

    #[test]
    fn test_set_mesh_stores_data() {
        let cfg = default_off_config();
        let mut doc = new_off_document(&cfg);
        let verts = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let faces = [[0, 1, 2]];
        off_set_mesh(&mut doc, &verts, &faces);
        assert_eq!(off_vertex_count(&doc), 3);
        assert_eq!(off_face_count(&doc), 1);
    }

    #[test]
    fn test_off_to_string_header() {
        let cfg = default_off_config();
        let doc = new_off_document(&cfg);
        let s = off_to_string(&doc);
        assert!(s.starts_with("OFF\n"));
    }

    #[test]
    fn test_off_to_string_coff_header() {
        let mut cfg = default_off_config();
        cfg.color_mode = true;
        let doc = new_off_document(&cfg);
        let s = off_to_string(&doc);
        assert!(s.starts_with("COFF\n"));
    }

    #[test]
    fn test_off_to_string_contains_counts() {
        let cfg = default_off_config();
        let mut doc = new_off_document(&cfg);
        let verts = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let faces = [[0u32, 1, 2]];
        off_set_mesh(&mut doc, &verts, &faces);
        let s = off_to_string(&doc);
        assert!(s.contains("3 1 0"));
    }

    #[test]
    fn test_off_set_color_mode() {
        let mut cfg = default_off_config();
        off_set_color_mode(&mut cfg, true);
        assert!(cfg.color_mode);
        off_set_color_mode(&mut cfg, false);
        assert!(!cfg.color_mode);
    }

    #[test]
    fn test_off_add_color() {
        let mut cfg = default_off_config();
        cfg.color_mode = true;
        let mut doc = new_off_document(&cfg);
        let verts = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let faces = [[0u32, 1, 2]];
        off_set_mesh(&mut doc, &verts, &faces);
        off_add_color(&mut doc, 0, 1.0, 0.0, 0.0);
        let s = off_to_string(&doc);
        assert!(s.contains("1.0000") || s.contains("1.000000"));
    }

    #[test]
    fn test_off_document_clear() {
        let cfg = default_off_config();
        let mut doc = new_off_document(&cfg);
        let verts = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let faces = [[0u32, 1, 2]];
        off_set_mesh(&mut doc, &verts, &faces);
        off_document_clear(&mut doc);
        assert_eq!(off_vertex_count(&doc), 0);
        assert_eq!(off_face_count(&doc), 0);
    }

    #[test]
    fn test_off_face_indices_in_output() {
        let cfg = default_off_config();
        let mut doc = new_off_document(&cfg);
        let verts = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let faces = [[0u32, 1, 2]];
        off_set_mesh(&mut doc, &verts, &faces);
        let s = off_to_string(&doc);
        assert!(s.contains("3 0 1 2"));
    }
}
