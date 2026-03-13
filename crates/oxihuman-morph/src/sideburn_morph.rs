// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Sideburn shape and length morph control.

/// Sideburn morph configuration.
#[derive(Debug, Clone)]
pub struct SideburnMorph {
    pub length: f32,
    pub width: f32,
    pub taper: f32,
    pub density: f32,
}

impl SideburnMorph {
    pub fn new() -> Self {
        Self {
            length: 0.5,
            width: 0.4,
            taper: 0.5,
            density: 0.5,
        }
    }
}

impl Default for SideburnMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new sideburn morph.
pub fn new_sideburn_morph() -> SideburnMorph {
    SideburnMorph::new()
}

/// Set vertical length of sideburns in normalized range.
pub fn sideburn_set_length(morph: &mut SideburnMorph, length: f32) {
    morph.length = length.clamp(0.0, 1.0);
}

/// Set horizontal width of sideburns.
pub fn sideburn_set_width(morph: &mut SideburnMorph, width: f32) {
    morph.width = width.clamp(0.0, 1.0);
}

/// Set taper factor (0 = straight, 1 = fully tapered).
pub fn sideburn_set_taper(morph: &mut SideburnMorph, taper: f32) {
    morph.taper = taper.clamp(0.0, 1.0);
}

/// Estimate visible area as length * average_width.
pub fn sideburn_area_estimate(morph: &SideburnMorph) -> f32 {
    let avg_width = morph.width * (1.0 - morph.taper * 0.5);
    morph.length * avg_width
}

/// Serialize to JSON-like string.
pub fn sideburn_morph_to_json(morph: &SideburnMorph) -> String {
    format!(
        r#"{{"length":{:.4},"width":{:.4},"taper":{:.4},"density":{:.4}}}"#,
        morph.length, morph.width, morph.taper, morph.density
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_sideburn_morph();
        assert!((m.width - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_length_clamp() {
        let mut m = new_sideburn_morph();
        sideburn_set_length(&mut m, 2.0);
        assert_eq!(m.length, 1.0);
    }

    #[test]
    fn test_width_set() {
        let mut m = new_sideburn_morph();
        sideburn_set_width(&mut m, 0.6);
        assert!((m.width - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_taper_set() {
        let mut m = new_sideburn_morph();
        sideburn_set_taper(&mut m, 0.8);
        assert!((m.taper - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_area_estimate_positive() {
        let m = new_sideburn_morph();
        assert!(sideburn_area_estimate(&m) >= 0.0);
    }

    #[test]
    fn test_area_zero_length() {
        let mut m = new_sideburn_morph();
        sideburn_set_length(&mut m, 0.0);
        assert_eq!(sideburn_area_estimate(&m), 0.0);
    }

    #[test]
    fn test_json() {
        let m = new_sideburn_morph();
        let s = sideburn_morph_to_json(&m);
        assert!(s.contains("taper"));
    }

    #[test]
    fn test_clone() {
        let m = new_sideburn_morph();
        let m2 = m.clone();
        assert!((m2.taper - m.taper).abs() < 1e-6);
    }

    #[test]
    fn test_default_trait() {
        let m: SideburnMorph = Default::default();
        assert!((m.density - 0.5).abs() < 1e-6);
    }
}
