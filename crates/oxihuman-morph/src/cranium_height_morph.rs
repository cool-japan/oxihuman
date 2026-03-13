// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct CraniumHeightMorph {
    pub vault_height: f32,
    pub brachycephaly: f32,
    pub dolichocephaly: f32,
}

pub fn new_cranium_height_morph() -> CraniumHeightMorph {
    CraniumHeightMorph {
        vault_height: 0.5,
        brachycephaly: 0.0,
        dolichocephaly: 0.0,
    }
}

pub fn cranium_set_vault_height(m: &mut CraniumHeightMorph, v: f32) {
    m.vault_height = v.clamp(0.0, 1.0);
}

pub fn cranium_set_brachycephaly(m: &mut CraniumHeightMorph, v: f32) {
    m.brachycephaly = v.clamp(0.0, 1.0);
}

pub fn cranium_cephalic_index(m: &CraniumHeightMorph) -> f32 {
    (0.77 + m.brachycephaly * 0.1 - m.dolichocephaly * 0.1).clamp(0.0, 1.0)
}

pub fn cranium_is_dolichocephalic(m: &CraniumHeightMorph) -> bool {
    cranium_cephalic_index(m) < 0.75
}

pub fn cranium_blend(a: &CraniumHeightMorph, b: &CraniumHeightMorph, t: f32) -> CraniumHeightMorph {
    let t = t.clamp(0.0, 1.0);
    CraniumHeightMorph {
        vault_height: a.vault_height + (b.vault_height - a.vault_height) * t,
        brachycephaly: a.brachycephaly + (b.brachycephaly - a.brachycephaly) * t,
        dolichocephaly: a.dolichocephaly + (b.dolichocephaly - a.dolichocephaly) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* default vault_height 0.5 */
        let m = new_cranium_height_morph();
        assert!((m.vault_height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_vault_height() {
        /* set vault height */
        let mut m = new_cranium_height_morph();
        cranium_set_vault_height(&mut m, 0.8);
        assert!((m.vault_height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_brachycephaly() {
        /* set brachycephaly */
        let mut m = new_cranium_height_morph();
        cranium_set_brachycephaly(&mut m, 0.5);
        assert!((m.brachycephaly - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cephalic_index_default() {
        /* default cephalic index 0.77 */
        let m = new_cranium_height_morph();
        assert!((cranium_cephalic_index(&m) - 0.77).abs() < 1e-5);
    }

    #[test]
    fn test_is_dolichocephalic() {
        /* high dolichocephaly => index < 0.75 */
        let mut m = new_cranium_height_morph();
        m.dolichocephaly = 1.0;
        assert!(cranium_is_dolichocephalic(&m));
    }

    #[test]
    fn test_blend_at_one() {
        /* blend t=1 returns b */
        let a = new_cranium_height_morph();
        let mut b = new_cranium_height_morph();
        cranium_set_vault_height(&mut b, 1.0);
        let r = cranium_blend(&a, &b, 1.0);
        assert!((r.vault_height - 1.0).abs() < 1e-6);
    }
}
