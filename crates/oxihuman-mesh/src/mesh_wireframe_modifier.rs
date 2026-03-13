// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::TAU;

/// Parameters for wireframe modifier.
pub struct WireframeParams {
    pub thickness: f32,
    pub use_replace: bool,
    pub boundary_edges_only: bool,
}

pub fn new_wireframe_params(thickness: f32) -> WireframeParams {
    WireframeParams {
        thickness: thickness.max(0.0),
        use_replace: true,
        boundary_edges_only: false,
    }
}

pub fn wireframe_edge_tube_verts(
    start: [f32; 3],
    end: [f32; 3],
    thickness: f32,
    sides: usize,
) -> Vec<[f32; 3]> {
    let sides = sides.max(3);
    let r = thickness * 0.5;
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let dz = end[2] - start[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-9);
    let nx = dx / len;
    let ny = dy / len;
    let nz = dz / len;
    // perpendicular
    let (px, py, pz) = if nx.abs() < 0.9 {
        let (cx, cy, cz) = (
            ny * 0.0 - nz * 0.0,
            nz * 1.0 - nx * 0.0,
            nx * 0.0 - ny * 1.0,
        );
        let l = (cx * cx + cy * cy + cz * cz).sqrt().max(1e-9);
        (cx / l, cy / l, cz / l)
    } else {
        let (cx, cy, cz) = (
            ny * 1.0 - nz * 0.0,
            nz * 0.0 - nx * 1.0,
            nx * 0.0 - ny * 0.0,
        );
        let l = (cx * cx + cy * cy + cz * cz).sqrt().max(1e-9);
        (cx / l, cy / l, cz / l)
    };
    // second perp via cross
    let (qx, qy, qz) = (ny * pz - nz * py, nz * px - nx * pz, nx * py - ny * px);
    let mut verts = Vec::with_capacity(sides * 2);
    for ring in 0..2 {
        let base = if ring == 0 { start } else { end };
        for s in 0..sides {
            let angle = TAU * s as f32 / sides as f32;
            let c = angle.cos();
            let sn = angle.sin();
            verts.push([
                base[0] + r * (c * px + sn * qx),
                base[1] + r * (c * py + sn * qy),
                base[2] + r * (c * pz + sn * qz),
            ]);
        }
    }
    verts
}

pub fn wireframe_edge_count(face_count: usize, avg_verts_per_face: usize) -> usize {
    face_count * avg_verts_per_face / 2
}

pub fn wireframe_vertex_count(edge_count: usize, sides: usize) -> usize {
    edge_count * sides * 2
}

pub fn wireframe_tube_length(start: [f32; 3], end: [f32; 3]) -> f32 {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let dz = end[2] - start[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wireframe_params() {
        let p = new_wireframe_params(0.02);
        assert!((p.thickness - 0.02).abs() < 1e-6);
    }

    #[test]
    fn test_wireframe_params_negative_clamped() {
        let p = new_wireframe_params(-1.0);
        assert!(p.thickness >= 0.0);
    }

    #[test]
    fn test_wireframe_edge_tube_vert_count() {
        let verts = wireframe_edge_tube_verts([0.0; 3], [1.0, 0.0, 0.0], 0.1, 6);
        assert_eq!(verts.len(), 12);
    }

    #[test]
    fn test_wireframe_edge_count() {
        assert_eq!(wireframe_edge_count(10, 4), 20);
    }

    #[test]
    fn test_wireframe_vertex_count() {
        assert_eq!(wireframe_vertex_count(20, 6), 240);
    }

    #[test]
    fn test_wireframe_tube_length() {
        let l = wireframe_tube_length([0.0; 3], [3.0, 4.0, 0.0]);
        assert!((l - 5.0).abs() < 1e-5);
    }
}
