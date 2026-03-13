// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Skin modifier — generates a mesh from edges/vertices.

/// A vertex in the skin modifier with radius parameters.
#[derive(Debug, Clone)]
pub struct SkinVertex {
    pub position: [f32; 3],
    pub radius: [f32; 2], /* X and Y radius */
    pub is_root: bool,
    pub use_smooth: bool,
}

impl SkinVertex {
    pub fn new(position: [f32; 3], radius_x: f32, radius_y: f32) -> Self {
        Self {
            position,
            radius: [radius_x, radius_y],
            is_root: false,
            use_smooth: true,
        }
    }

    pub fn as_root(mut self) -> Self {
        self.is_root = true;
        self
    }
}

/// Configuration for the skin modifier.
#[derive(Debug, Clone)]
pub struct SkinConfig {
    pub branch_smoothing: f32,
    pub use_smooth_shade: bool,
    pub use_x_symmetry: bool,
    pub use_y_symmetry: bool,
    pub use_z_symmetry: bool,
}

impl Default for SkinConfig {
    fn default() -> Self {
        Self {
            branch_smoothing: 0.0,
            use_smooth_shade: true,
            use_x_symmetry: false,
            use_y_symmetry: false,
            use_z_symmetry: false,
        }
    }
}

/// Result of skin modifier application.
#[derive(Debug, Clone, Default)]
pub struct SkinResult {
    pub vertex_count: usize,
    pub face_count: usize,
    pub branch_count: usize,
}

/// Compute cross-sectional area at a skin vertex.
pub fn skin_vertex_area(sv: &SkinVertex) -> f32 {
    std::f32::consts::PI * sv.radius[0] * sv.radius[1]
}

/// Estimate the skin mesh vertex count from a skeleton edge list.
pub fn estimate_skin_vertex_count(skin_vertices: &[SkinVertex], edges: &[(usize, usize)]) -> usize {
    skin_vertices.len() * 8 + edges.len() * 4
}

/// Find root vertices (entry points for the skin).
pub fn find_root_vertices(skin_vertices: &[SkinVertex]) -> Vec<usize> {
    skin_vertices
        .iter()
        .enumerate()
        .filter(|(_, sv)| sv.is_root)
        .map(|(i, _)| i)
        .collect()
}

/// Apply skin modifier stub.
pub fn apply_skin(
    skin_vertices: &[SkinVertex],
    edges: &[(usize, usize)],
    cfg: &SkinConfig,
) -> SkinResult {
    let _ = cfg;
    let vc = estimate_skin_vertex_count(skin_vertices, edges);
    SkinResult {
        vertex_count: vc,
        face_count: vc / 2,
        branch_count: find_root_vertices(skin_vertices).len(),
    }
}

/// Validate skin modifier inputs.
pub fn validate_skin_input(skin_vertices: &[SkinVertex], edges: &[(usize, usize)]) -> bool {
    !skin_vertices.is_empty()
        && edges
            .iter()
            .all(|&(a, b)| a < skin_vertices.len() && b < skin_vertices.len())
}

// ---- New API required by lib.rs ----

use std::f32::consts::TAU;

/// Simpler skin vertex for the new API.
pub struct SkinVertexNew {
    pub pos: [f32; 3],
    pub radius: f32,
}

pub fn new_skin_vertex(pos: [f32; 3], radius: f32) -> SkinVertexNew {
    SkinVertexNew {
        pos,
        radius: radius.max(0.0),
    }
}

pub fn skin_segment_verts(a: &SkinVertexNew, b: &SkinVertexNew, sides: usize) -> Vec<[f32; 3]> {
    let sides = sides.max(3);
    let center = [
        (a.pos[0] + b.pos[0]) * 0.5,
        (a.pos[1] + b.pos[1]) * 0.5,
        (a.pos[2] + b.pos[2]) * 0.5,
    ];
    let r = (a.radius + b.radius) * 0.5;
    let mut verts = Vec::with_capacity(sides);
    for s in 0..sides {
        let angle = TAU * s as f32 / sides as f32;
        verts.push([
            center[0] + r * angle.cos(),
            center[1] + r * angle.sin(),
            center[2],
        ]);
    }
    verts
}

pub fn skin_segment_length(a: &SkinVertexNew, b: &SkinVertexNew) -> f32 {
    let dx = b.pos[0] - a.pos[0];
    let dy = b.pos[1] - a.pos[1];
    let dz = b.pos[2] - a.pos[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub fn skin_cap_center(v: &SkinVertexNew) -> [f32; 3] {
    v.pos
}

pub fn skin_total_volume(verts: &[SkinVertexNew]) -> f32 {
    let n = verts.len();
    if n < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    for i in 0..n - 1 {
        let len = skin_segment_length(&verts[i], &verts[i + 1]);
        let r = (verts[i].radius + verts[i + 1].radius) * 0.5;
        total += std::f32::consts::PI * r * r * len;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_vertex_new() {
        let sv = SkinVertex::new([0.0; 3], 0.5, 0.5);
        assert!((sv.radius[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_skin_vertex_as_root() {
        let sv = SkinVertex::new([0.0; 3], 0.1, 0.1).as_root();
        assert!(sv.is_root);
    }

    #[test]
    fn test_skin_vertex_area() {
        let sv = SkinVertex::new([0.0; 3], 1.0, 1.0);
        let a = skin_vertex_area(&sv);
        assert!((a - std::f32::consts::PI).abs() < 1e-4);
    }

    #[test]
    fn test_estimate_skin_vertex_count() {
        let sverts = vec![SkinVertex::new([0.0; 3], 0.1, 0.1); 3];
        let edges = vec![(0, 1), (1, 2)];
        let cnt = estimate_skin_vertex_count(&sverts, &edges);
        assert_eq!(cnt, 3 * 8 + 2 * 4);
    }

    #[test]
    fn test_find_root_vertices_none() {
        let sverts = vec![SkinVertex::new([0.0; 3], 0.1, 0.1); 3];
        assert!(find_root_vertices(&sverts).is_empty());
    }

    #[test]
    fn test_find_root_vertices_one() {
        let sverts = vec![
            SkinVertex::new([0.0; 3], 0.1, 0.1).as_root(),
            SkinVertex::new([1.0; 3], 0.1, 0.1),
        ];
        assert_eq!(find_root_vertices(&sverts), vec![0]);
    }

    #[test]
    fn test_apply_skin_returns_result() {
        let sverts = vec![SkinVertex::new([0.0; 3], 0.1, 0.1)];
        let edges: Vec<(usize, usize)> = vec![];
        let cfg = SkinConfig::default();
        let res = apply_skin(&sverts, &edges, &cfg);
        assert_eq!(res.vertex_count, 8);
    }

    #[test]
    fn test_validate_skin_input_valid() {
        let sverts = vec![SkinVertex::new([0.0; 3], 0.1, 0.1); 3];
        let edges = vec![(0, 1), (1, 2)];
        assert!(validate_skin_input(&sverts, &edges));
    }

    #[test]
    fn test_validate_skin_input_oob_edge() {
        let sverts = vec![SkinVertex::new([0.0; 3], 0.1, 0.1)];
        let edges = vec![(0, 5)];
        assert!(!validate_skin_input(&sverts, &edges));
    }

    #[test]
    fn test_validate_skin_input_empty() {
        assert!(!validate_skin_input(&[], &[]));
    }
}
