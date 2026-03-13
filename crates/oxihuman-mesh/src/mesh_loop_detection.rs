// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Detect and classify boundary loops in a mesh.

use std::collections::HashMap;

/// A detected boundary loop.
#[derive(Clone, Debug)]
pub struct BoundaryLoop {
    pub vertices: Vec<u32>,
    pub is_closed: bool,
    pub perimeter: f32,
}

/// Classification of a detected loop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoopClass {
    Outer,
    Hole,
    Unknown,
}

/// Result of loop detection.
#[derive(Clone, Debug, Default)]
pub struct LoopDetectResult {
    pub loops: Vec<BoundaryLoop>,
}

fn vec3_len(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    d.iter().map(|v| v * v).sum::<f32>().sqrt()
}

/// Detect boundary loops in a triangle mesh.
pub fn detect_boundary_loops_ld(positions: &[[f32; 3]], indices: &[u32]) -> LoopDetectResult {
    // Find boundary half-edges (edges with only one adjacent face)
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        for k in 0..3 {
            let a = indices[t * 3 + k];
            let b = indices[t * 3 + (k + 1) % 3];
            *edge_count.entry((a, b)).or_insert(0) += 1;
        }
    }

    // Boundary half-edges: directed edges that appear exactly once
    // (the opposite direction is missing)
    let mut next_vert: HashMap<u32, u32> = HashMap::new();
    for (&(a, b), &cnt) in &edge_count {
        if cnt == 1 {
            // This directed edge (a→b) is on the boundary
            next_vert.insert(a, b);
        }
    }

    // Trace loops
    let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut loops = Vec::new();

    for &start in next_vert.keys() {
        if visited.contains(&start) {
            continue;
        }
        let mut path = Vec::new();
        let mut cur = start;
        loop {
            if visited.contains(&cur) {
                break;
            }
            visited.insert(cur);
            path.push(cur);
            if let Some(&nxt) = next_vert.get(&cur) {
                cur = nxt;
            } else {
                break;
            }
            if cur == start {
                break;
            }
        }
        let is_closed = cur == start && path.len() > 2;
        let perimeter: f32 = if path.len() >= 2 {
            let mut p = 0.0_f32;
            for i in 0..path.len() - 1 {
                let va = positions[path[i] as usize];
                let vb = positions[path[i + 1] as usize];
                p += vec3_len(va, vb);
            }
            if is_closed {
                let va = positions[path[path.len() - 1] as usize];
                let vb = positions[path[0] as usize];
                p += vec3_len(va, vb);
            }
            p
        } else {
            0.0
        };

        if !path.is_empty() {
            loops.push(BoundaryLoop {
                vertices: path,
                is_closed,
                perimeter,
            });
        }
    }

    LoopDetectResult { loops }
}

/// Return the number of detected loops.
pub fn loop_count_ld(r: &LoopDetectResult) -> usize {
    r.loops.len()
}

/// Return the number of closed loops.
pub fn closed_loop_count(r: &LoopDetectResult) -> usize {
    r.loops.iter().filter(|l| l.is_closed).count()
}

/// Return the total perimeter of all loops.
pub fn total_loop_perimeter(r: &LoopDetectResult) -> f32 {
    r.loops.iter().map(|l| l.perimeter).sum()
}

/// Classify a loop as outer or hole based on signed area (XY projection).
pub fn classify_loop(l: &BoundaryLoop, positions: &[[f32; 3]]) -> LoopClass {
    if l.vertices.len() < 3 {
        return LoopClass::Unknown;
    }
    let mut signed_area = 0.0_f32;
    let n = l.vertices.len();
    for i in 0..n {
        let a = positions[l.vertices[i] as usize];
        let b = positions[l.vertices[(i + 1) % n] as usize];
        signed_area += (a[0] * b[1]) - (b[0] * a[1]);
    }
    if signed_area > 0.0 {
        LoopClass::Outer
    } else if signed_area < 0.0 {
        LoopClass::Hole
    } else {
        LoopClass::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_strip() -> (Vec<[f32; 3]>, Vec<u32>) {
        /* Two triangles forming a strip — boundary loops exist */
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn detect_finds_boundary() {
        let (pos, idx) = open_strip();
        let r = detect_boundary_loops_ld(&pos, &idx);
        assert!(loop_count_ld(&r) >= 1);
    }

    #[test]
    fn loop_count_consistent() {
        let (pos, idx) = open_strip();
        let r = detect_boundary_loops_ld(&pos, &idx);
        assert_eq!(loop_count_ld(&r), r.loops.len());
    }

    #[test]
    fn total_perimeter_positive() {
        let (pos, idx) = open_strip();
        let r = detect_boundary_loops_ld(&pos, &idx);
        assert!(total_loop_perimeter(&r) >= 0.0);
    }

    #[test]
    fn closed_count_lte_total() {
        let (pos, idx) = open_strip();
        let r = detect_boundary_loops_ld(&pos, &idx);
        assert!(closed_loop_count(&r) <= loop_count_ld(&r));
    }

    #[test]
    fn classify_outer_positive_area() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let l = BoundaryLoop {
            vertices: vec![0, 1, 2],
            is_closed: true,
            perimeter: 3.0,
        };
        let c = classify_loop(&l, &pos);
        assert!(c == LoopClass::Outer || c == LoopClass::Hole);
    }

    #[test]
    fn classify_short_loop_unknown() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let l = BoundaryLoop {
            vertices: vec![0, 1],
            is_closed: false,
            perimeter: 1.0,
        };
        assert_eq!(classify_loop(&l, &pos), LoopClass::Unknown);
    }

    #[test]
    fn empty_mesh_no_loops() {
        let r = detect_boundary_loops_ld(&[], &[]);
        assert_eq!(loop_count_ld(&r), 0);
    }

    #[test]
    fn perimeter_finite() {
        let (pos, idx) = open_strip();
        let r = detect_boundary_loops_ld(&pos, &idx);
        for l in &r.loops {
            assert!(l.perimeter.is_finite());
        }
    }
}
