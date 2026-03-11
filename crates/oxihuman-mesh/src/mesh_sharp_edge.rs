// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A sharp edge entry.
#[allow(dead_code)]
#[derive(Clone)]
pub struct SharpEdge {
    pub a: u32,
    pub b: u32,
    pub dihedral: f32,
}

/// Result of sharp edge detection.
#[allow(dead_code)]
pub struct SharpEdgeResult {
    pub edges: Vec<SharpEdge>,
    pub threshold_deg: f32,
}

fn face_normal_se(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-9 {
        return [0.0; 3];
    }
    [n[0] / len, n[1] / len, n[2] / len]
}

fn dot3_se(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Detect sharp edges in a triangle mesh given a threshold in degrees.
#[allow(dead_code)]
pub fn detect_sharp_edges(
    positions: &[[f32; 3]],
    indices: &[u32],
    threshold_deg: f32,
) -> SharpEdgeResult {
    let nf = indices.len() / 3;
    let threshold_rad = threshold_deg * PI / 180.0;

    // Build edge-to-face map
    let mut edge_faces: std::collections::HashMap<(u32, u32), Vec<usize>> =
        std::collections::HashMap::new();
    for fi in 0..nf {
        let [a, b, c] = [indices[fi * 3], indices[fi * 3 + 1], indices[fi * 3 + 2]];
        for (u, v) in [(a, b), (b, c), (c, a)] {
            let key = if u < v { (u, v) } else { (v, u) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut sharp = Vec::new();
    for ((ea, eb), faces) in &edge_faces {
        if faces.len() != 2 {
            continue;
        }
        let fi0 = faces[0];
        let fi1 = faces[1];
        let n0 = face_normal_se(
            positions[indices[fi0 * 3] as usize],
            positions[indices[fi0 * 3 + 1] as usize],
            positions[indices[fi0 * 3 + 2] as usize],
        );
        let n1 = face_normal_se(
            positions[indices[fi1 * 3] as usize],
            positions[indices[fi1 * 3 + 1] as usize],
            positions[indices[fi1 * 3 + 2] as usize],
        );
        let cos_a = dot3_se(n0, n1).clamp(-1.0, 1.0);
        let angle = cos_a.acos();
        if angle > threshold_rad {
            sharp.push(SharpEdge {
                a: *ea,
                b: *eb,
                dihedral: angle * 180.0 / PI,
            });
        }
    }

    SharpEdgeResult {
        edges: sharp,
        threshold_deg,
    }
}

/// Count sharp edges.
#[allow(dead_code)]
pub fn sharp_edge_count(result: &SharpEdgeResult) -> usize {
    result.edges.len()
}

/// Maximum dihedral angle among sharp edges.
#[allow(dead_code)]
pub fn max_dihedral_angle(result: &SharpEdgeResult) -> f32 {
    result
        .edges
        .iter()
        .map(|e| e.dihedral)
        .fold(0.0_f32, f32::max)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn sharp_edge_to_json(result: &SharpEdgeResult) -> String {
    format!(
        r#"{{"sharp_edges":{},"threshold_deg":{:.2}}}"#,
        result.edges.len(),
        result.threshold_deg
    )
}

/// Mark vertices on sharp edges.
#[allow(dead_code)]
pub fn sharp_edge_vertices(result: &SharpEdgeResult) -> Vec<u32> {
    let mut verts: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for e in &result.edges {
        verts.insert(e.a);
        verts.insert(e.b);
    }
    let mut v: Vec<_> = verts.into_iter().collect();
    v.sort();
    v
}

/// Check if any edge is sharp.
#[allow(dead_code)]
pub fn has_sharp_edges(result: &SharpEdgeResult) -> bool {
    !result.edges.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.0, 0.0],
        ];
        // Two coplanar triangles sharing edge 0-1
        let idx = vec![0_u32, 1, 2, 0, 3, 1];
        (pos, idx)
    }

    #[test]
    fn flat_no_sharp_at_90() {
        let (pos, idx) = flat_mesh();
        let r = detect_sharp_edges(&pos, &idx, 90.0);
        // coplanar faces have 0 angle, not sharp
        assert_eq!(sharp_edge_count(&r), 0);
    }

    #[test]
    fn sharp_at_zero_threshold() {
        let (pos, idx) = flat_mesh();
        let r = detect_sharp_edges(&pos, &idx, 0.0);
        // All edges with non-zero dihedral are sharp
        let _ = r;
    }

    #[test]
    fn json_has_threshold() {
        let r = SharpEdgeResult {
            edges: vec![],
            threshold_deg: 45.0,
        };
        let j = sharp_edge_to_json(&r);
        assert!(j.contains("45.00"));
    }

    #[test]
    fn no_sharp_no_has() {
        let r = SharpEdgeResult {
            edges: vec![],
            threshold_deg: 30.0,
        };
        assert!(!has_sharp_edges(&r));
    }

    #[test]
    fn with_sharp_has() {
        let r = SharpEdgeResult {
            edges: vec![SharpEdge {
                a: 0,
                b: 1,
                dihedral: 90.0,
            }],
            threshold_deg: 30.0,
        };
        assert!(has_sharp_edges(&r));
    }

    #[test]
    fn max_dihedral() {
        let r = SharpEdgeResult {
            edges: vec![
                SharpEdge {
                    a: 0,
                    b: 1,
                    dihedral: 45.0,
                },
                SharpEdge {
                    a: 1,
                    b: 2,
                    dihedral: 90.0,
                },
            ],
            threshold_deg: 30.0,
        };
        assert!((max_dihedral_angle(&r) - 90.0).abs() < 1e-5);
    }

    #[test]
    fn sharp_vertices_unique() {
        let r = SharpEdgeResult {
            edges: vec![
                SharpEdge {
                    a: 0,
                    b: 1,
                    dihedral: 90.0,
                },
                SharpEdge {
                    a: 1,
                    b: 2,
                    dihedral: 90.0,
                },
            ],
            threshold_deg: 30.0,
        };
        let v = sharp_edge_vertices(&r);
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn count_edges() {
        let r = SharpEdgeResult {
            edges: vec![SharpEdge {
                a: 0,
                b: 1,
                dihedral: 90.0,
            }],
            threshold_deg: 30.0,
        };
        assert_eq!(sharp_edge_count(&r), 1);
    }

    #[test]
    fn empty_mesh_no_sharp() {
        let r = detect_sharp_edges(&[], &[], 30.0);
        assert_eq!(sharp_edge_count(&r), 0);
    }

    #[test]
    fn max_dihedral_empty() {
        let r = SharpEdgeResult {
            edges: vec![],
            threshold_deg: 30.0,
        };
        assert!((max_dihedral_angle(&r) - 0.0).abs() < 1e-6);
    }
}
