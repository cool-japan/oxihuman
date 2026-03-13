// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Depth image export (2D array of depth values).

#[allow(dead_code)]
pub struct DepthImageV2 {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_depth_image_v2(width: u32, height: u32) -> DepthImageV2 {
    DepthImageV2 { width, height, data: vec![0.0; (width * height) as usize] }
}

#[allow(dead_code)]
pub fn di_set_v2(img: &mut DepthImageV2, x: u32, y: u32, depth: f32) {
    let idx = (y * img.width + x) as usize;
    if idx < img.data.len() { img.data[idx] = depth; }
}

#[allow(dead_code)]
pub fn di_get_v2(img: &DepthImageV2, x: u32, y: u32) -> f32 {
    let idx = (y * img.width + x) as usize;
    if idx < img.data.len() { img.data[idx] } else { 0.0 }
}

#[allow(dead_code)]
pub fn di_min_depth_v2(img: &DepthImageV2) -> f32 {
    img.data.iter().copied().fold(f32::MAX, f32::min)
}

#[allow(dead_code)]
pub fn di_max_depth_v2(img: &DepthImageV2) -> f32 {
    img.data.iter().copied().fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn di_to_normalized_v2(img: &DepthImageV2) -> Vec<f32> {
    let mn = di_min_depth_v2(img);
    let mx = di_max_depth_v2(img);
    let range = mx - mn;
    if range < 1e-10 {
        return vec![0.0; img.data.len()];
    }
    img.data.iter().map(|&d| (d - mn) / range).collect()
}

#[allow(dead_code)]
pub fn di_pixel_count_v2(img: &DepthImageV2) -> usize {
    img.data.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let img = new_depth_image_v2(4, 4);
        assert_eq!(di_pixel_count_v2(&img), 16);
    }

    #[test]
    fn test_set_get() {
        let mut img = new_depth_image_v2(4, 4);
        di_set_v2(&mut img, 2, 1, 5.0);
        assert!((di_get_v2(&img, 2, 1) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_min_depth() {
        let mut img = new_depth_image_v2(2, 2);
        di_set_v2(&mut img, 0, 0, 3.0);
        di_set_v2(&mut img, 1, 0, 1.0);
        di_set_v2(&mut img, 0, 1, 2.0);
        di_set_v2(&mut img, 1, 1, 4.0);
        assert!((di_min_depth_v2(&img) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_depth() {
        let mut img = new_depth_image_v2(2, 1);
        di_set_v2(&mut img, 0, 0, 2.0);
        di_set_v2(&mut img, 1, 0, 7.0);
        assert!((di_max_depth_v2(&img) - 7.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize() {
        let mut img = new_depth_image_v2(2, 1);
        di_set_v2(&mut img, 0, 0, 0.0);
        di_set_v2(&mut img, 1, 0, 10.0);
        let norm = di_to_normalized_v2(&img);
        assert!((norm[1] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_pixel_count() {
        let img = new_depth_image_v2(8, 6);
        assert_eq!(di_pixel_count_v2(&img), 48);
    }

    #[test]
    fn test_default_zero() {
        let img = new_depth_image_v2(3, 3);
        assert!((di_get_v2(&img, 1, 1)).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_uniform_returns_zero() {
        let img = new_depth_image_v2(2, 2);
        let norm = di_to_normalized_v2(&img);
        assert_eq!(norm, vec![0.0; 4]);
    }

    #[test]
    fn test_out_of_bounds_get() {
        let img = new_depth_image_v2(2, 2);
        assert_eq!(di_get_v2(&img, 100, 100), 0.0);
    }
}
