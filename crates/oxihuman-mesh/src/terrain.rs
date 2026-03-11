// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ---------------------------------------------------------------------------
// HeightField
// ---------------------------------------------------------------------------

/// 2D height field grid stored in row-major order: `data[y * width + x]`.
pub struct HeightField {
    pub data: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl HeightField {
    /// Create a height field filled with zeros.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![0.0; width * height],
            width,
            height,
        }
    }

    /// Create a height field filled with a constant value.
    pub fn flat(width: usize, height: usize, h: f32) -> Self {
        Self {
            data: vec![h; width * height],
            width,
            height,
        }
    }

    /// Get height at grid position (x, y).
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.data[y * self.width + x]
    }

    /// Set height at grid position (x, y).
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        self.data[y * self.width + x] = val;
    }

    /// Return the minimum height value in the field.
    pub fn min_height(&self) -> f32 {
        self.data.iter().cloned().fold(f32::INFINITY, f32::min)
    }

    /// Return the maximum height value in the field.
    pub fn max_height(&self) -> f32 {
        self.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
    }

    /// Bilinear sample at normalised coordinates u, v in [0, 1].
    pub fn sample_bilinear(&self, u: f32, v: f32) -> f32 {
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let fx = u * (self.width as f32 - 1.0);
        let fz = v * (self.height as f32 - 1.0);

        let x0 = (fx.floor() as usize).min(self.width - 1);
        let z0 = (fz.floor() as usize).min(self.height - 1);
        let x1 = (x0 + 1).min(self.width - 1);
        let z1 = (z0 + 1).min(self.height - 1);

        let tx = fx - fx.floor();
        let tz = fz - fz.floor();

        let h00 = self.get(x0, z0);
        let h10 = self.get(x1, z0);
        let h01 = self.get(x0, z1);
        let h11 = self.get(x1, z1);

        let h0 = h00 + tx * (h10 - h00);
        let h1 = h01 + tx * (h11 - h01);
        h0 + tz * (h1 - h0)
    }

    /// Scale all height data so values span [0, 1].
    pub fn normalize(&mut self) {
        let lo = self.min_height();
        let hi = self.max_height();
        let range = hi - lo;
        if range < 1e-10 {
            for v in &mut self.data {
                *v = 0.0;
            }
        } else {
            for v in &mut self.data {
                *v = (*v - lo) / range;
            }
        }
    }

    /// Build a height field from a closure `f(x_norm, y_norm) -> height`
    /// where the normalised coordinates are in [0, 1].
    pub fn from_fn(width: usize, height: usize, f: impl Fn(f32, f32) -> f32) -> Self {
        let mut field = Self::new(width, height);
        for y in 0..height {
            let yn = if height > 1 {
                y as f32 / (height - 1) as f32
            } else {
                0.0
            };
            for x in 0..width {
                let xn = if width > 1 {
                    x as f32 / (width - 1) as f32
                } else {
                    0.0
                };
                field.set(x, y, f(xn, yn));
            }
        }
        field
    }
}

// ---------------------------------------------------------------------------
// TerrainParams
// ---------------------------------------------------------------------------

/// Parameters that control world-space scale and UV tiling of terrain meshes.
pub struct TerrainParams {
    pub scale_x: f32,
    pub scale_z: f32,
    pub scale_y: f32,
    pub uv_tile: f32,
}

impl Default for TerrainParams {
    fn default() -> Self {
        Self {
            scale_x: 10.0,
            scale_z: 10.0,
            scale_y: 1.0,
            uv_tile: 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Core terrain mesh builder
// ---------------------------------------------------------------------------

/// Generate a triangulated terrain mesh from a `HeightField`.
///
/// Each quad is split into two triangles (tl, bl, br) and (tl, br, tr).
pub fn terrain_from_heightfield(field: &HeightField, params: &TerrainParams) -> MeshBuffers {
    let w = field.width;
    let h = field.height;

    assert!(w >= 2 && h >= 2, "HeightField must be at least 2x2");

    let cols = w;
    let rows = h;

    let mut positions = Vec::with_capacity(cols * rows);
    let mut uvs = Vec::with_capacity(cols * rows);
    let mut indices = Vec::with_capacity((cols - 1) * (rows - 1) * 6);

    for row in 0..rows {
        let z = row as f32 * params.scale_z / (rows - 1) as f32;
        let v = row as f32 / (rows - 1) as f32 * params.uv_tile;
        for col in 0..cols {
            let x = col as f32 * params.scale_x / (cols - 1) as f32;
            let y = field.get(col, row) * params.scale_y;
            let u = col as f32 / (cols - 1) as f32 * params.uv_tile;
            positions.push([x, y, z]);
            uvs.push([u, v]);
        }
    }

    for row in 0..(rows - 1) {
        for col in 0..(cols - 1) {
            let tl = (row * cols + col) as u32;
            let bl = ((row + 1) * cols + col) as u32;
            let br = ((row + 1) * cols + col + 1) as u32;
            let tr = (row * cols + col + 1) as u32;

            // Triangle 1: tl, bl, br
            indices.push(tl);
            indices.push(bl);
            indices.push(br);

            // Triangle 2: tl, br, tr
            indices.push(tl);
            indices.push(br);
            indices.push(tr);
        }
    }

    let n = positions.len();
    let mut mesh = MeshBuffers {
        positions,
        normals: vec![[0.0, 1.0, 0.0]; n],
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
        uvs,
        indices,
        colors: None,
        has_suit: false,
    };

    compute_normals(&mut mesh);
    mesh
}

// ---------------------------------------------------------------------------
// Convenience generators
// ---------------------------------------------------------------------------

/// Generate a flat grid mesh with `rows` × `cols` quads.
pub fn generate_grid(rows: usize, cols: usize, width: f32, depth: f32) -> MeshBuffers {
    let field_cols = cols + 1;
    let field_rows = rows + 1;
    let field = HeightField::flat(field_cols, field_rows, 0.0);
    let params = TerrainParams {
        scale_x: width,
        scale_z: depth,
        scale_y: 1.0,
        uv_tile: 1.0,
    };
    terrain_from_heightfield(&field, &params)
}

/// Generate a sinusoidal hill terrain: `height = sin(x * freq * 2π) * cos(z * freq * 2π) * amplitude`.
pub fn generate_sine_terrain(
    width: usize,
    height: usize,
    frequency: f32,
    amplitude: f32,
) -> MeshBuffers {
    let field = HeightField::from_fn(width, height, |x, z| {
        use std::f32::consts::TAU;
        (x * frequency * TAU).sin() * (z * frequency * TAU).cos() * amplitude
    });
    let params = TerrainParams::default();
    terrain_from_heightfield(&field, &params)
}

/// Generate a radial dome terrain: `height = radius * (1 - r²)` where r is normalised distance from centre.
pub fn generate_dome_terrain(width: usize, height: usize, radius: f32) -> MeshBuffers {
    let field = HeightField::from_fn(width, height, |x, z| {
        let cx = x - 0.5;
        let cz = z - 0.5;
        let r2 = (cx * cx + cz * cz) * 4.0; // scale so r=1 at corners
        let r2 = r2.min(1.0);
        radius * (1.0 - r2)
    });
    let params = TerrainParams::default();
    terrain_from_heightfield(&field, &params)
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/// Extract a height field from a mesh by sampling Y values on a regular grid.
///
/// The sampling uses the bounding-box of the mesh to normalise XZ coordinates.
pub fn mesh_to_heightfield(mesh: &MeshBuffers, width: usize, height: usize) -> HeightField {
    let mut field = HeightField::new(width, height);

    if mesh.positions.is_empty() || width == 0 || height == 0 {
        return field;
    }

    let min_x = mesh
        .positions
        .iter()
        .map(|p| p[0])
        .fold(f32::INFINITY, f32::min);
    let max_x = mesh
        .positions
        .iter()
        .map(|p| p[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let min_z = mesh
        .positions
        .iter()
        .map(|p| p[2])
        .fold(f32::INFINITY, f32::min);
    let max_z = mesh
        .positions
        .iter()
        .map(|p| p[2])
        .fold(f32::NEG_INFINITY, f32::max);

    let range_x = (max_x - min_x).max(1e-10);
    let range_z = (max_z - min_z).max(1e-10);

    // For each grid cell, find the nearest vertex and take its Y.
    for gy in 0..height {
        let gz_norm = gy as f32 / (height - 1).max(1) as f32;
        for gx in 0..width {
            let gx_norm = gx as f32 / (width - 1).max(1) as f32;
            let wx = min_x + gx_norm * range_x;
            let wz = min_z + gz_norm * range_z;

            let best_y = mesh
                .positions
                .iter()
                .min_by(|a, b| {
                    let da = (a[0] - wx).powi(2) + (a[2] - wz).powi(2);
                    let db = (b[0] - wx).powi(2) + (b[2] - wz).powi(2);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|p| p[1])
                .unwrap_or(0.0);

            field.set(gx, gy, best_y);
        }
    }

    field
}

/// Compute the terrain slope (gradient magnitude) at each grid cell using finite differences.
pub fn compute_slope(field: &HeightField) -> HeightField {
    let w = field.width;
    let h = field.height;
    let mut out = HeightField::new(w, h);

    for y in 0..h {
        for x in 0..w {
            // Central difference where possible, forward/backward at edges.
            let x_prev = if x > 0 { x - 1 } else { x };
            let x_next = if x + 1 < w { x + 1 } else { x };
            let y_prev = if y > 0 { y - 1 } else { y };
            let y_next = if y + 1 < h { y + 1 } else { y };

            let dx = (x_next - x_prev) as f32;
            let dz = (y_next - y_prev) as f32;

            let dh_dx = (field.get(x_next, y) - field.get(x_prev, y)) / dx;
            let dh_dz = (field.get(x, y_next) - field.get(x, y_prev)) / dz;

            out.set(x, y, (dh_dx * dh_dx + dh_dz * dh_dz).sqrt());
        }
    }

    out
}

/// Smooth a height field with a uniform box blur of the given radius.
pub fn smooth_heightfield(field: &HeightField, radius: usize) -> HeightField {
    let w = field.width;
    let h = field.height;
    let mut out = HeightField::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let x0 = x.saturating_sub(radius);
            let x1 = (x + radius).min(w - 1);
            let y0 = y.saturating_sub(radius);
            let y1 = (y + radius).min(h - 1);

            let mut sum = 0.0f32;
            let mut count = 0u32;
            for sy in y0..=y1 {
                for sx in x0..=x1 {
                    sum += field.get(sx, sy);
                    count += 1;
                }
            }

            out.set(x, y, if count > 0 { sum / count as f32 } else { 0.0 });
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heightfield_new() {
        let f = HeightField::new(4, 3);
        assert_eq!(f.width, 4);
        assert_eq!(f.height, 3);
        assert_eq!(f.data.len(), 12);
        assert!(f.data.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_heightfield_get_set() {
        let mut f = HeightField::new(3, 3);
        f.set(1, 2, 5.0);
        assert_eq!(f.get(1, 2), 5.0);
        assert_eq!(f.get(0, 0), 0.0);
    }

    #[test]
    fn test_heightfield_flat() {
        let f = HeightField::flat(5, 4, 1.5);
        assert!(f.data.iter().all(|&v| (v - 1.5).abs() < 1e-6));
    }

    #[test]
    fn test_heightfield_min_max() {
        let mut f = HeightField::new(3, 3);
        f.set(0, 0, -1.0);
        f.set(2, 2, 5.0);
        assert!((f.min_height() - (-1.0)).abs() < 1e-6);
        assert!((f.max_height() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_heightfield_normalize() {
        let mut f = HeightField::new(2, 2);
        f.set(0, 0, 2.0);
        f.set(1, 0, 4.0);
        f.set(0, 1, 6.0);
        f.set(1, 1, 8.0);
        f.normalize();
        assert!((f.get(0, 0) - 0.0).abs() < 1e-6);
        assert!((f.get(1, 1) - 1.0).abs() < 1e-6);
        let mid = f.get(1, 0);
        assert!((mid - (2.0 / 6.0)).abs() < 1e-5, "mid={}", mid);
    }

    #[test]
    fn test_heightfield_from_fn() {
        let f = HeightField::from_fn(5, 5, |x, y| x + y);
        assert!((f.get(0, 0) - 0.0).abs() < 1e-6);
        assert!((f.get(4, 4) - 2.0).abs() < 1e-6);
        // Centre at (2,2) -> (0.5, 0.5) -> 1.0
        assert!((f.get(2, 2) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_heightfield_sample_bilinear() {
        // 2x2 field: corners 0,1,2,3
        let mut f = HeightField::new(2, 2);
        f.set(0, 0, 0.0); // top-left
        f.set(1, 0, 1.0); // top-right
        f.set(0, 1, 2.0); // bottom-left
        f.set(1, 1, 3.0); // bottom-right

        // At (0,0) -> 0.0
        assert!((f.sample_bilinear(0.0, 0.0) - 0.0).abs() < 1e-5);
        // At (1,0) -> 1.0
        assert!((f.sample_bilinear(1.0, 0.0) - 1.0).abs() < 1e-5);
        // At centre (0.5, 0.5) -> average = 1.5
        let mid = f.sample_bilinear(0.5, 0.5);
        assert!((mid - 1.5).abs() < 1e-5, "expected 1.5, got {}", mid);
    }

    #[test]
    fn test_terrain_from_heightfield_basic() {
        let mut f = HeightField::new(3, 3);
        f.set(1, 1, 2.0); // peak in the centre
        let params = TerrainParams::default();
        let mesh = terrain_from_heightfield(&f, &params);

        // 3x3 = 9 vertices, (3-1)*(3-1)*2*3 = 24 indices
        assert_eq!(mesh.positions.len(), 9);
        assert_eq!(mesh.indices.len(), 24);
        assert_eq!(mesh.uvs.len(), 9);

        // Peak vertex is at grid (1,1)
        let peak_idx = 4; // row=1, col=1
        let peak_y = mesh.positions[peak_idx][1];
        assert!((peak_y - 2.0).abs() < 1e-5, "peak_y={}", peak_y);

        // Normals must not contain NaN
        for n in &mesh.normals {
            assert!(!n[0].is_nan() && !n[1].is_nan() && !n[2].is_nan());
        }
    }

    #[test]
    fn test_generate_grid() {
        let mesh = generate_grid(4, 4, 10.0, 10.0);
        // (4+1)*(4+1) = 25 vertices, 4*4*6 = 96 indices
        assert_eq!(mesh.positions.len(), 25);
        assert_eq!(mesh.indices.len(), 96);
        // All Y must be 0
        for p in &mesh.positions {
            assert!((p[1] - 0.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_generate_sine_terrain() {
        let mesh = generate_sine_terrain(8, 8, 1.0, 2.0);
        // 8*8 = 64 vertices
        assert_eq!(mesh.positions.len(), 64);
        assert_eq!(mesh.indices.len(), 7 * 7 * 6);

        // Y values should be in [-2, 2]
        for p in &mesh.positions {
            assert!(p[1] >= -2.01 && p[1] <= 2.01, "y out of range: {}", p[1]);
        }
    }

    #[test]
    fn test_generate_dome_terrain() {
        let mesh = generate_dome_terrain(9, 9, 3.0);
        assert_eq!(mesh.positions.len(), 81);

        // Centre vertex should have the highest Y
        let centre_idx = 4 * 9 + 4; // row=4, col=4 in a 9x9 grid
        let centre_y = mesh.positions[centre_idx][1];
        for p in &mesh.positions {
            assert!(p[1] <= centre_y + 1e-4, "non-centre higher than centre");
        }
    }

    #[test]
    fn test_compute_slope() {
        // Flat field → slope should be near 0 everywhere
        let flat = HeightField::flat(5, 5, 1.0);
        let slope = compute_slope(&flat);
        for v in &slope.data {
            assert!(v.abs() < 1e-6, "slope on flat field should be 0, got {}", v);
        }

        // Ramp along X → dh/dx = 1, dh/dz = 0 → slope ≈ 1 (at interior)
        let ramp = HeightField::from_fn(5, 5, |x, _| x);
        let slope_ramp = compute_slope(&ramp);
        // Interior cell (2,2)
        let s = slope_ramp.get(2, 2);
        // central diff: (x_next - x_prev) / 2 = (0.75 - 0.25) / 2 = 0.25
        // But the field goes from 0..1 across 5 columns.
        // x_norm for col 3 = 0.75, col 1 = 0.25; dx grid = 2
        // dh/dx = (0.75 - 0.25) / 2 = 0.25
        assert!(s > 0.0, "slope should be positive on ramp, got {}", s);
    }

    #[test]
    fn test_smooth_heightfield() {
        // Noisy single spike → smoothing should reduce the peak
        let mut f = HeightField::flat(5, 5, 0.0);
        f.set(2, 2, 100.0);
        let smoothed = smooth_heightfield(&f, 1);
        let peak = smoothed.get(2, 2);
        // Peak should be reduced (average over 3x3 box = 100/9)
        assert!(peak < 100.0, "peak should be reduced after smoothing");
        assert!(peak > 0.0, "smoothed peak should be positive");
    }

    #[test]
    fn test_mesh_to_heightfield() {
        // Build a dome terrain and re-sample it
        let mesh = generate_dome_terrain(9, 9, 2.0);
        let field = mesh_to_heightfield(&mesh, 5, 5);
        assert_eq!(field.width, 5);
        assert_eq!(field.height, 5);
        // All heights should be non-negative (dome is ≥ 0)
        for v in &field.data {
            assert!(*v >= -1e-4, "height below zero: {}", v);
        }
    }
}
