// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Abdominal expansion morph — controls belly expansion from diaphragmatic breathing.

/// Abdominal expansion morph configuration.
#[derive(Debug, Clone)]
pub struct AbdomenExpandMorph {
    pub expansion: f32,
    pub upper_abdomen: f32,
    pub lower_abdomen: f32,
    pub lateral_bulge: f32,
}

impl AbdomenExpandMorph {
    pub fn new() -> Self {
        Self {
            expansion: 0.0,
            upper_abdomen: 0.0,
            lower_abdomen: 0.0,
            lateral_bulge: 0.0,
        }
    }
}

impl Default for AbdomenExpandMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new abdominal expansion morph.
pub fn new_abdomen_expand_morph() -> AbdomenExpandMorph {
    AbdomenExpandMorph::new()
}

/// Set overall abdominal expansion.
pub fn abdomen_expand_set_expansion(m: &mut AbdomenExpandMorph, v: f32) {
    m.expansion = v.clamp(0.0, 1.0);
}

/// Set upper abdominal contribution.
pub fn abdomen_expand_set_upper(m: &mut AbdomenExpandMorph, v: f32) {
    m.upper_abdomen = v.clamp(0.0, 1.0);
}

/// Set lower abdominal contribution.
pub fn abdomen_expand_set_lower(m: &mut AbdomenExpandMorph, v: f32) {
    m.lower_abdomen = v.clamp(0.0, 1.0);
}

/// Set lateral bulge.
pub fn abdomen_expand_set_lateral_bulge(m: &mut AbdomenExpandMorph, v: f32) {
    m.lateral_bulge = v.clamp(0.0, 1.0);
}

/// Mean expansion across all regions.
pub fn abdomen_expand_mean(m: &AbdomenExpandMorph) -> f32 {
    (m.expansion + m.upper_abdomen + m.lower_abdomen) / 3.0
}

/// Serialize to JSON-like string.
pub fn abdomen_expand_morph_to_json(m: &AbdomenExpandMorph) -> String {
    format!(
        r#"{{"expansion":{:.4},"upper_abdomen":{:.4},"lower_abdomen":{:.4},"lateral_bulge":{:.4}}}"#,
        m.expansion, m.upper_abdomen, m.lower_abdomen, m.lateral_bulge
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_abdomen_expand_morph();
        assert_eq!(m.expansion, 0.0);
    }

    #[test]
    fn test_expansion() {
        let mut m = new_abdomen_expand_morph();
        abdomen_expand_set_expansion(&mut m, 0.7);
        assert!((m.expansion - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_upper_clamp() {
        let mut m = new_abdomen_expand_morph();
        abdomen_expand_set_upper(&mut m, 3.0);
        assert_eq!(m.upper_abdomen, 1.0);
    }

    #[test]
    fn test_lower() {
        let mut m = new_abdomen_expand_morph();
        abdomen_expand_set_lower(&mut m, 0.5);
        assert!((m.lower_abdomen - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_lateral_bulge_clamp() {
        let mut m = new_abdomen_expand_morph();
        abdomen_expand_set_lateral_bulge(&mut m, -0.5);
        assert_eq!(m.lateral_bulge, 0.0);
    }

    #[test]
    fn test_mean_expansion_zero() {
        let m = new_abdomen_expand_morph();
        assert_eq!(abdomen_expand_mean(&m), 0.0);
    }

    #[test]
    fn test_mean_expansion_value() {
        let mut m = new_abdomen_expand_morph();
        abdomen_expand_set_expansion(&mut m, 0.3);
        abdomen_expand_set_upper(&mut m, 0.6);
        abdomen_expand_set_lower(&mut m, 0.9);
        let mean = abdomen_expand_mean(&m);
        assert!((mean - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_json_keys() {
        let m = new_abdomen_expand_morph();
        let s = abdomen_expand_morph_to_json(&m);
        assert!(s.contains("lateral_bulge"));
    }

    #[test]
    fn test_clone() {
        let m = new_abdomen_expand_morph();
        let m2 = m.clone();
        assert!((m2.expansion - m.expansion).abs() < 1e-6);
    }
}
