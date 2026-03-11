// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Catmull-Clark subdivision v2 with limit surface support.

/// Result of Catmull-Clark v2 subdivision.
#[allow(dead_code)]
pub struct CatmullClarkV2Result {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub level: u32,
}

/// Config for Catmull-Clark v2.
#[allow(dead_code)]
pub struct CatmullClarkV2Config {
    pub levels: u32,
    pub apply_limit_surface: bool,
}

#[allow(dead_code)]
impl Default for CatmullClarkV2Config {
    fn default() -> Self {
        Self { levels: 1, apply_limit_surface: false }
    }
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Compute the face point (centroid of face vertices).
#[allow(dead_code)]
pub fn face_point(verts: &[[f32; 3]]) -> [f32; 3] {
    let n = verts.len() as f32;
    let mut sum = [0.0f32; 3];
    for v in verts {
        sum = add3(sum, *v);
    }
    scale3(sum, 1.0 / n)
}

/// Compute the edge point for Catmull-Clark.
#[allow(dead_code)]
pub fn edge_point_cc_v2(p0: [f32; 3], p1: [f32; 3], f0: [f32; 3], f1: [f32; 3]) -> [f32; 3] {
    scale3(add3(add3(add3(p0, p1), f0), f1), 0.25)
}

/// Compute updated vertex point for Catmull-Clark.
#[allow(dead_code)]
pub fn vertex_point_cc_v2(
    p: [f32; 3],
    avg_face: [f32; 3],
    avg_edge_mid: [f32; 3],
    n: usize,
) -> [f32; 3] {
    let nf = n as f32;
    let term_f = scale3(avg_face, 1.0 / nf);
    let term_e = scale3(avg_edge_mid, 2.0 / nf);
    let term_p = scale3(p, (nf - 3.0) / nf);
    add3(add3(term_f, term_e), term_p)
}

/// Limit surface position for an interior vertex.
#[allow(dead_code)]
pub fn limit_position(center: [f32; 3], neighbours: &[[f32; 3]]) -> [f32; 3] {
    let n = neighbours.len() as f32;
    let mut nb_sum = [0.0f32; 3];
    for nb in neighbours {
        nb_sum = add3(nb_sum, *nb);
    }
    let w_c = n * n;
    let w_nb = 4.0;
    let denom = w_c + w_nb * n;
    let num = add3(scale3(center, w_c), scale3(nb_sum, w_nb));
    scale3(num, 1.0 / denom)
}

/// Simple one-step Catmull-Clark on quads: split each quad into 4 quads.
#[allow(dead_code)]
pub fn catmull_clark_v2_step(
    positions: &[[f32; 3]],
    quads: &[[u32; 4]],
) -> CatmullClarkV2Result {
    let mut new_pos = positions.to_vec();
    let mut new_quads: Vec<[u32; 4]> = Vec::with_capacity(quads.len() * 4);

    for quad in quads {
        let verts: Vec<[f32; 3]> = quad.iter().map(|&i| positions[i as usize]).collect();
        let fp = face_point(&verts);
        let fpi = new_pos.len() as u32;
        new_pos.push(fp);

        let mut edge_mids = [0u32; 4];
        for i in 0..4 {
            let a = verts[i];
            let b = verts[(i + 1) % 4];
            let em = scale3(add3(a, b), 0.5);
            edge_mids[i] = new_pos.len() as u32;
            new_pos.push(em);
        }

        new_quads.push([quad[0], edge_mids[0], fpi, edge_mids[3]]);
        new_quads.push([edge_mids[0], quad[1], edge_mids[1], fpi]);
        new_quads.push([fpi, edge_mids[1], quad[2], edge_mids[2]]);
        new_quads.push([edge_mids[3], fpi, edge_mids[2], quad[3]]);
    }

    let mut indices = Vec::with_capacity(new_quads.len() * 4);
    for q in &new_quads {
        indices.extend_from_slice(q);
    }

    CatmullClarkV2Result { positions: new_pos, indices, level: 1 }
}

/// Face count estimate after n levels.
#[allow(dead_code)]
pub fn cc_v2_face_count_estimate(initial_quads: usize, levels: u32) -> usize {
    initial_quads * 4usize.pow(levels)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_quad() -> (Vec<[f32; 3]>, Vec<[u32; 4]>) {
        let pos = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
        ];
        let quads = vec![[0, 1, 2, 3]];
        (pos, quads)
    }

    #[test]
    fn one_step_produces_4_quads() {
        let (p, q) = unit_quad();
        let r = catmull_clark_v2_step(&p, &q);
        assert_eq!(r.indices.len(), 16);
    }

    #[test]
    fn vertex_count_grows() {
        let (p, q) = unit_quad();
        let r = catmull_clark_v2_step(&p, &q);
        assert!(r.positions.len() > p.len());
    }

    #[test]
    fn face_point_is_centroid() {
        let verts = vec![[0.0,0.0,0.0],[2.0,0.0,0.0],[2.0,2.0,0.0],[0.0,2.0,0.0]];
        let fp = face_point(&verts);
        assert!((fp[0] - 1.0).abs() < 1e-5);
        assert!((fp[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn edge_point_cc_v2_midpoint_case() {
        let ep = edge_point_cc_v2(
            [0.0,0.0,0.0],[2.0,0.0,0.0],[1.0,1.0,0.0],[1.0,-1.0,0.0],
        );
        assert!((ep[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn vertex_point_interior() {
        let vp = vertex_point_cc_v2(
            [0.0,0.0,0.0],[0.0,0.0,0.0],[0.0,0.0,0.0],4,
        );
        assert!(vp[0].abs() < 1e-5);
    }

    #[test]
    fn limit_position_identical_neighbors() {
        let center = [1.0, 0.0, 0.0];
        let neighbors = vec![[1.0, 0.0, 0.0]; 4];
        let lp = limit_position(center, &neighbors);
        assert!((lp[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn face_count_estimate_two_levels() {
        assert_eq!(cc_v2_face_count_estimate(1, 2), 16);
    }

    #[test]
    fn level_field_is_one() {
        let (p, q) = unit_quad();
        let r = catmull_clark_v2_step(&p, &q);
        assert_eq!(r.level, 1);
    }

    #[test]
    fn default_config() {
        let c = CatmullClarkV2Config::default();
        assert_eq!(c.levels, 1);
        assert!(!c.apply_limit_surface);
    }
}
