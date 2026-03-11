// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export baked UV texture data (lightmaps, AO maps, etc.).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BakeUvExport {
    pub width: u32,
    pub height: u32,
    pub channels: u32,
    pub pixels: Vec<f32>,
    pub label: String,
}

#[allow(dead_code)]
pub fn new_bake_uv_export(label: &str, width: u32, height: u32, channels: u32) -> BakeUvExport {
    let size = (width * height * channels) as usize;
    BakeUvExport { width, height, channels, pixels: vec![0.0; size], label: label.to_string() }
}

#[allow(dead_code)]
pub fn bake_uv_set_pixel(b: &mut BakeUvExport, x: u32, y: u32, channel: u32, value: f32) {
    if x < b.width && y < b.height && channel < b.channels {
        let idx = ((y * b.width + x) * b.channels + channel) as usize;
        if idx < b.pixels.len() { b.pixels[idx] = value; }
    }
}

#[allow(dead_code)]
pub fn bake_uv_get_pixel(b: &BakeUvExport, x: u32, y: u32, channel: u32) -> f32 {
    if x < b.width && y < b.height && channel < b.channels {
        let idx = ((y * b.width + x) * b.channels + channel) as usize;
        if idx < b.pixels.len() { return b.pixels[idx]; }
    }
    0.0
}

#[allow(dead_code)]
pub fn bake_uv_total_pixels(b: &BakeUvExport) -> u32 { b.width * b.height }

#[allow(dead_code)]
pub fn bake_uv_byte_size(b: &BakeUvExport) -> usize { b.pixels.len() * 4 }

#[allow(dead_code)]
pub fn bake_uv_fill(b: &mut BakeUvExport, value: f32) {
    for p in &mut b.pixels { *p = value; }
}

#[allow(dead_code)]
pub fn bake_uv_validate(b: &BakeUvExport) -> bool {
    b.width > 0 && b.height > 0 && b.channels > 0 && b.pixels.len() == (b.width * b.height * b.channels) as usize
}

#[allow(dead_code)]
pub fn bake_uv_to_json(b: &BakeUvExport) -> String {
    format!("{{\"label\":\"{}\",\"width\":{},\"height\":{},\"channels\":{}}}", b.label, b.width, b.height, b.channels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_new() { let b = new_bake_uv_export("ao", 4, 4, 1); assert_eq!(b.pixels.len(), 16); }
    #[test] fn test_set_get() { let mut b = new_bake_uv_export("ao", 4, 4, 1); bake_uv_set_pixel(&mut b, 1, 1, 0, 0.5); assert!((bake_uv_get_pixel(&b, 1, 1, 0) - 0.5).abs() < 1e-6); }
    #[test] fn test_total_pixels() { let b = new_bake_uv_export("ao", 8, 8, 3); assert_eq!(bake_uv_total_pixels(&b), 64); }
    #[test] fn test_byte_size() { let b = new_bake_uv_export("ao", 2, 2, 1); assert_eq!(bake_uv_byte_size(&b), 16); }
    #[test] fn test_fill() { let mut b = new_bake_uv_export("ao", 2, 2, 1); bake_uv_fill(&mut b, 1.0); assert!((b.pixels[3] - 1.0).abs() < 1e-6); }
    #[test] fn test_validate() { let b = new_bake_uv_export("ao", 4, 4, 1); assert!(bake_uv_validate(&b)); }
    #[test] fn test_to_json() { let b = new_bake_uv_export("lightmap", 4, 4, 3); assert!(bake_uv_to_json(&b).contains("lightmap")); }
    #[test] fn test_out_of_bounds() { let b = new_bake_uv_export("ao", 2, 2, 1); assert!((bake_uv_get_pixel(&b, 10, 10, 0)).abs() < 1e-6); }
    #[test] fn test_label() { let b = new_bake_uv_export("test", 1, 1, 1); assert_eq!(b.label, "test"); }
    #[test] fn test_multi_channel() { let mut b = new_bake_uv_export("c", 2, 2, 3); bake_uv_set_pixel(&mut b, 0, 0, 2, 0.7); assert!((bake_uv_get_pixel(&b, 0, 0, 2) - 0.7).abs() < 1e-6); }
}
