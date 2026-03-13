// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A beveled edge entry.
#[allow(dead_code)]
#[derive(Clone)]
pub struct BevelEdge {
    pub a: u32,
    pub b: u32,
    pub width: f32,
    pub segments: usize,
}

/// Export bundle for edge bevels.
#[allow(dead_code)]
#[derive(Default)]
pub struct EdgeBevelExport {
    pub edges: Vec<BevelEdge>,
}

/// Create a new edge bevel export.
#[allow(dead_code)]
pub fn new_edge_bevel_export() -> EdgeBevelExport {
    EdgeBevelExport::default()
}

/// Add a bevel edge.
#[allow(dead_code)]
pub fn add_bevel_edge(export: &mut EdgeBevelExport, a: u32, b: u32, width: f32, segments: usize) {
    export.edges.push(BevelEdge {
        a,
        b,
        width,
        segments,
    });
}

/// Count bevel edges.
#[allow(dead_code)]
pub fn bevel_edge_count(export: &EdgeBevelExport) -> usize {
    export.edges.len()
}

/// Maximum bevel width.
#[allow(dead_code)]
pub fn max_bevel_width(export: &EdgeBevelExport) -> f32 {
    export.edges.iter().map(|e| e.width).fold(0.0_f32, f32::max)
}

/// Average bevel width.
#[allow(dead_code)]
pub fn avg_bevel_width(export: &EdgeBevelExport) -> f32 {
    if export.edges.is_empty() {
        return 0.0;
    }
    export.edges.iter().map(|e| e.width).sum::<f32>() / export.edges.len() as f32
}

/// Total bevel geometry faces (2*segments per edge).
#[allow(dead_code)]
pub fn total_bevel_faces(export: &EdgeBevelExport) -> usize {
    export.edges.iter().map(|e| e.segments * 2).sum()
}

/// Find a bevel edge connecting a and b.
#[allow(dead_code)]
pub fn find_bevel_edge(export: &EdgeBevelExport, a: u32, b: u32) -> Option<&BevelEdge> {
    export
        .edges
        .iter()
        .find(|e| (e.a == a && e.b == b) || (e.a == b && e.b == a))
}

/// Validate all bevels (positive width and segments > 0).
#[allow(dead_code)]
pub fn validate_bevel_export(export: &EdgeBevelExport) -> bool {
    export.edges.iter().all(|e| e.width > 0.0 && e.segments > 0)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn edge_bevel_to_json(export: &EdgeBevelExport) -> String {
    format!(
        r#"{{"bevel_edges":{},"total_faces":{}}}"#,
        export.edges.len(),
        total_bevel_faces(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count() {
        let mut e = new_edge_bevel_export();
        add_bevel_edge(&mut e, 0, 1, 0.1, 2);
        assert_eq!(bevel_edge_count(&e), 1);
    }

    #[test]
    fn max_width() {
        let mut e = new_edge_bevel_export();
        add_bevel_edge(&mut e, 0, 1, 0.2, 2);
        add_bevel_edge(&mut e, 1, 2, 0.5, 2);
        assert!((max_bevel_width(&e) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn avg_width() {
        let mut e = new_edge_bevel_export();
        add_bevel_edge(&mut e, 0, 1, 0.2, 1);
        add_bevel_edge(&mut e, 1, 2, 0.4, 1);
        assert!((avg_bevel_width(&e) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn total_faces() {
        let mut e = new_edge_bevel_export();
        add_bevel_edge(&mut e, 0, 1, 0.1, 3);
        assert_eq!(total_bevel_faces(&e), 6);
    }

    #[test]
    fn find_edge() {
        let mut e = new_edge_bevel_export();
        add_bevel_edge(&mut e, 0, 1, 0.1, 1);
        assert!(find_bevel_edge(&e, 1, 0).is_some());
    }

    #[test]
    fn validate_valid() {
        let mut e = new_edge_bevel_export();
        add_bevel_edge(&mut e, 0, 1, 0.1, 1);
        assert!(validate_bevel_export(&e));
    }

    #[test]
    fn validate_zero_width_fails() {
        let mut e = new_edge_bevel_export();
        add_bevel_edge(&mut e, 0, 1, 0.0, 1);
        assert!(!validate_bevel_export(&e));
    }

    #[test]
    fn json_has_edges() {
        let e = new_edge_bevel_export();
        let j = edge_bevel_to_json(&e);
        assert!(j.contains("\"bevel_edges\":0"));
    }

    #[test]
    fn empty_avg_width() {
        let e = new_edge_bevel_export();
        assert!((avg_bevel_width(&e) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn find_missing() {
        let e = new_edge_bevel_export();
        assert!(find_bevel_edge(&e, 0, 1).is_none());
    }
}
