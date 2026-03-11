// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::TAU;

/// Parameters for converting a polyline curve to a tube mesh.
pub struct CurveMeshParams {
    pub profile_radius: f32,
    pub profile_sides: usize,
    pub cap_ends: bool,
}

pub fn new_curve_mesh_params(radius: f32, sides: usize) -> CurveMeshParams {
    CurveMeshParams {
        profile_radius: radius.max(0.0),
        profile_sides: sides.max(3),
        cap_ends: true,
    }
}

pub fn curve_to_mesh_vertex_count(curve_points: usize, params: &CurveMeshParams) -> usize {
    if curve_points < 2 {
        return 0;
    }
    let ring_verts = curve_points * params.profile_sides;
    let cap_verts = if params.cap_ends { 2 } else { 0 };
    ring_verts + cap_verts
}

pub fn curve_to_mesh_face_count(curve_points: usize, params: &CurveMeshParams) -> usize {
    if curve_points < 2 {
        return 0;
    }
    let tube_faces = (curve_points - 1) * params.profile_sides;
    let cap_faces = if params.cap_ends {
        params.profile_sides * 2
    } else {
        0
    };
    tube_faces + cap_faces
}

pub fn curve_segment_ring(
    center: [f32; 3],
    normal: [f32; 3],
    radius: f32,
    sides: usize,
) -> Vec<[f32; 3]> {
    let sides = sides.max(3);
    let n = {
        let l = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
            .sqrt()
            .max(1e-9);
        [normal[0] / l, normal[1] / l, normal[2] / l]
    };
    // build perpendicular in arbitrary direction
    let perp = if n[0].abs() < 0.9 {
        let l = (n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
        [0.0, -n[2] / l, n[1] / l]
    } else {
        let l = (n[0] * n[0] + n[2] * n[2]).sqrt().max(1e-9);
        [n[2] / l, 0.0, -n[0] / l]
    };
    let bi = [
        n[1] * perp[2] - n[2] * perp[1],
        n[2] * perp[0] - n[0] * perp[2],
        n[0] * perp[1] - n[1] * perp[0],
    ];
    let mut verts = Vec::with_capacity(sides);
    for s in 0..sides {
        let angle = TAU * s as f32 / sides as f32;
        let c = angle.cos();
        let sn = angle.sin();
        verts.push([
            center[0] + radius * (c * perp[0] + sn * bi[0]),
            center[1] + radius * (c * perp[1] + sn * bi[1]),
            center[2] + radius * (c * perp[2] + sn * bi[2]),
        ]);
    }
    verts
}

pub fn curve_length(points: &[[f32; 3]]) -> f32 {
    let n = points.len();
    if n < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    for i in 0..n - 1 {
        let a = points[i];
        let b = points[i + 1];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_curve_mesh_params() {
        let p = new_curve_mesh_params(0.5, 8);
        assert!((p.profile_radius - 0.5).abs() < 1e-6);
        assert_eq!(p.profile_sides, 8);
    }

    #[test]
    fn test_curve_to_mesh_min_sides() {
        let p = new_curve_mesh_params(1.0, 0);
        assert_eq!(p.profile_sides, 3);
    }

    #[test]
    fn test_curve_to_mesh_vertex_count() {
        let p = new_curve_mesh_params(1.0, 6);
        let count = curve_to_mesh_vertex_count(5, &p);
        assert_eq!(count, 5 * 6 + 2);
    }

    #[test]
    fn test_curve_to_mesh_face_count() {
        let p = new_curve_mesh_params(1.0, 6);
        let count = curve_to_mesh_face_count(5, &p);
        assert_eq!(count, 4 * 6 + 6 * 2);
    }

    #[test]
    fn test_curve_segment_ring_count() {
        let ring = curve_segment_ring([0.0; 3], [0.0, 0.0, 1.0], 1.0, 8);
        assert_eq!(ring.len(), 8);
    }

    #[test]
    fn test_curve_length() {
        let pts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        assert!((curve_length(&pts) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_curve_to_mesh_zero_points() {
        let p = new_curve_mesh_params(1.0, 6);
        assert_eq!(curve_to_mesh_vertex_count(0, &p), 0);
        assert_eq!(curve_to_mesh_face_count(1, &p), 0);
    }
}
