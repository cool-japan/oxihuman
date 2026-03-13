// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Pubic arch angle morph.
#[derive(Debug, Clone)]
pub struct PubicArchMorph {
    /// Sub-pubic angle expressed as fraction (0.0 = narrow/male, 1.0 = wide/female).
    pub angle_factor: f32,
    /// Pubic symphysis height (0.0 = short, 1.0 = tall).
    pub symphysis_height: f32,
    /// Medial border concavity (0.0 = convex, 1.0 = concave).
    pub concavity: f32,
}

pub fn new_pubic_arch_morph() -> PubicArchMorph {
    PubicArchMorph {
        angle_factor: 0.0,
        symphysis_height: 0.0,
        concavity: 0.0,
    }
}

pub fn pub_set_angle_factor(m: &mut PubicArchMorph, v: f32) {
    m.angle_factor = v.clamp(0.0, 1.0);
}

pub fn pub_set_symphysis_height(m: &mut PubicArchMorph, v: f32) {
    m.symphysis_height = v.clamp(0.0, 1.0);
}

pub fn pub_set_concavity(m: &mut PubicArchMorph, v: f32) {
    m.concavity = v.clamp(0.0, 1.0);
}

/// Returns approximate sub-pubic angle in degrees (50° narrow … 90° wide).
pub fn pub_angle_deg(m: &PubicArchMorph) -> f32 {
    50.0 + m.angle_factor * 40.0
}

/// Returns approximate sub-pubic angle in radians.
pub fn pub_angle_rad(m: &PubicArchMorph) -> f32 {
    pub_angle_deg(m) * PI / 180.0
}

pub fn pub_overall_weight(m: &PubicArchMorph) -> f32 {
    (m.angle_factor + m.symphysis_height + m.concavity) / 3.0
}

pub fn pub_blend(a: &PubicArchMorph, b: &PubicArchMorph, t: f32) -> PubicArchMorph {
    let t = t.clamp(0.0, 1.0);
    PubicArchMorph {
        angle_factor: a.angle_factor + (b.angle_factor - a.angle_factor) * t,
        symphysis_height: a.symphysis_height + (b.symphysis_height - a.symphysis_height) * t,
        concavity: a.concavity + (b.concavity - a.concavity) * t,
    }
}

pub fn pub_is_neutral(m: &PubicArchMorph) -> bool {
    m.angle_factor < 1e-5 && m.symphysis_height < 1e-5 && m.concavity < 1e-5
}

pub fn pub_to_json(m: &PubicArchMorph) -> String {
    format!(
        r#"{{"angle_factor":{:.4},"symphysis_height":{:.4},"concavity":{:.4}}}"#,
        m.angle_factor, m.symphysis_height, m.concavity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* all zero */
        let m = new_pubic_arch_morph();
        assert_eq!(m.angle_factor, 0.0);
    }

    #[test]
    fn test_angle_deg_min() {
        /* narrow arch ~50 deg */
        let m = new_pubic_arch_morph();
        assert!((pub_angle_deg(&m) - 50.0).abs() < 1e-5);
    }

    #[test]
    fn test_angle_deg_max() {
        /* wide arch ~90 deg */
        let mut m = new_pubic_arch_morph();
        pub_set_angle_factor(&mut m, 1.0);
        assert!((pub_angle_deg(&m) - 90.0).abs() < 1e-5);
    }

    #[test]
    fn test_clamp_high() {
        /* clamp above 1 */
        let mut m = new_pubic_arch_morph();
        pub_set_angle_factor(&mut m, 5.0);
        assert_eq!(m.angle_factor, 1.0);
    }

    #[test]
    fn test_clamp_low() {
        /* clamp below 0 */
        let mut m = new_pubic_arch_morph();
        pub_set_concavity(&mut m, -0.5);
        assert_eq!(m.concavity, 0.0);
    }

    #[test]
    fn test_neutral_true() {
        /* default neutral */
        assert!(pub_is_neutral(&new_pubic_arch_morph()));
    }

    #[test]
    fn test_neutral_false() {
        /* non-zero */
        let mut m = new_pubic_arch_morph();
        pub_set_symphysis_height(&mut m, 0.5);
        assert!(!pub_is_neutral(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = new_pubic_arch_morph();
        let b = PubicArchMorph {
            angle_factor: 1.0,
            symphysis_height: 0.0,
            concavity: 0.0,
        };
        let c = pub_blend(&a, &b, 0.5);
        assert!((c.angle_factor - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        /* JSON has angle_factor */
        assert!(pub_to_json(&new_pubic_arch_morph()).contains("angle_factor"));
    }

    #[test]
    fn test_angle_rad_range() {
        /* angle in radians within plausible range */
        let m = new_pubic_arch_morph();
        let r = pub_angle_rad(&m);
        assert!(r > 0.5 && r < 2.0);
    }
}
