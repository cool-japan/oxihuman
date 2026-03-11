// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Mesh warping via Radial Basis Functions (RBF).
//!
//! Users place control points (handles) and drag them; the mesh deforms
//! smoothly using an RBF-based interpolation scheme.  The system solves a
//! small N×N linear system (Gaussian elimination) for the RBF coefficients,
//! then evaluates the warp at every mesh vertex.

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ─── helpers ────────────────────────────────────────────────────────────────

/// Euclidean distance between two 3-D points.
#[inline]
pub fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[inline]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

// ─── RbfKernel ──────────────────────────────────────────────────────────────

/// Kernel function used for RBF interpolation.
#[derive(Debug, Clone, PartialEq)]
pub enum RbfKernel {
    /// Thin-plate spline: r² · ln(r).
    ThinPlateSpline,
    /// Gaussian: exp(−c · r²), `c` = parameter (must be > 0).
    Gaussian(f32),
    /// Multiquadric: sqrt(r² + c²).
    Multiquadric(f32),
    /// Inverse-distance: 1 / (r² + c²).
    InverseDistance(f32),
    /// Linear: r.
    Linear,
}

impl RbfKernel {
    /// Evaluate the kernel for radius `r` (≥ 0).
    pub fn evaluate(&self, r: f32) -> f32 {
        match self {
            RbfKernel::ThinPlateSpline => {
                if r < 1e-12 {
                    0.0
                } else {
                    r * r * r.ln()
                }
            }
            RbfKernel::Gaussian(c) => (-(c * r * r)).exp(),
            RbfKernel::Multiquadric(c) => (r * r + c * c).sqrt(),
            RbfKernel::InverseDistance(c) => 1.0 / (r * r + c * c),
            RbfKernel::Linear => r,
        }
    }
}

// ─── WarpHandle ─────────────────────────────────────────────────────────────

/// A single control-point handle: original position + where it was dragged.
pub struct WarpHandle {
    /// Original control point location.
    pub source: [f32; 3],
    /// Target position (where the handle was dragged to).
    pub target: [f32; 3],
    /// Influence weight (default `1.0`).
    pub weight: f32,
}

impl WarpHandle {
    /// Convenience constructor with default weight 1.0.
    pub fn new(source: [f32; 3], target: [f32; 3]) -> Self {
        Self {
            source,
            target,
            weight: 1.0,
        }
    }

    /// Displacement vector (target − source).
    #[inline]
    pub fn delta(&self) -> [f32; 3] {
        vec3_sub(self.target, self.source)
    }
}

// ─── RbfWarpConfig ──────────────────────────────────────────────────────────

/// Configuration for the RBF warp solver.
pub struct RbfWarpConfig {
    /// Kernel type used for interpolation.
    pub kernel: RbfKernel,
    /// Spatial influence radius.  `0.0` means global (no falloff cutoff).
    pub falloff_radius: f32,
    /// Tikhonov regularization λ (default `1e-6`).
    pub regularization: f32,
}

impl Default for RbfWarpConfig {
    fn default() -> Self {
        Self {
            kernel: RbfKernel::ThinPlateSpline,
            falloff_radius: 0.0,
            regularization: 1e-6,
        }
    }
}

// ─── Gaussian elimination ────────────────────────────────────────────────────

/// Solve A·x = b in-place via Gaussian elimination with partial pivoting.
/// Returns `None` if the matrix is (near-)singular.
#[allow(clippy::needless_range_loop)]
fn gaussian_elimination(a: &mut [Vec<f32>], b: &mut [f32]) -> Option<Vec<f32>> {
    let n = b.len();
    for col in 0..n {
        // find pivot row
        let mut max_row = col;
        let mut max_val = a[col][col].abs();
        for row in (col + 1)..n {
            if a[row][col].abs() > max_val {
                max_val = a[row][col].abs();
                max_row = row;
            }
        }
        if max_val < 1e-12 {
            return None; // singular
        }
        a.swap(col, max_row);
        b.swap(col, max_row);

        let pivot = a[col][col];
        for row in (col + 1)..n {
            let factor = a[row][col] / pivot;
            b[row] -= factor * b[col];
            for k in col..n {
                a[row][k] -= factor * a[col][k];
            }
        }
    }
    // back substitution
    let mut x = vec![0.0f32; n];
    for i in (0..n).rev() {
        let mut sum = b[i];
        for j in (i + 1)..n {
            sum -= a[i][j] * x[j];
        }
        x[i] = sum / a[i][i];
    }
    Some(x)
}

// ─── RbfWarp ────────────────────────────────────────────────────────────────

/// Precomputed RBF warp (kernel matrix solved).
pub struct RbfWarp {
    /// Control-point handles used to build this warp.
    pub handles: Vec<WarpHandle>,
    /// Configuration used to build this warp.
    pub config: RbfWarpConfig,
    /// Solved RBF coefficients for the X axis.
    weights_x: Vec<f32>,
    /// Solved RBF coefficients for the Y axis.
    weights_y: Vec<f32>,
    /// Solved RBF coefficients for the Z axis.
    weights_z: Vec<f32>,
}

impl RbfWarp {
    /// Build and solve the RBF system for the given handles and config.
    ///
    /// Sets up the N×N kernel matrix K where `K[i][j] = kernel(|src_i − src_j|)`,
    /// adds `lambda · I` regularisation, then solves `K · w = delta` for each
    /// axis via Gaussian elimination.  Falls back to all-zero coefficients if
    /// the system is singular (e.g. duplicate handles).
    pub fn build(handles: Vec<WarpHandle>, config: RbfWarpConfig) -> Self {
        let n = handles.len();
        if n == 0 {
            return Self {
                handles,
                config,
                weights_x: Vec::new(),
                weights_y: Vec::new(),
                weights_z: Vec::new(),
            };
        }

        // Build kernel matrix K
        let mut k: Vec<Vec<f32>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        let r = dist3(handles[i].source, handles[j].source);
                        config.kernel.evaluate(r)
                    })
                    .collect()
            })
            .collect();

        // Add regularisation: lambda / weight on diagonal
        for i in 0..n {
            k[i][i] += config.regularization / handles[i].weight.max(1e-6);
        }

        // Right-hand sides (target − source) per axis
        let deltas_x: Vec<f32> = handles.iter().map(|h| h.delta()[0]).collect();
        let deltas_y: Vec<f32> = handles.iter().map(|h| h.delta()[1]).collect();
        let deltas_z: Vec<f32> = handles.iter().map(|h| h.delta()[2]).collect();

        let solve = |k: &Vec<Vec<f32>>, b: Vec<f32>| -> Vec<f32> {
            let mut a_copy: Vec<Vec<f32>> = k.to_vec();
            let mut b_copy = b;
            gaussian_elimination(&mut a_copy, &mut b_copy).unwrap_or_else(|| vec![0.0; n])
        };

        let weights_x = solve(&k, deltas_x);
        let weights_y = solve(&k, deltas_y);
        let weights_z = solve(&k, deltas_z);

        Self {
            handles,
            config,
            weights_x,
            weights_y,
            weights_z,
        }
    }

    /// Evaluate the warp displacement at a single point, returning the
    /// **warped** position (original + accumulated RBF displacement).
    pub fn eval(&self, point: [f32; 3]) -> [f32; 3] {
        let n = self.handles.len();
        if n == 0 {
            return point;
        }
        let (mut dx, mut dy, mut dz) = (0.0f32, 0.0f32, 0.0f32);
        for i in 0..n {
            let r = dist3(point, self.handles[i].source);
            // Optional falloff cutoff
            if self.config.falloff_radius > 0.0 && r > self.config.falloff_radius {
                continue;
            }
            let phi = self.config.kernel.evaluate(r);
            dx += self.weights_x[i] * phi;
            dy += self.weights_y[i] * phi;
            dz += self.weights_z[i] * phi;
        }
        [point[0] + dx, point[1] + dy, point[2] + dz]
    }

    /// Apply this warp to a full mesh, returning a new deformed mesh.
    pub fn apply(&self, mesh: &MeshBuffers) -> MeshBuffers {
        let mut out = mesh.clone();
        for p in out.positions.iter_mut() {
            *p = self.eval(*p);
        }
        compute_normals(&mut out);
        out
    }

    /// Number of control-point handles.
    #[inline]
    pub fn handle_count(&self) -> usize {
        self.handles.len()
    }
}

// ─── warp_mesh ───────────────────────────────────────────────────────────────

/// Convenience: build an [`RbfWarp`] from handles and config, apply it to
/// `mesh`, and return the deformed copy.
pub fn warp_mesh(
    mesh: &MeshBuffers,
    handles: Vec<WarpHandle>,
    config: &RbfWarpConfig,
) -> MeshBuffers {
    let cfg = RbfWarpConfig {
        kernel: config.kernel.clone(),
        falloff_radius: config.falloff_radius,
        regularization: config.regularization,
    };
    let warp = RbfWarp::build(handles, cfg);
    warp.apply(mesh)
}

// ─── simple_warp ─────────────────────────────────────────────────────────────

/// Simple distance-weighted warp: lerp each vertex toward the weighted-average
/// handle delta using a smooth falloff within `falloff_radius`.
///
/// When `falloff_radius` is 0.0 all handles contribute globally via
/// inverse-square weighting.
pub fn simple_warp(mesh: &MeshBuffers, handles: &[WarpHandle], falloff_radius: f32) -> MeshBuffers {
    if handles.is_empty() {
        return mesh.clone();
    }
    let mut out = mesh.clone();
    for p in out.positions.iter_mut() {
        let mut total_w = 0.0f32;
        let mut disp = [0.0f32; 3];
        for h in handles {
            let d = dist3(*p, h.source);
            let w = if falloff_radius > 0.0 {
                if d > falloff_radius {
                    continue;
                }
                let t = (d / falloff_radius).clamp(0.0, 1.0);
                (1.0 - t * t).max(0.0) * h.weight
            } else {
                h.weight / (d * d + 1e-6)
            };
            let delta = h.delta();
            disp[0] += w * delta[0];
            disp[1] += w * delta[1];
            disp[2] += w * delta[2];
            total_w += w;
        }
        if total_w > 1e-12 {
            let inv = 1.0 / total_w;
            p[0] += disp[0] * inv;
            p[1] += disp[1] * inv;
            p[2] += disp[2] * inv;
        }
    }
    compute_normals(&mut out);
    out
}

// ─── displacement_field ──────────────────────────────────────────────────────

/// Sample the warp displacement on a regular 3-D grid.
///
/// Returns a `Vec` of `(sample_point, displacement)` pairs where
/// `displacement = warped_point − sample_point`.
///
/// `steps[i]` must be ≥ 1; a value of 1 samples only `min[i]`.
#[allow(clippy::too_many_arguments)]
pub fn displacement_field(
    warp: &RbfWarp,
    min: [f32; 3],
    max: [f32; 3],
    steps: [usize; 3],
) -> Vec<([f32; 3], [f32; 3])> {
    let sx = steps[0].max(1);
    let sy = steps[1].max(1);
    let sz = steps[2].max(1);
    let mut out = Vec::with_capacity(sx * sy * sz);
    for iz in 0..sz {
        let tz = if sz > 1 {
            iz as f32 / (sz - 1) as f32
        } else {
            0.0
        };
        for iy in 0..sy {
            let ty = if sy > 1 {
                iy as f32 / (sy - 1) as f32
            } else {
                0.0
            };
            for ix in 0..sx {
                let tx = if sx > 1 {
                    ix as f32 / (sx - 1) as f32
                } else {
                    0.0
                };
                let p = [
                    min[0] + tx * (max[0] - min[0]),
                    min[1] + ty * (max[1] - min[1]),
                    min[2] + tz * (max[2] - min[2]),
                ];
                let warped = warp.eval(p);
                let disp = vec3_sub(warped, p);
                out.push((p, disp));
            }
        }
    }
    out
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;
    use std::io::Write;

    // ── helpers ──────────────────────────────────────────────────────────────

    fn flat_quad_mesh() -> MeshBuffers {
        // 4 vertices forming a 2×2 quad (two triangles)
        MeshBuffers::from_morph(MB {
            positions: vec![
                [-1.0, 0.0, -1.0],
                [1.0, 0.0, -1.0],
                [1.0, 0.0, 1.0],
                [-1.0, 0.0, 1.0],
            ],
            normals: vec![[0.0, 1.0, 0.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        })
    }

    fn write_tmp(name: &str, content: &str) {
        let path = format!("/tmp/{name}");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    // ── RbfKernel ────────────────────────────────────────────────────────────

    #[test]
    fn kernel_tps_zero_at_zero() {
        assert_eq!(RbfKernel::ThinPlateSpline.evaluate(0.0), 0.0);
    }

    #[test]
    fn kernel_tps_positive_nonzero() {
        let v = RbfKernel::ThinPlateSpline.evaluate(2.0);
        // r^2 * ln(r) at r=2: 4 * ln(2) > 0
        assert!(v > 0.0, "TPS at r=2 should be positive, got {v}");
    }

    #[test]
    fn kernel_gaussian_unit_at_zero() {
        let k = RbfKernel::Gaussian(1.0);
        assert!((k.evaluate(0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn kernel_gaussian_decays() {
        let k = RbfKernel::Gaussian(1.0);
        assert!(k.evaluate(2.0) < k.evaluate(1.0));
    }

    #[test]
    fn kernel_multiquadric_positive() {
        let k = RbfKernel::Multiquadric(1.0);
        assert!(k.evaluate(0.0) > 0.0);
        assert!(k.evaluate(1.0) > k.evaluate(0.0));
    }

    #[test]
    fn kernel_inverse_distance_decreasing() {
        let k = RbfKernel::InverseDistance(1.0);
        assert!(k.evaluate(0.0) > k.evaluate(1.0));
    }

    #[test]
    fn kernel_linear_identity() {
        let k = RbfKernel::Linear;
        assert!((k.evaluate(3.5) - 3.5).abs() < 1e-6);
    }

    // ── dist3 ────────────────────────────────────────────────────────────────

    #[test]
    fn dist3_known_values() {
        assert!((dist3([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]) - 5.0).abs() < 1e-5);
        assert!(dist3([1.0, 1.0, 1.0], [1.0, 1.0, 1.0]) < 1e-9);
    }

    // ── RbfWarp – no handles ──────────────────────────────────────────────────

    #[test]
    fn warp_no_handles_identity() {
        let config = RbfWarpConfig::default();
        let warp = RbfWarp::build(vec![], config);
        let p = [1.0f32, 2.0, 3.0];
        let q = warp.eval(p);
        assert!((q[0] - p[0]).abs() < 1e-6);
        assert!((q[1] - p[1]).abs() < 1e-6);
        assert!((q[2] - p[2]).abs() < 1e-6);
    }

    // ── RbfWarp – single handle exactly at source ─────────────────────────────

    #[test]
    fn warp_single_handle_interpolates_source() {
        let handle = WarpHandle::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let config = RbfWarpConfig {
            kernel: RbfKernel::Gaussian(1.0),
            falloff_radius: 0.0,
            regularization: 1e-9,
        };
        let warp = RbfWarp::build(vec![handle], config);
        // The source point itself should be displaced ~1.0 in Y
        let warped = warp.eval([0.0, 0.0, 0.0]);
        assert!(
            (warped[1] - 1.0).abs() < 0.01,
            "Y displacement at source should be ~1, got {}",
            warped[1]
        );
    }

    // ── RbfWarp – apply to mesh ───────────────────────────────────────────────

    #[test]
    fn warp_apply_changes_positions() {
        let mesh = flat_quad_mesh();
        let handles = vec![WarpHandle::new([0.0, 0.0, 0.0], [0.0, 0.5, 0.0])];
        let config = RbfWarpConfig {
            kernel: RbfKernel::Gaussian(0.5),
            falloff_radius: 0.0,
            regularization: 1e-6,
        };
        let warp = RbfWarp::build(handles, config);
        let warped = warp.apply(&mesh);
        assert_eq!(warped.positions.len(), mesh.positions.len());
        // At least one vertex should have moved in Y
        let moved = warped
            .positions
            .iter()
            .zip(mesh.positions.iter())
            .any(|(a, b)| (a[1] - b[1]).abs() > 1e-4);
        assert!(moved, "expected at least one vertex to move in Y");
    }

    // ── warp_mesh convenience ────────────────────────────────────────────────

    #[test]
    fn warp_mesh_convenience() {
        let mesh = flat_quad_mesh();
        let handles = vec![WarpHandle::new([-1.0, 0.0, -1.0], [-1.0, 1.0, -1.0])];
        let config = RbfWarpConfig::default();
        let out = warp_mesh(&mesh, handles, &config);
        assert_eq!(out.positions.len(), mesh.positions.len());
    }

    // ── simple_warp ──────────────────────────────────────────────────────────

    #[test]
    fn simple_warp_no_handles_identity() {
        let mesh = flat_quad_mesh();
        let out = simple_warp(&mesh, &[], 1.0);
        for (a, b) in out.positions.iter().zip(mesh.positions.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-6);
        }
    }

    #[test]
    fn simple_warp_moves_near_vertices() {
        let mesh = flat_quad_mesh();
        let handles = [WarpHandle::new([-1.0, 0.0, -1.0], [-1.0, 1.0, -1.0])];
        let out = simple_warp(&mesh, &handles, 2.0);
        // vertex 0 is at [-1,0,-1] which is the source; it should move in Y
        assert!(
            out.positions[0][1] > 0.0,
            "vertex 0 Y should be > 0, got {}",
            out.positions[0][1]
        );
    }

    // ── displacement_field ───────────────────────────────────────────────────

    #[test]
    fn displacement_field_count() {
        let handle = WarpHandle::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let config = RbfWarpConfig::default();
        let warp = RbfWarp::build(vec![handle], config);
        let field = displacement_field(&warp, [-1.0; 3], [1.0; 3], [3, 3, 3]);
        assert_eq!(field.len(), 27, "3x3x3 = 27 samples");
    }

    #[test]
    fn displacement_field_single_step() {
        let config = RbfWarpConfig::default();
        let warp = RbfWarp::build(vec![], config);
        let field = displacement_field(&warp, [0.0; 3], [1.0; 3], [1, 1, 1]);
        assert_eq!(field.len(), 1);
        // No handles -> zero displacement
        let (_, disp) = field[0];
        assert!(disp[0].abs() < 1e-6 && disp[1].abs() < 1e-6 && disp[2].abs() < 1e-6);
    }

    // ── handle_count ─────────────────────────────────────────────────────────

    #[test]
    fn handle_count_correct() {
        let handles = vec![
            WarpHandle::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
            WarpHandle::new([1.0, 0.0, 0.0], [2.0, 0.0, 0.0]),
        ];
        let warp = RbfWarp::build(handles, RbfWarpConfig::default());
        assert_eq!(warp.handle_count(), 2);
    }

    // ── multi-kernel smoke tests ─────────────────────────────────────────────

    #[test]
    fn multiquadric_kernel_warp() {
        let mesh = flat_quad_mesh();
        let handles = vec![WarpHandle::new([0.0, 0.0, 0.0], [0.0, 0.2, 0.0])];
        let config = RbfWarpConfig {
            kernel: RbfKernel::Multiquadric(1.0),
            falloff_radius: 0.0,
            regularization: 1e-5,
        };
        let out = warp_mesh(&mesh, handles, &config);
        assert_eq!(out.positions.len(), 4);
    }

    #[test]
    fn inverse_distance_kernel_warp() {
        let mesh = flat_quad_mesh();
        let handles = vec![WarpHandle::new([0.0, 0.0, 0.0], [0.0, 0.2, 0.0])];
        let config = RbfWarpConfig {
            kernel: RbfKernel::InverseDistance(1.0),
            falloff_radius: 0.0,
            regularization: 1e-5,
        };
        let out = warp_mesh(&mesh, handles, &config);
        assert_eq!(out.positions.len(), 4);
    }

    // ── falloff_radius clipping ───────────────────────────────────────────────

    #[test]
    fn falloff_radius_clips_distant_vertices() {
        // Handle at corner [-1,0,-1], very small radius
        let handles = vec![WarpHandle::new([-1.0, 0.0, -1.0], [-1.0, 2.0, -1.0])];
        let config = RbfWarpConfig {
            kernel: RbfKernel::Gaussian(1.0),
            falloff_radius: 0.1,
            regularization: 1e-6,
        };
        let warp = RbfWarp::build(handles, config);
        // Vertex far from handle should be nearly unchanged
        let far = warp.eval([1.0, 0.0, 1.0]);
        assert!(
            far[1].abs() < 0.5,
            "distant vertex Y should be small, got {}",
            far[1]
        );
    }

    // ── write output to /tmp ──────────────────────────────────────────────────

    #[test]
    fn write_displacement_field_to_tmp() {
        let handle = WarpHandle::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let config = RbfWarpConfig {
            kernel: RbfKernel::Gaussian(0.5),
            falloff_radius: 0.0,
            regularization: 1e-6,
        };
        let warp = RbfWarp::build(vec![handle], config);
        let field = displacement_field(&warp, [-2.0; 3], [2.0; 3], [4, 4, 4]);
        let mut buf = String::new();
        for (p, d) in &field {
            buf.push_str(&format!(
                "{:.3} {:.3} {:.3}  ->  {:.5} {:.5} {:.5}\n",
                p[0], p[1], p[2], d[0], d[1], d[2]
            ));
        }
        write_tmp("mesh_warp_displacement_field.txt", &buf);
        assert!(std::path::Path::new("/tmp/mesh_warp_displacement_field.txt").exists());
    }

    #[test]
    fn write_warped_positions_to_tmp() {
        let mesh = flat_quad_mesh();
        let handles = vec![
            WarpHandle::new([-1.0, 0.0, -1.0], [-1.0, 1.0, -1.0]),
            WarpHandle::new([1.0, 0.0, 1.0], [1.0, -0.5, 1.0]),
        ];
        let config = RbfWarpConfig {
            kernel: RbfKernel::ThinPlateSpline,
            falloff_radius: 0.0,
            regularization: 1e-4,
        };
        let out = warp_mesh(&mesh, handles, &config);
        let mut buf = String::new();
        for (i, p) in out.positions.iter().enumerate() {
            buf.push_str(&format!("v{i}: {:.4} {:.4} {:.4}\n", p[0], p[1], p[2]));
        }
        write_tmp("mesh_warp_positions.txt", &buf);
        assert!(std::path::Path::new("/tmp/mesh_warp_positions.txt").exists());
    }
}
