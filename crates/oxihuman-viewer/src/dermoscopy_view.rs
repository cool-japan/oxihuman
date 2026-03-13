// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct DermoscopyImage {
    pub width: u32,
    pub height: u32,
    pub pixels_rgb: Vec<[u8; 3]>,
    pub polarized: bool,
}

pub fn new_dermoscopy_image(w: u32, h: u32, polarized: bool) -> DermoscopyImage {
    let n = (w as usize) * (h as usize);
    DermoscopyImage {
        width: w,
        height: h,
        pixels_rgb: vec![[0, 0, 0]; n],
        polarized,
    }
}

pub fn dermo_set_pixel(img: &mut DermoscopyImage, x: u32, y: u32, rgb: [u8; 3]) {
    if x < img.width && y < img.height {
        img.pixels_rgb[y as usize * img.width as usize + x as usize] = rgb;
    }
}

pub fn dermo_get_pixel(img: &DermoscopyImage, x: u32, y: u32) -> [u8; 3] {
    if x < img.width && y < img.height {
        img.pixels_rgb[y as usize * img.width as usize + x as usize]
    } else {
        [0, 0, 0]
    }
}

pub fn dermo_mean_color(img: &DermoscopyImage) -> [f32; 3] {
    if img.pixels_rgb.is_empty() {
        return [0.0; 3];
    }
    let n = img.pixels_rgb.len() as f32;
    let mut sums = [0.0f32; 3];
    for p in &img.pixels_rgb {
        sums[0] += p[0] as f32;
        sums[1] += p[1] as f32;
        sums[2] += p[2] as f32;
    }
    [sums[0] / n, sums[1] / n, sums[2] / n]
}

/// Stub: asymmetry score is 0.0.
pub fn dermo_asymmetry_score(_img: &DermoscopyImage) -> f32 {
    0.0
}

pub fn dermo_pixel_count(img: &DermoscopyImage) -> usize {
    img.pixels_rgb.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_image() {
        /* polarized flag is set */
        let img = new_dermoscopy_image(64, 64, true);
        assert!(img.polarized);
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count = w*h */
        let img = new_dermoscopy_image(8, 6, false);
        assert_eq!(dermo_pixel_count(&img), 48);
    }

    #[test]
    fn test_set_get_pixel() {
        /* set and get */
        let mut img = new_dermoscopy_image(10, 10, false);
        dermo_set_pixel(&mut img, 3, 5, [200, 100, 50]);
        assert_eq!(dermo_get_pixel(&img, 3, 5), [200, 100, 50]);
    }

    #[test]
    fn test_get_pixel_oob() {
        /* out of bounds => black */
        let img = new_dermoscopy_image(5, 5, false);
        assert_eq!(dermo_get_pixel(&img, 100, 100), [0, 0, 0]);
    }

    #[test]
    fn test_mean_color_black() {
        /* all black => mean [0,0,0] */
        let img = new_dermoscopy_image(4, 4, false);
        let mc = dermo_mean_color(&img);
        assert!((mc[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_asymmetry_score_stub() {
        /* stub returns 0 */
        let img = new_dermoscopy_image(4, 4, false);
        assert!((dermo_asymmetry_score(&img) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_mean_color_uniform() {
        /* uniform red => mean [255,0,0] */
        let mut img = new_dermoscopy_image(2, 2, false);
        for y in 0..2 {
            for x in 0..2 {
                dermo_set_pixel(&mut img, x, y, [255, 0, 0]);
            }
        }
        let mc = dermo_mean_color(&img);
        assert!((mc[0] - 255.0).abs() < 1e-5);
        assert!((mc[1] - 0.0).abs() < 1e-5);
    }
}
