// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! RAL Classic color export stub — maps RAL codes to sRGB approximations.

/// A RAL Classic color entry.
#[derive(Debug, Clone)]
pub struct RalColor {
    pub code: u32,
    pub name: String,
    pub srgb: [u8; 3],
}

impl RalColor {
    /// Creates a new RAL color.
    pub fn new(code: u32, name: impl Into<String>, srgb: [u8; 3]) -> Self {
        Self {
            code,
            name: name.into(),
            srgb,
        }
    }

    /// Returns the hex color string (#RRGGBB).
    pub fn hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}",
            self.srgb[0], self.srgb[1], self.srgb[2]
        )
    }
}

/// A RAL color palette.
#[derive(Debug, Default, Clone)]
pub struct RalPalette {
    pub colors: Vec<RalColor>,
}

impl RalPalette {
    /// Creates an empty palette.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a color.
    pub fn add(&mut self, color: RalColor) {
        self.colors.push(color);
    }

    /// Finds a color by RAL code.
    pub fn find_by_code(&self, code: u32) -> Option<&RalColor> {
        self.colors.iter().find(|c| c.code == code)
    }

    /// Returns the number of colors.
    pub fn count(&self) -> usize {
        self.colors.len()
    }
}

/// Exports the palette as a CSV string.
pub fn export_ral_csv(palette: &RalPalette) -> String {
    let mut out = String::from("ral_code,name,hex\n");
    for c in &palette.colors {
        out.push_str(&format!("RAL {},{},{}\n", c.code, c.name, c.hex()));
    }
    out
}

/// Returns the closest RAL color to a target sRGB value.
pub fn closest_ral(palette: &RalPalette, target: [u8; 3]) -> Option<&RalColor> {
    palette.colors.iter().min_by_key(|c| {
        let dr = (c.srgb[0] as i32 - target[0] as i32).pow(2);
        let dg = (c.srgb[1] as i32 - target[1] as i32).pow(2);
        let db = (c.srgb[2] as i32 - target[2] as i32).pow(2);
        dr + dg + db
    })
}

/// Validates RAL codes (must be in range 1000..=9999 for Classic).
pub fn validate_ral_palette(palette: &RalPalette) -> bool {
    palette
        .colors
        .iter()
        .all(|c| (1000..=9999).contains(&c.code))
}

/// Creates a stub palette with a few well-known RAL colors.
pub fn sample_ral_palette() -> RalPalette {
    let mut p = RalPalette::new();
    p.add(RalColor::new(1000, "Green beige", [205, 186, 136]));
    p.add(RalColor::new(3000, "Flame red", [171, 35, 40]));
    p.add(RalColor::new(9016, "Traffic white", [241, 236, 225]));
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_palette_empty() {
        /* New palette should have zero colors */
        assert_eq!(RalPalette::new().count(), 0);
    }

    #[test]
    fn test_add_color() {
        /* Adding increases count */
        let mut p = RalPalette::new();
        p.add(RalColor::new(3000, "Flame red", [171, 35, 40]));
        assert_eq!(p.count(), 1);
    }

    #[test]
    fn test_find_by_code_found() {
        /* Should find known color */
        let p = sample_ral_palette();
        assert!(p.find_by_code(3000).is_some());
    }

    #[test]
    fn test_find_by_code_not_found() {
        /* Should return None for unknown code */
        let p = RalPalette::new();
        assert!(p.find_by_code(1234).is_none());
    }

    #[test]
    fn test_hex_format() {
        /* Hex should be 7 chars starting with # */
        let c = RalColor::new(3000, "Flame red", [171, 35, 40]);
        let h = c.hex();
        assert!(h.starts_with('#') && h.len() == 7);
    }

    #[test]
    fn test_export_csv_header() {
        /* CSV should start with header */
        assert!(export_ral_csv(&RalPalette::new()).starts_with("ral_code"));
    }

    #[test]
    fn test_closest_ral_empty() {
        /* Empty palette returns None */
        assert!(closest_ral(&RalPalette::new(), [0, 0, 0]).is_none());
    }

    #[test]
    fn test_closest_ral_finds_result() {
        /* Should return a result from non-empty palette */
        let p = sample_ral_palette();
        assert!(closest_ral(&p, [200, 180, 130]).is_some());
    }

    #[test]
    fn test_validate_valid() {
        /* Sample palette should validate */
        assert!(validate_ral_palette(&sample_ral_palette()));
    }

    #[test]
    fn test_validate_invalid_code() {
        /* Code outside 1000..9999 should fail */
        let mut p = RalPalette::new();
        p.add(RalColor::new(99, "bad", [0, 0, 0]));
        assert!(!validate_ral_palette(&p));
    }
}
