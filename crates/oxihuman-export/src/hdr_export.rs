//! HDR image export stub — RGBE-encoded HDR data with metadata.
//!
//! Provides a minimal in-memory HDR image type and helper functions for
//! encoding pixels as Radiance RGBE bytes and applying simple tone-mapping.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for HDR export (exposure, tone-map settings, …).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HdrExportConfig {
    /// Default exposure value applied during export.
    pub exposure: f32,
    /// Whether to embed a header comment in the output.
    pub write_header: bool,
    /// Gamma for LDR conversion (usually 2.2).
    pub gamma: f32,
}

/// A single HDR pixel stored as linear floating-point RGB.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HdrPixel {
    /// Red channel (linear, unbounded).
    pub r: f32,
    /// Green channel (linear, unbounded).
    pub g: f32,
    /// Blue channel (linear, unbounded).
    pub b: f32,
}

/// An in-memory HDR image (linear float RGB per pixel).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HdrImage {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Packed pixel buffer (row-major, width × height entries).
    pub pixels: Vec<HdrPixel>,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Returns a default [`HdrExportConfig`].
#[allow(dead_code)]
pub fn default_hdr_config() -> HdrExportConfig {
    HdrExportConfig {
        exposure: 1.0,
        write_header: true,
        gamma: 2.2,
    }
}

/// Creates a new black [`HdrImage`] of the given dimensions.
#[allow(dead_code)]
pub fn new_hdr_image(width: u32, height: u32) -> HdrImage {
    let pixel_count = (width as usize) * (height as usize);
    HdrImage {
        width,
        height,
        pixels: vec![HdrPixel { r: 0.0, g: 0.0, b: 0.0 }; pixel_count],
    }
}

/// Sets the pixel at `(x, y)` to the given linear RGB values.
/// Out-of-bounds coordinates are silently ignored.
#[allow(dead_code)]
pub fn hdr_set_pixel(img: &mut HdrImage, x: u32, y: u32, r: f32, g: f32, b: f32) {
    if x >= img.width || y >= img.height {
        return;
    }
    let idx = (y as usize) * (img.width as usize) + (x as usize);
    img.pixels[idx] = HdrPixel { r, g, b };
}

/// Returns the pixel at `(x, y)`, or black if out-of-bounds.
#[allow(dead_code)]
pub fn hdr_get_pixel(img: &HdrImage, x: u32, y: u32) -> HdrPixel {
    if x >= img.width || y >= img.height {
        return HdrPixel { r: 0.0, g: 0.0, b: 0.0 };
    }
    let idx = (y as usize) * (img.width as usize) + (x as usize);
    img.pixels[idx]
}

/// Encodes every pixel as a 4-byte Radiance RGBE value and returns the flat
/// byte buffer (4 bytes per pixel, row-major).
#[allow(dead_code)]
pub fn hdr_to_rgbe_bytes(img: &HdrImage) -> Vec<u8> {
    let mut out = Vec::with_capacity(img.pixels.len() * 4);
    for px in &img.pixels {
        let rgbe = float_rgb_to_rgbe(px.r, px.g, px.b);
        out.extend_from_slice(&rgbe);
    }
    out
}

/// Writes a minimal HDR file stub to `path`.  Returns an error string on
/// failure.  The format is a raw RGBE byte dump preceded by a small ASCII
/// header comment.
#[allow(dead_code)]
pub fn hdr_write_to_file(img: &HdrImage, path: &str, _cfg: &HdrExportConfig) -> Result<(), String> {
    let mut data = Vec::new();
    // Minimal Radiance HDR header (stub)
    let header = format!(
        "#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y {} +X {}\n",
        img.height, img.width
    );
    data.extend_from_slice(header.as_bytes());
    data.extend_from_slice(&hdr_to_rgbe_bytes(img));
    std::fs::write(path, data).map_err(|e| e.to_string())
}

/// Returns the width of `img`.
#[allow(dead_code)]
pub fn hdr_image_width(img: &HdrImage) -> u32 {
    img.width
}

/// Returns the height of `img`.
#[allow(dead_code)]
pub fn hdr_image_height(img: &HdrImage) -> u32 {
    img.height
}

/// Applies Reinhard tone-mapping with the given `exposure` and returns an
/// 8-bit RGB triple for every pixel (suitable for PNG/BMP output).
#[allow(dead_code)]
pub fn hdr_tone_map_reinhard(img: &HdrImage, exposure: f32) -> Vec<[u8; 3]> {
    img.pixels
        .iter()
        .map(|px| {
            let r = reinhard(px.r * exposure);
            let g = reinhard(px.g * exposure);
            let b = reinhard(px.b * exposure);
            [
                (r * 255.0).clamp(0.0, 255.0) as u8,
                (g * 255.0).clamp(0.0, 255.0) as u8,
                (b * 255.0).clamp(0.0, 255.0) as u8,
            ]
        })
        .collect()
}

/// Computes the average luminance of `img` (using standard Rec. 709 weights).
#[allow(dead_code)]
pub fn hdr_average_luminance(img: &HdrImage) -> f32 {
    if img.pixels.is_empty() {
        return 0.0;
    }
    let sum: f32 = img.pixels.iter().map(luminance).sum();
    sum / img.pixels.len() as f32
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn reinhard(x: f32) -> f32 {
    x / (1.0 + x)
}

fn luminance(px: &HdrPixel) -> f32 {
    0.2126 * px.r + 0.7152 * px.g + 0.0722 * px.b
}

/// Converts linear float RGB to a 4-byte Radiance RGBE encoding.
fn float_rgb_to_rgbe(r: f32, g: f32, b: f32) -> [u8; 4] {
    let max_c = r.max(g).max(b);
    if max_c < 1e-32 {
        return [0, 0, 0, 0];
    }
    // frexp equivalent: find exponent e such that max_c = mantissa * 2^e
    let mut e = 0i32;
    let mut v = max_c;
    while v >= 1.0 {
        v *= 0.5;
        e += 1;
    }
    while v < 0.5 {
        v *= 2.0;
        e -= 1;
    }
    let scale = (v * 256.0 / max_c).max(0.0);
    let rb = (r * scale).clamp(0.0, 255.0) as u8;
    let gb = (g * scale).clamp(0.0, 255.0) as u8;
    let bb = (b * scale).clamp(0.0, 255.0) as u8;
    let exp = (e + 128).clamp(0, 255) as u8;
    [rb, gb, bb, exp]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_hdr_config();
        assert!((cfg.exposure - 1.0).abs() < 1e-6);
        assert!(cfg.write_header);
        assert!((cfg.gamma - 2.2).abs() < 1e-4);
    }

    #[test]
    fn new_image_dimensions() {
        let img = new_hdr_image(16, 8);
        assert_eq!(hdr_image_width(&img), 16);
        assert_eq!(hdr_image_height(&img), 8);
        assert_eq!(img.pixels.len(), 128);
    }

    #[test]
    fn set_and_get_pixel() {
        let mut img = new_hdr_image(4, 4);
        hdr_set_pixel(&mut img, 2, 1, 1.0, 0.5, 0.25);
        let px = hdr_get_pixel(&img, 2, 1);
        assert!((px.r - 1.0).abs() < 1e-6);
        assert!((px.g - 0.5).abs() < 1e-6);
        assert!((px.b - 0.25).abs() < 1e-6);
    }

    #[test]
    fn out_of_bounds_get_returns_black() {
        let img = new_hdr_image(2, 2);
        let px = hdr_get_pixel(&img, 99, 99);
        assert_eq!(px.r, 0.0);
        assert_eq!(px.g, 0.0);
        assert_eq!(px.b, 0.0);
    }

    #[test]
    fn rgbe_bytes_length() {
        let img = new_hdr_image(8, 8);
        let bytes = hdr_to_rgbe_bytes(&img);
        assert_eq!(bytes.len(), 8 * 8 * 4);
    }

    #[test]
    fn tone_map_reinhard_count() {
        let img = new_hdr_image(3, 3);
        let ldr = hdr_tone_map_reinhard(&img, 1.0);
        assert_eq!(ldr.len(), 9);
    }

    #[test]
    fn average_luminance_black_image() {
        let img = new_hdr_image(4, 4);
        assert!((hdr_average_luminance(&img)).abs() < 1e-6);
    }

    #[test]
    fn average_luminance_white_pixel() {
        let mut img = new_hdr_image(1, 1);
        hdr_set_pixel(&mut img, 0, 0, 1.0, 1.0, 1.0);
        let lum = hdr_average_luminance(&img);
        // 0.2126 + 0.7152 + 0.0722 = 1.0
        assert!((lum - 1.0).abs() < 1e-4);
    }

    #[test]
    fn write_to_file_creates_file() {
        let mut img = new_hdr_image(2, 2);
        hdr_set_pixel(&mut img, 0, 0, 2.0, 1.0, 0.5);
        let cfg = default_hdr_config();
        let path = "/tmp/oxihuman_hdr_export_test.hdr";
        assert!(hdr_write_to_file(&img, path, &cfg).is_ok());
        let meta = std::fs::metadata(path).expect("should succeed");
        assert!(meta.len() > 0);
        let _ = std::fs::remove_file(path);
    }
}
