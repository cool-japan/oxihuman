// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SVG silhouette and contour export.

#[allow(dead_code)]
pub struct SvgConfig {
    pub width: u32,
    pub height: u32,
    pub stroke_color: String,
    pub fill_color: String,
    pub stroke_width: f32,
    pub background: String,
}

#[allow(dead_code)]
pub struct SvgPath {
    pub d: String,
    pub stroke: String,
    pub fill: String,
    pub stroke_width: f32,
}

#[allow(dead_code)]
pub struct SvgDocument {
    pub width: u32,
    pub height: u32,
    pub paths: Vec<SvgPath>,
    pub viewbox: [f32; 4],
}

#[allow(dead_code)]
pub fn default_svg_config() -> SvgConfig {
    SvgConfig {
        width: 800,
        height: 600,
        stroke_color: "#000000".to_string(),
        fill_color: "none".to_string(),
        stroke_width: 1.0,
        background: "#ffffff".to_string(),
    }
}

#[allow(dead_code)]
pub fn new_svg_document(width: u32, height: u32) -> SvgDocument {
    SvgDocument {
        width,
        height,
        paths: Vec::new(),
        viewbox: [0.0, 0.0, width as f32, height as f32],
    }
}

#[allow(dead_code)]
pub fn add_path(doc: &mut SvgDocument, path: SvgPath) {
    doc.paths.push(path);
}

#[allow(dead_code)]
pub fn positions_to_svg_path(positions_2d: &[[f32; 2]], closed: bool) -> String {
    if positions_2d.is_empty() {
        return String::new();
    }
    let mut d = format!("M {} {}", positions_2d[0][0], positions_2d[0][1]);
    for p in positions_2d.iter().skip(1) {
        d.push_str(&format!(" L {} {}", p[0], p[1]));
    }
    if closed {
        d.push_str(" Z");
    }
    d
}

/// Orthographic projection of 3D positions along a view direction.
#[allow(dead_code)]
pub fn project_to_2d(positions: &[[f32; 3]], view_dir: [f32; 3]) -> Vec<[f32; 2]> {
    // Build an orthonormal basis perpendicular to view_dir
    let vd = normalize3(view_dir);

    // Pick a stable up vector
    let up_candidate = if vd[1].abs() < 0.9 {
        [0.0_f32, 1.0, 0.0]
    } else {
        [1.0_f32, 0.0, 0.0]
    };
    let right = cross3(up_candidate, vd);
    let right = normalize3(right);
    let up = cross3(vd, right);

    positions
        .iter()
        .map(|p| [dot3(*p, right), dot3(*p, up)])
        .collect()
}

/// Find silhouette edges: edges shared by one front-facing and one back-facing triangle.
#[allow(dead_code)]
pub fn find_silhouette_edges(
    positions: &[[f32; 3]],
    indices: &[u32],
    view_dir: [f32; 3],
) -> Vec<[u32; 2]> {
    use std::collections::HashMap;

    let tri_count = indices.len() / 3;
    // For each edge (sorted pair), store whether front/back faces reference it
    let mut edge_front: HashMap<(u32, u32), bool> = HashMap::new();
    let mut edge_back: HashMap<(u32, u32), bool> = HashMap::new();

    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;

        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }

        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];

        let e01 = sub3(p1, p0);
        let e02 = sub3(p2, p0);
        let normal = cross3(e01, e02);
        let facing = dot3(normal, view_dir);

        let tri_edges = [
            edge_key(indices[t * 3], indices[t * 3 + 1]),
            edge_key(indices[t * 3 + 1], indices[t * 3 + 2]),
            edge_key(indices[t * 3 + 2], indices[t * 3]),
        ];

        for ek in &tri_edges {
            if facing >= 0.0 {
                edge_front.insert(*ek, true);
            } else {
                edge_back.insert(*ek, true);
            }
        }
    }

    // Silhouette: edges in both front and back
    edge_front
        .keys()
        .filter(|k| edge_back.contains_key(k))
        .map(|(a, b)| [*a, *b])
        .collect()
}

fn edge_key(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

#[allow(dead_code)]
pub fn mesh_silhouette_svg(
    positions: &[[f32; 3]],
    indices: &[u32],
    view_dir: [f32; 3],
    cfg: &SvgConfig,
) -> SvgDocument {
    let mut doc = new_svg_document(cfg.width, cfg.height);
    let pos2d = project_to_2d(positions, view_dir);
    let sil_edges = find_silhouette_edges(positions, indices, view_dir);
    let mut new_paths = edges_to_svg_paths(&sil_edges, &pos2d, cfg);
    for p in new_paths.drain(..) {
        doc.paths.push(p);
    }
    doc
}

#[allow(dead_code)]
pub fn edges_to_svg_paths(
    edges: &[[u32; 2]],
    positions_2d: &[[f32; 2]],
    cfg: &SvgConfig,
) -> Vec<SvgPath> {
    edges
        .iter()
        .filter_map(|e| {
            let i0 = e[0] as usize;
            let i1 = e[1] as usize;
            if i0 < positions_2d.len() && i1 < positions_2d.len() {
                let p0 = positions_2d[i0];
                let p1 = positions_2d[i1];
                Some(SvgPath {
                    d: format!("M {} {} L {} {}", p0[0], p0[1], p1[0], p1[1]),
                    stroke: cfg.stroke_color.clone(),
                    fill: cfg.fill_color.clone(),
                    stroke_width: cfg.stroke_width,
                })
            } else {
                None
            }
        })
        .collect()
}

#[allow(dead_code)]
pub fn svg_document_to_string(doc: &SvgDocument) -> String {
    let vb = doc.viewbox;
    let mut s = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="{} {} {} {}">"#,
        doc.width, doc.height, vb[0], vb[1], vb[2], vb[3]
    );
    for path in &doc.paths {
        s.push_str(&format!(
            r#"<path d="{}" stroke="{}" fill="{}" stroke-width="{}"/>"#,
            path.d, path.stroke, path.fill, path.stroke_width
        ));
    }
    s.push_str("</svg>");
    s
}

#[allow(dead_code)]
pub fn svg_bounds(doc: &SvgDocument) -> [f32; 4] {
    [
        doc.viewbox[0],
        doc.viewbox[1],
        doc.viewbox[2],
        doc.viewbox[3],
    ]
}

#[allow(dead_code)]
pub fn scale_svg(doc: &mut SvgDocument, factor: f32) {
    doc.viewbox[2] *= factor;
    doc.viewbox[3] *= factor;
    doc.width = (doc.width as f32 * factor) as u32;
    doc.height = (doc.height as f32 * factor) as u32;
}

#[allow(dead_code)]
pub fn path_count(doc: &SvgDocument) -> usize {
    doc.paths.len()
}

// ── Math helpers ─────────────────────────────────────────────────────────────

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

// ── New high-level API ────────────────────────────────────────────────────────

/// Configuration for the high-level SVG mesh export.
#[allow(dead_code)]
pub struct SvgExportConfig {
    pub width: u32,
    pub height: u32,
    pub stroke_color: String,
    pub fill_color: String,
    pub stroke_width: f32,
    pub background_color: String,
}

/// Axis-aligned viewbox for an SVG document.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SvgViewBox {
    pub min_x: f32,
    pub min_y: f32,
    pub width: f32,
    pub height: f32,
}

/// Return a default `SvgExportConfig`.
#[allow(dead_code)]
pub fn default_svg_export_config() -> SvgExportConfig {
    SvgExportConfig {
        width: 800,
        height: 600,
        stroke_color: "#000000".to_string(),
        fill_color: "none".to_string(),
        stroke_width: 1.0,
        background_color: "#ffffff".to_string(),
    }
}

/// Create a new `SvgDocument` from an `SvgExportConfig`.
#[allow(dead_code)]
pub fn new_svg_document_from_config(cfg: &SvgExportConfig) -> SvgDocument {
    SvgDocument {
        width: cfg.width,
        height: cfg.height,
        paths: Vec::new(),
        viewbox: [0.0, 0.0, cfg.width as f32, cfg.height as f32],
    }
}

/// Project a 3-D mesh to 2-D (along the -Z axis) and return an `SvgDocument`
/// containing one path per triangle edge.
#[allow(dead_code)]
pub fn project_mesh_to_svg(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    cfg: &SvgExportConfig,
) -> SvgDocument {
    let mut doc = new_svg_document_from_config(cfg);
    // Project along -Z (orthographic)
    let proj: Vec<[f32; 2]> = verts.iter().map(|v| [v[0], v[1]]).collect();

    for face in faces {
        let i0 = face[0] as usize;
        let i1 = face[1] as usize;
        let i2 = face[2] as usize;
        if i0 >= proj.len() || i1 >= proj.len() || i2 >= proj.len() {
            continue;
        }
        let p0 = proj[i0];
        let p1 = proj[i1];
        let p2 = proj[i2];
        let d = format!(
            "M {} {} L {} {} L {} {} Z",
            p0[0], p0[1], p1[0], p1[1], p2[0], p2[1]
        );
        doc.paths.push(SvgPath {
            d,
            stroke: cfg.stroke_color.clone(),
            fill: cfg.fill_color.clone(),
            stroke_width: cfg.stroke_width,
        });
    }
    doc
}

/// Add a path to an SVG document (high-level API alias).
#[allow(dead_code)]
pub fn svg_add_path(doc: &mut SvgDocument, path: SvgPath) {
    doc.paths.push(path);
}

/// Render an `SvgDocument` to an SVG XML string.
#[allow(dead_code)]
pub fn svg_to_string(doc: &SvgDocument) -> String {
    svg_document_to_string(doc)
}

/// Write an `SvgDocument` to a file on disk.
#[allow(dead_code)]
pub fn svg_write_to_file(doc: &SvgDocument, path: &str) -> Result<(), String> {
    let content = svg_to_string(doc);
    std::fs::write(path, content).map_err(|e| e.to_string())
}

/// Build an `SvgPath` from a closed 2-D contour.
#[allow(dead_code)]
pub fn svg_path_from_contour(points: &[[f32; 2]]) -> SvgPath {
    SvgPath {
        d: positions_to_svg_path(points, true),
        stroke: "#000000".to_string(),
        fill: "none".to_string(),
        stroke_width: 1.0,
    }
}

/// Compute the bounding `SvgViewBox` of all paths in a document.
#[allow(dead_code)]
pub fn svg_document_bounds(doc: &SvgDocument) -> SvgViewBox {
    SvgViewBox {
        min_x: doc.viewbox[0],
        min_y: doc.viewbox[1],
        width: doc.viewbox[2],
        height: doc.viewbox[3],
    }
}

/// Set the stroke color for all subsequent paths (updates the document default).
#[allow(dead_code)]
pub fn svg_set_stroke_color(doc: &mut SvgDocument, color: &str) {
    for path in &mut doc.paths {
        path.stroke = color.to_string();
    }
}

/// Set the fill color for all existing paths in a document.
#[allow(dead_code)]
pub fn svg_set_fill_color(doc: &mut SvgDocument, color: &str) {
    for path in &mut doc.paths {
        path.fill = color.to_string();
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_positions() -> Vec<[f32; 3]> {
        vec![
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ]
    }

    fn cube_indices() -> Vec<u32> {
        vec![
            0, 1, 2, 0, 2, 3, // -z face
            4, 6, 5, 4, 7, 6, // +z face
            0, 4, 5, 0, 5, 1, // -y face
            2, 6, 7, 2, 7, 3, // +y face
            0, 3, 7, 0, 7, 4, // -x face
            1, 5, 6, 1, 6, 2, // +x face
        ]
    }

    #[test]
    fn test_new_document() {
        let doc = new_svg_document(800, 600);
        assert_eq!(doc.width, 800);
        assert_eq!(doc.height, 600);
        assert!(doc.paths.is_empty());
    }

    #[test]
    fn test_add_path() {
        let mut doc = new_svg_document(100, 100);
        let p = SvgPath {
            d: "M 0 0 L 10 10".to_string(),
            stroke: "#000".to_string(),
            fill: "none".to_string(),
            stroke_width: 1.0,
        };
        add_path(&mut doc, p);
        assert_eq!(path_count(&doc), 1);
    }

    #[test]
    fn test_positions_to_svg_path_starts_with_m() {
        let pts = [[0.0_f32, 0.0], [10.0, 10.0], [20.0, 0.0]];
        let d = positions_to_svg_path(&pts, false);
        assert!(d.starts_with('M'));
    }

    #[test]
    fn test_positions_to_svg_path_closed() {
        let pts = [[0.0_f32, 0.0], [10.0, 10.0]];
        let d = positions_to_svg_path(&pts, true);
        assert!(d.ends_with('Z'));
    }

    #[test]
    fn test_positions_to_svg_path_empty() {
        let d = positions_to_svg_path(&[], false);
        assert!(d.is_empty());
    }

    #[test]
    fn test_document_to_string_contains_svg_tag() {
        let doc = new_svg_document(400, 300);
        let s = svg_document_to_string(&doc);
        assert!(s.contains("<svg"));
        assert!(s.contains("</svg>"));
    }

    #[test]
    fn test_project_to_2d_length() {
        let pos = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let proj = project_to_2d(&pos, [0.0, 0.0, 1.0]);
        assert_eq!(proj.len(), pos.len());
    }

    #[test]
    fn test_project_to_2d_no_nan() {
        let pos = vec![[1.0_f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let proj = project_to_2d(&pos, [0.0, 0.0, 1.0]);
        for p in &proj {
            assert!(!p[0].is_nan());
            assert!(!p[1].is_nan());
        }
    }

    #[test]
    fn test_silhouette_edges_nonempty_for_cube() {
        let pos = cube_positions();
        let idx = cube_indices();
        let edges = find_silhouette_edges(&pos, &idx, [0.0, 0.0, 1.0]);
        assert!(!edges.is_empty());
    }

    #[test]
    fn test_path_count_zero() {
        let doc = new_svg_document(100, 100);
        assert_eq!(path_count(&doc), 0);
    }

    #[test]
    fn test_path_count_multiple() {
        let mut doc = new_svg_document(100, 100);
        for _ in 0..5 {
            add_path(
                &mut doc,
                SvgPath {
                    d: "M 0 0 L 1 1".to_string(),
                    stroke: "#000".to_string(),
                    fill: "none".to_string(),
                    stroke_width: 1.0,
                },
            );
        }
        assert_eq!(path_count(&doc), 5);
    }

    #[test]
    fn test_svg_bounds_matches_viewbox() {
        let doc = new_svg_document(640, 480);
        let bounds = svg_bounds(&doc);
        assert_eq!(bounds[2], 640.0);
        assert_eq!(bounds[3], 480.0);
    }

    #[test]
    fn test_scale_svg() {
        let mut doc = new_svg_document(100, 100);
        scale_svg(&mut doc, 2.0);
        assert_eq!(doc.width, 200);
        assert_eq!(doc.height, 200);
    }

    #[test]
    fn test_default_svg_config() {
        let cfg = default_svg_config();
        assert_eq!(cfg.width, 800);
        assert_eq!(cfg.height, 600);
        assert!(!cfg.stroke_color.is_empty());
    }

    #[test]
    fn test_mesh_silhouette_svg_returns_document() {
        let pos = cube_positions();
        let idx = cube_indices();
        let cfg = default_svg_config();
        let doc = mesh_silhouette_svg(&pos, &idx, [0.0, 0.0, 1.0], &cfg);
        assert_eq!(doc.width, cfg.width);
    }

    #[test]
    fn test_edges_to_svg_paths() {
        let edges: Vec<[u32; 2]> = vec![[0, 1], [1, 2]];
        let pos2d: Vec<[f32; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
        let cfg = default_svg_config();
        let paths = edges_to_svg_paths(&edges, &pos2d, &cfg);
        assert_eq!(paths.len(), 2);
    }

    // ── New high-level API tests ───────────────────────────────────────────────

    #[test]
    fn test_default_svg_export_config() {
        let cfg = default_svg_export_config();
        assert_eq!(cfg.width, 800);
        assert_eq!(cfg.height, 600);
        assert_eq!(cfg.stroke_color, "#000000");
    }

    #[test]
    fn test_new_svg_document_from_config() {
        let cfg = default_svg_export_config();
        let doc = new_svg_document_from_config(&cfg);
        assert_eq!(doc.width, 800);
        assert_eq!(doc.height, 600);
        assert!(doc.paths.is_empty());
    }

    #[test]
    fn test_project_mesh_to_svg_produces_paths() {
        let verts: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let faces: Vec<[u32; 3]> = vec![[0, 1, 2]];
        let cfg = default_svg_export_config();
        let doc = project_mesh_to_svg(&verts, &faces, &cfg);
        assert_eq!(doc.paths.len(), 1);
    }

    #[test]
    fn test_svg_add_path() {
        let mut doc = new_svg_document(100, 100);
        let p = SvgPath {
            d: "M 0 0 L 5 5".to_string(),
            stroke: "#f00".to_string(),
            fill: "none".to_string(),
            stroke_width: 2.0,
        };
        svg_add_path(&mut doc, p);
        assert_eq!(path_count(&doc), 1);
    }

    #[test]
    fn test_svg_to_string_produces_svg() {
        let doc = new_svg_document(200, 200);
        let s = svg_to_string(&doc);
        assert!(s.contains("<svg"));
    }

    #[test]
    fn test_svg_path_from_contour_closed() {
        let pts = vec![[0.0_f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let p = svg_path_from_contour(&pts);
        assert!(p.d.ends_with('Z'));
    }

    #[test]
    fn test_svg_document_bounds() {
        let doc = new_svg_document(320, 240);
        let vb = svg_document_bounds(&doc);
        assert!((vb.width - 320.0).abs() < 1e-5);
        assert!((vb.height - 240.0).abs() < 1e-5);
    }

    #[test]
    fn test_svg_set_stroke_color() {
        let mut doc = new_svg_document(100, 100);
        svg_add_path(
            &mut doc,
            SvgPath {
                d: "M 0 0 L 1 1".to_string(),
                stroke: "#000".to_string(),
                fill: "none".to_string(),
                stroke_width: 1.0,
            },
        );
        svg_set_stroke_color(&mut doc, "#ff0000");
        assert_eq!(doc.paths[0].stroke, "#ff0000");
    }

    #[test]
    fn test_svg_set_fill_color() {
        let mut doc = new_svg_document(100, 100);
        svg_add_path(
            &mut doc,
            SvgPath {
                d: "M 0 0 L 1 1".to_string(),
                stroke: "#000".to_string(),
                fill: "none".to_string(),
                stroke_width: 1.0,
            },
        );
        svg_set_fill_color(&mut doc, "#0000ff");
        assert_eq!(doc.paths[0].fill, "#0000ff");
    }

    #[test]
    fn test_svg_write_to_file() {
        let doc = new_svg_document(50, 50);
        let path = "/tmp/test_svg_export_write.svg";
        let result = svg_write_to_file(&doc, path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_project_mesh_to_svg_oob_faces_skipped() {
        let verts: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]];
        let faces: Vec<[u32; 3]> = vec![[0, 1, 2]]; // indices 1,2 out of bounds
        let cfg = default_svg_export_config();
        let doc = project_mesh_to_svg(&verts, &faces, &cfg);
        assert_eq!(doc.paths.len(), 0);
    }
}
