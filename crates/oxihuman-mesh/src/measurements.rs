// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Body measurement extraction from a MeshBuffers.
//!
//! Computes bounding box dimensions and heuristic body measurements
//! for use in parameter profiles and UI feedback.

use crate::mesh::MeshBuffers;

/// Axis-aligned bounding box of the mesh.
#[derive(Debug, Clone, PartialEq)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb {
    /// Overall height of the bounding box (Y axis in MakeHuman coordinates).
    pub fn height(&self) -> f32 {
        self.max[1] - self.min[1]
    }

    /// Width (X axis).
    pub fn width(&self) -> f32 {
        self.max[0] - self.min[0]
    }

    /// Depth (Z axis).
    pub fn depth(&self) -> f32 {
        self.max[2] - self.min[2]
    }

    /// Center of the bounding box.
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }
}

/// Compute the axis-aligned bounding box of a mesh.
pub fn compute_aabb(buf: &MeshBuffers) -> Option<Aabb> {
    if buf.positions.is_empty() {
        return None;
    }
    let mut min = buf.positions[0];
    let mut max = buf.positions[0];
    for p in &buf.positions {
        for i in 0..3 {
            if p[i] < min[i] {
                min[i] = p[i];
            }
            if p[i] > max[i] {
                max[i] = p[i];
            }
        }
    }
    Some(Aabb { min, max })
}

/// Body measurements extracted from the mesh.
#[derive(Debug, Clone)]
pub struct BodyMeasurements {
    /// Total height of the mesh bounding box (Y axis).
    pub total_height: f32,
    /// Width at the widest point (X axis extent).
    pub max_width: f32,
    /// Depth at the deepest point (Z axis extent).
    pub max_depth: f32,
    /// Approximate torso height (50%–80% of total height).
    pub torso_height: f32,
    /// Approximate shoulder width: width at 75% of total height.
    pub shoulder_width: f32,
    /// Approximate waist width: width at 55% of total height.
    pub waist_width: f32,
    /// Approximate hip width: width at 35% of total height.
    pub hip_width: f32,
}

/// Compute heuristic body measurements from vertex positions.
pub fn compute_measurements(buf: &MeshBuffers) -> Option<BodyMeasurements> {
    let aabb = compute_aabb(buf)?;
    let total_height = aabb.height();

    if total_height < 1e-6 {
        return None;
    }

    // For each horizontal slice, find max X width
    let y_min = aabb.min[1];

    // Sample the mesh at various height fractions
    let shoulder_w = width_at_height_fraction(buf, y_min, total_height, 0.75);
    let waist_w = width_at_height_fraction(buf, y_min, total_height, 0.55);
    let hip_w = width_at_height_fraction(buf, y_min, total_height, 0.35);

    Some(BodyMeasurements {
        total_height,
        max_width: aabb.width(),
        max_depth: aabb.depth(),
        torso_height: total_height * 0.30, // torso ≈ 30% of total
        shoulder_width: shoulder_w,
        waist_width: waist_w,
        hip_width: hip_w,
    })
}

/// Find the width (X extent) of vertices within ±5% of a given height fraction.
fn width_at_height_fraction(
    buf: &MeshBuffers,
    y_min: f32,
    total_height: f32,
    fraction: f32,
) -> f32 {
    let target_y = y_min + total_height * fraction;
    let tolerance = total_height * 0.05;

    let mut x_min = f32::INFINITY;
    let mut x_max = f32::NEG_INFINITY;
    let mut found = false;

    for p in &buf.positions {
        if (p[1] - target_y).abs() <= tolerance {
            if p[0] < x_min {
                x_min = p[0];
            }
            if p[0] > x_max {
                x_max = p[0];
            }
            found = true;
        }
    }

    if found {
        x_max - x_min
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers as MyMesh;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn unit_cube_verts() -> MyMesh {
        // 8 corners of a unit cube
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
        ];
        MyMesh::from_morph(MB {
            positions,
            normals: vec![[0.0, 1.0, 0.0]; 8],
            uvs: vec![[0.0, 0.0]; 8],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    #[test]
    fn aabb_unit_cube() {
        let m = unit_cube_verts();
        let aabb = compute_aabb(&m).expect("should succeed");
        assert!((aabb.height() - 1.0).abs() < 1e-6);
        assert!((aabb.width() - 1.0).abs() < 1e-6);
        assert!((aabb.depth() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn aabb_center() {
        let m = unit_cube_verts();
        let aabb = compute_aabb(&m).expect("should succeed");
        let c = aabb.center();
        assert!((c[0] - 0.5).abs() < 1e-6);
        assert!((c[1] - 0.5).abs() < 1e-6);
        assert!((c[2] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn empty_mesh_returns_none() {
        let m = MyMesh::from_morph(MB {
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            indices: vec![],
            has_suit: false,
        });
        assert!(compute_aabb(&m).is_none());
        assert!(compute_measurements(&m).is_none());
    }

    #[test]
    fn measurements_non_negative() {
        let m = unit_cube_verts();
        let meas = compute_measurements(&m).expect("should succeed");
        assert!(meas.total_height > 0.0);
        assert!(meas.max_width > 0.0);
        assert!(meas.shoulder_width >= 0.0);
        assert!(meas.waist_width >= 0.0);
        assert!(meas.hip_width >= 0.0);
    }

    #[test]
    fn real_base_mesh_measurements() {
        use oxihuman_core::parser::obj::parse_obj;
        let path = std::env::var("MAKEHUMAN_DATA_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("/tmp/oxihuman_nonexistent_data"))
            .join("3dobjs/base.obj");
        if let Ok(src) = std::fs::read_to_string(&path) {
            if let Ok(obj) = parse_obj(&src) {
                let morph_buf = MB {
                    positions: obj.positions,
                    normals: obj.normals,
                    uvs: obj.uvs,
                    indices: obj.indices,
                    has_suit: false,
                };
                let mesh = MyMesh::from_morph(morph_buf);
                let meas = compute_measurements(&mesh).expect("should succeed");
                // MakeHuman base mesh is roughly 1.7m tall in its units
                assert!(
                    meas.total_height > 1.0,
                    "height too small: {}",
                    meas.total_height
                );
                assert!(meas.shoulder_width > 0.0);
            }
        }
    }
}
