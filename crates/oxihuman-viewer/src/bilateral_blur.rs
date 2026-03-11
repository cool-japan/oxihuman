// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bilateral blur filter — edge-preserving Gaussian blur using
//! spatial and range (intensity) kernels.

use std::f32::consts::PI;

/// Bilateral blur configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BilateralConfig {
    /// Spatial sigma (pixel radius).
    pub sigma_spatial: f32,
    /// Range sigma (intensity difference).
    pub sigma_range: f32,
    /// Kernel half-size in pixels.
    pub kernel_radius: u32,
}

impl Default for BilateralConfig {
    fn default() -> Self {
        Self {
            sigma_spatial: 3.0,
            sigma_range: 0.1,
            kernel_radius: 5,
        }
    }
}

/// Gaussian weight.
#[allow(dead_code)]
pub fn gaussian(x: f32, sigma: f32) -> f32 {
    if sigma.abs() < 1e-8 {
        return if x.abs() < 1e-8 { 1.0 } else { 0.0 };
    }
    (-0.5 * (x / sigma).powi(2)).exp()
}

/// Normalised 1D Gaussian.
#[allow(dead_code)]
pub fn gaussian_normalized(x: f32, sigma: f32) -> f32 {
    if sigma.abs() < 1e-8 {
        return if x.abs() < 1e-8 { 1.0 } else { 0.0 };
    }
    let norm = 1.0 / ((2.0 * PI).sqrt() * sigma);
    norm * (-0.5 * (x / sigma).powi(2)).exp()
}

/// Bilateral filter for a single pixel in a greyscale image.
///
/// `image` is row-major, `width x height`.
#[allow(dead_code)]
pub fn bilateral_filter_pixel(
    image: &[f32],
    width: u32,
    height: u32,
    px: u32,
    py: u32,
    config: &BilateralConfig,
) -> f32 {
    let idx = py as usize * width as usize + px as usize;
    if idx >= image.len() {
        return 0.0;
    }
    let centre = image[idx];
    let r = config.kernel_radius as i32;

    let mut total_weight = 0.0_f32;
    let mut total_value = 0.0_f32;

    for dy in -r..=r {
        for dx in -r..=r {
            let nx = px as i32 + dx;
            let ny = py as i32 + dy;
            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                continue;
            }
            let ni = ny as usize * width as usize + nx as usize;
            if ni >= image.len() {
                continue;
            }
            let neighbour = image[ni];
            let spatial_dist = ((dx * dx + dy * dy) as f32).sqrt();
            let range_dist = (neighbour - centre).abs();

            let ws = gaussian(spatial_dist, config.sigma_spatial);
            let wr = gaussian(range_dist, config.sigma_range);
            let w = ws * wr;

            total_weight += w;
            total_value += w * neighbour;
        }
    }

    if total_weight > 1e-8 {
        total_value / total_weight
    } else {
        centre
    }
}

/// Apply bilateral filter to an entire greyscale image.
#[allow(dead_code)]
pub fn bilateral_filter_image(
    image: &[f32],
    width: u32,
    height: u32,
    config: &BilateralConfig,
) -> Vec<f32> {
    let mut output = vec![0.0_f32; image.len()];
    for y in 0..height {
        for x in 0..width {
            let idx = y as usize * width as usize + x as usize;
            output[idx] = bilateral_filter_pixel(image, width, height, x, y, config);
        }
    }
    output
}

/// Compute the effective kernel size needed for a given sigma.
#[allow(dead_code)]
pub fn effective_radius(sigma: f32) -> u32 {
    (sigma * 3.0).ceil() as u32
}

/// Separable bilateral approximation — horizontal pass on a row.
#[allow(dead_code)]
pub fn bilateral_horizontal_pass(
    row: &[f32],
    output: &mut [f32],
    config: &BilateralConfig,
) {
    let len = row.len();
    let r = config.kernel_radius as i32;

    for i in 0..len {
        let centre = row[i];
        let mut total_w = 0.0_f32;
        let mut total_v = 0.0_f32;

        for dx in -r..=r {
            let ni = i as i32 + dx;
            if ni < 0 || ni >= len as i32 {
                continue;
            }
            let neighbour = row[ni as usize];
            let ws = gaussian(dx.abs() as f32, config.sigma_spatial);
            let wr = gaussian((neighbour - centre).abs(), config.sigma_range);
            let w = ws * wr;
            total_w += w;
            total_v += w * neighbour;
        }

        output[i] = if total_w > 1e-8 { total_v / total_w } else { centre };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_config() {
        let c = BilateralConfig::default();
        assert!(c.sigma_spatial > 0.0);
    }

    #[test]
    fn test_gaussian_zero() {
        let g = gaussian(0.0, 1.0);
        assert!((g - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_gaussian_falloff() {
        let g0 = gaussian(0.0, 1.0);
        let g3 = gaussian(3.0, 1.0);
        assert!(g0 > g3);
    }

    #[test]
    fn test_gaussian_normalized_integrates() {
        let sigma = 1.0;
        let mut sum = 0.0_f32;
        let steps = 1000;
        let range = 5.0;
        let dx = 2.0 * range / steps as f32;
        for i in 0..steps {
            let x = -range + i as f32 * dx;
            sum += gaussian_normalized(x, sigma) * dx;
        }
        assert!((sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_bilateral_uniform_image() {
        let image = vec![0.5; 9];
        let result = bilateral_filter_pixel(&image, 3, 3, 1, 1, &BilateralConfig::default());
        assert!((result - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_bilateral_preserves_edge() {
        // Sharp edge: left half = 0, right half = 1
        let mut image = vec![0.0; 16];
        for i in 0..16 {
            if i % 4 >= 2 {
                image[i] = 1.0;
            }
        }
        let config = BilateralConfig { sigma_spatial: 2.0, sigma_range: 0.05, kernel_radius: 2 };
        let left = bilateral_filter_pixel(&image, 4, 4, 0, 2, &config);
        let right = bilateral_filter_pixel(&image, 4, 4, 3, 2, &config);
        // Edge should be preserved (strong range filter)
        assert!(left < 0.3);
        assert!(right > 0.7);
    }

    #[test]
    fn test_bilateral_filter_image_size() {
        let image = vec![0.5; 25];
        let result = bilateral_filter_image(&image, 5, 5, &BilateralConfig::default());
        assert_eq!(result.len(), 25);
    }

    #[test]
    fn test_effective_radius() {
        let r = effective_radius(2.0);
        assert_eq!(r, 6);
    }

    #[test]
    fn test_horizontal_pass_uniform() {
        let row = vec![0.5; 10];
        let mut output = vec![0.0; 10];
        bilateral_horizontal_pass(&row, &mut output, &BilateralConfig::default());
        for &v in &output {
            assert!((v - 0.5).abs() < 1e-5);
        }
    }

    #[test]
    fn test_gaussian_zero_sigma() {
        let g = gaussian(0.0, 0.0);
        assert!((g - 1.0).abs() < 1e-5);
        let g2 = gaussian(1.0, 0.0);
        assert!(g2.abs() < 1e-5);
    }
}
