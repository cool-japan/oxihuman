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

/// Encode a Pascal string padded to a multiple of 4 bytes (length byte + chars + zero padding).
///
/// The total length of the output (1 + name_len + padding) is always a multiple of 4.
pub fn encode_pascal_string_padded4(name: &str) -> Vec<u8> {
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len().min(255);
    let raw_len = 1 + name_len;
    let padded_len = (raw_len + 3) & !3; // round up to multiple of 4
    let mut out = Vec::with_capacity(padded_len);
    out.push(name_len as u8);
    out.extend_from_slice(&name_bytes[..name_len]);
    out.resize(padded_len, 0u8);
    out
}

// ─── internal write helpers ──────────────────────────────────────────────────

#[inline]
fn push_u16_be(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_be_bytes());
}

#[inline]
fn push_u32_be(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_be_bytes());
}

#[inline]
fn push_i16_be(buf: &mut Vec<u8>, v: i16) {
    buf.extend_from_slice(&v.to_be_bytes());
}

/// Write Section 1 – File Header (26 bytes) into `buf`.
fn write_file_header(buf: &mut Vec<u8>, doc: &PsdExport) {
    buf.extend_from_slice(b"8BPS"); // signature
    push_u16_be(buf, 1); // version = 1
    buf.extend_from_slice(&[0u8; 6]); // reserved
    push_u16_be(buf, 4); // channels = 4 (RGBA)
    push_u32_be(buf, doc.height);
    push_u32_be(buf, doc.width);
    push_u16_be(buf, 8); // bits per channel
    push_u16_be(buf, 3); // color mode = RGB
}

/// Write Section 2 – Color Mode Data (4 bytes).
fn write_color_mode_data(buf: &mut Vec<u8>) {
    push_u32_be(buf, 0); // no color mode data for RGB
}

/// Write Section 3 – Image Resources (32 bytes: 4-byte length + 28-byte block).
fn write_image_resources(buf: &mut Vec<u8>) {
    // Resource block size = 28 bytes
    push_u32_be(buf, 28);
    buf.extend_from_slice(b"8BIM"); // signature
    push_u16_be(buf, 0x03ED); // resource ID 1005 = resolution info
    push_u16_be(buf, 0x0000); // pascal string name (empty, padded to even)
    push_u32_be(buf, 16); // resource data length
    // ResolutionInfo: hRes (72 DPI fixed-point), hResUnit, widthUnit, vRes, vResUnit, heightUnit
    push_u32_be(buf, 0x0048_0000); // 72 << 16
    push_u16_be(buf, 1); // hResUnit = pixels per inch
    push_u16_be(buf, 1); // widthUnit
    push_u32_be(buf, 0x0048_0000); // vRes
    push_u16_be(buf, 1); // vResUnit
    push_u16_be(buf, 1); // heightUnit
}

/// Build the raw bytes for the layer info sub-section (everything inside the
/// layer_info length field: layer_count + layer records + channel pixel data).
fn build_layer_info(doc: &PsdExport) -> Vec<u8> {
    let mut li: Vec<u8> = Vec::new();

    // layer_count (positive = no merged alpha)
    push_i16_be(&mut li, doc.layers.len() as i16);

    // Pre-calculate per-layer channel data length = 2 (compression u16) + w*h bytes
    // We iterate layers in reverse order (last to first) per PSD spec.
    let layers_rev: Vec<&PsdLayer> = doc.layers.iter().rev().collect();

    // --- Layer records ---
    for layer in &layers_rev {
        let w = layer.width;
        let h = layer.height;
        let pixel_data_len = 2u32 + w * h; // compression(2) + raw pixels

        // Bounding box
        push_i16_be(&mut li, 0); // top
        push_i16_be(&mut li, 0); // left
        push_i16_be(&mut li, h as i16); // bottom
        push_i16_be(&mut li, w as i16); // right

        // Channels (4): alpha(-1), R(0), G(1), B(2)
        push_u16_be(&mut li, 4);
        for channel_id in &[-1i16, 0, 1, 2] {
            push_i16_be(&mut li, *channel_id);
            push_u32_be(&mut li, pixel_data_len);
        }

        // Blend mode
        buf_extend_blend(&mut li, layer.blend_mode);

        // Opacity, clipping, flags, filler
        li.push(layer.opacity);
        li.push(0); // clipping = base
        li.push(if layer.visible { 0 } else { 2 }); // flags bit 1 = invisible
        li.push(0); // filler

        // Extra data: mask(4) + blending_ranges(4) + pascal name
        let name_enc = encode_pascal_string_padded4(&layer.name);
        let extra_len = 4u32 + 4 + name_enc.len() as u32;
        push_u32_be(&mut li, extra_len);
        push_u32_be(&mut li, 0); // mask data length = 0
        push_u32_be(&mut li, 0); // blending ranges length = 0
        li.extend_from_slice(&name_enc);
    }

    // --- Channel pixel data for each layer (same reversed order) ---
    for layer in &layers_rev {
        let w = layer.width as usize;
        let h = layer.height as usize;
        let n = w * h;

        // Channels in order: alpha(-1), R(0), G(1), B(2)
        // pixels are RGBA so index mapping: R=0, G=1, B=2, A=3
        for component_idx in [3usize, 0, 1, 2] {
            push_u16_be(&mut li, 0); // compression = raw
            // raw pixel data
            if layer.pixels.len() >= n {
                for px in &layer.pixels[..n] {
                    li.push(px[component_idx]);
                }
            } else {
                // Layer pixel data is inconsistent; fill with 255 (opaque white)
                li.extend(std::iter::repeat_n(255u8, n));
            }
        }
    }

    li
}

/// Push 4-byte blend mode key + "8BIM" signature into buf.
fn buf_extend_blend(buf: &mut Vec<u8>, blend_mode: PsdBlendMode) {
    buf.extend_from_slice(b"8BIM");
    buf.extend_from_slice(blend_mode.fourcc().as_bytes());
}

/// Write Section 4 – Layer and Mask Info into `buf`.
fn write_layer_and_mask_info(buf: &mut Vec<u8>, doc: &PsdExport) {
    let mut li = build_layer_info(doc);

    // Pad layer_info to even length
    if !li.len().is_multiple_of(2) {
        li.push(0);
    }

    let layer_info_len = li.len() as u32;

    // outer_length = 4 (layer_info_length field) + layer_info_len
    let outer_len = 4u32 + layer_info_len;
    push_u32_be(buf, outer_len); // outer section length
    push_u32_be(buf, layer_info_len); // layer_info sub-section length
    buf.extend_from_slice(&li);
}

/// Write Section 5 – Merged Image Data (composite) into `buf`.
fn write_merged_image_data(buf: &mut Vec<u8>, doc: &PsdExport) {
    let w = doc.width as usize;
    let h = doc.height as usize;
    let n = w * h;

    push_u16_be(buf, 0); // compression = raw uncompressed

    // Planar order: R, G, B, A
    // Use first layer's pixels if available, else white/opaque fallback.
    if let Some(layer) = doc.layers.first() {
        let pixels = &layer.pixels;
        let have = pixels.len().min(n);
        for component_idx in [0usize, 1, 2, 3] {
            for px in pixels.iter().take(have) {
                buf.push(px[component_idx]);
            }
            // If layer is smaller than the document canvas, pad with 255 (white/opaque).
            for _ in have..n {
                buf.push(255);
            }
        }
    } else {
        // No layers: white opaque composite
        for component_idx in 0..4usize {
            let fill: u8 = 255; // white R/G/B, full A
            let _ = component_idx;
            for _ in 0..n {
                buf.push(fill);
            }
        }
    }
}

/// Encode a PSD document to its binary representation.
///
/// Produces a valid Adobe Photoshop PSD (version 1) file that can be opened
/// by any PSD-compatible application. Layers are stored uncompressed (raw).
pub fn to_psd_bytes(doc: &PsdExport) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(estimate_psd_bytes(doc));

    write_file_header(&mut buf, doc); // Section 1
    write_color_mode_data(&mut buf); // Section 2
    write_image_resources(&mut buf); // Section 3
    write_layer_and_mask_info(&mut buf, doc); // Section 4
    write_merged_image_data(&mut buf, doc); // Section 5

    buf
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

    // ── to_psd_bytes tests ────────────────────────────────────────────────────

    #[test]
    fn test_to_psd_bytes_non_empty() {
        /* encoded output is non-empty for a valid document */
        let bytes = to_psd_bytes(&sample_doc());
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_to_psd_bytes_magic() {
        /* encoded output starts with the PSD magic bytes "8BPS" */
        let bytes = to_psd_bytes(&sample_doc());
        assert!(
            bytes.starts_with(b"8BPS"),
            "expected magic b\"8BPS\" at offset 0"
        );
    }

    #[test]
    fn test_to_psd_bytes_single_layer() {
        /* single 2×2 RGBA layer produces a parseable section structure */
        let mut doc = PsdExport::new(2, 2);
        doc.add_layer(PsdLayer::new_solid("bg", 2, 2, [10, 20, 30, 255]));
        let bytes = to_psd_bytes(&doc);

        // File header = 26 bytes, always present
        assert!(bytes.len() >= 26, "output too short to contain file header");

        // Magic
        assert_eq!(&bytes[0..4], b"8BPS");
        // Version = 1 (big-endian u16)
        assert_eq!(u16::from_be_bytes([bytes[4], bytes[5]]), 1u16);
        // channels = 4
        assert_eq!(u16::from_be_bytes([bytes[12], bytes[13]]), 4u16);
        // height = 2
        assert_eq!(u32::from_be_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]), 2u32);
        // width = 2
        assert_eq!(u32::from_be_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]), 2u32);
        // depth = 8
        assert_eq!(u16::from_be_bytes([bytes[22], bytes[23]]), 8u16);
        // color mode = 3 (RGB)
        assert_eq!(u16::from_be_bytes([bytes[24], bytes[25]]), 3u16);

        // Section 2 starts at offset 26: 4-byte length = 0
        assert_eq!(u32::from_be_bytes([bytes[26], bytes[27], bytes[28], bytes[29]]), 0u32);

        // Section 3 starts at offset 30: 4-byte length = 28
        assert_eq!(u32::from_be_bytes([bytes[30], bytes[31], bytes[32], bytes[33]]), 28u32);
        // "8BIM" at offset 34
        assert_eq!(&bytes[34..38], b"8BIM");
    }

    #[test]
    fn test_to_psd_bytes_empty_doc() {
        /* empty PsdExport produces a minimal valid PSD file */
        let doc = PsdExport::new(4, 4);
        let bytes = to_psd_bytes(&doc);

        // Must have file header + color mode + image resources + section 4 outer len
        assert!(bytes.len() >= 26 + 4 + 32 + 4, "output too short for empty doc");

        // Magic bytes
        assert_eq!(&bytes[0..4], b"8BPS");

        // Color mode section length = 0
        assert_eq!(u32::from_be_bytes([bytes[26], bytes[27], bytes[28], bytes[29]]), 0u32);

        // Image resources section length = 28
        assert_eq!(u32::from_be_bytes([bytes[30], bytes[31], bytes[32], bytes[33]]), 28u32);

        // Section 4 starts at offset 26 + 4 + 32 = 62
        // outer_length field: layer_info_length(4) + layer_info_payload(2, just the i16 count=0)
        // = 4 + 2 = 6, but layer_info is padded to even → 2 bytes → outer = 4+2 = 6
        let sec4_outer = u32::from_be_bytes([bytes[62], bytes[63], bytes[64], bytes[65]]);
        assert!(sec4_outer >= 4, "section 4 outer length too small for empty doc");
    }

    #[test]
    fn test_encode_pascal_string_padded4() {
        /* pascal string is padded to multiples of 4 */
        let enc = encode_pascal_string_padded4("bg");
        // len=1, "bg"=2 → raw 3 bytes → padded to 4
        assert_eq!(enc.len(), 4);
        assert_eq!(enc[0], 2u8);
        assert_eq!(&enc[1..3], b"bg");

        let enc2 = encode_pascal_string_padded4("Background");
        // len=1, "Background"=10 → raw 11 → padded to 12
        assert_eq!(enc2.len(), 12);
        assert_eq!(enc2[0], 10u8);

        let enc3 = encode_pascal_string_padded4("");
        // len=1, ""=0 → raw 1 → padded to 4
        assert_eq!(enc3.len(), 4);
        assert_eq!(enc3[0], 0u8);
    }
}
