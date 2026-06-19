// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! CSG boolean operations stub via SDF sampling (union/intersect/subtract).

/// CSG operation type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsgOp {
    Union,
    Intersect,
    Subtract,
}

/// A signed-distance sphere primitive.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SdfSphere {
    pub center: [f32; 3],
    pub radius: f32,
}

impl SdfSphere {
    #[allow(dead_code)]
    pub fn eval(&self, p: [f32; 3]) -> f32 {
        let dx = p[0] - self.center[0];
        let dy = p[1] - self.center[1];
        let dz = p[2] - self.center[2];
        (dx * dx + dy * dy + dz * dz).sqrt() - self.radius
    }
}

/// A signed-distance box primitive.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SdfBox {
    pub center: [f32; 3],
    pub half_size: [f32; 3],
}

impl SdfBox {
    #[allow(dead_code)]
    pub fn eval(&self, p: [f32; 3]) -> f32 {
        let qx = (p[0] - self.center[0]).abs() - self.half_size[0];
        let qy = (p[1] - self.center[1]).abs() - self.half_size[1];
        let qz = (p[2] - self.center[2]).abs() - self.half_size[2];
        let mx = qx.max(0.0);
        let my = qy.max(0.0);
        let mz = qz.max(0.0);
        (mx * mx + my * my + mz * mz).sqrt() + qx.max(qy).max(qz).min(0.0)
    }
}

/// Combine two SDF values with a CSG operation.
#[allow(dead_code)]
pub fn csg_combine(a: f32, b: f32, op: CsgOp) -> f32 {
    match op {
        CsgOp::Union => a.min(b),
        CsgOp::Intersect => a.max(b),
        CsgOp::Subtract => a.max(-b),
    }
}

/// Sample a CSG result on a grid and return cells where SDF < 0 (inside).
/// Grid spans [-extent, extent] in each axis with `resolution` steps.
#[allow(dead_code)]
pub fn sample_csg_grid(
    a: &SdfSphere,
    b: &SdfBox,
    op: CsgOp,
    resolution: usize,
    extent: f32,
) -> Vec<[f32; 3]> {
    let mut inside = Vec::new();
    let step = 2.0 * extent / resolution as f32;
    for ix in 0..resolution {
        for iy in 0..resolution {
            for iz in 0..resolution {
                let x = -extent + (ix as f32 + 0.5) * step;
                let y = -extent + (iy as f32 + 0.5) * step;
                let z = -extent + (iz as f32 + 0.5) * step;
                let p = [x, y, z];
                let val = csg_combine(a.eval(p), b.eval(p), op);
                if val < 0.0 {
                    inside.push(p);
                }
            }
        }
    }
    inside
}

/// Count cells inside the CSG result.
#[allow(dead_code)]
pub fn csg_inside_count(pts: &[[f32; 3]]) -> usize {
    pts.len()
}

/// Extract an iso-surface mesh from the union/intersection/subtraction of an
/// `SdfSphere` and an `SdfBox` by sampling both on a regular 3-D grid and
/// running Marching Cubes at iso-value 0.
///
/// # Parameters
/// - `a`: signed-distance sphere primitive.
/// - `b`: signed-distance box primitive.
/// - `op`: which CSG operation to apply when combining the two SDFs.
/// - `resolution`: number of grid **steps** per axis.  The grid spans
///   `[-extent, +extent]³` with `resolution` cells; actual grid points lie at
///   cell centres.
/// - `extent`: half-width of the sampling domain in world units.
///
/// # Returns
/// A [`crate::mesh::MeshBuffers`] containing the extracted iso-surface.
#[allow(dead_code)]
pub fn csg_to_mesh(
    a: &SdfSphere,
    b: &SdfBox,
    op: CsgOp,
    resolution: usize,
    extent: f32,
) -> crate::mesh::MeshBuffers {
    use crate::marching_cubes::{marching_cubes, ScalarField};

    let res = resolution.max(2);
    let step = 2.0 * extent / res as f32;

    // ScalarField stores grid-point values (not cell-centres) so we use
    // `res + 1` points per axis to cover the full `[−extent, +extent]` domain
    // with the first and last points landing exactly on the boundary.
    let pts = res + 1;
    let mut field = ScalarField::new(
        [pts, pts, pts],
        [-extent, -extent, -extent],
        [step, step, step],
    );

    for iz in 0..pts {
        for iy in 0..pts {
            for ix in 0..pts {
                let p = field.world_pos(ix, iy, iz);
                let val = csg_combine(a.eval(p), b.eval(p), op);
                field.set(ix, iy, iz, val);
            }
        }
    }

    marching_cubes(&field, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_sphere() -> SdfSphere {
        SdfSphere {
            center: [0.0, 0.0, 0.0],
            radius: 1.0,
        }
    }
    fn unit_box() -> SdfBox {
        SdfBox {
            center: [0.0, 0.0, 0.0],
            half_size: [1.0, 1.0, 1.0],
        }
    }

    #[test]
    fn sphere_inside() {
        let s = unit_sphere();
        assert!(s.eval([0.0, 0.0, 0.0]) < 0.0);
    }

    #[test]
    fn sphere_outside() {
        let s = unit_sphere();
        assert!(s.eval([2.0, 0.0, 0.0]) > 0.0);
    }

    #[test]
    fn box_inside() {
        let b = unit_box();
        assert!(b.eval([0.0, 0.0, 0.0]) < 0.0);
    }

    #[test]
    fn box_outside() {
        let b = unit_box();
        assert!(b.eval([2.0, 0.0, 0.0]) > 0.0);
    }

    #[test]
    fn union_superset() {
        let s = unit_sphere();
        let b = unit_box();
        let u = sample_csg_grid(&s, &b, CsgOp::Union, 4, 2.0);
        let i = sample_csg_grid(&s, &b, CsgOp::Intersect, 4, 2.0);
        assert!(u.len() >= i.len());
    }

    #[test]
    fn subtract_less_than_a() {
        let s = unit_sphere();
        let b = unit_box();
        let sub = sample_csg_grid(&s, &b, CsgOp::Subtract, 4, 2.0);
        let union = sample_csg_grid(&s, &b, CsgOp::Union, 4, 2.0);
        assert!(sub.len() < union.len());
    }

    #[test]
    fn csg_combine_union() {
        assert!((csg_combine(-1.0, 2.0, CsgOp::Union) - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn csg_combine_intersect() {
        assert!((csg_combine(-1.0, 2.0, CsgOp::Intersect) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn csg_combine_subtract() {
        assert!((csg_combine(-1.0, -2.0, CsgOp::Subtract) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn inside_count_matches() {
        let s = unit_sphere();
        let b = unit_box();
        let pts = sample_csg_grid(&s, &b, CsgOp::Union, 4, 2.0);
        assert_eq!(csg_inside_count(&pts), pts.len());
    }

    #[test]
    fn csg_to_mesh_sphere_union_box_non_empty() {
        let s = unit_sphere();
        let b = unit_box();
        let mesh = csg_to_mesh(&s, &b, CsgOp::Union, 16, 2.0);
        assert!(
            !mesh.positions.is_empty(),
            "union of sphere and box should produce a non-empty mesh, got {} vertices",
            mesh.positions.len()
        );
    }
}
