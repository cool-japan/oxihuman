// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge loop slide: slide an edge loop along adjacent faces.

use std::collections::HashSet;

/// A simple edge loop represented as an ordered list of vertex indices.
#[derive(Debug, Clone)]
pub struct SlideLoop {
    pub vertices: Vec<u32>,
    pub is_closed: bool,
}

/// Result of a loop-slide operation.
#[derive(Debug, Clone)]
pub struct LoopSlideResult {
    pub new_positions: Vec<[f32; 3]>,
    pub slide_offset: f32,
    pub vertices_moved: usize,
}

/// Returns `true` if the slide parameter is in valid range [0, 1].
pub fn slide_param_valid(t: f32) -> bool {
    (0.0..=1.0).contains(&t)
}

/// Linearly interpolates a vertex position along a slide direction.
pub fn slide_vertex(current: [f32; 3], target: [f32; 3], t: f32) -> [f32; 3] {
    [
        current[0] + (target[0] - current[0]) * t,
        current[1] + (target[1] - current[1]) * t,
        current[2] + (target[2] - current[2]) * t,
    ]
}

/// Slides a loop of vertices towards a set of target positions by factor `t`.
pub fn slide_loop(
    positions: &mut [[f32; 3]],
    loop_verts: &[u32],
    targets: &[[f32; 3]],
    t: f32,
) -> usize {
    let count = loop_verts.len().min(targets.len());
    let mut moved = 0usize;
    for i in 0..count {
        let vi = loop_verts[i] as usize;
        if vi >= positions.len() {
            continue;
        }
        let new_pos = slide_vertex(positions[vi], targets[i], t);
        positions[vi] = new_pos;
        moved += 1;
    }
    moved
}

/// Detects vertex indices that form a simple edge loop given an index buffer.
/// Returns an ordered list of consecutive boundary vertices.
pub fn extract_simple_loop(indices: &[u32], start: u32) -> SlideLoop {
    /* build adjacency: vertex → connected boundary neighbours */
    let mut adj: std::collections::HashMap<u32, Vec<u32>> = std::collections::HashMap::new();
    let n = indices.len() / 3;
    let mut edge_count: std::collections::HashMap<(u32, u32), usize> =
        std::collections::HashMap::new();
    for i in 0..n {
        let ia = indices[i * 3];
        let ib = indices[i * 3 + 1];
        let ic = indices[i * 3 + 2];
        for (a, b) in [(ia, ib), (ib, ic), (ic, ia)] {
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    /* boundary edges appear exactly once */
    let boundary: HashSet<(u32, u32)> = edge_count
        .into_iter()
        .filter(|(_, c)| *c == 1)
        .map(|(k, _)| k)
        .collect();
    for &(a, b) in &boundary {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }
    let mut loop_verts = vec![start];
    let mut visited: HashSet<u32> = HashSet::new();
    visited.insert(start);
    let mut current = start;
    loop {
        let next = adj
            .get(&current)
            .and_then(|ns| ns.iter().find(|&&n| !visited.contains(&n)));
        match next {
            Some(&v) => {
                loop_verts.push(v);
                visited.insert(v);
                current = v;
            }
            None => break,
        }
    }
    let is_closed = adj.get(&start).is_some_and(|ns| ns.contains(&current));
    SlideLoop {
        vertices: loop_verts,
        is_closed,
    }
}

/// Computes centroid of a set of positions.
pub fn loop_centroid(positions: &[[f32; 3]], loop_verts: &[u32]) -> [f32; 3] {
    if loop_verts.is_empty() {
        return [0.0; 3];
    }
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    let mut count = 0usize;
    for &vi in loop_verts {
        let vi = vi as usize;
        if vi < positions.len() {
            cx += positions[vi][0];
            cy += positions[vi][1];
            cz += positions[vi][2];
            count += 1;
        }
    }
    if count == 0 {
        return [0.0; 3];
    }
    [cx / count as f32, cy / count as f32, cz / count as f32]
}

/// Returns the edge count of a loop.
pub fn loop_edge_count_ls(loop_: &SlideLoop) -> usize {
    if loop_.is_closed {
        loop_.vertices.len()
    } else {
        loop_.vertices.len().saturating_sub(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slide_param_valid_bounds() {
        assert!(slide_param_valid(0.0));
        assert!(slide_param_valid(0.5));
        assert!(slide_param_valid(1.0));
        assert!(!slide_param_valid(1.1));
    }

    #[test]
    fn slide_vertex_halfway() {
        let p = slide_vertex([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], 0.5);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn slide_vertex_t0_unchanged() {
        let orig = [3.0f32, 4.0, 5.0];
        let p = slide_vertex(orig, [0.0, 0.0, 0.0], 0.0);
        assert_eq!(p, orig);
    }

    #[test]
    fn slide_loop_moves_vertices() {
        let mut pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let verts = [0u32, 1];
        let targets = [[1.0f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let moved = slide_loop(&mut pos, &verts, &targets, 1.0);
        assert_eq!(moved, 2);
        assert!((pos[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn loop_centroid_single() {
        let pos = vec![[3.0f32, 0.0, 0.0]];
        let verts = [0u32];
        let c = loop_centroid(&pos, &verts);
        assert!((c[0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn loop_centroid_empty() {
        let pos: Vec<[f32; 3]> = vec![];
        let c = loop_centroid(&pos, &[]);
        assert_eq!(c, [0.0; 3]);
    }

    #[test]
    fn loop_edge_count_closed() {
        let lp = SlideLoop {
            vertices: vec![0, 1, 2],
            is_closed: true,
        };
        assert_eq!(loop_edge_count_ls(&lp), 3);
    }

    #[test]
    fn loop_edge_count_open() {
        let lp = SlideLoop {
            vertices: vec![0, 1, 2],
            is_closed: false,
        };
        assert_eq!(loop_edge_count_ls(&lp), 2);
    }

    #[test]
    fn slide_loop_out_of_bounds_skipped() {
        let mut pos = vec![[0.0f32; 3]];
        let verts = [99u32]; /* out of bounds */
        let targets = [[1.0f32; 3]];
        let moved = slide_loop(&mut pos, &verts, &targets, 1.0);
        assert_eq!(moved, 0);
    }
}
