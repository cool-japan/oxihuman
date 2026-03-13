// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Procedural texture generation: RGBA pixel buffers for skin tones,
//! checker patterns, gradient maps, UV visualization, and normal maps.

#[allow(dead_code)]
/// An RGBA pixel buffer.
pub struct PixelBuffer {
    pub width: u32,
    pub height: u32,
    /// Row-major RGBA bytes [r, g, b, a, r, g, b, a, ...]
    pub pixels: Vec<u8>,
}

impl PixelBuffer {
    /// Create a new zeroed pixel buffer.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0u8; (width * height * 4) as usize],
        }
    }

    /// Set pixel at (x, y) to RGBA.
    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        let idx = ((y * self.width + x) * 4) as usize;
        self.pixels[idx] = r;
        self.pixels[idx + 1] = g;
        self.pixels[idx + 2] = b;
        self.pixels[idx + 3] = a;
    }

    /// Get pixel at (x, y).
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let idx = ((y * self.width + x) * 4) as usize;
        [
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ]
    }

    /// Fill entire buffer with a flat color.
    pub fn fill(&mut self, r: u8, g: u8, b: u8, a: u8) {
        for chunk in self.pixels.chunks_exact_mut(4) {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
            chunk[3] = a;
        }
    }

    /// Byte count.
    pub fn byte_len(&self) -> usize {
        self.pixels.len()
    }

    /// Export as a TGA file (simple uncompressed 32-bit RGBA TGA — no external crates needed).
    ///
    /// TGA format: 18-byte header + pixel data (top-to-bottom with flip flag).
    pub fn to_tga_bytes(&self) -> Vec<u8> {
        let width = self.width as u16;
        let height = self.height as u16;
        let pixel_count = (self.width * self.height) as usize;

        // 18-byte TGA header
        let mut out = Vec::with_capacity(18 + pixel_count * 4);

        out.push(0u8); // [0]  no image ID
        out.push(0u8); // [1]  no colormap
        out.push(2u8); // [2]  uncompressed true-color
        out.extend_from_slice(&[0u8, 0u8]); // [3..4] colormap origin
        out.extend_from_slice(&[0u8, 0u8]); // [5..6] colormap length
        out.push(0u8); // [7]  colormap depth
        out.extend_from_slice(&[0u8, 0u8]); // [8..9]  x-origin
        out.extend_from_slice(&[0u8, 0u8]); // [10..11] y-origin
        out.extend_from_slice(&width.to_le_bytes()); // [12..13] width
        out.extend_from_slice(&height.to_le_bytes()); // [14..15] height
        out.push(32u8); // [16] bits per pixel (32 = RGBA)
        out.push(0x28u8); // [17] image descriptor: top-left origin, 8 alpha bits

        // Pixel data: row by row (top to bottom), each pixel as BGRA
        for chunk in self.pixels.chunks_exact(4) {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];
            let a = chunk[3];
            out.push(b);
            out.push(g);
            out.push(r);
            out.push(a);
        }

        out
    }

    /// Save to a .tga file.
    pub fn save_tga(&self, path: &std::path::Path) -> anyhow::Result<()> {
        std::fs::write(path, self.to_tga_bytes()).map_err(Into::into)
    }
}

/// Generate a solid-color skin texture.
pub fn generate_skin_texture(width: u32, height: u32, r: u8, g: u8, b: u8) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height);
    buf.fill(r, g, b, 255);
    buf
}

/// Generate a checker pattern (for UV debugging).
pub fn generate_checker_texture(width: u32, height: u32, cell_size: u32) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height);
    let cell_size = cell_size.max(1);
    for y in 0..height {
        for x in 0..width {
            let cx = x / cell_size;
            let cy = y / cell_size;
            if (cx + cy).is_multiple_of(2) {
                buf.set_pixel(x, y, 255, 255, 255, 255);
            } else {
                buf.set_pixel(x, y, 0, 0, 0, 255);
            }
        }
    }
    buf
}

/// Generate a vertical gradient from `top` to `bottom` color.
pub fn generate_gradient_texture(
    width: u32,
    height: u32,
    top: [u8; 3],
    bottom: [u8; 3],
) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height);
    for y in 0..height {
        let t = if height <= 1 {
            0.0f32
        } else {
            y as f32 / (height - 1) as f32
        };
        let r = (top[0] as f32 * (1.0 - t) + bottom[0] as f32 * t).round() as u8;
        let g = (top[1] as f32 * (1.0 - t) + bottom[1] as f32 * t).round() as u8;
        let b = (top[2] as f32 * (1.0 - t) + bottom[2] as f32 * t).round() as u8;
        for x in 0..width {
            buf.set_pixel(x, y, r, g, b, 255);
        }
    }
    buf
}

/// Generate a UV coordinate visualization texture:
/// pixel (x,y) gets R = x/width * 255, G = y/height * 255, B = 0, A = 255.
pub fn generate_uv_texture(width: u32, height: u32) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let r = if width <= 1 {
                0u8
            } else {
                (x as f32 / (width - 1) as f32 * 255.0).round() as u8
            };
            let g = if height <= 1 {
                0u8
            } else {
                (y as f32 / (height - 1) as f32 * 255.0).round() as u8
            };
            buf.set_pixel(x, y, r, g, 0, 255);
        }
    }
    buf
}

/// Generate a normal map texture (flat normal pointing toward viewer: RGB = [128, 128, 255]).
pub fn generate_flat_normal_map(width: u32, height: u32) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height);
    buf.fill(128, 128, 255, 255);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_buffer_size() {
        let buf = PixelBuffer::new(4, 4);
        assert_eq!(buf.byte_len(), 64);
    }

    #[test]
    fn set_get_pixel() {
        let mut buf = PixelBuffer::new(4, 4);
        buf.set_pixel(1, 1, 255, 0, 128, 255);
        assert_eq!(buf.get_pixel(1, 1), [255, 0, 128, 255]);
    }

    #[test]
    fn fill_sets_all() {
        let mut buf = PixelBuffer::new(4, 4);
        buf.fill(255, 0, 0, 255);
        assert_eq!(buf.get_pixel(0, 0), [255, 0, 0, 255]);
        assert_eq!(buf.get_pixel(3, 3), [255, 0, 0, 255]);
    }

    #[test]
    fn checker_alternates() {
        let cell_size = 4u32;
        let buf = generate_checker_texture(16, 16, cell_size);
        let p0 = buf.get_pixel(0, 0);
        let p1 = buf.get_pixel(cell_size, 0);
        assert_ne!(p0, p1);
    }

    #[test]
    fn gradient_top_equals_top_color() {
        let top = [255u8, 0, 0];
        let bottom = [0u8, 0, 255];
        let buf = generate_gradient_texture(8, 8, top, bottom);
        let p = buf.get_pixel(0, 0);
        assert_eq!([p[0], p[1], p[2]], top);
    }

    #[test]
    fn gradient_bottom_equals_bottom_color() {
        let top = [255u8, 0, 0];
        let bottom = [0u8, 0, 255];
        let buf = generate_gradient_texture(8, 8, top, bottom);
        let p = buf.get_pixel(0, 7);
        assert_eq!([p[0], p[1], p[2]], bottom);
    }

    #[test]
    fn uv_texture_top_left_is_zero() {
        let buf = generate_uv_texture(8, 8);
        let p = buf.get_pixel(0, 0);
        assert_eq!(p[0], 0); // R = 0
        assert_eq!(p[1], 0); // G = 0
    }

    #[test]
    fn flat_normal_map_is_blue() {
        let buf = generate_flat_normal_map(4, 4);
        let p = buf.get_pixel(0, 0);
        assert_eq!(p[2], 255); // B = 255
    }

    #[test]
    fn tga_header_magic() {
        let buf = generate_skin_texture(4, 4, 200, 150, 130);
        let bytes = buf.to_tga_bytes();
        assert_eq!(bytes[2], 2); // uncompressed true-color type
    }

    #[test]
    fn save_tga_creates_file() {
        let buf = generate_uv_texture(16, 16);
        let path = std::path::Path::new("/tmp/test_texture.tga");
        buf.save_tga(path).expect("save_tga failed");
        assert!(path.exists());
    }
}
