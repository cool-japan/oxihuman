// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ZygomaticusMorph {
    pub major_contraction: f32,
    pub minor_contraction: f32,
    pub dimple_depth: f32,
}

pub fn new_zygomaticus_morph() -> ZygomaticusMorph {
    ZygomaticusMorph {
        major_contraction: 0.0,
        minor_contraction: 0.0,
        dimple_depth: 0.0,
    }
}

pub fn zygomaticus_set_major(m: &mut ZygomaticusMorph, v: f32) {
    m.major_contraction = v.clamp(0.0, 1.0);
}

pub fn zygomaticus_is_smiling(m: &ZygomaticusMorph) -> bool {
    m.major_contraction > 0.3
}

pub fn zygomaticus_overall_weight(m: &ZygomaticusMorph) -> f32 {
    (m.major_contraction + m.minor_contraction) * 0.5
}

pub fn zygomaticus_blend(a: &ZygomaticusMorph, b: &ZygomaticusMorph, t: f32) -> ZygomaticusMorph {
    let t = t.clamp(0.0, 1.0);
    ZygomaticusMorph {
        major_contraction: a.major_contraction + (b.major_contraction - a.major_contraction) * t,
        minor_contraction: a.minor_contraction + (b.minor_contraction - a.minor_contraction) * t,
        dimple_depth: a.dimple_depth + (b.dimple_depth - a.dimple_depth) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_zygomaticus_morph() {
        /* major starts at 0 */
        let m = new_zygomaticus_morph();
        assert!((m.major_contraction - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_zygomaticus_set_major() {
        /* set major contraction */
        let mut m = new_zygomaticus_morph();
        zygomaticus_set_major(&mut m, 0.5);
        assert!((m.major_contraction - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_zygomaticus_is_smiling_true() {
        /* > 0.3 is smiling */
        let mut m = new_zygomaticus_morph();
        zygomaticus_set_major(&mut m, 0.4);
        assert!(zygomaticus_is_smiling(&m));
    }

    #[test]
    fn test_zygomaticus_is_smiling_false() {
        /* 0 not smiling */
        let m = new_zygomaticus_morph();
        assert!(!zygomaticus_is_smiling(&m));
    }

    #[test]
    fn test_zygomaticus_blend() {
        /* blend midpoint */
        let a = new_zygomaticus_morph();
        let mut b = new_zygomaticus_morph();
        b.major_contraction = 1.0;
        let mid = zygomaticus_blend(&a, &b, 0.5);
        assert!((mid.major_contraction - 0.5).abs() < 1e-6);
    }
}
