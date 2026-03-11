// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! GIF animation stub export.

/// A single GIF frame (RGBA pixel data).
#[derive(Debug, Clone)]
pub struct GifFrame {
    pub width: u32,
    pub height: u32,
    pub delay_cs: u16,
    pub pixels: Vec<[u8; 4]>,
}

impl GifFrame {
    /// Create a new GIF frame filled with a solid color.
    pub fn new_solid(width: u32, height: u32, delay_cs: u16, color: [u8; 4]) -> Self {
        let pixels = vec![color; (width * height) as usize];
        Self {
            width,
            height,
            delay_cs,
            pixels,
        }
    }

    /// Pixel count.
    pub fn pixel_count(&self) -> usize {
        self.pixels.len()
    }

    /// Frame duration in milliseconds.
    pub fn delay_ms(&self) -> u32 {
        self.delay_cs as u32 * 10
    }
}

/// GIF animation stub.
#[derive(Debug, Clone)]
pub struct GifExport {
    pub width: u32,
    pub height: u32,
    pub loop_count: u16,
    pub frames: Vec<GifFrame>,
}

impl GifExport {
    /// Create a new GIF export.
    pub fn new(width: u32, height: u32, loop_count: u16) -> Self {
        Self {
            width,
            height,
            loop_count,
            frames: Vec::new(),
        }
    }

    /// Add a frame.
    pub fn add_frame(&mut self, frame: GifFrame) {
        self.frames.push(frame);
    }

    /// Return frame count.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Total animation duration in milliseconds.
    pub fn total_duration_ms(&self) -> u32 {
        self.frames.iter().map(|f| f.delay_ms()).sum()
    }
}

/// Estimate GIF file size in bytes (rough stub heuristic).
pub fn estimate_gif_size(gif: &GifExport) -> usize {
    /* header + per-frame estimated compressed size */
    6 + gif
        .frames
        .iter()
        .map(|f| f.pixel_count() / 2 + 20)
        .sum::<usize>()
}

/// Validate that all frames have matching dimensions.
pub fn validate_gif(gif: &GifExport) -> bool {
    gif.frames
        .iter()
        .all(|f| f.width == gif.width && f.height == gif.height)
}

/// Serialize GIF metadata to a JSON string (stub).
pub fn gif_metadata_json(gif: &GifExport) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"frames\":{},\"loop\":{}}}",
        gif.width,
        gif.height,
        gif.frame_count(),
        gif.loop_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_gif() -> GifExport {
        let mut gif = GifExport::new(16, 16, 0);
        gif.add_frame(GifFrame::new_solid(16, 16, 10, [255, 0, 0, 255]));
        gif.add_frame(GifFrame::new_solid(16, 16, 20, [0, 255, 0, 255]));
        gif
    }

    #[test]
    fn test_frame_count() {
        /* frame count is correct */
        let g = sample_gif();
        assert_eq!(g.frame_count(), 2);
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count is width * height */
        let f = GifFrame::new_solid(4, 4, 5, [0; 4]);
        assert_eq!(f.pixel_count(), 16);
    }

    #[test]
    fn test_delay_ms() {
        /* delay in ms is cs * 10 */
        let f = GifFrame::new_solid(4, 4, 10, [0; 4]);
        assert_eq!(f.delay_ms(), 100);
    }

    #[test]
    fn test_total_duration_ms() {
        /* total duration sums frame delays */
        let g = sample_gif();
        assert_eq!(g.total_duration_ms(), 300);
    }

    #[test]
    fn test_validate_gif_valid() {
        /* valid frames pass validation */
        let g = sample_gif();
        assert!(validate_gif(&g));
    }

    #[test]
    fn test_validate_gif_invalid() {
        /* mismatched frame dimensions fail validation */
        let mut g = GifExport::new(16, 16, 0);
        g.add_frame(GifFrame::new_solid(8, 8, 5, [0; 4]));
        assert!(!validate_gif(&g));
    }

    #[test]
    fn test_estimate_gif_size_positive() {
        /* estimated size is positive */
        let g = sample_gif();
        assert!(estimate_gif_size(&g) > 0);
    }

    #[test]
    fn test_metadata_json() {
        /* metadata JSON contains width */
        let g = sample_gif();
        let json = gif_metadata_json(&g);
        assert!(json.contains("16"));
    }
}
