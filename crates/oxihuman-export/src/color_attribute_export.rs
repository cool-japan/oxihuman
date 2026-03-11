// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Vertex color attribute export.

#![allow(dead_code)]

/// Configuration for color attribute export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorAttributeConfig {
    /// Name of the color attribute layer.
    pub name: String,
    /// Color space identifier (e.g. "sRGB", "Linear").
    pub color_space: String,
}

/// Container for a per-vertex color attribute.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorAttributeExport {
    /// Layer configuration.
    pub config: ColorAttributeConfig,
    /// Per-vertex RGBA colors (each channel 0.0–1.0).
    pub colors: Vec<[f32; 4]>,
}

/// Returns the default [`ColorAttributeConfig`].
#[allow(dead_code)]
pub fn default_color_attribute_config() -> ColorAttributeConfig {
    ColorAttributeConfig {
        name: "Col".to_string(),
        color_space: "sRGB".to_string(),
    }
}

/// Creates a new [`ColorAttributeExport`] with `vertex_count` white vertices.
#[allow(dead_code)]
pub fn new_color_attribute_export(config: ColorAttributeConfig, vertex_count: usize) -> ColorAttributeExport {
    ColorAttributeExport {
        config,
        colors: vec![[1.0, 1.0, 1.0, 1.0]; vertex_count],
    }
}

/// Sets the color at `index`.
#[allow(dead_code)]
pub fn ca_set_color(export: &mut ColorAttributeExport, index: usize, color: [f32; 4]) {
    if index < export.colors.len() {
        export.colors[index] = color;
    }
}

/// Returns the color at `index`, or `None` if out of range.
#[allow(dead_code)]
pub fn ca_get_color(export: &ColorAttributeExport, index: usize) -> Option<[f32; 4]> {
    export.colors.get(index).copied()
}

/// Returns the number of vertices.
#[allow(dead_code)]
pub fn ca_vertex_count(export: &ColorAttributeExport) -> usize {
    export.colors.len()
}

/// Serialises to a minimal JSON string.
#[allow(dead_code)]
pub fn ca_to_json(export: &ColorAttributeExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"color_space\":\"{}\",\"vertex_count\":{}}}",
        export.config.name, export.config.color_space, export.colors.len()
    )
}

/// Returns all colors packed as RGBA u8 bytes.
#[allow(dead_code)]
pub fn ca_to_bytes_rgba_u8(export: &ColorAttributeExport) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(export.colors.len() * 4);
    for &c in &export.colors {
        for ch in &c {
            bytes.push((ch.clamp(0.0, 1.0) * 255.0).round() as u8);
        }
    }
    bytes
}

/// Validates that all color values are in the 0.0–1.0 range.
#[allow(dead_code)]
pub fn ca_validate(export: &ColorAttributeExport) -> bool {
    export.colors.iter().all(|c| c.iter().all(|&v| (0.0..=1.0).contains(&v)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_color_attribute_config();
        assert_eq!(cfg.name, "Col");
    }

    #[test]
    fn test_new_export() {
        let export = new_color_attribute_export(default_color_attribute_config(), 4);
        assert_eq!(ca_vertex_count(&export), 4);
    }

    #[test]
    fn test_set_and_get_color() {
        let mut export = new_color_attribute_export(default_color_attribute_config(), 4);
        ca_set_color(&mut export, 0, [1.0, 0.0, 0.0, 1.0]);
        let c = ca_get_color(&export, 0).unwrap();
        assert_eq!(c[0], 1.0);
        assert_eq!(c[1], 0.0);
    }

    #[test]
    fn test_get_out_of_range() {
        let export = new_color_attribute_export(default_color_attribute_config(), 2);
        assert!(ca_get_color(&export, 99).is_none());
    }

    #[test]
    fn test_to_json() {
        let export = new_color_attribute_export(default_color_attribute_config(), 3);
        let json = ca_to_json(&export);
        assert!(json.contains("vertex_count"));
    }

    #[test]
    fn test_to_bytes_rgba_u8() {
        let export = new_color_attribute_export(default_color_attribute_config(), 1);
        let bytes = ca_to_bytes_rgba_u8(&export);
        assert_eq!(bytes.len(), 4);
        assert_eq!(bytes[0], 255); // white R
    }

    #[test]
    fn test_validate_valid() {
        let export = new_color_attribute_export(default_color_attribute_config(), 4);
        assert!(ca_validate(&export));
    }

    #[test]
    fn test_validate_invalid() {
        let mut export = new_color_attribute_export(default_color_attribute_config(), 2);
        ca_set_color(&mut export, 0, [2.0, 0.0, 0.0, 1.0]); // invalid
        assert!(!ca_validate(&export));
    }

    #[test]
    fn test_bytes_length() {
        let export = new_color_attribute_export(default_color_attribute_config(), 5);
        let bytes = ca_to_bytes_rgba_u8(&export);
        assert_eq!(bytes.len(), 20);
    }
}
