// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use oxihuman_mesh::MeshBuffers;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

pub struct MeshReportData {
    pub name: String,
    pub vertex_count: usize,
    pub face_count: usize,
    pub has_normals: bool,
    pub has_uvs: bool,
    pub has_colors: bool,
    pub bounding_box_min: [f32; 3],
    pub bounding_box_max: [f32; 3],
    pub file_size_bytes: Option<u64>,
    pub format: String,
}

pub struct PipelineReportData {
    pub title: String,
    pub timestamp: String,
    pub version: String,
    pub meshes: Vec<MeshReportData>,
    pub parameters: HashMap<String, f32>,
    pub export_paths: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

impl PipelineReportData {
    pub fn new(title: impl Into<String>) -> Self {
        PipelineReportData {
            title: title.into(),
            timestamp: String::new(),
            version: String::from("0.1.0"),
            meshes: Vec::new(),
            parameters: HashMap::new(),
            export_paths: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            duration_ms: 0,
        }
    }

    pub fn add_mesh(&mut self, mesh: MeshReportData) {
        self.meshes.push(mesh);
    }

    pub fn add_param(&mut self, key: impl Into<String>, value: f32) {
        self.parameters.insert(key.into(), value);
    }

    pub fn add_export_path(&mut self, path: impl Into<String>) {
        self.export_paths.push(path.into());
    }

    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }

    pub fn total_vertices(&self) -> usize {
        self.meshes.iter().map(|m| m.vertex_count).sum()
    }

    pub fn total_faces(&self) -> usize {
        self.meshes.iter().map(|m| m.face_count).sum()
    }
}

// ---------------------------------------------------------------------------
// HTML helpers
// ---------------------------------------------------------------------------

/// Escape HTML special characters.
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

fn bool_badge(v: bool) -> &'static str {
    if v {
        "Yes"
    } else {
        "No"
    }
}

fn fmt_opt_u64(v: Option<u64>) -> String {
    match v {
        Some(n) => format!("{n}"),
        None => String::from("—"),
    }
}

fn fmt_f32x3(v: [f32; 3]) -> String {
    format!("[{:.3}, {:.3}, {:.3}]", v[0], v[1], v[2])
}

// ---------------------------------------------------------------------------
// Inline CSS
// ---------------------------------------------------------------------------

const INLINE_CSS: &str = r#"
  body { font-family: 'Segoe UI', Arial, sans-serif; margin: 24px; background: #f5f5f5; color: #222; }
  h1   { color: #2c3e50; border-bottom: 2px solid #3498db; padding-bottom: 8px; }
  h2   { color: #34495e; margin-top: 32px; }
  table { border-collapse: collapse; width: 100%; margin-top: 12px; background: #fff; }
  th { background: #3498db; color: #fff; padding: 8px 12px; text-align: left; }
  td { padding: 7px 12px; border-bottom: 1px solid #dde; }
  tr:nth-child(even) td { background: #f0f4ff; }
  .summary-grid { display: flex; gap: 16px; flex-wrap: wrap; margin-top: 12px; }
  .summary-card { background: #fff; border: 1px solid #cce; border-radius: 6px;
                  padding: 14px 20px; min-width: 160px; }
  .summary-card .label { font-size: 0.8em; color: #666; }
  .summary-card .value { font-size: 1.6em; font-weight: bold; color: #2c3e50; }
  .warn-list li { background: #fffbe6; border-left: 4px solid #f1c40f;
                  padding: 6px 10px; margin: 4px 0; list-style: none; }
  .err-list  li { background: #fdecea; border-left: 4px solid #e74c3c;
                  padding: 6px 10px; margin: 4px 0; list-style: none; }
  .path-list li { background: #eafaf1; border-left: 4px solid #2ecc71;
                  padding: 6px 10px; margin: 4px 0; list-style: none; font-family: monospace; }
  footer { margin-top: 40px; font-size: 0.8em; color: #999; border-top: 1px solid #ddd; padding-top: 8px; }
"#;

// ---------------------------------------------------------------------------
// Core report generation
// ---------------------------------------------------------------------------

/// Generate a full self-contained HTML5 report string.
pub fn generate_html_report(data: &PipelineReportData) -> String {
    let mut html = String::with_capacity(8192);

    // DOCTYPE + head
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str(&format!("<title>{}</title>\n", html_escape(&data.title)));
    html.push_str("<style>");
    html.push_str(INLINE_CSS);
    html.push_str("</style>\n</head>\n<body>\n");

    // Header
    html.push_str(&format!("<h1>{}</h1>\n", html_escape(&data.title)));
    html.push_str(&format!(
        "<p><strong>Timestamp:</strong> {}&nbsp;&nbsp;<strong>Version:</strong> {}&nbsp;&nbsp;<strong>Duration:</strong> {} ms</p>\n",
        html_escape(&data.timestamp),
        html_escape(&data.version),
        data.duration_ms
    ));

    // Summary cards
    html.push_str("<h2>Summary</h2>\n<div class=\"summary-grid\">\n");
    html.push_str(&summary_card("Meshes", &data.meshes.len().to_string()));
    html.push_str(&summary_card(
        "Total Vertices",
        &data.total_vertices().to_string(),
    ));
    html.push_str(&summary_card(
        "Total Faces",
        &data.total_faces().to_string(),
    ));
    html.push_str(&summary_card(
        "Parameters",
        &data.parameters.len().to_string(),
    ));
    html.push_str(&summary_card(
        "Exports",
        &data.export_paths.len().to_string(),
    ));
    html.push_str(&summary_card("Warnings", &data.warnings.len().to_string()));
    html.push_str(&summary_card("Errors", &data.errors.len().to_string()));
    html.push_str("</div>\n");

    // Export paths
    if !data.export_paths.is_empty() {
        html.push_str("<h2>Export Paths</h2>\n<ul class=\"path-list\">\n");
        for p in &data.export_paths {
            html.push_str(&format!("<li>{}</li>\n", html_escape(p)));
        }
        html.push_str("</ul>\n");
    }

    // Mesh table
    html.push_str("<h2>Meshes</h2>\n");
    if data.meshes.is_empty() {
        html.push_str("<p><em>No meshes.</em></p>\n");
    } else {
        html.push_str("<table>\n<thead><tr>");
        for col in &[
            "Name",
            "Format",
            "Vertices",
            "Faces",
            "Normals",
            "UVs",
            "Colors",
            "BBox Min",
            "BBox Max",
            "File Size",
        ] {
            html.push_str(&format!("<th>{col}</th>"));
        }
        html.push_str("</tr></thead>\n<tbody>\n");
        for m in &data.meshes {
            html.push_str("<tr>");
            html.push_str(&td(&html_escape(&m.name)));
            html.push_str(&td(&html_escape(&m.format)));
            html.push_str(&td(&m.vertex_count.to_string()));
            html.push_str(&td(&m.face_count.to_string()));
            html.push_str(&td(bool_badge(m.has_normals)));
            html.push_str(&td(bool_badge(m.has_uvs)));
            html.push_str(&td(bool_badge(m.has_colors)));
            html.push_str(&td(&fmt_f32x3(m.bounding_box_min)));
            html.push_str(&td(&fmt_f32x3(m.bounding_box_max)));
            html.push_str(&td(&fmt_opt_u64(m.file_size_bytes)));
            html.push_str("</tr>\n");
        }
        html.push_str("</tbody>\n</table>\n");
    }

    // Parameters table
    html.push_str("<h2>Parameters</h2>\n");
    if data.parameters.is_empty() {
        html.push_str("<p><em>No parameters.</em></p>\n");
    } else {
        html.push_str("<table>\n<thead><tr><th>Key</th><th>Value</th></tr></thead>\n<tbody>\n");
        let mut sorted_keys: Vec<&String> = data.parameters.keys().collect();
        sorted_keys.sort();
        for k in sorted_keys {
            let v = data.parameters[k];
            html.push_str(&format!(
                "<tr>{}{}</tr>\n",
                td(&html_escape(k)),
                td(&format!("{v:.6}"))
            ));
        }
        html.push_str("</tbody>\n</table>\n");
    }

    // Warnings
    if !data.warnings.is_empty() {
        html.push_str("<h2>Warnings</h2>\n<ul class=\"warn-list\">\n");
        for w in &data.warnings {
            html.push_str(&format!("<li>{}</li>\n", html_escape(w)));
        }
        html.push_str("</ul>\n");
    }

    // Errors
    if !data.errors.is_empty() {
        html.push_str("<h2>Errors</h2>\n<ul class=\"err-list\">\n");
        for e in &data.errors {
            html.push_str(&format!("<li>{}</li>\n", html_escape(e)));
        }
        html.push_str("</ul>\n");
    }

    // Footer
    html.push_str("<footer>Generated by OxiHuman Export Pipeline</footer>\n");
    html.push_str("</body>\n</html>\n");

    html
}

fn summary_card(label: &str, value: &str) -> String {
    format!(
        "<div class=\"summary-card\"><div class=\"label\">{label}</div><div class=\"value\">{value}</div></div>\n"
    )
}

fn td(content: &str) -> String {
    format!("<td>{content}</td>")
}

// ---------------------------------------------------------------------------
// File export
// ---------------------------------------------------------------------------

/// Export HTML report to a file.
pub fn export_html_report(data: &PipelineReportData, path: &std::path::Path) -> anyhow::Result<()> {
    let html = generate_html_report(data);
    std::fs::write(path, html)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Mesh summary table
// ---------------------------------------------------------------------------

/// Generate a minimal mesh summary HTML table for a single mesh.
pub fn mesh_summary_html(mesh: &MeshReportData) -> String {
    let mut html = String::with_capacity(512);
    html.push_str("<table>\n<thead><tr><th>Property</th><th>Value</th></tr></thead>\n<tbody>\n");
    let rows: &[(&str, String)] = &[
        ("Name", html_escape(&mesh.name)),
        ("Format", html_escape(&mesh.format)),
        ("Vertices", mesh.vertex_count.to_string()),
        ("Faces", mesh.face_count.to_string()),
        ("Normals", bool_badge(mesh.has_normals).to_string()),
        ("UVs", bool_badge(mesh.has_uvs).to_string()),
        ("Colors", bool_badge(mesh.has_colors).to_string()),
        ("BBox Min", fmt_f32x3(mesh.bounding_box_min)),
        ("BBox Max", fmt_f32x3(mesh.bounding_box_max)),
        ("File Size", fmt_opt_u64(mesh.file_size_bytes)),
    ];
    for (k, v) in rows {
        html.push_str(&format!(
            "<tr><td><strong>{k}</strong></td><td>{v}</td></tr>\n"
        ));
    }
    html.push_str("</tbody>\n</table>\n");
    html
}

// ---------------------------------------------------------------------------
// Auto-populate MeshReportData from MeshBuffers
// ---------------------------------------------------------------------------

/// Build a `MeshReportData` by reading the fields of a `MeshBuffers`.
pub fn mesh_report_from_buffers(mesh: &MeshBuffers, name: &str, format: &str) -> MeshReportData {
    let vertex_count = mesh.positions.len();
    let face_count = mesh.indices.len() / 3;
    let has_normals = !mesh.normals.is_empty();
    let has_uvs = !mesh.uvs.is_empty();
    let has_colors = mesh.colors.is_some();

    // Bounding box
    let mut bb_min = [f32::INFINITY; 3];
    let mut bb_max = [f32::NEG_INFINITY; 3];
    for pos in &mesh.positions {
        for i in 0..3 {
            if pos[i] < bb_min[i] {
                bb_min[i] = pos[i];
            }
            if pos[i] > bb_max[i] {
                bb_max[i] = pos[i];
            }
        }
    }
    // Handle empty mesh
    if mesh.positions.is_empty() {
        bb_min = [0.0; 3];
        bb_max = [0.0; 3];
    }

    MeshReportData {
        name: name.to_string(),
        vertex_count,
        face_count,
        has_normals,
        has_uvs,
        has_colors,
        bounding_box_min: bb_min,
        bounding_box_max: bb_max,
        file_size_bytes: None,
        format: format.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn simple_mesh_buffers() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    fn simple_mesh_report_data() -> MeshReportData {
        MeshReportData {
            name: String::from("body"),
            vertex_count: 100,
            face_count: 80,
            has_normals: true,
            has_uvs: true,
            has_colors: false,
            bounding_box_min: [-1.0, -1.0, -1.0],
            bounding_box_max: [1.0, 1.0, 1.0],
            file_size_bytes: Some(4096),
            format: String::from("GLB"),
        }
    }

    fn simple_pipeline() -> PipelineReportData {
        let mut p = PipelineReportData::new("Test Pipeline");
        p.timestamp = String::from("2026-02-22T00:00:00Z");
        p.version = String::from("1.2.3");
        p.duration_ms = 42;
        p.add_mesh(simple_mesh_report_data());
        p.add_param("height", 1.75);
        p.add_param("weight", 70.0);
        p.add_export_path("/tmp/output/body.glb");
        p.add_warning("normals may be inverted");
        p.add_error("texture missing");
        p
    }

    // --- html_escape tests ---

    #[test]
    fn test_html_escape_basic() {
        assert_eq!(html_escape("hello"), "hello");
        assert_eq!(html_escape(""), "");
    }

    #[test]
    fn test_html_escape_special_chars() {
        assert_eq!(html_escape("&"), "&amp;");
        assert_eq!(html_escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(html_escape("say \"hi\""), "say &quot;hi&quot;");
        assert_eq!(
            html_escape("<a href=\"foo\">bar & baz</a>"),
            "&lt;a href=&quot;foo&quot;&gt;bar &amp; baz&lt;/a&gt;"
        );
    }

    // --- PipelineReportData construction ---

    #[test]
    fn test_pipeline_report_new() {
        let p = PipelineReportData::new("My Report");
        assert_eq!(p.title, "My Report");
        assert!(p.meshes.is_empty());
        assert!(p.parameters.is_empty());
        assert!(p.export_paths.is_empty());
        assert!(p.warnings.is_empty());
        assert!(p.errors.is_empty());
        assert_eq!(p.duration_ms, 0);
    }

    #[test]
    fn test_pipeline_report_add_mesh() {
        let mut p = PipelineReportData::new("R");
        assert_eq!(p.meshes.len(), 0);
        p.add_mesh(simple_mesh_report_data());
        assert_eq!(p.meshes.len(), 1);
        assert_eq!(p.meshes[0].name, "body");
    }

    #[test]
    fn test_pipeline_report_totals() {
        let mut p = PipelineReportData::new("R");
        p.add_mesh(MeshReportData {
            vertex_count: 300,
            face_count: 200,
            ..simple_mesh_report_data()
        });
        p.add_mesh(MeshReportData {
            name: String::from("head"),
            vertex_count: 150,
            face_count: 100,
            ..simple_mesh_report_data()
        });
        assert_eq!(p.total_vertices(), 450);
        assert_eq!(p.total_faces(), 300);
    }

    // --- generate_html_report content checks ---

    #[test]
    fn test_generate_html_has_doctype() {
        let p = simple_pipeline();
        let html = generate_html_report(&p);
        assert!(html.starts_with("<!DOCTYPE html>"), "Missing DOCTYPE");
    }

    #[test]
    fn test_generate_html_has_title() {
        let p = simple_pipeline();
        let html = generate_html_report(&p);
        assert!(html.contains("<title>Test Pipeline</title>"));
        assert!(html.contains("<h1>Test Pipeline</h1>"));
    }

    #[test]
    fn test_generate_html_has_mesh_table() {
        let p = simple_pipeline();
        let html = generate_html_report(&p);
        assert!(html.contains("<h2>Meshes</h2>"));
        assert!(html.contains("body"));
        assert!(html.contains("GLB"));
        assert!(html.contains("100")); // vertex count
        assert!(html.contains("80")); // face count
    }

    #[test]
    fn test_generate_html_has_params() {
        let p = simple_pipeline();
        let html = generate_html_report(&p);
        assert!(html.contains("<h2>Parameters</h2>"));
        assert!(html.contains("height"));
        assert!(html.contains("weight"));
    }

    #[test]
    fn test_generate_html_warnings() {
        let p = simple_pipeline();
        let html = generate_html_report(&p);
        assert!(html.contains("<h2>Warnings</h2>"));
        assert!(html.contains("normals may be inverted"));
        assert!(html.contains("<h2>Errors</h2>"));
        assert!(html.contains("texture missing"));
    }

    // --- mesh_summary_html ---

    #[test]
    fn test_mesh_summary_html() {
        let m = simple_mesh_report_data();
        let html = mesh_summary_html(&m);
        assert!(html.contains("<table>"));
        assert!(html.contains("body"));
        assert!(html.contains("GLB"));
        assert!(html.contains("100"));
        assert!(html.contains("80"));
        assert!(html.contains("Yes")); // has_normals / has_uvs
    }

    // --- export_html_report writes file ---

    #[test]
    fn test_export_html_report() {
        let p = simple_pipeline();
        let path = std::path::Path::new("/tmp/oxihuman_test_report.html");
        export_html_report(&p, path).expect("export failed");
        let contents = std::fs::read_to_string(path).expect("read failed");
        assert!(contents.contains("<!DOCTYPE html>"));
        assert!(contents.contains("Test Pipeline"));
    }

    // --- mesh_report_from_buffers ---

    #[test]
    fn test_mesh_report_from_buffers() {
        let mb = simple_mesh_buffers();
        let rd = mesh_report_from_buffers(&mb, "tri", "OBJ");
        assert_eq!(rd.name, "tri");
        assert_eq!(rd.format, "OBJ");
        assert_eq!(rd.vertex_count, 3);
        assert_eq!(rd.face_count, 1);
        assert!(rd.has_normals);
        assert!(rd.has_uvs);
        assert!(!rd.has_colors);
        // bounding box: x in [-1, 1], y in [0, 1], z in [0, 0]
        assert!((rd.bounding_box_min[0] - (-1.0)).abs() < 1e-6);
        assert!((rd.bounding_box_max[0] - 1.0).abs() < 1e-6);
        assert!((rd.bounding_box_min[1] - 0.0).abs() < 1e-6);
        assert!((rd.bounding_box_max[1] - 1.0).abs() < 1e-6);
    }
}
