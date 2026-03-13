// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Free-Form Deformation (FFD) lattice deformation.
//!
//! Implements the Sederberg-Parry FFD algorithm using a trilinear/Bezier lattice.
//! A mesh is deformed by displacing control points of a 3-D lattice, then
//! interpolating mesh vertices with Bernstein polynomial blending.

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ─── helpers ────────────────────────────────────────────────────────────────

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

// ─── Bernstein basis ─────────────────────────────────────────────────────────

/// Bernstein basis polynomial B(n, i, t) = C(n, i) * t^i * (1-t)^(n-i).
pub fn bernstein(n: usize, i: usize, t: f32) -> f32 {
    if i > n {
        return 0.0;
    }
    let coeff = binomial(n, i) as f32;
    let t_pow = t.powi(i as i32);
    let one_minus_t_pow = (1.0 - t).powi((n - i) as i32);
    coeff * t_pow * one_minus_t_pow
}

/// Binomial coefficient C(n, k).
pub fn binomial(n: usize, k: usize) -> f64 {
    if k > n {
        return 0.0;
    }
    let k = k.min(n - k); // symmetry: C(n,k) = C(n,n-k)
    if k == 0 {
        return 1.0;
    }
    let mut result = 1.0_f64;
    for i in 0..k {
        result = result * (n - i) as f64 / (i + 1) as f64;
    }
    result
}

// ─── FfdLattice ─────────────────────────────────────────────────────────────

/// A 3-D Bezier/trilinear FFD lattice.
///
/// The lattice has `dims[0] * dims[1] * dims[2]` control points laid out
/// uniformly inside `[origin, origin + extent]`.
///
/// Grid index to flat index: `i + dims[0] * (j + dims[1] * k)`.
#[derive(Debug, Clone)]
pub struct FfdLattice {
    /// Control point grid dimensions: (l+1, m+1, n+1) along (x, y, z).
    pub dims: [usize; 3],
    /// Control points in object space.
    pub control_points: Vec<[f32; 3]>,
    /// AABB origin – minimum corner of the lattice bounding box.
    pub origin: [f32; 3],
    /// Lattice extent along each axis.
    pub extent: [f32; 3],
}

impl FfdLattice {
    // ── construction ────────────────────────────────────────────────────────

    /// Create a lattice with explicit bounds.
    ///
    /// Control points are placed uniformly in `[origin, origin + extent]`.
    pub fn new(dims: [usize; 3], origin: [f32; 3], extent: [f32; 3]) -> Self {
        let total = dims[0] * dims[1] * dims[2];
        let mut control_points = Vec::with_capacity(total);

        let [lp1, mp1, np1] = dims;
        let l = (lp1.saturating_sub(1)).max(1) as f32;
        let m = (mp1.saturating_sub(1)).max(1) as f32;
        let n = (np1.saturating_sub(1)).max(1) as f32;

        for k in 0..np1 {
            for j in 0..mp1 {
                for i in 0..lp1 {
                    let x = origin[0] + extent[0] * i as f32 / l;
                    let y = origin[1] + extent[1] * j as f32 / m;
                    let z = origin[2] + extent[2] * k as f32 / n;
                    control_points.push([x, y, z]);
                }
            }
        }

        FfdLattice {
            dims,
            control_points,
            origin,
            extent,
        }
    }

    /// Create an undeformed lattice around a mesh's bounding box (with padding).
    pub fn from_mesh(mesh: &MeshBuffers, dims: [usize; 3], padding: f32) -> Self {
        let (mut min, mut max) = mesh_aabb(&mesh.positions);
        for i in 0..3 {
            min[i] -= padding;
            max[i] += padding;
        }
        let extent = [
            (max[0] - min[0]).max(1e-6),
            (max[1] - min[1]).max(1e-6),
            (max[2] - min[2]).max(1e-6),
        ];
        Self::new(dims, min, extent)
    }

    // ── accessors ────────────────────────────────────────────────────────────

    fn flat_idx(&self, i: usize, j: usize, k: usize) -> usize {
        i + self.dims[0] * (j + self.dims[1] * k)
    }

    /// Get control point at grid index (i, j, k).
    pub fn get(&self, i: usize, j: usize, k: usize) -> [f32; 3] {
        self.control_points[self.flat_idx(i, j, k)]
    }

    /// Set control point at grid index (i, j, k).
    pub fn set(&mut self, i: usize, j: usize, k: usize, pos: [f32; 3]) {
        let idx = self.flat_idx(i, j, k);
        self.control_points[idx] = pos;
    }

    /// Move a control point by a delta.
    pub fn displace(&mut self, i: usize, j: usize, k: usize, delta: [f32; 3]) {
        let idx = self.flat_idx(i, j, k);
        let cp = &mut self.control_points[idx];
        cp[0] += delta[0];
        cp[1] += delta[1];
        cp[2] += delta[2];
    }

    /// Total number of control points.
    pub fn control_point_count(&self) -> usize {
        self.dims[0] * self.dims[1] * self.dims[2]
    }

    // ── evaluation ───────────────────────────────────────────────────────────

    /// Convert world position to lattice-local coordinates (s, t, u) in [0, 1]^3.
    pub fn to_local(&self, pos: [f32; 3]) -> [f32; 3] {
        [
            clamp01((pos[0] - self.origin[0]) / self.extent[0]),
            clamp01((pos[1] - self.origin[1]) / self.extent[1]),
            clamp01((pos[2] - self.origin[2]) / self.extent[2]),
        ]
    }

    /// Evaluate the deformed position using Bernstein polynomial blending.
    ///
    /// For a lattice of dims [l+1, m+1, n+1]:
    /// ```text
    /// result = Σ_{i=0}^{l} Σ_{j=0}^{m} Σ_{k=0}^{n}
    ///          B(l,i,s) * B(m,j,t) * B(n,k,u) * cp[i,j,k]
    /// ```
    pub fn evaluate(&self, s: f32, t: f32, u: f32) -> [f32; 3] {
        let [lp1, mp1, np1] = self.dims;
        let l = lp1.saturating_sub(1);
        let m = mp1.saturating_sub(1);
        let n = np1.saturating_sub(1);

        let mut result = [0.0_f32; 3];

        for ki in 0..np1 {
            let bk = bernstein(n, ki, u);
            if bk == 0.0 {
                continue;
            }
            for ji in 0..mp1 {
                let bj = bernstein(m, ji, t);
                if bj == 0.0 {
                    continue;
                }
                for ii in 0..lp1 {
                    let bi = bernstein(l, ii, s);
                    let w = bi * bj * bk;
                    let cp = self.get(ii, ji, ki);
                    result[0] += w * cp[0];
                    result[1] += w * cp[1];
                    result[2] += w * cp[2];
                }
            }
        }

        result
    }

    /// Apply FFD to all vertices in a mesh and return the deformed mesh.
    pub fn apply(&self, mesh: &MeshBuffers) -> MeshBuffers {
        apply_ffd(mesh, self)
    }
}

// ─── free functions ──────────────────────────────────────────────────────────

/// Apply FFD to a mesh with the given lattice.
pub fn apply_ffd(mesh: &MeshBuffers, lattice: &FfdLattice) -> MeshBuffers {
    let new_positions: Vec<[f32; 3]> = mesh
        .positions
        .iter()
        .map(|&p| {
            let stu = lattice.to_local(p);
            lattice.evaluate(stu[0], stu[1], stu[2])
        })
        .collect();

    let n = new_positions.len();
    let mut deformed = MeshBuffers {
        positions: new_positions,
        normals: vec![[0.0, 1.0, 0.0]; n],
        tangents: mesh.tangents.clone(),
        uvs: mesh.uvs.clone(),
        indices: mesh.indices.clone(),
        colors: mesh.colors.clone(),
        has_suit: mesh.has_suit,
    };
    compute_normals(&mut deformed);
    deformed
}

/// Create a twist deformation lattice.
///
/// Builds a lattice with 4 slices along `axis` and 2 along the others.
/// Each slice at index `row` along `axis` is rotated by
/// `twist_angle * (row / max_row)` around that axis.
pub fn make_twist_lattice(mesh: &MeshBuffers, twist_angle: f32, axis: usize) -> FfdLattice {
    let axis = axis % 3;
    // 4 slices along the twist axis, 2 along the others.
    let mut dims = [2usize, 2, 2];
    dims[axis] = 4;

    let mut lattice = FfdLattice::from_mesh(mesh, dims, 0.05);

    let [lp1, mp1, np1] = lattice.dims;
    let slice_count = match axis {
        0 => lp1,
        1 => mp1,
        _ => np1,
    };
    let max_row = (slice_count - 1).max(1) as f32;

    // Clone to avoid borrow issues while mutating
    let original_cps = lattice.control_points.clone();

    for k in 0..np1 {
        for j in 0..mp1 {
            for i in 0..lp1 {
                let row = match axis {
                    0 => i,
                    1 => j,
                    _ => k,
                };
                let angle = twist_angle * row as f32 / max_row;
                let (sa, ca) = (angle.sin(), angle.cos());

                let flat = i + lp1 * (j + mp1 * k);
                let cp = original_cps[flat];

                // Rotate in the plane perpendicular to `axis`
                let (a1, a2) = perpendicular_axes(axis);
                let v1 = cp[a1];
                let v2 = cp[a2];
                let mut new_cp = cp;
                new_cp[a1] = ca * v1 - sa * v2;
                new_cp[a2] = sa * v1 + ca * v2;

                lattice.set(i, j, k, new_cp);
            }
        }
    }

    lattice
}

/// Create a bend deformation lattice.
///
/// Builds a lattice with 4 slices along `axis` and 2 along the others.
/// Control points are arc-bent by rotating around the axis perpendicular to
/// both `axis` and one of the remaining axes.
pub fn make_bend_lattice(mesh: &MeshBuffers, bend_angle: f32, axis: usize) -> FfdLattice {
    let axis = axis % 3;
    // 4 slices along the bend axis, 2 along the others.
    let mut dims = [2usize, 2, 2];
    dims[axis] = 4;

    let mut lattice = FfdLattice::from_mesh(mesh, dims, 0.05);

    let [lp1, mp1, np1] = lattice.dims;
    let slice_count = match axis {
        0 => lp1,
        1 => mp1,
        _ => np1,
    };
    let max_row = (slice_count - 1).max(1) as f32;

    let origin = lattice.origin;
    let original_cps = lattice.control_points.clone();

    // Rotate in the plane spanned by `axis` and `a_perp`.
    let (_, a_perp) = perpendicular_axes(axis);

    for k in 0..np1 {
        for j in 0..mp1 {
            for i in 0..lp1 {
                let row = match axis {
                    0 => i,
                    1 => j,
                    _ => k,
                };
                let t = row as f32 / max_row;
                let angle = bend_angle * t;
                let (sa, ca) = (angle.sin(), angle.cos());

                let flat = i + lp1 * (j + mp1 * k);
                let cp = original_cps[flat];

                // Arc-bend: rotate (axis, a_perp) plane around the lattice origin.
                let u1 = cp[axis] - origin[axis];
                let u2 = cp[a_perp] - origin[a_perp];
                let mut new_cp = cp;
                new_cp[axis] = origin[axis] + ca * u1 - sa * u2;
                new_cp[a_perp] = origin[a_perp] + sa * u1 + ca * u2;

                lattice.set(i, j, k, new_cp);
            }
        }
    }

    lattice
}

// ─── internal helpers ────────────────────────────────────────────────────────

/// Return the two axes perpendicular to `axis` (in order).
fn perpendicular_axes(axis: usize) -> (usize, usize) {
    match axis % 3 {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    }
}

/// Compute the AABB of a set of positions.
///
/// Returns `([0,0,0], [1,1,1])` for an empty slice so callers always get a
/// valid (if unit) bounding box.
fn mesh_aabb(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    if positions.is_empty() {
        return ([0.0; 3], [1.0; 3]);
    }
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for p in positions {
        for i in 0..3 {
            if p[i] < min[i] {
                min[i] = p[i];
            }
            if p[i] > max[i] {
                max[i] = p[i];
            }
        }
    }
    (min, max)
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ─────────────────────────────────────────────────────────────

    fn simple_mesh() -> MeshBuffers {
        // A flat quad made of two triangles, vertices in [0,1]^2 at z=0.
        MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            colors: None,
            has_suit: false,
        }
    }

    fn approx_eq3(a: [f32; 3], b: [f32; 3], eps: f32) -> bool {
        (a[0] - b[0]).abs() < eps && (a[1] - b[1]).abs() < eps && (a[2] - b[2]).abs() < eps
    }

    // ── Bernstein ────────────────────────────────────────────────────────────

    #[test]
    fn test_bernstein_degree1() {
        // B(1, 0, t) = 1 - t
        assert!((bernstein(1, 0, 0.0) - 1.0).abs() < 1e-6);
        assert!((bernstein(1, 0, 1.0) - 0.0).abs() < 1e-6);
        assert!((bernstein(1, 0, 0.5) - 0.5).abs() < 1e-6);
        // B(1, 1, t) = t
        assert!((bernstein(1, 1, 0.0) - 0.0).abs() < 1e-6);
        assert!((bernstein(1, 1, 1.0) - 1.0).abs() < 1e-6);
        assert!((bernstein(1, 1, 0.5) - 0.5).abs() < 1e-6);
        // Partition of unity
        let t = 0.3;
        assert!((bernstein(1, 0, t) + bernstein(1, 1, t) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bernstein_degree2() {
        // B(2, 0, t) = (1-t)^2, B(2, 1, t) = 2t(1-t), B(2, 2, t) = t^2
        let t = 0.4;
        let b0 = (1.0 - t) * (1.0 - t);
        let b1 = 2.0 * t * (1.0 - t);
        let b2 = t * t;
        assert!((bernstein(2, 0, t) - b0).abs() < 1e-6);
        assert!((bernstein(2, 1, t) - b1).abs() < 1e-6);
        assert!((bernstein(2, 2, t) - b2).abs() < 1e-6);
        // Partition of unity
        assert!((b0 + b1 + b2 - 1.0).abs() < 1e-6);
    }

    // ── Binomial ─────────────────────────────────────────────────────────────

    #[test]
    fn test_binomial_basic() {
        assert!((binomial(0, 0) - 1.0).abs() < 1e-9);
        assert!((binomial(1, 0) - 1.0).abs() < 1e-9);
        assert!((binomial(1, 1) - 1.0).abs() < 1e-9);
        assert!((binomial(4, 2) - 6.0).abs() < 1e-9);
        assert!((binomial(5, 0) - 1.0).abs() < 1e-9);
        assert!((binomial(5, 5) - 1.0).abs() < 1e-9);
        assert!((binomial(5, 2) - 10.0).abs() < 1e-9);
        assert!((binomial(6, 3) - 20.0).abs() < 1e-9);
        // Out-of-range
        assert!((binomial(3, 4) - 0.0).abs() < 1e-9);
    }

    // ── FfdLattice::new ───────────────────────────────────────────────────────

    #[test]
    fn test_lattice_new() {
        let lat = FfdLattice::new([2, 2, 2], [0.0; 3], [1.0; 3]);
        assert_eq!(lat.dims, [2, 2, 2]);
        assert_eq!(lat.control_points.len(), 8);
        // Corner at (0,0,0) → origin
        assert!(approx_eq3(lat.get(0, 0, 0), [0.0, 0.0, 0.0], 1e-6));
        // Corner at (1,1,1) → origin + extent
        assert!(approx_eq3(lat.get(1, 1, 1), [1.0, 1.0, 1.0], 1e-6));
        // Corner at (1,0,0) → [1,0,0]
        assert!(approx_eq3(lat.get(1, 0, 0), [1.0, 0.0, 0.0], 1e-6));
    }

    // ── FfdLattice::from_mesh ─────────────────────────────────────────────────

    #[test]
    fn test_lattice_from_mesh() {
        let mesh = simple_mesh();
        let lat = FfdLattice::from_mesh(&mesh, [2, 2, 2], 0.1);
        // Origin should be below mesh min (with padding)
        assert!(lat.origin[0] < 0.0);
        assert!(lat.origin[1] < 0.0);
        // Total control points
        assert_eq!(lat.control_point_count(), 8);
    }

    // ── get / set ─────────────────────────────────────────────────────────────

    #[test]
    fn test_lattice_get_set() {
        let mut lat = FfdLattice::new([3, 3, 3], [0.0; 3], [1.0; 3]);
        let new_pos = [0.5, 0.7, 0.3];
        lat.set(1, 1, 1, new_pos);
        assert!(approx_eq3(lat.get(1, 1, 1), new_pos, 1e-6));

        // displace
        lat.displace(0, 0, 0, [0.1, 0.2, 0.3]);
        let after = lat.get(0, 0, 0);
        assert!((after[0] - 0.1).abs() < 1e-6);
        assert!((after[1] - 0.2).abs() < 1e-6);
        assert!((after[2] - 0.3).abs() < 1e-6);
    }

    // ── to_local ──────────────────────────────────────────────────────────────

    #[test]
    fn test_lattice_to_local() {
        let lat = FfdLattice::new([2, 2, 2], [0.0; 3], [2.0; 3]);
        // Center of the lattice
        let stu = lat.to_local([1.0, 1.0, 1.0]);
        assert!((stu[0] - 0.5).abs() < 1e-6);
        assert!((stu[1] - 0.5).abs() < 1e-6);
        assert!((stu[2] - 0.5).abs() < 1e-6);

        // Clamp below origin
        let stu2 = lat.to_local([-1.0, -1.0, -1.0]);
        assert!((stu2[0] - 0.0).abs() < 1e-6);

        // Clamp above extent
        let stu3 = lat.to_local([10.0, 10.0, 10.0]);
        assert!((stu3[0] - 1.0).abs() < 1e-6);
    }

    // ── evaluate identity ─────────────────────────────────────────────────────

    #[test]
    fn test_lattice_evaluate_identity() {
        // An undeformed 2×2×2 lattice spanning [0,1]^3 should map (s,t,u) → (s,t,u).
        let lat = FfdLattice::new([2, 2, 2], [0.0; 3], [1.0; 3]);

        let test_pts = [
            [0.0_f32, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.5, 0.5, 0.5],
            [0.25, 0.75, 0.1],
        ];

        for &[s, t, u] in &test_pts {
            let out = lat.evaluate(s, t, u);
            assert!(
                approx_eq3(out, [s, t, u], 1e-5),
                "evaluate({s},{t},{u}) = {out:?}, expected [{s},{t},{u}]"
            );
        }
    }

    // ── apply identity ────────────────────────────────────────────────────────

    #[test]
    fn test_ffd_apply_identity() {
        // A lattice exactly encompassing the mesh with no deformation should
        // return (approximately) the same vertex positions.
        let mesh = simple_mesh();
        let lat = FfdLattice::new([2, 2, 2], [0.0; 3], [1.0; 3]);
        let deformed = lat.apply(&mesh);

        assert_eq!(deformed.positions.len(), mesh.positions.len());
        for (orig, def) in mesh.positions.iter().zip(deformed.positions.iter()) {
            assert!(approx_eq3(*orig, *def, 1e-4), "orig={orig:?} def={def:?}");
        }
    }

    // ── apply translation ─────────────────────────────────────────────────────

    #[test]
    fn test_ffd_apply_translation() {
        // Shift all control points by (+1, 0, 0) → every vertex should move +1 in X.
        let mesh = simple_mesh();

        // Build an identity lattice spanning [0,1]^3
        let mut lat = FfdLattice::new([2, 2, 2], [0.0; 3], [1.0; 3]);

        // Displace all control points +1 in X
        for k in 0..2 {
            for j in 0..2 {
                for i in 0..2 {
                    lat.displace(i, j, k, [1.0, 0.0, 0.0]);
                }
            }
        }

        let deformed = apply_ffd(&mesh, &lat);
        for (orig, def) in mesh.positions.iter().zip(deformed.positions.iter()) {
            assert!((def[0] - (orig[0] + 1.0)).abs() < 1e-4, "X shift failed");
            assert!((def[1] - orig[1]).abs() < 1e-4, "Y changed unexpectedly");
            assert!((def[2] - orig[2]).abs() < 1e-4, "Z changed unexpectedly");
        }
    }

    // ── make_twist_lattice ────────────────────────────────────────────────────

    #[test]
    fn test_make_twist_lattice() {
        let mesh = simple_mesh();
        let lat = make_twist_lattice(&mesh, 0.5, 1); // twist around Y axis
                                                     // Should have 4 slices along Y (axis=1 → mp1=4)
        assert_eq!(lat.dims[1], 4);
        assert_eq!(
            lat.control_point_count(),
            lat.dims[0] * lat.dims[1] * lat.dims[2]
        );
        // The deformed mesh has same vertex count
        let deformed = lat.apply(&mesh);
        assert_eq!(deformed.positions.len(), mesh.positions.len());
    }

    // ── make_bend_lattice ─────────────────────────────────────────────────────

    #[test]
    fn test_make_bend_lattice() {
        let mesh = simple_mesh();
        let lat = make_bend_lattice(&mesh, 0.3, 0); // bend along X axis
                                                    // Should have 4 slices along X (axis=0 → lp1=4)
        assert_eq!(lat.dims[0], 4);
        assert_eq!(
            lat.control_point_count(),
            lat.dims[0] * lat.dims[1] * lat.dims[2]
        );
        let deformed = lat.apply(&mesh);
        assert_eq!(deformed.positions.len(), mesh.positions.len());
    }

    // ── control_point_count ───────────────────────────────────────────────────

    #[test]
    fn test_control_point_count() {
        assert_eq!(
            FfdLattice::new([2, 3, 4], [0.0; 3], [1.0; 3]).control_point_count(),
            24
        );
        assert_eq!(
            FfdLattice::new([5, 5, 5], [0.0; 3], [1.0; 3]).control_point_count(),
            125
        );
        assert_eq!(
            FfdLattice::new([1, 1, 1], [0.0; 3], [1.0; 3]).control_point_count(),
            1
        );
    }
}
