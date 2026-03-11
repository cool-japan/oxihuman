// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Displacement map generation and export for sculpted detail.

/// Displacement mode selection.
#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum DisplacementMode {
    /// Scalar (single-channel height) displacement.
    Scalar,
    /// Vector (3-channel XYZ offset) displacement.
    Vector,
}

/// Configuration for displacement map generation.
#[allow(dead_code)]
pub struct DisplacementConfig {
    pub width: u32,
    pub height: u32,
    pub mode: DisplacementMode,
    pub scale: f32,
}

/// Pixel buffer storing displacement values.
///
/// For [`DisplacementMode::Scalar`] each pixel is one `f32`.
/// For [`DisplacementMode::Vector`] each pixel is three consecutive `f32`s (x, y, z).
#[allow(dead_code)]
pub struct DisplacementBuffer {
    pub pixels: Vec<f32>,
    pub width: u32,
    pub height: u32,
    pub mode: DisplacementMode,
}

/// Type alias for displacement statistics.
#[allow(dead_code)]
pub type DisplacementStatsResult = (f32, f32, f32);

// ── Public API ────────────────────────────────────────────────────────────────

/// Create a default displacement configuration with `Scalar` mode.
#[allow(dead_code)]
pub fn default_displacement_config(width: u32, height: u32) -> DisplacementConfig {
    DisplacementConfig {
        width,
        height,
        mode: DisplacementMode::Scalar,
        scale: 1.0,
    }
}

/// Allocate a new displacement buffer filled with zeros.
#[allow(dead_code)]
pub fn new_displacement_buffer(
    width: u32,
    height: u32,
    mode: DisplacementMode,
) -> DisplacementBuffer {
    let channels = channels_for_mode(&mode);
    let count = (width as usize) * (height as usize) * channels;
    DisplacementBuffer {
        pixels: vec![0.0; count],
        width,
        height,
        mode,
    }
}

/// Set a displacement value at `(x, y)`.
/// For `Scalar` mode, supply `[value, 0, 0]`; only the first element is used.
/// For `Vector` mode all three components are stored.
#[allow(dead_code)]
pub fn set_displacement(buf: &mut DisplacementBuffer, x: u32, y: u32, value: [f32; 3]) {
    if x >= buf.width || y >= buf.height {
        return;
    }
    let channels = channels_for_mode(&buf.mode);
    let base = ((y * buf.width + x) as usize) * channels;
    buf.pixels[base..(base + channels)].copy_from_slice(&value[..channels]);
}

/// Get the displacement value at `(x, y)`.
/// Returns `[0.0; 3]` if coordinates are out of bounds.
#[allow(dead_code)]
pub fn get_displacement(buf: &DisplacementBuffer, x: u32, y: u32) -> [f32; 3] {
    if x >= buf.width || y >= buf.height {
        return [0.0; 3];
    }
    let channels = channels_for_mode(&buf.mode);
    let base = ((y * buf.width + x) as usize) * channels;
    let mut out = [0.0_f32; 3];
    out[..channels].copy_from_slice(&buf.pixels[base..(base + channels)]);
    out
}

/// Compute per-vertex displacement from a base mesh and a sculpted mesh.
/// Both slices must have equal length; each `[f32; 3]` is a position.
/// Returns a vector of displacement vectors `sculpted[i] - base[i]`.
#[allow(dead_code)]
pub fn compute_displacement_from_meshes(base: &[[f32; 3]], sculpted: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let len = base.len().min(sculpted.len());
    (0..len)
        .map(|i| {
            [
                sculpted[i][0] - base[i][0],
                sculpted[i][1] - base[i][1],
                sculpted[i][2] - base[i][2],
            ]
        })
        .collect()
}

/// Encode a scalar displacement buffer as a 16-bit grayscale PPM (P5, maxval 65535).
#[allow(dead_code)]
pub fn encode_displacement_ppm(buf: &DisplacementBuffer) -> Vec<u8> {
    let w = buf.width;
    let h = buf.height;
    let header = format!("P5\n{w} {h}\n65535\n");
    let pixel_count = (w as usize) * (h as usize);
    let mut out = Vec::with_capacity(header.len() + pixel_count * 2);
    out.extend_from_slice(header.as_bytes());
    let channels = channels_for_mode(&buf.mode);
    for i in 0..pixel_count {
        let v = buf.pixels[i * channels].clamp(0.0, 1.0);
        let u16_val = (v * 65535.0) as u16;
        out.push((u16_val >> 8) as u8);
        out.push((u16_val & 0xFF) as u8);
    }
    out
}

/// Compute the magnitude of each displacement vector and return as a flat `f32` slice.
#[allow(dead_code)]
pub fn displacement_magnitude_map(buf: &DisplacementBuffer) -> Vec<f32> {
    let pixel_count = (buf.width as usize) * (buf.height as usize);
    let channels = channels_for_mode(&buf.mode);
    (0..pixel_count)
        .map(|i| {
            let base = i * channels;
            if channels == 1 {
                buf.pixels[base].abs()
            } else {
                let x = buf.pixels[base];
                let y = buf.pixels[base + 1];
                let z = buf.pixels[base + 2];
                (x * x + y * y + z * z).sqrt()
            }
        })
        .collect()
}

/// Linearly remap all displacement values from `[old_min, old_max]` to `[new_min, new_max]`.
#[allow(dead_code)]
pub fn remap_displacement_range(
    buf: &mut DisplacementBuffer,
    old_min: f32,
    old_max: f32,
    new_min: f32,
    new_max: f32,
) {
    let range = old_max - old_min;
    if range.abs() < 1e-12 {
        return;
    }
    let inv = 1.0 / range;
    for v in &mut buf.pixels {
        let t = (*v - old_min) * inv;
        *v = new_min + t * (new_max - new_min);
    }
}

/// Return `(min, max, avg)` over all channels of the displacement buffer.
#[allow(dead_code)]
pub fn displacement_stats(buf: &DisplacementBuffer) -> DisplacementStatsResult {
    if buf.pixels.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    let mut sum = 0.0_f64;
    for &v in &buf.pixels {
        if v < min {
            min = v;
        }
        if v > max {
            max = v;
        }
        sum += v as f64;
    }
    let avg = (sum / buf.pixels.len() as f64) as f32;
    (min, max, avg)
}

/// Export displacement values as CSV (one row per pixel).
#[allow(dead_code)]
pub fn displacement_to_csv(buf: &DisplacementBuffer) -> String {
    let channels = channels_for_mode(&buf.mode);
    let pixel_count = (buf.width as usize) * (buf.height as usize);
    let mut out = String::new();
    if channels == 1 {
        out.push_str("x,y,value\n");
    } else {
        out.push_str("x,y,dx,dy,dz\n");
    }
    for i in 0..pixel_count {
        let px = i % (buf.width as usize);
        let py = i / (buf.width as usize);
        let base = i * channels;
        if channels == 1 {
            out.push_str(&format!("{},{},{:.6}\n", px, py, buf.pixels[base]));
        } else {
            out.push_str(&format!(
                "{},{},{:.6},{:.6},{:.6}\n",
                px,
                py,
                buf.pixels[base],
                buf.pixels[base + 1],
                buf.pixels[base + 2],
            ));
        }
    }
    out
}

/// Apply a 3x3 box-blur smoothing pass to the displacement buffer.
#[allow(dead_code)]
pub fn smooth_displacement(buf: &mut DisplacementBuffer) {
    let w = buf.width as usize;
    let h = buf.height as usize;
    let channels = channels_for_mode(&buf.mode);
    let old = buf.pixels.clone();
    for y in 0..h {
        for x in 0..w {
            let mut sums = [0.0_f32; 3];
            let mut count = 0u32;
            for dy in 0..3usize {
                for dx in 0..3usize {
                    let ny = (y + dy).wrapping_sub(1);
                    let nx = (x + dx).wrapping_sub(1);
                    if nx < w && ny < h {
                        let base = (ny * w + nx) * channels;
                        for c in 0..channels {
                            sums[c] += old[base + c];
                        }
                        count += 1;
                    }
                }
            }
            let base = (y * w + x) * channels;
            let inv = 1.0 / count as f32;
            for (dst, &s) in buf.pixels[base..base + channels]
                .iter_mut()
                .zip(sums[..channels].iter())
            {
                *dst = s * inv;
            }
        }
    }
}

/// Return the total number of pixels in the displacement buffer.
#[allow(dead_code)]
pub fn displacement_pixel_count(buf: &DisplacementBuffer) -> usize {
    (buf.width as usize) * (buf.height as usize)
}

/// Negate all displacement values (flip direction).
#[allow(dead_code)]
pub fn invert_displacement(buf: &mut DisplacementBuffer) {
    for v in &mut buf.pixels {
        *v = -*v;
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn channels_for_mode(mode: &DisplacementMode) -> usize {
    match mode {
        DisplacementMode::Scalar => 1,
        DisplacementMode::Vector => 3,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_scalar() {
        let cfg = default_displacement_config(64, 64);
        assert_eq!(cfg.mode, DisplacementMode::Scalar);
        assert_eq!(cfg.width, 64);
        assert_eq!(cfg.height, 64);
        assert!((cfg.scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_buffer_scalar_size() {
        let buf = new_displacement_buffer(4, 4, DisplacementMode::Scalar);
        assert_eq!(buf.pixels.len(), 16);
    }

    #[test]
    fn new_buffer_vector_size() {
        let buf = new_displacement_buffer(4, 4, DisplacementMode::Vector);
        assert_eq!(buf.pixels.len(), 48);
    }

    #[test]
    fn set_get_scalar() {
        let mut buf = new_displacement_buffer(4, 4, DisplacementMode::Scalar);
        set_displacement(&mut buf, 1, 2, [0.5, 0.0, 0.0]);
        let v = get_displacement(&buf, 1, 2);
        assert!((v[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn set_get_vector() {
        let mut buf = new_displacement_buffer(4, 4, DisplacementMode::Vector);
        set_displacement(&mut buf, 0, 0, [0.1, 0.2, 0.3]);
        let v = get_displacement(&buf, 0, 0);
        assert!((v[0] - 0.1).abs() < 1e-6);
        assert!((v[1] - 0.2).abs() < 1e-6);
        assert!((v[2] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn get_out_of_bounds_returns_zero() {
        let buf = new_displacement_buffer(2, 2, DisplacementMode::Scalar);
        assert_eq!(get_displacement(&buf, 5, 5), [0.0; 3]);
    }

    #[test]
    fn compute_displacement_from_meshes_basic() {
        let base = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let sculpt = vec![[0.1, 0.2, 0.3], [1.0, 0.5, 0.0]];
        let d = compute_displacement_from_meshes(&base, &sculpt);
        assert_eq!(d.len(), 2);
        assert!((d[0][0] - 0.1).abs() < 1e-6);
        assert!((d[0][1] - 0.2).abs() < 1e-6);
        assert!((d[1][1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn encode_ppm_header() {
        let buf = new_displacement_buffer(2, 2, DisplacementMode::Scalar);
        let ppm = encode_displacement_ppm(&buf);
        let header = "P5\n2 2\n65535\n";
        assert!(ppm.starts_with(header.as_bytes()));
        // 4 pixels * 2 bytes each = 8 data bytes
        assert_eq!(ppm.len(), header.len() + 8);
    }

    #[test]
    fn magnitude_map_scalar() {
        let mut buf = new_displacement_buffer(2, 1, DisplacementMode::Scalar);
        buf.pixels[0] = -0.5;
        buf.pixels[1] = 0.3;
        let mag = displacement_magnitude_map(&buf);
        assert!((mag[0] - 0.5).abs() < 1e-6);
        assert!((mag[1] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn magnitude_map_vector() {
        let mut buf = new_displacement_buffer(1, 1, DisplacementMode::Vector);
        buf.pixels[0] = 3.0;
        buf.pixels[1] = 4.0;
        buf.pixels[2] = 0.0;
        let mag = displacement_magnitude_map(&buf);
        assert!((mag[0] - 5.0).abs() < 1e-4);
    }

    #[test]
    fn remap_range() {
        let mut buf = new_displacement_buffer(2, 1, DisplacementMode::Scalar);
        buf.pixels[0] = 0.0;
        buf.pixels[1] = 1.0;
        remap_displacement_range(&mut buf, 0.0, 1.0, -1.0, 1.0);
        assert!((buf.pixels[0] - (-1.0)).abs() < 1e-6);
        assert!((buf.pixels[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn stats_basic() {
        let mut buf = new_displacement_buffer(3, 1, DisplacementMode::Scalar);
        buf.pixels[0] = -1.0;
        buf.pixels[1] = 0.0;
        buf.pixels[2] = 2.0;
        let (mn, mx, avg) = displacement_stats(&buf);
        assert!((mn - (-1.0)).abs() < 1e-6);
        assert!((mx - 2.0).abs() < 1e-6);
        assert!((avg - (1.0 / 3.0)).abs() < 1e-4);
    }

    #[test]
    fn csv_scalar_format() {
        let mut buf = new_displacement_buffer(2, 1, DisplacementMode::Scalar);
        buf.pixels[0] = 0.5;
        buf.pixels[1] = 1.0;
        let csv = displacement_to_csv(&buf);
        assert!(csv.starts_with("x,y,value\n"));
        assert!(csv.contains("0,0,0.500000"));
    }

    #[test]
    fn smooth_does_not_crash() {
        let mut buf = new_displacement_buffer(4, 4, DisplacementMode::Scalar);
        set_displacement(&mut buf, 2, 2, [1.0, 0.0, 0.0]);
        smooth_displacement(&mut buf);
        // Center pixel should have been spread
        let v = get_displacement(&buf, 2, 2);
        assert!(v[0] < 1.0);
    }

    #[test]
    fn pixel_count() {
        let buf = new_displacement_buffer(8, 4, DisplacementMode::Vector);
        assert_eq!(displacement_pixel_count(&buf), 32);
    }

    #[test]
    fn invert_displacement_basic() {
        let mut buf = new_displacement_buffer(2, 1, DisplacementMode::Scalar);
        buf.pixels[0] = 0.5;
        buf.pixels[1] = -0.3;
        invert_displacement(&mut buf);
        assert!((buf.pixels[0] - (-0.5)).abs() < 1e-6);
        assert!((buf.pixels[1] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn stats_empty_buffer() {
        let buf = DisplacementBuffer {
            pixels: vec![],
            width: 0,
            height: 0,
            mode: DisplacementMode::Scalar,
        };
        let (mn, mx, avg) = displacement_stats(&buf);
        assert!((mn).abs() < 1e-6);
        assert!((mx).abs() < 1e-6);
        assert!((avg).abs() < 1e-6);
    }
}
