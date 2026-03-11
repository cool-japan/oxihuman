// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Tongue tip shape morph — controls apex protrusion, curl, and sharpness.

/// Tongue tip shape morph configuration.
#[derive(Debug, Clone)]
pub struct TongueTipMorph {
    pub protrusion: f32,
    pub curl: f32,
    pub sharpness: f32,
    pub lateral_spread: f32,
}

impl TongueTipMorph {
    pub fn new() -> Self {
        Self {
            protrusion: 0.0,
            curl: 0.0,
            sharpness: 0.5,
            lateral_spread: 0.0,
        }
    }
}

impl Default for TongueTipMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new tongue tip morph.
pub fn new_tongue_tip_morph() -> TongueTipMorph {
    TongueTipMorph::new()
}

/// Set tip protrusion in normalised range.
pub fn tongue_tip_set_protrusion(m: &mut TongueTipMorph, v: f32) {
    m.protrusion = v.clamp(0.0, 1.0);
}

/// Set apex curl amount (negative = down-curl).
pub fn tongue_tip_set_curl(m: &mut TongueTipMorph, v: f32) {
    m.curl = v.clamp(-1.0, 1.0);
}

/// Set sharpness of the tongue tip.
pub fn tongue_tip_set_sharpness(m: &mut TongueTipMorph, v: f32) {
    m.sharpness = v.clamp(0.0, 1.0);
}

/// Set lateral spread of the tongue tip.
pub fn tongue_tip_set_lateral_spread(m: &mut TongueTipMorph, v: f32) {
    m.lateral_spread = v.clamp(0.0, 1.0);
}

/// Serialize to JSON-like string.
pub fn tongue_tip_morph_to_json(m: &TongueTipMorph) -> String {
    format!(
        r#"{{"protrusion":{:.4},"curl":{:.4},"sharpness":{:.4},"lateral_spread":{:.4}}}"#,
        m.protrusion, m.curl, m.sharpness, m.lateral_spread
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_tongue_tip_morph();
        assert_eq!(m.protrusion, 0.0);
        assert_eq!(m.curl, 0.0);
    }

    #[test]
    fn test_set_protrusion() {
        let mut m = new_tongue_tip_morph();
        tongue_tip_set_protrusion(&mut m, 0.7);
        assert!((m.protrusion - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_protrusion_clamp_high() {
        let mut m = new_tongue_tip_morph();
        tongue_tip_set_protrusion(&mut m, 5.0);
        assert_eq!(m.protrusion, 1.0);
    }

    #[test]
    fn test_set_curl_negative() {
        let mut m = new_tongue_tip_morph();
        tongue_tip_set_curl(&mut m, -0.5);
        assert!((m.curl + 0.5).abs() < 1e-6); /* down-curl */
    }

    #[test]
    fn test_curl_clamp() {
        let mut m = new_tongue_tip_morph();
        tongue_tip_set_curl(&mut m, 2.0);
        assert_eq!(m.curl, 1.0);
    }

    #[test]
    fn test_set_sharpness() {
        let mut m = new_tongue_tip_morph();
        tongue_tip_set_sharpness(&mut m, 0.9);
        assert!((m.sharpness - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_lateral_spread() {
        let mut m = new_tongue_tip_morph();
        tongue_tip_set_lateral_spread(&mut m, 0.3);
        assert!((m.lateral_spread - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_json_contains_keys() {
        let m = new_tongue_tip_morph();
        let s = tongue_tip_morph_to_json(&m);
        assert!(s.contains("sharpness"));
    }

    #[test]
    fn test_clone() {
        let m = new_tongue_tip_morph();
        let m2 = m.clone();
        assert!((m2.sharpness - m.sharpness).abs() < 1e-6);
    }
}
