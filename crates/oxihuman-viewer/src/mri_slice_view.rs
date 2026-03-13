// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct MriSlice {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<i16>,
    pub window_center: i16,
    pub window_width: i16,
    pub slice_thickness_mm: f32,
}

pub fn new_mri_slice(w: u16, h: u16) -> MriSlice {
    let n = (w as usize) * (h as usize);
    MriSlice {
        width: w,
        height: h,
        pixels: vec![0; n],
        window_center: 40,
        window_width: 400,
        slice_thickness_mm: 1.0,
    }
}

pub fn mri_set_pixel(s: &mut MriSlice, x: u16, y: u16, v: i16) {
    if x < s.width && y < s.height {
        let idx = y as usize * s.width as usize + x as usize;
        s.pixels[idx] = v;
    }
}

pub fn mri_get_pixel(s: &MriSlice, x: u16, y: u16) -> i16 {
    if x < s.width && y < s.height {
        s.pixels[y as usize * s.width as usize + x as usize]
    } else {
        0
    }
}

pub fn mri_window_to_display(s: &MriSlice, hu: i16) -> u8 {
    let lower = s.window_center - s.window_width / 2;
    let upper = s.window_center + s.window_width / 2;
    if hu <= lower {
        return 0;
    }
    if hu >= upper {
        return 255;
    }
    let t = (hu - lower) as f32 / (upper - lower) as f32;
    (t * 255.0) as u8
}

pub fn mri_pixel_count(s: &MriSlice) -> usize {
    s.width as usize * s.height as usize
}

pub fn mri_mean_value(s: &MriSlice) -> f32 {
    if s.pixels.is_empty() {
        return 0.0;
    }
    let sum: i64 = s.pixels.iter().map(|&v| v as i64).sum();
    sum as f32 / s.pixels.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_slice() {
        /* pixel count = w*h */
        let s = new_mri_slice(64, 64);
        assert_eq!(mri_pixel_count(&s), 64 * 64);
    }

    #[test]
    fn test_set_get_pixel() {
        /* set and get pixel */
        let mut s = new_mri_slice(10, 10);
        mri_set_pixel(&mut s, 3, 4, 500);
        assert_eq!(mri_get_pixel(&s, 3, 4), 500);
    }

    #[test]
    fn test_get_pixel_oob() {
        /* out of bounds returns 0 */
        let s = new_mri_slice(5, 5);
        assert_eq!(mri_get_pixel(&s, 100, 100), 0);
    }

    #[test]
    fn test_window_display_min() {
        /* value below window => 0 */
        let s = new_mri_slice(1, 1);
        assert_eq!(mri_window_to_display(&s, -10000), 0);
    }

    #[test]
    fn test_window_display_max() {
        /* value above window => 255 */
        let s = new_mri_slice(1, 1);
        assert_eq!(mri_window_to_display(&s, 10000), 255);
    }

    #[test]
    fn test_mean_value_zero() {
        /* all zeros => mean 0 */
        let s = new_mri_slice(4, 4);
        assert!((mri_mean_value(&s) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_mean_value() {
        /* set some pixels */
        let mut s = new_mri_slice(2, 1);
        mri_set_pixel(&mut s, 0, 0, 100);
        mri_set_pixel(&mut s, 1, 0, 200);
        assert!((mri_mean_value(&s) - 150.0).abs() < 1e-4);
    }
}
