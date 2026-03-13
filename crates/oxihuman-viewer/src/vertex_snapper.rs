// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! VertexSnapper — find nearest vertex/edge/face for interactive snapping.

#![allow(dead_code)]

/// Result of a snap operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SnapResult {
    pub snapped: bool,
    pub position: [f32; 3],
    pub distance: f32,
    pub vertex_index: Option<usize>,
}

/// Vertex snapper configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexSnapper {
    pub threshold: f32,
    pub enabled: bool,
}

impl Default for VertexSnapper {
    fn default() -> Self {
        VertexSnapper { threshold: 0.05, enabled: true }
    }
}

/// Create a new `VertexSnapper` with the given threshold.
#[allow(dead_code)]
pub fn new_vertex_snapper(threshold: f32) -> VertexSnapper {
    VertexSnapper { threshold, enabled: true }
}

/// Return the snap threshold.
#[allow(dead_code)]
pub fn snap_threshold(snapper: &VertexSnapper) -> f32 {
    snapper.threshold
}

/// Find the nearest vertex to `query` in `vertices` (flat [x,y,z,...]).
#[allow(dead_code)]
pub fn find_snap_target(snapper: &VertexSnapper, vertices: &[[f32; 3]], query: [f32; 3]) -> SnapResult {
    if !snapper.enabled || vertices.is_empty() {
        return SnapResult { snapped: false, position: query, distance: f32::INFINITY, vertex_index: None };
    }
    let result = vertices
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let d = ((v[0] - query[0]).powi(2) + (v[1] - query[1]).powi(2) + (v[2] - query[2]).powi(2)).sqrt();
            (i, d)
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let Some((idx, dist)) = result else {
        return SnapResult { snapped: false, position: query, distance: f32::INFINITY, vertex_index: None };
    };
    if dist <= snapper.threshold {
        SnapResult { snapped: true, position: vertices[idx], distance: dist, vertex_index: Some(idx) }
    } else {
        SnapResult { snapped: false, position: query, distance: dist, vertex_index: None }
    }
}

/// Snap the query point to the nearest vertex.
#[allow(dead_code)]
pub fn snap_to_vertex(snapper: &VertexSnapper, vertices: &[[f32; 3]], query: [f32; 3]) -> [f32; 3] {
    find_snap_target(snapper, vertices, query).position
}

/// Snap to the midpoint of the nearest edge.
#[allow(dead_code)]
pub fn snap_to_edge_midpoint(edges: &[[usize; 2]], vertices: &[[f32; 3]], query: [f32; 3]) -> [f32; 3] {
    if edges.is_empty() || vertices.is_empty() {
        return query;
    }
    let (best_mid, _) = edges
        .iter()
        .filter_map(|e| {
            let a = *vertices.get(e[0])?;
            let b = *vertices.get(e[1])?;
            let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0];
            let d = ((mid[0] - query[0]).powi(2) + (mid[1] - query[1]).powi(2) + (mid[2] - query[2]).powi(2)).sqrt();
            Some((mid, d))
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or((query, f32::INFINITY));
    best_mid
}

/// Snap to the centroid of the nearest triangle face.
#[allow(dead_code)]
pub fn snap_to_face_center(faces: &[[usize; 3]], vertices: &[[f32; 3]], query: [f32; 3]) -> [f32; 3] {
    if faces.is_empty() || vertices.is_empty() {
        return query;
    }
    let (best, _) = faces
        .iter()
        .filter_map(|f| {
            let a = *vertices.get(f[0])?;
            let b = *vertices.get(f[1])?;
            let c = *vertices.get(f[2])?;
            let cen = [(a[0]+b[0]+c[0])/3.0, (a[1]+b[1]+c[1])/3.0, (a[2]+b[2]+c[2])/3.0];
            let d = ((cen[0]-query[0]).powi(2)+(cen[1]-query[1]).powi(2)+(cen[2]-query[2]).powi(2)).sqrt();
            Some((cen, d))
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or((query, f32::INFINITY));
    best
}

/// Return whether snapping is enabled.
#[allow(dead_code)]
pub fn snapper_enabled(snapper: &VertexSnapper) -> bool {
    snapper.enabled
}

/// Return the snap distance for a given `SnapResult`.
#[allow(dead_code)]
pub fn snap_distance(result: &SnapResult) -> f32 {
    result.distance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_snapper_enabled() {
        let s = new_vertex_snapper(0.1);
        assert!(snapper_enabled(&s));
        assert!((snap_threshold(&s) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_snap_to_vertex_hits() {
        let s = new_vertex_snapper(0.5);
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let result = find_snap_target(&s, &verts, [0.1, 0.0, 0.0]);
        assert!(result.snapped);
        assert_eq!(result.vertex_index, Some(0));
    }

    #[test]
    fn test_snap_to_vertex_misses() {
        let s = new_vertex_snapper(0.05);
        let verts = vec![[0.0, 0.0, 0.0]];
        let result = find_snap_target(&s, &verts, [1.0, 0.0, 0.0]);
        assert!(!result.snapped);
    }

    #[test]
    fn test_snap_to_vertex_returns_position() {
        let s = new_vertex_snapper(1.0);
        let verts = vec![[5.0, 0.0, 0.0]];
        let pos = snap_to_vertex(&s, &verts, [4.9, 0.0, 0.0]);
        assert!((pos[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_snap_empty_vertices() {
        let s = new_vertex_snapper(0.1);
        let result = find_snap_target(&s, &[], [0.0, 0.0, 0.0]);
        assert!(!result.snapped);
    }

    #[test]
    fn test_snap_to_edge_midpoint() {
        let verts = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let edges = vec![[0usize, 1usize]];
        let mid = snap_to_edge_midpoint(&edges, &verts, [1.0, 0.0, 0.0]);
        assert!((mid[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_snap_to_face_center() {
        let verts = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
        let faces = vec![[0usize, 1, 2]];
        let cen = snap_to_face_center(&faces, &verts, [1.0, 1.0, 0.0]);
        assert!((cen[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_snap_distance() {
        let r = SnapResult { snapped: false, position: [0.0; 3], distance: 3.5, vertex_index: None };
        assert!((snap_distance(&r) - 3.5).abs() < 1e-6);
    }
}
