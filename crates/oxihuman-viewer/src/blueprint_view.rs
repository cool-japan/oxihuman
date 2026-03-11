// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Technical blueprint overlay stub.

/// Blueprint color scheme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlueprintScheme {
    Classic,
    WhiteOnBlue,
    BlackOnWhite,
}

/// Blueprint view configuration.
#[derive(Debug, Clone)]
pub struct BlueprintView {
    pub scheme: BlueprintScheme,
    pub grid_spacing: f32,
    pub line_width: f32,
    pub annotation_scale: f32,
    pub show_dimensions: bool,
    pub enabled: bool,
}

impl BlueprintView {
    pub fn new() -> Self {
        BlueprintView {
            scheme: BlueprintScheme::Classic,
            grid_spacing: 32.0,
            line_width: 1.0,
            annotation_scale: 1.0,
            show_dimensions: true,
            enabled: true,
        }
    }
}

impl Default for BlueprintView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new blueprint view.
pub fn new_blueprint_view() -> BlueprintView {
    BlueprintView::new()
}

/// Set color scheme.
pub fn bpv_set_scheme(view: &mut BlueprintView, scheme: BlueprintScheme) {
    view.scheme = scheme;
}

/// Set grid spacing.
pub fn bpv_set_grid_spacing(view: &mut BlueprintView, spacing: f32) {
    view.grid_spacing = spacing.clamp(4.0, 256.0);
}

/// Set line width.
pub fn bpv_set_line_width(view: &mut BlueprintView, width: f32) {
    view.line_width = width.clamp(0.5, 4.0);
}

/// Toggle dimension annotations.
pub fn bpv_set_show_dimensions(view: &mut BlueprintView, show: bool) {
    view.show_dimensions = show;
}

/// Enable or disable.
pub fn bpv_set_enabled(view: &mut BlueprintView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn bpv_to_json(view: &BlueprintView) -> String {
    let scheme = match view.scheme {
        BlueprintScheme::Classic => "classic",
        BlueprintScheme::WhiteOnBlue => "white_on_blue",
        BlueprintScheme::BlackOnWhite => "black_on_white",
    };
    format!(
        r#"{{"scheme":"{}","grid_spacing":{},"line_width":{},"show_dimensions":{},"enabled":{}}}"#,
        scheme, view.grid_spacing, view.line_width, view.show_dimensions, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_scheme() {
        let v = new_blueprint_view();
        assert_eq!(
            v.scheme,
            BlueprintScheme::Classic /* default scheme must be Classic */
        );
    }

    #[test]
    fn test_set_scheme() {
        let mut v = new_blueprint_view();
        bpv_set_scheme(&mut v, BlueprintScheme::WhiteOnBlue);
        assert_eq!(
            v.scheme,
            BlueprintScheme::WhiteOnBlue /* scheme must be set */
        );
    }

    #[test]
    fn test_grid_spacing_clamped() {
        let mut v = new_blueprint_view();
        bpv_set_grid_spacing(&mut v, 0.0);
        assert!((v.grid_spacing - 4.0).abs() < 1e-6 /* grid_spacing clamped to 4.0 */);
    }

    #[test]
    fn test_line_width_clamped() {
        let mut v = new_blueprint_view();
        bpv_set_line_width(&mut v, 10.0);
        assert!((v.line_width - 4.0).abs() < 1e-6 /* line_width clamped to 4.0 */);
    }

    #[test]
    fn test_show_dimensions_toggle() {
        let mut v = new_blueprint_view();
        bpv_set_show_dimensions(&mut v, false);
        assert!(!v.show_dimensions /* show_dimensions must be disabled */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_blueprint_view();
        bpv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_scheme() {
        let v = new_blueprint_view();
        let j = bpv_to_json(&v);
        assert!(j.contains("\"scheme\"") /* JSON must have scheme */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_blueprint_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_grid_spacing() {
        let v = new_blueprint_view();
        assert!((v.grid_spacing - 32.0).abs() < 1e-6 /* default grid_spacing must be 32.0 */);
    }

    #[test]
    fn test_show_dimensions_default() {
        let v = new_blueprint_view();
        assert!(v.show_dimensions /* show_dimensions must default to true */);
    }
}
