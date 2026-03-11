// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::mesh::MeshBuffers;
use std::f32::consts::PI;

/// UV projection method.
#[derive(Debug, Clone, Copy)]
pub enum UvProjection {
    /// Cylindrical projection: u=angle/(2π), v=height/(max_y-min_y)
    Cylindrical,
    /// Spherical projection: u=longitude, v=latitude
    Spherical,
    /// Planar projection from above (XZ plane): u=x/width, v=z/depth
    PlanarTop,
    /// Planar projection from front (XY plane): u=x/width, v=y/height
    PlanarFront,
    /// Box projection: choose the face with the dominant normal component
    Box,
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn compute_centroid(positions: &[[f32; 3]]) -> [f32; 3] {
    let n = positions.len().max(1) as f32;
    let cx = positions.iter().map(|p| p[0]).sum::<f32>() / n;
    let cy = positions.iter().map(|p| p[1]).sum::<f32>() / n;
    let cz = positions.iter().map(|p| p[2]).sum::<f32>() / n;
    [cx, cy, cz]
}

fn project_cylindrical(positions: &[[f32; 3]]) -> Vec<[f32; 2]> {
    let n = positions.len().max(1) as f32;
    let center_x = positions.iter().map(|p| p[0]).sum::<f32>() / n;
    let center_z = positions.iter().map(|p| p[2]).sum::<f32>() / n;
    let min_y = positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    let max_y = positions.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
    let height = (max_y - min_y).max(1e-6);

    positions
        .iter()
        .map(|p| {
            let dx = p[0] - center_x;
            let dz = p[2] - center_z;
            let u = (dz.atan2(dx) / (2.0 * PI) + 0.5).rem_euclid(1.0);
            let v = (p[1] - min_y) / height;
            [u, v]
        })
        .collect()
}

fn project_spherical(positions: &[[f32; 3]]) -> Vec<[f32; 2]> {
    let center = compute_centroid(positions);
    positions
        .iter()
        .map(|p| {
            let dx = p[0] - center[0];
            let dy = p[1] - center[1];
            let dz = p[2] - center[2];
            let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-10);
            let theta = (dy / len).acos(); // polar angle [0..π]
            let phi = dz.atan2(dx); // azimuthal angle [-π..π]
            let u = (phi / (2.0 * PI) + 0.5).rem_euclid(1.0);
            let v = theta / PI;
            [u, v]
        })
        .collect()
}

fn project_planar_top(positions: &[[f32; 3]]) -> Vec<[f32; 2]> {
    let min_x = positions.iter().map(|p| p[0]).fold(f32::MAX, f32::min);
    let max_x = positions.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
    let min_z = positions.iter().map(|p| p[2]).fold(f32::MAX, f32::min);
    let max_z = positions.iter().map(|p| p[2]).fold(f32::MIN, f32::max);
    let width = (max_x - min_x).max(1e-6);
    let depth = (max_z - min_z).max(1e-6);
    positions
        .iter()
        .map(|p| {
            let u = (p[0] - min_x) / width;
            let v = (p[2] - min_z) / depth;
            [u, v]
        })
        .collect()
}

fn project_planar_front(positions: &[[f32; 3]]) -> Vec<[f32; 2]> {
    let min_x = positions.iter().map(|p| p[0]).fold(f32::MAX, f32::min);
    let max_x = positions.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
    let min_y = positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    let max_y = positions.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
    let width = (max_x - min_x).max(1e-6);
    let height = (max_y - min_y).max(1e-6);
    positions
        .iter()
        .map(|p| {
            let u = (p[0] - min_x) / width;
            let v = (p[1] - min_y) / height;
            [u, v]
        })
        .collect()
}

fn project_box(positions: &[[f32; 3]], normals: &[[f32; 3]]) -> Vec<[f32; 2]> {
    let min_x = positions.iter().map(|p| p[0]).fold(f32::MAX, f32::min);
    let max_x = positions.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
    let min_y = positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    let max_y = positions.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
    let min_z = positions.iter().map(|p| p[2]).fold(f32::MAX, f32::min);
    let max_z = positions.iter().map(|p| p[2]).fold(f32::MIN, f32::max);
    let width = (max_x - min_x).max(1e-6);
    let height = (max_y - min_y).max(1e-6);
    let depth = (max_z - min_z).max(1e-6);

    positions
        .iter()
        .zip(normals.iter())
        .map(|(p, n)| {
            let nx = n[0].abs();
            let ny = n[1].abs();
            let nz = n[2].abs();
            if nx > ny && nx > nz {
                // left/right face → u=z/depth, v=y/height
                let u = (p[2] - min_z) / depth;
                let v = (p[1] - min_y) / height;
                [u, v]
            } else if ny > nx && ny > nz {
                // top/bottom face → u=x/width, v=z/depth
                let u = (p[0] - min_x) / width;
                let v = (p[2] - min_z) / depth;
                [u, v]
            } else {
                // front/back face → u=x/width, v=y/height
                let u = (p[0] - min_x) / width;
                let v = (p[1] - min_y) / height;
                [u, v]
            }
        })
        .collect()
}

// ── public API ────────────────────────────────────────────────────────────────

/// Generate UV coordinates using a procedural projection.
/// Returns a new `MeshBuffers` with updated `uvs` field (same length as `positions`).
pub fn project_uvs(mesh: &MeshBuffers, method: UvProjection) -> MeshBuffers {
    let uvs = match method {
        UvProjection::Cylindrical => project_cylindrical(&mesh.positions),
        UvProjection::Spherical => project_spherical(&mesh.positions),
        UvProjection::PlanarTop => project_planar_top(&mesh.positions),
        UvProjection::PlanarFront => project_planar_front(&mesh.positions),
        UvProjection::Box => project_box(&mesh.positions, &mesh.normals),
    };
    MeshBuffers {
        positions: mesh.positions.clone(),
        normals: mesh.normals.clone(),
        tangents: mesh.tangents.clone(),
        uvs,
        indices: mesh.indices.clone(),
        colors: mesh.colors.clone(),
        has_suit: mesh.has_suit,
    }
}

/// Normalize UV coordinates to [0..1] range.
pub fn normalize_uvs(uvs: &[[f32; 2]]) -> Vec<[f32; 2]> {
    if uvs.is_empty() {
        return Vec::new();
    }
    let min_u = uvs.iter().map(|uv| uv[0]).fold(f32::MAX, f32::min);
    let max_u = uvs.iter().map(|uv| uv[0]).fold(f32::MIN, f32::max);
    let min_v = uvs.iter().map(|uv| uv[1]).fold(f32::MAX, f32::min);
    let max_v = uvs.iter().map(|uv| uv[1]).fold(f32::MIN, f32::max);
    let range_u = (max_u - min_u).max(1e-6);
    let range_v = (max_v - min_v).max(1e-6);
    uvs.iter()
        .map(|uv| [(uv[0] - min_u) / range_u, (uv[1] - min_v) / range_v])
        .collect()
}

/// Flip UV V coordinate: v_new = 1.0 - v. (Some tools use Y-up UV conventions.)
pub fn flip_v(uvs: &[[f32; 2]]) -> Vec<[f32; 2]> {
    uvs.iter().map(|uv| [uv[0], 1.0 - uv[1]]).collect()
}

/// Tile UVs by a scale factor: u_new = u * scale_u, v_new = v * scale_v.
pub fn tile_uvs(uvs: &[[f32; 2]], scale_u: f32, scale_v: f32) -> Vec<[f32; 2]> {
    uvs.iter()
        .map(|uv| [uv[0] * scale_u, uv[1] * scale_v])
        .collect()
}

/// Offset UVs: u_new = u + offset_u, v_new = v + offset_v.
pub fn offset_uvs(uvs: &[[f32; 2]], offset_u: f32, offset_v: f32) -> Vec<[f32; 2]> {
    uvs.iter()
        .map(|uv| [uv[0] + offset_u, uv[1] + offset_v])
        .collect()
}

/// Rotate UVs around the UV center (0.5, 0.5) by `angle_rad` radians.
pub fn rotate_uvs(uvs: &[[f32; 2]], angle_rad: f32) -> Vec<[f32; 2]> {
    let (sin_a, cos_a) = angle_rad.sin_cos();
    uvs.iter()
        .map(|uv| {
            let du = uv[0] - 0.5;
            let dv = uv[1] - 0.5;
            let u_new = cos_a * du - sin_a * dv + 0.5;
            let v_new = sin_a * du + cos_a * dv + 0.5;
            [u_new, v_new]
        })
        .collect()
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere_points() -> MeshBuffers {
        MeshBuffers {
            positions: vec![
                [-1.0, -1.0, -1.0],
                [1.0, -1.0, -1.0],
                [1.0, 1.0, -1.0],
                [-1.0, 1.0, -1.0],
                [-1.0, -1.0, 1.0],
                [1.0, -1.0, 1.0],
                [1.0, 1.0, 1.0],
                [-1.0, 1.0, 1.0],
            ],
            normals: vec![[0.0, 1.0, 0.0]; 8],
            uvs: vec![[0.0, 0.0]; 8],
            tangents: vec![],
            colors: None,
            indices: vec![
                0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 0, 4, 7, 0, 7, 3, 1, 5, 6, 1, 6, 2, 0, 1, 5, 0,
                5, 4, 3, 2, 6, 3, 6, 7,
            ],
            has_suit: true,
        }
    }

    #[test]
    fn cylindrical_uv_count() {
        let mesh = sphere_points();
        let out = project_uvs(&mesh, UvProjection::Cylindrical);
        assert_eq!(out.uvs.len(), 8);
    }

    #[test]
    fn cylindrical_u_in_range() {
        let mesh = sphere_points();
        let out = project_uvs(&mesh, UvProjection::Cylindrical);
        for uv in &out.uvs {
            assert!(uv[0] >= 0.0 && uv[0] <= 1.0, "u out of range: {}", uv[0]);
        }
    }

    #[test]
    fn cylindrical_v_in_range() {
        let mesh = sphere_points();
        let out = project_uvs(&mesh, UvProjection::Cylindrical);
        for uv in &out.uvs {
            assert!(uv[1] >= 0.0 && uv[1] <= 1.0, "v out of range: {}", uv[1]);
        }
    }

    #[test]
    fn spherical_u_in_range() {
        let mesh = sphere_points();
        let out = project_uvs(&mesh, UvProjection::Spherical);
        for uv in &out.uvs {
            assert!(uv[0] >= 0.0 && uv[0] <= 1.0, "u out of range: {}", uv[0]);
        }
    }

    #[test]
    fn normalize_uvs_result_in_range() {
        let uvs = vec![[0.2f32, 0.5], [0.8, 0.1], [0.5, 0.9]];
        let norm = normalize_uvs(&uvs);
        for uv in &norm {
            assert!(uv[0] >= 0.0 && uv[0] <= 1.0, "u out of range: {}", uv[0]);
            assert!(uv[1] >= 0.0 && uv[1] <= 1.0, "v out of range: {}", uv[1]);
        }
    }

    #[test]
    fn flip_v_inverts() {
        let result = flip_v(&[[0.3, 0.7]]);
        assert!(
            (result[0][1] - 0.3).abs() < 1e-6,
            "flip_v failed: {}",
            result[0][1]
        );
    }

    #[test]
    fn tile_uvs_doubles() {
        let uvs = vec![[0.4f32, 0.3]];
        let tiled = tile_uvs(&uvs, 2.0, 2.0);
        assert!((tiled[0][0] - uvs[0][0] * 2.0).abs() < 1e-6);
    }

    #[test]
    fn offset_uvs_shifts() {
        let uvs = vec![[0.2f32, 0.3], [0.5, 0.6]];
        let shifted = offset_uvs(&uvs, 0.1, 0.0);
        for (orig, sh) in uvs.iter().zip(shifted.iter()) {
            assert!((sh[0] - (orig[0] + 0.1)).abs() < 1e-6);
        }
    }

    #[test]
    fn rotate_uvs_at_zero_unchanged() {
        let uvs = vec![[0.3f32, 0.7], [0.5, 0.5], [0.1, 0.9]];
        let rotated = rotate_uvs(&uvs, 0.0);
        for (orig, rot) in uvs.iter().zip(rotated.iter()) {
            assert!((rot[0] - orig[0]).abs() < 1e-6);
            assert!((rot[1] - orig[1]).abs() < 1e-6);
        }
    }

    #[test]
    fn planar_top_uv_count() {
        let mesh = sphere_points();
        let out = project_uvs(&mesh, UvProjection::PlanarTop);
        assert_eq!(out.uvs.len(), mesh.positions.len());
    }
}
