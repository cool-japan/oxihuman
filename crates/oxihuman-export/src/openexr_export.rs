// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! OpenEXR image stub export.

/// OpenEXR channel type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExrChannelType {
    Half,
    Float,
    Uint,
}

impl ExrChannelType {
    /// Return bytes per value.
    pub fn byte_size(&self) -> usize {
        match self {
            Self::Half => 2,
            Self::Float => 4,
            Self::Uint => 4,
        }
    }
}

/// An OpenEXR channel definition.
#[derive(Debug, Clone)]
pub struct ExrChannel {
    pub name: String,
    pub channel_type: ExrChannelType,
}

/// OpenEXR image stub.
#[derive(Debug, Clone)]
pub struct ExrExport {
    pub width: u32,
    pub height: u32,
    pub channels: Vec<ExrChannel>,
    pub pixels: Vec<Vec<f32>>,
}

impl ExrExport {
    /// Create a new EXR export.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            channels: Vec::new(),
            pixels: Vec::new(),
        }
    }

    /// Add a channel.
    pub fn add_channel(&mut self, name: &str, channel_type: ExrChannelType) {
        self.channels.push(ExrChannel {
            name: name.to_string(),
            channel_type,
        });
        let n = (self.width * self.height) as usize;
        self.pixels.push(vec![0.0_f32; n]);
    }

    /// Return channel count.
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Set a pixel value in a channel.
    pub fn set_pixel(&mut self, channel: usize, x: u32, y: u32, value: f32) {
        if channel < self.pixels.len() {
            let idx = (y * self.width + x) as usize;
            if let Some(v) = self.pixels[channel].get_mut(idx) {
                *v = value;
            }
        }
    }

    /// Get a pixel value from a channel.
    pub fn get_pixel(&self, channel: usize, x: u32, y: u32) -> f32 {
        if channel < self.pixels.len() {
            let idx = (y * self.width + x) as usize;
            self.pixels[channel].get(idx).copied().unwrap_or(0.0)
        } else {
            0.0
        }
    }
}

/// Estimate EXR file size (stub).
pub fn estimate_exr_bytes(export: &ExrExport) -> usize {
    let pixels_per_channel = (export.width * export.height) as usize;
    let bytes_per_channel: usize = export
        .channels
        .iter()
        .map(|c| c.channel_type.byte_size() * pixels_per_channel)
        .sum();
    bytes_per_channel + 2048
}

/// Serialize EXR metadata to JSON (stub).
pub fn exr_metadata_json(export: &ExrExport) -> String {
    let ch_names: Vec<String> = export
        .channels
        .iter()
        .map(|c| format!("\"{}\"", c.name))
        .collect();
    format!(
        "{{\"width\":{},\"height\":{},\"channels\":[{}]}}",
        export.width,
        export.height,
        ch_names.join(",")
    )
}

/// Find channel index by name.
pub fn find_channel(export: &ExrExport, name: &str) -> Option<usize> {
    export.channels.iter().position(|c| c.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba_exr() -> ExrExport {
        let mut e = ExrExport::new(4, 4);
        e.add_channel("R", ExrChannelType::Half);
        e.add_channel("G", ExrChannelType::Half);
        e.add_channel("B", ExrChannelType::Half);
        e.add_channel("A", ExrChannelType::Float);
        e
    }

    #[test]
    fn test_channel_count() {
        /* channel count is correct */
        assert_eq!(rgba_exr().channel_count(), 4);
    }

    #[test]
    fn test_set_get_pixel() {
        /* set/get round-trip is correct */
        let mut e = rgba_exr();
        e.set_pixel(0, 2, 1, 0.75);
        assert!((e.get_pixel(0, 2, 1) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_estimate_bytes_positive() {
        /* byte estimate is positive */
        assert!(estimate_exr_bytes(&rgba_exr()) > 0);
    }

    #[test]
    fn test_metadata_json_contains_channel() {
        /* metadata JSON contains channel names */
        let json = exr_metadata_json(&rgba_exr());
        assert!(json.contains("\"R\""));
    }

    #[test]
    fn test_find_channel_found() {
        /* find_channel locates existing channel */
        let e = rgba_exr();
        assert_eq!(find_channel(&e, "A"), Some(3));
    }

    #[test]
    fn test_find_channel_not_found() {
        /* find_channel returns None for missing channel */
        let e = rgba_exr();
        assert!(find_channel(&e, "Z").is_none());
    }

    #[test]
    fn test_channel_type_byte_size() {
        /* byte sizes are correct */
        assert_eq!(ExrChannelType::Half.byte_size(), 2);
        assert_eq!(ExrChannelType::Float.byte_size(), 4);
    }

    #[test]
    fn test_get_pixel_out_of_bounds() {
        /* out-of-bounds channel returns 0 */
        let e = rgba_exr();
        assert_eq!(e.get_pixel(99, 0, 0), 0.0);
    }
}
