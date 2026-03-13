// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! COLLADA (`.dae`) 3D format export — ISO/IEC 17506.
//!
//! Writes COLLADA 1.4.1 XML directly as strings; no external XML crate required.

#![allow(dead_code)]

use std::path::Path;

use anyhow::Context;
use oxihuman_mesh::MeshBuffers;

// ── Options ───────────────────────────────────────────────────────────────────

/// Options controlling COLLADA export output.
pub struct ColladaExportOptions {
    /// Name used for the asset and mesh geometry. Default: `"OxiHumanMesh"`.
    pub asset_name: String,
    /// Author metadata string. Default: `"OxiHuman"`.
    pub author: String,
    /// Unit name string. Default: `"meter"`.
    pub unit_name: String,
    /// Meters-per-unit scale factor. Default: `1.0`.
    pub unit_meter: f32,
    /// Up-axis string, e.g. `"Y_UP"` or `"Z_UP"`. Default: `"Y_UP"`.
    pub up_axis: String,
    /// Emit normals source and input. Default: `true`.
    pub include_normals: bool,
    /// Emit UV-texcoord source and input. Default: `true`.
    pub include_uvs: bool,
    /// Emit `<double_sided>` extra element. Default: `false`.
    pub double_sided: bool,
}

impl Default for ColladaExportOptions {
    fn default() -> Self {
        Self {
            asset_name: "OxiHumanMesh".to_string(),
            author: "OxiHuman".to_string(),
            unit_name: "meter".to_string(),
            unit_meter: 1.0,
            up_axis: "Y_UP".to_string(),
            include_normals: true,
            include_uvs: true,
            double_sided: false,
        }
    }
}

// ── Stats ─────────────────────────────────────────────────────────────────────

/// Statistics returned after a successful COLLADA export.
pub struct ColladaExportStats {
    pub vertex_count: usize,
    pub face_count: usize,
    pub has_normals: bool,
    pub has_uvs: bool,
    pub byte_size: usize,
}

// ── Formatting helpers ────────────────────────────────────────────────────────

/// Format a flat `&[f32]` slice as a space-separated string.
pub fn format_float_array(values: &[f32]) -> String {
    values
        .iter()
        .map(|v| format!("{}", v))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format a flat `&[u32]` slice as a space-separated string.
pub fn format_int_array_collada(values: &[u32]) -> String {
    values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Flatten a `[[f32; 3]]` slice into a `Vec<f32>`.
fn flatten3(data: &[[f32; 3]]) -> Vec<f32> {
    data.iter().flat_map(|v| [v[0], v[1], v[2]]).collect()
}

/// Flatten a `[[f32; 2]]` slice into a `Vec<f32>`.
fn flatten2(data: &[[f32; 2]]) -> Vec<f32> {
    data.iter().flat_map(|v| [v[0], v[1]]).collect()
}

// ── Core geometry builder ─────────────────────────────────────────────────────

/// Build the `<geometry>` XML block for one mesh.
///
/// `geo_id` is used as the `id` attribute, e.g. `"mesh0"`.
fn build_geometry_block(
    mesh: &MeshBuffers,
    geo_id: &str,
    name: &str,
    opts: &ColladaExportOptions,
) -> String {
    let v_count = mesh.positions.len();
    let f_count = mesh.indices.len() / 3;

    let has_normals = opts.include_normals && !mesh.normals.is_empty();
    let has_uvs = opts.include_uvs && !mesh.uvs.is_empty();

    let mut out = String::new();

    out.push_str(&format!(
        "    <geometry id=\"{geo_id}\" name=\"{name}\">\n      <mesh>\n"
    ));

    // ── positions source ──
    let pos_flat = flatten3(&mesh.positions);
    out.push_str(&format!("        <source id=\"{geo_id}-positions\">\n"));
    out.push_str(&format!(
        "          <float_array id=\"{geo_id}-positions-array\" count=\"{count}\">{data}</float_array>\n",
        count = pos_flat.len(),
        data = format_float_array(&pos_flat),
    ));
    out.push_str("          <technique_common>\n");
    out.push_str(&format!(
        "            <accessor source=\"#{geo_id}-positions-array\" count=\"{v}\" stride=\"3\">\n",
        v = v_count
    ));
    out.push_str("              <param name=\"X\" type=\"float\"/>\n");
    out.push_str("              <param name=\"Y\" type=\"float\"/>\n");
    out.push_str("              <param name=\"Z\" type=\"float\"/>\n");
    out.push_str("            </accessor>\n");
    out.push_str("          </technique_common>\n");
    out.push_str("        </source>\n");

    // ── normals source ──
    if has_normals {
        let norm_flat = flatten3(&mesh.normals);
        out.push_str(&format!("        <source id=\"{geo_id}-normals\">\n"));
        out.push_str(&format!(
            "          <float_array id=\"{geo_id}-normals-array\" count=\"{count}\">{data}</float_array>\n",
            count = norm_flat.len(),
            data = format_float_array(&norm_flat),
        ));
        out.push_str("          <technique_common>\n");
        out.push_str(&format!(
            "            <accessor source=\"#{geo_id}-normals-array\" count=\"{v}\" stride=\"3\">\n",
            v = v_count
        ));
        out.push_str("              <param name=\"X\" type=\"float\"/>\n");
        out.push_str("              <param name=\"Y\" type=\"float\"/>\n");
        out.push_str("              <param name=\"Z\" type=\"float\"/>\n");
        out.push_str("            </accessor>\n");
        out.push_str("          </technique_common>\n");
        out.push_str("        </source>\n");
    }

    // ── UVs source ──
    if has_uvs {
        let uv_flat = flatten2(&mesh.uvs);
        out.push_str(&format!("        <source id=\"{geo_id}-uvs\">\n"));
        out.push_str(&format!(
            "          <float_array id=\"{geo_id}-uvs-array\" count=\"{count}\">{data}</float_array>\n",
            count = uv_flat.len(),
            data = format_float_array(&uv_flat),
        ));
        out.push_str("          <technique_common>\n");
        out.push_str(&format!(
            "            <accessor source=\"#{geo_id}-uvs-array\" count=\"{v}\" stride=\"2\">\n",
            v = v_count
        ));
        out.push_str("              <param name=\"S\" type=\"float\"/>\n");
        out.push_str("              <param name=\"T\" type=\"float\"/>\n");
        out.push_str("            </accessor>\n");
        out.push_str("          </technique_common>\n");
        out.push_str("        </source>\n");
    }

    // ── vertices ──
    out.push_str(&format!("        <vertices id=\"{geo_id}-vertices\">\n"));
    out.push_str(&format!(
        "          <input semantic=\"POSITION\" source=\"#{geo_id}-positions\"/>\n"
    ));
    out.push_str("        </vertices>\n");

    // ── triangles ──
    // Determine stride based on which channels are present.
    let stride: usize = 1 + if has_normals { 1 } else { 0 } + if has_uvs { 1 } else { 0 };

    let mut normal_offset = 0usize;
    let mut uv_offset = 0usize;
    let mut current_offset = 1usize; // VERTEX is always offset 0
    if has_normals {
        normal_offset = current_offset;
        current_offset += 1;
    }
    if has_uvs {
        uv_offset = current_offset;
    }

    out.push_str(&format!("        <triangles count=\"{f_count}\">\n"));
    out.push_str(&format!(
        "          <input semantic=\"VERTEX\" source=\"#{geo_id}-vertices\" offset=\"0\"/>\n"
    ));
    if has_normals {
        out.push_str(&format!(
            "          <input semantic=\"NORMAL\" source=\"#{geo_id}-normals\" offset=\"{normal_offset}\"/>\n"
        ));
    }
    if has_uvs {
        out.push_str(&format!(
            "          <input semantic=\"TEXCOORD\" source=\"#{geo_id}-uvs\" offset=\"{uv_offset}\" set=\"0\"/>\n"
        ));
    }

    // Build interleaved index list: for each triangle vertex, emit indices for
    // all active channels. Since we use per-vertex data (not per-face-corner),
    // all channel indices are the same as the vertex index.
    let mut p_parts: Vec<String> = Vec::with_capacity(mesh.indices.len() * stride);
    for &idx in &mesh.indices {
        for _ in 0..stride {
            p_parts.push(idx.to_string());
        }
    }
    out.push_str(&format!("          <p>{}</p>\n", p_parts.join(" ")));
    out.push_str("        </triangles>\n");

    // ── double_sided extra ──
    if opts.double_sided {
        out.push_str("        <extra>\n");
        out.push_str("          <technique profile=\"MAYA\">\n");
        out.push_str("            <double_sided>1</double_sided>\n");
        out.push_str("          </technique>\n");
        out.push_str("        </extra>\n");
    }

    out.push_str("      </mesh>\n    </geometry>\n");
    out
}

// ── COLLADA header / footer ───────────────────────────────────────────────────

fn collada_header(opts: &ColladaExportOptions) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <COLLADA xmlns=\"http://www.collada.org/2005/11/COLLADASchema\" version=\"1.4.1\">\n\
           <asset>\n\
         \x20   <contributor><author>{author}</author></contributor>\n\
         \x20   <created>2026-01-01</created>\n\
         \x20   <modified>2026-01-01</modified>\n\
         \x20   <unit name=\"{unit_name}\" meter=\"{unit_meter}\"/>\n\
         \x20   <up_axis>{up_axis}</up_axis>\n\
           </asset>\n",
        author = opts.author,
        unit_name = opts.unit_name,
        unit_meter = opts.unit_meter,
        up_axis = opts.up_axis,
    )
}

fn collada_footer() -> &'static str {
    "  <scene>\n    <instance_visual_scene url=\"#Scene\"/>\n  </scene>\n</COLLADA>\n"
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Build a COLLADA XML string for a single mesh.
///
/// Returns the XML content and export statistics.
pub fn build_collada(
    mesh: &MeshBuffers,
    options: &ColladaExportOptions,
) -> (String, ColladaExportStats) {
    let geo_id = "mesh0";
    let has_normals = options.include_normals && !mesh.normals.is_empty();
    let has_uvs = options.include_uvs && !mesh.uvs.is_empty();
    let v_count = mesh.positions.len();
    let f_count = mesh.indices.len() / 3;

    let mut out = collada_header(options);

    // library_geometries
    out.push_str("  <library_geometries>\n");
    out.push_str(&build_geometry_block(
        mesh,
        geo_id,
        &options.asset_name,
        options,
    ));
    out.push_str("  </library_geometries>\n");

    // library_visual_scenes
    out.push_str("  <library_visual_scenes>\n");
    out.push_str("    <visual_scene id=\"Scene\" name=\"Scene\">\n");
    out.push_str(&format!(
        "      <node id=\"Mesh\" name=\"{name}\" type=\"NODE\">\n\
         \x20       <instance_geometry url=\"#{geo_id}\"/>\n\
               </node>\n",
        name = options.asset_name,
        geo_id = geo_id,
    ));
    out.push_str("    </visual_scene>\n");
    out.push_str("  </library_visual_scenes>\n");

    out.push_str(collada_footer());

    let byte_size = out.len();
    let stats = ColladaExportStats {
        vertex_count: v_count,
        face_count: f_count,
        has_normals,
        has_uvs,
        byte_size,
    };
    (out, stats)
}

/// Export a single mesh as a `.dae` COLLADA file.
pub fn export_collada(
    mesh: &MeshBuffers,
    path: &Path,
    options: &ColladaExportOptions,
) -> anyhow::Result<ColladaExportStats> {
    let (content, stats) = build_collada(mesh, options);
    std::fs::write(path, &content)
        .with_context(|| format!("Failed to write COLLADA file: {}", path.display()))?;
    Ok(stats)
}

/// Build a COLLADA XML string for multiple meshes (each as a separate geometry).
///
/// `meshes` is a slice of `(mesh_ref, name_str)` pairs.
pub fn build_collada_scene(
    meshes: &[(&MeshBuffers, &str)],
    options: &ColladaExportOptions,
) -> String {
    let mut out = collada_header(options);

    // library_geometries
    out.push_str("  <library_geometries>\n");
    for (i, (mesh, name)) in meshes.iter().enumerate() {
        let geo_id = format!("mesh{i}");
        out.push_str(&build_geometry_block(mesh, &geo_id, name, options));
    }
    out.push_str("  </library_geometries>\n");

    // library_visual_scenes
    out.push_str("  <library_visual_scenes>\n");
    out.push_str("    <visual_scene id=\"Scene\" name=\"Scene\">\n");
    for (i, (_mesh, name)) in meshes.iter().enumerate() {
        let geo_id = format!("mesh{i}");
        let node_id = format!("Node{i}");
        out.push_str(&format!(
            "      <node id=\"{node_id}\" name=\"{name}\" type=\"NODE\">\n\
             \x20       <instance_geometry url=\"#{geo_id}\"/>\n\
                   </node>\n"
        ));
    }
    out.push_str("    </visual_scene>\n");
    out.push_str("  </library_visual_scenes>\n");

    out.push_str(collada_footer());
    out
}

/// Export multiple meshes as a `.dae` COLLADA scene file.
pub fn export_collada_scene(
    meshes: &[(&MeshBuffers, &str)],
    path: &Path,
    options: &ColladaExportOptions,
) -> anyhow::Result<()> {
    let content = build_collada_scene(meshes, options);
    std::fs::write(path, &content)
        .with_context(|| format!("Failed to write COLLADA scene file: {}", path.display()))?;
    Ok(())
}

/// Validate a COLLADA XML string by checking for required key elements.
///
/// Returns `Ok(())` if all required elements are present, or `Err(msg)` on failure.
pub fn validate_collada(content: &str) -> Result<(), String> {
    let required = [
        "<?xml",
        "<COLLADA",
        "http://www.collada.org/2005/11/COLLADASchema",
        "<asset>",
        "<library_geometries>",
        "<library_visual_scenes>",
        "<visual_scene",
        "<scene>",
        "</COLLADA>",
    ];
    for token in &required {
        if !content.contains(token) {
            return Err(format!("Missing required COLLADA element: {token}"));
        }
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn simple_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
            indices: vec![0, 1, 2, 1, 3, 2],
            has_suit: false,
        })
    }

    fn single_tri_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    // Test 1: format_float_array basic
    #[test]
    fn test_format_float_array_basic() {
        let v = vec![1.0f32, 2.0, 3.0];
        let s = format_float_array(&v);
        assert!(s.contains("1"), "should contain 1");
        assert!(s.contains("2"), "should contain 2");
        assert!(s.contains("3"), "should contain 3");
    }

    // Test 2: format_float_array empty
    #[test]
    fn test_format_float_array_empty() {
        let s = format_float_array(&[]);
        assert_eq!(s, "");
    }

    // Test 3: format_int_array_collada basic
    #[test]
    fn test_format_int_array_collada_basic() {
        let v = vec![0u32, 1, 2, 3];
        let s = format_int_array_collada(&v);
        assert_eq!(s, "0 1 2 3");
    }

    // Test 4: format_int_array_collada empty
    #[test]
    fn test_format_int_array_collada_empty() {
        let s = format_int_array_collada(&[]);
        assert_eq!(s, "");
    }

    // Test 5: build_collada returns valid XML declaration
    #[test]
    fn test_build_collada_xml_declaration() {
        let mesh = simple_mesh();
        let opts = ColladaExportOptions::default();
        let (xml, _) = build_collada(&mesh, &opts);
        assert!(xml.starts_with("<?xml version=\"1.0\""));
    }

    // Test 6: build_collada contains COLLADA root element
    #[test]
    fn test_build_collada_root_element() {
        let mesh = simple_mesh();
        let opts = ColladaExportOptions::default();
        let (xml, _) = build_collada(&mesh, &opts);
        assert!(xml.contains("<COLLADA"));
        assert!(xml.contains("</COLLADA>"));
    }

    // Test 7: build_collada stats vertex/face count
    #[test]
    fn test_build_collada_stats_counts() {
        let mesh = simple_mesh();
        let opts = ColladaExportOptions::default();
        let (_, stats) = build_collada(&mesh, &opts);
        assert_eq!(stats.vertex_count, 4);
        assert_eq!(stats.face_count, 2);
    }

    // Test 8: build_collada stats has_normals and has_uvs
    #[test]
    fn test_build_collada_stats_channels() {
        let mesh = simple_mesh();
        let opts = ColladaExportOptions::default();
        let (_, stats) = build_collada(&mesh, &opts);
        assert!(stats.has_normals);
        assert!(stats.has_uvs);
    }

    // Test 9: build_collada byte_size matches string length
    #[test]
    fn test_build_collada_byte_size() {
        let mesh = simple_mesh();
        let opts = ColladaExportOptions::default();
        let (xml, stats) = build_collada(&mesh, &opts);
        assert_eq!(stats.byte_size, xml.len());
    }

    // Test 10: build_collada excludes normals when include_normals=false
    #[test]
    fn test_build_collada_no_normals() {
        let mesh = simple_mesh();
        let opts = ColladaExportOptions {
            include_normals: false,
            ..Default::default()
        };
        let (xml, stats) = build_collada(&mesh, &opts);
        assert!(
            !xml.contains("normals"),
            "should not contain normals source"
        );
        assert!(!stats.has_normals);
    }

    // Test 11: build_collada excludes uvs when include_uvs=false
    #[test]
    fn test_build_collada_no_uvs() {
        let mesh = simple_mesh();
        let opts = ColladaExportOptions {
            include_uvs: false,
            ..Default::default()
        };
        let (xml, stats) = build_collada(&mesh, &opts);
        assert!(
            !xml.contains("TEXCOORD"),
            "should not contain texcoord input"
        );
        assert!(!stats.has_uvs);
    }

    // Test 12: validate_collada passes for valid output
    #[test]
    fn test_validate_collada_valid() {
        let mesh = simple_mesh();
        let opts = ColladaExportOptions::default();
        let (xml, _) = build_collada(&mesh, &opts);
        assert!(validate_collada(&xml).is_ok());
    }

    // Test 13: validate_collada fails for truncated content
    #[test]
    fn test_validate_collada_invalid() {
        let bad = "<?xml version=\"1.0\"?><notcollada/>";
        assert!(validate_collada(bad).is_err());
    }

    // Test 14: export_collada writes file to /tmp/
    #[test]
    fn test_export_collada_writes_file() {
        let mesh = single_tri_mesh();
        let opts = ColladaExportOptions::default();
        let path = std::path::Path::new("/tmp/oxihuman_test_collada.dae");
        let stats = export_collada(&mesh, path, &opts).expect("export_collada failed");
        assert!(path.exists(), "file should exist");
        assert!(stats.byte_size > 0);
        let content = std::fs::read_to_string(path).expect("should succeed");
        assert!(validate_collada(&content).is_ok());
    }

    // Test 15: build_collada_scene with two meshes
    #[test]
    fn test_build_collada_scene_two_meshes() {
        let m1 = simple_mesh();
        let m2 = single_tri_mesh();
        let opts = ColladaExportOptions::default();
        let meshes: Vec<(&MeshBuffers, &str)> = vec![(&m1, "Mesh1"), (&m2, "Mesh2")];
        let xml = build_collada_scene(&meshes, &opts);
        assert!(xml.contains("id=\"mesh0\""), "should have mesh0 geometry");
        assert!(xml.contains("id=\"mesh1\""), "should have mesh1 geometry");
        assert!(validate_collada(&xml).is_ok());
    }

    // Test 16: export_collada_scene writes file to /tmp/
    #[test]
    fn test_export_collada_scene_writes_file() {
        let m1 = simple_mesh();
        let m2 = single_tri_mesh();
        let opts = ColladaExportOptions::default();
        let meshes: Vec<(&MeshBuffers, &str)> = vec![(&m1, "Body"), (&m2, "Head")];
        let path = std::path::Path::new("/tmp/oxihuman_test_collada_scene.dae");
        export_collada_scene(&meshes, path, &opts).expect("export_collada_scene failed");
        assert!(path.exists());
        let content = std::fs::read_to_string(path).expect("should succeed");
        assert!(validate_collada(&content).is_ok());
    }

    // Test 17: double_sided extra element appears when enabled
    #[test]
    fn test_build_collada_double_sided() {
        let mesh = single_tri_mesh();
        let opts = ColladaExportOptions {
            double_sided: true,
            ..Default::default()
        };
        let (xml, _) = build_collada(&mesh, &opts);
        assert!(xml.contains("<double_sided>1</double_sided>"));
    }

    // Test 18: up_axis is reflected in output
    #[test]
    fn test_build_collada_up_axis_z() {
        let mesh = single_tri_mesh();
        let opts = ColladaExportOptions {
            up_axis: "Z_UP".to_string(),
            ..Default::default()
        };
        let (xml, _) = build_collada(&mesh, &opts);
        assert!(xml.contains("<up_axis>Z_UP</up_axis>"));
    }

    // Test 19: asset_name reflected in geometry name attribute
    #[test]
    fn test_build_collada_asset_name_in_geometry() {
        let mesh = single_tri_mesh();
        let opts = ColladaExportOptions {
            asset_name: "TestBody".to_string(),
            ..Default::default()
        };
        let (xml, _) = build_collada(&mesh, &opts);
        assert!(xml.contains("name=\"TestBody\""));
    }

    // Test 20: build_collada_scene with empty slice
    #[test]
    fn test_build_collada_scene_empty() {
        let opts = ColladaExportOptions::default();
        let meshes: Vec<(&MeshBuffers, &str)> = vec![];
        let xml = build_collada_scene(&meshes, &opts);
        assert!(xml.contains("<library_geometries>"));
        assert!(xml.contains("</library_geometries>"));
    }
}
