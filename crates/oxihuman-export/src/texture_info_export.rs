#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Texture information export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextureInfoExport {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub mip_count: u32,
    pub name: String,
}

/// Create a texture info export.
#[allow(dead_code)]
pub fn export_texture_info(
    name: &str,
    width: u32,
    height: u32,
    format: &str,
    mip_count: u32,
) -> TextureInfoExport {
    TextureInfoExport {
        width,
        height,
        format: format.to_string(),
        mip_count,
        name: name.to_string(),
    }
}

/// Return texture width.
#[allow(dead_code)]
pub fn texture_width(t: &TextureInfoExport) -> u32 {
    t.width
}

/// Return texture height.
#[allow(dead_code)]
pub fn texture_height(t: &TextureInfoExport) -> u32 {
    t.height
}

/// Return format name.
#[allow(dead_code)]
pub fn texture_format_name(t: &TextureInfoExport) -> &str {
    &t.format
}

/// Return mip-map count.
#[allow(dead_code)]
pub fn texture_mip_count(t: &TextureInfoExport) -> u32 {
    t.mip_count
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn texture_to_json(t: &TextureInfoExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"width\":{},\"height\":{},\"format\":\"{}\",\"mips\":{}}}",
        t.name, t.width, t.height, t.format, t.mip_count,
    )
}

/// Return estimated byte size of the texture info record.
#[allow(dead_code)]
pub fn texture_info_size(t: &TextureInfoExport) -> usize {
    // name + format string lengths + 3 u32 fields
    t.name.len() + t.format.len() + 12
}

/// Validate: width/height > 0, non-empty name and format.
#[allow(dead_code)]
pub fn validate_texture_info(t: &TextureInfoExport) -> bool {
    t.width > 0 && t.height > 0 && !t.name.is_empty() && !t.format.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> TextureInfoExport {
        export_texture_info("diffuse", 1024, 1024, "RGBA8", 10)
    }

    #[test]
    fn test_width() {
        assert_eq!(texture_width(&sample()), 1024);
    }

    #[test]
    fn test_height() {
        assert_eq!(texture_height(&sample()), 1024);
    }

    #[test]
    fn test_format_name() {
        assert_eq!(texture_format_name(&sample()), "RGBA8");
    }

    #[test]
    fn test_mip_count() {
        assert_eq!(texture_mip_count(&sample()), 10);
    }

    #[test]
    fn test_to_json() {
        let j = texture_to_json(&sample());
        assert!(j.contains("\"diffuse\""));
        assert!(j.contains("\"width\":1024"));
    }

    #[test]
    fn test_info_size() {
        assert!(texture_info_size(&sample()) > 0);
    }

    #[test]
    fn test_validate_ok() {
        assert!(validate_texture_info(&sample()));
    }

    #[test]
    fn test_validate_zero_width() {
        let t = export_texture_info("x", 0, 512, "RGB8", 1);
        assert!(!validate_texture_info(&t));
    }

    #[test]
    fn test_validate_empty_name() {
        let t = export_texture_info("", 512, 512, "RGB8", 1);
        assert!(!validate_texture_info(&t));
    }

    #[test]
    fn test_validate_empty_format() {
        let t = export_texture_info("n", 512, 512, "", 1);
        assert!(!validate_texture_info(&t));
    }
}
