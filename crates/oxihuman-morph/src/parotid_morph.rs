// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ParotidMorph {
    pub size: f32,
    pub prominence: f32,
}

pub fn new_parotid_morph() -> ParotidMorph {
    ParotidMorph {
        size: 0.0,
        prominence: 0.0,
    }
}

pub fn parotid_set_size(m: &mut ParotidMorph, v: f32) {
    m.size = v.clamp(0.0, 1.0);
}

pub fn parotid_is_prominent(m: &ParotidMorph) -> bool {
    m.size > 0.5
}

pub fn parotid_overall_weight(m: &ParotidMorph) -> f32 {
    (m.size + m.prominence) * 0.5
}

pub fn parotid_blend(a: &ParotidMorph, b: &ParotidMorph, t: f32) -> ParotidMorph {
    let t = t.clamp(0.0, 1.0);
    ParotidMorph {
        size: a.size + (b.size - a.size) * t,
        prominence: a.prominence + (b.prominence - a.prominence) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_parotid_morph() {
        /* size starts at 0 */
        let m = new_parotid_morph();
        assert!((m.size - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_parotid_set_size() {
        /* set and retrieve size */
        let mut m = new_parotid_morph();
        parotid_set_size(&mut m, 0.6);
        assert!((m.size - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_parotid_is_prominent_true() {
        /* > 0.5 is prominent */
        let mut m = new_parotid_morph();
        parotid_set_size(&mut m, 0.8);
        assert!(parotid_is_prominent(&m));
    }

    #[test]
    fn test_parotid_is_prominent_false() {
        /* 0 not prominent */
        let m = new_parotid_morph();
        assert!(!parotid_is_prominent(&m));
    }

    #[test]
    fn test_parotid_blend() {
        /* blend midpoint */
        let a = new_parotid_morph();
        let mut b = new_parotid_morph();
        b.size = 1.0;
        let mid = parotid_blend(&a, &b, 0.5);
        assert!((mid.size - 0.5).abs() < 1e-6);
    }
}
