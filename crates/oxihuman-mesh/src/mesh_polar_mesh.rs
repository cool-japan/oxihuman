// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::{PI, TAU};

/// Polar mesh generated from rings and sectors.
#[allow(dead_code)]
pub struct PolarMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub rings: usize,
    pub sectors: usize,
}

/// Generate a polar mesh (latitude/longitude grid on a hemisphere or sphere).
#[allow(dead_code)]
pub fn generate_polar_mesh(rings: usize, sectors: usize, radius: f32) -> PolarMesh {
    assert!(rings >= 1);
    assert!(sectors >= 3);
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for r in 0..=rings {
        let phi = PI * r as f32 / rings as f32; // 0 to PI
        for s in 0..=sectors {
            let theta = TAU * s as f32 / sectors as f32;
            let x = radius * phi.sin() * theta.cos();
            let y = radius * phi.cos();
            let z = radius * phi.sin() * theta.sin();
            positions.push([x, y, z]);
        }
    }

    let stride = sectors + 1;
    for r in 0..rings {
        for s in 0..sectors {
            let a = (r * stride + s) as u32;
            let b = (r * stride + s + 1) as u32;
            let c = ((r + 1) * stride + s + 1) as u32;
            let d = ((r + 1) * stride + s) as u32;
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }

    PolarMesh {
        positions,
        indices,
        rings,
        sectors,
    }
}

/// Count faces in a polar mesh.
#[allow(dead_code)]
pub fn polar_face_count(rings: usize, sectors: usize) -> usize {
    rings * sectors * 2
}

/// Count vertices in a polar mesh.
#[allow(dead_code)]
pub fn polar_vertex_count(rings: usize, sectors: usize) -> usize {
    (rings + 1) * (sectors + 1)
}

/// Compute the bounding radius of the mesh.
#[allow(dead_code)]
pub fn polar_bounding_radius(mesh: &PolarMesh) -> f32 {
    mesh.positions
        .iter()
        .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt())
        .fold(0.0_f32, f32::max)
}

/// Check that a polar mesh is non-empty.
#[allow(dead_code)]
pub fn polar_is_valid(mesh: &PolarMesh) -> bool {
    !mesh.positions.is_empty() && !mesh.indices.is_empty()
}

/// Serialize a polar mesh description to JSON stub.
#[allow(dead_code)]
pub fn polar_to_json(mesh: &PolarMesh) -> String {
    format!(
        r#"{{"rings":{},"sectors":{},"vertices":{},"faces":{}}}"#,
        mesh.rings,
        mesh.sectors,
        mesh.positions.len(),
        mesh.indices.len() / 3
    )
}

/// Scale all positions of a polar mesh by a factor.
#[allow(dead_code)]
pub fn polar_scale(mesh: &mut PolarMesh, factor: f32) {
    for p in &mut mesh.positions {
        p[0] *= factor;
        p[1] *= factor;
        p[2] *= factor;
    }
}

/// Return the index count for a polar mesh.
#[allow(dead_code)]
pub fn polar_index_count(rings: usize, sectors: usize) -> usize {
    rings * sectors * 6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_count_correct() {
        let m = generate_polar_mesh(4, 8, 1.0);
        assert_eq!(m.positions.len(), polar_vertex_count(4, 8));
    }

    #[test]
    fn face_count_correct() {
        let m = generate_polar_mesh(4, 8, 1.0);
        assert_eq!(m.indices.len() / 3, polar_face_count(4, 8));
    }

    #[test]
    fn bounding_radius_near_one() {
        let m = generate_polar_mesh(8, 16, 1.0);
        let r = polar_bounding_radius(&m);
        assert!((r - 1.0).abs() < 0.02);
    }

    #[test]
    fn is_valid_nonempty() {
        let m = generate_polar_mesh(2, 4, 1.0);
        assert!(polar_is_valid(&m));
    }

    #[test]
    fn to_json_contains_rings() {
        let m = generate_polar_mesh(3, 6, 1.0);
        let j = polar_to_json(&m);
        assert!(j.contains("\"rings\":3"));
    }

    #[test]
    fn scale_doubles_radius() {
        let mut m = generate_polar_mesh(4, 8, 1.0);
        polar_scale(&mut m, 2.0);
        let r = polar_bounding_radius(&m);
        assert!((r - 2.0).abs() < 0.05);
    }

    #[test]
    fn index_count_formula() {
        assert_eq!(polar_index_count(4, 8), 4 * 8 * 6);
    }

    #[test]
    fn indices_in_bounds() {
        let m = generate_polar_mesh(4, 8, 1.0);
        let n = m.positions.len() as u32;
        assert!(m.indices.iter().all(|&i| i < n));
    }

    #[test]
    fn single_ring_sector() {
        let m = generate_polar_mesh(1, 3, 1.0);
        assert!(polar_is_valid(&m));
    }

    #[test]
    fn zero_radius_origin() {
        let m = generate_polar_mesh(2, 4, 0.0);
        assert!(m
            .positions
            .iter()
            .all(|p| p[0].abs() < 1e-6 && p[2].abs() < 1e-6));
    }
}
