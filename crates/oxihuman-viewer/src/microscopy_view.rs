// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct MicroscopyImage {
    pub width: u32,
    pub height: u32,
    pub pixels_rgb: Vec<[u8; 3]>,
    pub magnification: f32,
    pub pixel_size_um: f32,
}

pub fn new_microscopy_image(w: u32, h: u32, mag: f32) -> MicroscopyImage {
    let n = (w as usize) * (h as usize);
    MicroscopyImage {
        width: w,
        height: h,
        pixels_rgb: vec![[0, 0, 0]; n],
        magnification: mag,
        pixel_size_um: 100.0 / mag.max(1.0),
    }
}

pub fn micro_set_pixel(img: &mut MicroscopyImage, x: u32, y: u32, rgb: [u8; 3]) {
    if x < img.width && y < img.height {
        img.pixels_rgb[y as usize * img.width as usize + x as usize] = rgb;
    }
}

pub fn micro_get_pixel(img: &MicroscopyImage, x: u32, y: u32) -> [u8; 3] {
    if x < img.width && y < img.height {
        img.pixels_rgb[y as usize * img.width as usize + x as usize]
    } else {
        [0, 0, 0]
    }
}

pub fn micro_field_of_view_um(img: &MicroscopyImage) -> [f32; 2] {
    [
        img.width as f32 * img.pixel_size_um,
        img.height as f32 * img.pixel_size_um,
    ]
}

pub fn micro_resolution_um(img: &MicroscopyImage) -> f32 {
    img.pixel_size_um
}

pub fn micro_pixel_count(img: &MicroscopyImage) -> usize {
    img.pixels_rgb.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_image() {
        /* pixel count = w*h */
        let img = new_microscopy_image(64, 64, 10.0);
        assert_eq!(micro_pixel_count(&img), 64 * 64);
    }

    #[test]
    fn test_set_get_pixel() {
        /* set and get */
        let mut img = new_microscopy_image(10, 10, 40.0);
        micro_set_pixel(&mut img, 3, 4, [255, 128, 0]);
        assert_eq!(micro_get_pixel(&img, 3, 4), [255, 128, 0]);
    }

    #[test]
    fn test_get_pixel_oob() {
        /* out of bounds => black */
        let img = new_microscopy_image(5, 5, 10.0);
        assert_eq!(micro_get_pixel(&img, 100, 100), [0, 0, 0]);
    }

    #[test]
    fn test_field_of_view() {
        /* fov = dims * pixel_size */
        let img = new_microscopy_image(10, 10, 10.0);
        let fov = micro_field_of_view_um(&img);
        assert!((fov[0] - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_resolution_um() {
        /* resolution_um = pixel_size_um */
        let img = new_microscopy_image(10, 10, 10.0);
        assert!((micro_resolution_um(&img) - img.pixel_size_um).abs() < 1e-6);
    }

    #[test]
    fn test_magnification_increases_resolution() {
        /* higher magnification => smaller pixel size */
        let low = new_microscopy_image(10, 10, 10.0);
        let high = new_microscopy_image(10, 10, 100.0);
        assert!(high.pixel_size_um < low.pixel_size_um);
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count correct */
        let img = new_microscopy_image(8, 6, 20.0);
        assert_eq!(micro_pixel_count(&img), 48);
    }
}
