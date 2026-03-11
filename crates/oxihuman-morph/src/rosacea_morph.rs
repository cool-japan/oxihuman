// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Rosacea redness pattern morph stub.

/// Rosacea subtype.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RosaceaSubtype {
    Erythematotelangiectatic,
    Papulopustular,
    Phymatous,
    Ocular,
}

/// Rosacea morph controller.
#[derive(Debug, Clone)]
pub struct RosaceaMorph {
    pub subtype: RosaceaSubtype,
    pub redness: f32,
    pub telangiectasia: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl RosaceaMorph {
    pub fn new(morph_count: usize) -> Self {
        RosaceaMorph {
            subtype: RosaceaSubtype::Erythematotelangiectatic,
            redness: 0.3,
            telangiectasia: 0.2,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new rosacea morph.
pub fn new_rosacea_morph(morph_count: usize) -> RosaceaMorph {
    RosaceaMorph::new(morph_count)
}

/// Set subtype.
pub fn rsm_set_subtype(morph: &mut RosaceaMorph, subtype: RosaceaSubtype) {
    morph.subtype = subtype;
}

/// Set redness level.
pub fn rsm_set_redness(morph: &mut RosaceaMorph, redness: f32) {
    morph.redness = redness.clamp(0.0, 1.0);
}

/// Set telangiectasia (visible blood vessel) density.
pub fn rsm_set_telangiectasia(morph: &mut RosaceaMorph, density: f32) {
    morph.telangiectasia = density.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: uniform from redness).
pub fn rsm_evaluate(morph: &RosaceaMorph) -> Vec<f32> {
    /* Stub: uniform weight from redness */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.redness; morph.morph_count]
}

/// Enable or disable.
pub fn rsm_set_enabled(morph: &mut RosaceaMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn rsm_to_json(morph: &RosaceaMorph) -> String {
    let sub = match morph.subtype {
        RosaceaSubtype::Erythematotelangiectatic => "erythematotelangiectatic",
        RosaceaSubtype::Papulopustular => "papulopustular",
        RosaceaSubtype::Phymatous => "phymatous",
        RosaceaSubtype::Ocular => "ocular",
    };
    format!(
        r#"{{"subtype":"{}","redness":{},"telangiectasia":{},"morph_count":{},"enabled":{}}}"#,
        sub, morph.redness, morph.telangiectasia, morph.morph_count, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_subtype() {
        let m = new_rosacea_morph(4);
        assert_eq!(
            m.subtype,
            RosaceaSubtype::Erythematotelangiectatic /* default must be Erythematotelangiectatic */
        );
    }

    #[test]
    fn test_set_subtype() {
        let mut m = new_rosacea_morph(4);
        rsm_set_subtype(&mut m, RosaceaSubtype::Phymatous);
        assert_eq!(
            m.subtype,
            RosaceaSubtype::Phymatous /* subtype must be set */
        );
    }

    #[test]
    fn test_redness_clamp() {
        let mut m = new_rosacea_morph(4);
        rsm_set_redness(&mut m, 2.0);
        assert!((m.redness - 1.0).abs() < 1e-6 /* redness clamped to 1.0 */);
    }

    #[test]
    fn test_telangiectasia_clamp() {
        let mut m = new_rosacea_morph(4);
        rsm_set_telangiectasia(&mut m, -0.1);
        assert!(m.telangiectasia.abs() < 1e-6 /* telangiectasia clamped to 0.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_rosacea_morph(5);
        assert_eq!(
            rsm_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_rosacea_morph(4);
        rsm_set_enabled(&mut m, false);
        assert!(rsm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_subtype() {
        let m = new_rosacea_morph(4);
        let j = rsm_to_json(&m);
        assert!(j.contains("\"subtype\"") /* JSON must have subtype */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_rosacea_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_redness() {
        let mut m = new_rosacea_morph(3);
        rsm_set_redness(&mut m, 0.6);
        let out = rsm_evaluate(&m);
        assert!((out[0] - 0.6).abs() < 1e-5 /* evaluate must match redness */);
    }
}
