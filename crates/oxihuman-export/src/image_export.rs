#![allow(dead_code)]
//! Image export stub.

/// Image export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ImageExport {
    pub width: u32,
    pub height: u32,
    pub channels: u32,
    pub format: String,
    pub data: Vec<u8>,
}

/// Export an image stub.
#[allow(dead_code)]
pub fn export_image_stub(width: u32, height: u32, channels: u32, format: &str) -> ImageExport {
    let size = (width * height * channels) as usize;
    ImageExport {
        width,
        height,
        channels,
        format: format.to_string(),
        data: vec![0u8; size],
    }
}

/// Get image width.
#[allow(dead_code)]
pub fn image_width_ie(e: &ImageExport) -> u32 {
    e.width
}

/// Get image height.
#[allow(dead_code)]
pub fn image_height_ie(e: &ImageExport) -> u32 {
    e.height
}

/// Get image format.
#[allow(dead_code)]
pub fn image_format(e: &ImageExport) -> &str {
    &e.format
}

/// Get image data as bytes.
#[allow(dead_code)]
pub fn image_to_bytes(e: &ImageExport) -> &[u8] {
    &e.data
}

/// Get channel count.
#[allow(dead_code)]
pub fn image_channels(e: &ImageExport) -> u32 {
    e.channels
}

/// Get export size.
#[allow(dead_code)]
pub fn image_export_size(e: &ImageExport) -> usize {
    e.data.len()
}

/// Validate image.
#[allow(dead_code)]
pub fn validate_image(e: &ImageExport) -> bool {
    e.width > 0
        && e.height > 0
        && e.channels > 0
        && !e.format.is_empty()
        && e.data.len() == (e.width * e.height * e.channels) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_image_stub() {
        let e = export_image_stub(4, 4, 4, "RGBA");
        assert_eq!(e.data.len(), 64);
    }

    #[test]
    fn test_image_width() {
        let e = export_image_stub(8, 4, 3, "RGB");
        assert_eq!(image_width_ie(&e), 8);
    }

    #[test]
    fn test_image_height() {
        let e = export_image_stub(8, 4, 3, "RGB");
        assert_eq!(image_height_ie(&e), 4);
    }

    #[test]
    fn test_image_format() {
        let e = export_image_stub(1, 1, 4, "RGBA");
        assert_eq!(image_format(&e), "RGBA");
    }

    #[test]
    fn test_image_to_bytes() {
        let e = export_image_stub(2, 2, 1, "GRAY");
        assert_eq!(image_to_bytes(&e).len(), 4);
    }

    #[test]
    fn test_image_channels() {
        let e = export_image_stub(1, 1, 3, "RGB");
        assert_eq!(image_channels(&e), 3);
    }

    #[test]
    fn test_image_export_size() {
        let e = export_image_stub(2, 2, 4, "RGBA");
        assert_eq!(image_export_size(&e), 16);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_image_stub(2, 2, 4, "RGBA");
        assert!(validate_image(&e));
    }

    #[test]
    fn test_validate_zero_width() {
        let e = export_image_stub(0, 2, 4, "RGBA");
        assert!(!validate_image(&e));
    }

    #[test]
    fn test_validate_empty_format() {
        let mut e = export_image_stub(2, 2, 4, "RGBA");
        e.format = String::new();
        assert!(!validate_image(&e));
    }
}
