// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bridge two edge loops.

/// An ordered edge loop (list of vertex indices).
#[derive(Debug, Clone)]
pub struct BridgeLoop {
    pub vertices: Vec<u32>,
}

/// Result of bridging two loops.
#[derive(Debug, Clone)]
pub struct BridgeToolResult {
    pub new_positions: Vec<[f32; 3]>,
    pub new_indices: Vec<u32>,
    pub quad_count: usize,
}

/// Number of interpolation segments along the bridge.
#[derive(Debug, Clone, Copy)]
pub struct BridgeToolConfig {
    pub segments: usize,
    pub twist: i32,
}

impl Default for BridgeToolConfig {
    fn default() -> Self {
        BridgeToolConfig {
            segments: 1,
            twist: 0,
        }
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute the centroid of a loop.
pub fn loop_centroid(loop_verts: &BridgeLoop, positions: &[[f32; 3]]) -> [f32; 3] {
    if loop_verts.vertices.is_empty() {
        return [0.0; 3];
    }
    let n = loop_verts.vertices.len() as f32;
    let s = loop_verts.vertices.iter().fold([0.0f32; 3], |acc, &v| {
        let p = positions[v as usize];
        [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
    });
    [s[0] / n, s[1] / n, s[2] / n]
}

/// Find the best starting vertex on loop B to minimise total bridge length.
pub fn align_loops_twist(
    loop_a: &BridgeLoop,
    loop_b: &BridgeLoop,
    positions: &[[f32; 3]],
    extra_twist: i32,
) -> usize {
    let n = loop_b.vertices.len();
    if n == 0 {
        return 0;
    }
    let mut best_start = extra_twist.rem_euclid(n as i32) as usize;
    let mut best_cost = f32::MAX;
    for start in 0..n {
        let cost: f32 = loop_a
            .vertices
            .iter()
            .enumerate()
            .map(|(i, &av)| {
                let bv = loop_b.vertices[(start + i) % n];
                dist3(positions[av as usize], positions[bv as usize])
            })
            .sum();
        if cost < best_cost {
            best_cost = cost;
            best_start = start;
        }
    }
    best_start
}

/// Bridge two edge loops with optional intermediate segments.
pub fn bridge_loops(
    positions: &[[f32; 3]],
    loop_a: &BridgeLoop,
    loop_b: &BridgeLoop,
    config: &BridgeToolConfig,
) -> BridgeToolResult {
    let na = loop_a.vertices.len();
    let nb = loop_b.vertices.len();
    if na == 0 || nb == 0 || na != nb {
        return BridgeToolResult {
            new_positions: vec![],
            new_indices: vec![],
            quad_count: 0,
        };
    }
    let segments = config.segments.max(1);
    let start_b = align_loops_twist(loop_a, loop_b, positions, config.twist);
    let n = na;

    /* Build intermediate ring positions */
    let mut new_positions = positions.to_vec();
    let mut ring_base_indices: Vec<Vec<u32>> = Vec::new();

    /* ring 0 = loop_a vertex indices (original) */
    ring_base_indices.push(loop_a.vertices.to_vec());

    for seg in 1..segments {
        let t = seg as f32 / segments as f32;
        let mut ring = Vec::with_capacity(n);
        for i in 0..n {
            let av = loop_a.vertices[i];
            let bv = loop_b.vertices[(start_b + i) % n];
            let p = lerp3(positions[av as usize], positions[bv as usize], t);
            let new_idx = new_positions.len() as u32;
            new_positions.push(p);
            ring.push(new_idx);
        }
        ring_base_indices.push(ring);
    }

    /* last ring = loop_b */
    ring_base_indices.push((0..n).map(|i| loop_b.vertices[(start_b + i) % n]).collect());

    /* Build quads between consecutive rings */
    let mut new_indices = Vec::new();
    let mut quad_count = 0usize;
    for ri in 0..ring_base_indices.len() - 1 {
        let r0 = &ring_base_indices[ri];
        let r1 = &ring_base_indices[ri + 1];
        for i in 0..n {
            let j = (i + 1) % n;
            /* quad: a0, a1, b1, b0 → two triangles */
            let (a0, a1) = (r0[i], r0[j]);
            let (b0, b1) = (r1[i], r1[j]);
            new_indices.extend_from_slice(&[a0, a1, b1, a0, b1, b0]);
            quad_count += 1;
        }
    }

    BridgeToolResult {
        new_positions,
        new_indices,
        quad_count,
    }
}

/// Create a simple loop from a list of vertex indices.
pub fn make_bridge_loop(vertices: Vec<u32>) -> BridgeLoop {
    BridgeLoop { vertices }
}

/// Validate that two loops are bridge-compatible (same length).
pub fn loops_compatible(loop_a: &BridgeLoop, loop_b: &BridgeLoop) -> bool {
    !loop_a.vertices.is_empty() && loop_a.vertices.len() == loop_b.vertices.len()
}

/// Return the number of quads a bridge of two loops of length n will produce.
pub fn bridge_quad_estimate(loop_len: usize, segments: usize) -> usize {
    loop_len * segments.max(1)
}

/// Return vertex indices introduced by the bridge (excluding original loops).
pub fn bridge_new_vertex_count(n: usize, segments: usize) -> usize {
    n * (segments.saturating_sub(1))
}

/// Compute total bridge length (sum of corresponding vertex distances).
pub fn bridge_total_length(
    loop_a: &BridgeLoop,
    loop_b: &BridgeLoop,
    positions: &[[f32; 3]],
) -> f32 {
    loop_a
        .vertices
        .iter()
        .zip(loop_b.vertices.iter())
        .map(|(&a, &b)| dist3(positions[a as usize], positions[b as usize]))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ]
    }

    /* bridge two compatible loops */
    #[test]
    fn test_bridge_loops_basic() {
        let pos = square_positions();
        let la = make_bridge_loop(vec![0, 1, 2, 3]);
        let lb = make_bridge_loop(vec![4, 5, 6, 7]);
        let cfg = BridgeToolConfig::default();
        let res = bridge_loops(&pos, &la, &lb, &cfg);
        assert_eq!(res.quad_count, 4);
    }

    /* incompatible loops → empty result */
    #[test]
    fn test_bridge_incompatible() {
        let pos = square_positions();
        let la = make_bridge_loop(vec![0, 1, 2]);
        let lb = make_bridge_loop(vec![4, 5, 6, 7]);
        let cfg = BridgeToolConfig::default();
        let res = bridge_loops(&pos, &la, &lb, &cfg);
        assert_eq!(res.quad_count, 0);
    }

    /* loop_centroid */
    #[test]
    fn test_loop_centroid() {
        let pos = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let lp = make_bridge_loop(vec![0, 1]);
        let c = loop_centroid(&lp, &pos);
        assert!((c[0] - 1.0).abs() < 1e-5);
    }

    /* loops_compatible */
    #[test]
    fn test_loops_compatible() {
        let la = make_bridge_loop(vec![0, 1, 2, 3]);
        let lb = make_bridge_loop(vec![4, 5, 6, 7]);
        assert!(loops_compatible(&la, &lb));
    }

    /* bridge_quad_estimate */
    #[test]
    fn test_bridge_quad_estimate() {
        assert_eq!(bridge_quad_estimate(4, 2), 8);
    }

    /* bridge_new_vertex_count */
    #[test]
    fn test_bridge_new_vertex_count() {
        assert_eq!(bridge_new_vertex_count(4, 3), 8);
    }

    /* bridge_total_length */
    #[test]
    fn test_bridge_total_length() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        let la = make_bridge_loop(vec![0, 1]);
        let lb = make_bridge_loop(vec![2, 3]);
        let len = bridge_total_length(&la, &lb, &pos);
        assert!(len > 0.0);
    }

    /* bridge with 2 segments creates intermediate verts */
    #[test]
    fn test_bridge_with_segments() {
        let pos = square_positions();
        let la = make_bridge_loop(vec![0, 1, 2, 3]);
        let lb = make_bridge_loop(vec![4, 5, 6, 7]);
        let cfg = BridgeToolConfig {
            segments: 2,
            twist: 0,
        };
        let res = bridge_loops(&pos, &la, &lb, &cfg);
        assert_eq!(res.quad_count, 8);
        assert!(res.new_positions.len() > 8);
    }
}
