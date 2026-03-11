//! Render export configuration and frame utilities.
#![allow(dead_code)]

/// Configuration for rendering export.
#[allow(dead_code)]
pub struct RenderExportConfig2 {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
}

/// A rendered frame.
#[allow(dead_code)]
pub struct RenderFrame2 {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Create a new render frame filled with zeros.
#[allow(dead_code)]
pub fn new_render_frame2(width: u32, height: u32) -> RenderFrame2 {
    RenderFrame2 {
        pixels: vec![0u8; (width * height * 4) as usize],
        width,
        height,
    }
}

/// Serialize frame to raw RGBA bytes.
#[allow(dead_code)]
pub fn render_frame2_to_bytes(frame: &RenderFrame2) -> &[u8] {
    &frame.pixels
}

/// Default render export config.
#[allow(dead_code)]
pub fn default_render_export_config2() -> RenderExportConfig2 {
    RenderExportConfig2 { width: 1920, height: 1080, samples: 64 }
}

/// Get render width.
#[allow(dead_code)]
pub fn render2_width(frame: &RenderFrame2) -> u32 { frame.width }

/// Get render height.
#[allow(dead_code)]
pub fn render2_height(frame: &RenderFrame2) -> u32 { frame.height }

/// Get total pixel count.
#[allow(dead_code)]
pub fn pixel2_count(frame: &RenderFrame2) -> u32 { frame.width * frame.height }

/// Convert frame pixels to RGBA vec.
#[allow(dead_code)]
pub fn frame2_to_rgba(frame: &RenderFrame2) -> Vec<[u8; 4]> {
    frame.pixels.chunks(4).map(|c| [c[0],c[1],c[2],c[3]]).collect()
}

/// Convert frame to grayscale (average of RGB channels).
#[allow(dead_code)]
pub fn frame2_to_grayscale(frame: &RenderFrame2) -> Vec<u8> {
    frame.pixels.chunks(4).map(|c| {
        ((c[0] as u16 + c[1] as u16 + c[2] as u16) / 3) as u8
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_render_frame_size() {
        let f = new_render_frame2(4, 4);
        assert_eq!(f.pixels.len(), 64);
    }

    #[test]
    fn test_render_width() {
        let f = new_render_frame2(10, 5);
        assert_eq!(render2_width(&f), 10);
    }

    #[test]
    fn test_render_height() {
        let f = new_render_frame2(10, 5);
        assert_eq!(render2_height(&f), 5);
    }

    #[test]
    fn test_pixel_count() {
        let f = new_render_frame2(3, 4);
        assert_eq!(pixel2_count(&f), 12);
    }

    #[test]
    fn test_frame_to_rgba() {
        let f = new_render_frame2(2, 2);
        let rgba = frame2_to_rgba(&f);
        assert_eq!(rgba.len(), 4);
    }

    #[test]
    fn test_frame_to_grayscale() {
        let f = new_render_frame2(2, 2);
        let gs = frame2_to_grayscale(&f);
        assert_eq!(gs.len(), 4);
    }

    #[test]
    fn test_render_frame_to_bytes() {
        let f = new_render_frame2(2, 2);
        let b = render_frame2_to_bytes(&f);
        assert_eq!(b.len(), 16);
    }

    #[test]
    fn test_default_config_width() {
        let c = default_render_export_config2();
        assert_eq!(c.width, 1920);
    }

    #[test]
    fn test_default_config_samples() {
        let c = default_render_export_config2();
        assert_eq!(c.samples, 64);
    }
}
