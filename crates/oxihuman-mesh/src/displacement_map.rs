// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! UV-based 2D displacement map application to mesh vertices.
//!
//! Applies a grayscale displacement map (stored as f32 grid) to mesh vertices
//! along their normals, using UV coordinates for sampling.

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ── DisplacementMap2D ────────────────────────────────────────────────────────

/// A 2D grayscale displacement map (values in [0, 1]).
pub struct DisplacementMap2D {
    /// Row-major storage: data[y * width + x]
    pub data: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl DisplacementMap2D {
    /// Create a new displacement map filled with zeros.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![0.0f32; width * height],
            width,
            height,
        }
    }

    /// Create a displacement map from a closure `f(u, v) -> value` where u, v in [0, 1].
    pub fn from_fn(width: usize, height: usize, f: impl Fn(f32, f32) -> f32) -> Self {
        let mut data = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                let u = if width > 1 {
                    x as f32 / (width - 1) as f32
                } else {
                    0.5
                };
                let v = if height > 1 {
                    y as f32 / (height - 1) as f32
                } else {
                    0.5
                };
                data.push(f(u, v));
            }
        }
        Self {
            data,
            width,
            height,
        }
    }

    /// Get pixel value at (x, y).
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.data[y * self.width + x]
    }

    /// Set pixel value at (x, y).
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        self.data[y * self.width + x] = val;
    }

    /// Bilinear sample at UV coordinates in [0, 1]. Clamps to boundary.
    pub fn sample(&self, u: f32, v: f32) -> f32 {
        if self.width == 0 || self.height == 0 {
            return 0.0;
        }
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let px = u * (self.width - 1) as f32;
        let py = v * (self.height - 1) as f32;

        let x0 = (px.floor() as usize).min(self.width - 1);
        let y0 = (py.floor() as usize).min(self.height - 1);
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let fx = px - px.floor();
        let fy = py - py.floor();

        let v00 = self.get(x0, y0);
        let v10 = self.get(x1, y0);
        let v01 = self.get(x0, y1);
        let v11 = self.get(x1, y1);

        let top = v00 + (v10 - v00) * fx;
        let bot = v01 + (v11 - v01) * fx;
        top + (bot - top) * fy
    }

    /// Tile sample: wraps UVs modulo 1.0 before sampling.
    pub fn sample_tiled(&self, u: f32, v: f32) -> f32 {
        let u = u - u.floor();
        let v = v - v.floor();
        self.sample(u, v)
    }

    /// Scale all values to [0, 1].
    pub fn normalize(&mut self) {
        let min_val = self.data.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_val = self.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let range = max_val - min_val;
        if range < 1e-12 {
            for v in &mut self.data {
                *v = 0.0;
            }
        } else {
            for v in &mut self.data {
                *v = (*v - min_val) / range;
            }
        }
    }

    /// Invert all values: each value becomes `1.0 - val`.
    pub fn invert(&mut self) {
        for v in &mut self.data {
            *v = 1.0 - *v;
        }
    }

    /// Generate from simple hash-based value noise.
    pub fn from_value_noise(width: usize, height: usize, scale: f32, seed: u32) -> Self {
        Self::from_fn(width, height, |u, v| {
            value_noise_2d(u * scale, v * scale, seed)
        })
    }

    /// Generate a checkerboard pattern.
    ///
    /// Cells with `(x * tiles / width + y * tiles / height) % 2 == 0` are 1.0, others 0.0.
    pub fn checkerboard(width: usize, height: usize, tiles: usize) -> Self {
        let mut map = Self::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let cx = if width > 0 { x * tiles / width } else { 0 };
                let cy = if height > 0 { y * tiles / height } else { 0 };
                map.set(x, y, if (cx + cy) % 2 == 0 { 1.0 } else { 0.0 });
            }
        }
        map
    }
}

// ── Value noise helper ───────────────────────────────────────────────────────

/// Simple hash-based value noise at a 2D position.
fn value_noise_2d(x: f32, y: f32, seed: u32) -> f32 {
    let hash = |ix: i32, iy: i32| -> f32 {
        let h = (ix.wrapping_mul(1619) ^ iy.wrapping_mul(31337) ^ seed as i32) as u32;
        let h = h
            .wrapping_mul(0x9e3779b9)
            .wrapping_add(h << 6)
            .wrapping_add(h >> 2);
        (h & 0xFFFF) as f32 / 65535.0
    };

    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - x.floor();
    let fy = y - y.floor();

    // Smoothstep
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);

    let v00 = hash(ix, iy);
    let v10 = hash(ix + 1, iy);
    let v01 = hash(ix, iy + 1);
    let v11 = hash(ix + 1, iy + 1);

    let top = v00 + (v10 - v00) * ux;
    let bot = v01 + (v11 - v01) * ux;
    top + (bot - top) * uy
}

// ── DisplacementApplyParams ──────────────────────────────────────────────────

/// Parameters controlling displacement map application.
pub struct DisplacementApplyParams {
    /// Peak displacement amplitude (positive or negative).
    pub amplitude: f32,
    /// Map value that means "no displacement" (default 0.5).
    pub midlevel: f32,
    /// Wrap UVs modulo 1.0 when sampling (default false).
    pub use_tiling: bool,
    /// Recompute normals after displacement (default true).
    pub smooth_normals: bool,
}

impl Default for DisplacementApplyParams {
    fn default() -> Self {
        Self {
            amplitude: 0.02,
            midlevel: 0.5,
            use_tiling: false,
            smooth_normals: true,
        }
    }
}

// ── DisplacedMeshResult ──────────────────────────────────────────────────────

/// Result of applying a displacement map to a mesh.
pub struct DisplacedMeshResult {
    /// The displaced mesh (with recomputed normals if requested).
    pub mesh: MeshBuffers,
    /// Minimum displacement scalar across all vertices.
    pub min_displacement: f32,
    /// Maximum displacement scalar across all vertices.
    pub max_displacement: f32,
    /// Mean displacement scalar across all vertices.
    pub mean_displacement: f32,
}

// ── Core functions ───────────────────────────────────────────────────────────

/// Apply a displacement map to mesh vertices using UV coordinates.
///
/// For each vertex, samples the map at the vertex UV and displaces the position
/// along its normal by `(sample - midlevel) * amplitude`.
pub fn apply_displacement_map(
    mesh: &MeshBuffers,
    map: &DisplacementMap2D,
    params: &DisplacementApplyParams,
) -> DisplacedMeshResult {
    let ones = vec![1.0f32; mesh.positions.len()];
    apply_displacement_masked(mesh, map, params, &ones)
}

/// Apply displacement to a subset of vertices using per-vertex blend weights.
///
/// `mask[i]` is the blend weight in [0, 1] for vertex i.
pub fn apply_displacement_masked(
    mesh: &MeshBuffers,
    map: &DisplacementMap2D,
    params: &DisplacementApplyParams,
    mask: &[f32],
) -> DisplacedMeshResult {
    let n = mesh.positions.len();
    let mut positions = mesh.positions.clone();

    let mut min_disp = f32::INFINITY;
    let mut max_disp = f32::NEG_INFINITY;
    let mut sum_disp = 0.0f32;

    for i in 0..n {
        let uv = if i < mesh.uvs.len() {
            mesh.uvs[i]
        } else {
            [0.5, 0.5]
        };

        let val = if params.use_tiling {
            map.sample_tiled(uv[0], uv[1])
        } else {
            map.sample(uv[0], uv[1])
        };

        let disp = (val - params.midlevel) * params.amplitude;
        let blend = if i < mask.len() { mask[i] } else { 1.0 };
        let effective_disp = disp * blend;

        let normal = if i < mesh.normals.len() {
            mesh.normals[i]
        } else {
            [0.0, 1.0, 0.0]
        };

        positions[i] = [
            mesh.positions[i][0] + normal[0] * effective_disp,
            mesh.positions[i][1] + normal[1] * effective_disp,
            mesh.positions[i][2] + normal[2] * effective_disp,
        ];

        min_disp = min_disp.min(effective_disp);
        max_disp = max_disp.max(effective_disp);
        sum_disp += effective_disp;
    }

    let mean_displacement = if n > 0 { sum_disp / n as f32 } else { 0.0 };

    if min_disp == f32::INFINITY {
        min_disp = 0.0;
    }
    if max_disp == f32::NEG_INFINITY {
        max_disp = 0.0;
    }

    let mut result_mesh = MeshBuffers {
        positions,
        normals: mesh.normals.clone(),
        tangents: mesh.tangents.clone(),
        uvs: mesh.uvs.clone(),
        indices: mesh.indices.clone(),
        colors: mesh.colors.clone(),
        has_suit: mesh.has_suit,
    };

    if params.smooth_normals {
        compute_normals(&mut result_mesh);
    }

    DisplacedMeshResult {
        mesh: result_mesh,
        min_displacement: min_disp,
        max_displacement: max_disp,
        mean_displacement,
    }
}

/// Bake displaced vertex positions back into a displacement map (inverse of apply).
///
/// For each pixel in the output map, finds the closest vertex by UV and computes
/// the displacement scalar as the position difference projected onto the normal.
pub fn mesh_to_displacement_map(
    base_mesh: &MeshBuffers,
    displaced_mesh: &MeshBuffers,
    width: usize,
    height: usize,
) -> DisplacementMap2D {
    let mut map = DisplacementMap2D::new(width, height);

    if base_mesh.positions.is_empty() || width == 0 || height == 0 {
        return map;
    }

    // Build UV list with indices
    let uvs: Vec<[f32; 2]> = base_mesh.uvs.clone();

    for py in 0..height {
        for px in 0..width {
            let u = if width > 1 {
                px as f32 / (width - 1) as f32
            } else {
                0.5
            };
            let v = if height > 1 {
                py as f32 / (height - 1) as f32
            } else {
                0.5
            };

            // Find closest vertex by UV distance
            let mut best_idx = 0usize;
            let mut best_dist = f32::INFINITY;
            for (i, uv) in uvs.iter().enumerate() {
                let du = uv[0] - u;
                let dv = uv[1] - v;
                let d2 = du * du + dv * dv;
                if d2 < best_dist {
                    best_dist = d2;
                    best_idx = i;
                }
            }

            // Project position difference onto normal
            let disp_val = if best_idx < displaced_mesh.positions.len()
                && best_idx < base_mesh.normals.len()
            {
                let dp = [
                    displaced_mesh.positions[best_idx][0] - base_mesh.positions[best_idx][0],
                    displaced_mesh.positions[best_idx][1] - base_mesh.positions[best_idx][1],
                    displaced_mesh.positions[best_idx][2] - base_mesh.positions[best_idx][2],
                ];
                let n = base_mesh.normals[best_idx];
                dp[0] * n[0] + dp[1] * n[1] + dp[2] * n[2]
            } else {
                0.0
            };

            map.set(px, py, disp_val);
        }
    }

    map
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn make_quad_mesh() -> MeshBuffers {
        // Two-triangle quad in XY plane, normals pointing +Z
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        })
    }

    fn make_flat_map(width: usize, height: usize, value: f32) -> DisplacementMap2D {
        let mut m = DisplacementMap2D::new(width, height);
        for v in &mut m.data {
            *v = value;
        }
        m
    }

    #[test]
    fn test_displacement_map_new() {
        let m = DisplacementMap2D::new(4, 4);
        assert_eq!(m.width, 4);
        assert_eq!(m.height, 4);
        assert_eq!(m.data.len(), 16);
        assert!(m.data.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_displacement_map_get_set() {
        let mut m = DisplacementMap2D::new(3, 3);
        m.set(1, 2, 0.75);
        assert!((m.get(1, 2) - 0.75).abs() < 1e-6);
        assert!((m.get(0, 0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_displacement_map_sample_exact() {
        let mut m = DisplacementMap2D::new(3, 3);
        // Set corners
        m.set(0, 0, 0.0);
        m.set(2, 0, 1.0);
        m.set(0, 2, 0.5);
        m.set(2, 2, 0.25);

        // Sample exact corner u=0, v=0 → (0,0) → 0.0
        let s = m.sample(0.0, 0.0);
        assert!((s - 0.0).abs() < 1e-5, "Expected 0.0, got {s}");

        // Sample exact corner u=1, v=0 → (2,0) → 1.0
        let s = m.sample(1.0, 0.0);
        assert!((s - 1.0).abs() < 1e-5, "Expected 1.0, got {s}");
    }

    #[test]
    fn test_displacement_map_sample_bilinear() {
        let mut m = DisplacementMap2D::new(3, 3);
        // Fill with known values
        // Center pixel at (1,1)
        m.set(0, 0, 0.0);
        m.set(1, 0, 0.0);
        m.set(2, 0, 0.0);
        m.set(0, 1, 0.0);
        m.set(1, 1, 1.0);
        m.set(2, 1, 0.0);
        m.set(0, 2, 0.0);
        m.set(1, 2, 0.0);
        m.set(2, 2, 0.0);

        // Sample at center UV (0.5, 0.5) should map to pixel (1,1) exactly → 1.0
        let s = m.sample(0.5, 0.5);
        assert!(
            (s - 1.0).abs() < 1e-5,
            "Center sample should be 1.0, got {s}"
        );

        // Sample at (0.0, 0.0) → pixel (0,0) → 0.0
        let s = m.sample(0.0, 0.0);
        assert!(
            (s - 0.0).abs() < 1e-5,
            "Corner sample should be 0.0, got {s}"
        );
    }

    #[test]
    fn test_displacement_map_tiled() {
        let mut m = DisplacementMap2D::new(2, 2);
        m.set(0, 0, 1.0);
        m.set(1, 0, 0.0);
        m.set(0, 1, 0.0);
        m.set(1, 1, 0.5);

        // UV 1.25 wraps to 0.25 in u
        // UV 0.0  in v
        let s_tiled = m.sample_tiled(1.0, 0.0);
        let s_ref = m.sample(0.0, 0.0);
        assert!(
            (s_tiled - s_ref).abs() < 1e-5,
            "Tiled should wrap to same as [0,0]"
        );
    }

    #[test]
    fn test_displacement_map_from_fn() {
        let m = DisplacementMap2D::from_fn(5, 5, |u, _v| u);
        // u=0 at x=0, u=1 at x=4
        assert!((m.get(0, 0) - 0.0).abs() < 1e-5);
        assert!((m.get(4, 0) - 1.0).abs() < 1e-5);
        // Middle u=0.5
        assert!((m.get(2, 0) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_displacement_map_invert() {
        let mut m = DisplacementMap2D::new(2, 2);
        m.set(0, 0, 0.0);
        m.set(1, 0, 0.3);
        m.set(0, 1, 0.7);
        m.set(1, 1, 1.0);
        m.invert();
        assert!((m.get(0, 0) - 1.0).abs() < 1e-5);
        assert!((m.get(1, 0) - 0.7).abs() < 1e-5);
        assert!((m.get(0, 1) - 0.3).abs() < 1e-5);
        assert!((m.get(1, 1) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_displacement_map_normalize() {
        let mut m = DisplacementMap2D::new(2, 2);
        m.set(0, 0, 2.0);
        m.set(1, 0, 4.0);
        m.set(0, 1, 6.0);
        m.set(1, 1, 8.0);
        m.normalize();
        // Range is [2,8], so each value → (v - 2) / 6
        assert!((m.get(0, 0) - 0.0).abs() < 1e-5);
        assert!((m.get(1, 1) - 1.0).abs() < 1e-5);
        assert!((m.get(0, 1) - (4.0 / 6.0)).abs() < 1e-5);
    }

    #[test]
    fn test_checkerboard() {
        let m = DisplacementMap2D::checkerboard(4, 4, 2);
        assert_eq!(m.width, 4);
        assert_eq!(m.height, 4);
        // x=0,y=0: (0*2/4 + 0*2/4) = 0 → even → 1.0
        assert!((m.get(0, 0) - 1.0).abs() < 1e-5);
        // x=2,y=0: (2*2/4 + 0) = 1 → odd → 0.0
        assert!((m.get(2, 0) - 0.0).abs() < 1e-5);
        // x=2,y=2: (1 + 1) = 2 → even → 1.0
        assert!((m.get(2, 2) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_displacement_zero_midlevel() {
        // With midlevel=0.5 and map value=0.5, displacement should be zero
        let mesh = make_quad_mesh();
        let map = make_flat_map(4, 4, 0.5);
        let params = DisplacementApplyParams {
            smooth_normals: false,
            ..Default::default()
        };
        let result = apply_displacement_map(&mesh, &map, &params);
        for (orig, disp) in mesh.positions.iter().zip(result.mesh.positions.iter()) {
            let d = ((disp[0] - orig[0]).powi(2)
                + (disp[1] - orig[1]).powi(2)
                + (disp[2] - orig[2]).powi(2))
            .sqrt();
            assert!(d < 1e-5, "Zero displacement expected, got {d}");
        }
        assert!(result.min_displacement.abs() < 1e-5);
        assert!(result.max_displacement.abs() < 1e-5);
    }

    #[test]
    fn test_apply_displacement_basic() {
        // Map value=1.0, midlevel=0.5, amplitude=0.1 → disp=+0.05 along normal (+Z)
        let mesh = make_quad_mesh();
        let map = make_flat_map(4, 4, 1.0);
        let params = DisplacementApplyParams {
            amplitude: 0.1,
            midlevel: 0.5,
            use_tiling: false,
            smooth_normals: false,
        };
        let result = apply_displacement_map(&mesh, &map, &params);

        // Normal is [0,0,1], so Z should increase by 0.05
        for (orig, disp) in mesh.positions.iter().zip(result.mesh.positions.iter()) {
            assert!((disp[0] - orig[0]).abs() < 1e-5, "X should not change");
            assert!((disp[1] - orig[1]).abs() < 1e-5, "Y should not change");
            assert!(
                (disp[2] - (orig[2] + 0.05)).abs() < 1e-5,
                "Z should increase by 0.05"
            );
        }

        assert!((result.min_displacement - 0.05).abs() < 1e-5);
        assert!((result.max_displacement - 0.05).abs() < 1e-5);
        assert!((result.mean_displacement - 0.05).abs() < 1e-5);
    }

    #[test]
    fn test_apply_displacement_masked() {
        let mesh = make_quad_mesh();
        let map = make_flat_map(4, 4, 1.0);
        let params = DisplacementApplyParams {
            amplitude: 0.1,
            midlevel: 0.5,
            use_tiling: false,
            smooth_normals: false,
        };

        // Mask: vertex 0 fully displaced, vertex 1 not displaced
        let mask = vec![1.0f32, 0.0, 1.0, 0.0];
        let result = apply_displacement_masked(&mesh, &map, &params, &mask);

        // Vertex 0 (mask=1.0): Z should increase by 0.05
        assert!((result.mesh.positions[0][2] - 0.05).abs() < 1e-5);
        // Vertex 1 (mask=0.0): Z should remain 0.0
        assert!(result.mesh.positions[1][2].abs() < 1e-5);
    }

    #[test]
    fn test_displacement_params_default() {
        let p = DisplacementApplyParams::default();
        assert!((p.amplitude - 0.02).abs() < 1e-6);
        assert!((p.midlevel - 0.5).abs() < 1e-6);
        assert!(!p.use_tiling);
        assert!(p.smooth_normals);
    }
}
