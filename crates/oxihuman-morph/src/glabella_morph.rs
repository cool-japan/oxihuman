// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct GlabellaMorph {
    pub prominence: f32,
}

pub fn new_glabella_morph() -> GlabellaMorph {
    GlabellaMorph { prominence: 0.0 }
}

pub fn glabella_set_prominence(m: &mut GlabellaMorph, v: f32) {
    m.prominence = v.clamp(0.0, 1.0);
}

pub fn glabella_is_pronounced(m: &GlabellaMorph) -> bool {
    m.prominence > 0.5
}

pub fn glabella_overall_weight(m: &GlabellaMorph) -> f32 {
    m.prominence
}

pub fn glabella_blend(a: &GlabellaMorph, b: &GlabellaMorph, t: f32) -> GlabellaMorph {
    let t = t.clamp(0.0, 1.0);
    GlabellaMorph {
        prominence: a.prominence + (b.prominence - a.prominence) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* zero init */
        let m = new_glabella_morph();
        assert_eq!(m.prominence, 0.0);
    }

    #[test]
    fn test_set_prominence() {
        /* valid value */
        let mut m = new_glabella_morph();
        glabella_set_prominence(&mut m, 0.6);
        assert!((m.prominence - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_prominence_clamp_high() {
        /* clamp high */
        let mut m = new_glabella_morph();
        glabella_set_prominence(&mut m, 5.0);
        assert_eq!(m.prominence, 1.0);
    }

    #[test]
    fn test_set_prominence_clamp_low() {
        /* clamp low */
        let mut m = new_glabella_morph();
        glabella_set_prominence(&mut m, -1.0);
        assert_eq!(m.prominence, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* weight equals prominence */
        let m = GlabellaMorph { prominence: 0.7 };
        assert!((glabella_overall_weight(&m) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_is_pronounced_false() {
        /* default not pronounced */
        let m = new_glabella_morph();
        assert!(!glabella_is_pronounced(&m));
    }

    #[test]
    fn test_is_pronounced_true() {
        /* above 0.5 */
        let m = GlabellaMorph { prominence: 0.9 };
        assert!(glabella_is_pronounced(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = GlabellaMorph { prominence: 0.0 };
        let b = GlabellaMorph { prominence: 1.0 };
        let c = glabella_blend(&a, &b, 0.5);
        assert!((c.prominence - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp_t() {
        /* t clamped to [0,1] */
        let a = GlabellaMorph { prominence: 0.0 };
        let b = GlabellaMorph { prominence: 1.0 };
        let c = glabella_blend(&a, &b, 2.0);
        assert_eq!(c.prominence, 1.0);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = GlabellaMorph { prominence: 0.3 };
        let m2 = m.clone();
        assert_eq!(m.prominence, m2.prominence);
    }
}
