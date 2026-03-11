// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Diffuse color export for material definitions.

/// Diffuse color entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffuseColorExport {
    pub name: String,
    pub color: [f32; 4],
}

/// Collection of diffuse colors.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffuseColorBundle {
    pub entries: Vec<DiffuseColorExport>,
}

/// Create new bundle.
#[allow(dead_code)]
pub fn new_diffuse_bundle() -> DiffuseColorBundle {
    DiffuseColorBundle { entries: vec![] }
}

/// Add entry.
#[allow(dead_code)]
pub fn add_diffuse_color(b: &mut DiffuseColorBundle, name: &str, rgba: [f32; 4]) {
    b.entries.push(DiffuseColorExport {
        name: name.to_string(),
        color: rgba,
    });
}

/// Entry count.
#[allow(dead_code)]
pub fn dc_count(b: &DiffuseColorBundle) -> usize {
    b.entries.len()
}

/// Get color by name.
#[allow(dead_code)]
pub fn get_diffuse_color(b: &DiffuseColorBundle, name: &str) -> Option<[f32; 4]> {
    b.entries.iter().find(|e| e.name == name).map(|e| e.color)
}

/// Convert sRGB to linear.
#[allow(dead_code)]
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear to sRGB.
#[allow(dead_code)]
pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Validate (all channels in `[0,1]`).
#[allow(dead_code)]
pub fn dc_validate(b: &DiffuseColorBundle) -> bool {
    b.entries
        .iter()
        .all(|e| e.color.iter().all(|c| (0.0..=1.0).contains(c)))
}

/// Export to JSON.
#[allow(dead_code)]
pub fn diffuse_bundle_to_json(b: &DiffuseColorBundle) -> String {
    format!("{{\"count\":{}}}", dc_count(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let b = new_diffuse_bundle();
        assert_eq!(dc_count(&b), 0);
    }
    #[test]
    fn test_add() {
        let mut b = new_diffuse_bundle();
        add_diffuse_color(&mut b, "skin", [0.8, 0.6, 0.5, 1.0]);
        assert_eq!(dc_count(&b), 1);
    }
    #[test]
    fn test_get() {
        let mut b = new_diffuse_bundle();
        add_diffuse_color(&mut b, "hair", [0.1, 0.1, 0.1, 1.0]);
        assert!(get_diffuse_color(&b, "hair").is_some());
    }
    #[test]
    fn test_get_missing() {
        let b = new_diffuse_bundle();
        assert!(get_diffuse_color(&b, "x").is_none());
    }
    #[test]
    fn test_srgb_linear_roundtrip() {
        let v = 0.5f32;
        let l = srgb_to_linear(v);
        let s = linear_to_srgb(l);
        assert!((s - v).abs() < 1e-4);
    }
    #[test]
    fn test_srgb_zero() {
        assert!((srgb_to_linear(0.0)).abs() < 1e-9);
    }
    #[test]
    fn test_linear_zero() {
        assert!((linear_to_srgb(0.0)).abs() < 1e-9);
    }
    #[test]
    fn test_validate() {
        let mut b = new_diffuse_bundle();
        add_diffuse_color(&mut b, "a", [0.5; 4]);
        assert!(dc_validate(&b));
    }
    #[test]
    fn test_validate_bad() {
        let mut b = new_diffuse_bundle();
        add_diffuse_color(&mut b, "a", [1.5, 0.0, 0.0, 1.0]);
        assert!(!dc_validate(&b));
    }
    #[test]
    fn test_to_json() {
        let b = new_diffuse_bundle();
        assert!(diffuse_bundle_to_json(&b).contains("\"count\":0"));
    }
}
