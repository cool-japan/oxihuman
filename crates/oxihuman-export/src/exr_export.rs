// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! OpenEXR-style HDR image export (simplified header + float pixel data).

/// EXR export configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExrExportConfig {
    pub compression: ExrCompression,
    pub half_float: bool,
}

/// EXR compression modes.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExrCompression { None, Zip, Piz, Rle }

/// EXR image buffer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExrImage {
    pub width: u32,
    pub height: u32,
    pub channels: u32,
    pub data: Vec<f32>,
}

#[allow(dead_code)]
pub fn default_exr_config() -> ExrExportConfig {
    ExrExportConfig { compression: ExrCompression::Zip, half_float: false }
}

#[allow(dead_code)]
pub fn new_exr_image(w: u32, h: u32, channels: u32) -> ExrImage {
    ExrImage { width: w, height: h, channels, data: vec![0.0; (w * h * channels) as usize] }
}

#[allow(dead_code)]
pub fn exr_set_pixel(img: &mut ExrImage, x: u32, y: u32, values: &[f32]) {
    let base = ((y * img.width + x) * img.channels) as usize;
    for (i, &v) in values.iter().enumerate() {
        if base + i < img.data.len() { img.data[base + i] = v; }
    }
}

#[allow(dead_code)]
pub fn exr_get_pixel(img: &ExrImage, x: u32, y: u32) -> Vec<f32> {
    let base = ((y * img.width + x) * img.channels) as usize;
    let end = (base + img.channels as usize).min(img.data.len());
    img.data[base..end].to_vec()
}

#[allow(dead_code)]
pub fn exr_pixel_count(img: &ExrImage) -> usize {
    (img.width * img.height) as usize
}

#[allow(dead_code)]
pub fn exr_data_size(img: &ExrImage) -> usize {
    img.data.len() * 4
}

#[allow(dead_code)]
pub fn exr_compression_name(c: ExrCompression) -> &'static str {
    match c { ExrCompression::None => "none", ExrCompression::Zip => "zip",
        ExrCompression::Piz => "piz", ExrCompression::Rle => "rle" }
}

#[allow(dead_code)]
pub fn exr_to_bytes(img: &ExrImage) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(img.data.len() * 4 + 20);
    // Magic number
    bytes.extend_from_slice(&[0x76, 0x2F, 0x31, 0x01]);
    bytes.extend_from_slice(&img.width.to_le_bytes());
    bytes.extend_from_slice(&img.height.to_le_bytes());
    bytes.extend_from_slice(&img.channels.to_le_bytes());
    for &v in &img.data {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

#[allow(dead_code)]
pub fn exr_to_json(img: &ExrImage) -> String {
    format!(r#"{{"width":{},"height":{},"channels":{},"data_size":{}}}"#,
        img.width, img.height, img.channels, exr_data_size(img))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_exr_config();
        assert_eq!(c.compression, ExrCompression::Zip);
    }

    #[test]
    fn test_new_image() {
        let img = new_exr_image(4, 4, 3);
        assert_eq!(exr_pixel_count(&img), 16);
    }

    #[test]
    fn test_set_get_pixel() {
        let mut img = new_exr_image(2, 2, 3);
        exr_set_pixel(&mut img, 0, 0, &[1.0, 0.5, 0.25]);
        let p = exr_get_pixel(&img, 0, 0);
        assert!((p[0] - 1.0).abs() < 1e-5);
        assert!((p[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_data_size() {
        let img = new_exr_image(2, 2, 4);
        assert_eq!(exr_data_size(&img), 2 * 2 * 4 * 4);
    }

    #[test]
    fn test_compression_name() {
        assert_eq!(exr_compression_name(ExrCompression::Piz), "piz");
    }

    #[test]
    fn test_to_bytes() {
        let img = new_exr_image(1, 1, 1);
        let bytes = exr_to_bytes(&img);
        assert!(bytes.len() > 4);
        assert_eq!(bytes[0], 0x76);
    }

    #[test]
    fn test_to_json() {
        let img = new_exr_image(3, 3, 3);
        let json = exr_to_json(&img);
        assert!(json.contains("width"));
    }

    #[test]
    fn test_hdr_values() {
        let mut img = new_exr_image(1, 1, 3);
        exr_set_pixel(&mut img, 0, 0, &[100.0, 200.0, 0.5]);
        let p = exr_get_pixel(&img, 0, 0);
        assert!((p[0] - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_image() {
        let img = new_exr_image(0, 0, 3);
        assert_eq!(exr_pixel_count(&img), 0);
    }

    #[test]
    fn test_single_channel() {
        let mut img = new_exr_image(2, 2, 1);
        exr_set_pixel(&mut img, 1, 1, &[0.75]);
        let p = exr_get_pixel(&img, 1, 1);
        assert_eq!(p.len(), 1);
        assert!((p[0] - 0.75).abs() < 1e-5);
    }

}
