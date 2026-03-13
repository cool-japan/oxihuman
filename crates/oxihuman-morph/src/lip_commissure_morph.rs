// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lip commissure angle morph — controls the corner-of-mouth angle and depth.

/// Lip commissure morph configuration.
#[derive(Debug, Clone)]
pub struct LipCommissureMorph {
    pub angle: f32,
    pub depth: f32,
    pub width: f32,
    pub downturn: f32,
    pub dimple_depth: f32,
}

impl LipCommissureMorph {
    pub fn new() -> Self {
        Self {
            angle: 0.0,
            depth: 0.3,
            width: 0.5,
            downturn: 0.0,
            dimple_depth: 0.0,
        }
    }
}

impl Default for LipCommissureMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new lip commissure morph.
pub fn new_lip_commissure_morph() -> LipCommissureMorph {
    LipCommissureMorph::new()
}

/// Set commissure angle (-1 = down-turned, 0 = neutral, 1 = up-turned).
pub fn lcom_set_angle(m: &mut LipCommissureMorph, v: f32) {
    m.angle = v.clamp(-1.0, 1.0);
}

/// Set commissure fold depth.
pub fn lcom_set_depth(m: &mut LipCommissureMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

/// Set inter-commissure width.
pub fn lcom_set_width(m: &mut LipCommissureMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

/// Set downturn offset (marionette line influence).
pub fn lcom_set_downturn(m: &mut LipCommissureMorph, v: f32) {
    m.downturn = v.clamp(-1.0, 1.0);
}

/// Compute effective smile angle taking downturn into account.
pub fn lcom_effective_angle(m: &LipCommissureMorph) -> f32 {
    (m.angle - m.downturn * 0.5).clamp(-1.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn lip_commissure_morph_to_json(m: &LipCommissureMorph) -> String {
    format!(
        r#"{{"angle":{:.4},"depth":{:.4},"width":{:.4},"downturn":{:.4},"dimple_depth":{:.4}}}"#,
        m.angle, m.depth, m.width, m.downturn, m.dimple_depth
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_lip_commissure_morph();
        assert_eq!(m.angle, 0.0);
        assert_eq!(m.dimple_depth, 0.0);
    }

    #[test]
    fn test_angle_clamp_positive() {
        let mut m = new_lip_commissure_morph();
        lcom_set_angle(&mut m, 3.0);
        assert_eq!(m.angle, 1.0);
    }

    #[test]
    fn test_angle_clamp_negative() {
        let mut m = new_lip_commissure_morph();
        lcom_set_angle(&mut m, -3.0);
        assert_eq!(m.angle, -1.0);
    }

    #[test]
    fn test_depth_set() {
        let mut m = new_lip_commissure_morph();
        lcom_set_depth(&mut m, 0.7);
        assert!((m.depth - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_width_set() {
        let mut m = new_lip_commissure_morph();
        lcom_set_width(&mut m, 0.9);
        assert!((m.width - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_downturn_clamp() {
        let mut m = new_lip_commissure_morph();
        lcom_set_downturn(&mut m, -2.0);
        assert_eq!(m.downturn, -1.0);
    }

    #[test]
    fn test_effective_angle_neutral() {
        let m = new_lip_commissure_morph();
        assert_eq!(lcom_effective_angle(&m), 0.0);
    }

    #[test]
    fn test_effective_angle_clamped() {
        let mut m = new_lip_commissure_morph();
        m.angle = 1.0;
        m.downturn = -1.0;
        let ea = lcom_effective_angle(&m);
        assert!((-1.0..=1.0).contains(&ea));
    }

    #[test]
    fn test_json_keys() {
        let m = new_lip_commissure_morph();
        let s = lip_commissure_morph_to_json(&m);
        assert!(s.contains("dimple_depth"));
    }

    #[test]
    fn test_clone() {
        let m = new_lip_commissure_morph();
        let m2 = m.clone();
        assert!((m2.angle - m.angle).abs() < 1e-6);
    }
}
