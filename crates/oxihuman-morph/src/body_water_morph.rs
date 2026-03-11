// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct BodyWaterMorph {
    pub hydration: f32,
    pub skin_turgor: f32,
}

pub fn new_body_water_morph() -> BodyWaterMorph {
    BodyWaterMorph {
        hydration: 0.6,
        skin_turgor: 0.5,
    }
}

pub fn body_water_set_hydration(m: &mut BodyWaterMorph, v: f32) {
    m.hydration = v.clamp(0.0, 1.0);
}

pub fn body_water_is_dehydrated(m: &BodyWaterMorph) -> bool {
    m.hydration < 0.3
}

pub fn body_water_overall_weight(m: &BodyWaterMorph) -> f32 {
    (m.hydration + m.skin_turgor) * 0.5
}

pub fn body_water_blend(a: &BodyWaterMorph, b: &BodyWaterMorph, t: f32) -> BodyWaterMorph {
    let t = t.clamp(0.0, 1.0);
    BodyWaterMorph {
        hydration: a.hydration + (b.hydration - a.hydration) * t,
        skin_turgor: a.skin_turgor + (b.skin_turgor - a.skin_turgor) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default hydration above zero */
        let m = new_body_water_morph();
        assert!(m.hydration > 0.0);
    }

    #[test]
    fn test_set_hydration() {
        /* clamped */
        let mut m = new_body_water_morph();
        body_water_set_hydration(&mut m, 0.2);
        assert!((m.hydration - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_is_dehydrated() {
        /* dehydrated when hydration < 0.3 */
        let mut m = new_body_water_morph();
        body_water_set_hydration(&mut m, 0.1);
        assert!(body_water_is_dehydrated(&m));
    }

    #[test]
    fn test_not_dehydrated_by_default() {
        /* default hydration 0.6 is not dehydrated */
        let m = new_body_water_morph();
        assert!(!body_water_is_dehydrated(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = BodyWaterMorph {
            hydration: 0.0,
            skin_turgor: 0.0,
        };
        let b = BodyWaterMorph {
            hydration: 1.0,
            skin_turgor: 1.0,
        };
        let c = body_water_blend(&a, &b, 0.5);
        assert!((c.hydration - 0.5).abs() < 1e-5);
    }
}
