// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pantone color reference export stub — maps Pantone codes to sRGB approximations.

/// A Pantone color entry.
#[derive(Debug, Clone)]
pub struct PantoneColor {
    pub code: String,
    pub name: String,
    pub srgb: [u8; 3],
}

impl PantoneColor {
    /// Creates a new Pantone color entry.
    pub fn new(code: impl Into<String>, name: impl Into<String>, srgb: [u8; 3]) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
            srgb,
        }
    }

    /// Returns the hex string representation (#RRGGBB).
    pub fn hex_string(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}",
            self.srgb[0], self.srgb[1], self.srgb[2]
        )
    }

    /// Returns the sRGB values as normalized floats `[0,1]`.
    pub fn srgb_f32(&self) -> [f32; 3] {
        [
            self.srgb[0] as f32 / 255.0,
            self.srgb[1] as f32 / 255.0,
            self.srgb[2] as f32 / 255.0,
        ]
    }
}

/// A Pantone palette export.
#[derive(Debug, Default, Clone)]
pub struct PantonePalette {
    pub colors: Vec<PantoneColor>,
}

impl PantonePalette {
    /// Creates an empty palette.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a color.
    pub fn add(&mut self, color: PantoneColor) {
        self.colors.push(color);
    }

    /// Finds a color by code.
    pub fn find_by_code(&self, code: &str) -> Option<&PantoneColor> {
        self.colors.iter().find(|c| c.code == code)
    }

    /// Returns the number of colors.
    pub fn count(&self) -> usize {
        self.colors.len()
    }
}

/// Exports the palette as a CSV string.
pub fn export_pantone_csv(palette: &PantonePalette) -> String {
    let mut out = String::from("code,name,hex\n");
    for c in &palette.colors {
        out.push_str(&format!("{},{},{}\n", c.code, c.name, c.hex_string()));
    }
    out
}

/// Returns the closest Pantone color to a target sRGB value.
pub fn closest_pantone(palette: &PantonePalette, target: [u8; 3]) -> Option<&PantoneColor> {
    palette.colors.iter().min_by_key(|c| {
        let dr = (c.srgb[0] as i32 - target[0] as i32).pow(2);
        let dg = (c.srgb[1] as i32 - target[1] as i32).pow(2);
        let db = (c.srgb[2] as i32 - target[2] as i32).pow(2);
        dr + dg + db
    })
}

/// Validates that all sRGB components are valid bytes.
pub fn validate_pantone_palette(palette: &PantonePalette) -> bool {
    palette
        .colors
        .iter()
        .all(|c| !c.code.is_empty() && !c.name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red_pantone() -> PantoneColor {
        PantoneColor::new("485 C", "Red 485", [218, 41, 28])
    }

    #[test]
    fn test_hex_string() {
        /* Red 485 should produce correct hex */
        assert_eq!(red_pantone().hex_string(), "#DA291C");
    }

    #[test]
    fn test_srgb_f32_range() {
        /* All f32 values should be in [0,1] */
        let vals = red_pantone().srgb_f32();
        assert!(vals.iter().all(|&v| (0.0..=1.0).contains(&v)));
    }

    #[test]
    fn test_add_color() {
        /* Adding a color should increase count */
        let mut palette = PantonePalette::new();
        palette.add(red_pantone());
        assert_eq!(palette.count(), 1);
    }

    #[test]
    fn test_find_by_code_found() {
        /* Should find color by exact code */
        let mut palette = PantonePalette::new();
        palette.add(red_pantone());
        assert!(palette.find_by_code("485 C").is_some());
    }

    #[test]
    fn test_find_by_code_not_found() {
        /* Should return None for missing code */
        let palette = PantonePalette::new();
        assert!(palette.find_by_code("999 C").is_none());
    }

    #[test]
    fn test_export_csv_header() {
        /* CSV should start with header */
        let palette = PantonePalette::new();
        assert!(export_pantone_csv(&palette).starts_with("code"));
    }

    #[test]
    fn test_closest_pantone_empty() {
        /* Empty palette should return None */
        let palette = PantonePalette::new();
        assert!(closest_pantone(&palette, [255, 0, 0]).is_none());
    }

    #[test]
    fn test_closest_pantone_single() {
        /* Single color should always be closest */
        let mut palette = PantonePalette::new();
        palette.add(red_pantone());
        assert!(closest_pantone(&palette, [0, 0, 0]).is_some());
    }

    #[test]
    fn test_validate_valid() {
        /* Non-empty code and name should validate */
        let mut palette = PantonePalette::new();
        palette.add(red_pantone());
        assert!(validate_pantone_palette(&palette));
    }

    #[test]
    fn test_validate_empty_code_fails() {
        /* Empty code should fail validation */
        let mut palette = PantonePalette::new();
        palette.add(PantoneColor::new("", "red", [255, 0, 0]));
        assert!(!validate_pantone_palette(&palette));
    }
}
