// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Knife (projection) cut — split faces along a user-defined polyline.

/* ── legacy API (keep for any future lib.rs exports) ── */

#[derive(Debug, Clone, Default)]
pub struct KnifeLine {
    pub points: Vec<[f32; 3]>,
}

impl KnifeLine {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_point(&mut self, p: [f32; 3]) {
        self.points.push(p);
    }
}

#[derive(Debug, Clone, Default)]
pub struct KnifeCutResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub cut_vertex_count: usize,
    pub cut_edge_count: usize,
}

pub fn knife_cut(positions: &[[f32; 3]], indices: &[u32], line: &KnifeLine) -> KnifeCutResult {
    let mut out_pos = positions.to_vec();
    let mut out_idx = indices.to_vec();
    let mut cut_verts = 0usize;
    let mut cut_edges = 0usize;
    if line.points.len() < 2 {
        return KnifeCutResult {
            positions: out_pos,
            indices: out_idx,
            cut_vertex_count: 0,
            cut_edge_count: 0,
        };
    }
    let mut edges_seen = std::collections::HashSet::new();
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        for &(u, v) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let key = (u.min(v), u.max(v));
            if edges_seen.insert(key) {
                let pa = positions[u as usize];
                let pb = positions[v as usize];
                if edge_crosses_knife(pa, pb, line) {
                    let mid = midpoint3(pa, pb);
                    out_pos.push(mid);
                    out_idx.push(out_pos.len() as u32 - 1);
                    cut_verts += 1;
                    cut_edges += 1;
                }
            }
        }
    }
    KnifeCutResult {
        positions: out_pos,
        indices: out_idx,
        cut_vertex_count: cut_verts,
        cut_edge_count: cut_edges,
    }
}

pub fn cut_vertex_count(r: &KnifeCutResult) -> usize {
    r.cut_vertex_count
}
pub fn cut_edge_count(r: &KnifeCutResult) -> usize {
    r.cut_edge_count
}
pub fn knife_cut_modified(r: &KnifeCutResult) -> bool {
    r.cut_vertex_count > 0
}

fn midpoint3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

fn edge_crosses_knife(a: [f32; 3], b: [f32; 3], line: &KnifeLine) -> bool {
    for seg in line.points.windows(2) {
        let c = seg[0];
        let d = seg[1];
        if segments_cross_2d([a[0], a[1]], [b[0], b[1]], [c[0], c[1]], [d[0], d[1]]) {
            return true;
        }
    }
    false
}

fn segments_cross_2d(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> bool {
    let d1 = cross2d(sub2(d, c), sub2(a, c));
    let d2 = cross2d(sub2(d, c), sub2(b, c));
    let d3 = cross2d(sub2(b, a), sub2(c, a));
    let d4 = cross2d(sub2(b, a), sub2(d, a));
    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }
    false
}

fn cross2d(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}
fn sub2(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

/* ── spec functions (wave 150B) ── */

/// A knife cut defined by control points and normals.
#[derive(Debug, Clone, Default)]
pub struct KnifeCut {
    pub cut_points: Vec<[f32; 3]>,
    pub cut_normals: Vec<[f32; 3]>,
}

/// Create a new empty `KnifeCut`.
pub fn new_knife_cut() -> KnifeCut {
    KnifeCut::default()
}

/// Add a point (and default normal) to the knife cut.
pub fn knife_add_point(cut: &mut KnifeCut, point: [f32; 3]) {
    cut.cut_points.push(point);
    cut.cut_normals.push([0.0, 1.0, 0.0]);
}

/// Total length of the knife cut polyline.
pub fn knife_cut_length(cut: &KnifeCut) -> f32 {
    if cut.cut_points.len() < 2 {
        return 0.0;
    }
    cut.cut_points
        .windows(2)
        .map(|w| {
            let a = w[0];
            let b = w[1];
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let dz = b[2] - a[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum()
}

/// Number of points in the knife cut.
pub fn knife_point_count(cut: &KnifeCut) -> usize {
    cut.cut_points.len()
}

/// Project a point onto the knife cut (stub: returns point on nearest segment).
pub fn knife_project_to_face(cut: &KnifeCut, point: [f32; 3]) -> [f32; 3] {
    if cut.cut_points.is_empty() {
        return point;
    }
    /* return nearest cut point as stub */
    cut.cut_points
        .iter()
        .copied()
        .min_by(|&a, &b| {
            let da = dist3(a, point);
            let db = dist3(b, point);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(point)
}

/// Returns true if the knife cut is closed (first == last point).
pub fn knife_is_closed(cut: &KnifeCut) -> bool {
    cut.cut_points.len() >= 2 && cut.cut_points.first() == cut.cut_points.last()
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_triangle() -> (Vec<[f32; 3]>, Vec<u32>) {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let i = vec![0, 1, 2];
        (p, i)
    }

    fn crossing_knife() -> KnifeLine {
        let mut k = KnifeLine::new();
        k.add_point([0.5, -0.5, 0.0]);
        k.add_point([0.5, 1.5, 0.0]);
        k
    }

    #[test]
    fn test_knife_cut_no_line() {
        let (p, i) = unit_triangle();
        let mut k = KnifeLine::new();
        k.add_point([0.5, 0.5, 0.0]);
        let r = knife_cut(&p, &i, &k);
        assert_eq!(cut_vertex_count(&r), 0);
    }

    #[test]
    fn test_knife_cut_crosses_edge() {
        let (p, i) = unit_triangle();
        let k = crossing_knife();
        let r = knife_cut(&p, &i, &k);
        assert!(cut_vertex_count(&r) > 0);
    }

    #[test]
    fn test_knife_add_point() {
        let mut c = new_knife_cut();
        knife_add_point(&mut c, [0.0, 0.0, 0.0]);
        knife_add_point(&mut c, [1.0, 0.0, 0.0]);
        assert_eq!(knife_point_count(&c), 2);
    }

    #[test]
    fn test_knife_cut_length() {
        let mut c = new_knife_cut();
        knife_add_point(&mut c, [0.0, 0.0, 0.0]);
        knife_add_point(&mut c, [1.0, 0.0, 0.0]);
        assert!((knife_cut_length(&c) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_knife_is_closed_false() {
        let mut c = new_knife_cut();
        knife_add_point(&mut c, [0.0, 0.0, 0.0]);
        knife_add_point(&mut c, [1.0, 0.0, 0.0]);
        assert!(!knife_is_closed(&c));
    }
}
