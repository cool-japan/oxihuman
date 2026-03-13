// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Stereo image pair export.

#[allow(dead_code)]
pub struct StereoImage {
    pub width: u32,
    pub height: u32,
    pub left: Vec<[u8; 3]>,
    pub right: Vec<[u8; 3]>,
}

#[allow(dead_code)]
pub fn new_stereo_image(width: u32, height: u32) -> StereoImage {
    let n = (width * height) as usize;
    StereoImage { width, height, left: vec![[0; 3]; n], right: vec![[0; 3]; n] }
}

fn idx(img: &StereoImage, x: u32, y: u32) -> Option<usize> {
    let i = (y * img.width + x) as usize;
    if i < img.left.len() { Some(i) } else { None }
}

#[allow(dead_code)]
pub fn si_set_left(img: &mut StereoImage, x: u32, y: u32, rgb: [u8; 3]) {
    if let Some(i) = idx(img, x, y) { img.left[i] = rgb; }
}

#[allow(dead_code)]
pub fn si_set_right(img: &mut StereoImage, x: u32, y: u32, rgb: [u8; 3]) {
    if let Some(i) = idx(img, x, y) { img.right[i] = rgb; }
}

#[allow(dead_code)]
pub fn si_get_left(img: &StereoImage, x: u32, y: u32) -> [u8; 3] {
    idx(img, x, y).map(|i| img.left[i]).unwrap_or([0; 3])
}

#[allow(dead_code)]
pub fn si_get_right(img: &StereoImage, x: u32, y: u32) -> [u8; 3] {
    idx(img, x, y).map(|i| img.right[i]).unwrap_or([0; 3])
}

#[allow(dead_code)]
pub fn si_disparity(img: &StereoImage, x: u32, y: u32) -> f32 {
    let l = si_get_left(img, x, y);
    let r = si_get_right(img, x, y);
    (l[0] as f32 - r[0] as f32).abs()
}

#[allow(dead_code)]
pub fn si_pixel_count(img: &StereoImage) -> usize {
    img.left.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let img = new_stereo_image(8, 4);
        assert_eq!(si_pixel_count(&img), 32);
    }

    #[test]
    fn test_set_get_left() {
        let mut img = new_stereo_image(4, 4);
        si_set_left(&mut img, 2, 1, [100, 150, 200]);
        assert_eq!(si_get_left(&img, 2, 1), [100, 150, 200]);
    }

    #[test]
    fn test_set_get_right() {
        let mut img = new_stereo_image(4, 4);
        si_set_right(&mut img, 1, 3, [50, 60, 70]);
        assert_eq!(si_get_right(&img, 1, 3), [50, 60, 70]);
    }

    #[test]
    fn test_disparity_same() {
        let img = new_stereo_image(4, 4);
        assert!((si_disparity(&img, 0, 0)).abs() < 1e-5);
    }

    #[test]
    fn test_disparity_different() {
        let mut img = new_stereo_image(4, 4);
        si_set_left(&mut img, 0, 0, [200, 0, 0]);
        si_set_right(&mut img, 0, 0, [100, 0, 0]);
        assert!((si_disparity(&img, 0, 0) - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_pixel_count() {
        let img = new_stereo_image(5, 3);
        assert_eq!(si_pixel_count(&img), 15);
    }

    #[test]
    fn test_out_of_bounds() {
        let img = new_stereo_image(2, 2);
        assert_eq!(si_get_left(&img, 99, 99), [0; 3]);
    }

    #[test]
    fn test_independent_channels() {
        let mut img = new_stereo_image(4, 4);
        si_set_left(&mut img, 0, 0, [10, 0, 0]);
        si_set_right(&mut img, 0, 0, [20, 0, 0]);
        assert_ne!(si_get_left(&img, 0, 0), si_get_right(&img, 0, 0));
    }
}
