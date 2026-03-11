#![allow(dead_code)]
//! Export vertex color data.

/// Color export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ColorExport {
    pub colors: Vec<[f32; 4]>,
}

/// Export vertex colors.
#[allow(dead_code)]
pub fn export_vertex_colors(colors: &[[f32; 4]]) -> ColorExport {
    ColorExport { colors: colors.to_vec() }
}

/// Return the color format string.
#[allow(dead_code)]
pub fn color_format() -> &'static str {
    "FLOAT4_RGBA"
}

/// Convert colors to bytes.
#[allow(dead_code)]
pub fn color_to_bytes(export: &ColorExport) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(export.colors.len() * 16);
    for c in &export.colors {
        for &v in c {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    bytes
}

/// Return the color count.
#[allow(dead_code)]
pub fn color_count_export(export: &ColorExport) -> usize {
    export.colors.len()
}

/// Convert RGBA to RGB (drop alpha).
#[allow(dead_code)]
pub fn rgba_to_rgb(color: [f32; 4]) -> [f32; 3] {
    [color[0], color[1], color[2]]
}

/// Convert RGB to RGBA (add alpha = 1.0).
#[allow(dead_code)]
pub fn rgb_to_rgba(color: [f32; 3]) -> [f32; 4] {
    [color[0], color[1], color[2], 1.0]
}

/// Validate that colors are in [0,1] range.
#[allow(dead_code)]
pub fn validate_colors(export: &ColorExport) -> bool {
    export.colors.iter().all(|c| {
        c.iter().all(|&v| (0.0..=1.0).contains(&v))
    })
}

/// Return the export size in bytes.
#[allow(dead_code)]
pub fn color_export_size(export: &ColorExport) -> usize {
    export.colors.len() * 16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_vertex_colors() {
        let c = vec![[1.0, 0.0, 0.0, 1.0]];
        let exp = export_vertex_colors(&c);
        assert_eq!(color_count_export(&exp), 1);
    }

    #[test]
    fn test_color_format() {
        assert_eq!(color_format(), "FLOAT4_RGBA");
    }

    #[test]
    fn test_color_to_bytes() {
        let exp = export_vertex_colors(&[[1.0, 0.0, 0.0, 1.0]]);
        let bytes = color_to_bytes(&exp);
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_color_count() {
        let exp = export_vertex_colors(&[[0.0; 4]; 5]);
        assert_eq!(color_count_export(&exp), 5);
    }

    #[test]
    fn test_rgba_to_rgb() {
        assert_eq!(rgba_to_rgb([1.0, 0.5, 0.0, 0.8]), [1.0, 0.5, 0.0]);
    }

    #[test]
    fn test_rgb_to_rgba() {
        assert_eq!(rgb_to_rgba([1.0, 0.5, 0.0]), [1.0, 0.5, 0.0, 1.0]);
    }

    #[test]
    fn test_validate_colors() {
        let exp = export_vertex_colors(&[[0.5, 0.5, 0.5, 1.0]]);
        assert!(validate_colors(&exp));
    }

    #[test]
    fn test_validate_colors_bad() {
        let exp = export_vertex_colors(&[[1.5, 0.0, 0.0, 1.0]]);
        assert!(!validate_colors(&exp));
    }

    #[test]
    fn test_color_export_size() {
        let exp = export_vertex_colors(&[[0.0; 4]; 3]);
        assert_eq!(color_export_size(&exp), 48);
    }

    #[test]
    fn test_empty_colors() {
        let exp = export_vertex_colors(&[]);
        assert_eq!(color_count_export(&exp), 0);
        assert!(validate_colors(&exp));
    }
}
