// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cel shading / toon render view stub.

/// Number of shading bands for cel shading.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadeBands {
    Two,
    Three,
    Four,
    Custom(u32),
}

/// Cel shade view configuration.
#[derive(Debug, Clone)]
pub struct CelShadeView {
    pub bands: ShadeBands,
    pub outline_width: f32,
    pub outline_color: [f32; 4],
    pub specular: bool,
    pub enabled: bool,
}

impl CelShadeView {
    pub fn new() -> Self {
        CelShadeView {
            bands: ShadeBands::Three,
            outline_width: 1.5,
            outline_color: [0.0, 0.0, 0.0, 1.0],
            specular: true,
            enabled: true,
        }
    }
}

impl Default for CelShadeView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new cel shade view.
pub fn new_cel_shade_view() -> CelShadeView {
    CelShadeView::new()
}

/// Set shade band count.
pub fn clv_set_bands(view: &mut CelShadeView, bands: ShadeBands) {
    view.bands = bands;
}

/// Set outline width.
pub fn clv_set_outline_width(view: &mut CelShadeView, width: f32) {
    view.outline_width = width.clamp(0.0, 10.0);
}

/// Set outline color RGBA.
pub fn clv_set_outline_color(view: &mut CelShadeView, r: f32, g: f32, b: f32, a: f32) {
    view.outline_color = [
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
        a.clamp(0.0, 1.0),
    ];
}

/// Toggle specular highlight.
pub fn clv_set_specular(view: &mut CelShadeView, specular: bool) {
    view.specular = specular;
}

/// Enable or disable.
pub fn clv_set_enabled(view: &mut CelShadeView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn clv_to_json(view: &CelShadeView) -> String {
    let bands = match view.bands {
        ShadeBands::Two => "2".to_string(),
        ShadeBands::Three => "3".to_string(),
        ShadeBands::Four => "4".to_string(),
        ShadeBands::Custom(n) => n.to_string(),
    };
    format!(
        r#"{{"bands":{},"outline_width":{},"specular":{},"enabled":{}}}"#,
        bands, view.outline_width, view.specular, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bands() {
        let v = new_cel_shade_view();
        assert_eq!(
            v.bands,
            ShadeBands::Three /* default bands must be Three */
        );
    }

    #[test]
    fn test_set_bands() {
        let mut v = new_cel_shade_view();
        clv_set_bands(&mut v, ShadeBands::Four);
        assert_eq!(v.bands, ShadeBands::Four /* bands must be set */);
    }

    #[test]
    fn test_custom_bands() {
        let mut v = new_cel_shade_view();
        clv_set_bands(&mut v, ShadeBands::Custom(6));
        assert_eq!(
            v.bands,
            ShadeBands::Custom(6) /* custom band count must be stored */
        );
    }

    #[test]
    fn test_outline_width_clamped() {
        let mut v = new_cel_shade_view();
        clv_set_outline_width(&mut v, 100.0);
        assert!((v.outline_width - 10.0).abs() < 1e-6 /* outline_width clamped to 10.0 */);
    }

    #[test]
    fn test_set_outline_color() {
        let mut v = new_cel_shade_view();
        clv_set_outline_color(&mut v, 1.0, 0.0, 0.0, 1.0);
        assert!((v.outline_color[0] - 1.0).abs() < 1e-6 /* red channel must be 1.0 */);
    }

    #[test]
    fn test_set_specular() {
        let mut v = new_cel_shade_view();
        clv_set_specular(&mut v, false);
        assert!(!v.specular /* specular must be disabled */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_cel_shade_view();
        clv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_bands() {
        let v = new_cel_shade_view();
        let j = clv_to_json(&v);
        assert!(j.contains("\"bands\"") /* JSON must have bands */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_cel_shade_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_outline_width() {
        let v = new_cel_shade_view();
        assert!((v.outline_width - 1.5).abs() < 1e-6 /* default outline_width must be 1.5 */);
    }
}
