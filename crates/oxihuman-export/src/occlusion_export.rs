// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Ambient occlusion map export.

#[allow(dead_code)]
pub struct OcclusionMapConfig {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub radius: f32,
    pub power: f32,
    pub invert: bool,
}

#[allow(dead_code)]
pub struct OcclusionMapBuffer {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
pub fn default_occlusion_config(width: u32, height: u32) -> OcclusionMapConfig {
    OcclusionMapConfig {
        width,
        height,
        samples: 16,
        radius: 0.1,
        power: 1.0,
        invert: false,
    }
}

#[allow(dead_code)]
pub fn new_occlusion_buffer(width: u32, height: u32) -> OcclusionMapBuffer {
    let count = (width * height) as usize;
    OcclusionMapBuffer {
        pixels: vec![255u8; count],
        width,
        height,
    }
}

#[allow(dead_code)]
pub fn set_occlusion_pixel(buf: &mut OcclusionMapBuffer, x: u32, y: u32, value: f32) {
    if x < buf.width && y < buf.height {
        let idx = (y * buf.width + x) as usize;
        buf.pixels[idx] = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
}

#[allow(dead_code)]
pub fn get_occlusion_pixel(buf: &OcclusionMapBuffer, x: u32, y: u32) -> f32 {
    if x < buf.width && y < buf.height {
        let idx = (y * buf.width + x) as usize;
        buf.pixels[idx] as f32 / 255.0
    } else {
        1.0
    }
}

#[allow(dead_code)]
pub fn fill_occlusion_buffer(buf: &mut OcclusionMapBuffer, value: f32) {
    let v = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    for pixel in buf.pixels.iter_mut() {
        *pixel = v;
    }
}

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn bake_ao_to_buffer(
    _positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    _indices: &[u32],
    cfg: &OcclusionMapConfig,
) -> OcclusionMapBuffer {
    let mut buf = new_occlusion_buffer(cfg.width, cfg.height);
    let count = normals.len().min(uvs.len());
    for i in 0..count {
        let u = uvs[i][0].clamp(0.0, 1.0);
        let v = uvs[i][1].clamp(0.0, 1.0);
        let px = (u * (cfg.width as f32 - 1.0)).round() as u32;
        let py = (v * (cfg.height as f32 - 1.0)).round() as u32;
        // Simple AO: use upward-facing normals as unoccluded
        let ao = normals[i][2].clamp(0.0, 1.0);
        let ao = ao.powf(cfg.power);
        set_occlusion_pixel(&mut buf, px, py, ao);
    }
    buf
}

#[allow(dead_code)]
pub fn encode_occlusion_ppm(buf: &OcclusionMapBuffer) -> Vec<u8> {
    let header = format!("P5\n{} {}\n255\n", buf.width, buf.height);
    let mut out = header.into_bytes();
    out.extend_from_slice(&buf.pixels);
    out
}

#[allow(dead_code)]
pub fn apply_occlusion_power(buf: &mut OcclusionMapBuffer, power: f32) {
    for pixel in buf.pixels.iter_mut() {
        let v = (*pixel as f32 / 255.0).powf(power);
        *pixel = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
}

#[allow(dead_code)]
pub fn blur_occlusion_buffer(buf: &mut OcclusionMapBuffer) {
    let w = buf.width as usize;
    let h = buf.height as usize;
    if w < 3 || h < 3 {
        return;
    }
    let src = buf.pixels.clone();
    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            let mut sum = 0u32;
            for dy in 0..3usize {
                for dx in 0..3usize {
                    let nx = x + dx - 1;
                    let ny = y + dy - 1;
                    sum += src[ny * w + nx] as u32;
                }
            }
            buf.pixels[y * w + x] = (sum / 9) as u8;
        }
    }
}

#[allow(dead_code)]
pub fn occlusion_buffer_average(buf: &OcclusionMapBuffer) -> f32 {
    if buf.pixels.is_empty() {
        return 0.0;
    }
    let sum: u64 = buf.pixels.iter().map(|&v| v as u64).sum();
    sum as f32 / (buf.pixels.len() as f32 * 255.0)
}

#[allow(dead_code)]
pub fn occlusion_pixel_count(buf: &OcclusionMapBuffer) -> usize {
    buf.pixels.len()
}

#[allow(dead_code)]
pub fn occlusion_map_to_rgba(buf: &OcclusionMapBuffer) -> Vec<[u8; 4]> {
    buf.pixels.iter().map(|&v| [v, v, v, 255]).collect()
}

#[allow(dead_code)]
pub fn composite_ao_with_albedo(albedo: &[[u8; 4]], ao: &OcclusionMapBuffer) -> Vec<[u8; 4]> {
    let count = albedo.len().min(ao.pixels.len());
    let mut out = Vec::with_capacity(count);
    for (i, alb) in albedo.iter().enumerate().take(count) {
        let a = ao.pixels[i] as f32 / 255.0;
        out.push([
            (alb[0] as f32 * a).round() as u8,
            (alb[1] as f32 * a).round() as u8,
            (alb[2] as f32 * a).round() as u8,
            alb[3],
        ]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_occlusion_config(512, 256);
        assert_eq!(cfg.width, 512);
        assert_eq!(cfg.height, 256);
        assert_eq!(cfg.samples, 16);
        assert!(!cfg.invert);
    }

    #[test]
    fn test_new_buffer() {
        let buf = new_occlusion_buffer(4, 4);
        assert_eq!(buf.width, 4);
        assert_eq!(buf.height, 4);
        assert_eq!(buf.pixels.len(), 16);
    }

    #[test]
    fn test_set_get_pixel_round_trip() {
        let mut buf = new_occlusion_buffer(8, 8);
        set_occlusion_pixel(&mut buf, 3, 2, 0.5);
        let got = get_occlusion_pixel(&buf, 3, 2);
        assert!((got - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_set_pixel_clamp() {
        let mut buf = new_occlusion_buffer(4, 4);
        set_occlusion_pixel(&mut buf, 0, 0, 1.5);
        assert_eq!(buf.pixels[0], 255);
        set_occlusion_pixel(&mut buf, 0, 0, -0.5);
        assert_eq!(buf.pixels[0], 0);
    }

    #[test]
    fn test_fill_buffer() {
        let mut buf = new_occlusion_buffer(4, 4);
        fill_occlusion_buffer(&mut buf, 0.0);
        for &p in &buf.pixels {
            assert_eq!(p, 0);
        }
    }

    #[test]
    fn test_encode_ppm_starts_with_p5() {
        let buf = new_occlusion_buffer(2, 2);
        let ppm = encode_occlusion_ppm(&buf);
        assert!(ppm.starts_with(b"P5"));
    }

    #[test]
    fn test_encode_ppm_size() {
        let buf = new_occlusion_buffer(3, 3);
        let ppm = encode_occlusion_ppm(&buf);
        let header_len = "P5\n3 3\n255\n".len();
        assert_eq!(ppm.len(), header_len + 9);
    }

    #[test]
    fn test_apply_power() {
        let mut buf = new_occlusion_buffer(4, 4);
        fill_occlusion_buffer(&mut buf, 1.0);
        apply_occlusion_power(&mut buf, 2.0);
        // 1.0^2 = 1.0
        for &p in &buf.pixels {
            assert_eq!(p, 255);
        }
    }

    #[test]
    fn test_buffer_average_full() {
        let buf = new_occlusion_buffer(4, 4);
        let avg = occlusion_buffer_average(&buf);
        assert!((avg - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_buffer_average_zero() {
        let mut buf = new_occlusion_buffer(4, 4);
        fill_occlusion_buffer(&mut buf, 0.0);
        let avg = occlusion_buffer_average(&buf);
        assert!((avg - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_pixel_count() {
        let buf = new_occlusion_buffer(8, 8);
        assert_eq!(occlusion_pixel_count(&buf), 64);
    }

    #[test]
    fn test_to_rgba_size() {
        let buf = new_occlusion_buffer(4, 4);
        let rgba = occlusion_map_to_rgba(&buf);
        assert_eq!(rgba.len(), 16);
        assert_eq!(rgba[0][3], 255);
    }

    #[test]
    fn test_blur_doesnt_crash() {
        let mut buf = new_occlusion_buffer(8, 8);
        fill_occlusion_buffer(&mut buf, 0.5);
        blur_occlusion_buffer(&mut buf);
        assert_eq!(buf.pixels.len(), 64);
    }

    #[test]
    fn test_composite_ao_with_albedo() {
        let albedo = vec![[200u8, 100, 50, 255]; 4];
        let mut ao = new_occlusion_buffer(2, 2);
        fill_occlusion_buffer(&mut ao, 1.0);
        let result = composite_ao_with_albedo(&albedo, &ao);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0][0], 200);
    }

    #[test]
    fn test_get_pixel_out_of_bounds() {
        let buf = new_occlusion_buffer(4, 4);
        let v = get_occlusion_pixel(&buf, 10, 10);
        assert!((v - 1.0).abs() < 0.01);
    }
}
