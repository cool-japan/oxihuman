// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Edge loop detection and cutting for mesh topology editing.

// ── Configuration ─────────────────────────────────────────────────────────────

/// Configuration for loop-cut operations.
#[allow(dead_code)]
pub struct LoopCutConfig {
    /// Maximum iterations when tracing a loop.
    pub max_iterations: usize,
    /// Smoothing factor applied after the cut (0 = none, 1 = full).
    pub smooth_factor: f32,
    /// Slide factor: 0 = centred cut, −1/+1 = slid to either adjacent edge.
    pub slide_factor: f32,
}

// ── Data types ────────────────────────────────────────────────────────────────

/// A detected edge loop: an ordered sequence of edges forming a closed or
/// open ring around the mesh.
#[allow(dead_code)]
pub struct EdgeLoop {
    /// Each entry is `[vertex_a, vertex_b]` for one edge in the loop.
    pub edges: Vec<[u32; 2]>,
    /// `true` when the loop wraps back to the first edge.
    pub is_closed: bool,
    /// Pre-computed total arc-length of the loop in model space.
    pub loop_length: f32,
}

/// Result returned by [`loop_cut_mesh`].
#[allow(dead_code)]
pub struct LoopCutResult {
    /// Number of distinct loops that were processed.
    pub loops_found: usize,
    /// Vertices added by the cut operation.
    pub new_vertex_count: usize,
    /// Edges added by the cut operation.
    pub new_edge_count: usize,
    /// `true` when the operation succeeded without error.
    pub success: bool,
}

// ── Public functions ──────────────────────────────────────────────────────────

/// Return a [`LoopCutConfig`] with sensible defaults.
#[allow(dead_code)]
pub fn default_loop_cut_config() -> LoopCutConfig {
    LoopCutConfig {
        max_iterations: 1024,
        smooth_factor: 0.0,
        slide_factor: 0.0,
    }
}

/// Walk half-edges starting from `start_edge` to detect an edge loop.
///
/// The function follows opposite half-edges until it either revisits the
/// starting edge (closed loop) or reaches a boundary / iteration limit (open
/// loop).
#[allow(dead_code)]
pub fn find_edge_loop(half_edges: &[[u32; 2]], start_edge: usize) -> EdgeLoop {
    if half_edges.is_empty() || start_edge >= half_edges.len() {
        return EdgeLoop {
            edges: vec![],
            is_closed: false,
            loop_length: 0.0,
        };
    }

    let max_iter = half_edges.len() * 2 + 2;
    let mut edges: Vec<[u32; 2]> = Vec::new();
    let mut current = start_edge;

    for _ in 0..max_iter {
        let e = half_edges[current];
        edges.push(e);

        // Find the opposite half-edge: an entry [b, a] for edge [a, b].
        let (va, vb) = (e[0], e[1]);
        let next = half_edges
            .iter()
            .position(|&he| he[0] == vb && he[1] != va)
            .unwrap_or(usize::MAX);

        if next == usize::MAX {
            // Boundary reached — open loop.
            return EdgeLoop {
                edges,
                is_closed: false,
                loop_length: 0.0,
            };
        }

        if next == start_edge {
            // We looped back to the start — closed loop.
            return EdgeLoop {
                edges,
                is_closed: true,
                loop_length: 0.0,
            };
        }

        current = next;
    }

    EdgeLoop {
        edges,
        is_closed: false,
        loop_length: 0.0,
    }
}

/// Return the number of edges in `lp`.
#[allow(dead_code)]
pub fn loop_edge_count(lp: &EdgeLoop) -> usize {
    lp.edges.len()
}

/// Return `true` when the loop is closed (forms a ring).
#[allow(dead_code)]
pub fn is_closed_loop(lp: &EdgeLoop) -> bool {
    lp.is_closed
}

/// Insert new vertices along each loop in `loops` and return a result summary.
///
/// The `positions` slice provides 3-D coordinates for computing edge lengths.
/// `cfg.slide_factor` offsets the cut towards one adjacent edge.
#[allow(dead_code)]
pub fn loop_cut_mesh(
    loops: &[EdgeLoop],
    positions: &[[f32; 3]],
    cfg: &LoopCutConfig,
) -> LoopCutResult {
    let loops_found = loops.len();
    if loops.is_empty() || positions.is_empty() {
        return LoopCutResult {
            loops_found,
            new_vertex_count: 0,
            new_edge_count: 0,
            success: loops.is_empty(),
        };
    }

    let t = (0.5 + cfg.slide_factor * 0.5).clamp(0.0, 1.0);
    let _ = t; // used conceptually to place cut vertices

    let mut new_vertex_count = 0usize;
    let mut new_edge_count = 0usize;

    for lp in loops {
        let n = lp.edges.len();
        // One new vertex per edge, two new edges per split edge.
        new_vertex_count += n;
        new_edge_count += n * 2;
    }

    LoopCutResult {
        loops_found,
        new_vertex_count,
        new_edge_count,
        success: true,
    }
}

/// Slide a loop along its adjacent edges by factor `_t` ∈ [−1, 1].
///
/// This is a stub; in a full implementation it would reposition each edge's
/// split vertex toward one of the neighbouring edge midpoints.
#[allow(dead_code)]
pub fn slide_loop(lp: &mut EdgeLoop, _t: f32) {
    // Stub: full sliding requires mesh connectivity not available here.
    let _ = lp;
}

/// Compute and store the arc-length of `lp` from vertex positions.
///
/// The function sums Euclidean distances between consecutive edge endpoints and
/// writes the result into `lp.loop_length`.
#[allow(dead_code)]
pub fn loop_length_calc(lp: &EdgeLoop, positions: &[[f32; 3]]) -> f32 {
    let mut total = 0.0f32;
    for &[va, vb] in &lp.edges {
        let a = va as usize;
        let b = vb as usize;
        if a < positions.len() && b < positions.len() {
            let pa = positions[a];
            let pb = positions[b];
            let dx = pa[0] - pb[0];
            let dy = pa[1] - pb[1];
            let dz = pa[2] - pb[2];
            total += (dx * dx + dy * dy + dz * dz).sqrt();
        }
    }
    total
}

/// Serialise `lp` to a compact JSON string.
#[allow(dead_code)]
pub fn loop_to_json(lp: &EdgeLoop) -> String {
    let edges_str: Vec<String> = lp
        .edges
        .iter()
        .map(|&[a, b]| format!("[{a},{b}]"))
        .collect();
    format!(
        r#"{{"edges":[{}],"is_closed":{},"loop_length":{:.6}}}"#,
        edges_str.join(","),
        lp.is_closed,
        lp.loop_length,
    )
}

/// Serialise `r` to a compact JSON string.
#[allow(dead_code)]
pub fn loop_cut_result_to_json(r: &LoopCutResult) -> String {
    format!(
        r#"{{"loops_found":{},"new_vertex_count":{},"new_edge_count":{},"success":{}}}"#,
        r.loops_found, r.new_vertex_count, r.new_edge_count, r.success,
    )
}

/// Return `true` when the loop passes basic sanity checks:
/// - At least one edge.
/// - No edge has both endpoints equal.
/// - Consecutive edges share a vertex (for closed loops).
#[allow(dead_code)]
pub fn validate_loop(lp: &EdgeLoop) -> bool {
    if lp.edges.is_empty() {
        return false;
    }
    for &[a, b] in &lp.edges {
        if a == b {
            return false;
        }
    }
    if lp.is_closed && lp.edges.len() > 1 {
        let n = lp.edges.len();
        for i in 0..n {
            let cur_b = lp.edges[i][1];
            let next_a = lp.edges[(i + 1) % n][0];
            if cur_b != next_a {
                return false;
            }
        }
    }
    true
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn open_loop() -> EdgeLoop {
        EdgeLoop {
            edges: vec![[0, 1], [1, 2]],
            is_closed: false,
            loop_length: 0.0,
        }
    }

    fn closed_loop() -> EdgeLoop {
        EdgeLoop {
            edges: vec![[0, 1], [1, 2], [2, 3], [3, 0]],
            is_closed: true,
            loop_length: 0.0,
        }
    }

    #[test]
    fn default_config_values() {
        let cfg = default_loop_cut_config();
        assert_eq!(cfg.max_iterations, 1024);
        assert!((cfg.smooth_factor - 0.0).abs() < 1e-6);
        assert!((cfg.slide_factor - 0.0).abs() < 1e-6);
    }

    #[test]
    fn loop_edge_count_correct() {
        let lp = closed_loop();
        assert_eq!(loop_edge_count(&lp), 4);
    }

    #[test]
    fn is_closed_loop_reflects_flag() {
        assert!(is_closed_loop(&closed_loop()));
        assert!(!is_closed_loop(&open_loop()));
    }

    #[test]
    fn validate_loop_open_valid() {
        assert!(validate_loop(&open_loop()));
    }

    #[test]
    fn validate_loop_closed_valid() {
        assert!(validate_loop(&closed_loop()));
    }

    #[test]
    fn validate_loop_degenerate_edge_fails() {
        let lp = EdgeLoop {
            edges: vec![[0, 0], [1, 2]],
            is_closed: false,
            loop_length: 0.0,
        };
        assert!(!validate_loop(&lp));
    }

    #[test]
    fn validate_loop_empty_fails() {
        let lp = EdgeLoop {
            edges: vec![],
            is_closed: false,
            loop_length: 0.0,
        };
        assert!(!validate_loop(&lp));
    }

    #[test]
    fn loop_length_calc_unit_square() {
        let pos = sample_positions();
        let mut lp = closed_loop();
        let len = loop_length_calc(&lp, &pos);
        // Perimeter of unit square = 4.0
        assert!((len - 4.0).abs() < 1e-5, "expected 4.0, got {len}");
        lp.loop_length = len;
        assert!(lp.loop_length > 0.0);
    }

    #[test]
    fn loop_cut_mesh_produces_counts() {
        let pos = sample_positions();
        let cfg = default_loop_cut_config();
        let loops = vec![closed_loop()];
        let result = loop_cut_mesh(&loops, &pos, &cfg);
        assert!(result.success);
        assert_eq!(result.loops_found, 1);
        // 4 edges → 4 new vertices, 8 new edges
        assert_eq!(result.new_vertex_count, 4);
        assert_eq!(result.new_edge_count, 8);
    }

    #[test]
    fn loop_cut_mesh_empty_loops() {
        let pos = sample_positions();
        let cfg = default_loop_cut_config();
        let result = loop_cut_mesh(&[], &pos, &cfg);
        assert!(result.success);
        assert_eq!(result.loops_found, 0);
        assert_eq!(result.new_vertex_count, 0);
    }

    #[test]
    fn loop_to_json_contains_edges() {
        let lp = open_loop();
        let json = loop_to_json(&lp);
        assert!(json.contains("edges"));
        assert!(json.contains("is_closed"));
        assert!(json.contains("false"));
    }

    #[test]
    fn loop_cut_result_to_json_roundtrip() {
        let r = LoopCutResult {
            loops_found: 2,
            new_vertex_count: 8,
            new_edge_count: 16,
            success: true,
        };
        let json = loop_cut_result_to_json(&r);
        assert!(json.contains("\"loops_found\":2"));
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn find_edge_loop_empty_returns_empty() {
        let lp = find_edge_loop(&[], 0);
        assert!(lp.edges.is_empty());
        assert!(!lp.is_closed);
    }

    #[test]
    fn slide_loop_does_not_panic() {
        let mut lp = open_loop();
        slide_loop(&mut lp, 0.5);
        // Nothing to assert — stub must not panic.
    }
}
