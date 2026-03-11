// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

use crate::mesh::MeshBuffers;
use std::io::Write;

// ---------------------------------------------------------------------------
// Internal math helpers
// ---------------------------------------------------------------------------

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l > 1e-10 {
        scale3(v, 1.0 / l)
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[inline]
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

// Clamp scalar to [lo, hi]
#[inline]
fn clamp_f32(v: f32, lo: f32, hi: f32) -> f32 {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

// ---------------------------------------------------------------------------
// NormalMapBakeParams
// ---------------------------------------------------------------------------

/// Parameters for normal map baking.
#[derive(Debug, Clone)]
pub struct NormalMapBakeParams {
    /// Texture width in pixels (default 512).
    pub width: usize,
    /// Texture height in pixels (default 512).
    pub height: usize,
    /// Ray origin offset along low-poly normal (default 0.01).
    pub cage_offset: f32,
    /// true = tangent-space normal map, false = object-space.
    pub tangent_space: bool,
    /// Background color for unmapped pixels (default [0.5, 0.5, 1.0]).
    pub background: [f32; 3],
}

impl Default for NormalMapBakeParams {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            cage_offset: 0.01,
            tangent_space: true,
            background: [0.5, 0.5, 1.0],
        }
    }
}

// ---------------------------------------------------------------------------
// NormalMapTexture
// ---------------------------------------------------------------------------

/// Baked normal map texture with RGB pixels in [0, 1].
#[derive(Debug, Clone)]
pub struct NormalMapTexture {
    /// RGB pixels in row-major order, values in [0, 1].
    pub pixels: Vec<[f32; 3]>,
    pub width: usize,
    pub height: usize,
    pub filled_pixels: usize,
    /// filled_pixels / (width * height)
    pub coverage: f32,
}

impl NormalMapTexture {
    /// Create a new texture filled with the background color.
    pub fn new(width: usize, height: usize, background: [f32; 3]) -> Self {
        let total = width * height;
        Self {
            pixels: vec![background; total],
            width,
            height,
            filled_pixels: 0,
            coverage: 0.0,
        }
    }

    /// Get pixel color at (x, y).
    pub fn get(&self, x: usize, y: usize) -> [f32; 3] {
        self.pixels[y * self.width + x]
    }

    /// Set pixel color at (x, y).
    pub fn set(&mut self, x: usize, y: usize, color: [f32; 3]) {
        self.pixels[y * self.width + x] = color;
    }

    /// Convert to u8 RGB (values clamped to [0, 255]).
    pub fn to_rgb_u8(&self) -> Vec<[u8; 3]> {
        self.pixels
            .iter()
            .map(|p| {
                [
                    (clamp_f32(p[0], 0.0, 1.0) * 255.0).round() as u8,
                    (clamp_f32(p[1], 0.0, 1.0) * 255.0).round() as u8,
                    (clamp_f32(p[2], 0.0, 1.0) * 255.0).round() as u8,
                ]
            })
            .collect()
    }

    /// Export as PPM P6 binary format.
    pub fn save_ppm(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let mut file = std::fs::File::create(path)?;
        // Write PPM header
        write!(file, "P6\n{} {}\n255\n", self.width, self.height)?;
        // Write binary RGB bytes
        let rgb_u8 = self.to_rgb_u8();
        for pixel in &rgb_u8 {
            file.write_all(pixel)?;
        }
        file.flush()?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Normal encoding / decoding
// ---------------------------------------------------------------------------

/// Encode a normal vector to RGB (normal map convention: `[0,1]` range).
/// out = (n + 1.0) / 2.0
pub fn normal_to_rgb(normal: [f32; 3]) -> [f32; 3] {
    [
        (normal[0] + 1.0) * 0.5,
        (normal[1] + 1.0) * 0.5,
        (normal[2] + 1.0) * 0.5,
    ]
}

/// Decode RGB to normal vector.
pub fn rgb_to_normal(rgb: [f32; 3]) -> [f32; 3] {
    [rgb[0] * 2.0 - 1.0, rgb[1] * 2.0 - 1.0, rgb[2] * 2.0 - 1.0]
}

// ---------------------------------------------------------------------------
// UV triangle rasterizer
// ---------------------------------------------------------------------------

/// Rasterize a UV triangle into texture pixels (fill-rule: center-point test).
/// Returns list of (pixel_x, pixel_y, barycentric_u, barycentric_v, barycentric_w).
pub fn rasterize_uv_triangle(
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
    tex_w: usize,
    tex_h: usize,
) -> Vec<(usize, usize, f32, f32, f32)> {
    // Convert UV to pixel coords: px = u * w, py = (1 - v) * h (V flip)
    let tw = tex_w as f32;
    let th = tex_h as f32;

    let px0 = uv0[0] * tw;
    let py0 = (1.0 - uv0[1]) * th;
    let px1 = uv1[0] * tw;
    let py1 = (1.0 - uv1[1]) * th;
    let px2 = uv2[0] * tw;
    let py2 = (1.0 - uv2[1]) * th;

    // Bounding box
    let min_x = px0.min(px1).min(px2).floor().max(0.0) as usize;
    let max_x = px0.max(px1).max(px2).ceil().min(tw - 1.0) as usize;
    let min_y = py0.min(py1).min(py2).floor().max(0.0) as usize;
    let max_y = py0.max(py1).max(py2).ceil().min(th - 1.0) as usize;

    if min_x > max_x || min_y > max_y {
        return Vec::new();
    }

    // Triangle area (signed) for barycentric computation
    let denom = (py1 - py2) * (px0 - px2) + (px2 - px1) * (py0 - py2);
    if denom.abs() < 1e-10 {
        return Vec::new();
    }

    let mut result = Vec::new();

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            // Test pixel center
            let cx = px as f32 + 0.5;
            let cy = py as f32 + 0.5;

            // Barycentric coordinates
            let w0 = ((py1 - py2) * (cx - px2) + (px2 - px1) * (cy - py2)) / denom;
            let w1 = ((py2 - py0) * (cx - px2) + (px0 - px2) * (cy - py2)) / denom;
            let w2 = 1.0 - w0 - w1;

            // Inside test: all coords >= 0 (with small epsilon for robustness)
            if w0 >= -1e-5 && w1 >= -1e-5 && w2 >= -1e-5 {
                result.push((px, py, w0, w1, w2));
            }
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Closest surface point on mesh
// ---------------------------------------------------------------------------

/// Find the closest point on the high-poly mesh surface to a world-space point.
/// Returns (closest_position, interpolated_normal).
pub fn closest_surface_point(mesh: &MeshBuffers, point: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let mut best_dist_sq = f32::MAX;
    let mut best_pos = point;
    let mut best_normal = [0.0f32, 0.0, 1.0];

    let num_faces = mesh.indices.len() / 3;

    for face_idx in 0..num_faces {
        let i0 = mesh.indices[face_idx * 3] as usize;
        let i1 = mesh.indices[face_idx * 3 + 1] as usize;
        let i2 = mesh.indices[face_idx * 3 + 2] as usize;

        let v0 = mesh.positions[i0];
        let v1 = mesh.positions[i1];
        let v2 = mesh.positions[i2];

        let n0 = mesh.normals[i0];
        let n1 = mesh.normals[i1];
        let n2 = mesh.normals[i2];

        let (closest, bary) = closest_point_on_triangle(point, v0, v1, v2);

        let dx = closest[0] - point[0];
        let dy = closest[1] - point[1];
        let dz = closest[2] - point[2];
        let dist_sq = dx * dx + dy * dy + dz * dz;

        if dist_sq < best_dist_sq {
            best_dist_sq = dist_sq;
            best_pos = closest;
            // Interpolate normal using barycentric coords
            let interp = add3(
                add3(scale3(n0, bary[0]), scale3(n1, bary[1])),
                scale3(n2, bary[2]),
            );
            best_normal = normalize3(interp);
        }
    }

    (best_pos, best_normal)
}

/// Find the closest point on a triangle to a query point.
/// Returns (closest_point, barycentric_coords).
fn closest_point_on_triangle(
    p: [f32; 3],
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
) -> ([f32; 3], [f32; 3]) {
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let ap = sub3(p, a);

    let d1 = dot3(ab, ap);
    let d2 = dot3(ac, ap);

    // Check if P is in vertex region of A
    if d1 <= 0.0 && d2 <= 0.0 {
        return (a, [1.0, 0.0, 0.0]);
    }

    let bp = sub3(p, b);
    let d3 = dot3(ab, bp);
    let d4 = dot3(ac, bp);

    // Check if P is in vertex region of B
    if d3 >= 0.0 && d4 <= d3 {
        return (b, [0.0, 1.0, 0.0]);
    }

    // Check if P is in edge region of AB
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        let pt = add3(a, scale3(ab, v));
        return (pt, [1.0 - v, v, 0.0]);
    }

    let cp = sub3(p, c);
    let d5 = dot3(ab, cp);
    let d6 = dot3(ac, cp);

    // Check if P is in vertex region of C
    if d6 >= 0.0 && d5 <= d6 {
        return (c, [0.0, 0.0, 1.0]);
    }

    // Check if P is in edge region of AC
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        let pt = add3(a, scale3(ac, w));
        return (pt, [1.0 - w, 0.0, w]);
    }

    // Check if P is in edge region of BC
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let pt = add3(b, scale3(sub3(c, b), w));
        return (pt, [0.0, 1.0 - w, w]);
    }

    // P is inside the triangle
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let u = 1.0 - v - w;
    let pt = add3(add3(scale3(a, u), scale3(b, v)), scale3(c, w));
    (pt, [u, v, w])
}

// ---------------------------------------------------------------------------
// TBN matrix computation
// ---------------------------------------------------------------------------

/// Compute TBN matrix for a low-poly triangle given positions, normals, uvs.
/// Returns (tangent, bitangent, normal) — all normalized.
fn compute_tbn(
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
    n_interp: [f32; 3],
) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let edge1 = sub3(p1, p0);
    let edge2 = sub3(p2, p0);

    let delta_uv1 = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
    let delta_uv2 = [uv2[0] - uv0[0], uv2[1] - uv0[1]];

    let denom = delta_uv1[0] * delta_uv2[1] - delta_uv2[0] * delta_uv1[1];

    let tangent = if denom.abs() > 1e-10 {
        let inv = 1.0 / denom;
        normalize3([
            inv * (delta_uv2[1] * edge1[0] - delta_uv1[1] * edge2[0]),
            inv * (delta_uv2[1] * edge1[1] - delta_uv1[1] * edge2[1]),
            inv * (delta_uv2[1] * edge1[2] - delta_uv1[1] * edge2[2]),
        ])
    } else {
        // Fallback tangent if UV degenerate
        let fallback = if edge1[0].abs() > 0.9 {
            [0.0, 1.0, 0.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        normalize3(fallback)
    };

    let n = normalize3(n_interp);
    // Re-orthogonalize tangent against normal (Gram-Schmidt)
    let t_proj = dot3(tangent, n);
    let tangent = normalize3(sub3(tangent, scale3(n, t_proj)));
    let bitangent = cross3(n, tangent);

    (tangent, bitangent, n)
}

// ---------------------------------------------------------------------------
// Main bake function
// ---------------------------------------------------------------------------

/// Bake normal map from high-poly onto low-poly UV space.
pub fn bake_normal_map(
    low_poly: &MeshBuffers,
    high_poly: &MeshBuffers,
    params: &NormalMapBakeParams,
) -> NormalMapTexture {
    let mut texture = NormalMapTexture::new(params.width, params.height, params.background);

    // Track which pixels have been written
    let total_pixels = params.width * params.height;
    let mut written = vec![false; total_pixels];

    let num_faces = low_poly.indices.len() / 3;

    for face_idx in 0..num_faces {
        let i0 = low_poly.indices[face_idx * 3] as usize;
        let i1 = low_poly.indices[face_idx * 3 + 1] as usize;
        let i2 = low_poly.indices[face_idx * 3 + 2] as usize;

        let p0 = low_poly.positions[i0];
        let p1 = low_poly.positions[i1];
        let p2 = low_poly.positions[i2];

        let n0 = low_poly.normals[i0];
        let n1 = low_poly.normals[i1];
        let n2 = low_poly.normals[i2];

        let uv0 = low_poly.uvs[i0];
        let uv1 = low_poly.uvs[i1];
        let uv2 = low_poly.uvs[i2];

        // Rasterize UV triangle
        let pixels = rasterize_uv_triangle(uv0, uv1, uv2, params.width, params.height);

        for (px, py, w0, w1, w2) in pixels {
            // Interpolate 3D position using barycentric weights
            let world_pos = add3(add3(scale3(p0, w0), scale3(p1, w1)), scale3(p2, w2));

            // Offset along interpolated low-poly normal to avoid self-intersection
            let low_n_interp =
                normalize3(add3(add3(scale3(n0, w0), scale3(n1, w1)), scale3(n2, w2)));
            let ray_origin = add3(world_pos, scale3(low_n_interp, params.cage_offset));

            // Find closest point on high-poly mesh
            let (_closest_pos, high_normal) = closest_surface_point(high_poly, ray_origin);

            let final_normal = if params.tangent_space {
                // Transform high-poly normal into tangent space of low-poly triangle
                let (tangent, bitangent, face_normal) =
                    compute_tbn(p0, p1, p2, uv0, uv1, uv2, low_n_interp);

                let ts_x = dot3(high_normal, tangent);
                let ts_y = dot3(high_normal, bitangent);
                let ts_z = dot3(high_normal, face_normal);

                normalize3([ts_x, ts_y, ts_z])
            } else {
                high_normal
            };

            let rgb = normal_to_rgb(final_normal);
            texture.set(px, py, rgb);

            let pixel_idx = py * params.width + px;
            if !written[pixel_idx] {
                written[pixel_idx] = true;
            }
        }
    }

    // Count filled pixels using the written bitmap
    let filled = written.iter().filter(|&&w| w).count();
    texture.filled_pixels = filled;
    texture.coverage = filled as f32 / (params.width * params.height) as f32;

    texture
}

// ---------------------------------------------------------------------------
// Interpolation helper (used in tests)
// ---------------------------------------------------------------------------

fn lerp_normal(n0: [f32; 3], n1: [f32; 3], t: f32) -> [f32; 3] {
    normalize3(lerp3(n0, n1, t))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MorphMB;

    fn make_mesh(
        positions: Vec<[f32; 3]>,
        normals: Vec<[f32; 3]>,
        uvs: Vec<[f32; 2]>,
        indices: Vec<u32>,
    ) -> MeshBuffers {
        MeshBuffers::from_morph(MorphMB {
            positions,
            normals,
            uvs,
            indices,
            has_suit: false,
        })
    }

    /// Simple unit quad mesh (2 triangles) covering UV [0,1]x[0,1]
    fn quad_mesh() -> MeshBuffers {
        make_mesh(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0.0, 0.0, 1.0]; 4],
            vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            vec![0, 1, 2, 0, 2, 3],
        )
    }

    /// Simple single triangle mesh
    fn single_tri_mesh() -> MeshBuffers {
        make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0.0, 0.0, 1.0]; 3],
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            vec![0, 1, 2],
        )
    }

    // -----------------------------------------------------------------------
    // Texture construction
    // -----------------------------------------------------------------------

    #[test]
    fn test_normal_map_texture_new() {
        let bg = [0.5f32, 0.5, 1.0];
        let tex = NormalMapTexture::new(4, 4, bg);
        assert_eq!(tex.width, 4);
        assert_eq!(tex.height, 4);
        assert_eq!(tex.pixels.len(), 16);
        assert_eq!(tex.filled_pixels, 0);
        assert!((tex.coverage - 0.0).abs() < 1e-6);
        for p in &tex.pixels {
            assert!((p[0] - bg[0]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_normal_map_get_set() {
        let mut tex = NormalMapTexture::new(8, 8, [0.5, 0.5, 1.0]);
        let color = [0.1f32, 0.9, 0.5];
        tex.set(3, 5, color);
        let got = tex.get(3, 5);
        assert!((got[0] - color[0]).abs() < 1e-6);
        assert!((got[1] - color[1]).abs() < 1e-6);
        assert!((got[2] - color[2]).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // Normal encoding/decoding
    // -----------------------------------------------------------------------

    #[test]
    fn test_normal_to_rgb_encoding() {
        // (0, 0, 1) -> (0.5, 0.5, 1.0)
        let n = [0.0f32, 0.0, 1.0];
        let rgb = normal_to_rgb(n);
        assert!((rgb[0] - 0.5).abs() < 1e-6);
        assert!((rgb[1] - 0.5).abs() < 1e-6);
        assert!((rgb[2] - 1.0).abs() < 1e-6);

        // (-1, 0, 0) -> (0, 0.5, 0.5)
        let n2 = [-1.0f32, 0.0, 0.0];
        let rgb2 = normal_to_rgb(n2);
        assert!((rgb2[0] - 0.0).abs() < 1e-6);
        assert!((rgb2[1] - 0.5).abs() < 1e-6);
        assert!((rgb2[2] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_rgb_to_normal_decoding() {
        // (0.5, 0.5, 1.0) -> (0, 0, 1)
        let rgb = [0.5f32, 0.5, 1.0];
        let n = rgb_to_normal(rgb);
        assert!((n[0] - 0.0).abs() < 1e-6);
        assert!((n[1] - 0.0).abs() < 1e-6);
        assert!((n[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normal_roundtrip() {
        let normals = [
            [0.0f32, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, -1.0],
        ];
        for n in &normals {
            let rgb = normal_to_rgb(*n);
            let decoded = rgb_to_normal(rgb);
            assert!((decoded[0] - n[0]).abs() < 1e-5, "x mismatch for {:?}", n);
            assert!((decoded[1] - n[1]).abs() < 1e-5, "y mismatch for {:?}", n);
            assert!((decoded[2] - n[2]).abs() < 1e-5, "z mismatch for {:?}", n);
        }
    }

    // -----------------------------------------------------------------------
    // Rasterization
    // -----------------------------------------------------------------------

    #[test]
    fn test_rasterize_uv_triangle_basic() {
        // Full texture-covering triangle
        let uv0 = [0.0f32, 0.0];
        let uv1 = [1.0, 0.0];
        let uv2 = [0.5, 1.0];
        let pixels = rasterize_uv_triangle(uv0, uv1, uv2, 8, 8);
        // Should cover some pixels
        assert!(!pixels.is_empty(), "Should rasterize at least one pixel");

        // All barycentric coords should sum to ~1
        for (_, _, w0, w1, w2) in &pixels {
            let sum = w0 + w1 + w2;
            assert!((sum - 1.0).abs() < 1e-4, "bary sum = {}", sum);
        }
    }

    #[test]
    fn test_rasterize_uv_triangle_empty() {
        // Degenerate (zero-area) triangle
        let uv0 = [0.5f32, 0.5];
        let uv1 = [0.5, 0.5];
        let uv2 = [0.5, 0.5];
        let pixels = rasterize_uv_triangle(uv0, uv1, uv2, 8, 8);
        assert!(
            pixels.is_empty(),
            "Degenerate triangle should produce no pixels"
        );
    }

    // -----------------------------------------------------------------------
    // PPM export
    // -----------------------------------------------------------------------

    #[test]
    fn test_save_ppm() {
        let tex = NormalMapTexture::new(4, 4, [0.5, 0.5, 1.0]);
        let path = std::path::Path::new("/tmp/test_normal_map.ppm");
        let result = tex.save_ppm(path);
        assert!(result.is_ok(), "save_ppm failed: {:?}", result.err());
        // Verify file exists and has non-zero size
        let meta = std::fs::metadata(path).unwrap();
        assert!(meta.len() > 0);
        // Verify PPM header by reading first bytes
        let data = std::fs::read(path).unwrap();
        assert_eq!(&data[0..2], b"P6");
    }

    // -----------------------------------------------------------------------
    // Closest surface point
    // -----------------------------------------------------------------------

    #[test]
    fn test_closest_surface_point() {
        let mesh = single_tri_mesh();
        // Query point directly above the triangle centroid
        let centroid = [1.0 / 3.0, 1.0 / 3.0, 1.0];
        let (pos, normal) = closest_surface_point(&mesh, centroid);

        // Closest point should be on the triangle (z=0 plane)
        assert!((pos[2] - 0.0).abs() < 0.01, "Closest point z = {}", pos[2]);

        // Normal should point roughly in +Z direction
        assert!(normal[2] > 0.5, "Normal z = {}", normal[2]);

        // x,y should be in triangle region
        assert!(pos[0] >= -0.01 && pos[0] <= 1.01);
        assert!(pos[1] >= -0.01 && pos[1] <= 1.01);
    }

    // -----------------------------------------------------------------------
    // Bake
    // -----------------------------------------------------------------------

    #[test]
    fn test_bake_normal_map_basic() {
        let low = quad_mesh();
        let high = quad_mesh();
        let params = NormalMapBakeParams {
            width: 16,
            height: 16,
            cage_offset: 0.01,
            tangent_space: false, // object-space for simplicity
            background: [0.5, 0.5, 1.0],
        };
        let result = bake_normal_map(&low, &high, &params);
        assert_eq!(result.width, 16);
        assert_eq!(result.height, 16);
        assert_eq!(result.pixels.len(), 256);
        // Should fill at least some pixels
        assert!(result.filled_pixels > 0, "No pixels were filled");
    }

    #[test]
    fn test_coverage() {
        let low = quad_mesh();
        let high = quad_mesh();
        let params = NormalMapBakeParams {
            width: 8,
            height: 8,
            cage_offset: 0.01,
            tangent_space: false,
            background: [0.5, 0.5, 1.0],
        };
        let result = bake_normal_map(&low, &high, &params);
        // Coverage should be between 0 and 1
        assert!(result.coverage >= 0.0 && result.coverage <= 1.0);
        // For a quad covering the full UV space, coverage should be significant
        assert!(
            result.coverage > 0.3,
            "Coverage too low: {}",
            result.coverage
        );
        // Verify coverage == filled_pixels / total_pixels
        let expected = result.filled_pixels as f32 / (8 * 8) as f32;
        assert!((result.coverage - expected).abs() < 1e-5);
    }

    // -----------------------------------------------------------------------
    // to_rgb_u8
    // -----------------------------------------------------------------------

    #[test]
    fn test_to_rgb_u8() {
        let mut tex = NormalMapTexture::new(2, 2, [0.5, 0.5, 1.0]);
        tex.set(0, 0, [0.0, 0.0, 0.0]);
        tex.set(1, 0, [1.0, 1.0, 1.0]);
        let u8_pixels = tex.to_rgb_u8();
        assert_eq!(u8_pixels.len(), 4);
        // Black pixel
        assert_eq!(u8_pixels[0], [0, 0, 0]);
        // White pixel
        assert_eq!(u8_pixels[1], [255, 255, 255]);
    }

    // -----------------------------------------------------------------------
    // Tangent-space bake
    // -----------------------------------------------------------------------

    #[test]
    fn test_bake_tangent_space() {
        let low = single_tri_mesh();
        let high = single_tri_mesh();
        let params = NormalMapBakeParams {
            width: 8,
            height: 8,
            cage_offset: 0.005,
            tangent_space: true,
            background: [0.5, 0.5, 1.0],
        };
        let result = bake_normal_map(&low, &high, &params);
        assert_eq!(result.width, 8);
        assert_eq!(result.height, 8);
        // In tangent space, a flat mesh should produce ~(0.5, 0.5, 1.0) normals
        // (pointing straight up in tangent space)
        for pixel in &result.pixels {
            // All pixels should be in [0, 1]
            assert!(pixel[0] >= 0.0 && pixel[0] <= 1.0);
            assert!(pixel[1] >= 0.0 && pixel[1] <= 1.0);
            assert!(pixel[2] >= 0.0 && pixel[2] <= 1.0);
        }
    }

    // -----------------------------------------------------------------------
    // lerp_normal helper (internal)
    // -----------------------------------------------------------------------

    #[test]
    fn test_lerp_normal_helper() {
        let n0 = [1.0f32, 0.0, 0.0];
        let n1 = [0.0f32, 1.0, 0.0];
        let mid = lerp_normal(n0, n1, 0.5);
        let expected_len = (mid[0] * mid[0] + mid[1] * mid[1] + mid[2] * mid[2]).sqrt();
        assert!(
            (expected_len - 1.0).abs() < 1e-5,
            "lerp_normal not normalized"
        );
    }
}
