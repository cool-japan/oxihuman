// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! False color exposure view for cinematography exposure monitoring.

/// False color palette preset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FalseColorPalette {
    Cinema,
    Broadcast,
    Scientific,
    Rainbow,
}

/// False color view configuration.
#[derive(Debug, Clone)]
pub struct FalseColorView {
    pub palette: FalseColorPalette,
    pub min_luminance: f32,
    pub max_luminance: f32,
    pub enabled: bool,
    pub legend_visible: bool,
}

impl FalseColorView {
    pub fn new() -> Self {
        Self {
            palette: FalseColorPalette::Cinema,
            min_luminance: 0.0,
            max_luminance: 1.0,
            enabled: false,
            legend_visible: true,
        }
    }
}

impl Default for FalseColorView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new false color view.
pub fn new_false_color_view() -> FalseColorView {
    FalseColorView::new()
}

/// Set the false color palette.
pub fn fcv_set_palette(view: &mut FalseColorView, palette: FalseColorPalette) {
    view.palette = palette;
}

/// Set luminance range for false color mapping.
pub fn fcv_set_luminance_range(view: &mut FalseColorView, min: f32, max: f32) {
    view.min_luminance = min.clamp(0.0, 1.0);
    view.max_luminance = max.clamp(0.0, 1.0);
}

/// Toggle false color overlay.
pub fn fcv_set_enabled(view: &mut FalseColorView, enabled: bool) {
    view.enabled = enabled;
}

/// Toggle legend display.
pub fn fcv_set_legend_visible(view: &mut FalseColorView, visible: bool) {
    view.legend_visible = visible;
}

/// Map a luminance value `[0,1]` to a normalized palette position.
pub fn fcv_map_luminance(view: &FalseColorView, luminance: f32) -> f32 {
    let range = view.max_luminance - view.min_luminance;
    if range.abs() < 1e-6 {
        return 0.0;
    }
    ((luminance - view.min_luminance) / range).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn false_color_view_to_json(view: &FalseColorView) -> String {
    let palette_str = match view.palette {
        FalseColorPalette::Cinema => "cinema",
        FalseColorPalette::Broadcast => "broadcast",
        FalseColorPalette::Scientific => "scientific",
        FalseColorPalette::Rainbow => "rainbow",
    };
    format!(
        r#"{{"palette":"{palette_str}","min":{:.4},"max":{:.4},"enabled":{}}}"#,
        view.min_luminance, view.max_luminance, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_false_color_view();
        assert_eq!(v.palette, FalseColorPalette::Cinema);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_palette() {
        let mut v = new_false_color_view();
        fcv_set_palette(&mut v, FalseColorPalette::Rainbow);
        assert_eq!(v.palette, FalseColorPalette::Rainbow);
    }

    #[test]
    fn test_luminance_range() {
        let mut v = new_false_color_view();
        fcv_set_luminance_range(&mut v, 0.1, 0.9);
        assert!((v.min_luminance - 0.1).abs() < 1e-6);
        assert!((v.max_luminance - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_enabled_toggle() {
        let mut v = new_false_color_view();
        fcv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_map_luminance_mid() {
        let v = new_false_color_view();
        let pos = fcv_map_luminance(&v, 0.5);
        assert!((pos - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_map_luminance_clamped() {
        let v = new_false_color_view();
        let pos = fcv_map_luminance(&v, 2.0);
        assert_eq!(pos, 1.0);
    }

    #[test]
    fn test_legend_toggle() {
        let mut v = new_false_color_view();
        fcv_set_legend_visible(&mut v, false);
        assert!(!v.legend_visible);
    }

    #[test]
    fn test_json() {
        let v = new_false_color_view();
        let s = false_color_view_to_json(&v);
        assert!(s.contains("cinema"));
    }

    #[test]
    fn test_clone() {
        let v = new_false_color_view();
        let v2 = v.clone();
        assert_eq!(v2.palette, v.palette);
    }
}
