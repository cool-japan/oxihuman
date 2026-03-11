// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! EPS (Encapsulated PostScript) vector export stub.

/// EPS document options.
#[allow(dead_code)]
pub struct EpsOptions {
    pub title: String,
    pub width_pt: f32,
    pub height_pt: f32,
    pub line_width: f32,
}

impl Default for EpsOptions {
    fn default() -> Self {
        EpsOptions {
            title: "OxiHuman Export".to_string(),
            width_pt: 595.0,
            height_pt: 842.0,
            line_width: 1.0,
        }
    }
}

/// A 2-D path in EPS coordinates.
#[allow(dead_code)]
pub struct EpsPath {
    pub points: Vec<[f32; 2]>,
    pub closed: bool,
    pub stroke_rgb: [f32; 3],
    pub fill_rgb: Option<[f32; 3]>,
}

/// EPS document accumulator.
#[allow(dead_code)]
pub struct EpsDocument {
    pub options: EpsOptions,
    pub paths: Vec<EpsPath>,
}

/// Create a new EPS document.
#[allow(dead_code)]
pub fn new_eps_document(options: EpsOptions) -> EpsDocument {
    EpsDocument {
        options,
        paths: Vec::new(),
    }
}

/// Add a path to the document.
#[allow(dead_code)]
pub fn add_eps_path(doc: &mut EpsDocument, path: EpsPath) {
    doc.paths.push(path);
}

/// Serialize the EPS document to a string.
#[allow(dead_code)]
pub fn export_eps(doc: &EpsDocument) -> String {
    let mut out = String::new();
    out.push_str("%!PS-Adobe-3.0 EPSF-3.0\n");
    out.push_str(&format!(
        "%%BoundingBox: 0 0 {} {}\n",
        doc.options.width_pt as i32, doc.options.height_pt as i32
    ));
    out.push_str(&format!("%%Title: {}\n", doc.options.title));
    out.push_str("%%EndComments\n");
    out.push_str(&format!("{} setlinewidth\n", doc.options.line_width));
    for path in &doc.paths {
        if path.points.is_empty() {
            continue;
        }
        let [r, g, b] = path.stroke_rgb;
        out.push_str(&format!("{:.4} {:.4} {:.4} setrgbcolor\n", r, g, b));
        let p0 = path.points[0];
        out.push_str(&format!("{:.4} {:.4} moveto\n", p0[0], p0[1]));
        for &p in &path.points[1..] {
            out.push_str(&format!("{:.4} {:.4} lineto\n", p[0], p[1]));
        }
        if path.closed {
            out.push_str("closepath\n");
            if let Some([fr, fg, fb]) = path.fill_rgb {
                out.push_str("gsave\n");
                out.push_str(&format!("{:.4} {:.4} {:.4} setrgbcolor\n", fr, fg, fb));
                out.push_str("fill\ngrestore\n");
            }
        }
        out.push_str("stroke\n");
    }
    out.push_str("%%EOF\n");
    out
}

/// Path count.
#[allow(dead_code)]
pub fn eps_path_count(doc: &EpsDocument) -> usize {
    doc.paths.len()
}

/// Compute the bounding box of all paths in the document.
#[allow(dead_code)]
pub fn eps_bounding_box(doc: &EpsDocument) -> ([f32; 2], [f32; 2]) {
    let mut mn = [f32::INFINITY; 2];
    let mut mx = [f32::NEG_INFINITY; 2];
    for path in &doc.paths {
        for &p in &path.points {
            mn[0] = mn[0].min(p[0]);
            mn[1] = mn[1].min(p[1]);
            mx[0] = mx[0].max(p[0]);
            mx[1] = mx[1].max(p[1]);
        }
    }
    if mn[0].is_infinite() {
        mn = [0.0; 2];
        mx = [0.0; 2];
    }
    (mn, mx)
}

/// Convert mesh silhouette edges to EPS paths.
#[allow(dead_code)]
pub fn edges_to_eps_paths(
    positions_2d: &[[f32; 2]],
    edges: &[[u32; 2]],
    stroke: [f32; 3],
) -> Vec<EpsPath> {
    edges
        .iter()
        .map(|&[a, b]| EpsPath {
            points: vec![positions_2d[a as usize], positions_2d[b as usize]],
            closed: false,
            stroke_rgb: stroke,
            fill_rgb: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_path() -> EpsPath {
        EpsPath {
            points: vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0]],
            closed: false,
            stroke_rgb: [0.0, 0.0, 0.0],
            fill_rgb: None,
        }
    }

    #[test]
    fn eps_has_header() {
        let doc = new_eps_document(EpsOptions::default());
        let eps = export_eps(&doc);
        assert!(eps.contains("%!PS-Adobe"));
    }

    #[test]
    fn eps_has_bounding_box() {
        let doc = new_eps_document(EpsOptions::default());
        let eps = export_eps(&doc);
        assert!(eps.contains("BoundingBox"));
    }

    #[test]
    fn eps_has_eof() {
        let doc = new_eps_document(EpsOptions::default());
        let eps = export_eps(&doc);
        assert!(eps.contains("%%EOF"));
    }

    #[test]
    fn add_path_increases_count() {
        let mut doc = new_eps_document(EpsOptions::default());
        add_eps_path(&mut doc, simple_path());
        assert_eq!(eps_path_count(&doc), 1);
    }

    #[test]
    fn eps_contains_moveto() {
        let mut doc = new_eps_document(EpsOptions::default());
        add_eps_path(&mut doc, simple_path());
        let eps = export_eps(&doc);
        assert!(eps.contains("moveto"));
    }

    #[test]
    fn eps_contains_lineto() {
        let mut doc = new_eps_document(EpsOptions::default());
        add_eps_path(&mut doc, simple_path());
        let eps = export_eps(&doc);
        assert!(eps.contains("lineto"));
    }

    #[test]
    fn bounding_box_empty() {
        let doc = new_eps_document(EpsOptions::default());
        let (mn, mx) = eps_bounding_box(&doc);
        assert_eq!(mn, [0.0, 0.0]);
        assert_eq!(mx, [0.0, 0.0]);
    }

    #[test]
    fn bounding_box_correct() {
        let mut doc = new_eps_document(EpsOptions::default());
        add_eps_path(&mut doc, simple_path());
        let (mn, mx) = eps_bounding_box(&doc);
        assert!((mn[0] - 0.0).abs() < 1e-5);
        assert!((mx[0] - 100.0).abs() < 1e-5);
    }

    #[test]
    fn edges_to_eps_paths_count() {
        let pts = vec![[0.0f32, 0.0], [100.0, 0.0], [100.0, 100.0]];
        let edges = vec![[0u32, 1], [1, 2]];
        let paths = edges_to_eps_paths(&pts, &edges, [0.0, 0.0, 0.0]);
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn closed_path_has_closepath() {
        let mut doc = new_eps_document(EpsOptions::default());
        let p = EpsPath {
            points: vec![[0.0, 0.0], [50.0, 0.0], [25.0, 50.0]],
            closed: true,
            stroke_rgb: [0.0, 0.0, 0.0],
            fill_rgb: None,
        };
        add_eps_path(&mut doc, p);
        let eps = export_eps(&doc);
        assert!(eps.contains("closepath"));
    }

    #[test]
    fn title_in_eps() {
        let opts = EpsOptions {
            title: "MyTest".to_string(),
            ..Default::default()
        };
        let doc = new_eps_document(opts);
        let eps = export_eps(&doc);
        assert!(eps.contains("MyTest"));
    }
}
