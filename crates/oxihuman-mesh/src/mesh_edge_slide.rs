// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge slide: slide an edge along adjacent faces.

/// Slide a vertex `v` along the edge to neighbor `target` by `factor` (0..1).
pub fn edge_slide_vert(positions: &[[f32; 3]], v: usize, target: usize, factor: f32) -> [f32; 3] {
    let factor = factor.clamp(0.0, 1.0);
    let a = positions[v];
    let b = positions[target];
    [
        a[0] + (b[0] - a[0]) * factor,
        a[1] + (b[1] - a[1]) * factor,
        a[2] + (b[2] - a[2]) * factor,
    ]
}

/// Slide both endpoints of an edge by `factor`, returning their new positions.
pub fn edge_slide_edge(
    positions: &[[f32; 3]],
    v0: usize,
    v1: usize,
    neighbor0: usize,
    neighbor1: usize,
    factor: f32,
) -> ([f32; 3], [f32; 3]) {
    (
        edge_slide_vert(positions, v0, neighbor0, factor),
        edge_slide_vert(positions, v1, neighbor1, factor),
    )
}

/// Midpoint of a slid edge.
pub fn edge_slide_midpoint(
    positions: &[[f32; 3]],
    v0: usize,
    v1: usize,
    neighbor0: usize,
    neighbor1: usize,
    factor: f32,
) -> [f32; 3] {
    let (a, b) = edge_slide_edge(positions, v0, v1, neighbor0, neighbor1, factor);
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

/// Length of an edge.
pub fn edge_length(positions: &[[f32; 3]], v0: usize, v1: usize) -> f32 {
    let a = positions[v0];
    let b = positions[v1];
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Unit direction of an edge from v0 to v1.
pub fn edge_direction(positions: &[[f32; 3]], v0: usize, v1: usize) -> [f32; 3] {
    let a = positions[v0];
    let b = positions[v1];
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    if len < f32::EPSILON {
        return [0.0; 3];
    }
    [dx / len, dy / len, dz / len]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ]
    }

    #[test]
    fn test_edge_slide_vert_half() {
        let pos = line();
        let p = edge_slide_vert(&pos, 0, 1, 0.5);
        assert!((p[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_edge_slide_vert_full() {
        let pos = line();
        let p = edge_slide_vert(&pos, 0, 1, 1.0);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_edge_slide_edge() {
        let pos = line();
        let (a, b) = edge_slide_edge(&pos, 0, 1, 1, 2, 1.0);
        assert!((a[0] - 1.0).abs() < 1e-5);
        assert!((b[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_edge_length() {
        let pos = line();
        assert!((edge_length(&pos, 0, 1) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_edge_direction() {
        let pos = line();
        let d = edge_direction(&pos, 0, 1);
        assert!((d[0] - 1.0).abs() < 1e-5);
    }
}
