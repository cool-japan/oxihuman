// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct NasalTipProjectionMorph {
    pub projection: f32,
    pub rotation: f32,
    pub definition: f32,
}

pub fn new_nasal_tip_projection_morph() -> NasalTipProjectionMorph {
    NasalTipProjectionMorph {
        projection: 0.0,
        rotation: 0.0,
        definition: 0.0,
    }
}

pub fn ntp_set_projection(m: &mut NasalTipProjectionMorph, v: f32) {
    m.projection = v.clamp(0.0, 1.0);
}

pub fn ntp_set_rotation(m: &mut NasalTipProjectionMorph, v: f32) {
    m.rotation = v.clamp(0.0, 1.0);
}

pub fn ntp_overall_weight(m: &NasalTipProjectionMorph) -> f32 {
    (m.projection + m.rotation + m.definition) / 3.0
}

pub fn ntp_blend(
    a: &NasalTipProjectionMorph,
    b: &NasalTipProjectionMorph,
    t: f32,
) -> NasalTipProjectionMorph {
    let t = t.clamp(0.0, 1.0);
    NasalTipProjectionMorph {
        projection: a.projection + (b.projection - a.projection) * t,
        rotation: a.rotation + (b.rotation - a.rotation) * t,
        definition: a.definition + (b.definition - a.definition) * t,
    }
}

pub fn ntp_is_projected(m: &NasalTipProjectionMorph) -> bool {
    m.projection > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_nasal_tip_projection_morph();
        assert_eq!(m.projection, 0.0);
    }

    #[test]
    fn test_set_projection() {
        /* stores value */
        let mut m = new_nasal_tip_projection_morph();
        ntp_set_projection(&mut m, 0.7);
        assert!((m.projection - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_projection_clamp() {
        /* clamps high */
        let mut m = new_nasal_tip_projection_morph();
        ntp_set_projection(&mut m, 3.0);
        assert_eq!(m.projection, 1.0);
    }

    #[test]
    fn test_set_rotation_clamp() {
        /* clamps low */
        let mut m = new_nasal_tip_projection_morph();
        ntp_set_rotation(&mut m, -1.0);
        assert_eq!(m.rotation, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* average of three */
        let m = NasalTipProjectionMorph {
            projection: 0.3,
            rotation: 0.6,
            definition: 0.9,
        };
        assert!((ntp_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_projected_false() {
        /* default not projected */
        let m = new_nasal_tip_projection_morph();
        assert!(!ntp_is_projected(&m));
    }

    #[test]
    fn test_is_projected_true() {
        /* above threshold */
        let m = NasalTipProjectionMorph {
            projection: 0.9,
            rotation: 0.0,
            definition: 0.0,
        };
        assert!(ntp_is_projected(&m));
    }

    #[test]
    fn test_blend() {
        /* t=0.5 midpoint */
        let a = NasalTipProjectionMorph {
            projection: 0.0,
            rotation: 0.0,
            definition: 0.0,
        };
        let b = NasalTipProjectionMorph {
            projection: 1.0,
            rotation: 1.0,
            definition: 1.0,
        };
        let c = ntp_blend(&a, &b, 0.5);
        assert!((c.projection - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = NasalTipProjectionMorph {
            projection: 0.3,
            rotation: 0.2,
            definition: 0.1,
        };
        let m2 = m.clone();
        assert_eq!(m.projection, m2.projection);
    }
}
