// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Rib cage expansion morph — controls thoracic expansion during breathing.

/// Rib cage morph configuration.
#[derive(Debug, Clone)]
pub struct RibCageMorph {
    pub expansion: f32,
    pub upper_chest: f32,
    pub lower_chest: f32,
    pub lateral_flare: f32,
}

impl RibCageMorph {
    pub fn new() -> Self {
        Self {
            expansion: 0.0,
            upper_chest: 0.0,
            lower_chest: 0.0,
            lateral_flare: 0.0,
        }
    }
}

impl Default for RibCageMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new rib cage morph.
pub fn new_rib_cage_morph() -> RibCageMorph {
    RibCageMorph::new()
}

/// Set overall thoracic expansion.
pub fn rib_cage_set_expansion(m: &mut RibCageMorph, v: f32) {
    m.expansion = v.clamp(0.0, 1.0);
}

/// Set upper chest contribution.
pub fn rib_cage_set_upper_chest(m: &mut RibCageMorph, v: f32) {
    m.upper_chest = v.clamp(0.0, 1.0);
}

/// Set lower chest contribution.
pub fn rib_cage_set_lower_chest(m: &mut RibCageMorph, v: f32) {
    m.lower_chest = v.clamp(0.0, 1.0);
}

/// Set lateral rib flare.
pub fn rib_cage_set_lateral_flare(m: &mut RibCageMorph, v: f32) {
    m.lateral_flare = v.clamp(0.0, 1.0);
}

/// Weighted average expansion across chest regions.
pub fn rib_cage_mean_expansion(m: &RibCageMorph) -> f32 {
    (m.expansion + m.upper_chest + m.lower_chest) / 3.0
}

/// Serialize to JSON-like string.
pub fn rib_cage_morph_to_json(m: &RibCageMorph) -> String {
    format!(
        r#"{{"expansion":{:.4},"upper_chest":{:.4},"lower_chest":{:.4},"lateral_flare":{:.4}}}"#,
        m.expansion, m.upper_chest, m.lower_chest, m.lateral_flare
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_rib_cage_morph();
        assert_eq!(m.expansion, 0.0);
    }

    #[test]
    fn test_expansion() {
        let mut m = new_rib_cage_morph();
        rib_cage_set_expansion(&mut m, 0.6);
        assert!((m.expansion - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_upper_chest() {
        let mut m = new_rib_cage_morph();
        rib_cage_set_upper_chest(&mut m, 0.8);
        assert!((m.upper_chest - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_lower_chest_clamp() {
        let mut m = new_rib_cage_morph();
        rib_cage_set_lower_chest(&mut m, 2.0);
        assert_eq!(m.lower_chest, 1.0);
    }

    #[test]
    fn test_lateral_flare() {
        let mut m = new_rib_cage_morph();
        rib_cage_set_lateral_flare(&mut m, 0.3);
        assert!((m.lateral_flare - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_mean_expansion_zero() {
        let m = new_rib_cage_morph();
        assert_eq!(rib_cage_mean_expansion(&m), 0.0);
    }

    #[test]
    fn test_mean_expansion_value() {
        let mut m = new_rib_cage_morph();
        rib_cage_set_expansion(&mut m, 0.6);
        rib_cage_set_upper_chest(&mut m, 0.3);
        rib_cage_set_lower_chest(&mut m, 0.9);
        let mean = rib_cage_mean_expansion(&m);
        assert!((mean - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_json_keys() {
        let m = new_rib_cage_morph();
        let s = rib_cage_morph_to_json(&m);
        assert!(s.contains("lateral_flare"));
    }

    #[test]
    fn test_clone() {
        let m = new_rib_cage_morph();
        let m2 = m.clone();
        assert!((m2.expansion - m.expansion).abs() < 1e-6);
    }
}
