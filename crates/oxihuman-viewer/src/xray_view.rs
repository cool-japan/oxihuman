// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! X-ray visualization mode stub.

/// X-ray rendering style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum XrayStyle {
    Silhouette,
    Skeleton,
    Density,
}

/// X-ray view configuration.
#[derive(Debug, Clone)]
pub struct XrayView {
    pub style: XrayStyle,
    pub opacity: f32,
    pub edge_intensity: f32,
    pub background_color: [f32; 4],
    pub xray_color: [f32; 4],
    pub enabled: bool,
}

impl XrayView {
    pub fn new() -> Self {
        XrayView {
            style: XrayStyle::Silhouette,
            opacity: 0.7,
            edge_intensity: 1.0,
            background_color: [0.0, 0.0, 0.0, 1.0],
            xray_color: [0.0, 0.8, 1.0, 1.0],
            enabled: true,
        }
    }
}

impl Default for XrayView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new x-ray view.
pub fn new_xray_view() -> XrayView {
    XrayView::new()
}

/// Compute blended pixel color (stub: returns xray_color).
pub fn xrv_blend_pixel(xrv: &XrayView, _surface_color: [f32; 4]) -> [f32; 4] {
    /* Stub: returns xray_color modulated by opacity */
    [
        xrv.xray_color[0] * xrv.opacity,
        xrv.xray_color[1] * xrv.opacity,
        xrv.xray_color[2] * xrv.opacity,
        xrv.xray_color[3],
    ]
}

/// Set rendering style.
pub fn xrv_set_style(xrv: &mut XrayView, style: XrayStyle) {
    xrv.style = style;
}

/// Set opacity.
pub fn xrv_set_opacity(xrv: &mut XrayView, opacity: f32) {
    xrv.opacity = opacity.clamp(0.0, 1.0);
}

/// Set edge intensity.
pub fn xrv_set_edge_intensity(xrv: &mut XrayView, intensity: f32) {
    xrv.edge_intensity = intensity.clamp(0.0, 2.0);
}

/// Enable or disable.
pub fn xrv_set_enabled(xrv: &mut XrayView, enabled: bool) {
    xrv.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn xrv_to_json(xrv: &XrayView) -> String {
    let style = match xrv.style {
        XrayStyle::Silhouette => "silhouette",
        XrayStyle::Skeleton => "skeleton",
        XrayStyle::Density => "density",
    };
    format!(
        r#"{{"style":"{}","opacity":{},"edge_intensity":{},"enabled":{}}}"#,
        style, xrv.opacity, xrv.edge_intensity, xrv.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_style_silhouette() {
        let x = new_xray_view();
        assert_eq!(
            x.style,
            XrayStyle::Silhouette, /* default style must be silhouette */
        );
    }

    #[test]
    fn test_default_opacity() {
        let x = new_xray_view();
        assert!((x.opacity - 0.7).abs() < 1e-5, /* default opacity must be 0.7 */);
    }

    #[test]
    fn test_blend_pixel_uses_opacity() {
        let x = new_xray_view();
        let out = xrv_blend_pixel(&x, [1.0; 4]);
        assert!(
            (out[1] - x.xray_color[1] * x.opacity).abs() < 1e-5, /* blend must apply opacity */
        );
    }

    #[test]
    fn test_set_style() {
        let mut x = new_xray_view();
        xrv_set_style(&mut x, XrayStyle::Skeleton);
        assert_eq!(x.style, XrayStyle::Skeleton /* style must be set */,);
    }

    #[test]
    fn test_set_opacity_clamped() {
        let mut x = new_xray_view();
        xrv_set_opacity(&mut x, 2.0);
        assert!((x.opacity - 1.0).abs() < 1e-6, /* opacity clamped to 1.0 */);
    }

    #[test]
    fn test_set_edge_intensity() {
        let mut x = new_xray_view();
        xrv_set_edge_intensity(&mut x, 1.5);
        assert!((x.edge_intensity - 1.5).abs() < 1e-5, /* edge intensity must be set */);
    }

    #[test]
    fn test_edge_intensity_clamped() {
        let mut x = new_xray_view();
        xrv_set_edge_intensity(&mut x, 3.0);
        assert!((x.edge_intensity - 2.0).abs() < 1e-6, /* edge intensity clamped to 2.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut x = new_xray_view();
        xrv_set_enabled(&mut x, false);
        assert!(!x.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_style() {
        let x = new_xray_view();
        let j = xrv_to_json(&x);
        assert!(j.contains("\"style\"") /* json must contain style */,);
    }

    #[test]
    fn test_enabled_default() {
        let x = new_xray_view();
        assert!(x.enabled /* must be enabled by default */,);
    }
}
