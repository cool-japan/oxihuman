// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Offscreen render target (framebuffer) abstraction.

#![allow(dead_code)]

// ── RenderTargetFormat ────────────────────────────────────────────────────────

/// Pixel / depth format for a render target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderTargetFormat {
    /// 8-bit RGBA.
    Rgba8,
    /// 16-bit float RGBA.
    Rgba16F,
    /// 32-bit float single channel.
    R32F,
    /// 24-bit depth.
    Depth24,
}

// ── RenderTargetConfig ────────────────────────────────────────────────────────

/// Configuration used when creating a render target.
#[derive(Debug, Clone)]
pub struct RenderTargetConfig {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel format.
    pub format: RenderTargetFormat,
    /// MSAA sample count (1 = no MSAA).
    pub msaa_samples: u32,
    /// Whether to auto-generate mipmaps after rendering.
    pub generate_mipmaps: bool,
}

// ── RenderTarget ──────────────────────────────────────────────────────────────

/// An offscreen render target (framebuffer) descriptor.
///
/// In a full GPU backend `texture_id` and `framebuffer_id` would be
/// driver-allocated handles; here they are stub values.
#[derive(Debug, Clone)]
pub struct RenderTarget {
    /// Configuration this render target was created with.
    pub config: RenderTargetConfig,
    /// Stub texture handle (GPU id would be allocated at runtime).
    pub texture_id: u32,
    /// Stub framebuffer handle.
    pub framebuffer_id: u32,
    /// Whether the render target is in a valid / usable state.
    pub is_valid: bool,
}

// ── BlitConfig ────────────────────────────────────────────────────────────────

/// Configuration for a blit (framebuffer copy) operation.
#[derive(Debug, Clone)]
pub struct BlitConfig {
    /// Source X offset.
    pub src_x: u32,
    /// Source Y offset.
    pub src_y: u32,
    /// Destination X offset.
    pub dst_x: u32,
    /// Destination Y offset.
    pub dst_y: u32,
    /// Blit width in pixels.
    pub width: u32,
    /// Blit height in pixels.
    pub height: u32,
}

// ── Constructor / utility functions ──────────────────────────────────────────

/// Return a default `RenderTargetConfig` for the given dimensions.
///
/// Format defaults to `Rgba8`, MSAA off, no mipmaps.
#[allow(dead_code)]
pub fn default_render_target_config(width: u32, height: u32) -> RenderTargetConfig {
    RenderTargetConfig {
        width,
        height,
        format: RenderTargetFormat::Rgba8,
        msaa_samples: 1,
        generate_mipmaps: false,
    }
}

/// Allocate (stub) a render target from `cfg`.
///
/// `texture_id` and `framebuffer_id` are set to `1` as placeholder handles.
#[allow(dead_code)]
pub fn new_render_target(cfg: RenderTargetConfig) -> RenderTarget {
    RenderTarget {
        config: cfg,
        texture_id: 1,
        framebuffer_id: 1,
        is_valid: true,
    }
}

/// Return the width of a render target.
#[allow(dead_code)]
pub fn render_target_width(rt: &RenderTarget) -> u32 {
    rt.config.width
}

/// Return the height of a render target.
#[allow(dead_code)]
pub fn render_target_height(rt: &RenderTarget) -> u32 {
    rt.config.height
}

/// Return a human-readable name for the render target's pixel format.
#[allow(dead_code)]
pub fn render_target_format_name(rt: &RenderTarget) -> &'static str {
    match rt.config.format {
        RenderTargetFormat::Rgba8 => "RGBA8",
        RenderTargetFormat::Rgba16F => "RGBA16F",
        RenderTargetFormat::R32F => "R32F",
        RenderTargetFormat::Depth24 => "DEPTH24",
    }
}

/// Return the total number of pixels (width × height).
#[allow(dead_code)]
pub fn render_target_pixel_count(rt: &RenderTarget) -> u32 {
    rt.config.width * rt.config.height
}

/// Estimate the byte size of the render target's texture data.
#[allow(dead_code)]
pub fn render_target_byte_size(rt: &RenderTarget) -> usize {
    let bpp = format_bytes_per_pixel(&rt.config.format) as usize;
    rt.config.width as usize * rt.config.height as usize * bpp
}

/// Mark the render target as invalid (e.g. after the underlying context is lost).
#[allow(dead_code)]
pub fn invalidate_render_target(rt: &mut RenderTarget) {
    rt.is_valid = false;
    rt.texture_id = 0;
    rt.framebuffer_id = 0;
}

/// Resize the render target (stub: updates dimensions, resets to valid).
#[allow(dead_code)]
pub fn resize_render_target(rt: &mut RenderTarget, w: u32, h: u32) {
    rt.config.width = w;
    rt.config.height = h;
    rt.is_valid = true;
}

/// Serialise the render target descriptor to a JSON-like string.
#[allow(dead_code)]
pub fn render_target_to_json(rt: &RenderTarget) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"format\":\"{}\",\"msaa\":{},\
         \"mipmaps\":{},\"valid\":{}}}",
        rt.config.width,
        rt.config.height,
        render_target_format_name(rt),
        rt.config.msaa_samples,
        rt.config.generate_mipmaps,
        rt.is_valid
    )
}

/// Build a `BlitConfig`.
#[allow(dead_code)]
pub fn new_blit_config(
    src_x: u32,
    src_y: u32,
    dst_x: u32,
    dst_y: u32,
    w: u32,
    h: u32,
) -> BlitConfig {
    BlitConfig {
        src_x,
        src_y,
        dst_x,
        dst_y,
        width: w,
        height: h,
    }
}

/// Return the area covered by the blit region (width × height).
#[allow(dead_code)]
pub fn blit_area(cfg: &BlitConfig) -> u32 {
    cfg.width * cfg.height
}

/// Return the number of bytes per pixel for a given format.
#[allow(dead_code)]
pub fn format_bytes_per_pixel(fmt: &RenderTargetFormat) -> u32 {
    match fmt {
        RenderTargetFormat::Rgba8 => 4,
        RenderTargetFormat::Rgba16F => 8,
        RenderTargetFormat::R32F => 4,
        RenderTargetFormat::Depth24 => 4,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // 1 – default_render_target_config
    #[test]
    fn test_default_render_target_config() {
        let cfg = default_render_target_config(1920, 1080);
        assert_eq!(cfg.width, 1920);
        assert_eq!(cfg.height, 1080);
        assert_eq!(cfg.format, RenderTargetFormat::Rgba8);
        assert_eq!(cfg.msaa_samples, 1);
        assert!(!cfg.generate_mipmaps);
    }

    // 2 – new_render_target is valid
    #[test]
    fn test_new_render_target_valid() {
        let cfg = default_render_target_config(512, 512);
        let rt = new_render_target(cfg);
        assert!(rt.is_valid);
        assert_ne!(rt.texture_id, 0);
        assert_ne!(rt.framebuffer_id, 0);
    }

    // 3 – render_target_width / height
    #[test]
    fn test_render_target_dimensions() {
        let rt = new_render_target(default_render_target_config(800, 600));
        assert_eq!(render_target_width(&rt), 800);
        assert_eq!(render_target_height(&rt), 600);
    }

    // 4 – render_target_format_name
    #[test]
    fn test_render_target_format_names() {
        let mut cfg = default_render_target_config(1, 1);
        cfg.format = RenderTargetFormat::Rgba8;
        assert_eq!(render_target_format_name(&new_render_target(cfg.clone())), "RGBA8");
        cfg.format = RenderTargetFormat::Rgba16F;
        assert_eq!(render_target_format_name(&new_render_target(cfg.clone())), "RGBA16F");
        cfg.format = RenderTargetFormat::R32F;
        assert_eq!(render_target_format_name(&new_render_target(cfg.clone())), "R32F");
        cfg.format = RenderTargetFormat::Depth24;
        assert_eq!(render_target_format_name(&new_render_target(cfg)), "DEPTH24");
    }

    // 5 – render_target_pixel_count
    #[test]
    fn test_render_target_pixel_count() {
        let rt = new_render_target(default_render_target_config(100, 50));
        assert_eq!(render_target_pixel_count(&rt), 5000);
    }

    // 6 – render_target_byte_size RGBA8
    #[test]
    fn test_render_target_byte_size_rgba8() {
        let rt = new_render_target(default_render_target_config(100, 100));
        assert_eq!(render_target_byte_size(&rt), 100 * 100 * 4);
    }

    // 7 – render_target_byte_size RGBA16F
    #[test]
    fn test_render_target_byte_size_rgba16f() {
        let mut cfg = default_render_target_config(64, 64);
        cfg.format = RenderTargetFormat::Rgba16F;
        let rt = new_render_target(cfg);
        assert_eq!(render_target_byte_size(&rt), 64 * 64 * 8);
    }

    // 8 – invalidate_render_target
    #[test]
    fn test_invalidate_render_target() {
        let mut rt = new_render_target(default_render_target_config(256, 256));
        invalidate_render_target(&mut rt);
        assert!(!rt.is_valid);
        assert_eq!(rt.texture_id, 0);
        assert_eq!(rt.framebuffer_id, 0);
    }

    // 9 – resize_render_target
    #[test]
    fn test_resize_render_target() {
        let mut rt = new_render_target(default_render_target_config(100, 100));
        resize_render_target(&mut rt, 200, 150);
        assert_eq!(render_target_width(&rt), 200);
        assert_eq!(render_target_height(&rt), 150);
        assert!(rt.is_valid);
    }

    // 10 – render_target_to_json contains expected keys
    #[test]
    fn test_render_target_to_json() {
        let rt = new_render_target(default_render_target_config(320, 240));
        let json = render_target_to_json(&rt);
        assert!(json.contains("\"width\":320"));
        assert!(json.contains("\"height\":240"));
        assert!(json.contains("\"format\":\"RGBA8\""));
        assert!(json.contains("\"valid\":true"));
    }

    // 11 – new_blit_config / blit_area
    #[test]
    fn test_blit_config_area() {
        let blit = new_blit_config(0, 0, 10, 10, 64, 32);
        assert_eq!(blit_area(&blit), 64 * 32);
        assert_eq!(blit.src_x, 0);
        assert_eq!(blit.dst_x, 10);
    }

    // 12 – format_bytes_per_pixel
    #[test]
    fn test_format_bytes_per_pixel() {
        assert_eq!(format_bytes_per_pixel(&RenderTargetFormat::Rgba8), 4);
        assert_eq!(format_bytes_per_pixel(&RenderTargetFormat::Rgba16F), 8);
        assert_eq!(format_bytes_per_pixel(&RenderTargetFormat::R32F), 4);
        assert_eq!(format_bytes_per_pixel(&RenderTargetFormat::Depth24), 4);
    }

    // 13 – resize then invalidate
    #[test]
    fn test_resize_then_invalidate() {
        let mut rt = new_render_target(default_render_target_config(10, 10));
        resize_render_target(&mut rt, 20, 20);
        assert!(rt.is_valid);
        invalidate_render_target(&mut rt);
        assert!(!rt.is_valid);
        assert_eq!(render_target_width(&rt), 20);
    }
}
