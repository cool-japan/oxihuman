// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

const SPEED_OF_SOUND_MM_PER_US: f32 = 1.54; // ~1540 m/s

#[derive(Debug, Clone)]
pub struct UltrasoundImage {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<u8>,
    pub frequency_mhz: f32,
    pub depth_cm: f32,
}

pub fn new_ultrasound_image(w: u16, h: u16, freq: f32, depth: f32) -> UltrasoundImage {
    let n = (w as usize) * (h as usize);
    UltrasoundImage {
        width: w,
        height: h,
        pixels: vec![0; n],
        frequency_mhz: freq,
        depth_cm: depth,
    }
}

pub fn us_set_pixel(img: &mut UltrasoundImage, x: u16, y: u16, v: u8) {
    if x < img.width && y < img.height {
        img.pixels[y as usize * img.width as usize + x as usize] = v;
    }
}

pub fn us_get_pixel(img: &UltrasoundImage, x: u16, y: u16) -> u8 {
    if x < img.width && y < img.height {
        img.pixels[y as usize * img.width as usize + x as usize]
    } else {
        0
    }
}

/// Axial resolution ≈ speed_of_sound / (2 * freq_mhz) in mm.
pub fn us_axial_resolution_mm(img: &UltrasoundImage) -> f32 {
    if img.frequency_mhz < 1e-9 {
        return 0.0;
    }
    SPEED_OF_SOUND_MM_PER_US / (2.0 * img.frequency_mhz)
}

pub fn us_pixel_to_depth_mm(img: &UltrasoundImage, y: u16) -> f32 {
    if img.height == 0 {
        return 0.0;
    }
    (y as f32 / img.height as f32) * img.depth_cm * 10.0
}

pub fn us_mean_echogenicity(img: &UltrasoundImage) -> f32 {
    if img.pixels.is_empty() {
        return 0.0;
    }
    let sum: u64 = img.pixels.iter().map(|&v| v as u64).sum();
    sum as f32 / img.pixels.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_image() {
        /* pixel count = w*h */
        let img = new_ultrasound_image(64, 64, 5.0, 10.0);
        assert_eq!(img.pixels.len(), 64 * 64);
    }

    #[test]
    fn test_set_get_pixel() {
        /* set and get */
        let mut img = new_ultrasound_image(10, 10, 5.0, 5.0);
        us_set_pixel(&mut img, 2, 3, 128);
        assert_eq!(us_get_pixel(&img, 2, 3), 128);
    }

    #[test]
    fn test_get_pixel_oob() {
        /* out of bounds => 0 */
        let img = new_ultrasound_image(5, 5, 5.0, 5.0);
        assert_eq!(us_get_pixel(&img, 100, 100), 0);
    }

    #[test]
    fn test_axial_resolution() {
        /* 5 MHz => 0.154 mm */
        let img = new_ultrasound_image(64, 64, 5.0, 10.0);
        let r = us_axial_resolution_mm(&img);
        assert!((r - SPEED_OF_SOUND_MM_PER_US / 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_pixel_to_depth_mm() {
        /* y=height => depth_cm*10 */
        let img = new_ultrasound_image(10, 10, 5.0, 10.0);
        let d = us_pixel_to_depth_mm(&img, 10);
        assert!((d - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_mean_echogenicity_zero() {
        /* all zeros => 0 */
        let img = new_ultrasound_image(4, 4, 5.0, 5.0);
        assert!((us_mean_echogenicity(&img) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_mean_echogenicity_nonzero() {
        /* set one pixel */
        let mut img = new_ultrasound_image(2, 1, 5.0, 5.0);
        us_set_pixel(&mut img, 0, 0, 200);
        us_set_pixel(&mut img, 1, 0, 100);
        let m = us_mean_echogenicity(&img);
        assert!((m - 150.0).abs() < 1e-4);
    }
}
