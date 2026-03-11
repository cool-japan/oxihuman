// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct HistologySlide {
    pub width: u32,
    pub height: u32,
    pub pixels_rgb: Vec<[u8; 3]>,
    pub stain: String,
    pub tissue_type: String,
}

pub fn new_histology_slide(w: u32, h: u32, stain: &str) -> HistologySlide {
    let n = (w as usize) * (h as usize);
    HistologySlide {
        width: w,
        height: h,
        pixels_rgb: vec![[255, 255, 255]; n],
        stain: stain.to_string(),
        tissue_type: String::new(),
    }
}

pub fn histo_set_pixel(s: &mut HistologySlide, x: u32, y: u32, rgb: [u8; 3]) {
    if x < s.width && y < s.height {
        s.pixels_rgb[y as usize * s.width as usize + x as usize] = rgb;
    }
}

pub fn histo_get_pixel(s: &HistologySlide, x: u32, y: u32) -> [u8; 3] {
    if x < s.width && y < s.height {
        s.pixels_rgb[y as usize * s.width as usize + x as usize]
    } else {
        [0, 0, 0]
    }
}

/// Mean blue channel normalized to `[0,1]`.
pub fn histo_mean_hematoxylin(s: &HistologySlide) -> f32 {
    if s.pixels_rgb.is_empty() {
        return 0.0;
    }
    let sum: u64 = s.pixels_rgb.iter().map(|p| p[2] as u64).sum();
    sum as f32 / (s.pixels_rgb.len() as f32 * 255.0)
}

/// Mean red channel normalized to `[0,1]`.
pub fn histo_mean_eosin(s: &HistologySlide) -> f32 {
    if s.pixels_rgb.is_empty() {
        return 0.0;
    }
    let sum: u64 = s.pixels_rgb.iter().map(|p| p[0] as u64).sum();
    sum as f32 / (s.pixels_rgb.len() as f32 * 255.0)
}

pub fn histo_pixel_count(s: &HistologySlide) -> usize {
    s.pixels_rgb.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_slide() {
        /* stain is set */
        let s = new_histology_slide(4, 4, "H&E");
        assert_eq!(s.stain, "H&E");
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count = w*h */
        let s = new_histology_slide(8, 6, "H&E");
        assert_eq!(histo_pixel_count(&s), 48);
    }

    #[test]
    fn test_set_get_pixel() {
        /* set and get */
        let mut s = new_histology_slide(10, 10, "H&E");
        histo_set_pixel(&mut s, 2, 3, [100, 150, 200]);
        assert_eq!(histo_get_pixel(&s, 2, 3), [100, 150, 200]);
    }

    #[test]
    fn test_get_pixel_oob() {
        /* out of bounds => black */
        let s = new_histology_slide(5, 5, "H&E");
        assert_eq!(histo_get_pixel(&s, 100, 100), [0, 0, 0]);
    }

    #[test]
    fn test_mean_hematoxylin_default() {
        /* default white pixels => blue=255/255=1 */
        let s = new_histology_slide(2, 2, "H&E");
        let h = histo_mean_hematoxylin(&s);
        assert!((h - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_mean_eosin_default() {
        /* default white pixels => red=255/255=1 */
        let s = new_histology_slide(2, 2, "H&E");
        let e = histo_mean_eosin(&s);
        assert!((e - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_mean_hematoxylin_blue() {
        /* pure blue pixel */
        let mut s = new_histology_slide(1, 1, "H&E");
        histo_set_pixel(&mut s, 0, 0, [0, 0, 255]);
        let h = histo_mean_hematoxylin(&s);
        assert!((h - 1.0).abs() < 1e-5);
    }
}
