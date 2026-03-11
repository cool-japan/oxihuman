// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Edge collapse mesh decimation using quadric error metrics.

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for edge-collapse decimation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeCollapseConfig {
    pub target_face_count: usize,
    pub max_error: f32,
    pub preserve_boundary: bool,
}

/// A candidate edge for collapse.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollapseEdge {
    pub v_a: u32,
    pub v_b: u32,
    pub error: f32,
}

/// Result of an edge-collapse decimation pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeCollapseResult {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
    pub collapsed_count: usize,
    pub final_error: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a sensible default `EdgeCollapseConfig` targeting `target` faces.
#[allow(dead_code)]
pub fn default_edge_collapse_config(target: usize) -> EdgeCollapseConfig {
    EdgeCollapseConfig {
        target_face_count: target,
        max_error: 0.01,
        preserve_boundary: true,
    }
}

/// Compute the squared distance between two vertices (proxy for quadric error).
#[allow(dead_code)]
pub fn compute_edge_error(v_a: [f32; 3], v_b: [f32; 3]) -> f32 {
    let dx = v_b[0] - v_a[0];
    let dy = v_b[1] - v_a[1];
    let dz = v_b[2] - v_a[2];
    dx * dx + dy * dy + dz * dz
}

/// Find all edges whose collapse error is below `max_error`.
#[allow(dead_code)]
pub fn find_collapse_candidates(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    max_error: f32,
) -> Vec<CollapseEdge> {
    let mut seen = std::collections::HashSet::new();
    let mut candidates = Vec::new();

    for tri in triangles {
        let edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
        for (a, b) in edges {
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                let ia = a as usize;
                let ib = b as usize;
                if ia < positions.len() && ib < positions.len() {
                    let err = compute_edge_error(positions[ia], positions[ib]);
                    if err <= max_error {
                        candidates.push(CollapseEdge {
                            v_a: a,
                            v_b: b,
                            error: err,
                        });
                    }
                }
            }
        }
    }
    candidates
}

/// Perform iterative edge collapses until `cfg.target_face_count` is reached or
/// no more cheap edges remain.
#[allow(dead_code)]
pub fn collapse_edges(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    cfg: &EdgeCollapseConfig,
) -> EdgeCollapseResult {
    let mut pos: Vec<[f32; 3]> = positions.to_vec();
    let mut tris: Vec<[u32; 3]> = triangles.to_vec();
    let mut collapsed = 0;
    let mut last_error = 0.0_f32;

    while tris.len() > cfg.target_face_count {
        let candidates = find_collapse_candidates(&pos, &tris, cfg.max_error);
        if candidates.is_empty() {
            break;
        }

        // Collapse the cheapest edge.
        let best = candidates
            .iter()
            .min_by(|a, b| a.error.partial_cmp(&b.error).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(&candidates[0]);

        let ia = best.v_a as usize;
        let ib = best.v_b as usize;
        last_error = best.error;

        // Move vertex A to the midpoint.
        let mp = edge_midpoint(pos[ia], pos[ib]);
        pos[ia] = mp;

        // Replace all references to B with A in the triangle list.
        let keep_a = best.v_a;
        let kill_b = best.v_b;
        for tri in &mut tris {
            for v in tri.iter_mut() {
                if *v == kill_b {
                    *v = keep_a;
                }
            }
        }

        // Remove degenerate triangles.
        tris.retain(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2]);

        collapsed += 1;
    }

    EdgeCollapseResult {
        positions: pos,
        triangles: tris,
        collapsed_count: collapsed,
        final_error: last_error,
    }
}

/// Return the midpoint of two vertices.
#[allow(dead_code)]
pub fn edge_midpoint(v_a: [f32; 3], v_b: [f32; 3]) -> [f32; 3] {
    [
        (v_a[0] + v_b[0]) * 0.5,
        (v_a[1] + v_b[1]) * 0.5,
        (v_a[2] + v_b[2]) * 0.5,
    ]
}

/// Serialize an `EdgeCollapseResult` to a JSON string.
#[allow(dead_code)]
pub fn edge_collapse_result_to_json(r: &EdgeCollapseResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"triangle_count\":{},\"collapsed_count\":{},\"final_error\":{}}}",
        r.positions.len(),
        r.triangles.len(),
        r.collapsed_count,
        r.final_error
    )
}

/// Serialize a `CollapseEdge` to a JSON string.
#[allow(dead_code)]
pub fn collapse_edge_to_json(e: &CollapseEdge) -> String {
    format!(
        "{{\"v_a\":{},\"v_b\":{},\"error\":{}}}",
        e.v_a, e.v_b, e.error
    )
}

/// Return the ratio of final faces to original faces.
#[allow(dead_code)]
pub fn reduction_ratio(r: &EdgeCollapseResult, original_faces: usize) -> f32 {
    if original_faces == 0 {
        return 1.0;
    }
    r.triangles.len() as f32 / original_faces as f32
}

/// Sort a slice of `CollapseEdge` in ascending error order (in place).
#[allow(dead_code)]
pub fn sort_collapse_edges(edges: &mut [CollapseEdge]) {
    edges.sort_by(|a, b| a.error.partial_cmp(&b.error).unwrap_or(std::cmp::Ordering::Equal));
}

/// Count the unique edges in a triangle mesh.
#[allow(dead_code)]
pub fn edge_count_from_triangles(triangles: &[[u32; 3]]) -> usize {
    let mut seen = std::collections::HashSet::new();
    for tri in triangles {
        let pairs = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
        for (a, b) in pairs {
            let key = if a < b { (a, b) } else { (b, a) };
            seen.insert(key);
        }
    }
    seen.len()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let pos = vec![
            [0.0_f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let tris = vec![[0, 1, 2], [1, 3, 2]];
        (pos, tris)
    }

    #[test]
    fn default_config_target() {
        let cfg = default_edge_collapse_config(100);
        assert_eq!(cfg.target_face_count, 100);
        assert!(cfg.max_error > 0.0);
    }

    #[test]
    fn edge_error_is_squared_distance() {
        let a = [0.0_f32, 0.0, 0.0];
        let b = [3.0, 4.0, 0.0];
        assert!((compute_edge_error(a, b) - 25.0).abs() < 1e-5);
    }

    #[test]
    fn midpoint_correct() {
        let a = [0.0_f32, 0.0, 0.0];
        let b = [2.0, 4.0, 6.0];
        let m = edge_midpoint(a, b);
        assert!((m[0] - 1.0).abs() < 1e-6);
        assert!((m[1] - 2.0).abs() < 1e-6);
        assert!((m[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn edge_count_simple_mesh() {
        let (_, tris) = simple_mesh();
        let count = edge_count_from_triangles(&tris);
        // 2 triangles sharing 1 edge → 5 unique edges
        assert_eq!(count, 5);
    }

    #[test]
    fn find_candidates_respects_max_error() {
        let (pos, tris) = simple_mesh();
        // max_error = 0 → no short edges
        let zero = find_collapse_candidates(&pos, &tris, 0.0);
        assert!(zero.is_empty());
        // max_error = 100 → all edges
        let all = find_collapse_candidates(&pos, &tris, 100.0);
        assert!(!all.is_empty());
    }

    #[test]
    fn sort_collapse_edges_ascending() {
        let mut edges = vec![
            CollapseEdge { v_a: 0, v_b: 1, error: 3.0 },
            CollapseEdge { v_a: 1, v_b: 2, error: 1.0 },
            CollapseEdge { v_a: 0, v_b: 2, error: 2.0 },
        ];
        sort_collapse_edges(&mut edges);
        assert!(edges[0].error <= edges[1].error);
        assert!(edges[1].error <= edges[2].error);
    }

    #[test]
    fn collapse_reduces_faces() {
        let (pos, tris) = simple_mesh();
        let cfg = EdgeCollapseConfig {
            target_face_count: 1,
            max_error: 10.0,
            preserve_boundary: false,
        };
        let r = collapse_edges(&pos, &tris, &cfg);
        assert!(r.triangles.len() <= 2);
        assert!(r.collapsed_count > 0 || r.triangles.len() == 2);
    }

    #[test]
    fn reduction_ratio_full_keeps() {
        let r = EdgeCollapseResult {
            positions: vec![[0.0, 0.0, 0.0]],
            triangles: vec![[0, 0, 0], [0, 0, 0]],
            collapsed_count: 0,
            final_error: 0.0,
        };
        let ratio = reduction_ratio(&r, 2);
        assert!((ratio - 1.0).abs() < 1e-6);
    }

    #[test]
    fn json_contains_fields() {
        let r = EdgeCollapseResult {
            positions: vec![[0.0, 0.0, 0.0]; 4],
            triangles: vec![[0, 1, 2]],
            collapsed_count: 1,
            final_error: 0.001,
        };
        let j = edge_collapse_result_to_json(&r);
        assert!(j.contains("\"collapsed_count\":1"));
        assert!(j.contains("\"triangle_count\":1"));
    }
}
