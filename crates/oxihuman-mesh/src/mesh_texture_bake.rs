// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! GPU-less CPU texture baking pipeline.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Math helpers
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
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
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
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l > 1e-10 {
        scale3(v, 1.0 / l)
    } else {
        [0.0, 1.0, 0.0]
    }
}

// ---------------------------------------------------------------------------
// LCG random number generator
// ---------------------------------------------------------------------------

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(1),
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    fn next_f32(&mut self) -> f32 {
        let bits = self.next_u64();
        (bits >> 40) as f32 / (1u64 << 24) as f32
    }
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Texture bake target (RGBA f32).
#[derive(Debug, Clone)]
pub struct BakeTarget {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[f32; 4]>,
}

/// A ray for baking.
#[derive(Debug, Clone, Copy)]
pub struct BakeRay {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

/// Bake mode selector.
#[derive(Debug, Clone)]
pub enum BakeMode {
    AmbientOcclusion { samples: u32, max_dist: f32 },
    NormalMap,
    CurvatureMap { min_curv: f32, max_curv: f32 },
    VertexColor,
    Thickness { samples: u32, max_dist: f32 },
}

// ---------------------------------------------------------------------------
// Core baking helpers
// ---------------------------------------------------------------------------

/// Create a black RGBA bake target.
pub fn new_bake_target(width: u32, height: u32) -> BakeTarget {
    BakeTarget {
        width,
        height,
        pixels: vec![[0.0, 0.0, 0.0, 1.0]; (width * height) as usize],
    }
}

/// Möller–Trumbore ray-triangle intersection. Returns Some(t) where t >= 0.
fn ray_triangle_intersect(
    origin: [f32; 3],
    dir: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<f32> {
    let e1 = sub3(v1, v0);
    let e2 = sub3(v2, v0);
    let h = cross3(dir, e2);
    let a = dot3(e1, h);
    if a.abs() < 1e-10 {
        return None;
    }
    let f = 1.0 / a;
    let s = sub3(origin, v0);
    let u = f * dot3(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross3(s, e1);
    let v = f * dot3(dir, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * dot3(e2, q);
    if t > 1e-6 {
        Some(t)
    } else {
        None
    }
}

/// Test whether a ray hits any triangle before max_dist.
fn ray_hits_mesh(
    origin: [f32; 3],
    dir: [f32; 3],
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    max_dist: f32,
) -> bool {
    for tri in triangles {
        let v0 = positions[tri[0] as usize];
        let v1 = positions[tri[1] as usize];
        let v2 = positions[tri[2] as usize];
        if let Some(t) = ray_triangle_intersect(origin, dir, v0, v1, v2) {
            if t < max_dist {
                return true;
            }
        }
    }
    false
}

/// Hemisphere cosine-weighted random direction in world space aligned to `n`.
fn hemisphere_sample(n: [f32; 3], lcg: &mut Lcg) -> [f32; 3] {
    // Uniform cosine-weighted sample
    let r1 = lcg.next_f32();
    let r2 = lcg.next_f32();
    let sin_theta = (1.0 - r1 * r1).sqrt();
    let phi = 2.0 * std::f32::consts::PI * r2;
    let local = [sin_theta * phi.cos(), sin_theta * phi.sin(), r1];

    // Build TBN from n
    let up = if n[1].abs() < 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let t = normalize3(cross3(up, n));
    let b = cross3(n, t);

    let wx = local[0] * t[0] + local[1] * b[0] + local[2] * n[0];
    let wy = local[0] * t[1] + local[1] * b[1] + local[2] * n[1];
    let wz = local[0] * t[2] + local[1] * b[2] + local[2] * n[2];
    normalize3([wx, wy, wz])
}

/// Compute face normal from triangle vertices.
fn face_normal(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    normalize3(cross3(sub3(v1, v0), sub3(v2, v0)))
}

/// Compute face centroid.
fn face_centroid(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    [
        (v0[0] + v1[0] + v2[0]) / 3.0,
        (v0[1] + v1[1] + v2[1]) / 3.0,
        (v0[2] + v1[2] + v2[2]) / 3.0,
    ]
}

/// Rasterize a UV triangle, calling `callback` for each (px, py, bary) inside.
fn rasterize_uv_triangle<F>(
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
    width: u32,
    height: u32,
    mut callback: F,
) where
    F: FnMut(u32, u32, [f32; 3]),
{
    let w = width as f32;
    let h = height as f32;

    // Compute pixel-space bounding box
    let px0 = (uv0[0] * w).floor() as i32;
    let py0 = (uv0[1] * h).floor() as i32;
    let px1 = (uv1[0] * w).floor() as i32;
    let py1 = (uv1[1] * h).floor() as i32;
    let px2 = (uv2[0] * w).floor() as i32;
    let py2 = (uv2[1] * h).floor() as i32;

    let min_x = px0.min(px1).min(px2).max(0) as u32;
    let min_y = py0.min(py1).min(py2).max(0) as u32;
    let max_x = (px0.max(px1).max(px2) + 1).min(width as i32 - 1) as u32;
    let max_y = (py0.max(py1).max(py2) + 1).min(height as i32 - 1) as u32;

    // 2D barycentric in UV space
    let denom = (uv1[1] - uv2[1]) * (uv0[0] - uv2[0]) + (uv2[0] - uv1[0]) * (uv0[1] - uv2[1]);
    if denom.abs() < 1e-10 {
        return;
    }

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let pu = (px as f32 + 0.5) / w;
            let pv = (py as f32 + 0.5) / h;

            let l1 =
                ((uv1[1] - uv2[1]) * (pu - uv2[0]) + (uv2[0] - uv1[0]) * (pv - uv2[1])) / denom;
            let l2 =
                ((uv2[1] - uv0[1]) * (pu - uv2[0]) + (uv0[0] - uv2[0]) * (pv - uv2[1])) / denom;
            let l3 = 1.0 - l1 - l2;

            if l1 >= -1e-4 && l2 >= -1e-4 && l3 >= -1e-4 {
                callback(px, py, [l1, l2, l3]);
            }
        }
    }
}

/// Interpolate a 3D position from barycentric coordinates.
fn bary_interp3(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3], bary: [f32; 3]) -> [f32; 3] {
    [
        bary[0] * v0[0] + bary[1] * v1[0] + bary[2] * v2[0],
        bary[0] * v0[1] + bary[1] * v1[1] + bary[2] * v2[1],
        bary[0] * v0[2] + bary[1] * v1[2] + bary[2] * v2[2],
    ]
}

// ---------------------------------------------------------------------------
// Public baking functions
// ---------------------------------------------------------------------------

/// Bake ambient occlusion into a texture.
#[allow(clippy::too_many_arguments)]
pub fn bake_ambient_occlusion(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    uvs: &[[f32; 2]],
    target: &mut BakeTarget,
    samples: u32,
    max_dist: f32,
) {
    let mut lcg = Lcg::new(42);
    let w = target.width;
    let h = target.height;

    for (tri_idx, tri) in triangles.iter().enumerate() {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        if i0 >= uvs.len() || i1 >= uvs.len() || i2 >= uvs.len() {
            continue;
        }

        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];
        let uv0 = uvs[i0];
        let uv1 = uvs[i1];
        let uv2 = uvs[i2];
        let n = face_normal(p0, p1, p2);

        let pixels = &mut target.pixels;
        let _ = tri_idx; // suppress unused

        rasterize_uv_triangle(uv0, uv1, uv2, w, h, |px, py, bary| {
            let world_pos = bary_interp3(p0, p1, p2, bary);
            let offset = add3(world_pos, scale3(n, 1e-4));

            let mut unoccluded = 0u32;
            for _ in 0..samples {
                let dir = hemisphere_sample(n, &mut lcg);
                if !ray_hits_mesh(offset, dir, positions, triangles, max_dist) {
                    unoccluded += 1;
                }
            }
            let ao = if samples > 0 {
                unoccluded as f32 / samples as f32
            } else {
                1.0
            };
            let idx = (py * w + px) as usize;
            pixels[idx] = [ao, ao, ao, 1.0];
        });
    }
}

/// Bake normal map into a texture (normals encoded as RGB).
pub fn bake_normal_map(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    triangles: &[[u32; 3]],
    uvs: &[[f32; 2]],
    target: &mut BakeTarget,
) {
    let w = target.width;
    let h = target.height;

    for tri in triangles {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        if i0 >= normals.len() || i1 >= normals.len() || i2 >= normals.len() {
            continue;
        }
        if i0 >= uvs.len() || i1 >= uvs.len() || i2 >= uvs.len() {
            continue;
        }
        let _ = positions;

        let n0 = normals[i0];
        let n1 = normals[i1];
        let n2 = normals[i2];
        let uv0 = uvs[i0];
        let uv1 = uvs[i1];
        let uv2 = uvs[i2];

        let pixels = &mut target.pixels;
        rasterize_uv_triangle(uv0, uv1, uv2, w, h, |px, py, bary| {
            let n = normalize3(bary_interp3(n0, n1, n2, bary));
            let r = n[0] * 0.5 + 0.5;
            let g = n[1] * 0.5 + 0.5;
            let b = n[2] * 0.5 + 0.5;
            let idx = (py * w + px) as usize;
            pixels[idx] = [r, g, b, 1.0];
        });
    }
}

/// Bake mean curvature as grayscale.
pub fn bake_curvature(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    uvs: &[[f32; 2]],
    target: &mut BakeTarget,
) {
    // Compute per-vertex mean curvature via angle deficit
    let nv = positions.len();
    let mut angle_sum = vec![0.0f32; nv];
    let mut area_sum = vec![0.0f32; nv];

    for tri in triangles {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        if i0 >= nv || i1 >= nv || i2 >= nv {
            continue;
        }
        let v = [positions[i0], positions[i1], positions[i2]];
        let area = {
            let e1 = sub3(v[1], v[0]);
            let e2 = sub3(v[2], v[0]);
            len3(cross3(e1, e2)) * 0.5
        };
        for k in 0..3 {
            let a = v[k];
            let b = v[(k + 1) % 3];
            let c = v[(k + 2) % 3];
            let ab = normalize3(sub3(b, a));
            let ac = normalize3(sub3(c, a));
            let cos_a = dot3(ab, ac).clamp(-1.0, 1.0);
            let angle = cos_a.acos();
            let vi = tri[k] as usize;
            angle_sum[vi] += angle;
            area_sum[vi] += area / 3.0;
        }
    }

    let curv: Vec<f32> = (0..nv)
        .map(|i| {
            if area_sum[i] > 1e-10 {
                (2.0 * std::f32::consts::PI - angle_sum[i]) / area_sum[i]
            } else {
                0.0
            }
        })
        .collect();

    let max_c = curv.iter().cloned().fold(0.0f32, f32::max).max(1e-6);
    let min_c = curv.iter().cloned().fold(f32::MAX, f32::min);

    let w = target.width;
    let h = target.height;

    for tri in triangles {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        if i0 >= uvs.len() || i1 >= uvs.len() || i2 >= uvs.len() {
            continue;
        }
        let c0 = if i0 < curv.len() { curv[i0] } else { 0.0 };
        let c1 = if i1 < curv.len() { curv[i1] } else { 0.0 };
        let c2 = if i2 < curv.len() { curv[i2] } else { 0.0 };
        let uv0 = uvs[i0];
        let uv1 = uvs[i1];
        let uv2 = uvs[i2];

        let pixels = &mut target.pixels;
        rasterize_uv_triangle(uv0, uv1, uv2, w, h, |px, py, bary| {
            let c = bary[0] * c0 + bary[1] * c1 + bary[2] * c2;
            let val = if (max_c - min_c).abs() > 1e-10 {
                (c - min_c) / (max_c - min_c)
            } else {
                0.5
            };
            let idx = (py * w + px) as usize;
            pixels[idx] = [val, val, val, 1.0];
        });
    }
}

/// Bake thickness by casting inward rays.
#[allow(clippy::too_many_arguments)]
pub fn bake_thickness(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    uvs: &[[f32; 2]],
    target: &mut BakeTarget,
    samples: u32,
    max_dist: f32,
) {
    let mut lcg = Lcg::new(12345);
    let w = target.width;
    let h = target.height;

    for tri in triangles {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        if i0 >= uvs.len() || i1 >= uvs.len() || i2 >= uvs.len() {
            continue;
        }

        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];
        let uv0 = uvs[i0];
        let uv1 = uvs[i1];
        let uv2 = uvs[i2];
        let n = face_normal(p0, p1, p2);
        let neg_n = scale3(n, -1.0);

        let pixels = &mut target.pixels;
        rasterize_uv_triangle(uv0, uv1, uv2, w, h, |px, py, bary| {
            let world_pos = bary_interp3(p0, p1, p2, bary);
            let offset = add3(world_pos, scale3(neg_n, 1e-4));

            let mut total_t = 0.0f32;
            let mut hit_count = 0u32;
            for _ in 0..samples {
                let dir = hemisphere_sample(neg_n, &mut lcg);
                for hit_tri in triangles {
                    let hv0 = positions[hit_tri[0] as usize];
                    let hv1 = positions[hit_tri[1] as usize];
                    let hv2 = positions[hit_tri[2] as usize];
                    if let Some(t) = ray_triangle_intersect(offset, dir, hv0, hv1, hv2) {
                        if t < max_dist {
                            total_t += t;
                            hit_count += 1;
                            break;
                        }
                    }
                }
            }
            let thickness = if hit_count > 0 {
                (total_t / hit_count as f32) / max_dist
            } else {
                1.0
            };
            let idx = (py * w + px) as usize;
            pixels[idx] = [thickness, thickness, thickness, 1.0];
        });
    }
}

/// Dispatch baking based on mode.
#[allow(clippy::too_many_arguments)]
pub fn bake_dispatch(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    triangles: &[[u32; 3]],
    uvs: &[[f32; 2]],
    target: &mut BakeTarget,
    mode: &BakeMode,
) {
    match mode {
        BakeMode::AmbientOcclusion { samples, max_dist } => {
            bake_ambient_occlusion(positions, triangles, uvs, target, *samples, *max_dist);
        }
        BakeMode::NormalMap => {
            bake_normal_map(positions, normals, triangles, uvs, target);
        }
        BakeMode::CurvatureMap { .. } => {
            bake_curvature(positions, triangles, uvs, target);
        }
        BakeMode::VertexColor => {
            // Fill with white (no vertex colors in this interface)
            for px in target.pixels.iter_mut() {
                *px = [1.0, 1.0, 1.0, 1.0];
            }
        }
        BakeMode::Thickness { samples, max_dist } => {
            bake_thickness(positions, triangles, uvs, target, *samples, *max_dist);
        }
    }
}

/// Encode bake target as PPM bytes.
pub fn save_bake_ppm(target: &BakeTarget) -> Vec<u8> {
    let header = format!("P6\n{} {}\n255\n", target.width, target.height);
    let mut out = header.into_bytes();
    out.reserve((target.width * target.height * 3) as usize);
    for px in &target.pixels {
        out.push((px[0].clamp(0.0, 1.0) * 255.0).round() as u8);
        out.push((px[1].clamp(0.0, 1.0) * 255.0).round() as u8);
        out.push((px[2].clamp(0.0, 1.0) * 255.0).round() as u8);
    }
    out
}

/// Load bake target from flat f32 slice (RGBA interleaved).
pub fn load_bake_from_f32_slice(width: u32, height: u32, data: &[f32]) -> BakeTarget {
    let n = (width * height) as usize;
    let mut pixels = Vec::with_capacity(n);
    for i in 0..n {
        let base = i * 4;
        if base + 3 < data.len() {
            pixels.push([data[base], data[base + 1], data[base + 2], data[base + 3]]);
        } else {
            pixels.push([0.0, 0.0, 0.0, 1.0]);
        }
    }
    BakeTarget {
        width,
        height,
        pixels,
    }
}

/// Bilinear sample at (u, v) in [0, 1].
pub fn sample_bake_at_uv(target: &BakeTarget, u: f32, v: f32) -> [f32; 4] {
    let w = target.width as f32;
    let h = target.height as f32;
    let fx = (u * w - 0.5).max(0.0);
    let fy = (v * h - 0.5).max(0.0);
    let x0 = fx.floor() as u32;
    let y0 = fy.floor() as u32;
    let x1 = (x0 + 1).min(target.width - 1);
    let y1 = (y0 + 1).min(target.height - 1);
    let tx = fx - fx.floor();
    let ty = fy - fy.floor();

    let p00 = target.pixels[(y0 * target.width + x0) as usize];
    let p10 = target.pixels[(y0 * target.width + x1) as usize];
    let p01 = target.pixels[(y1 * target.width + x0) as usize];
    let p11 = target.pixels[(y1 * target.width + x1) as usize];

    let mut out = [0.0f32; 4];
    for c in 0..4 {
        let top = p00[c] * (1.0 - tx) + p10[c] * tx;
        let bot = p01[c] * (1.0 - tx) + p11[c] * tx;
        out[c] = top * (1.0 - ty) + bot * ty;
    }
    out
}

/// Convert bake target to u8 Vec (RGBA, clamped).
pub fn bake_target_to_u8(target: &BakeTarget) -> Vec<u8> {
    let mut out = Vec::with_capacity(target.pixels.len() * 4);
    for px in &target.pixels {
        out.push((px[0].clamp(0.0, 1.0) * 255.0).round() as u8);
        out.push((px[1].clamp(0.0, 1.0) * 255.0).round() as u8);
        out.push((px[2].clamp(0.0, 1.0) * 255.0).round() as u8);
        out.push((px[3].clamp(0.0, 1.0) * 255.0).round() as u8);
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    type QuadMesh = (Vec<[f32; 3]>, Vec<[u32; 3]>, Vec<[f32; 2]>, Vec<[f32; 3]>);

    fn simple_quad_mesh() -> QuadMesh {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let triangles = vec![[0u32, 1, 2], [0, 2, 3]];
        let uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let normals = vec![
            [0.0f32, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        (positions, triangles, uvs, normals)
    }

    #[test]
    fn test_new_bake_target_size() {
        let t = new_bake_target(16, 8);
        assert_eq!(t.width, 16);
        assert_eq!(t.height, 8);
        assert_eq!(t.pixels.len(), 128);
    }

    #[test]
    fn test_new_bake_target_initial_alpha() {
        let t = new_bake_target(4, 4);
        for px in &t.pixels {
            assert_eq!(px[3], 1.0);
        }
    }

    #[test]
    fn test_bake_normal_map_produces_output() {
        let (pos, tris, uvs, norms) = simple_quad_mesh();
        let mut target = new_bake_target(8, 8);
        bake_normal_map(&pos, &norms, &tris, &uvs, &mut target);
        // At least some pixels should differ from default
        let non_zero = target.pixels.iter().filter(|p| p[2] > 0.5).count();
        assert!(non_zero > 0, "Normal map should produce some output");
    }

    #[test]
    fn test_bake_normal_map_z_channel() {
        let (pos, tris, uvs, norms) = simple_quad_mesh();
        let mut target = new_bake_target(8, 8);
        bake_normal_map(&pos, &norms, &tris, &uvs, &mut target);
        // Since normals are all [0,0,1], blue channel should be ~1.0
        for px in &target.pixels {
            if px[3] > 0.5 {
                // Only check pixels that were written
                assert!(px[2] > 0.4, "Z normal should encode to ~1.0 in blue");
            }
        }
    }

    #[test]
    fn test_bake_ao_produces_values() {
        let (pos, tris, uvs, _) = simple_quad_mesh();
        let mut target = new_bake_target(8, 8);
        bake_ambient_occlusion(&pos, &tris, &uvs, &mut target, 4, 10.0);
        // Some pixels should be written
        let written = target.pixels.iter().filter(|p| p[0] > 0.0).count();
        assert!(written > 0);
    }

    #[test]
    fn test_bake_curvature_runs() {
        let (pos, tris, uvs, _) = simple_quad_mesh();
        let mut target = new_bake_target(8, 8);
        bake_curvature(&pos, &tris, &uvs, &mut target);
        // Should not panic; just verify size is intact
        assert_eq!(target.pixels.len(), 64);
    }

    #[test]
    fn test_bake_thickness_runs() {
        let (pos, tris, uvs, _) = simple_quad_mesh();
        let mut target = new_bake_target(8, 8);
        bake_thickness(&pos, &tris, &uvs, &mut target, 4, 5.0);
        assert_eq!(target.pixels.len(), 64);
    }

    #[test]
    fn test_bake_dispatch_normal() {
        let (pos, tris, uvs, norms) = simple_quad_mesh();
        let mut target = new_bake_target(8, 8);
        bake_dispatch(&pos, &norms, &tris, &uvs, &mut target, &BakeMode::NormalMap);
        let non_default = target
            .pixels
            .iter()
            .filter(|p| p[0] != 0.0 || p[1] != 0.0)
            .count();
        assert!(non_default > 0);
    }

    #[test]
    fn test_bake_dispatch_ao() {
        let (pos, tris, uvs, norms) = simple_quad_mesh();
        let mut target = new_bake_target(8, 8);
        bake_dispatch(
            &pos,
            &norms,
            &tris,
            &uvs,
            &mut target,
            &BakeMode::AmbientOcclusion {
                samples: 4,
                max_dist: 10.0,
            },
        );
        assert_eq!(target.pixels.len(), 64);
    }

    #[test]
    fn test_save_bake_ppm_header() {
        let t = new_bake_target(4, 4);
        let bytes = save_bake_ppm(&t);
        let header = std::str::from_utf8(&bytes[..10]).unwrap_or("");
        assert!(header.starts_with("P6"));
    }

    #[test]
    fn test_save_bake_ppm_length() {
        let t = new_bake_target(4, 4);
        let bytes = save_bake_ppm(&t);
        // header + 4*4*3 = 48 pixel bytes
        assert!(bytes.len() > 48);
    }

    #[test]
    fn test_load_bake_from_f32_slice() {
        let data: Vec<f32> = (0..16).map(|i| i as f32 / 16.0).collect();
        let t = load_bake_from_f32_slice(2, 2, &data);
        assert_eq!(t.width, 2);
        assert_eq!(t.height, 2);
        assert_eq!(t.pixels.len(), 4);
        assert!((t.pixels[0][0] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_sample_bake_at_uv_center() {
        let mut t = new_bake_target(4, 4);
        // Fill all pixels with red
        for px in t.pixels.iter_mut() {
            *px = [1.0, 0.0, 0.0, 1.0];
        }
        let s = sample_bake_at_uv(&t, 0.5, 0.5);
        assert!(s[0] > 0.5);
    }

    #[test]
    fn test_bake_target_to_u8_length() {
        let t = new_bake_target(4, 4);
        let bytes = bake_target_to_u8(&t);
        assert_eq!(bytes.len(), 4 * 4 * 4);
    }

    #[test]
    fn test_bake_target_to_u8_clamp() {
        let mut t = new_bake_target(2, 2);
        t.pixels[0] = [2.0, -1.0, 0.5, 1.0];
        let bytes = bake_target_to_u8(&t);
        assert_eq!(bytes[0], 255); // clamped to 1.0
        assert_eq!(bytes[1], 0); // clamped to 0.0
        assert_eq!(bytes[2], 128); // 0.5 * 255 ≈ 128
    }

    #[test]
    fn test_bake_dispatch_vertex_color() {
        let (pos, tris, uvs, norms) = simple_quad_mesh();
        let mut target = new_bake_target(4, 4);
        bake_dispatch(
            &pos,
            &norms,
            &tris,
            &uvs,
            &mut target,
            &BakeMode::VertexColor,
        );
        // All pixels should be white
        for px in &target.pixels {
            assert_eq!(px[0], 1.0);
        }
    }

    #[test]
    fn test_ray_triangle_intersect_hit() {
        let origin = [0.0f32, 0.0, 1.0];
        let dir = [0.0f32, 0.0, -1.0];
        let v0 = [-1.0f32, -1.0, 0.0];
        let v1 = [1.0f32, -1.0, 0.0];
        let v2 = [0.0f32, 1.0, 0.0];
        let result = ray_triangle_intersect(origin, dir, v0, v1, v2);
        assert!(result.is_some());
        assert!((result.expect("should succeed") - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_ray_triangle_intersect_miss() {
        let origin = [5.0f32, 5.0, 1.0];
        let dir = [0.0f32, 0.0, -1.0];
        let v0 = [-1.0f32, -1.0, 0.0];
        let v1 = [1.0f32, -1.0, 0.0];
        let v2 = [0.0f32, 1.0, 0.0];
        let result = ray_triangle_intersect(origin, dir, v0, v1, v2);
        assert!(result.is_none());
    }
}
