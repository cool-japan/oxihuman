// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Deep image export (per-pixel depth sample lists).

/// A single depth sample at a pixel.
#[derive(Debug, Clone)]
pub struct DeepPixel {
    pub depth: f32,
    pub alpha: f32,
    pub color: [f32; 3],
}

/// A 2D deep image with variable-length sample lists per pixel.
#[derive(Debug, Clone)]
pub struct DeepImage {
    pub width: usize,
    pub height: usize,
    pub samples: Vec<Vec<DeepPixel>>,
}

/// Create a new `DeepPixel`.
pub fn new_deep_pixel(depth: f32, alpha: f32, color: [f32; 3]) -> DeepPixel {
    DeepPixel {
        depth,
        alpha,
        color,
    }
}

/// Push a sample into a pixel's sample list.
pub fn deep_pixel_push(samples: &mut Vec<DeepPixel>, pixel: DeepPixel) {
    samples.push(pixel);
}

/// Flatten (composite) a pixel's samples into a single RGBA value (front-to-back).
pub fn deep_pixel_flatten(samples: &[DeepPixel]) -> [f32; 4] {
    let mut out = [0.0f32; 4];
    for s in samples.iter() {
        let remaining = 1.0 - out[3];
        out[0] += s.color[0] * s.alpha * remaining;
        out[1] += s.color[1] * s.alpha * remaining;
        out[2] += s.color[2] * s.alpha * remaining;
        out[3] += s.alpha * remaining;
        if out[3] >= 1.0 - f32::EPSILON {
            break;
        }
    }
    out
}

/// Create a new zeroed `DeepImage`.
pub fn new_deep_image(width: usize, height: usize) -> DeepImage {
    DeepImage {
        width,
        height,
        samples: vec![Vec::new(); width * height],
    }
}

/// Total sample count across all pixels.
pub fn deep_image_sample_count(img: &DeepImage) -> usize {
    img.samples.iter().map(|s| s.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_deep_pixel() {
        let p = new_deep_pixel(1.0, 0.8, [1.0, 0.0, 0.0]);
        assert!((p.depth - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_deep_pixel_push() {
        let mut s: Vec<DeepPixel> = Vec::new();
        deep_pixel_push(&mut s, new_deep_pixel(1.0, 1.0, [1.0, 1.0, 1.0]));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_deep_pixel_flatten_opaque() {
        /* single fully opaque red sample → red output */
        let s = vec![new_deep_pixel(1.0, 1.0, [1.0, 0.0, 0.0])];
        let rgba = deep_pixel_flatten(&s);
        assert!((rgba[0] - 1.0).abs() < 1e-5);
        assert!((rgba[3] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_new_deep_image() {
        let img = new_deep_image(4, 4);
        assert_eq!(img.samples.len(), 16);
    }

    #[test]
    fn test_deep_image_sample_count() {
        let mut img = new_deep_image(2, 2);
        img.samples[0].push(new_deep_pixel(1.0, 0.5, [1.0, 1.0, 1.0]));
        assert_eq!(deep_image_sample_count(&img), 1);
    }
}
