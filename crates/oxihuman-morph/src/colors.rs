// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Color theme system for skin tone, hair color, and eye color.
//!
//! Provides [`Color`] (RGBA), named [`ColorTheme`] presets, and a [`ThemePalette`]
//! for managing multiple themes. Colors feed into the PBR material system and
//! vertex color export.

/// An RGBA color with components in [0..1].
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a color from RGBA components.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create an opaque color from RGB components.
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Parse from hex string: `"#RRGGBB"`, `"#RRGGBBAA"`, or `"RRGGBB"`.
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
                Ok(Color::rgb(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                ))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|e| e.to_string())?;
                Ok(Color::rgba(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                ))
            }
            _ => Err(format!("invalid hex length: {}", hex.len())),
        }
    }

    /// Convert to hex string `"#RRGGBB"`.
    pub fn to_hex(&self) -> String {
        let r = (self.r.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (self.g.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (self.b.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }

    /// Convert to `[f32; 4]` array.
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Linearly interpolate between two colors.
    pub fn lerp(&self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    /// Convert to sRGB u8 values [0..255].
    pub fn to_srgb_u8(&self) -> [u8; 4] {
        [
            (self.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.b.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.a.clamp(0.0, 1.0) * 255.0).round() as u8,
        ]
    }

    /// Opaque white.
    pub const WHITE: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);
    /// Opaque black.
    pub const BLACK: Color = Color::rgba(0.0, 0.0, 0.0, 1.0);
    /// Fully transparent black.
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
}

/// Body color theme: skin tone, hair, and eye colors.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ColorTheme {
    pub name: String,
    pub skin: Color,
    pub hair: Color,
    pub eye: Color,
    pub lip: Color,
}

impl ColorTheme {
    /// Create a new theme with the given name and colors.
    pub fn new(name: impl Into<String>, skin: Color, hair: Color, eye: Color, lip: Color) -> Self {
        Self {
            name: name.into(),
            skin,
            hair,
            eye,
            lip,
        }
    }

    /// Caucasian theme: pale skin, brown hair, blue eyes.
    pub fn caucasian() -> Self {
        Self::new(
            "caucasian",
            Color::from_hex("#F5D5C0").unwrap_or(Color::WHITE),
            Color::from_hex("#6B3A2A").unwrap_or(Color::WHITE),
            Color::from_hex("#5B8DB8").unwrap_or(Color::WHITE),
            Color::from_hex("#C7736A").unwrap_or(Color::WHITE),
        )
    }

    /// African theme: dark skin, black hair, brown eyes.
    pub fn african() -> Self {
        Self::new(
            "african",
            Color::from_hex("#6B3A2A").unwrap_or(Color::WHITE),
            Color::from_hex("#1A0F0A").unwrap_or(Color::WHITE),
            Color::from_hex("#5C3317").unwrap_or(Color::WHITE),
            Color::from_hex("#8B4A3A").unwrap_or(Color::WHITE),
        )
    }

    /// Asian theme: medium skin, black hair, brown eyes.
    pub fn asian() -> Self {
        Self::new(
            "asian",
            Color::from_hex("#E8C9A0").unwrap_or(Color::WHITE),
            Color::from_hex("#1A0F0A").unwrap_or(Color::WHITE),
            Color::from_hex("#5C3317").unwrap_or(Color::WHITE),
            Color::from_hex("#C06050").unwrap_or(Color::WHITE),
        )
    }

    /// Albino theme: very pale skin, white/pale hair, pink/pale eyes.
    pub fn albino() -> Self {
        Self::new(
            "albino",
            Color::from_hex("#FFF5F0").unwrap_or(Color::WHITE),
            Color::from_hex("#FAFAFA").unwrap_or(Color::WHITE),
            Color::from_hex("#F0C0C0").unwrap_or(Color::WHITE),
            Color::from_hex("#FFCCCC").unwrap_or(Color::WHITE),
        )
    }

    /// Create a custom theme from hex strings for skin, hair, and eye colors.
    pub fn custom(skin_hex: &str, hair_hex: &str, eye_hex: &str) -> Result<Self, String> {
        let skin = Color::from_hex(skin_hex)?;
        let hair = Color::from_hex(hair_hex)?;
        let eye = Color::from_hex(eye_hex)?;
        // Default lip color derived from skin with slight reddish tint
        let lip = Color::rgb(
            (skin.r * 0.9 + 0.1).clamp(0.0, 1.0),
            skin.g * 0.7,
            skin.b * 0.7,
        );
        Ok(Self::new("custom", skin, hair, eye, lip))
    }

    /// Apply this theme's skin color to a Vec of vertex colors (one per vertex).
    /// Fills with `skin` color for all vertices.
    pub fn apply_skin_to_vertices(&self, n_verts: usize) -> Vec<[f32; 4]> {
        vec![self.skin.to_array(); n_verts]
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

/// A palette of named themes.
pub struct ThemePalette {
    themes: Vec<ColorTheme>,
}

impl ThemePalette {
    /// Create an empty palette.
    pub fn new() -> Self {
        Self { themes: Vec::new() }
    }

    /// Add a theme to the palette.
    pub fn add(&mut self, theme: ColorTheme) {
        self.themes.push(theme);
    }

    /// Look up a theme by name (case-insensitive).
    pub fn get(&self, name: &str) -> Option<&ColorTheme> {
        let lower = name.to_lowercase();
        self.themes.iter().find(|t| t.name.to_lowercase() == lower)
    }

    /// Return the names of all themes in the palette.
    pub fn names(&self) -> Vec<&str> {
        self.themes.iter().map(|t| t.name.as_str()).collect()
    }

    /// Standard palette with the 4 built-in themes.
    pub fn standard() -> Self {
        let mut palette = Self::new();
        palette.add(ColorTheme::caucasian());
        palette.add(ColorTheme::african());
        palette.add(ColorTheme::asian());
        palette.add(ColorTheme::albino());
        palette
    }
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_hex_valid() {
        assert!(Color::from_hex("#FF8040").is_ok());
    }

    #[test]
    fn from_hex_without_hash() {
        assert!(Color::from_hex("FF8040").is_ok());
    }

    #[test]
    fn from_hex_invalid() {
        assert!(Color::from_hex("ZZZZZZ").is_err());
    }

    #[test]
    fn from_hex_with_alpha() {
        let c = Color::from_hex("#FF804080").unwrap();
        assert!((c.a - 0.502).abs() < 0.005, "alpha was {}", c.a);
    }

    #[test]
    fn to_hex_roundtrip() {
        let hex = "#FF8040";
        let c = Color::from_hex(hex).unwrap();
        assert_eq!(c.to_hex(), hex);
    }

    #[test]
    fn lerp_midpoint() {
        let mid = Color::WHITE.lerp(Color::BLACK, 0.5);
        assert!((mid.r - 0.5).abs() < 1e-6, "r was {}", mid.r);
    }

    #[test]
    fn to_srgb_u8_white() {
        assert_eq!(Color::WHITE.to_srgb_u8(), [255, 255, 255, 255]);
    }

    #[test]
    fn standard_palette_has_four() {
        assert_eq!(ThemePalette::standard().names().len(), 4);
    }

    #[test]
    fn get_theme_case_insensitive() {
        let palette = ThemePalette::standard();
        assert!(palette.get("CAUCASIAN").is_some());
    }

    #[test]
    fn apply_skin_fills_verts() {
        let theme = ColorTheme::caucasian();
        assert_eq!(theme.apply_skin_to_vertices(10).len(), 10);
    }
}
