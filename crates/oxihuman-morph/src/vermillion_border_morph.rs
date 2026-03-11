// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct VermillionBorderMorph {
    pub sharpness: f32,
    pub upper_height: f32,
    pub lower_height: f32,
}

pub fn new_vermillion_border_morph() -> VermillionBorderMorph {
    VermillionBorderMorph {
        sharpness: 0.0,
        upper_height: 0.0,
        lower_height: 0.0,
    }
}

pub fn vermillion_set_sharpness(m: &mut VermillionBorderMorph, v: f32) {
    m.sharpness = v.clamp(0.0, 1.0);
}

pub fn vermillion_is_defined(m: &VermillionBorderMorph) -> bool {
    m.sharpness > 0.3
}

pub fn vermillion_overall_weight(m: &VermillionBorderMorph) -> f32 {
    (m.sharpness.abs() + m.upper_height.abs() + m.lower_height.abs()) / 3.0
}

pub fn vermillion_blend(
    a: &VermillionBorderMorph,
    b: &VermillionBorderMorph,
    t: f32,
) -> VermillionBorderMorph {
    let t = t.clamp(0.0, 1.0);
    VermillionBorderMorph {
        sharpness: a.sharpness + (b.sharpness - a.sharpness) * t,
        upper_height: a.upper_height + (b.upper_height - a.upper_height) * t,
        lower_height: a.lower_height + (b.lower_height - a.lower_height) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vermillion_border_morph() {
        /* sharpness defaults to 0 */
        let m = new_vermillion_border_morph();
        assert_eq!(m.sharpness, 0.0);
    }

    #[test]
    fn test_vermillion_set_sharpness() {
        /* sharpness is set */
        let mut m = new_vermillion_border_morph();
        vermillion_set_sharpness(&mut m, 0.9);
        assert!((m.sharpness - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_vermillion_is_defined_false() {
        /* not defined at default */
        let m = new_vermillion_border_morph();
        assert!(!vermillion_is_defined(&m));
    }

    #[test]
    fn test_vermillion_is_defined_true() {
        /* defined when sharpness > 0.3 */
        let mut m = new_vermillion_border_morph();
        m.sharpness = 0.5;
        assert!(vermillion_is_defined(&m));
    }

    #[test]
    fn test_vermillion_blend() {
        /* blend at t=0 returns a */
        let a = new_vermillion_border_morph();
        let b = VermillionBorderMorph {
            sharpness: 1.0,
            upper_height: 1.0,
            lower_height: 1.0,
        };
        let r = vermillion_blend(&a, &b, 0.0);
        assert_eq!(r.sharpness, 0.0);
    }
}
