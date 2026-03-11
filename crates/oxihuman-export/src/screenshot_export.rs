// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Screenshot capture to PNG-like byte buffer (simple PPM/raw/TGA format, no external deps).

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum ScreenshotFormat {
    Ppm,
    Raw,
    Tga,
}

#[allow(dead_code)]
pub struct ScreenshotConfig {
    pub width: u32,
    pub height: u32,
    pub format: ScreenshotFormat,
    pub gamma_correct: bool,
    pub include_alpha: bool,
}

#[allow(dead_code)]
pub struct ScreenshotBuffer {
    pub pixels: Vec<u8>, // RGBA u8 pixels
    pub width: u32,
    pub height: u32,
    pub format: ScreenshotFormat,
}

#[allow(dead_code)]
pub fn default_screenshot_config(width: u32, height: u32) -> ScreenshotConfig {
    ScreenshotConfig {
        width,
        height,
        format: ScreenshotFormat::Ppm,
        gamma_correct: false,
        include_alpha: false,
    }
}

#[allow(dead_code)]
pub fn new_screenshot_buffer(width: u32, height: u32) -> ScreenshotBuffer {
    let size = (width as usize) * (height as usize) * 4;
    ScreenshotBuffer {
        pixels: vec![0u8; size],
        width,
        height,
        format: ScreenshotFormat::Raw,
    }
}

#[allow(dead_code)]
pub fn set_pixel(buf: &mut ScreenshotBuffer, x: u32, y: u32, rgba: [u8; 4]) {
    if x >= buf.width || y >= buf.height {
        return;
    }
    let base = ((y as usize) * (buf.width as usize) + (x as usize)) * 4;
    if base + 3 < buf.pixels.len() {
        buf.pixels[base] = rgba[0];
        buf.pixels[base + 1] = rgba[1];
        buf.pixels[base + 2] = rgba[2];
        buf.pixels[base + 3] = rgba[3];
    }
}

#[allow(dead_code)]
pub fn get_pixel(buf: &ScreenshotBuffer, x: u32, y: u32) -> [u8; 4] {
    if x >= buf.width || y >= buf.height {
        return [0u8; 4];
    }
    let base = ((y as usize) * (buf.width as usize) + (x as usize)) * 4;
    if base + 3 < buf.pixels.len() {
        [
            buf.pixels[base],
            buf.pixels[base + 1],
            buf.pixels[base + 2],
            buf.pixels[base + 3],
        ]
    } else {
        [0u8; 4]
    }
}

/// Encode as PPM binary format (P6).
/// PPM does not include alpha; only RGB is written.
#[allow(dead_code)]
pub fn encode_ppm(buf: &ScreenshotBuffer) -> Vec<u8> {
    let header = format!("P6\n{} {}\n255\n", buf.width, buf.height);
    let pixel_count = (buf.width as usize) * (buf.height as usize);
    let mut out = Vec::with_capacity(header.len() + pixel_count * 3);
    out.extend_from_slice(header.as_bytes());
    for i in 0..pixel_count {
        let base = i * 4;
        if base + 2 < buf.pixels.len() {
            out.push(buf.pixels[base]);
            out.push(buf.pixels[base + 1]);
            out.push(buf.pixels[base + 2]);
        } else {
            out.extend_from_slice(&[0u8, 0u8, 0u8]);
        }
    }
    out
}

/// Encode as TGA (type 2, uncompressed true-color).
#[allow(dead_code)]
pub fn encode_tga(buf: &ScreenshotBuffer) -> Vec<u8> {
    let w = buf.width as u16;
    let h = buf.height as u16;
    // TGA header: 18 bytes
    let mut out = Vec::with_capacity(18 + (buf.width as usize) * (buf.height as usize) * 4);
    out.push(0); // ID length
    out.push(0); // color map type: none
    out.push(2); // image type: uncompressed true-color
    out.extend_from_slice(&[0u8; 5]); // color map spec (not used)
    out.extend_from_slice(&0u16.to_le_bytes()); // x origin
    out.extend_from_slice(&0u16.to_le_bytes()); // y origin
    out.extend_from_slice(&w.to_le_bytes()); // width
    out.extend_from_slice(&h.to_le_bytes()); // height
    out.push(32); // bits per pixel (BGRA)
    out.push(0x08); // image descriptor: 8 bits alpha, origin top-left

    let pixel_count = (buf.width as usize) * (buf.height as usize);
    for i in 0..pixel_count {
        let base = i * 4;
        if base + 3 < buf.pixels.len() {
            // TGA stores BGRA
            out.push(buf.pixels[base + 2]); // B
            out.push(buf.pixels[base + 1]); // G
            out.push(buf.pixels[base]); // R
            out.push(buf.pixels[base + 3]); // A
        } else {
            out.extend_from_slice(&[0u8; 4]);
        }
    }
    out
}

/// Encode as raw RGBA bytes (no header).
#[allow(dead_code)]
pub fn encode_raw(buf: &ScreenshotBuffer) -> Vec<u8> {
    buf.pixels.clone()
}

#[allow(dead_code)]
pub fn apply_gamma_correction(buf: &mut ScreenshotBuffer, gamma: f32) {
    let inv_gamma = if gamma.abs() < 1e-6 { 1.0 } else { 1.0 / gamma };
    for (i, pixel) in buf.pixels.iter_mut().enumerate() {
        // Skip alpha channel (every 4th byte)
        if i % 4 != 3 {
            let linear = (*pixel as f32) / 255.0;
            let corrected = linear.powf(inv_gamma);
            *pixel = (corrected * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
}

#[allow(dead_code)]
pub fn flip_vertical(buf: &mut ScreenshotBuffer) {
    let row_bytes = (buf.width as usize) * 4;
    let height = buf.height as usize;
    for row in 0..height / 2 {
        let top = row * row_bytes;
        let bot = (height - 1 - row) * row_bytes;
        for col in 0..row_bytes {
            buf.pixels.swap(top + col, bot + col);
        }
    }
}

#[allow(dead_code)]
pub fn crop_screenshot(buf: &ScreenshotBuffer, x: u32, y: u32, w: u32, h: u32) -> ScreenshotBuffer {
    let x = x.min(buf.width);
    let y = y.min(buf.height);
    let w = w.min(buf.width.saturating_sub(x));
    let h = h.min(buf.height.saturating_sub(y));
    let mut out = new_screenshot_buffer(w, h);
    for row in 0..h {
        for col in 0..w {
            let src_base = (((y + row) as usize) * (buf.width as usize) + ((x + col) as usize)) * 4;
            let dst_base = ((row as usize) * (w as usize) + (col as usize)) * 4;
            if src_base + 3 < buf.pixels.len() && dst_base + 3 < out.pixels.len() {
                out.pixels[dst_base] = buf.pixels[src_base];
                out.pixels[dst_base + 1] = buf.pixels[src_base + 1];
                out.pixels[dst_base + 2] = buf.pixels[src_base + 2];
                out.pixels[dst_base + 3] = buf.pixels[src_base + 3];
            }
        }
    }
    out
}

#[allow(dead_code)]
pub fn screenshot_size_bytes(cfg: &ScreenshotConfig) -> usize {
    let pixels = (cfg.width as usize) * (cfg.height as usize);
    match cfg.format {
        ScreenshotFormat::Ppm => {
            // header is at most ~30 bytes
            let header_est = format!("P6\n{} {}\n255\n", cfg.width, cfg.height).len();
            header_est + pixels * 3
        }
        ScreenshotFormat::Raw => pixels * 4,
        ScreenshotFormat::Tga => 18 + pixels * 4,
    }
}

#[allow(dead_code)]
pub fn blend_overlay(base: &mut ScreenshotBuffer, overlay: &ScreenshotBuffer, alpha: f32) {
    let alpha = alpha.clamp(0.0, 1.0);
    let w = base.width.min(overlay.width) as usize;
    let h = base.height.min(overlay.height) as usize;
    for row in 0..h {
        for col in 0..w {
            let bi = (row * (base.width as usize) + col) * 4;
            let oi = (row * (overlay.width as usize) + col) * 4;
            if bi + 3 < base.pixels.len() && oi + 3 < overlay.pixels.len() {
                for c in 0..4 {
                    let bv = base.pixels[bi + c] as f32;
                    let ov = overlay.pixels[oi + c] as f32;
                    base.pixels[bi + c] =
                        (bv * (1.0 - alpha) + ov * alpha).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn clear_screenshot(buf: &mut ScreenshotBuffer, color: [u8; 4]) {
    let pixel_count = (buf.width as usize) * (buf.height as usize);
    for i in 0..pixel_count {
        let base = i * 4;
        if base + 3 < buf.pixels.len() {
            buf.pixels[base] = color[0];
            buf.pixels[base + 1] = color[1];
            buf.pixels[base + 2] = color[2];
            buf.pixels[base + 3] = color[3];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_screenshot_buffer() {
        let buf = new_screenshot_buffer(4, 4);
        assert_eq!(buf.width, 4);
        assert_eq!(buf.height, 4);
        assert_eq!(buf.pixels.len(), 4 * 4 * 4);
        assert!(buf.pixels.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_set_and_get_pixel() {
        let mut buf = new_screenshot_buffer(10, 10);
        set_pixel(&mut buf, 3, 5, [255, 128, 64, 255]);
        let px = get_pixel(&buf, 3, 5);
        assert_eq!(px, [255, 128, 64, 255]);
    }

    #[test]
    fn test_get_pixel_out_of_bounds() {
        let buf = new_screenshot_buffer(4, 4);
        let px = get_pixel(&buf, 100, 100);
        assert_eq!(px, [0u8; 4]);
    }

    #[test]
    fn test_encode_ppm_starts_with_p6() {
        let buf = new_screenshot_buffer(2, 2);
        let ppm = encode_ppm(&buf);
        let header = std::str::from_utf8(&ppm[..2]).unwrap();
        assert_eq!(header, "P6");
    }

    #[test]
    fn test_encode_ppm_correct_size() {
        let buf = new_screenshot_buffer(3, 3);
        let ppm = encode_ppm(&buf);
        let header = "P6\n3 3\n255\n".to_string();
        let expected_len = header.len() + 3 * 3 * 3; // RGB, not RGBA
        assert_eq!(ppm.len(), expected_len);
    }

    #[test]
    fn test_encode_tga_minimum_size() {
        let buf = new_screenshot_buffer(4, 4);
        let tga = encode_tga(&buf);
        assert!(
            tga.len() >= 18,
            "TGA should at least have the 18-byte header"
        );
    }

    #[test]
    fn test_encode_tga_header_image_type() {
        let buf = new_screenshot_buffer(2, 2);
        let tga = encode_tga(&buf);
        assert_eq!(
            tga[2], 2,
            "TGA image type should be 2 (uncompressed true-color)"
        );
    }

    #[test]
    fn test_encode_raw_length() {
        let buf = new_screenshot_buffer(5, 5);
        let raw = encode_raw(&buf);
        assert_eq!(
            raw.len(),
            5 * 5 * 4,
            "raw should be exactly width*height*4 bytes"
        );
    }

    #[test]
    fn test_flip_vertical() {
        let mut buf = new_screenshot_buffer(2, 2);
        set_pixel(&mut buf, 0, 0, [255, 0, 0, 255]); // top-left red
        set_pixel(&mut buf, 0, 1, [0, 0, 255, 255]); // bottom-left blue
        flip_vertical(&mut buf);
        // After flip: top-left should now be blue, bottom-left should be red
        assert_eq!(get_pixel(&buf, 0, 0), [0, 0, 255, 255]);
        assert_eq!(get_pixel(&buf, 0, 1), [255, 0, 0, 255]);
    }

    #[test]
    fn test_crop_screenshot() {
        let mut buf = new_screenshot_buffer(4, 4);
        set_pixel(&mut buf, 2, 2, [100, 200, 50, 255]);
        let cropped = crop_screenshot(&buf, 2, 2, 2, 2);
        assert_eq!(cropped.width, 2);
        assert_eq!(cropped.height, 2);
        assert_eq!(get_pixel(&cropped, 0, 0), [100, 200, 50, 255]);
    }

    #[test]
    fn test_clear_screenshot() {
        let mut buf = new_screenshot_buffer(3, 3);
        clear_screenshot(&mut buf, [128, 64, 32, 255]);
        for y in 0..3 {
            for x in 0..3 {
                assert_eq!(get_pixel(&buf, x, y), [128, 64, 32, 255]);
            }
        }
    }

    #[test]
    fn test_screenshot_size_bytes_raw() {
        let cfg = ScreenshotConfig {
            width: 10,
            height: 10,
            format: ScreenshotFormat::Raw,
            gamma_correct: false,
            include_alpha: false,
        };
        assert_eq!(screenshot_size_bytes(&cfg), 10 * 10 * 4);
    }

    #[test]
    fn test_screenshot_size_bytes_tga() {
        let cfg = ScreenshotConfig {
            width: 8,
            height: 8,
            format: ScreenshotFormat::Tga,
            gamma_correct: false,
            include_alpha: true,
        };
        assert_eq!(screenshot_size_bytes(&cfg), 18 + 8 * 8 * 4);
    }

    #[test]
    fn test_blend_overlay() {
        let mut base = new_screenshot_buffer(2, 2);
        clear_screenshot(&mut base, [0, 0, 0, 255]);
        let mut overlay = new_screenshot_buffer(2, 2);
        clear_screenshot(&mut overlay, [200, 200, 200, 255]);
        blend_overlay(&mut base, &overlay, 0.5);
        let px = get_pixel(&base, 0, 0);
        // 0 * 0.5 + 200 * 0.5 = 100 (rounded)
        assert_eq!(px[0], 100);
    }

    #[test]
    fn test_apply_gamma_correction() {
        let mut buf = new_screenshot_buffer(1, 1);
        set_pixel(&mut buf, 0, 0, [128, 128, 128, 255]);
        apply_gamma_correction(&mut buf, 2.2);
        let px = get_pixel(&buf, 0, 0);
        // After gamma correction the value changes, alpha stays
        assert_eq!(px[3], 255, "alpha should be unchanged");
        // value should differ from 128 after gamma
        assert_ne!(px[0], 128, "gamma correction should change the value");
    }

    #[test]
    fn test_default_screenshot_config() {
        let cfg = default_screenshot_config(1920, 1080);
        assert_eq!(cfg.width, 1920);
        assert_eq!(cfg.height, 1080);
        assert_eq!(cfg.format, ScreenshotFormat::Ppm);
        assert!(!cfg.gamma_correct);
    }
}
