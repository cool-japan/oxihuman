// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Photoshop PSD stub export.

/// PSD layer blend mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PsdBlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
}

impl PsdBlendMode {
    /// Four-character code for this blend mode.
    pub fn fourcc(&self) -> &'static str {
        match self {
            Self::Normal => "norm",
            Self::Multiply => "mul ",
            Self::Screen => "scrn",
            Self::Overlay => "over",
        }
    }
}

/// A PSD layer.
#[derive(Debug, Clone)]
pub struct PsdLayer {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub opacity: u8,
    pub blend_mode: PsdBlendMode,
    pub visible: bool,
    pub pixels: Vec<[u8; 4]>,
}

impl PsdLayer {
    /// Create a new PSD layer filled with solid color.
    pub fn new_solid(name: &str, width: u32, height: u32, color: [u8; 4]) -> Self {
        let pixels = vec![color; (width * height) as usize];
        Self {
            name: name.to_string(),
            width,
            height,
            opacity: 255,
            blend_mode: PsdBlendMode::Normal,
            visible: true,
            pixels,
        }
    }

    /// Pixel count.
    pub fn pixel_count(&self) -> usize {
        self.pixels.len()
    }
}

/// PSD document stub.
#[derive(Debug, Clone)]
pub struct PsdExport {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<PsdLayer>,
}

impl PsdExport {
    /// Create a new PSD document.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            layers: Vec::new(),
        }
    }

    /// Add a layer.
    pub fn add_layer(&mut self, layer: PsdLayer) {
        self.layers.push(layer);
    }

    /// Return layer count.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Count visible layers.
    pub fn visible_layer_count(&self) -> usize {
        self.layers.iter().filter(|l| l.visible).count()
    }
}

/// Validate PSD document.
pub fn validate_psd(doc: &PsdExport) -> bool {
    doc.width > 0 && doc.height > 0
}

/// Estimate PSD file size (stub).
pub fn estimate_psd_bytes(doc: &PsdExport) -> usize {
    let layer_data: usize = doc.layers.iter().map(|l| l.pixel_count() * 4 + 256).sum();
    26 + layer_data + (doc.width * doc.height) as usize * 4
}

/// Serialize PSD metadata to JSON (stub).
pub fn psd_metadata_json(doc: &PsdExport) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"layers\":{}}}",
        doc.width,
        doc.height,
        doc.layer_count()
    )
}

/// Find layer by name.
pub fn find_psd_layer<'a>(doc: &'a PsdExport, name: &str) -> Option<&'a PsdLayer> {
    doc.layers.iter().find(|l| l.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> PsdExport {
        let mut doc = PsdExport::new(128, 128);
        doc.add_layer(PsdLayer::new_solid(
            "Background",
            128,
            128,
            [200, 200, 200, 255],
        ));
        let mut fg = PsdLayer::new_solid("Foreground", 128, 128, [100, 0, 0, 200]);
        fg.blend_mode = PsdBlendMode::Multiply;
        doc.add_layer(fg);
        doc
    }

    #[test]
    fn test_layer_count() {
        /* document has correct layer count */
        assert_eq!(sample_doc().layer_count(), 2);
    }

    #[test]
    fn test_visible_layer_count() {
        /* all layers visible by default */
        assert_eq!(sample_doc().visible_layer_count(), 2);
    }

    #[test]
    fn test_validate_valid() {
        /* valid document passes */
        assert!(validate_psd(&sample_doc()));
    }

    #[test]
    fn test_blend_mode_fourcc() {
        /* blend mode fourcc codes are distinct */
        assert_ne!(
            PsdBlendMode::Normal.fourcc(),
            PsdBlendMode::Multiply.fourcc()
        );
    }

    #[test]
    fn test_estimate_bytes_positive() {
        /* estimated size is positive */
        assert!(estimate_psd_bytes(&sample_doc()) > 0);
    }

    #[test]
    fn test_metadata_json() {
        /* metadata JSON contains layer count */
        let json = psd_metadata_json(&sample_doc());
        assert!(json.contains("layers"));
    }

    #[test]
    fn test_find_psd_layer() {
        /* find_psd_layer locates layer by name */
        let doc = sample_doc();
        assert!(find_psd_layer(&doc, "Background").is_some());
        assert!(find_psd_layer(&doc, "None").is_none());
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count matches dimensions */
        let l = PsdLayer::new_solid("L", 16, 16, [0; 4]);
        assert_eq!(l.pixel_count(), 256);
    }
}
