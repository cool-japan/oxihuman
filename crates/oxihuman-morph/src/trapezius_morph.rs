// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct TrapeziusMorph {
    pub upper_trap_size: f32,
    pub shoulder_height: f32,
    pub definition: f32,
}

pub fn new_trapezius_morph() -> TrapeziusMorph {
    TrapeziusMorph {
        upper_trap_size: 0.0,
        shoulder_height: 0.0,
        definition: 0.0,
    }
}

pub fn trapezius_set_size(m: &mut TrapeziusMorph, v: f32) {
    m.upper_trap_size = v.clamp(0.0, 1.0);
}

pub fn trapezius_is_muscular(m: &TrapeziusMorph) -> bool {
    m.upper_trap_size > 0.6
}

pub fn trapezius_overall_weight(m: &TrapeziusMorph) -> f32 {
    (m.upper_trap_size + m.definition) * 0.5
}

pub fn trapezius_blend(a: &TrapeziusMorph, b: &TrapeziusMorph, t: f32) -> TrapeziusMorph {
    let t = t.clamp(0.0, 1.0);
    TrapeziusMorph {
        upper_trap_size: a.upper_trap_size + (b.upper_trap_size - a.upper_trap_size) * t,
        shoulder_height: a.shoulder_height + (b.shoulder_height - a.shoulder_height) * t,
        definition: a.definition + (b.definition - a.definition) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_trapezius_morph() {
        /* upper_trap_size starts at 0 */
        let m = new_trapezius_morph();
        assert!((m.upper_trap_size - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_trapezius_set_size() {
        /* set size */
        let mut m = new_trapezius_morph();
        trapezius_set_size(&mut m, 0.7);
        assert!((m.upper_trap_size - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_trapezius_is_muscular_true() {
        /* > 0.6 is muscular */
        let mut m = new_trapezius_morph();
        trapezius_set_size(&mut m, 0.9);
        assert!(trapezius_is_muscular(&m));
    }

    #[test]
    fn test_trapezius_is_muscular_false() {
        /* 0 not muscular */
        let m = new_trapezius_morph();
        assert!(!trapezius_is_muscular(&m));
    }

    #[test]
    fn test_trapezius_blend() {
        /* blend midpoint */
        let a = new_trapezius_morph();
        let mut b = new_trapezius_morph();
        b.upper_trap_size = 1.0;
        let mid = trapezius_blend(&a, &b, 0.5);
        assert!((mid.upper_trap_size - 0.5).abs() < 1e-6);
    }
}
