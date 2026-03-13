// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ScleraShowMorph {
    pub inferior_show: f32,
    pub superior_show: f32,
    pub lateral_show: f32,
}

pub fn new_sclera_show_morph() -> ScleraShowMorph {
    ScleraShowMorph {
        inferior_show: 0.0,
        superior_show: 0.0,
        lateral_show: 0.0,
    }
}

pub fn sclera_set_inferior(m: &mut ScleraShowMorph, v: f32) {
    m.inferior_show = v.clamp(0.0, 1.0);
}

pub fn sclera_has_sanpaku(m: &ScleraShowMorph) -> bool {
    m.inferior_show > 0.3
}

pub fn sclera_overall_weight(m: &ScleraShowMorph) -> f32 {
    (m.inferior_show.abs() + m.superior_show.abs() + m.lateral_show.abs()) / 3.0
}

pub fn sclera_blend(a: &ScleraShowMorph, b: &ScleraShowMorph, t: f32) -> ScleraShowMorph {
    let t = t.clamp(0.0, 1.0);
    ScleraShowMorph {
        inferior_show: a.inferior_show + (b.inferior_show - a.inferior_show) * t,
        superior_show: a.superior_show + (b.superior_show - a.superior_show) * t,
        lateral_show: a.lateral_show + (b.lateral_show - a.lateral_show) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sclera_show_morph() {
        /* defaults to zero */
        let m = new_sclera_show_morph();
        assert_eq!(m.inferior_show, 0.0);
    }

    #[test]
    fn test_sclera_set_inferior() {
        /* inferior is set */
        let mut m = new_sclera_show_morph();
        sclera_set_inferior(&mut m, 0.5);
        assert!((m.inferior_show - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_sclera_has_sanpaku_false() {
        /* not sanpaku at default */
        let m = new_sclera_show_morph();
        assert!(!sclera_has_sanpaku(&m));
    }

    #[test]
    fn test_sclera_has_sanpaku_true() {
        /* sanpaku when inferior > 0.3 */
        let mut m = new_sclera_show_morph();
        m.inferior_show = 0.4;
        assert!(sclera_has_sanpaku(&m));
    }

    #[test]
    fn test_sclera_blend() {
        /* blend at 0.5 is midpoint */
        let a = new_sclera_show_morph();
        let b = ScleraShowMorph {
            inferior_show: 1.0,
            superior_show: 1.0,
            lateral_show: 1.0,
        };
        let r = sclera_blend(&a, &b, 0.5);
        assert!((r.inferior_show - 0.5).abs() < 1e-6);
    }
}
