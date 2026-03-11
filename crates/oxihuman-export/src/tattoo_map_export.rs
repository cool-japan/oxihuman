// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct TattooMap {
    pub width: u32,
    pub height: u32,
    pub pixels_rgba: Vec<[u8; 4]>,
    pub body_region: String,
}

pub fn new_tattoo_map(w: u32, h: u32, region: &str) -> TattooMap {
    let n = (w * h) as usize;
    TattooMap {
        width: w,
        height: h,
        pixels_rgba: vec![[0, 0, 0, 0]; n],
        body_region: region.to_string(),
    }
}

fn idx(m: &TattooMap, x: u32, y: u32) -> usize {
    (y * m.width + x) as usize
}

pub fn tattoo_set_pixel(m: &mut TattooMap, x: u32, y: u32, rgba: [u8; 4]) {
    let i = idx(m, x, y);
    m.pixels_rgba[i] = rgba;
}

pub fn tattoo_get_pixel(m: &TattooMap, x: u32, y: u32) -> [u8; 4] {
    m.pixels_rgba[idx(m, x, y)]
}

pub fn tattoo_coverage(m: &TattooMap) -> f32 {
    let total = m.pixels_rgba.len();
    if total == 0 {
        return 0.0;
    }
    let ink = m.pixels_rgba.iter().filter(|p| p[3] > 0).count();
    ink as f32 / total as f32
}

pub fn tattoo_to_bytes(m: &TattooMap) -> Vec<u8> {
    m.pixels_rgba
        .iter()
        .flat_map(|p| p.iter().copied())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tattoo_map() {
        /* transparent by default */
        let m = new_tattoo_map(4, 4, "arm");
        assert_eq!(m.pixels_rgba.len(), 16);
        assert_eq!(tattoo_get_pixel(&m, 0, 0)[3], 0);
    }

    #[test]
    fn test_set_get_pixel() {
        /* round-trip */
        let mut m = new_tattoo_map(4, 4, "back");
        tattoo_set_pixel(&mut m, 1, 2, [255, 0, 0, 200]);
        let p = tattoo_get_pixel(&m, 1, 2);
        assert_eq!(p[0], 255);
        assert_eq!(p[3], 200);
    }

    #[test]
    fn test_coverage_zero() {
        /* transparent => 0 */
        let m = new_tattoo_map(4, 4, "chest");
        assert!((tattoo_coverage(&m)).abs() < 1e-6);
    }

    #[test]
    fn test_coverage_partial() {
        /* one pixel inked */
        let mut m = new_tattoo_map(2, 1, "arm");
        tattoo_set_pixel(&mut m, 0, 0, [0, 0, 0, 255]);
        assert!((tattoo_coverage(&m) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_bytes_length() {
        /* 4 bytes per pixel */
        let m = new_tattoo_map(3, 3, "chest");
        let bytes = tattoo_to_bytes(&m);
        assert_eq!(bytes.len(), 3 * 3 * 4);
    }
}
