// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SVG wireframe and silhouette export for OxiHuman meshes.
//!
//! Projects a 3D mesh onto a 2D plane and writes an SVG file — useful for
//! technical drawings, silhouettes, UV maps, and shape thumbnails.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::Result;
use oxihuman_mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Projection mode for 3D→2D conversion.
pub enum SvgProjection {
    /// Orthographic along -Z axis (view from front): keeps (X, Y).
    Front,
    /// Orthographic along -X axis (view from right side): keeps (Z, Y).
    Side,
    /// Orthographic along -Y axis (view from top): keeps (X, Z).
    Top,
    /// Custom camera: look_from, look_at, up.
    Custom {
        from: [f32; 3],
        at: [f32; 3],
        up: [f32; 3],
    },
}

/// Options controlling SVG export.
pub struct SvgExportOptions {
    pub projection: SvgProjection,
    /// SVG canvas width in pixels.
    pub width: u32,
    /// SVG canvas height in pixels.
    pub height: u32,
    /// Fractional margin `0..1` applied on each side (default `0.05`).
    pub margin: f32,
    /// CSS-style stroke colour, e.g. `"#333333"`.
    pub stroke_color: String,
    /// Stroke width in pixels.
    pub stroke_width: f32,
    /// Optional fill colour for faces; `None` means no fill.
    pub fill_color: Option<String>,
    /// Optional background colour; `None` means transparent.
    pub background: Option<String>,
    /// Draw all mesh edges.
    pub draw_wireframe: bool,
    /// Draw only silhouette edges.
    pub draw_silhouette: bool,
}

impl Default for SvgExportOptions {
    fn default() -> Self {
        Self {
            projection: SvgProjection::Front,
            width: 512,
            height: 512,
            margin: 0.05,
            stroke_color: "#222222".to_string(),
            stroke_width: 0.5,
            fill_color: None,
            background: None,
            draw_wireframe: true,
            draw_silhouette: false,
        }
    }
}

/// Statistics returned by [`export_svg`].
pub struct SvgExportStats {
    pub edge_count: usize,
    pub vertex_count: usize,
    pub face_count: usize,
    pub silhouette_edge_count: usize,
}

// ---------------------------------------------------------------------------
// Internal maths helpers
// ---------------------------------------------------------------------------

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

// ---------------------------------------------------------------------------
// Projection
// ---------------------------------------------------------------------------

/// Project a single vertex position using the chosen projection mode.
fn project_vertex(pos: [f32; 3], proj: &SvgProjection) -> [f32; 2] {
    match proj {
        SvgProjection::Front => [pos[0], pos[1]],
        SvgProjection::Side => [pos[2], pos[1]],
        SvgProjection::Top => [pos[0], pos[2]],
        SvgProjection::Custom { from, at, up } => {
            // Forward vector (from camera toward target).
            let forward = vec3_normalize(vec3_sub(*at, *from));
            // Right vector.
            let right = vec3_normalize(vec3_cross(forward, *up));
            // Recomputed up (orthonormal).
            let up_ortho = vec3_cross(right, forward);
            // Offset from camera origin.
            let offset = vec3_sub(pos, *from);
            let u = vec3_dot(offset, right);
            let v = vec3_dot(offset, up_ortho);
            [u, v]
        }
    }
}

/// Project all mesh vertices to 2D using `proj`.
pub fn project_mesh(mesh: &MeshBuffers, proj: &SvgProjection) -> Vec<[f32; 2]> {
    mesh.positions
        .iter()
        .map(|&p| project_vertex(p, proj))
        .collect()
}

// ---------------------------------------------------------------------------
// Fit-to-canvas
// ---------------------------------------------------------------------------

/// Scale and translate projected 2D points so they fit inside the canvas with
/// `margin` fractional padding on each side.  SVG Y-axis is flipped.
fn fit_to_canvas(pts: &[[f32; 2]], width: u32, height: u32, margin: f32) -> Vec<[f32; 2]> {
    if pts.is_empty() {
        return vec![];
    }

    let mut min_x = pts[0][0];
    let mut max_x = pts[0][0];
    let mut min_y = pts[0][1];
    let mut max_y = pts[0][1];

    for p in pts {
        if p[0] < min_x {
            min_x = p[0];
        }
        if p[0] > max_x {
            max_x = p[0];
        }
        if p[1] < min_y {
            min_y = p[1];
        }
        if p[1] > max_y {
            max_y = p[1];
        }
    }

    let w = width as f32;
    let h = height as f32;
    let canvas_w = w * (1.0 - 2.0 * margin);
    let canvas_h = h * (1.0 - 2.0 * margin);

    let range_x = max_x - min_x;
    let range_y = max_y - min_y;

    // Keep aspect ratio: pick the smaller scale factor.
    let scale = if range_x < 1e-9 && range_y < 1e-9 {
        1.0_f32
    } else if range_x < 1e-9 {
        canvas_h / range_y
    } else if range_y < 1e-9 {
        canvas_w / range_x
    } else {
        (canvas_w / range_x).min(canvas_h / range_y)
    };

    let scaled_w = range_x * scale;
    let scaled_h = range_y * scale;
    let offset_x = w * margin + (canvas_w - scaled_w) * 0.5;
    let offset_y = h * margin + (canvas_h - scaled_h) * 0.5;

    pts.iter()
        .map(|p| {
            let sx = (p[0] - min_x) * scale + offset_x;
            // Flip Y: SVG origin is top-left, mathematical origin is bottom-left.
            let sy = h - ((p[1] - min_y) * scale + offset_y);
            [sx, sy]
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Edge extraction
// ---------------------------------------------------------------------------

/// Build a deduplicated set of edges from triangle index list.
fn collect_edges(indices: &[u32]) -> HashSet<(u32, u32)> {
    let mut edges = HashSet::new();
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        edges.insert((a.min(b), a.max(b)));
        edges.insert((b.min(c), b.max(c)));
        edges.insert((a.min(c), a.max(c)));
    }
    edges
}

// ---------------------------------------------------------------------------
// Silhouette detection
// ---------------------------------------------------------------------------

/// Return the face normal for triangle `(i0, i1, i2)`.
fn face_normal(positions: &[[f32; 3]], i0: u32, i1: u32, i2: u32) -> [f32; 3] {
    let p0 = positions[i0 as usize];
    let p1 = positions[i1 as usize];
    let p2 = positions[i2 as usize];
    let e1 = vec3_sub(p1, p0);
    let e2 = vec3_sub(p2, p0);
    vec3_normalize(vec3_cross(e1, e2))
}

/// Find silhouette edges: edges shared by exactly two faces where one face
/// is front-facing and the other is back-facing with respect to `view_dir`.
pub fn find_silhouette_edges(mesh: &MeshBuffers, view_dir: [f32; 3]) -> Vec<(u32, u32)> {
    // Map each ordered edge (min, max) to its face dot-product signs.
    let mut edge_faces: HashMap<(u32, u32), Vec<f32>> = HashMap::new();

    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0], tri[1], tri[2]);
        let n = face_normal(&mesh.positions, i0, i1, i2);
        let dot = vec3_dot(n, view_dir);

        let pairs = [
            (i0.min(i1), i0.max(i1)),
            (i1.min(i2), i1.max(i2)),
            (i0.min(i2), i0.max(i2)),
        ];
        for edge in pairs {
            edge_faces.entry(edge).or_default().push(dot);
        }
    }

    edge_faces
        .into_iter()
        .filter(|(_, dots)| {
            // Silhouette: at least one positive and one negative dot product.
            let has_front = dots.iter().any(|&d| d > 0.0);
            let has_back = dots.iter().any(|&d| d < 0.0);
            has_front && has_back
        })
        .map(|(edge, _)| edge)
        .collect()
}

// ---------------------------------------------------------------------------
// SVG builders
// ---------------------------------------------------------------------------

/// Build an SVG string from the mesh using `opts`.
pub fn build_svg(mesh: &MeshBuffers, opts: &SvgExportOptions) -> String {
    let projected = project_mesh(mesh, &opts.projection);
    let canvas = fit_to_canvas(&projected, opts.width, opts.height, opts.margin);

    let w = opts.width;
    let h = opts.height;
    let mut svg = String::new();

    // XML header and SVG opening tag.
    svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
        w, h, w, h
    ));

    // Optional background rect.
    if let Some(ref bg) = opts.background {
        svg.push_str(&format!(
            "  <rect width=\"{}\" height=\"{}\" fill=\"{}\"/>\n",
            w, h, bg
        ));
    }

    if canvas.is_empty() || mesh.indices.is_empty() {
        svg.push_str("</svg>\n");
        return svg;
    }

    let fill = opts.fill_color.as_deref().unwrap_or("none");

    // Draw filled polygons (one per face).
    if fill != "none" {
        for tri in mesh.indices.chunks_exact(3) {
            let p0 = canvas[tri[0] as usize];
            let p1 = canvas[tri[1] as usize];
            let p2 = canvas[tri[2] as usize];
            svg.push_str(&format!(
                "  <polygon points=\"{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}\" fill=\"{}\" stroke=\"none\"/>\n",
                p0[0], p0[1], p1[0], p1[1], p2[0], p2[1], fill
            ));
        }
    }

    // Determine which edges to draw.
    let view_dir = view_direction(&opts.projection);

    if opts.draw_wireframe {
        let edges = collect_edges(&mesh.indices);
        for (a, b) in &edges {
            let pa = canvas[*a as usize];
            let pb = canvas[*b as usize];
            svg.push_str(&format!(
                "  <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                pa[0], pa[1], pb[0], pb[1], opts.stroke_color, opts.stroke_width
            ));
        }
    }

    if opts.draw_silhouette {
        let sil = find_silhouette_edges(mesh, view_dir);
        for (a, b) in &sil {
            let pa = canvas[*a as usize];
            let pb = canvas[*b as usize];
            svg.push_str(&format!(
                "  <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                pa[0], pa[1], pb[0], pb[1], opts.stroke_color, opts.stroke_width * 2.0
            ));
        }
    }

    svg.push_str("</svg>\n");
    svg
}

/// Return a unit view direction vector for the given projection.
fn view_direction(proj: &SvgProjection) -> [f32; 3] {
    match proj {
        SvgProjection::Front => [0.0, 0.0, -1.0],
        SvgProjection::Side => [-1.0, 0.0, 0.0],
        SvgProjection::Top => [0.0, -1.0, 0.0],
        SvgProjection::Custom { from, at, .. } => vec3_normalize(vec3_sub(*at, *from)),
    }
}

// ---------------------------------------------------------------------------
// UV SVG
// ---------------------------------------------------------------------------

/// Build an SVG string showing the UV layout of the mesh.
pub fn build_uv_svg(mesh: &MeshBuffers, width: u32, height: u32) -> String {
    let w = width as f32;
    let h = height as f32;

    let mut svg = String::new();
    svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
        width, height, width, height
    ));
    svg.push_str(&format!(
        "  <rect width=\"{}\" height=\"{}\" fill=\"#f8f8f8\" stroke=\"#cccccc\" stroke-width=\"1\"/>\n",
        width, height
    ));

    if !mesh.uvs.is_empty() {
        for tri in mesh.indices.chunks_exact(3) {
            // UV coords: flip V because SVG is top-down.
            let uv0 = mesh.uvs[tri[0] as usize];
            let uv1 = mesh.uvs[tri[1] as usize];
            let uv2 = mesh.uvs[tri[2] as usize];
            let (x0, y0) = (uv0[0] * w, (1.0 - uv0[1]) * h);
            let (x1, y1) = (uv1[0] * w, (1.0 - uv1[1]) * h);
            let (x2, y2) = (uv2[0] * w, (1.0 - uv2[1]) * h);
            svg.push_str(&format!(
                "  <polygon points=\"{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}\" fill=\"none\" stroke=\"#3366cc\" stroke-width=\"0.5\"/>\n",
                x0, y0, x1, y1, x2, y2
            ));
        }
    }

    svg.push_str("</svg>\n");
    svg
}

/// Export the mesh UV layout as an SVG file at `path`.
pub fn export_uv_svg(mesh: &MeshBuffers, path: &Path) -> Result<()> {
    let svg = build_uv_svg(mesh, 512, 512);
    std::fs::write(path, svg)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// File export
// ---------------------------------------------------------------------------

/// Export a mesh as an SVG wireframe/silhouette file.
///
/// Returns [`SvgExportStats`] with counts of edges, vertices, faces and
/// silhouette edges included in the output.
pub fn export_svg(
    mesh: &MeshBuffers,
    path: &Path,
    opts: &SvgExportOptions,
) -> Result<SvgExportStats> {
    let svg = build_svg(mesh, opts);
    std::fs::write(path, &svg)?;

    let edges = collect_edges(&mesh.indices);
    let view_dir = view_direction(&opts.projection);
    let silhouette = find_silhouette_edges(mesh, view_dir);

    Ok(SvgExportStats {
        edge_count: edges.len(),
        vertex_count: mesh.positions.len(),
        face_count: mesh.indices.len() / 3,
        silhouette_edge_count: silhouette.len(),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    /// Two-triangle quad mesh lying in the Z=0 plane.
    fn two_tri_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0_f32, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        })
    }

    // -----------------------------------------------------------------------
    // Projection tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_project_front() {
        let mesh = two_tri_mesh();
        let pts = project_mesh(&mesh, &SvgProjection::Front);
        assert_eq!(pts.len(), 4);
        // Front: (X, Y) — Z is discarded.
        assert!((pts[0][0] - 0.0).abs() < 1e-5);
        assert!((pts[0][1] - 0.0).abs() < 1e-5);
        assert!((pts[2][0] - 1.0).abs() < 1e-5);
        assert!((pts[2][1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_project_side() {
        let mesh = two_tri_mesh();
        let pts = project_mesh(&mesh, &SvgProjection::Side);
        assert_eq!(pts.len(), 4);
        // Side: (Z, Y) — X is discarded.  All Z=0, so X of projected = 0.
        for p in &pts {
            assert!(
                (p[0] - 0.0).abs() < 1e-5,
                "expected Z=0 => u=0, got {}",
                p[0]
            );
        }
        // Y values should match original Y.
        assert!((pts[2][1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_project_top() {
        let mesh = two_tri_mesh();
        let pts = project_mesh(&mesh, &SvgProjection::Top);
        assert_eq!(pts.len(), 4);
        // Top: (X, Z) — Y is discarded.  All Z=0, so V component = 0.
        for p in &pts {
            assert!(
                (p[1] - 0.0).abs() < 1e-5,
                "expected Z=0 => v=0, got {}",
                p[1]
            );
        }
        assert!((pts[1][0] - 1.0).abs() < 1e-5);
    }

    // -----------------------------------------------------------------------
    // build_svg tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_build_svg_has_xml_header() {
        let mesh = two_tri_mesh();
        let svg = build_svg(&mesh, &SvgExportOptions::default());
        assert!(svg.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    }

    #[test]
    fn test_build_svg_has_svg_tag() {
        let mesh = two_tri_mesh();
        let svg = build_svg(&mesh, &SvgExportOptions::default());
        assert!(svg.contains("<svg xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_build_svg_basic() {
        let mesh = two_tri_mesh();
        let opts = SvgExportOptions::default();
        let svg = build_svg(&mesh, &opts);
        // Wireframe is on by default: expect <line> elements.
        assert!(
            svg.contains("<line "),
            "expected <line> elements in wireframe SVG"
        );
        // Should encode canvas size.
        assert!(svg.contains("width=\"512\""));
        assert!(svg.contains("height=\"512\""));
    }

    #[test]
    fn test_build_svg_with_background() {
        let mesh = two_tri_mesh();
        let opts = SvgExportOptions {
            background: Some("#ffffff".to_string()),
            ..Default::default()
        };
        let svg = build_svg(&mesh, &opts);
        assert!(svg.contains("<rect"), "expected <rect> for background");
        assert!(svg.contains("#ffffff"));
    }

    // -----------------------------------------------------------------------
    // UV SVG tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_build_uv_svg() {
        let mesh = two_tri_mesh();
        let svg = build_uv_svg(&mesh, 256, 256);
        assert!(svg.starts_with("<?xml version=\"1.0\""));
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(
            svg.contains("<polygon"),
            "expected UV triangles rendered as polygons"
        );
    }

    // -----------------------------------------------------------------------
    // File export tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_export_svg_to_file() {
        let mesh = two_tri_mesh();
        let path = Path::new("/tmp/test_oxihuman_export.svg");
        let stats = export_svg(&mesh, path, &SvgExportOptions::default()).unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("<svg"));
        assert_eq!(stats.vertex_count, 4);
        assert_eq!(stats.face_count, 2);
        // A quad made of 2 triangles has 5 unique edges.
        assert_eq!(stats.edge_count, 5);
    }

    #[test]
    fn test_export_uv_svg_to_file() {
        let mesh = two_tri_mesh();
        let path = Path::new("/tmp/test_oxihuman_uv_export.svg");
        export_uv_svg(&mesh, path).unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("<svg"));
        assert!(content.contains("<polygon"));
    }

    // -----------------------------------------------------------------------
    // Silhouette tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_find_silhouette_edges() {
        // Tetrahedron: with view_dir=[0,0,-1], two faces are back-facing and
        // one is front-facing, so the edges shared between front and back faces
        // are silhouette edges.
        //   A=(0,0,0) B=(1,0,0) C=(0.5,0,1) D=(0.5,1,0.5)
        // Face normals vs view [0,0,-1]:
        //   ABC: [0,-1,0]  dot=0   (tangent — neither)
        //   ADB: [0,0.45,-0.89] dot=0.89  (front-facing)
        //   BDC: [0.87,0.22,0.44] dot=-0.44 (back-facing)
        //   ACD: [-0.87,0.22,0.44] dot=-0.44 (back-facing)
        // => silhouette edges: (A,D)=(0,3) and (B,D)=(1,3)
        let mesh = MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0_f32, 0.0, 0.0], // A=0
                [1.0, 0.0, 0.0],     // B=1
                [0.5, 0.0, 1.0],     // C=2
                [0.5, 1.0, 0.5],     // D=3
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![
                0, 1, 2, // ABC — tangent
                0, 3, 1, // ADB — front-facing
                1, 3, 2, // BDC — back-facing
                0, 2, 3, // ACD — back-facing
            ],
            has_suit: false,
        });

        let view_dir = [0.0_f32, 0.0, -1.0];
        let sil = find_silhouette_edges(&mesh, view_dir);
        // Edges (0,3) and (1,3) are silhouette edges.
        assert!(
            sil.len() >= 2,
            "expected at least 2 silhouette edges, got {}",
            sil.len()
        );
        assert!(
            sil.contains(&(0, 3)),
            "expected edge (0,3) to be a silhouette edge"
        );
        assert!(
            sil.contains(&(1, 3)),
            "expected edge (1,3) to be a silhouette edge"
        );
    }

    // -----------------------------------------------------------------------
    // Default options test
    // -----------------------------------------------------------------------

    #[test]
    fn test_svg_options_default() {
        let opts = SvgExportOptions::default();
        assert_eq!(opts.width, 512);
        assert_eq!(opts.height, 512);
        assert!((opts.margin - 0.05).abs() < 1e-6);
        assert_eq!(opts.stroke_color, "#222222");
        assert!((opts.stroke_width - 0.5).abs() < 1e-6);
        assert!(opts.fill_color.is_none());
        assert!(opts.background.is_none());
        assert!(opts.draw_wireframe);
        assert!(!opts.draw_silhouette);
    }
}
