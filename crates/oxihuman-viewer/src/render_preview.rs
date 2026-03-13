// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Render preview state (progressive rendering).

#![allow(dead_code)]

/// Progressive render preview state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPreview {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub current_sample: u32,
    pub pixels: Vec<[f32; 4]>,
    pub complete: bool,
}

/// Creates a new `RenderPreview` with all pixels set to black/transparent.
#[allow(dead_code)]
pub fn new_render_preview(w: u32, h: u32) -> RenderPreview {
    let count = (w * h) as usize;
    RenderPreview {
        width: w,
        height: h,
        samples: 64,
        current_sample: 0,
        pixels: vec![[0.0, 0.0, 0.0, 0.0]; count],
        complete: false,
    }
}

/// Returns rendering progress in [0, 1].
#[allow(dead_code)]
pub fn render_preview_progress(p: &RenderPreview) -> f32 {
    if p.samples == 0 {
        return 1.0;
    }
    (p.current_sample as f32 / p.samples as f32).min(1.0)
}

/// Advances the render by one sample, marking complete when done.
#[allow(dead_code)]
pub fn add_sample(p: &mut RenderPreview) {
    if p.current_sample < p.samples {
        p.current_sample += 1;
    }
    if p.current_sample >= p.samples {
        p.complete = true;
    }
}

/// Resets the preview to the initial state.
#[allow(dead_code)]
pub fn clear_preview(p: &mut RenderPreview) {
    p.current_sample = 0;
    p.complete = false;
    for px in p.pixels.iter_mut() {
        *px = [0.0, 0.0, 0.0, 0.0];
    }
}

/// Returns the pixel at (x, y) or a black pixel if out of bounds.
#[allow(dead_code)]
pub fn pixel_at(p: &RenderPreview, x: u32, y: u32) -> [f32; 4] {
    if x >= p.width || y >= p.height {
        return [0.0, 0.0, 0.0, 0.0];
    }
    let idx = (y * p.width + x) as usize;
    p.pixels[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_render_preview_dimensions() {
        let p = new_render_preview(4, 4);
        assert_eq!(p.width, 4);
        assert_eq!(p.height, 4);
        assert_eq!(p.pixels.len(), 16);
    }

    #[test]
    fn test_initial_progress_zero() {
        let p = new_render_preview(4, 4);
        assert!((render_preview_progress(&p) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_add_sample_advances() {
        let mut p = new_render_preview(4, 4);
        add_sample(&mut p);
        assert_eq!(p.current_sample, 1);
    }

    #[test]
    fn test_complete_after_all_samples() {
        let mut p = new_render_preview(2, 2);
        p.samples = 3;
        add_sample(&mut p);
        add_sample(&mut p);
        assert!(!p.complete);
        add_sample(&mut p);
        assert!(p.complete);
    }

    #[test]
    fn test_add_sample_no_overflow() {
        let mut p = new_render_preview(1, 1);
        p.samples = 1;
        add_sample(&mut p);
        add_sample(&mut p); // should not overflow
        assert_eq!(p.current_sample, 1);
    }

    #[test]
    fn test_clear_preview_resets() {
        let mut p = new_render_preview(2, 2);
        p.current_sample = 10;
        p.complete = true;
        clear_preview(&mut p);
        assert_eq!(p.current_sample, 0);
        assert!(!p.complete);
    }

    #[test]
    fn test_pixel_at_default_black() {
        let p = new_render_preview(4, 4);
        let px = pixel_at(&p, 0, 0);
        assert!((px[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_pixel_at_out_of_bounds() {
        let p = new_render_preview(4, 4);
        let px = pixel_at(&p, 10, 10);
        assert!((px[3] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_progress_capped_at_one() {
        let mut p = new_render_preview(1, 1);
        p.samples = 0;
        assert!((render_preview_progress(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_progress_at_half() {
        let mut p = new_render_preview(1, 1);
        p.samples = 10;
        p.current_sample = 5;
        assert!((render_preview_progress(&p) - 0.5).abs() < 1e-6);
    }
}
