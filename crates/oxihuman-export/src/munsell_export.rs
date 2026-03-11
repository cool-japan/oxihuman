// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Munsell color notation export — encodes and exports Munsell color notations.

/// A Munsell color specification (Hue Value/Chroma).
#[derive(Debug, Clone)]
pub struct MunsellColor {
    pub hue_prefix: f32,
    pub hue_letter: String,
    pub value: f32,
    pub chroma: f32,
}

impl MunsellColor {
    /// Creates a new Munsell color.
    pub fn new(hue_prefix: f32, hue_letter: impl Into<String>, value: f32, chroma: f32) -> Self {
        Self {
            hue_prefix,
            hue_letter: hue_letter.into(),
            value,
            chroma,
        }
    }

    /// Returns the standard Munsell notation string.
    pub fn notation(&self) -> String {
        format!(
            "{}{} {}/{}",
            self.hue_prefix, self.hue_letter, self.value, self.chroma
        )
    }
}

/// A Munsell color export record.
#[derive(Debug, Default, Clone)]
pub struct MunsellExport {
    pub entries: Vec<MunsellColor>,
}

impl MunsellExport {
    /// Creates a new export.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a Munsell color entry.
    pub fn add(&mut self, color: MunsellColor) {
        self.entries.push(color);
    }

    /// Returns the entry count.
    pub fn count(&self) -> usize {
        self.entries.len()
    }
}

/// Exports all Munsell notations to a newline-separated string.
pub fn export_munsell_list(export: &MunsellExport) -> String {
    export
        .entries
        .iter()
        .map(|e| e.notation())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Validates a Munsell color (value 0..=10, chroma >= 0).
pub fn validate_munsell(color: &MunsellColor) -> bool {
    (0.0..=10.0).contains(&color.value) && color.chroma >= 0.0
}

/// Converts Munsell value to approximate CIE Y (Judd formula stub).
pub fn munsell_value_to_y(v: f32) -> f32 {
    let v = v.clamp(0.0, 10.0);
    v * (1.1914 + v * (-0.22533 + v * (0.23352 + v * (-0.020484 + v * 0.00081939))))
}

/// Returns the achromatic (neutral) notation for a given value.
pub fn achromatic_notation(value: f32) -> String {
    format!("N {}", value.clamp(0.0, 10.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red() -> MunsellColor {
        MunsellColor::new(5.0, "R", 5.0, 12.0)
    }

    #[test]
    fn test_notation_format() {
        /* Notation should be 5R 5/12 */
        assert_eq!(red().notation(), "5R 5/12");
    }

    #[test]
    fn test_new_export_empty() {
        /* New export should have zero entries */
        assert_eq!(MunsellExport::new().count(), 0);
    }

    #[test]
    fn test_add_entry() {
        /* Adding an entry should increase count */
        let mut exp = MunsellExport::new();
        exp.add(red());
        assert_eq!(exp.count(), 1);
    }

    #[test]
    fn test_export_list_single() {
        /* Export list should contain the notation */
        let mut exp = MunsellExport::new();
        exp.add(red());
        assert!(export_munsell_list(&exp).contains("5R"));
    }

    #[test]
    fn test_validate_valid() {
        /* Red color with valid value and chroma should validate */
        assert!(validate_munsell(&red()));
    }

    #[test]
    fn test_validate_invalid_value() {
        /* Value > 10 should fail */
        let bad = MunsellColor::new(5.0, "R", 11.0, 4.0);
        assert!(!validate_munsell(&bad));
    }

    #[test]
    fn test_validate_negative_chroma() {
        /* Negative chroma should fail */
        let bad = MunsellColor::new(5.0, "R", 5.0, -1.0);
        assert!(!validate_munsell(&bad));
    }

    #[test]
    fn test_munsell_value_to_y_black() {
        /* Value 0 → Y ≈ 0 */
        assert!(munsell_value_to_y(0.0).abs() < 0.01);
    }

    #[test]
    fn test_achromatic_notation() {
        /* Achromatic at value 5 should be "N 5" */
        assert_eq!(achromatic_notation(5.0), "N 5");
    }

    #[test]
    fn test_export_list_empty() {
        /* Empty export should return empty string */
        assert_eq!(export_munsell_list(&MunsellExport::new()), "");
    }
}
