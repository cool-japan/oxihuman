// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Vertex slide: slide a vertex along an edge.

/* ── legacy API (keep for any future lib.rs exports) ── */

#[derive(Debug, Clone)]
pub struct VertexSlideResult {
    pub positions: Vec<[f32; 3]>,
    pub moved: bool,
}

pub fn slide_vertex(
    positions: &[[f32; 3]],
    vertex: usize,
    target_neighbor: usize,
    factor: f32,
) -> VertexSlideResult {
    let factor = factor.clamp(0.0, 1.0);
    let mut new_pos = positions.to_vec();
    let a = positions[vertex];
    let b = positions[target_neighbor];
    new_pos[vertex] = [
        a[0] + (b[0] - a[0]) * factor,
        a[1] + (b[1] - a[1]) * factor,
        a[2] + (b[2] - a[2]) * factor,
    ];
    VertexSlideResult {
        positions: new_pos,
        moved: factor > 0.0,
    }
}

pub fn slide_vertices(positions: &[[f32; 3]], slides: &[(usize, usize, f32)]) -> VertexSlideResult {
    let mut new_pos = positions.to_vec();
    let mut moved = false;
    for &(vertex, target, factor) in slides {
        let factor = factor.clamp(0.0, 1.0);
        let a = new_pos[vertex];
        let b = positions[target];
        new_pos[vertex] = [
            a[0] + (b[0] - a[0]) * factor,
            a[1] + (b[1] - a[1]) * factor,
            a[2] + (b[2] - a[2]) * factor,
        ];
        if factor > 0.0 {
            moved = true;
        }
    }
    VertexSlideResult {
        positions: new_pos,
        moved,
    }
}

pub fn vertex_distance(positions: &[[f32; 3]], a: usize, b: usize) -> f32 {
    let dx = positions[a][0] - positions[b][0];
    let dy = positions[a][1] - positions[b][1];
    let dz = positions[a][2] - positions[b][2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub fn nearest_neighbor(
    positions: &[[f32; 3]],
    vertex: usize,
    candidates: &[usize],
) -> Option<usize> {
    candidates.iter().copied().min_by(|&a, &b| {
        let da = vertex_distance(positions, vertex, a);
        let db = vertex_distance(positions, vertex, b);
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })
}

pub fn slide_to_json(result: &VertexSlideResult) -> String {
    format!(
        "{{\"vertices\":{},\"moved\":{}}}",
        result.positions.len(),
        result.moved
    )
}

/* ── spec functions (wave 150B) ── */

/// Slide position of vertex `v` along edge to `target` by `factor`.
pub fn vertex_slide_position(
    positions: &[[f32; 3]],
    v: usize,
    target: usize,
    factor: f32,
) -> [f32; 3] {
    let factor = factor.clamp(0.0, 1.0);
    let a = positions[v];
    let b = positions[target];
    [
        a[0] + (b[0] - a[0]) * factor,
        a[1] + (b[1] - a[1]) * factor,
        a[2] + (b[2] - a[2]) * factor,
    ]
}

/// Slide a vertex toward its nearest candidate neighbor.
pub fn vertex_slide_toward_nearest(
    positions: &[[f32; 3]],
    v: usize,
    candidates: &[usize],
    factor: f32,
) -> [f32; 3] {
    match nearest_neighbor(positions, v, candidates) {
        Some(n) => vertex_slide_position(positions, v, n, factor),
        None => positions[v],
    }
}

/// Clamp a slide factor to [0, 1].
pub fn vertex_slide_clamp(factor: f32) -> f32 {
    factor.clamp(0.0, 1.0)
}

/// Unit direction from vertex `v` to vertex `target`.
pub fn vertex_edge_direction(positions: &[[f32; 3]], v: usize, target: usize) -> [f32; 3] {
    let a = positions[v];
    let b = positions[target];
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    if len < f32::EPSILON {
        return [0.0; 3];
    }
    [dx / len, dy / len, dz / len]
}

/// Offset a vertex along edge direction by a scalar distance.
pub fn vertex_slide_offset(
    positions: &[[f32; 3]],
    v: usize,
    target: usize,
    distance: f32,
) -> [f32; 3] {
    let dir = vertex_edge_direction(positions, v, target);
    let p = positions[v];
    [
        p[0] + dir[0] * distance,
        p[1] + dir[1] * distance,
        p[2] + dir[2] * distance,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_verts() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [4.0, 0.0, 0.0]]
    }

    #[test]
    fn test_slide_half() {
        let pos = line_verts();
        let r = slide_vertex(&pos, 0, 1, 0.5);
        assert!((r.positions[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_slide_full() {
        let pos = line_verts();
        let r = slide_vertex(&pos, 0, 1, 1.0);
        assert!((r.positions[0][0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_slide_zero_not_moved() {
        let pos = line_verts();
        let r = slide_vertex(&pos, 0, 1, 0.0);
        assert!(!r.moved);
    }

    #[test]
    fn test_vertex_slide_position() {
        let pos = line_verts();
        let p = vertex_slide_position(&pos, 0, 1, 0.5);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_vertex_edge_direction() {
        let pos = line_verts();
        let d = vertex_edge_direction(&pos, 0, 1);
        assert!((d[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_vertex_slide_offset() {
        let pos = line_verts();
        let p = vertex_slide_offset(&pos, 0, 1, 1.0);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_vertex_slide_toward_nearest() {
        let pos = line_verts();
        let p = vertex_slide_toward_nearest(&pos, 0, &[1, 2], 1.0);
        /* nearest to 0 is 1 (at x=2) */
        assert!((p[0] - 2.0).abs() < 1e-5);
    }
}
