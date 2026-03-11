//! Texture preview renderer stub — metadata and sampling for previewing textures.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for texture preview rendering.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TexturePreviewConfig {
    /// Background color shown outside the texture bounds (RGBA).
    pub background: [f32; 4],
    /// Maximum zoom factor allowed.
    pub max_zoom: f32,
    /// Minimum zoom factor allowed.
    pub min_zoom: f32,
    /// Whether to generate mip levels automatically.
    pub auto_mips: bool,
}

/// Texel (pixel) format of the preview texture.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TexelFormat {
    /// 8-bit RGBA.
    Rgba8,
    /// 8-bit RGB.
    Rgb8,
    /// 8-bit single channel (greyscale).
    Grayscale8,
    /// 32-bit float HDR (4 channels).
    Hdr,
}

/// Stub representation of a texture preview (metadata + sampling).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TexturePreview {
    /// Width in texels.
    pub width: u32,
    /// Height in texels.
    pub height: u32,
    /// Texel format.
    pub format: TexelFormat,
    /// Number of mip levels (1 = no mips).
    pub mip_count: u32,
    /// Current zoom factor.
    pub zoom: f32,
    /// Flat RGBA pixel data (width × height × 4 × f32). Used for sampling.
    pub pixels: Vec<f32>,
}

// ── Construction ──────────────────────────────────────────────────────────────

/// Returns a default [`TexturePreviewConfig`].
#[allow(dead_code)]
pub fn default_texture_preview_config() -> TexturePreviewConfig {
    TexturePreviewConfig {
        background: [0.1, 0.1, 0.1, 1.0],
        max_zoom: 16.0,
        min_zoom: 0.0625,
        auto_mips: true,
    }
}

/// Creates a new [`TexturePreview`] with the given dimensions and format.
/// Pixel data is initialised to zero (transparent black).
#[allow(dead_code)]
pub fn new_texture_preview(width: u32, height: u32, format: TexelFormat) -> TexturePreview {
    let mip_count = if width > 0 && height > 0 {
        // floor(log2(max(w, h))) + 1
        let max_dim = width.max(height);
        (max_dim as f32).log2().floor() as u32 + 1
    } else {
        1
    };
    let pixel_count = (width as usize) * (height as usize) * 4;
    TexturePreview {
        width,
        height,
        format,
        mip_count,
        zoom: 1.0,
        pixels: vec![0.0f32; pixel_count],
    }
}

// ── Accessors ─────────────────────────────────────────────────────────────────

/// Returns the width of the preview texture.
#[allow(dead_code)]
pub fn texture_preview_width(preview: &TexturePreview) -> u32 {
    preview.width
}

/// Returns the height of the preview texture.
#[allow(dead_code)]
pub fn texture_preview_height(preview: &TexturePreview) -> u32 {
    preview.height
}

/// Returns the texel format.
#[allow(dead_code)]
pub fn texture_preview_format(preview: &TexturePreview) -> TexelFormat {
    preview.format
}

/// Returns the number of mip levels.
#[allow(dead_code)]
pub fn texture_preview_mip_count(preview: &TexturePreview) -> u32 {
    preview.mip_count
}

/// Sets the zoom factor, clamped to \[min_zoom, max_zoom\] (hardcoded 0.0625–16.0).
#[allow(dead_code)]
pub fn set_texture_preview_zoom(preview: &mut TexturePreview, zoom: f32) {
    preview.zoom = zoom.clamp(0.0625, 16.0);
}

// ── Sampling ──────────────────────────────────────────────────────────────────

/// Samples the preview at normalised UV coordinates (u, v ∈ \[0, 1\]).
/// Returns \[r, g, b, a\] using nearest-neighbour sampling.
/// Out-of-range UVs return `[0, 0, 0, 0]`.
#[allow(dead_code)]
pub fn texture_preview_sample(preview: &TexturePreview, u: f32, v: f32) -> [f32; 4] {
    if preview.width == 0 || preview.height == 0 || preview.pixels.is_empty() {
        return [0.0; 4];
    }
    let px = (u * preview.width as f32).floor() as i32;
    let py = (v * preview.height as f32).floor() as i32;
    if px < 0 || px >= preview.width as i32 || py < 0 || py >= preview.height as i32 {
        return [0.0; 4];
    }
    let idx = (py as usize * preview.width as usize + px as usize) * 4;
    if idx + 3 >= preview.pixels.len() {
        return [0.0; 4];
    }
    [
        preview.pixels[idx],
        preview.pixels[idx + 1],
        preview.pixels[idx + 2],
        preview.pixels[idx + 3],
    ]
}

// ── Format helpers ────────────────────────────────────────────────────────────

/// Returns a human-readable name for the texel format.
#[allow(dead_code)]
pub fn texel_format_name(fmt: TexelFormat) -> &'static str {
    match fmt {
        TexelFormat::Rgba8 => "RGBA8",
        TexelFormat::Rgb8 => "RGB8",
        TexelFormat::Grayscale8 => "Grayscale8",
        TexelFormat::Hdr => "HDR",
    }
}

/// Returns the number of channels for the given texel format.
#[allow(dead_code)]
pub fn texel_format_channels(fmt: TexelFormat) -> u32 {
    match fmt {
        TexelFormat::Rgba8 => 4,
        TexelFormat::Rgb8 => 3,
        TexelFormat::Grayscale8 => 1,
        TexelFormat::Hdr => 4,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_texture_preview_config();
        assert!(cfg.auto_mips);
        assert!((cfg.max_zoom - 16.0).abs() < 1e-6);
        assert!((cfg.min_zoom - 0.0625).abs() < 1e-6);
    }

    #[test]
    fn test_new_preview_dimensions() {
        let p = new_texture_preview(64, 64, TexelFormat::Rgba8);
        assert_eq!(texture_preview_width(&p), 64);
        assert_eq!(texture_preview_height(&p), 64);
    }

    #[test]
    fn test_new_preview_mip_count() {
        // 64×64 → log2(64)+1 = 7
        let p = new_texture_preview(64, 64, TexelFormat::Rgba8);
        assert_eq!(texture_preview_mip_count(&p), 7);
    }

    #[test]
    fn test_new_preview_format() {
        let p = new_texture_preview(32, 32, TexelFormat::Hdr);
        assert_eq!(texture_preview_format(&p), TexelFormat::Hdr);
    }

    #[test]
    fn test_set_zoom_clamp() {
        let mut p = new_texture_preview(32, 32, TexelFormat::Rgba8);
        set_texture_preview_zoom(&mut p, 100.0);
        assert!((p.zoom - 16.0).abs() < 1e-6);
        set_texture_preview_zoom(&mut p, 0.0);
        assert!((p.zoom - 0.0625).abs() < 1e-6);
    }

    #[test]
    fn test_sample_out_of_range() {
        let p = new_texture_preview(4, 4, TexelFormat::Rgba8);
        let sample = texture_preview_sample(&p, 1.5, 1.5);
        assert_eq!(sample, [0.0; 4]);
    }

    #[test]
    fn test_sample_zero_pixels() {
        let p = new_texture_preview(2, 2, TexelFormat::Rgba8);
        // All pixels are zero (transparent black)
        let sample = texture_preview_sample(&p, 0.1, 0.1);
        assert_eq!(sample, [0.0; 4]);
    }

    #[test]
    fn test_texel_format_name() {
        assert_eq!(texel_format_name(TexelFormat::Rgba8), "RGBA8");
        assert_eq!(texel_format_name(TexelFormat::Rgb8), "RGB8");
        assert_eq!(texel_format_name(TexelFormat::Grayscale8), "Grayscale8");
        assert_eq!(texel_format_name(TexelFormat::Hdr), "HDR");
    }

    #[test]
    fn test_texel_format_channels() {
        assert_eq!(texel_format_channels(TexelFormat::Rgba8), 4);
        assert_eq!(texel_format_channels(TexelFormat::Rgb8), 3);
        assert_eq!(texel_format_channels(TexelFormat::Grayscale8), 1);
        assert_eq!(texel_format_channels(TexelFormat::Hdr), 4);
    }
}
