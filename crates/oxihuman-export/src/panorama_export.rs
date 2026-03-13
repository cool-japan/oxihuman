// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Equirectangular panorama export.

#[allow(dead_code)]
pub struct PanoramaExport {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[u8; 3]>,
}

#[allow(dead_code)]
pub fn new_panorama_export(width: u32, height: u32) -> PanoramaExport {
    PanoramaExport { width, height, pixels: vec![[0, 0, 0]; (width * height) as usize] }
}

#[allow(dead_code)]
pub fn pano_set_pixel(exp: &mut PanoramaExport, x: u32, y: u32, rgb: [u8; 3]) {
    let idx = (y * exp.width + x) as usize;
    if idx < exp.pixels.len() { exp.pixels[idx] = rgb; }
}

#[allow(dead_code)]
pub fn pano_get_pixel(exp: &PanoramaExport, x: u32, y: u32) -> [u8; 3] {
    let idx = (y * exp.width + x) as usize;
    if idx < exp.pixels.len() { exp.pixels[idx] } else { [0, 0, 0] }
}

#[allow(dead_code)]
pub fn pano_pixel_count(exp: &PanoramaExport) -> usize {
    exp.pixels.len()
}

#[allow(dead_code)]
pub fn pano_latlon_to_xy(exp: &PanoramaExport, lat_deg: f32, lon_deg: f32) -> (u32, u32) {
    let lon_norm = (lon_deg + 180.0) / 360.0;
    let lat_norm = (90.0 - lat_deg) / 180.0;
    let x = (lon_norm * exp.width as f32).clamp(0.0, (exp.width - 1) as f32) as u32;
    let y = (lat_norm * exp.height as f32).clamp(0.0, (exp.height - 1) as f32) as u32;
    (x, y)
}

#[allow(dead_code)]
pub fn pano_fill(exp: &mut PanoramaExport, rgb: [u8; 3]) {
    for p in &mut exp.pixels { *p = rgb; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let exp = new_panorama_export(100, 50);
        assert_eq!(pano_pixel_count(&exp), 5000);
    }

    #[test]
    fn test_set_get_pixel() {
        let mut exp = new_panorama_export(10, 10);
        pano_set_pixel(&mut exp, 3, 2, [255, 0, 128]);
        assert_eq!(pano_get_pixel(&exp, 3, 2), [255, 0, 128]);
    }

    #[test]
    fn test_pixel_count() {
        let exp = new_panorama_export(8, 4);
        assert_eq!(pano_pixel_count(&exp), 32);
    }

    #[test]
    fn test_latlon_origin_xy_valid() {
        let exp = new_panorama_export(360, 180);
        let (x, y) = pano_latlon_to_xy(&exp, 0.0, 0.0);
        assert!(x < 360);
        assert!(y < 180);
    }

    #[test]
    fn test_fill() {
        let mut exp = new_panorama_export(4, 4);
        pano_fill(&mut exp, [10, 20, 30]);
        assert_eq!(pano_get_pixel(&exp, 0, 0), [10, 20, 30]);
        assert_eq!(pano_get_pixel(&exp, 3, 3), [10, 20, 30]);
    }

    #[test]
    fn test_out_of_bounds_get() {
        let exp = new_panorama_export(4, 4);
        assert_eq!(pano_get_pixel(&exp, 100, 100), [0, 0, 0]);
    }

    #[test]
    fn test_latlon_north_pole() {
        let exp = new_panorama_export(360, 180);
        let (_x, y) = pano_latlon_to_xy(&exp, 90.0, 0.0);
        assert_eq!(y, 0);
    }

    #[test]
    fn test_latlon_west_boundary() {
        let exp = new_panorama_export(360, 180);
        let (x, _y) = pano_latlon_to_xy(&exp, 0.0, -180.0);
        assert_eq!(x, 0);
    }

    #[test]
    fn test_default_pixel_is_black() {
        let exp = new_panorama_export(2, 2);
        assert_eq!(pano_pixel_count(&exp), 4);
        assert_eq!(pano_get_pixel(&exp, 0, 0), [0u8, 0, 0]);
    }
}
