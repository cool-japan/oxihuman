// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Eyelash length, curl, and density morph controls.

/// Configuration for eyelash morph parameters.
#[derive(Debug, Clone)]
pub struct EyelashMorph {
    pub length: f32,
    pub curl: f32,
    pub density: f32,
    pub thickness: f32,
}

impl EyelashMorph {
    pub fn new() -> Self {
        Self {
            length: 0.5,
            curl: 0.5,
            density: 0.5,
            thickness: 0.5,
        }
    }
}

impl Default for EyelashMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new eyelash morph with default values.
pub fn new_eyelash_morph() -> EyelashMorph {
    EyelashMorph::new()
}

/// Set eyelash length in normalized range [0, 1].
pub fn eyelash_set_length(morph: &mut EyelashMorph, length: f32) {
    morph.length = length.clamp(0.0, 1.0);
}

/// Set eyelash curl amount in normalized range [0, 1].
pub fn eyelash_set_curl(morph: &mut EyelashMorph, curl: f32) {
    morph.curl = curl.clamp(0.0, 1.0);
}

/// Set eyelash strand density in normalized range [0, 1].
pub fn eyelash_set_density(morph: &mut EyelashMorph, density: f32) {
    morph.density = density.clamp(0.0, 1.0);
}

/// Serialize eyelash morph state to a JSON-like string.
pub fn eyelash_morph_to_json(morph: &EyelashMorph) -> String {
    format!(
        r#"{{"length":{:.4},"curl":{:.4},"density":{:.4},"thickness":{:.4}}}"#,
        morph.length, morph.curl, morph.density, morph.thickness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let m = new_eyelash_morph();
        assert!((0.0..=1.0).contains(&m.length));
        assert!((0.0..=1.0).contains(&m.curl));
    }

    #[test]
    fn test_set_length_clamp() {
        let mut m = new_eyelash_morph();
        eyelash_set_length(&mut m, 2.0);
        assert_eq!(m.length, 1.0);
        eyelash_set_length(&mut m, -1.0);
        assert_eq!(m.length, 0.0);
    }

    #[test]
    fn test_set_curl_clamp() {
        let mut m = new_eyelash_morph();
        eyelash_set_curl(&mut m, 0.7);
        assert!((m.curl - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_density() {
        let mut m = new_eyelash_morph();
        eyelash_set_density(&mut m, 0.3);
        assert!((m.density - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_json_output() {
        let m = new_eyelash_morph();
        let s = eyelash_morph_to_json(&m);
        assert!(s.contains("length"));
        assert!(s.contains("curl"));
    }

    #[test]
    fn test_default_trait() {
        let m: EyelashMorph = Default::default();
        assert!((m.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        let m = new_eyelash_morph();
        let m2 = m.clone();
        assert!((m2.curl - m.curl).abs() < 1e-6);
    }

    #[test]
    fn test_set_density_clamp() {
        let mut m = new_eyelash_morph();
        eyelash_set_density(&mut m, 1.5);
        assert_eq!(m.density, 1.0);
    }

    #[test]
    fn test_json_contains_density() {
        let mut m = new_eyelash_morph();
        eyelash_set_density(&mut m, 0.9);
        let s = eyelash_morph_to_json(&m);
        assert!(s.contains("density"));
    }
}
