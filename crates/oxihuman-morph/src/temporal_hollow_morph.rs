// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct TemporalHollowMorph {
    pub depth: f32,
    pub width: f32,
}

pub fn new_temporal_hollow_morph() -> TemporalHollowMorph {
    TemporalHollowMorph {
        depth: 0.0,
        width: 0.5,
    }
}

pub fn temporal_hollow_set_depth(m: &mut TemporalHollowMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

pub fn temporal_hollow_is_sunken(m: &TemporalHollowMorph) -> bool {
    m.depth > 0.5
}

pub fn temporal_hollow_overall_weight(m: &TemporalHollowMorph) -> f32 {
    (m.depth + m.width) * 0.5
}

pub fn temporal_hollow_blend(
    a: &TemporalHollowMorph,
    b: &TemporalHollowMorph,
    t: f32,
) -> TemporalHollowMorph {
    let t = t.clamp(0.0, 1.0);
    TemporalHollowMorph {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_temporal_hollow_morph() {
        /* depth starts at 0 */
        let m = new_temporal_hollow_morph();
        assert!((m.depth - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_temporal_hollow_set_depth() {
        /* set and retrieve depth */
        let mut m = new_temporal_hollow_morph();
        temporal_hollow_set_depth(&mut m, 0.6);
        assert!((m.depth - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_temporal_hollow_is_sunken_true() {
        /* depth > 0.5 is sunken */
        let mut m = new_temporal_hollow_morph();
        temporal_hollow_set_depth(&mut m, 0.8);
        assert!(temporal_hollow_is_sunken(&m));
    }

    #[test]
    fn test_temporal_hollow_is_sunken_false() {
        /* zero depth is not sunken */
        let m = new_temporal_hollow_morph();
        assert!(!temporal_hollow_is_sunken(&m));
    }

    #[test]
    fn test_temporal_hollow_blend() {
        /* blend at t=0.5 midpoint */
        let a = new_temporal_hollow_morph();
        let mut b = new_temporal_hollow_morph();
        b.depth = 1.0;
        let mid = temporal_hollow_blend(&a, &b, 0.5);
        assert!((mid.depth - 0.5).abs() < 1e-6);
    }
}
