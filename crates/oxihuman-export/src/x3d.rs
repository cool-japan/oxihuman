// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! X3D (XML-based 3D) mesh export — ISO/IEC 19775.
//!
//! Writes X3D 3.3 XML directly as strings; no external XML crate required.

#![allow(dead_code)]

use std::path::Path;

use anyhow::Context;
use oxihuman_mesh::MeshBuffers;

// ── Options ──────────────────────────────────────────────────────────────────

/// Options controlling X3D export output.
pub struct X3dExportOptions {
    /// X3D profile string, e.g. `"Interchange"`.  Default: `"Interchange"`.
    pub profile: String,
    /// Emit `<Normal>` nodes. Default: `true`.
    pub include_normals: bool,
    /// Emit `<TextureCoordinate>` nodes. Default: `true`.
    pub include_uvs: bool,
    /// `DEF` name given to the primary `<Shape>` node. Default: `"OxiHumanMesh"`.
    pub mesh_name: String,
    /// Author metadata string. Default: `"OxiHuman"`.
    pub author: String,
    /// `solid` attribute on `IndexedFaceSet`.  `false` = two-sided. Default: `false`.
    pub solid: bool,
    /// `colorPerVertex` attribute on `IndexedFaceSet`. Default: `false`.
    pub color_per_vertex: bool,
    /// Number of spaces per indentation level. Default: `2`.
    pub indent: usize,
}

impl Default for X3dExportOptions {
    fn default() -> Self {
        Self {
            profile: "Interchange".to_string(),
            include_normals: true,
            include_uvs: true,
            mesh_name: "OxiHumanMesh".to_string(),
            author: "OxiHuman".to_string(),
            solid: false,
            color_per_vertex: false,
            indent: 2,
        }
    }
}

// ── Stats ─────────────────────────────────────────────────────────────────────

/// Statistics returned after a successful X3D export.
pub struct X3dExportStats {
    pub vertex_count: usize,
    pub face_count: usize,
    pub has_normals: bool,
    pub has_uvs: bool,
    pub byte_size: usize,
}

// ── Formatting helpers ────────────────────────────────────────────────────────

/// Format a `[[f32; 3]]` slice as space-separated X3D coord string:
/// `"x1 y1 z1, x2 y2 z2, ..."`.
pub fn format_coord_array(coords: &[[f32; 3]]) -> String {
    coords
        .iter()
        .map(|v| format!("{} {} {}", v[0], v[1], v[2]))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Format a `[[f32; 2]]` slice as space-separated X3D texcoord string:
/// `"u1 v1, u2 v2, ..."`.
fn format_uv_array(uvs: &[[f32; 2]]) -> String {
    uvs.iter()
        .map(|v| format!("{} {}", v[0], v[1]))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Format a flat index array with `-1` face terminators.
///
/// `stride` is the number of vertices per face (typically `3` for triangles).
/// Every `stride` indices will be followed by a `-1` terminator.
pub fn format_index_array(indices: &[u32], stride: usize) -> String {
    if stride == 0 || indices.is_empty() {
        return String::new();
    }
    let mut parts: Vec<String> = Vec::with_capacity(indices.len() + indices.len() / stride);
    for chunk in indices.chunks(stride) {
        for idx in chunk {
            parts.push(idx.to_string());
        }
        parts.push("-1".to_string());
    }
    parts.join(" ")
}

/// Escape text for XML attribute values.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Return a string of `n` spaces.
fn ind(n: usize) -> String {
    " ".repeat(n)
}

// ── Single-mesh builder ───────────────────────────────────────────────────────

/// Build the `<Shape>` block for one mesh.
fn build_shape_block(
    mesh: &MeshBuffers,
    name: &str,
    opts: &X3dExportOptions,
    base_indent: usize,
) -> String {
    let sp = ind(base_indent);
    let sp2 = ind(base_indent + opts.indent);
    let sp3 = ind(base_indent + opts.indent * 2);
    let sp4 = ind(base_indent + opts.indent * 3);

    let emit_normals = opts.include_normals && !mesh.normals.is_empty();
    let emit_uvs = opts.include_uvs && !mesh.uvs.is_empty();

    let solid_str = if opts.solid { "true" } else { "false" };
    let cpv_str = if opts.color_per_vertex {
        "true"
    } else {
        "false"
    };

    let coord_idx = format_index_array(&mesh.indices, 3);
    let coord_str = format_coord_array(&mesh.positions);

    let mut out = String::new();

    // <Shape DEF="...">
    out.push_str(&format!("{}<Shape DEF=\"{}\">\n", sp, xml_escape(name)));

    // <Appearance>
    out.push_str(&format!("{}<Appearance>\n", sp2));
    out.push_str(&format!(
        "{}<Material diffuseColor=\"0.8 0.8 0.8\"/>\n",
        sp3
    ));
    out.push_str(&format!("{}</Appearance>\n", sp2));

    // <IndexedFaceSet ...>
    out.push_str(&format!(
        "{}<IndexedFaceSet solid=\"{}\" colorPerVertex=\"{}\" coordIndex=\"{}\"",
        sp2, solid_str, cpv_str, coord_idx
    ));
    if emit_normals {
        out.push_str(&format!(
            " normalIndex=\"{}\"",
            format_index_array(&mesh.indices, 3)
        ));
    }
    if emit_uvs {
        out.push_str(&format!(
            " texCoordIndex=\"{}\"",
            format_index_array(&mesh.indices, 3)
        ));
    }
    out.push_str(">\n");

    // <Coordinate>
    out.push_str(&format!("{}<Coordinate point=\"{}\"/>\n", sp3, coord_str));

    // <Normal>
    if emit_normals {
        let normal_str = format_coord_array(&mesh.normals);
        out.push_str(&format!("{}<Normal vector=\"{}\"/>\n", sp3, normal_str));
    }

    // <TextureCoordinate>
    if emit_uvs {
        let uv_str = format_uv_array(&mesh.uvs);
        out.push_str(&format!(
            "{}<TextureCoordinate point=\"{}\"/>\n",
            sp3, uv_str
        ));
    }

    // close tags
    out.push_str(&format!("{}</IndexedFaceSet>\n", sp2));
    out.push_str(&format!("{}</Shape>\n", sp));

    // suppress unused-variable warning on sp4
    let _ = sp4;

    out
}

/// Build a complete X3D XML document for a single mesh.
///
/// Returns the XML string and export statistics.
pub fn build_x3d(mesh: &MeshBuffers, options: &X3dExportOptions) -> (String, X3dExportStats) {
    let sp1 = ind(options.indent);
    let sp2 = ind(options.indent * 2);

    let emit_normals = options.include_normals && !mesh.normals.is_empty();
    let emit_uvs = options.include_uvs && !mesh.uvs.is_empty();
    let face_count = mesh.indices.len() / 3;

    let mut out = String::new();

    // XML + DOCTYPE declaration
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(
        "<!DOCTYPE X3D PUBLIC \"ISO//Web3D//DTD X3D 3.3//EN\" \
         \"http://www.web3d.org/specifications/x3d-3.3.dtd\">\n",
    );

    // <X3D ...>
    out.push_str(&format!(
        "<X3D profile=\"{}\" version=\"3.3\"\n",
        xml_escape(&options.profile)
    ));
    out.push_str(
        "     xmlns:xsd=\"http://www.w3.org/2001/XMLSchema-instance\"\
         \n     xsd:noNamespaceSchemaLocation=\"http://www.web3d.org/specifications/x3d-3.3.xsd\">\n",
    );

    // <head>
    out.push_str(&format!("{}<head>\n", sp1));
    out.push_str(&format!(
        "{}<meta name=\"author\" content=\"{}\"/>\n",
        sp2,
        xml_escape(&options.author)
    ));
    out.push_str(&format!(
        "{}<meta name=\"generator\" content=\"OxiHuman x3d exporter\"/>\n",
        sp2
    ));
    out.push_str(&format!("{}</head>\n", sp1));

    // <Scene>
    out.push_str(&format!("{}<Scene>\n", sp1));

    // Shape block at indent level 2
    out.push_str(&build_shape_block(
        mesh,
        &options.mesh_name,
        options,
        options.indent * 2,
    ));

    out.push_str(&format!("{}</Scene>\n", sp1));
    out.push_str("</X3D>\n");

    let byte_size = out.len();

    let stats = X3dExportStats {
        vertex_count: mesh.positions.len(),
        face_count,
        has_normals: emit_normals,
        has_uvs: emit_uvs,
        byte_size,
    };

    (out, stats)
}

// ── File export ───────────────────────────────────────────────────────────────

/// Export a single mesh to an X3D file.
pub fn export_x3d(
    mesh: &MeshBuffers,
    path: &Path,
    options: &X3dExportOptions,
) -> anyhow::Result<X3dExportStats> {
    let (content, mut stats) = build_x3d(mesh, options);
    std::fs::write(path, &content)
        .with_context(|| format!("Failed to write X3D file: {}", path.display()))?;
    stats.byte_size = content.len();
    Ok(stats)
}

// ── Multi-mesh scene ──────────────────────────────────────────────────────────

/// Build a complete X3D XML document containing multiple meshes, each as a
/// separate `<Shape>` node inside the same `<Scene>`.
pub fn build_x3d_scene(meshes: &[(&MeshBuffers, &str)], options: &X3dExportOptions) -> String {
    let sp1 = ind(options.indent);
    let sp2 = ind(options.indent * 2);

    let mut out = String::new();

    // XML + DOCTYPE declaration
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(
        "<!DOCTYPE X3D PUBLIC \"ISO//Web3D//DTD X3D 3.3//EN\" \
         \"http://www.web3d.org/specifications/x3d-3.3.dtd\">\n",
    );

    out.push_str(&format!(
        "<X3D profile=\"{}\" version=\"3.3\"\n",
        xml_escape(&options.profile)
    ));
    out.push_str(
        "     xmlns:xsd=\"http://www.w3.org/2001/XMLSchema-instance\"\
         \n     xsd:noNamespaceSchemaLocation=\"http://www.web3d.org/specifications/x3d-3.3.xsd\">\n",
    );

    // <head>
    out.push_str(&format!("{}<head>\n", sp1));
    out.push_str(&format!(
        "{}<meta name=\"author\" content=\"{}\"/>\n",
        sp2,
        xml_escape(&options.author)
    ));
    out.push_str(&format!(
        "{}<meta name=\"generator\" content=\"OxiHuman x3d exporter\"/>\n",
        sp2
    ));
    out.push_str(&format!("{}</head>\n", sp1));

    // <Scene>
    out.push_str(&format!("{}<Scene>\n", sp1));

    for (mesh, name) in meshes {
        out.push_str(&build_shape_block(mesh, name, options, options.indent * 2));
    }

    out.push_str(&format!("{}</Scene>\n", sp1));
    out.push_str("</X3D>\n");

    out
}

/// Export a multi-mesh scene to an X3D file.
pub fn export_x3d_scene(
    meshes: &[(&MeshBuffers, &str)],
    path: &Path,
    options: &X3dExportOptions,
) -> anyhow::Result<()> {
    let content = build_x3d_scene(meshes, options);
    std::fs::write(path, &content)
        .with_context(|| format!("Failed to write X3D scene file: {}", path.display()))?;
    Ok(())
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Validate that a string looks like well-formed X3D XML.
///
/// Checks for:
/// - XML declaration
/// - `<X3D` root element with `profile` and `version` attributes
/// - `<Scene>` element
/// - Balanced `<X3D>` open/close tags
pub fn validate_x3d(content: &str) -> Result<(), String> {
    if !content.starts_with("<?xml") {
        return Err("Missing XML declaration".to_string());
    }
    if !content.contains("<X3D") {
        return Err("Missing <X3D> root element".to_string());
    }
    if !content.contains("profile=") {
        return Err("Missing 'profile' attribute on <X3D>".to_string());
    }
    if !content.contains("version=") {
        return Err("Missing 'version' attribute on <X3D>".to_string());
    }
    if !content.contains("<Scene") {
        return Err("Missing <Scene> element".to_string());
    }
    if !content.contains("</Scene>") {
        return Err("Missing </Scene> closing tag".to_string());
    }
    if !content.contains("</X3D>") {
        return Err("Missing </X3D> closing tag".to_string());
    }

    // Check that <X3D appears before </X3D>
    let open_pos = match content.find("<X3D") {
        Some(p) => p,
        None => return Err("Missing <X3D opening tag".to_string()),
    };
    let close_pos = match content.find("</X3D>") {
        Some(p) => p,
        None => return Err("Missing </X3D> closing tag".to_string()),
    };
    if open_pos >= close_pos {
        return Err("<X3D> open tag appears after </X3D> close tag".to_string());
    }

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::MeshBuffers;

    fn make_triangle_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            indices: vec![0, 1, 2],
            colors: None,
            has_suit: true,
        }
    }

    fn make_quad_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            colors: None,
            has_suit: true,
        }
    }

    fn make_empty_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![],
            normals: vec![],
            tangents: vec![],
            uvs: vec![],
            indices: vec![],
            colors: None,
            has_suit: true,
        }
    }

    // ── 1. format_coord_array ────────────────────────────────────────────────

    #[test]
    fn test_format_coord_array_basic() {
        let pts = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let s = format_coord_array(&pts);
        assert!(s.contains("1 2 3"), "Expected '1 2 3', got: {s}");
        assert!(s.contains("4 5 6"), "Expected '4 5 6', got: {s}");
        assert!(s.contains(','), "Missing comma separator");
    }

    #[test]
    fn test_format_coord_array_empty() {
        let s = format_coord_array(&[]);
        assert!(s.is_empty(), "Expected empty string for empty array");
    }

    // ── 2. format_index_array ────────────────────────────────────────────────

    #[test]
    fn test_format_index_array_triangles() {
        let idx = vec![0u32, 1, 2, 3, 4, 5];
        let s = format_index_array(&idx, 3);
        assert_eq!(s, "0 1 2 -1 3 4 5 -1");
    }

    #[test]
    fn test_format_index_array_empty() {
        let s = format_index_array(&[], 3);
        assert!(s.is_empty());
    }

    #[test]
    fn test_format_index_array_zero_stride() {
        let idx = vec![0u32, 1, 2];
        let s = format_index_array(&idx, 0);
        assert!(s.is_empty());
    }

    // ── 3. build_x3d structure ───────────────────────────────────────────────

    #[test]
    fn test_build_x3d_xml_declaration() {
        let mesh = make_triangle_mesh();
        let opts = X3dExportOptions::default();
        let (xml, _) = build_x3d(&mesh, &opts);
        assert!(
            xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"),
            "XML declaration missing or wrong"
        );
    }

    #[test]
    fn test_build_x3d_contains_required_elements() {
        let mesh = make_triangle_mesh();
        let opts = X3dExportOptions::default();
        let (xml, _) = build_x3d(&mesh, &opts);
        assert!(xml.contains("<X3D"), "Missing <X3D>");
        assert!(xml.contains("</X3D>"), "Missing </X3D>");
        assert!(xml.contains("<Scene"), "Missing <Scene>");
        assert!(xml.contains("</Scene>"), "Missing </Scene>");
        assert!(xml.contains("<Shape"), "Missing <Shape>");
        assert!(xml.contains("<Appearance>"), "Missing <Appearance>");
        assert!(xml.contains("<Material"), "Missing <Material>");
        assert!(xml.contains("<IndexedFaceSet"), "Missing <IndexedFaceSet>");
        assert!(xml.contains("<Coordinate"), "Missing <Coordinate>");
    }

    #[test]
    fn test_build_x3d_stats() {
        let mesh = make_triangle_mesh();
        let opts = X3dExportOptions::default();
        let (xml, stats) = build_x3d(&mesh, &opts);
        assert_eq!(stats.vertex_count, 3);
        assert_eq!(stats.face_count, 1);
        assert!(stats.has_normals);
        assert!(stats.has_uvs);
        assert_eq!(stats.byte_size, xml.len());
    }

    #[test]
    fn test_build_x3d_no_normals_no_uvs() {
        let mesh = make_triangle_mesh();
        let opts = X3dExportOptions {
            include_normals: false,
            include_uvs: false,
            ..Default::default()
        };
        let (xml, stats) = build_x3d(&mesh, &opts);
        assert!(!xml.contains("<Normal"), "Should not emit <Normal>");
        assert!(
            !xml.contains("<TextureCoordinate"),
            "Should not emit <TextureCoordinate>"
        );
        assert!(!stats.has_normals);
        assert!(!stats.has_uvs);
    }

    #[test]
    fn test_build_x3d_solid_true() {
        let mesh = make_triangle_mesh();
        let opts = X3dExportOptions {
            solid: true,
            ..Default::default()
        };
        let (xml, _) = build_x3d(&mesh, &opts);
        assert!(xml.contains("solid=\"true\""), "Expected solid=true");
    }

    #[test]
    fn test_build_x3d_profile_custom() {
        let mesh = make_triangle_mesh();
        let opts = X3dExportOptions {
            profile: "Full".to_string(),
            ..Default::default()
        };
        let (xml, _) = build_x3d(&mesh, &opts);
        assert!(xml.contains("profile=\"Full\""), "Custom profile not found");
    }

    // ── 4. validate_x3d ──────────────────────────────────────────────────────

    #[test]
    fn test_validate_x3d_valid() {
        let mesh = make_triangle_mesh();
        let opts = X3dExportOptions::default();
        let (xml, _) = build_x3d(&mesh, &opts);
        assert!(
            validate_x3d(&xml).is_ok(),
            "Valid XML should pass validation"
        );
    }

    #[test]
    fn test_validate_x3d_missing_declaration() {
        let bad = "<X3D profile=\"Interchange\" version=\"3.3\"><Scene></Scene></X3D>";
        assert!(validate_x3d(bad).is_err());
    }

    #[test]
    fn test_validate_x3d_missing_scene() {
        let bad = "<?xml version=\"1.0\"?><X3D profile=\"X\" version=\"3.3\"></X3D>";
        let result = validate_x3d(bad);
        assert!(result.is_err());
    }

    // ── 5. export_x3d ────────────────────────────────────────────────────────

    #[test]
    fn test_export_x3d_writes_file() {
        let mesh = make_triangle_mesh();
        let opts = X3dExportOptions::default();
        let path = std::path::Path::new("/tmp/oxihuman_x3d_test_single.x3d");
        let stats = export_x3d(&mesh, path, &opts).expect("export_x3d failed");
        assert!(path.exists(), "Output file not created");
        let content = std::fs::read_to_string(path).expect("should succeed");
        assert!(
            validate_x3d(&content).is_ok(),
            "Exported file failed validation"
        );
        assert_eq!(stats.vertex_count, 3);
        assert_eq!(stats.face_count, 1);
    }

    // ── 6. build_x3d_scene ───────────────────────────────────────────────────

    #[test]
    fn test_build_x3d_scene_multiple_meshes() {
        let m1 = make_triangle_mesh();
        let m2 = make_quad_mesh();
        let opts = X3dExportOptions::default();
        let meshes: Vec<(&MeshBuffers, &str)> = vec![(&m1, "Body"), (&m2, "Head")];
        let xml = build_x3d_scene(&meshes, &opts);
        assert!(validate_x3d(&xml).is_ok(), "Scene XML failed validation");
        assert!(xml.contains("DEF=\"Body\""), "Missing Body shape");
        assert!(xml.contains("DEF=\"Head\""), "Missing Head shape");
    }

    // ── 7. export_x3d_scene ──────────────────────────────────────────────────

    #[test]
    fn test_export_x3d_scene_writes_file() {
        let m1 = make_triangle_mesh();
        let m2 = make_quad_mesh();
        let opts = X3dExportOptions::default();
        let meshes: Vec<(&MeshBuffers, &str)> = vec![(&m1, "Body"), (&m2, "Clothes")];
        let path = std::path::Path::new("/tmp/oxihuman_x3d_test_scene.x3d");
        export_x3d_scene(&meshes, path, &opts).expect("export_x3d_scene failed");
        assert!(path.exists(), "Scene output file not created");
        let content = std::fs::read_to_string(path).expect("should succeed");
        assert!(
            validate_x3d(&content).is_ok(),
            "Scene file failed validation"
        );
    }

    // ── 8. empty mesh ────────────────────────────────────────────────────────

    #[test]
    fn test_build_x3d_empty_mesh() {
        let mesh = make_empty_mesh();
        let opts = X3dExportOptions::default();
        let (xml, stats) = build_x3d(&mesh, &opts);
        assert!(
            validate_x3d(&xml).is_ok(),
            "Empty mesh XML failed validation"
        );
        assert_eq!(stats.vertex_count, 0);
        assert_eq!(stats.face_count, 0);
        assert!(!stats.has_normals);
        assert!(!stats.has_uvs);
    }

    // ── 9. custom indent ─────────────────────────────────────────────────────

    #[test]
    fn test_build_x3d_custom_indent() {
        let mesh = make_triangle_mesh();
        let opts = X3dExportOptions {
            indent: 4,
            ..Default::default()
        };
        let (xml, _) = build_x3d(&mesh, &opts);
        // With indent=4, the <head> should be indented with 4 spaces
        assert!(
            xml.contains("    <head>"),
            "Expected 4-space indented <head>"
        );
    }

    // ── 10. author meta ──────────────────────────────────────────────────────

    #[test]
    fn test_build_x3d_author_meta() {
        let mesh = make_triangle_mesh();
        let opts = X3dExportOptions {
            author: "TestAuthor".to_string(),
            ..Default::default()
        };
        let (xml, _) = build_x3d(&mesh, &opts);
        assert!(
            xml.contains("content=\"TestAuthor\""),
            "Author meta missing"
        );
    }

    // ── 11. quad mesh with two triangles ─────────────────────────────────────

    #[test]
    fn test_build_x3d_quad_mesh_two_faces() {
        let mesh = make_quad_mesh();
        let opts = X3dExportOptions::default();
        let (xml, stats) = build_x3d(&mesh, &opts);
        assert_eq!(stats.face_count, 2);
        assert_eq!(stats.vertex_count, 4);
        // coordIndex should contain two -1 terminators
        let terminators = xml.matches("-1").count();
        assert!(
            terminators >= 2,
            "Expected at least 2 -1 terminators, got {terminators}"
        );
    }

    // ── 12. xml_escape in mesh name ──────────────────────────────────────────

    #[test]
    fn test_build_x3d_xml_escape_mesh_name() {
        let mesh = make_triangle_mesh();
        let opts = X3dExportOptions {
            mesh_name: "Mesh<1>&\"2\"".to_string(),
            ..Default::default()
        };
        let (xml, _) = build_x3d(&mesh, &opts);
        // The raw '<' should NOT appear in the DEF attribute value
        assert!(
            xml.contains("DEF=\"Mesh&lt;1&gt;&amp;&quot;2&quot;\""),
            "XML escape not applied to mesh name"
        );
    }

    // ── 13. scene with empty meshes list ─────────────────────────────────────

    #[test]
    fn test_build_x3d_scene_empty() {
        let opts = X3dExportOptions::default();
        let meshes: Vec<(&MeshBuffers, &str)> = vec![];
        let xml = build_x3d_scene(&meshes, &opts);
        assert!(
            validate_x3d(&xml).is_ok(),
            "Empty scene XML failed validation"
        );
        assert!(
            !xml.contains("<Shape"),
            "Empty scene should have no Shape nodes"
        );
    }
}
