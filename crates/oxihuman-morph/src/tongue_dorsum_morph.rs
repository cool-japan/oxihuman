// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Tongue dorsum/back shape morph — controls body height, groove, and arch.

/// Tongue dorsum morph configuration.
#[derive(Debug, Clone)]
pub struct TongueDorsumMorph {
    pub arch_height: f32,
    pub groove_depth: f32,
    pub posterior_raise: f32,
    pub width: f32,
}

impl TongueDorsumMorph {
    pub fn new() -> Self {
        Self {
            arch_height: 0.0,
            groove_depth: 0.0,
            posterior_raise: 0.0,
            width: 0.5,
        }
    }
}

impl Default for TongueDorsumMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new tongue dorsum morph.
pub fn new_tongue_dorsum_morph() -> TongueDorsumMorph {
    TongueDorsumMorph::new()
}

/// Set dorsal arch height.
pub fn tongue_dorsum_set_arch(m: &mut TongueDorsumMorph, v: f32) {
    m.arch_height = v.clamp(0.0, 1.0);
}

/// Set median groove depth.
pub fn tongue_dorsum_set_groove(m: &mut TongueDorsumMorph, v: f32) {
    m.groove_depth = v.clamp(0.0, 1.0);
}

/// Set posterior raise (velum contact).
pub fn tongue_dorsum_set_posterior_raise(m: &mut TongueDorsumMorph, v: f32) {
    m.posterior_raise = v.clamp(0.0, 1.0);
}

/// Set dorsal width.
pub fn tongue_dorsum_set_width(m: &mut TongueDorsumMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

/// Serialize to JSON-like string.
pub fn tongue_dorsum_morph_to_json(m: &TongueDorsumMorph) -> String {
    format!(
        r#"{{"arch_height":{:.4},"groove_depth":{:.4},"posterior_raise":{:.4},"width":{:.4}}}"#,
        m.arch_height, m.groove_depth, m.posterior_raise, m.width
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_tongue_dorsum_morph();
        assert_eq!(m.arch_height, 0.0);
        assert!((m.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_arch() {
        let mut m = new_tongue_dorsum_morph();
        tongue_dorsum_set_arch(&mut m, 0.6);
        assert!((m.arch_height - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_arch_clamp() {
        let mut m = new_tongue_dorsum_morph();
        tongue_dorsum_set_arch(&mut m, 2.0);
        assert_eq!(m.arch_height, 1.0);
    }

    #[test]
    fn test_set_groove() {
        let mut m = new_tongue_dorsum_morph();
        tongue_dorsum_set_groove(&mut m, 0.4);
        assert!((m.groove_depth - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_posterior_raise() {
        let mut m = new_tongue_dorsum_morph();
        tongue_dorsum_set_posterior_raise(&mut m, 0.8);
        assert!((m.posterior_raise - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_width_clamp() {
        let mut m = new_tongue_dorsum_morph();
        tongue_dorsum_set_width(&mut m, -1.0);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_tongue_dorsum_morph();
        let s = tongue_dorsum_morph_to_json(&m);
        assert!(s.contains("groove_depth"));
    }

    #[test]
    fn test_clone() {
        let m = new_tongue_dorsum_morph();
        let m2 = m.clone();
        assert!((m2.width - m.width).abs() < 1e-6);
    }

    #[test]
    fn test_width_set() {
        let mut m = new_tongue_dorsum_morph();
        tongue_dorsum_set_width(&mut m, 0.75);
        assert!((m.width - 0.75).abs() < 1e-6);
    }
}
