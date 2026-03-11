// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Parietal bone curvature morph.
#[derive(Debug, Clone)]
pub struct ParietalMorph {
    /// Coronal curvature (0.0 = flat, 1.0 = highly curved).
    pub coronal_curve: f32,
    /// Sagittal curvature (0.0 = flat, 1.0 = domed).
    pub sagittal_curve: f32,
    /// Parietal boss prominence (0.0 = absent, 1.0 = prominent).
    pub boss: f32,
}

pub fn new_parietal_morph() -> ParietalMorph {
    ParietalMorph {
        coronal_curve: 0.0,
        sagittal_curve: 0.0,
        boss: 0.0,
    }
}

pub fn par_set_coronal_curve(m: &mut ParietalMorph, v: f32) {
    m.coronal_curve = v.clamp(0.0, 1.0);
}

pub fn par_set_sagittal_curve(m: &mut ParietalMorph, v: f32) {
    m.sagittal_curve = v.clamp(0.0, 1.0);
}

pub fn par_set_boss(m: &mut ParietalMorph, v: f32) {
    m.boss = v.clamp(0.0, 1.0);
}

pub fn par_overall_weight(m: &ParietalMorph) -> f32 {
    (m.coronal_curve + m.sagittal_curve + m.boss) / 3.0
}

pub fn par_blend(a: &ParietalMorph, b: &ParietalMorph, t: f32) -> ParietalMorph {
    let t = t.clamp(0.0, 1.0);
    ParietalMorph {
        coronal_curve: a.coronal_curve + (b.coronal_curve - a.coronal_curve) * t,
        sagittal_curve: a.sagittal_curve + (b.sagittal_curve - a.sagittal_curve) * t,
        boss: a.boss + (b.boss - a.boss) * t,
    }
}

pub fn par_is_neutral(m: &ParietalMorph) -> bool {
    m.coronal_curve < 1e-5 && m.sagittal_curve < 1e-5 && m.boss < 1e-5
}

pub fn par_to_json(m: &ParietalMorph) -> String {
    format!(
        r#"{{"coronal_curve":{:.4},"sagittal_curve":{:.4},"boss":{:.4}}}"#,
        m.coronal_curve, m.sagittal_curve, m.boss
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* all zero */
        let m = new_parietal_morph();
        assert_eq!(m.coronal_curve, 0.0);
    }

    #[test]
    fn test_set_coronal() {
        /* valid value */
        let mut m = new_parietal_morph();
        par_set_coronal_curve(&mut m, 0.5);
        assert!((m.coronal_curve - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_high() {
        /* clamp */
        let mut m = new_parietal_morph();
        par_set_coronal_curve(&mut m, 2.0);
        assert_eq!(m.coronal_curve, 1.0);
    }

    #[test]
    fn test_clamp_low() {
        /* clamp low */
        let mut m = new_parietal_morph();
        par_set_boss(&mut m, -1.0);
        assert_eq!(m.boss, 0.0);
    }

    #[test]
    fn test_neutral_true() {
        /* default neutral */
        assert!(par_is_neutral(&new_parietal_morph()));
    }

    #[test]
    fn test_neutral_false() {
        /* after setting boss */
        let mut m = new_parietal_morph();
        par_set_boss(&mut m, 0.1);
        assert!(!par_is_neutral(&m));
    }

    #[test]
    fn test_weight() {
        /* formula */
        let m = ParietalMorph {
            coronal_curve: 0.3,
            sagittal_curve: 0.6,
            boss: 0.9,
        };
        assert!((par_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = new_parietal_morph();
        let b = ParietalMorph {
            coronal_curve: 1.0,
            sagittal_curve: 0.0,
            boss: 0.0,
        };
        let c = par_blend(&a, &b, 0.5);
        assert!((c.coronal_curve - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        /* JSON has coronal_curve */
        assert!(par_to_json(&new_parietal_morph()).contains("coronal_curve"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = ParietalMorph {
            coronal_curve: 0.1,
            sagittal_curve: 0.2,
            boss: 0.3,
        };
        let m2 = m.clone();
        assert_eq!(m.boss, m2.boss);
    }
}
