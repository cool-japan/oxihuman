// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Solidify modifier — create a thin-shell copy of a mesh by offsetting along normals.

/// Configuration for the solidify operation.
#[derive(Debug, Clone)]
pub struct SolidifyConfig {
    pub thickness: f32,
    /// Offset factor [0, 1]; 0 = only negative side, 1 = only positive.
    pub offset: f32,
    pub fill_rim: bool,
}

impl Default for SolidifyConfig {
    fn default() -> Self {
        Self {
            thickness: 0.05,
            offset: 0.5,
            fill_rim: true,
        }
    }
}

/// Result of the solidify operation.
#[derive(Debug, Clone, Default)]
pub struct SolidifyResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub rim_face_count: usize,
}

/// Solidify a mesh by extruding a shell along per-vertex normals.
pub fn solidify(
    positions: &[[f32; 3]],
    indices: &[u32],
    normals: &[[f32; 3]],
    cfg: &SolidifyConfig,
) -> SolidifyResult {
    let nv = positions.len();
    let min_n = nv.min(normals.len());
    let mut out_pos: Vec<[f32; 3]> = Vec::with_capacity(nv * 2);
    let t = cfg.thickness;
    let off = cfg.offset.clamp(0.0, 1.0);
    for i in 0..nv {
        let p = positions[i];
        let n = if i < min_n {
            normals[i]
        } else {
            [0.0, 1.0, 0.0]
        };
        let top = [
            p[0] + n[0] * t * off,
            p[1] + n[1] * t * off,
            p[2] + n[2] * t * off,
        ];
        out_pos.push(top);
    }
    /* bottom shell — mirror offset */
    for i in 0..nv {
        let p = positions[i];
        let n = if i < min_n {
            normals[i]
        } else {
            [0.0, 1.0, 0.0]
        };
        let bot = [
            p[0] - n[0] * t * (1.0 - off),
            p[1] - n[1] * t * (1.0 - off),
            p[2] - n[2] * t * (1.0 - off),
        ];
        out_pos.push(bot);
    }
    /* top shell indices (original winding) */
    let mut out_idx: Vec<u32> = indices.to_vec();
    /* bottom shell indices (flipped winding) */
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        out_idx.extend_from_slice(&[tri[2] + nv as u32, tri[1] + nv as u32, tri[0] + nv as u32]);
    }
    /* rim: connect boundary edges between top and bottom shells */
    let mut rim_faces = 0usize;
    if cfg.fill_rim {
        let boundary = find_boundary_edges(indices);
        for (a, b) in boundary {
            let ta = a as u32;
            let tb = b as u32;
            let ba = a as u32 + nv as u32;
            let bb = b as u32 + nv as u32;
            out_idx.extend_from_slice(&[ta, tb, bb]);
            out_idx.extend_from_slice(&[ta, bb, ba]);
            rim_faces += 2;
        }
    }
    SolidifyResult {
        positions: out_pos,
        indices: out_idx,
        rim_face_count: rim_faces,
    }
}

/// Default solidify configuration.
pub fn default_solidify_config() -> SolidifyConfig {
    SolidifyConfig::default()
}

/// Total vertex count in solidify result.
pub fn solidify_vertex_count(r: &SolidifyResult) -> usize {
    r.positions.len()
}

/// Total triangle count in solidify result.
pub fn solidify_triangle_count(r: &SolidifyResult) -> usize {
    r.indices.len() / 3
}

/// Shell thickness from config.
pub fn shell_thickness(cfg: &SolidifyConfig) -> f32 {
    cfg.thickness
}

// ---- New API required by lib.rs ----

/// Simple solidify parameters (new API).
pub struct SolidifyParams {
    pub thickness: f32,
    pub offset: f32,
    pub fill_rim: bool,
}

pub fn new_solidify_params(thickness: f32) -> SolidifyParams {
    SolidifyParams {
        thickness: thickness.max(0.0),
        offset: 0.0,
        fill_rim: true,
    }
}

/// Return the inner and outer vertices for a given vertex, normal and params.
pub fn solidify_vertex(
    pos: [f32; 3],
    normal: [f32; 3],
    params: &SolidifyParams,
) -> ([f32; 3], [f32; 3]) {
    let t = params.thickness;
    let off = params.offset;
    let inner = [
        pos[0] - normal[0] * t * (1.0 - off),
        pos[1] - normal[1] * t * (1.0 - off),
        pos[2] - normal[2] * t * (1.0 - off),
    ];
    let outer = [
        pos[0] + normal[0] * t * off,
        pos[1] + normal[1] * t * off,
        pos[2] + normal[2] * t * off,
    ];
    (inner, outer)
}

/// Face count estimate (2x original + rim quads if fill_rim).
pub fn solidify_face_count(input_face_count: usize, params: &SolidifyParams) -> usize {
    if params.fill_rim {
        input_face_count * 2 + input_face_count
    } else {
        input_face_count * 2
    }
}

pub fn solidify_flip_normal(n: [f32; 3]) -> [f32; 3] {
    [-n[0], -n[1], -n[2]]
}

pub fn solidify_vert_count(input_vertex_count: usize) -> usize {
    input_vertex_count * 2
}

fn find_boundary_edges(indices: &[u32]) -> Vec<(usize, usize)> {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        for &(u, v) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let key = (u.min(v) as usize, u.max(v) as usize);
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count
        .into_iter()
        .filter(|(_, c)| *c == 1)
        .map(|(e, _)| e)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_tri() -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 3]>) {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let i = vec![0u32, 1, 2];
        let n = vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        (p, i, n)
    }

    #[test]
    fn test_solidify_doubles_vertices() {
        /* solidify doubles the vertex count */
        let (p, i, n) = flat_tri();
        let r = solidify(&p, &i, &n, &default_solidify_config());
        assert_eq!(solidify_vertex_count(&r), p.len() * 2);
    }

    #[test]
    fn test_solidify_min_triangle_count() {
        /* at minimum two shells = 2 original tris */
        let (p, i, n) = flat_tri();
        let r = solidify(
            &p,
            &i,
            &n,
            &SolidifyConfig {
                thickness: 0.1,
                offset: 0.5,
                fill_rim: false,
            },
        );
        assert!(solidify_triangle_count(&r) >= 2);
    }

    #[test]
    fn test_solidify_with_rim() {
        /* rim adds extra faces */
        let (p, i, n) = flat_tri();
        let no_rim = solidify(
            &p,
            &i,
            &n,
            &SolidifyConfig {
                thickness: 0.1,
                offset: 0.5,
                fill_rim: false,
            },
        );
        let with_rim = solidify(&p, &i, &n, &default_solidify_config());
        assert!(solidify_triangle_count(&with_rim) >= solidify_triangle_count(&no_rim));
    }

    #[test]
    fn test_shell_thickness_accessor() {
        /* shell_thickness reads config field */
        let cfg = SolidifyConfig {
            thickness: 0.25,
            offset: 0.5,
            fill_rim: false,
        };
        assert!((shell_thickness(&cfg) - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_default_config_positive_thickness() {
        /* default thickness is positive */
        assert!(default_solidify_config().thickness > 0.0);
    }

    #[test]
    fn test_solidify_empty_indices() {
        /* empty indices still doubles vertices */
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let i: Vec<u32> = vec![];
        let n = vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let r = solidify(&p, &i, &n, &default_solidify_config());
        assert_eq!(solidify_vertex_count(&r), 4);
    }

    #[test]
    fn test_top_shell_pos_above_original() {
        /* top shell position is offset above original when offset=1 */
        let p = vec![[0.0, 0.0, 0.0]];
        let n = vec![[0.0, 1.0, 0.0]];
        let cfg = SolidifyConfig {
            thickness: 0.1,
            offset: 1.0,
            fill_rim: false,
        };
        let r = solidify(&p, &[], &n, &cfg);
        assert!(r.positions[0][1] > p[0][1]);
    }

    #[test]
    fn test_solidify_rim_count_positive() {
        /* rim face count is recorded */
        let (p, i, n) = flat_tri();
        let r = solidify(&p, &i, &n, &default_solidify_config());
        assert!(r.rim_face_count > 0);
    }

    #[test]
    fn test_find_boundary_edges_single_tri() {
        /* single triangle has 3 boundary edges */
        let i: Vec<u32> = vec![0, 1, 2];
        let be = find_boundary_edges(&i);
        assert_eq!(be.len(), 3);
    }
}
