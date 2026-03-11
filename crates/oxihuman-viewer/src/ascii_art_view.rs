// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ASCII art render mode stub.

/// ASCII character palette.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AsciiPalette {
    Standard,
    Extended,
    BlockElements,
    Minimal,
}

/// ASCII art view configuration.
#[derive(Debug, Clone)]
pub struct AsciiArtView {
    pub palette: AsciiPalette,
    pub cell_width: u32,
    pub cell_height: u32,
    pub colored: bool,
    pub invert: bool,
    pub enabled: bool,
}

impl AsciiArtView {
    pub fn new() -> Self {
        AsciiArtView {
            palette: AsciiPalette::Standard,
            cell_width: 8,
            cell_height: 16,
            colored: false,
            invert: false,
            enabled: true,
        }
    }
}

impl Default for AsciiArtView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new ASCII art view.
pub fn new_ascii_art_view() -> AsciiArtView {
    AsciiArtView::new()
}

/// Set character palette.
pub fn aav_set_palette(view: &mut AsciiArtView, palette: AsciiPalette) {
    view.palette = palette;
}

/// Set cell dimensions.
pub fn aav_set_cell_size(view: &mut AsciiArtView, width: u32, height: u32) {
    view.cell_width = width.clamp(4, 32);
    view.cell_height = height.clamp(4, 32);
}

/// Toggle color output.
pub fn aav_set_colored(view: &mut AsciiArtView, colored: bool) {
    view.colored = colored;
}

/// Toggle intensity inversion.
pub fn aav_set_invert(view: &mut AsciiArtView, invert: bool) {
    view.invert = invert;
}

/// Enable or disable.
pub fn aav_set_enabled(view: &mut AsciiArtView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn aav_to_json(view: &AsciiArtView) -> String {
    let palette = match view.palette {
        AsciiPalette::Standard => "standard",
        AsciiPalette::Extended => "extended",
        AsciiPalette::BlockElements => "block_elements",
        AsciiPalette::Minimal => "minimal",
    };
    format!(
        r#"{{"palette":"{}","cell_width":{},"cell_height":{},"colored":{},"invert":{},"enabled":{}}}"#,
        palette, view.cell_width, view.cell_height, view.colored, view.invert, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_palette() {
        let v = new_ascii_art_view();
        assert_eq!(
            v.palette,
            AsciiPalette::Standard /* default palette must be Standard */
        );
    }

    #[test]
    fn test_set_palette() {
        let mut v = new_ascii_art_view();
        aav_set_palette(&mut v, AsciiPalette::BlockElements);
        assert_eq!(
            v.palette,
            AsciiPalette::BlockElements /* palette must be set */
        );
    }

    #[test]
    fn test_cell_size_clamped() {
        let mut v = new_ascii_art_view();
        aav_set_cell_size(&mut v, 0, 0);
        assert_eq!(v.cell_width, 4 /* cell_width clamped to 4 */);
        assert_eq!(v.cell_height, 4 /* cell_height clamped to 4 */);
    }

    #[test]
    fn test_cell_size_max_clamped() {
        let mut v = new_ascii_art_view();
        aav_set_cell_size(&mut v, 100, 100);
        assert_eq!(v.cell_width, 32 /* cell_width clamped to 32 */);
    }

    #[test]
    fn test_set_colored() {
        let mut v = new_ascii_art_view();
        aav_set_colored(&mut v, true);
        assert!(v.colored /* colored must be enabled */);
    }

    #[test]
    fn test_set_invert() {
        let mut v = new_ascii_art_view();
        aav_set_invert(&mut v, true);
        assert!(v.invert /* invert must be enabled */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_ascii_art_view();
        aav_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_palette() {
        let v = new_ascii_art_view();
        let j = aav_to_json(&v);
        assert!(j.contains("\"palette\"") /* JSON must have palette */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_ascii_art_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_cell_size() {
        let v = new_ascii_art_view();
        assert_eq!(v.cell_width, 8 /* default cell_width must be 8 */);
        assert_eq!(v.cell_height, 16 /* default cell_height must be 16 */);
    }
}
