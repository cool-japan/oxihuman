// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct GluteMorph {
    pub volume: f32,
    pub roundness: f32,
    pub lift: f32,
}

pub fn new_glute_morph() -> GluteMorph {
    GluteMorph {
        volume: 0.4,
        roundness: 0.4,
        lift: 0.3,
    }
}

pub fn glute_set_volume(m: &mut GluteMorph, v: f32) {
    m.volume = v.clamp(0.0, 1.0);
}

pub fn glute_is_prominent(m: &GluteMorph) -> bool {
    (m.volume + m.lift) * 0.5 > 0.5
}

pub fn glute_overall_weight(m: &GluteMorph) -> f32 {
    (m.volume + m.roundness + m.lift) / 3.0
}

pub fn glute_blend(a: &GluteMorph, b: &GluteMorph, t: f32) -> GluteMorph {
    let t = t.clamp(0.0, 1.0);
    GluteMorph {
        volume: a.volume + (b.volume - a.volume) * t,
        roundness: a.roundness + (b.roundness - a.roundness) * t,
        lift: a.lift + (b.lift - a.lift) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* volume > 0 by default */
        let m = new_glute_morph();
        assert!(m.volume > 0.0);
    }

    #[test]
    fn test_set_volume() {
        /* clamped */
        let mut m = new_glute_morph();
        glute_set_volume(&mut m, 0.9);
        assert!((m.volume - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_is_prominent() {
        /* prominence check */
        let mut m = new_glute_morph();
        glute_set_volume(&mut m, 0.8);
        m.lift = 0.8;
        assert!(glute_is_prominent(&m));
    }

    #[test]
    fn test_overall_weight_range() {
        /* in range */
        let m = new_glute_morph();
        let w = glute_overall_weight(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = GluteMorph {
            volume: 0.0,
            roundness: 0.0,
            lift: 0.0,
        };
        let b = GluteMorph {
            volume: 1.0,
            roundness: 1.0,
            lift: 1.0,
        };
        let c = glute_blend(&a, &b, 0.5);
        assert!((c.volume - 0.5).abs() < 1e-5);
    }
}
