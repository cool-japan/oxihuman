// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! KTX texture container export for GPU-ready texture formats.

/// KTX format type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KtxFormat { Rgba8, Rgba16f, R8, Rg8, Bc1, Bc7 }

/// KTX export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KtxExport {
    pub width: u32,
    pub height: u32,
    pub mip_levels: u32,
    pub format: KtxFormat,
    pub data: Vec<u8>,
}

#[allow(dead_code)]
pub fn new_ktx_export(w: u32, h: u32, format: KtxFormat) -> KtxExport {
    let bpp = ktx_bytes_per_pixel(format);
    KtxExport { width: w, height: h, mip_levels: 1, format, data: vec![0u8; (w * h) as usize * bpp] }
}

#[allow(dead_code)]
pub fn ktx_bytes_per_pixel(format: KtxFormat) -> usize {
    match format {
        KtxFormat::Rgba8 => 4, KtxFormat::Rgba16f => 8, KtxFormat::R8 => 1,
        KtxFormat::Rg8 => 2, KtxFormat::Bc1 => 1, KtxFormat::Bc7 => 1,
    }
}

#[allow(dead_code)]
pub fn ktx_data_size(export: &KtxExport) -> usize { export.data.len() }

#[allow(dead_code)]
pub fn ktx_format_name(f: KtxFormat) -> &'static str {
    match f {
        KtxFormat::Rgba8 => "RGBA8", KtxFormat::Rgba16f => "RGBA16F",
        KtxFormat::R8 => "R8", KtxFormat::Rg8 => "RG8",
        KtxFormat::Bc1 => "BC1", KtxFormat::Bc7 => "BC7",
    }
}

#[allow(dead_code)]
pub fn ktx_set_mip_levels(export: &mut KtxExport, levels: u32) {
    export.mip_levels = levels.max(1);
}

#[allow(dead_code)]
pub fn ktx_to_header_bytes(export: &KtxExport) -> Vec<u8> {
    let mut bytes = Vec::new();
    // KTX2 identifier
    bytes.extend_from_slice(&[0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A]);
    bytes.extend_from_slice(&export.width.to_le_bytes());
    bytes.extend_from_slice(&export.height.to_le_bytes());
    bytes.extend_from_slice(&export.mip_levels.to_le_bytes());
    bytes
}

#[allow(dead_code)]
pub fn ktx_to_json(export: &KtxExport) -> String {
    format!(r#"{{"width":{},"height":{},"format":"{}","mips":{},"size":{}}}"#,
        export.width, export.height, ktx_format_name(export.format), export.mip_levels, ktx_data_size(export))
}

#[allow(dead_code)]
pub fn ktx_estimated_mip_size(w: u32, h: u32, level: u32) -> (u32, u32) {
    ((w >> level).max(1), (h >> level).max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ktx() {
        let k = new_ktx_export(4, 4, KtxFormat::Rgba8);
        assert_eq!(ktx_data_size(&k), 64);
    }

    #[test]
    fn test_bytes_per_pixel() {
        assert_eq!(ktx_bytes_per_pixel(KtxFormat::Rgba16f), 8);
    }

    #[test]
    fn test_format_name() {
        assert_eq!(ktx_format_name(KtxFormat::Bc7), "BC7");
    }

    #[test]
    fn test_set_mip_levels() {
        let mut k = new_ktx_export(8, 8, KtxFormat::Rgba8);
        ktx_set_mip_levels(&mut k, 4);
        assert_eq!(k.mip_levels, 4);
    }

    #[test]
    fn test_header_bytes() {
        let k = new_ktx_export(2, 2, KtxFormat::R8);
        let header = ktx_to_header_bytes(&k);
        assert_eq!(header[0], 0xAB);
    }

    #[test]
    fn test_to_json() {
        let k = new_ktx_export(4, 4, KtxFormat::Rg8);
        let json = ktx_to_json(&k);
        assert!(json.contains("RG8"));
    }

    #[test]
    fn test_mip_size() {
        let (w, h) = ktx_estimated_mip_size(256, 256, 2);
        assert_eq!(w, 64);
        assert_eq!(h, 64);
    }

    #[test]
    fn test_mip_size_min() {
        let (w, h) = ktx_estimated_mip_size(4, 4, 10);
        assert_eq!(w, 1);
        assert_eq!(h, 1);
    }

    #[test]
    fn test_r8_size() {
        let k = new_ktx_export(3, 3, KtxFormat::R8);
        assert_eq!(ktx_data_size(&k), 9);
    }

    #[test]
    fn test_mip_levels_min() {
        let mut k = new_ktx_export(1, 1, KtxFormat::Rgba8);
        ktx_set_mip_levels(&mut k, 0);
        assert_eq!(k.mip_levels, 1);
    }

}
