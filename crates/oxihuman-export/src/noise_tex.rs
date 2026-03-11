// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Procedural noise texture generation: value noise, fractal Brownian motion,
//! wood, marble, and Voronoi-like textures. All math done from scratch — no external crates.

use crate::texture::PixelBuffer;

/// Simple integer hash function for value noise (no external deps).
/// Returns a pseudo-random f32 in [0, 1].
fn hash2d(ix: i32, iy: i32) -> f32 {
    let n = ix.wrapping_mul(1619).wrapping_add(iy.wrapping_mul(31337));
    let n = n
        .wrapping_mul(n)
        .wrapping_mul(60493)
        .wrapping_add(n.wrapping_mul(19990303));
    (n.unsigned_abs() as f32) / (u32::MAX as f32)
}

/// Smooth step (Hermite) function: 3t² − 2t³.
///
/// Maps [0, 1] to [0, 1] with zero first derivatives at endpoints.
#[allow(dead_code)]
pub fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Smoother step: 6t⁵ − 15t⁴ + 10t³.
///
/// Maps [0, 1] to [0, 1] with zero first and second derivatives at endpoints.
#[allow(dead_code)]
pub fn smootherstep(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Value noise at (x, y) using bilinear interpolation of hashed grid points.
///
/// Returns a value in [0, 1].
#[allow(dead_code)]
pub fn value_noise(x: f32, y: f32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - x.floor();
    let fy = y - y.floor();

    let ux = smoothstep(fx);
    let uy = smoothstep(fy);

    let v00 = hash2d(ix, iy);
    let v10 = hash2d(ix + 1, iy);
    let v01 = hash2d(ix, iy + 1);
    let v11 = hash2d(ix + 1, iy + 1);

    let bottom = v00 + ux * (v10 - v00);
    let top = v01 + ux * (v11 - v01);
    bottom + uy * (top - bottom)
}

/// Fractal Brownian Motion (fBm): sum of value_noise at increasing frequencies.
///
/// - `octaves`: number of octaves (e.g. 4–8)
/// - `lacunarity`: frequency multiplier per octave (e.g. 2.0)
/// - `gain`: amplitude multiplier per octave (e.g. 0.5)
///
/// Returns a value approximately in [0, 1] (clamped to [0, 1]).
#[allow(dead_code)]
pub fn fbm(x: f32, y: f32, octaves: u32, lacunarity: f32, gain: f32) -> f32 {
    let mut value = 0.0f32;
    let mut amplitude = 0.5f32;
    let mut frequency = 1.0f32;

    // Normalization factor: sum of amplitudes
    let mut max_value = 0.0f32;

    for _ in 0..octaves {
        value += amplitude * value_noise(x * frequency, y * frequency);
        max_value += amplitude;
        amplitude *= gain;
        frequency *= lacunarity;
    }

    if max_value > 0.0 {
        (value / max_value).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Generate a noise texture as a [`PixelBuffer`] (grayscale → RGBA).
///
/// Each pixel's RGB = noise_value × 255, A = 255.
/// `scale` controls how many noise "tiles" span the texture.
#[allow(dead_code)]
pub fn generate_noise_texture(width: u32, height: u32, scale: f32) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height);
    let scale = scale.max(f32::EPSILON);
    for y in 0..height {
        for x in 0..width {
            let nx = x as f32 / width as f32 * scale;
            let ny = y as f32 / height as f32 * scale;
            let v = value_noise(nx, ny);
            let c = (v * 255.0).round() as u8;
            buf.set_pixel(x, y, c, c, c, 255);
        }
    }
    buf
}

/// Generate a fractal noise texture using fBm.
///
/// `scale` controls the base frequency; `octaves` sets the fBm octave count.
#[allow(dead_code)]
pub fn generate_fbm_texture(width: u32, height: u32, scale: f32, octaves: u32) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height);
    let scale = scale.max(f32::EPSILON);
    for y in 0..height {
        for x in 0..width {
            let nx = x as f32 / width as f32 * scale;
            let ny = y as f32 / height as f32 * scale;
            let v = fbm(nx, ny, octaves, 2.0, 0.5);
            let c = (v * 255.0).round() as u8;
            buf.set_pixel(x, y, c, c, c, 255);
        }
    }
    buf
}

/// Linearly interpolate between two u8 colors by factor t in [0, 1].
#[inline]
fn lerp_color(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    [
        (a[0] as f32 + t * (b[0] as f32 - a[0] as f32)).round() as u8,
        (a[1] as f32 + t * (b[1] as f32 - a[1] as f32)).round() as u8,
        (a[2] as f32 + t * (b[2] as f32 - a[2] as f32)).round() as u8,
    ]
}

/// Generate a wood-ring texture using noise + sine waves.
///
/// - `rings`: number of annual rings across the texture
/// - `noise_scale`: amount of distortion applied by value noise
/// - `color_a`: light wood color
/// - `color_b`: dark wood color
#[allow(dead_code)]
pub fn generate_wood_texture(
    width: u32,
    height: u32,
    rings: f32,
    noise_scale: f32,
    color_a: [u8; 3],
    color_b: [u8; 3],
) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height);
    let rings = rings.max(1.0);
    let noise_scale = noise_scale.max(f32::EPSILON);
    for y in 0..height {
        for x in 0..width {
            let nx = x as f32 / width as f32;
            let ny = y as f32 / height as f32;

            // Radial distance from centre, perturbed by noise
            let cx = nx - 0.5;
            let cy = ny - 0.5;
            let dist = (cx * cx + cy * cy).sqrt();
            let perturbation = value_noise(nx * noise_scale, ny * noise_scale) * 0.15;
            let pattern = ((dist + perturbation) * rings * std::f32::consts::PI * 2.0).sin();
            // Remap [-1, 1] → [0, 1]
            let t = (pattern + 1.0) * 0.5;
            let [r, g, b] = lerp_color(color_a, color_b, t);
            buf.set_pixel(x, y, r, g, b, 255);
        }
    }
    buf
}

/// Generate a marble texture using fBm + sine pattern.
///
/// - `scale`: overall texture frequency
/// - `color_a` / `color_b`: the two marble vein colors
#[allow(dead_code)]
pub fn generate_marble_texture(
    width: u32,
    height: u32,
    scale: f32,
    color_a: [u8; 3],
    color_b: [u8; 3],
) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height);
    let scale = scale.max(f32::EPSILON);
    for y in 0..height {
        for x in 0..width {
            let nx = x as f32 / width as f32 * scale;
            let ny = y as f32 / height as f32 * scale;
            let turbulence = fbm(nx, ny, 6, 2.0, 0.5);
            // Sine function modulated by turbulence creates vein pattern
            let pattern = ((nx + turbulence * 4.0) * std::f32::consts::PI).sin();
            let t = (pattern + 1.0) * 0.5;
            let [r, g, b] = lerp_color(color_a, color_b, t);
            buf.set_pixel(x, y, r, g, b, 255);
        }
    }
    buf
}

/// Generate a cellular / Voronoi-like texture.
///
/// A grid of random seed points is created; each pixel is coloured by its
/// distance to the nearest seed, blending from `color_a` (nearest) to
/// `color_b` (farthest within the cell).
#[allow(dead_code)]
pub fn generate_voronoi_texture(
    width: u32,
    height: u32,
    cell_size: f32,
    color_a: [u8; 3],
    color_b: [u8; 3],
) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height);
    let cell_size = cell_size.max(1.0);

    for y in 0..height {
        for x in 0..width {
            let px = x as f32;
            let py = y as f32;

            // Cell coordinates
            let cx = (px / cell_size).floor() as i32;
            let cy = (py / cell_size).floor() as i32;

            let mut min_dist = f32::MAX;

            // Search 3×3 neighbourhood of cells
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let nx = cx + dx;
                    let ny = cy + dy;
                    // Random seed point inside the neighbouring cell
                    let seed_x = (nx as f32 + hash2d(nx, ny * 7 + 1)) * cell_size;
                    let seed_y = (ny as f32 + hash2d(nx * 3 + 2, ny)) * cell_size;
                    let ddx = px - seed_x;
                    let ddy = py - seed_y;
                    let dist = (ddx * ddx + ddy * ddy).sqrt();
                    if dist < min_dist {
                        min_dist = dist;
                    }
                }
            }

            // Normalise distance to approximately [0, 1] within the cell
            let t = (min_dist / cell_size).clamp(0.0, 1.0);
            let [r, g, b] = lerp_color(color_a, color_b, t);
            buf.set_pixel(x, y, r, g, b, 255);
        }
    }
    buf
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_noise_in_range_0_1() {
        for i in 0..100 {
            let v = value_noise(i as f32 * 0.13, i as f32 * 0.07);
            assert!(
                (0.0..=1.0).contains(&v),
                "value_noise out of range: {v} at i={i}"
            );
        }
    }

    #[test]
    fn value_noise_continuity() {
        let a = value_noise(1.0, 1.0);
        let b = value_noise(1.0 + 1e-4, 1.0 + 1e-4);
        assert!(
            (a - b).abs() < 0.05,
            "value_noise not continuous: {a} vs {b}"
        );
    }

    #[test]
    fn smoothstep_zero_returns_zero() {
        assert_eq!(smoothstep(0.0), 0.0);
    }

    #[test]
    fn smoothstep_one_returns_one() {
        assert!((smoothstep(1.0) - 1.0).abs() < f32::EPSILON * 4.0);
    }

    #[test]
    fn smootherstep_midpoint_is_half() {
        assert!((smootherstep(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn fbm_in_range_0_1() {
        for i in 0..50 {
            let v = fbm(i as f32 * 0.17, i as f32 * 0.23, 4, 2.0, 0.5);
            assert!((0.0..=1.0).contains(&v), "fbm out of range: {v} at i={i}");
        }
    }

    #[test]
    fn generate_noise_texture_correct_size() {
        let buf = generate_noise_texture(32, 32, 4.0);
        assert_eq!(buf.width, 32);
        assert_eq!(buf.height, 32);
        assert_eq!(buf.byte_len(), 32 * 32 * 4);
    }

    #[test]
    fn generate_noise_texture_all_pixels_valid() {
        let buf = generate_noise_texture(16, 16, 4.0);
        for chunk in buf.pixels.chunks_exact(4) {
            assert_eq!(chunk[3], 255, "alpha must be 255");
            // R == G == B for grayscale
            assert_eq!(chunk[0], chunk[1]);
            assert_eq!(chunk[1], chunk[2]);
        }
    }

    #[test]
    fn generate_fbm_texture_not_uniform() {
        let buf = generate_fbm_texture(32, 32, 4.0, 4);
        let first = buf.get_pixel(0, 0)[0];
        let all_same = buf.pixels.chunks_exact(4).all(|c| c[0] == first);
        assert!(!all_same, "fbm texture should not be uniform");
    }

    #[test]
    fn generate_wood_texture_size_correct() {
        let buf = generate_wood_texture(64, 64, 8.0, 4.0, [210, 180, 140], [139, 90, 43]);
        assert_eq!(buf.width, 64);
        assert_eq!(buf.height, 64);
        assert_eq!(buf.byte_len(), 64 * 64 * 4);
    }

    #[test]
    fn generate_marble_texture_size_correct() {
        let buf = generate_marble_texture(64, 64, 4.0, [240, 240, 240], [80, 80, 80]);
        assert_eq!(buf.width, 64);
        assert_eq!(buf.height, 64);
    }

    #[test]
    fn generate_voronoi_texture_not_all_same_color() {
        let buf = generate_voronoi_texture(64, 64, 16.0, [255, 255, 255], [0, 0, 0]);
        let first = buf.get_pixel(0, 0)[0];
        let all_same = buf.pixels.chunks_exact(4).all(|c| c[0] == first);
        assert!(!all_same, "voronoi texture should not be uniform");
    }

    #[test]
    fn noise_texture_alpha_is_255() {
        let buf = generate_noise_texture(16, 16, 4.0);
        for chunk in buf.pixels.chunks_exact(4) {
            assert_eq!(chunk[3], 255);
        }
    }
}
