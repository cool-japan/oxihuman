// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ToothMorph {
    pub size: f32,
    pub protrusion: f32,
    pub spacing: f32,
    pub rotation: f32,
}

pub fn new_tooth_morph() -> ToothMorph {
    ToothMorph {
        size: 0.0,
        protrusion: 0.0,
        spacing: 0.0,
        rotation: 0.0,
    }
}

pub fn tooth_set_size(m: &mut ToothMorph, v: f32) {
    m.size = v.clamp(0.0, 1.0);
}

pub fn tooth_is_prominent(m: &ToothMorph) -> bool {
    m.protrusion > 0.5
}

pub fn tooth_overall_weight(m: &ToothMorph) -> f32 {
    (m.size.abs() + m.protrusion.abs() + m.spacing.abs() + m.rotation.abs()) * 0.25
}

pub fn tooth_blend(a: &ToothMorph, b: &ToothMorph, t: f32) -> ToothMorph {
    let t = t.clamp(0.0, 1.0);
    ToothMorph {
        size: a.size + (b.size - a.size) * t,
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        spacing: a.spacing + (b.spacing - a.spacing) * t,
        rotation: a.rotation + (b.rotation - a.rotation) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tooth_morph() {
        /* default should have all zeros */
        let m = new_tooth_morph();
        assert_eq!(m.size, 0.0);
        assert_eq!(m.protrusion, 0.0);
    }

    #[test]
    fn test_tooth_set_size() {
        /* set size clamps to 0..=1 */
        let mut m = new_tooth_morph();
        tooth_set_size(&mut m, 0.7);
        assert!((m.size - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_tooth_is_prominent() {
        /* protrusion > 0.5 means prominent */
        let mut m = new_tooth_morph();
        m.protrusion = 0.6;
        assert!(tooth_is_prominent(&m));
        m.protrusion = 0.3;
        assert!(!tooth_is_prominent(&m));
    }

    #[test]
    fn test_tooth_overall_weight() {
        /* all fields zero -> weight zero */
        let m = new_tooth_morph();
        assert_eq!(tooth_overall_weight(&m), 0.0);
    }

    #[test]
    fn test_tooth_blend() {
        /* blend at t=0 returns a, at t=1 returns b */
        let a = ToothMorph {
            size: 0.0,
            protrusion: 0.0,
            spacing: 0.0,
            rotation: 0.0,
        };
        let b = ToothMorph {
            size: 1.0,
            protrusion: 1.0,
            spacing: 1.0,
            rotation: 1.0,
        };
        let r = tooth_blend(&a, &b, 0.5);
        assert!((r.size - 0.5).abs() < 1e-6);
    }
}
