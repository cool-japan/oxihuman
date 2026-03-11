// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sharp/crease edge marking for subdivision and rendering.

/// A marked sharp edge between two vertex indices.
#[derive(Debug, Clone, PartialEq)]
pub struct SharpEdgeMark {
    pub v0: usize,
    pub v1: usize,
    /// Sharpness in [0.0, 1.0]; 1.0 = fully sharp.
    pub sharpness: f32,
}

/// Collection of sharp edge marks.
#[derive(Debug, Clone, Default)]
pub struct SharpEdgeSet {
    pub edges: Vec<SharpEdgeMark>,
}

impl SharpEdgeSet {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Mark a single edge as sharp with the given sharpness.
pub fn mark_sharp_edge(set: &mut SharpEdgeSet, v0: usize, v1: usize, sharpness: f32) {
    let s = sharpness.clamp(0.0, 1.0);
    set.edges.push(SharpEdgeMark {
        v0: v0.min(v1),
        v1: v0.max(v1),
        sharpness: s,
    });
}

/// Remove all sharp marks for a given edge (by vertex pair).
pub fn unmark_sharp_edge(set: &mut SharpEdgeSet, v0: usize, v1: usize) {
    let a = v0.min(v1);
    let b = v0.max(v1);
    set.edges.retain(|e| !(e.v0 == a && e.v1 == b));
}

/// Return the sharpness of an edge, or `None` if not marked.
pub fn get_sharpness(set: &SharpEdgeSet, v0: usize, v1: usize) -> Option<f32> {
    let a = v0.min(v1);
    let b = v0.max(v1);
    set.edges.iter().find(|e| e.v0 == a && e.v1 == b).map(|e| e.sharpness)
}

/// Detect sharp edges from a triangle mesh by dihedral angle threshold (degrees).
pub fn detect_sharp_edges_by_angle(
    positions: &[[f32; 3]],
    indices: &[u32],
    threshold_deg: f32,
) -> SharpEdgeSet {
    use std::collections::HashMap;
    let mut edge_faces: HashMap<(usize, usize), Vec<[f32; 3]>> = HashMap::new();
    let thresh = threshold_deg.to_radians();
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let n = face_normal_se(positions, a, b, c);
        for &(u, v) in &[(a, b), (b, c), (c, a)] {
            let key = (u.min(v), u.max(v));
            edge_faces.entry(key).or_default().push(n);
        }
    }
    let mut set = SharpEdgeSet::new();
    for ((v0, v1), normals) in &edge_faces {
        if normals.len() < 2 {
            continue;
        }
        let angle = angle_between_normals(normals[0], normals[1]);
        if angle > thresh {
            let sharpness = (angle / std::f32::consts::PI).clamp(0.0, 1.0);
            set.edges.push(SharpEdgeMark { v0: *v0, v1: *v1, sharpness });
        }
    }
    set
}

/// Total count of sharp edges in the set.
pub fn sharp_edge_count(set: &SharpEdgeSet) -> usize {
    set.edges.len()
}

/// Average sharpness across all marked edges.
pub fn avg_sharpness(set: &SharpEdgeSet) -> f32 {
    if set.edges.is_empty() {
        return 0.0;
    }
    let sum: f32 = set.edges.iter().map(|e| e.sharpness).sum();
    sum / set.edges.len() as f32
}

fn face_normal_se(pos: &[[f32; 3]], a: usize, b: usize, c: usize) -> [f32; 3] {
    let pa = pos[a];
    let pb = pos[b];
    let pc = pos[c];
    let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
    let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-12 { n } else { [n[0] / len, n[1] / len, n[2] / len] }
}

fn angle_between_normals(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
    dot.acos()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_set() -> SharpEdgeSet {
        let mut s = SharpEdgeSet::new();
        mark_sharp_edge(&mut s, 0, 1, 1.0);
        mark_sharp_edge(&mut s, 1, 2, 0.5);
        s
    }

    #[test]
    fn test_mark_adds_edge() {
        /* marking an edge should appear in the set */
        let s = simple_set();
        assert_eq!(sharp_edge_count(&s), 2);
    }

    #[test]
    fn test_get_sharpness_found() {
        /* sharpness should round-trip */
        let s = simple_set();
        assert!((get_sharpness(&s, 0, 1).unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_get_sharpness_not_found() {
        /* unmarked edge returns None */
        let s = simple_set();
        assert!(get_sharpness(&s, 5, 6).is_none());
    }

    #[test]
    fn test_unmark_removes_edge() {
        /* after unmarking the count drops */
        let mut s = simple_set();
        unmark_sharp_edge(&mut s, 0, 1);
        assert_eq!(sharp_edge_count(&s), 1);
    }

    #[test]
    fn test_avg_sharpness() {
        /* average should be 0.75 */
        let s = simple_set();
        assert!((avg_sharpness(&s) - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_empty_set() {
        /* empty set has zero count and zero avg */
        let s = SharpEdgeSet::new();
        assert_eq!(sharp_edge_count(&s), 0);
        assert!((avg_sharpness(&s)).abs() < 1e-6);
    }

    #[test]
    fn test_canonical_order() {
        /* marks are stored with v0 <= v1 */
        let mut s = SharpEdgeSet::new();
        mark_sharp_edge(&mut s, 3, 1, 0.8);
        assert!(s.edges[0].v0 <= s.edges[0].v1);
    }

    #[test]
    fn test_sharpness_clamped() {
        /* values out of range are clamped to [0,1] */
        let mut s = SharpEdgeSet::new();
        mark_sharp_edge(&mut s, 0, 1, 5.0);
        assert!((get_sharpness(&s, 0, 1).unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_detect_sharp_edges_basic() {
        /* two tris sharing an edge at 90 deg should be detected */
        let positions: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let indices: Vec<u32> = vec![0, 1, 2, 0, 1, 3];
        let set = detect_sharp_edges_by_angle(&positions, &indices, 30.0);
        assert!(sharp_edge_count(&set) > 0);
    }
}
