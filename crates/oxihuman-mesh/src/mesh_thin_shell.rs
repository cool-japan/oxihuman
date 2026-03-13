// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Thin-shell mesh generation: offset a surface inward/outward by a small thickness.

use std::f32::consts::PI;

/// Result of a thin-shell operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThinShellResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub thickness: f32,
}

/// Configuration for thin-shell generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThinShellConfig {
    pub thickness: f32,
    pub outward: bool,
    pub cap_open_edges: bool,
}

impl Default for ThinShellConfig {
    fn default() -> Self {
        Self {
            thickness: 0.01,
            outward: true,
            cap_open_edges: false,
        }
    }
}

/// Compute a per-vertex normal from a triangle mesh.
#[allow(dead_code)]
pub fn compute_vertex_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0_f32; 3]; positions.len()];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let pa = positions[a];
        let pb = positions[b];
        let pc = positions[c];
        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let n = cross(ab, ac);
        for &vi in &[a, b, c] {
            normals[vi][0] += n[0];
            normals[vi][1] += n[1];
            normals[vi][2] += n[2];
        }
    }
    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-8 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        }
    }
    normals
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Offset each vertex along its normal by `thickness`.
#[allow(dead_code)]
pub fn offset_by_normal(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    thickness: f32,
) -> Vec<[f32; 3]> {
    positions
        .iter()
        .zip(normals.iter())
        .map(|(p, n)| {
            [
                p[0] + n[0] * thickness,
                p[1] + n[1] * thickness,
                p[2] + n[2] * thickness,
            ]
        })
        .collect()
}

/// Generate a thin shell by duplicating and offsetting vertices.
#[allow(dead_code)]
pub fn thin_shell(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &ThinShellConfig,
) -> ThinShellResult {
    let sign = if config.outward { 1.0 } else { -1.0 };
    let normals = compute_vertex_normals(positions, indices);
    let offset_pos = offset_by_normal(positions, &normals, sign * config.thickness);

    let mut new_positions = positions.to_vec();
    new_positions.extend_from_slice(&offset_pos);

    let base_offset = positions.len() as u32;
    let mut new_indices = indices.to_vec();
    // Flip winding for inner shell
    for tri in indices.chunks_exact(3) {
        new_indices.push(tri[0] + base_offset);
        new_indices.push(tri[2] + base_offset);
        new_indices.push(tri[1] + base_offset);
    }

    ThinShellResult {
        positions: new_positions,
        indices: new_indices,
        thickness: config.thickness,
    }
}

/// Estimate the volume enclosed by a thin shell (approx).
#[allow(dead_code)]
pub fn shell_volume(result: &ThinShellResult) -> f32 {
    let n = result.positions.len() / 2;
    let surface_area: f32 = result.indices[..result.indices.len() / 2]
        .chunks_exact(3)
        .map(|tri| {
            let a = result.positions[tri[0] as usize];
            let b = result.positions[tri[1] as usize];
            let c = result.positions[tri[2] as usize];
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cr = cross(ab, ac);
            0.5 * (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt()
        })
        .sum();
    // rough approximation: volume ≈ area * thickness
    let _ = n;
    surface_area * result.thickness
}

/// Return the number of triangle faces in the shell result.
#[allow(dead_code)]
pub fn shell_face_count(result: &ThinShellResult) -> usize {
    result.indices.len() / 3
}

/// Compute a reference angular sweep to verify PI usage.
#[allow(dead_code)]
pub fn half_turn_radians() -> f32 {
    PI
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tri_pos() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }
    fn unit_tri_idx() -> Vec<u32> {
        vec![0, 1, 2]
    }

    #[test]
    fn test_default_config() {
        let cfg = ThinShellConfig::default();
        assert!((0.0..=1.0).contains(&cfg.thickness));
    }

    #[test]
    fn test_vertex_normals_upward() {
        let n = compute_vertex_normals(&unit_tri_pos(), &unit_tri_idx());
        assert_eq!(n.len(), 3);
        assert!(n[0][2] > 0.5);
    }

    #[test]
    fn test_offset_by_normal_direction() {
        let pos = vec![[0.0_f32; 3]];
        let normals = vec![[0.0, 0.0, 1.0]];
        let out = offset_by_normal(&pos, &normals, 1.0);
        assert!((out[0][2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_thin_shell_doubles_vertices() {
        let cfg = ThinShellConfig::default();
        let res = thin_shell(&unit_tri_pos(), &unit_tri_idx(), &cfg);
        assert_eq!(res.positions.len(), 6);
    }

    #[test]
    fn test_thin_shell_doubles_faces() {
        let cfg = ThinShellConfig::default();
        let res = thin_shell(&unit_tri_pos(), &unit_tri_idx(), &cfg);
        assert_eq!(shell_face_count(&res), 2);
    }

    #[test]
    fn test_shell_volume_positive() {
        let cfg = ThinShellConfig::default();
        let res = thin_shell(&unit_tri_pos(), &unit_tri_idx(), &cfg);
        assert!(shell_volume(&res) > 0.0);
    }

    #[test]
    fn test_inward_offset() {
        let cfg = ThinShellConfig {
            outward: false,
            ..Default::default()
        };
        let res = thin_shell(&unit_tri_pos(), &unit_tri_idx(), &cfg);
        // inner vertices should have negative z compared to outer
        let outer_z = res.positions[0][2];
        let inner_z = res.positions[3][2];
        assert!(inner_z <= outer_z);
    }

    #[test]
    fn test_half_turn() {
        assert!((half_turn_radians() - std::f32::consts::PI).abs() < 1e-6);
    }

    #[test]
    fn test_cross_product() {
        let a = [1.0_f32, 0.0, 0.0];
        let b = [0.0_f32, 1.0, 0.0];
        let c = cross(a, b);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_thickness_stored() {
        let cfg = ThinShellConfig {
            thickness: 0.05,
            ..Default::default()
        };
        let res = thin_shell(&unit_tri_pos(), &unit_tri_idx(), &cfg);
        assert!((res.thickness - 0.05).abs() < 1e-6);
    }
}
