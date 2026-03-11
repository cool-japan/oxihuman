// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Normal map encoding debug visualization.

/// Encoding scheme for normal map data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NormalEncoding {
    Rgb,
    RgDerived,
    Spheremap,
}

/// Configuration for normal encoding view.
#[derive(Debug, Clone)]
pub struct NormalEncodingViewConfig {
    pub encoding: NormalEncoding,
    pub flip_y: bool,
}

impl Default for NormalEncodingViewConfig {
    fn default() -> Self {
        Self { encoding: NormalEncoding::Rgb, flip_y: false }
    }
}

/// State for normal encoding visualization.
#[derive(Debug, Clone)]
pub struct NormalEncodingView {
    pub config: NormalEncodingViewConfig,
    pub enabled: bool,
}

impl Default for NormalEncodingView {
    fn default() -> Self {
        Self { config: NormalEncodingViewConfig::default(), enabled: false }
    }
}

/// Enable normal encoding view.
pub fn nev_enable(view: &mut NormalEncodingView) {
    view.enabled = true;
}

/// Disable normal encoding view.
pub fn nev_disable(view: &mut NormalEncodingView) {
    view.enabled = false;
}

/// Decode an RGB-encoded normal to a unit vector.
pub fn nev_decode_rgb(r: f32, g: f32, b: f32) -> [f32; 3] {
    [r * 2.0 - 1.0, g * 2.0 - 1.0, b * 2.0 - 1.0]
}

/// Re-encode a normal vector to an RGB display color.
pub fn nev_encode_to_color(nx: f32, ny: f32, nz: f32) -> [f32; 4] {
    [nx * 0.5 + 0.5, ny * 0.5 + 0.5, nz * 0.5 + 0.5, 1.0]
}

/// Set the encoding scheme.
pub fn nev_set_encoding(view: &mut NormalEncodingView, enc: NormalEncoding) {
    view.config.encoding = enc;
}

/// Export config to JSON string (stub).
pub fn nev_to_json(view: &NormalEncodingView) -> String {
    let enc = match view.config.encoding {
        NormalEncoding::Rgb => "rgb",
        NormalEncoding::RgDerived => "rg_derived",
        NormalEncoding::Spheremap => "spheremap",
    };
    format!(r#"{{"encoding":"{}","flip_y":{},"enabled":{}}}"#, enc, view.config.flip_y, view.enabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = NormalEncodingView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable/disable should toggle */
        let mut v = NormalEncodingView::default();
        nev_enable(&mut v);
        assert!(v.enabled);
        nev_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_decode_rgb_neutral() {
        /* (0.5, 0.5, 1.0) should decode to approximately (0, 0, 1) */
        let n = nev_decode_rgb(0.5, 0.5, 1.0);
        assert!(n[0].abs() < 1e-5);
        assert!(n[1].abs() < 1e-5);
        assert!((n[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_encode_to_color_neutral() {
        /* (0, 0, 1) normal should encode to (0.5, 0.5, 1.0) */
        let c = nev_encode_to_color(0.0, 0.0, 1.0);
        assert!((c[0] - 0.5).abs() < 1e-6);
        assert!((c[1] - 0.5).abs() < 1e-6);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_encode_alpha_one() {
        /* alpha should be 1.0 */
        let c = nev_encode_to_color(0.0, 1.0, 0.0);
        assert!((c[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_encoding() {
        /* encoding should be updated */
        let mut v = NormalEncodingView::default();
        nev_set_encoding(&mut v, NormalEncoding::Spheremap);
        assert_eq!(v.config.encoding, NormalEncoding::Spheremap);
    }

    #[test]
    fn test_roundtrip() {
        /* decode then re-encode should produce original color */
        let r = 0.6_f32;
        let g = 0.4_f32;
        let b = 0.8_f32;
        let n = nev_decode_rgb(r, g, b);
        let c = nev_encode_to_color(n[0], n[1], n[2]);
        assert!((c[0] - r).abs() < 1e-5);
        assert!((c[1] - g).abs() < 1e-5);
        assert!((c[2] - b).abs() < 1e-5);
    }

    #[test]
    fn test_to_json_encoding() {
        /* JSON should contain encoding field */
        let v = NormalEncodingView::default();
        let json = nev_to_json(&v);
        assert!(json.contains("rgb"));
    }

    #[test]
    fn test_default_encoding() {
        /* default encoding should be RGB */
        let v = NormalEncodingView::default();
        assert_eq!(v.config.encoding, NormalEncoding::Rgb);
    }
}
