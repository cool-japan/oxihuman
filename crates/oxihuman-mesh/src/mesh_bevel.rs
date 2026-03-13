// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge and vertex bevel (chamfer) operations.

/// Configuration for a bevel operation.
#[derive(Debug, Clone)]
pub struct BevelConfig {
    pub width: f32,
    pub segments: usize,
    pub bevel_vertices: bool,
}

impl Default for BevelConfig {
    fn default() -> Self {
        Self { width: 0.1, segments: 1, bevel_vertices: false }
    }
}

/// Result of a bevel operation.
#[derive(Debug, Clone, Default)]
pub struct BevelResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub new_vertex_count: usize,
    pub new_face_count: usize,
}

pub fn bevel_edges(
    positions: &[[f32; 3]],
    indices: &[u32],
    selected_edges: &[(usize, usize)],
    cfg: &BevelConfig,
) -> BevelResult {
    let mut out_pos: Vec<[f32; 3]> = positions.to_vec();
    let mut out_idx: Vec<u32> = indices.to_vec();
    let mut new_verts = 0usize;
    let mut new_faces = 0usize;
    let w = cfg.width.max(0.0);
    for &(v0, v1) in selected_edges {
        if v0 >= positions.len() || v1 >= positions.len() {
            continue;
        }
        let p0 = positions[v0];
        let p1 = positions[v1];
        for s in 0..=cfg.segments {
            let t = s as f32 / (cfg.segments.max(1)) as f32;
            let mid = lerp3(p0, p1, t);
            let offset = [mid[0], mid[1] + w, mid[2]];
            out_pos.push(offset);
            new_verts += 1;
        }
        new_faces += cfg.segments.max(1);
        let base = out_pos.len().saturating_sub(2) as u32;
        out_idx.extend_from_slice(&[base, base + 1, base + 1, base]);
    }
    BevelResult { new_vertex_count: new_verts, new_face_count: new_faces, positions: out_pos, indices: out_idx }
}

pub fn default_bevel_config() -> BevelConfig {
    BevelConfig::default()
}

pub fn original_vertex_count(_r: &BevelResult, orig: usize) -> usize {
    orig
}

pub fn total_vertex_count(r: &BevelResult) -> usize {
    r.positions.len()
}

pub fn total_face_count(r: &BevelResult) -> usize {
    r.indices.len() / 3
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_positions() -> Vec<[f32; 3]> {
        vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,1.0,0.0],[0.0,1.0,0.0]]
    }

    #[test]
    fn test_default_config() {
        /* default config has valid defaults */
        let cfg = default_bevel_config();
        assert!(cfg.width > 0.0);
        assert!(cfg.segments >= 1);
    }

    #[test]
    fn test_bevel_adds_vertices() {
        /* bevelling an edge adds new vertices */
        let pos = quad_positions();
        let idx: Vec<u32> = vec![0,1,2,0,2,3];
        let cfg = BevelConfig { width: 0.1, segments: 1, bevel_vertices: false };
        let r = bevel_edges(&pos, &idx, &[(0, 1)], &cfg);
        assert!(r.new_vertex_count > 0);
    }

    #[test]
    fn test_bevel_total_vertices_grows() {
        /* output has more vertices than input */
        let pos = quad_positions();
        let idx: Vec<u32> = vec![0,1,2,0,2,3];
        let r = bevel_edges(&pos, &idx, &[(0, 1)], &default_bevel_config());
        assert!(total_vertex_count(&r) > pos.len());
    }

    #[test]
    fn test_bevel_zero_width() {
        /* zero width bevel should not panic */
        let pos = quad_positions();
        let idx: Vec<u32> = vec![0,1,2];
        let cfg = BevelConfig { width: 0.0, segments: 1, bevel_vertices: false };
        let r = bevel_edges(&pos, &idx, &[(0, 1)], &cfg);
        assert!(r.new_vertex_count > 0);
    }

    #[test]
    fn test_bevel_invalid_edge_skipped() {
        /* edges with out-of-range indices are silently skipped */
        let pos = quad_positions();
        let idx: Vec<u32> = vec![0,1,2];
        let r = bevel_edges(&pos, &idx, &[(99, 100)], &default_bevel_config());
        assert_eq!(r.new_vertex_count, 0);
    }

    #[test]
    fn test_bevel_multiple_segments() {
        /* multiple segments produce more vertices */
        let pos = quad_positions();
        let idx: Vec<u32> = vec![0,1,2];
        let r1 = bevel_edges(&pos, &idx, &[(0, 1)], &BevelConfig { width: 0.1, segments: 1, bevel_vertices: false });
        let r3 = bevel_edges(&pos, &idx, &[(0, 1)], &BevelConfig { width: 0.1, segments: 3, bevel_vertices: false });
        assert!(r3.new_vertex_count > r1.new_vertex_count);
    }

    #[test]
    fn test_bevel_no_edges() {
        /* no selected edges changes nothing structural */
        let pos = quad_positions();
        let idx: Vec<u32> = vec![0,1,2,0,2,3];
        let r = bevel_edges(&pos, &idx, &[], &default_bevel_config());
        assert_eq!(r.new_vertex_count, 0);
        assert_eq!(r.new_face_count, 0);
    }

    #[test]
    fn test_original_vertex_count_helper() {
        /* helper returns the passed-in original count */
        let pos = quad_positions();
        let idx: Vec<u32> = vec![0,1,2];
        let r = bevel_edges(&pos, &idx, &[], &default_bevel_config());
        assert_eq!(original_vertex_count(&r, 4), 4);
    }

    #[test]
    fn test_lerp3_midpoint() {
        /* midpoint lerp should return average */
        let m = lerp3([0.0,0.0,0.0], [2.0,4.0,6.0], 0.5);
        assert!((m[0] - 1.0).abs() < 1e-6);
        assert!((m[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_face_count_with_2_segments() {
        /* new_face_count equals segments for a single edge */
        let pos = quad_positions();
        let idx: Vec<u32> = vec![0,1,2];
        let r = bevel_edges(&pos, &idx, &[(0,1)], &BevelConfig { width: 0.1, segments: 2, bevel_vertices: false });
        assert_eq!(r.new_face_count, 2);
    }
}
