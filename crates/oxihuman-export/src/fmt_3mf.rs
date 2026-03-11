// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 3MF (3D Manufacturing Format) export for 3D printing.
//!
//! 3MF is a ZIP-based format containing XML files.  This module builds the
//! required ZIP archive entirely in memory (no external crate required beyond
//! the internal `zip_pack` helper).

use crate::zip_pack::{zip_bytes, ZipEntry};
use oxihuman_mesh::MeshBuffers;

// ── Public types ───────────────────────────────────────────────────────────

/// Unit system used inside the 3MF model XML.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ThreeMfUnit {
    Millimeter,
    Centimeter,
    Meter,
    Inch,
}

/// Options controlling 3MF export behaviour.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThreeMfOptions {
    /// Measurement unit used in the output XML.
    pub unit: ThreeMfUnit,
    /// Human-readable object name embedded in the model XML.
    pub object_name: String,
    /// Multiply every position component by this factor before writing.
    /// The default `1000.0` converts meter-scale OxiHuman meshes to millimetres.
    pub scale: f32,
    /// Author string written into the model metadata comment.
    pub author: String,
}

impl Default for ThreeMfOptions {
    fn default() -> Self {
        ThreeMfOptions {
            unit: ThreeMfUnit::Millimeter,
            object_name: "OxiHuman".to_string(),
            scale: 1000.0,
            author: "COOLJAPAN OU (Team KitaSan)".to_string(),
        }
    }
}

/// Result returned by [`export_3mf`].
#[allow(dead_code)]
#[derive(Debug)]
pub struct ThreeMfExportResult {
    /// Raw bytes of the complete ZIP archive ready to write to disk or send
    /// over the network.
    pub zip_bytes: Vec<u8>,
    /// Number of vertices in the exported mesh.
    pub vertex_count: usize,
    /// Number of triangles in the exported mesh.
    pub triangle_count: usize,
    /// Byte length of the raw model XML string (before ZIP compression).
    pub model_xml_size: usize,
}

// ── Core API ───────────────────────────────────────────────────────────────

/// Export `mesh` to the 3MF format and return the result in memory.
///
/// The returned [`ThreeMfExportResult::zip_bytes`] can be written directly to
/// a `.3mf` file.
pub fn export_3mf(mesh: &MeshBuffers, opts: &ThreeMfOptions) -> ThreeMfExportResult {
    let model_xml = build_3mf_model_xml(mesh, opts);
    let model_xml_size = model_xml.len();

    let content_types = build_content_types_xml();
    let rels = build_rels_xml();

    let entries = vec![
        ZipEntry {
            filename: "[Content_Types].xml".to_string(),
            data: content_types.into_bytes(),
        },
        ZipEntry {
            filename: "_rels/.rels".to_string(),
            data: rels.into_bytes(),
        },
        ZipEntry {
            filename: "3D/3dmodel.model".to_string(),
            data: model_xml.into_bytes(),
        },
    ];

    let zip = zip_bytes(&entries);

    ThreeMfExportResult {
        zip_bytes: zip,
        vertex_count: mesh.positions.len(),
        triangle_count: mesh.indices.len() / 3,
        model_xml_size,
    }
}

/// Build the `3D/3dmodel.model` XML string for the given mesh.
pub fn build_3mf_model_xml(mesh: &MeshBuffers, opts: &ThreeMfOptions) -> String {
    let unit_str = unit_string(&opts.unit);
    let s = opts.scale;

    // ── Vertices ──────────────────────────────────────────────────────────
    let mut vertices_xml = String::new();
    for p in &mesh.positions {
        vertices_xml.push_str(&format!(
            "          <vertex x=\"{:.6}\" y=\"{:.6}\" z=\"{:.6}\"/>\n",
            p[0] * s,
            p[1] * s,
            p[2] * s,
        ));
    }

    // ── Triangles ─────────────────────────────────────────────────────────
    let mut triangles_xml = String::new();
    let idx = &mesh.indices;
    let tri_count = idx.len() / 3;
    for t in 0..tri_count {
        triangles_xml.push_str(&format!(
            "          <triangle v1=\"{}\" v2=\"{}\" v3=\"{}\"/>\n",
            idx[t * 3],
            idx[t * 3 + 1],
            idx[t * 3 + 2],
        ));
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!-- Author: {} -->\n\
         <model unit=\"{}\" xml:lang=\"en-US\" \
         xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">\n\
           <resources>\n\
             <object id=\"1\" name=\"{}\" type=\"model\">\n\
               <mesh>\n\
                 <vertices>\n\
         {}\
                 </vertices>\n\
                 <triangles>\n\
         {}\
                 </triangles>\n\
               </mesh>\n\
             </object>\n\
           </resources>\n\
           <build>\n\
             <item objectid=\"1\"/>\n\
           </build>\n\
         </model>\n",
        opts.author, unit_str, opts.object_name, vertices_xml, triangles_xml,
    )
}

/// Build the `[Content_Types].xml` entry required by the 3MF specification.
pub fn build_content_types_xml() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
     <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\n\
       <Default Extension=\"rels\" \
         ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\n\
       <Default Extension=\"model\" \
         ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\n\
     </Types>\n"
        .to_string()
}

/// Build the `_rels/.rels` relationship file required by the 3MF specification.
pub fn build_rels_xml() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
     <Relationships \
       xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n\
       <Relationship Id=\"rel0\" \
         Target=\"/3D/3dmodel.model\" \
         Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\n\
     </Relationships>\n"
        .to_string()
}

/// Return the XML unit attribute string for the given [`ThreeMfUnit`].
pub fn unit_string(u: &ThreeMfUnit) -> &'static str {
    match u {
        ThreeMfUnit::Millimeter => "millimeter",
        ThreeMfUnit::Centimeter => "centimeter",
        ThreeMfUnit::Meter => "meter",
        ThreeMfUnit::Inch => "inch",
    }
}

/// Validate that `data` looks like a 3MF ZIP archive.
///
/// Checks:
/// 1. Starts with the ZIP PK magic bytes `[0x50, 0x4B, 0x03, 0x04]`.
/// 2. Contains the byte sequence `"3dmodel.model"` somewhere in the archive.
pub fn validate_3mf_zip(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    // ZIP local-file-header magic: PK\x03\x04
    if data[0..4] != [0x50, 0x4B, 0x03, 0x04] {
        return false;
    }
    // Check that the archive references the required model file
    let needle = b"3dmodel.model";
    data.windows(needle.len()).any(|w| w == needle)
}

/// Heuristic check for whether a mesh is suitable for 3D printing.
///
/// Returns `true` when:
/// * The mesh has at least one vertex.
/// * The index count is a multiple of 3 (complete triangles).
/// * Every index is within `[0, vertex_count)`.
pub fn mesh_is_printable(mesh: &MeshBuffers) -> bool {
    if mesh.positions.is_empty() {
        return false;
    }
    if !mesh.indices.len().is_multiple_of(3) {
        return false;
    }
    let n = mesh.positions.len() as u32;
    mesh.indices.iter().all(|&i| i < n)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn simple_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [0.001, 0.0, 0.0], [0.0, 0.001, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    fn empty_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            indices: vec![],
            has_suit: false,
        })
    }

    // ── unit_string ───────────────────────────────────────────────────────

    #[test]
    fn unit_string_millimeter() {
        assert_eq!(unit_string(&ThreeMfUnit::Millimeter), "millimeter");
    }

    #[test]
    fn unit_string_centimeter() {
        assert_eq!(unit_string(&ThreeMfUnit::Centimeter), "centimeter");
    }

    #[test]
    fn unit_string_meter() {
        assert_eq!(unit_string(&ThreeMfUnit::Meter), "meter");
    }

    #[test]
    fn unit_string_inch() {
        assert_eq!(unit_string(&ThreeMfUnit::Inch), "inch");
    }

    // ── build_3mf_model_xml ───────────────────────────────────────────────

    #[test]
    fn model_xml_contains_vertices_tag() {
        let mesh = simple_mesh();
        let xml = build_3mf_model_xml(&mesh, &ThreeMfOptions::default());
        assert!(xml.contains("<vertices>"), "XML must contain <vertices>");
    }

    #[test]
    fn model_xml_contains_triangles_tag() {
        let mesh = simple_mesh();
        let xml = build_3mf_model_xml(&mesh, &ThreeMfOptions::default());
        assert!(xml.contains("<triangles>"), "XML must contain <triangles>");
    }

    #[test]
    fn model_xml_vertex_count_matches() {
        let mesh = simple_mesh();
        let xml = build_3mf_model_xml(&mesh, &ThreeMfOptions::default());
        let count = xml.matches("<vertex ").count();
        assert_eq!(count, mesh.positions.len(), "XML vertex count mismatch");
    }

    #[test]
    fn model_xml_triangle_count_matches() {
        let mesh = simple_mesh();
        let xml = build_3mf_model_xml(&mesh, &ThreeMfOptions::default());
        let count = xml.matches("<triangle ").count();
        assert_eq!(count, mesh.indices.len() / 3, "XML triangle count mismatch");
    }

    #[test]
    fn model_xml_contains_unit() {
        let opts = ThreeMfOptions::default();
        let mesh = simple_mesh();
        let xml = build_3mf_model_xml(&mesh, &opts);
        assert!(
            xml.contains("millimeter"),
            "XML should contain unit attribute"
        );
    }

    // ── build_content_types_xml ───────────────────────────────────────────

    #[test]
    fn content_types_contains_3dmodel() {
        let ct = build_content_types_xml();
        assert!(
            ct.contains("3dmanufacturing-3dmodel"),
            "content types should reference 3dmodel content type"
        );
    }

    #[test]
    fn content_types_is_xml() {
        let ct = build_content_types_xml();
        assert!(ct.starts_with("<?xml"), "should start with XML declaration");
    }

    // ── build_rels_xml ────────────────────────────────────────────────────

    #[test]
    fn rels_contains_relationship() {
        let rels = build_rels_xml();
        assert!(
            rels.contains("Relationship"),
            "rels XML should contain Relationship element"
        );
    }

    #[test]
    fn rels_points_to_3dmodel() {
        let rels = build_rels_xml();
        assert!(
            rels.contains("3dmodel.model"),
            "rels XML should reference 3dmodel.model"
        );
    }

    // ── export_3mf ────────────────────────────────────────────────────────

    #[test]
    fn export_3mf_zip_starts_with_pk_magic() {
        let mesh = simple_mesh();
        let result = export_3mf(&mesh, &ThreeMfOptions::default());
        assert!(result.zip_bytes.len() >= 4, "ZIP bytes should be non-empty");
        assert_eq!(
            &result.zip_bytes[0..4],
            &[0x50, 0x4B, 0x03, 0x04],
            "ZIP must start with PK magic"
        );
    }

    #[test]
    fn export_3mf_result_vertex_count_matches() {
        let mesh = simple_mesh();
        let result = export_3mf(&mesh, &ThreeMfOptions::default());
        assert_eq!(result.vertex_count, 3);
    }

    #[test]
    fn export_3mf_result_triangle_count_matches() {
        let mesh = simple_mesh();
        let result = export_3mf(&mesh, &ThreeMfOptions::default());
        assert_eq!(result.triangle_count, 1);
    }

    #[test]
    fn export_3mf_model_xml_size_positive() {
        let mesh = simple_mesh();
        let result = export_3mf(&mesh, &ThreeMfOptions::default());
        assert!(result.model_xml_size > 0);
    }

    // ── validate_3mf_zip ──────────────────────────────────────────────────

    #[test]
    fn validate_3mf_zip_valid() {
        let mesh = simple_mesh();
        let result = export_3mf(&mesh, &ThreeMfOptions::default());
        assert!(
            validate_3mf_zip(&result.zip_bytes),
            "exported 3MF should pass validation"
        );
    }

    #[test]
    fn validate_3mf_zip_invalid_data() {
        let garbage = b"not a zip at all";
        assert!(!validate_3mf_zip(garbage));
    }

    #[test]
    fn validate_3mf_zip_empty_data() {
        assert!(!validate_3mf_zip(&[]));
    }

    // ── mesh_is_printable ─────────────────────────────────────────────────

    #[test]
    fn mesh_is_printable_valid_mesh() {
        assert!(mesh_is_printable(&simple_mesh()));
    }

    #[test]
    fn mesh_is_printable_empty_mesh_false() {
        assert!(!mesh_is_printable(&empty_mesh()));
    }

    #[test]
    fn mesh_is_printable_out_of_range_index_false() {
        let mut mesh = simple_mesh();
        mesh.indices.push(999); // out of range, also makes count not divisible by 3
        assert!(!mesh_is_printable(&mesh));
    }
}
