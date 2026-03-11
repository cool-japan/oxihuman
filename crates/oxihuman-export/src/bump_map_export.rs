// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bump/displacement map generation and export.

/// Bump map mode selection.
#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum BumpMapMode {
    HeightMap,
    DisplacementMap,
}

/// Configuration for bump map generation.
#[allow(dead_code)]
pub struct BumpMapConfig {
    pub width: u32,
    pub height: u32,
    pub mode: BumpMapMode,
    pub scale: f32,
}

/// Pixel buffer storing bump/height values in [0, 1].
#[allow(dead_code)]
pub struct BumpMapBuffer {
    pub pixels: Vec<f32>,
    pub width: u32,
    pub height: u32,
}

/// Min/max range for a bump map.
#[allow(dead_code)]
pub struct BumpMapRange {
    pub min: f32,
    pub max: f32,
}

/// Type alias for normal vector output.
#[allow(dead_code)]
pub type NormalVector = [f32; 3];

#[allow(dead_code)]
pub fn default_bump_map_config(width: u32, height: u32) -> BumpMapConfig {
    BumpMapConfig {
        width,
        height,
        mode: BumpMapMode::HeightMap,
        scale: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_bump_map_buffer(width: u32, height: u32) -> BumpMapBuffer {
    let count = (width * height) as usize;
    BumpMapBuffer {
        pixels: vec![0.0; count],
        width,
        height,
    }
}

#[allow(dead_code)]
pub fn set_bump_value(buffer: &mut BumpMapBuffer, x: u32, y: u32, value: f32) {
    if x < buffer.width && y < buffer.height {
        let idx = (y * buffer.width + x) as usize;
        buffer.pixels[idx] = value;
    }
}

#[allow(dead_code)]
pub fn get_bump_value(buffer: &BumpMapBuffer, x: u32, y: u32) -> f32 {
    if x < buffer.width && y < buffer.height {
        let idx = (y * buffer.width + x) as usize;
        buffer.pixels[idx]
    } else {
        0.0
    }
}

/// Compute height values from vertex positions relative to a reference plane.
/// The plane is defined by a point and normal. Heights are projected onto the normal.
#[allow(dead_code)]
pub fn bump_from_positions(
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    plane_point: [f32; 3],
    plane_normal: [f32; 3],
    width: u32,
    height: u32,
) -> BumpMapBuffer {
    let mut buffer = new_bump_map_buffer(width, height);
    let count = positions.len().min(uvs.len());
    let nlen = (plane_normal[0] * plane_normal[0]
        + plane_normal[1] * plane_normal[1]
        + plane_normal[2] * plane_normal[2])
        .sqrt()
        .max(1e-8);
    let nn = [
        plane_normal[0] / nlen,
        plane_normal[1] / nlen,
        plane_normal[2] / nlen,
    ];
    for i in 0..count {
        let dx = positions[i][0] - plane_point[0];
        let dy = positions[i][1] - plane_point[1];
        let dz = positions[i][2] - plane_point[2];
        let h = dx * nn[0] + dy * nn[1] + dz * nn[2];
        let u = uvs[i][0].clamp(0.0, 1.0);
        let v = uvs[i][1].clamp(0.0, 1.0);
        let px = (u * (width as f32 - 1.0)).round() as u32;
        let py = (v * (height as f32 - 1.0)).round() as u32;
        set_bump_value(&mut buffer, px, py, h);
    }
    buffer
}

/// Encode bump map as grayscale PGM (P5).
#[allow(dead_code)]
pub fn encode_bump_map_ppm(buffer: &BumpMapBuffer) -> Vec<u8> {
    let header = format!("P5\n{} {}\n255\n", buffer.width, buffer.height);
    let mut out = header.into_bytes();
    for &val in &buffer.pixels {
        let byte = (val.clamp(0.0, 1.0) * 255.0).round() as u8;
        out.push(byte);
    }
    out
}

/// Convert a height map to a normal map via finite differences.
/// Returns normal vectors encoded as [nx, ny, nz] per pixel.
#[allow(dead_code)]
pub fn bump_to_normal_map(buffer: &BumpMapBuffer, strength: f32) -> Vec<NormalVector> {
    let w = buffer.width as i32;
    let h = buffer.height as i32;
    let count = (buffer.width * buffer.height) as usize;
    let mut normals = Vec::with_capacity(count);

    let sample = |x: i32, y: i32| -> f32 {
        let cx = x.clamp(0, w - 1) as u32;
        let cy = y.clamp(0, h - 1) as u32;
        get_bump_value(buffer, cx, cy)
    };

    for y in 0..h {
        for x in 0..w {
            let left = sample(x - 1, y);
            let right = sample(x + 1, y);
            let up = sample(x, y - 1);
            let down = sample(x, y + 1);
            let dx = (right - left) * strength;
            let dy = (down - up) * strength;
            let nz = 1.0f32;
            let len = (dx * dx + dy * dy + nz * nz).sqrt().max(1e-8);
            normals.push([-dx / len, -dy / len, nz / len]);
        }
    }
    normals
}

/// Scale all bump values by a factor.
#[allow(dead_code)]
pub fn scale_bump_values(buffer: &mut BumpMapBuffer, factor: f32) {
    for px in buffer.pixels.iter_mut() {
        *px *= factor;
    }
}

/// Invert the bump map (1.0 - value for each pixel).
#[allow(dead_code)]
pub fn invert_bump_map(buffer: &mut BumpMapBuffer) {
    for px in buffer.pixels.iter_mut() {
        *px = 1.0 - *px;
    }
}

/// Apply a box blur to the bump map.
#[allow(dead_code)]
pub fn blur_bump_map(buffer: &mut BumpMapBuffer, radius: u32) {
    if radius == 0 {
        return;
    }
    let w = buffer.width;
    let h = buffer.height;
    let r = radius as i32;
    let src = buffer.pixels.clone();

    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f64;
            let mut count = 0u32;
            for dy in -r..=r {
                for dx in -r..=r {
                    let sx = (x as i32 + dx).clamp(0, w as i32 - 1) as u32;
                    let sy = (y as i32 + dy).clamp(0, h as i32 - 1) as u32;
                    sum += src[(sy * w + sx) as usize] as f64;
                    count += 1;
                }
            }
            buffer.pixels[(y * w + x) as usize] = (sum / count as f64) as f32;
        }
    }
}

/// Return min/max of the bump map.
#[allow(dead_code)]
pub fn bump_map_range(buffer: &BumpMapBuffer) -> BumpMapRange {
    if buffer.pixels.is_empty() {
        return BumpMapRange { min: 0.0, max: 0.0 };
    }
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for &v in &buffer.pixels {
        if v < min {
            min = v;
        }
        if v > max {
            max = v;
        }
    }
    BumpMapRange { min, max }
}

/// Return total pixel count.
#[allow(dead_code)]
pub fn bump_map_pixel_count(buffer: &BumpMapBuffer) -> usize {
    buffer.pixels.len()
}

/// Clamp all bump values to [lo, hi].
#[allow(dead_code)]
pub fn clamp_bump_values(buffer: &mut BumpMapBuffer, lo: f32, hi: f32) {
    for px in buffer.pixels.iter_mut() {
        *px = px.clamp(lo, hi);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_bump_map_config(512, 256);
        assert_eq!(cfg.width, 512);
        assert_eq!(cfg.height, 256);
        assert_eq!(cfg.mode, BumpMapMode::HeightMap);
        assert!((cfg.scale - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_new_buffer() {
        let buf = new_bump_map_buffer(8, 8);
        assert_eq!(buf.width, 8);
        assert_eq!(buf.height, 8);
        assert_eq!(buf.pixels.len(), 64);
    }

    #[test]
    fn test_set_get_value() {
        let mut buf = new_bump_map_buffer(4, 4);
        set_bump_value(&mut buf, 2, 1, 0.5);
        assert!((get_bump_value(&buf, 2, 1) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_get_out_of_bounds() {
        let buf = new_bump_map_buffer(4, 4);
        assert!((get_bump_value(&buf, 10, 10) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_out_of_bounds() {
        let mut buf = new_bump_map_buffer(4, 4);
        set_bump_value(&mut buf, 10, 10, 1.0);
        for &px in &buf.pixels {
            assert!((px - 0.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_bump_from_positions() {
        let positions = [[0.0f32, 0.0, 1.0], [1.0, 0.0, 2.0]];
        let uvs = [[0.0f32, 0.0], [1.0, 1.0]];
        let buf = bump_from_positions(&positions, &uvs, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 8, 8);
        assert_eq!(buf.width, 8);
        assert!((get_bump_value(&buf, 0, 0) - 1.0).abs() < f32::EPSILON);
        assert!((get_bump_value(&buf, 7, 7) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_encode_ppm_starts_with_p5() {
        let buf = new_bump_map_buffer(2, 2);
        let ppm = encode_bump_map_ppm(&buf);
        assert!(ppm.starts_with(b"P5"));
    }

    #[test]
    fn test_encode_ppm_size() {
        let buf = new_bump_map_buffer(4, 4);
        let ppm = encode_bump_map_ppm(&buf);
        let header = "P5\n4 4\n255\n".to_string();
        assert_eq!(ppm.len(), header.len() + 16);
    }

    #[test]
    fn test_bump_to_normal_map() {
        let mut buf = new_bump_map_buffer(4, 4);
        // Flat surface => normals should be ~(0, 0, 1)
        for px in buf.pixels.iter_mut() {
            *px = 0.5;
        }
        let normals = bump_to_normal_map(&buf, 1.0);
        assert_eq!(normals.len(), 16);
        for n in &normals {
            assert!((n[2] - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_scale_bump_values() {
        let mut buf = new_bump_map_buffer(2, 2);
        for px in buf.pixels.iter_mut() {
            *px = 0.5;
        }
        scale_bump_values(&mut buf, 2.0);
        for &px in &buf.pixels {
            assert!((px - 1.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_invert_bump_map() {
        let mut buf = new_bump_map_buffer(2, 2);
        set_bump_value(&mut buf, 0, 0, 0.3);
        invert_bump_map(&mut buf);
        assert!((get_bump_value(&buf, 0, 0) - 0.7).abs() < 1e-6);
        // Default 0.0 inverts to 1.0
        assert!((get_bump_value(&buf, 1, 0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_blur_bump_map() {
        let mut buf = new_bump_map_buffer(4, 4);
        set_bump_value(&mut buf, 2, 2, 1.0);
        blur_bump_map(&mut buf, 1);
        // Center should be less than 1.0 after blur
        let center = get_bump_value(&buf, 2, 2);
        assert!(center < 1.0);
        assert!(center > 0.0);
    }

    #[test]
    fn test_blur_radius_zero() {
        let mut buf = new_bump_map_buffer(4, 4);
        set_bump_value(&mut buf, 1, 1, 0.5);
        blur_bump_map(&mut buf, 0);
        assert!((get_bump_value(&buf, 1, 1) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bump_map_range() {
        let mut buf = new_bump_map_buffer(4, 4);
        set_bump_value(&mut buf, 0, 0, 0.1);
        set_bump_value(&mut buf, 1, 0, 0.9);
        let range = bump_map_range(&buf);
        assert!((range.min - 0.0).abs() < f32::EPSILON);
        assert!((range.max - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bump_map_range_empty() {
        let buf = BumpMapBuffer {
            pixels: vec![],
            width: 0,
            height: 0,
        };
        let range = bump_map_range(&buf);
        assert!((range.min - 0.0).abs() < f32::EPSILON);
        assert!((range.max - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bump_map_pixel_count() {
        let buf = new_bump_map_buffer(8, 4);
        assert_eq!(bump_map_pixel_count(&buf), 32);
    }

    #[test]
    fn test_clamp_bump_values() {
        let mut buf = new_bump_map_buffer(2, 2);
        set_bump_value(&mut buf, 0, 0, -0.5);
        set_bump_value(&mut buf, 1, 0, 1.5);
        clamp_bump_values(&mut buf, 0.0, 1.0);
        assert!((get_bump_value(&buf, 0, 0) - 0.0).abs() < f32::EPSILON);
        assert!((get_bump_value(&buf, 1, 0) - 1.0).abs() < f32::EPSILON);
    }
}
