// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ColumellaMorph {
    pub inclination: f32,
    pub width: f32,
    pub hanging: f32,
}

pub fn new_columella_morph() -> ColumellaMorph {
    ColumellaMorph {
        inclination: 0.0,
        width: 0.0,
        hanging: 0.0,
    }
}

pub fn columella_set_inclination(m: &mut ColumellaMorph, v: f32) {
    m.inclination = v.clamp(0.0, 1.0);
}

pub fn columella_set_width(m: &mut ColumellaMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn columella_overall_weight(m: &ColumellaMorph) -> f32 {
    (m.inclination + m.width + m.hanging) / 3.0
}

pub fn columella_blend(a: &ColumellaMorph, b: &ColumellaMorph, t: f32) -> ColumellaMorph {
    let t = t.clamp(0.0, 1.0);
    ColumellaMorph {
        inclination: a.inclination + (b.inclination - a.inclination) * t,
        width: a.width + (b.width - a.width) * t,
        hanging: a.hanging + (b.hanging - a.hanging) * t,
    }
}

pub fn columella_is_hanging(m: &ColumellaMorph) -> bool {
    m.hanging > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_columella_morph();
        assert_eq!(m.inclination, 0.0);
    }

    #[test]
    fn test_set_inclination() {
        /* stores valid value */
        let mut m = new_columella_morph();
        columella_set_inclination(&mut m, 0.6);
        assert!((m.inclination - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_inclination_clamp() {
        /* clamps high */
        let mut m = new_columella_morph();
        columella_set_inclination(&mut m, 2.0);
        assert_eq!(m.inclination, 1.0);
    }

    #[test]
    fn test_set_width() {
        /* valid width stored */
        let mut m = new_columella_morph();
        columella_set_width(&mut m, 0.3);
        assert!((m.width - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = ColumellaMorph {
            inclination: 0.3,
            width: 0.6,
            hanging: 0.9,
        };
        assert!((columella_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_hanging_false() {
        /* default not hanging */
        let m = new_columella_morph();
        assert!(!columella_is_hanging(&m));
    }

    #[test]
    fn test_is_hanging_true() {
        /* above threshold */
        let m = ColumellaMorph {
            inclination: 0.0,
            width: 0.0,
            hanging: 0.8,
        };
        assert!(columella_is_hanging(&m));
    }

    #[test]
    fn test_blend() {
        /* blend at 0.5 */
        let a = ColumellaMorph {
            inclination: 0.0,
            width: 0.0,
            hanging: 0.0,
        };
        let b = ColumellaMorph {
            inclination: 1.0,
            width: 1.0,
            hanging: 1.0,
        };
        let c = columella_blend(&a, &b, 0.5);
        assert!((c.hanging - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = ColumellaMorph {
            inclination: 0.2,
            width: 0.4,
            hanging: 0.6,
        };
        let m2 = m.clone();
        assert_eq!(m.hanging, m2.hanging);
    }
}
