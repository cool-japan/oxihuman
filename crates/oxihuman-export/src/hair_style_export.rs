// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export hair style parameters (length, curl, clump, noise settings).

/// Hair style parameter set.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairStyle {
    pub name: String,
    pub length: f32,
    pub curl_amount: f32,
    pub curl_frequency: f32,
    pub clump_factor: f32,
    pub noise_scale: f32,
    pub noise_strength: f32,
    pub root_width: f32,
    pub tip_width: f32,
}

impl Default for HairStyle {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            length: 0.2,
            curl_amount: 0.0,
            curl_frequency: 1.0,
            clump_factor: 0.0,
            noise_scale: 1.0,
            noise_strength: 0.0,
            root_width: 0.002,
            tip_width: 0.0005,
        }
    }
}

/// Collection of hair styles.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct HairStyleLibrary {
    pub styles: Vec<HairStyle>,
}

/// Create a new library.
#[allow(dead_code)]
pub fn new_hair_style_library() -> HairStyleLibrary {
    HairStyleLibrary::default()
}

/// Add a style.
#[allow(dead_code)]
pub fn add_style(lib: &mut HairStyleLibrary, style: HairStyle) {
    lib.styles.push(style);
}

/// Find a style by name.
#[allow(dead_code)]
pub fn find_style<'a>(lib: &'a HairStyleLibrary, name: &str) -> Option<&'a HairStyle> {
    lib.styles.iter().find(|s| s.name == name)
}

/// Compute the average strand length across styles.
#[allow(dead_code)]
pub fn average_length(lib: &HairStyleLibrary) -> f32 {
    if lib.styles.is_empty() {
        return 0.0;
    }
    lib.styles.iter().map(|s| s.length).sum::<f32>() / lib.styles.len() as f32
}

/// Validate that all lengths are positive.
#[allow(dead_code)]
pub fn all_lengths_positive(lib: &HairStyleLibrary) -> bool {
    lib.styles.iter().all(|s| s.length > 0.0)
}

/// Serialise a style to a flat f32 buffer.
#[allow(dead_code)]
pub fn serialise_style(style: &HairStyle) -> Vec<f32> {
    vec![
        style.length,
        style.curl_amount,
        style.curl_frequency,
        style.clump_factor,
        style.noise_scale,
        style.noise_strength,
        style.root_width,
        style.tip_width,
    ]
}

/// Scale all lengths by a factor.
#[allow(dead_code)]
pub fn scale_lengths(lib: &mut HairStyleLibrary, factor: f32) {
    for s in &mut lib.styles {
        s.length *= factor;
    }
}

/// Check if a style is "straight" (no curl, no noise).
#[allow(dead_code)]
pub fn is_straight(style: &HairStyle) -> bool {
    style.curl_amount.abs() < 1e-4 && style.noise_strength.abs() < 1e-4
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_lib() -> HairStyleLibrary {
        let mut lib = new_hair_style_library();
        add_style(&mut lib, HairStyle::default());
        lib
    }

    #[test]
    fn test_new_library_empty() {
        let lib = new_hair_style_library();
        assert!(lib.styles.is_empty());
    }

    #[test]
    fn test_add_style() {
        let lib = default_lib();
        assert_eq!(lib.styles.len(), 1);
    }

    #[test]
    fn test_find_style_found() {
        let lib = default_lib();
        assert!(find_style(&lib, "default").is_some());
    }

    #[test]
    fn test_find_style_not_found() {
        let lib = default_lib();
        assert!(find_style(&lib, "wavy").is_none());
    }

    #[test]
    fn test_average_length() {
        let lib = default_lib();
        assert!((average_length(&lib) - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_all_lengths_positive_true() {
        let lib = default_lib();
        assert!(all_lengths_positive(&lib));
    }

    #[test]
    fn test_all_lengths_positive_false() {
        let mut lib = default_lib();
        lib.styles[0].length = -0.1;
        assert!(!all_lengths_positive(&lib));
    }

    #[test]
    fn test_serialise_style_length() {
        let s = HairStyle::default();
        assert_eq!(serialise_style(&s).len(), 8);
    }

    #[test]
    fn test_scale_lengths() {
        let mut lib = default_lib();
        scale_lengths(&mut lib, 2.0);
        assert!((lib.styles[0].length - 0.4).abs() < 1e-5);
    }

    #[test]
    fn test_is_straight_default() {
        let s = HairStyle::default();
        assert!(is_straight(&s));
    }
}
