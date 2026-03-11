// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct BrowBoneBossingMorph {
    pub central: f32,
    pub lateral_left: f32,
    pub lateral_right: f32,
}

pub fn new_brow_bone_bossing_morph() -> BrowBoneBossingMorph {
    BrowBoneBossingMorph {
        central: 0.0,
        lateral_left: 0.0,
        lateral_right: 0.0,
    }
}

pub fn bbb_set_central(m: &mut BrowBoneBossingMorph, v: f32) {
    m.central = v.clamp(0.0, 1.0);
}

pub fn bbb_set_lateral(m: &mut BrowBoneBossingMorph, v: f32) {
    let v = v.clamp(0.0, 1.0);
    m.lateral_left = v;
    m.lateral_right = v;
}

pub fn bbb_overall_weight(m: &BrowBoneBossingMorph) -> f32 {
    (m.central + m.lateral_left + m.lateral_right) / 3.0
}

pub fn bbb_blend(
    a: &BrowBoneBossingMorph,
    b: &BrowBoneBossingMorph,
    t: f32,
) -> BrowBoneBossingMorph {
    let t = t.clamp(0.0, 1.0);
    BrowBoneBossingMorph {
        central: a.central + (b.central - a.central) * t,
        lateral_left: a.lateral_left + (b.lateral_left - a.lateral_left) * t,
        lateral_right: a.lateral_right + (b.lateral_right - a.lateral_right) * t,
    }
}

pub fn bbb_is_prominent(m: &BrowBoneBossingMorph) -> bool {
    m.central > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_brow_bone_bossing_morph();
        assert_eq!(m.central, 0.0);
    }

    #[test]
    fn test_set_central() {
        /* stores value */
        let mut m = new_brow_bone_bossing_morph();
        bbb_set_central(&mut m, 0.6);
        assert!((m.central - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_central_clamp() {
        /* clamp high */
        let mut m = new_brow_bone_bossing_morph();
        bbb_set_central(&mut m, 2.0);
        assert_eq!(m.central, 1.0);
    }

    #[test]
    fn test_set_lateral() {
        /* both lateral fields set */
        let mut m = new_brow_bone_bossing_morph();
        bbb_set_lateral(&mut m, 0.5);
        assert!((m.lateral_left - 0.5).abs() < 1e-6);
        assert!((m.lateral_right - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = BrowBoneBossingMorph {
            central: 0.3,
            lateral_left: 0.6,
            lateral_right: 0.9,
        };
        assert!((bbb_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_prominent_false() {
        /* default not prominent */
        let m = new_brow_bone_bossing_morph();
        assert!(!bbb_is_prominent(&m));
    }

    #[test]
    fn test_is_prominent_true() {
        /* central > 0.5 */
        let m = BrowBoneBossingMorph {
            central: 0.9,
            lateral_left: 0.0,
            lateral_right: 0.0,
        };
        assert!(bbb_is_prominent(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = BrowBoneBossingMorph {
            central: 0.0,
            lateral_left: 0.0,
            lateral_right: 0.0,
        };
        let b = BrowBoneBossingMorph {
            central: 1.0,
            lateral_left: 1.0,
            lateral_right: 1.0,
        };
        let c = bbb_blend(&a, &b, 0.5);
        assert!((c.central - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = BrowBoneBossingMorph {
            central: 0.4,
            lateral_left: 0.3,
            lateral_right: 0.2,
        };
        let m2 = m.clone();
        assert_eq!(m.central, m2.central);
    }
}
