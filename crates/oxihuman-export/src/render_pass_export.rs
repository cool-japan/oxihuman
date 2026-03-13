// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export render pass configuration.

/* ── legacy API (kept) ── */

#[derive(Debug, Clone)]
pub struct RenderPassEntry {
    pub name: String,
    pub pass_type: String,
    pub enabled: bool,
    pub samples: u32,
}

#[derive(Debug, Clone, Default)]
pub struct RenderPassExport {
    pub passes: Vec<RenderPassEntry>,
}

pub fn new_render_pass_export() -> RenderPassExport {
    RenderPassExport { passes: Vec::new() }
}

pub fn add_render_pass(exp: &mut RenderPassExport, name: &str, type_: &str, samples: u32) {
    exp.passes.push(RenderPassEntry {
        name: name.to_string(),
        pass_type: type_.to_string(),
        enabled: true,
        samples,
    });
}

/* ── spec functions (wave 150B) ── */

/// Spec-style render pass (a 2D pixel buffer).
#[derive(Debug, Clone)]
pub struct RenderPass {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<f32>,
}

/// Create a new zeroed `RenderPass`.
pub fn new_render_pass(name: &str, width: usize, height: usize) -> RenderPass {
    RenderPass {
        name: name.to_string(),
        width,
        height,
        pixels: vec![0.0; width * height],
    }
}

/// Set a pixel value at (x, y).
pub fn pass_set_pixel(pass: &mut RenderPass, x: usize, y: usize, value: f32) {
    if x < pass.width && y < pass.height {
        pass.pixels[y * pass.width + x] = value;
    }
}

/// Get a pixel value at (x, y).
pub fn pass_get_pixel(pass: &RenderPass, x: usize, y: usize) -> f32 {
    if x < pass.width && y < pass.height {
        pass.pixels[y * pass.width + x]
    } else {
        0.0
    }
}

/// Return pixel data as bytes (f32 → 4 bytes each, little-endian).
pub fn pass_to_bytes(pass: &RenderPass) -> Vec<u8> {
    pass.pixels.iter().flat_map(|&f| f.to_le_bytes()).collect()
}

/// Mean pixel value.
pub fn pass_mean(pass: &RenderPass) -> f32 {
    if pass.pixels.is_empty() {
        return 0.0;
    }
    pass.pixels.iter().sum::<f32>() / pass.pixels.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_render_pass() {
        let p = new_render_pass("beauty", 4, 4);
        assert_eq!(p.pixels.len(), 16);
    }

    #[test]
    fn test_set_get_pixel() {
        let mut p = new_render_pass("r", 4, 4);
        pass_set_pixel(&mut p, 1, 2, 0.5);
        assert!((pass_get_pixel(&p, 1, 2) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_pass_to_bytes_len() {
        let p = new_render_pass("r", 2, 2);
        assert_eq!(pass_to_bytes(&p).len(), 16);
    }

    #[test]
    fn test_pass_mean_zero() {
        let p = new_render_pass("r", 2, 2);
        assert!((pass_mean(&p)).abs() < 1e-5);
    }

    #[test]
    fn test_pass_mean_nonzero() {
        let mut p = new_render_pass("r", 2, 1);
        pass_set_pixel(&mut p, 0, 0, 1.0);
        pass_set_pixel(&mut p, 1, 0, 1.0);
        assert!((pass_mean(&p) - 1.0).abs() < 1e-5);
    }
}
