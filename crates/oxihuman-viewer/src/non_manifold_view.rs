// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Non-manifold edge/vertex highlight visualization.

/// Non-manifold issue type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NonManifoldKind {
    Edge,
    Vertex,
    BoundaryEdge,
    IsolatedVertex,
}

impl NonManifoldKind {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            NonManifoldKind::Edge => "non_manifold_edge",
            NonManifoldKind::Vertex => "non_manifold_vertex",
            NonManifoldKind::BoundaryEdge => "boundary_edge",
            NonManifoldKind::IsolatedVertex => "isolated_vertex",
        }
    }
}

/// Non-manifold visualization config.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NonManifoldConfig {
    pub edge_color: [f32; 3],
    pub vertex_color: [f32; 3],
    pub boundary_color: [f32; 3],
    pub isolated_color: [f32; 3],
    pub point_size: f32,
    pub line_width: f32,
    pub enabled: bool,
}

impl Default for NonManifoldConfig {
    fn default() -> Self {
        NonManifoldConfig {
            edge_color: [1.0, 0.0, 0.0],
            vertex_color: [1.0, 0.5, 0.0],
            boundary_color: [0.0, 0.7, 1.0],
            isolated_color: [1.0, 1.0, 0.0],
            point_size: 8.0,
            line_width: 2.0,
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_non_manifold_config() -> NonManifoldConfig {
    NonManifoldConfig::default()
}

#[allow(dead_code)]
pub fn nm_enable(cfg: &mut NonManifoldConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn nm_disable(cfg: &mut NonManifoldConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn nm_set_point_size(cfg: &mut NonManifoldConfig, s: f32) {
    cfg.point_size = s.clamp(1.0, 50.0);
}

#[allow(dead_code)]
pub fn nm_set_line_width(cfg: &mut NonManifoldConfig, w: f32) {
    cfg.line_width = w.clamp(0.5, 20.0);
}

/// Get display color for a given non-manifold kind.
#[allow(dead_code)]
pub fn nm_kind_color(cfg: &NonManifoldConfig, kind: NonManifoldKind) -> [f32; 3] {
    match kind {
        NonManifoldKind::Edge => cfg.edge_color,
        NonManifoldKind::Vertex => cfg.vertex_color,
        NonManifoldKind::BoundaryEdge => cfg.boundary_color,
        NonManifoldKind::IsolatedVertex => cfg.isolated_color,
    }
}

/// Detect if an edge is non-manifold (belongs to more than 2 faces).
#[allow(dead_code)]
pub fn nm_is_non_manifold_edge(face_count: u32) -> bool {
    face_count > 2
}

/// Detect boundary edge (belongs to exactly 1 face).
#[allow(dead_code)]
pub fn nm_is_boundary_edge(face_count: u32) -> bool {
    face_count == 1
}

/// Count non-manifold edges from a list of per-edge face counts.
#[allow(dead_code)]
pub fn nm_count_non_manifold_edges(face_counts: &[u32]) -> usize {
    face_counts
        .iter()
        .filter(|&&c| nm_is_non_manifold_edge(c))
        .count()
}

/// Count boundary edges.
#[allow(dead_code)]
pub fn nm_count_boundary_edges(face_counts: &[u32]) -> usize {
    face_counts
        .iter()
        .filter(|&&c| nm_is_boundary_edge(c))
        .count()
}

/// Mesh topology stats from edge face counts.
#[allow(dead_code)]
pub fn nm_topology_stats(face_counts: &[u32]) -> (usize, usize, usize) {
    let non_manifold = nm_count_non_manifold_edges(face_counts);
    let boundary = nm_count_boundary_edges(face_counts);
    let interior = face_counts.iter().filter(|&&c| c == 2).count();
    (non_manifold, boundary, interior)
}

#[allow(dead_code)]
pub fn nm_to_json(cfg: &NonManifoldConfig) -> String {
    format!(
        r#"{{"point_size":{:.4},"line_width":{:.4},"enabled":{}}}"#,
        cfg.point_size, cfg.line_width, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_non_manifold_config().enabled);
    }

    #[test]
    fn non_manifold_edge_three_faces() {
        assert!(nm_is_non_manifold_edge(3));
    }

    #[test]
    fn manifold_edge_two_faces() {
        assert!(!nm_is_non_manifold_edge(2));
    }

    #[test]
    fn boundary_edge_one_face() {
        assert!(nm_is_boundary_edge(1));
    }

    #[test]
    fn not_boundary_two_faces() {
        assert!(!nm_is_boundary_edge(2));
    }

    #[test]
    fn count_non_manifold() {
        let counts = vec![1u32, 2, 3, 2, 4];
        assert_eq!(nm_count_non_manifold_edges(&counts), 2);
    }

    #[test]
    fn count_boundary() {
        let counts = vec![1u32, 2, 1, 3];
        assert_eq!(nm_count_boundary_edges(&counts), 2);
    }

    #[test]
    fn topology_stats() {
        let counts = vec![2u32, 3, 1, 2, 1];
        let (nm, bd, int) = nm_topology_stats(&counts);
        assert_eq!(nm, 1);
        assert_eq!(bd, 2);
        assert_eq!(int, 2);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_non_manifold_config();
        nm_enable(&mut cfg);
        assert!(cfg.enabled);
        nm_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn to_json_has_point_size() {
        assert!(nm_to_json(&default_non_manifold_config()).contains("point_size"));
    }
}
