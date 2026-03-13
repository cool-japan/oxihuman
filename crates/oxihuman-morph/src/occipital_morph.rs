// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Occipital bone shape morph.
#[derive(Debug, Clone)]
pub struct OccipitalMorph {
    /// Occipital protuberance prominence (0.0 = flat, 1.0 = prominent).
    pub protuberance: f32,
    /// Nuchal plane width (0.0 = narrow, 1.0 = wide).
    pub nuchal_width: f32,
    /// Curve of the squama (0.0 = flat, 1.0 = strongly curved).
    pub squama_curve: f32,
}

pub fn new_occipital_morph() -> OccipitalMorph {
    OccipitalMorph {
        protuberance: 0.0,
        nuchal_width: 0.0,
        squama_curve: 0.0,
    }
}

pub fn occ_set_protuberance(m: &mut OccipitalMorph, v: f32) {
    m.protuberance = v.clamp(0.0, 1.0);
}

pub fn occ_set_nuchal_width(m: &mut OccipitalMorph, v: f32) {
    m.nuchal_width = v.clamp(0.0, 1.0);
}

pub fn occ_set_squama_curve(m: &mut OccipitalMorph, v: f32) {
    m.squama_curve = v.clamp(0.0, 1.0);
}

pub fn occ_overall_weight(m: &OccipitalMorph) -> f32 {
    (m.protuberance + m.nuchal_width + m.squama_curve) / 3.0
}

pub fn occ_blend(a: &OccipitalMorph, b: &OccipitalMorph, t: f32) -> OccipitalMorph {
    let t = t.clamp(0.0, 1.0);
    OccipitalMorph {
        protuberance: a.protuberance + (b.protuberance - a.protuberance) * t,
        nuchal_width: a.nuchal_width + (b.nuchal_width - a.nuchal_width) * t,
        squama_curve: a.squama_curve + (b.squama_curve - a.squama_curve) * t,
    }
}

pub fn occ_is_neutral(m: &OccipitalMorph) -> bool {
    m.protuberance < 1e-5 && m.nuchal_width < 1e-5 && m.squama_curve < 1e-5
}

pub fn occ_to_json(m: &OccipitalMorph) -> String {
    format!(
        r#"{{"protuberance":{:.4},"nuchal_width":{:.4},"squama_curve":{:.4}}}"#,
        m.protuberance, m.nuchal_width, m.squama_curve
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* all zero */
        let m = new_occipital_morph();
        assert_eq!(m.protuberance, 0.0);
    }

    #[test]
    fn test_set_protuberance() {
        /* valid value */
        let mut m = new_occipital_morph();
        occ_set_protuberance(&mut m, 0.4);
        assert!((m.protuberance - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_high() {
        /* clamp above 1 */
        let mut m = new_occipital_morph();
        occ_set_protuberance(&mut m, 5.0);
        assert_eq!(m.protuberance, 1.0);
    }

    #[test]
    fn test_clamp_low() {
        /* clamp below 0 */
        let mut m = new_occipital_morph();
        occ_set_squama_curve(&mut m, -2.0);
        assert_eq!(m.squama_curve, 0.0);
    }

    #[test]
    fn test_neutral_true() {
        /* default neutral */
        assert!(occ_is_neutral(&new_occipital_morph()));
    }

    #[test]
    fn test_neutral_false() {
        /* non-zero breaks neutral */
        let mut m = new_occipital_morph();
        occ_set_nuchal_width(&mut m, 0.2);
        assert!(!occ_is_neutral(&m));
    }

    #[test]
    fn test_weight() {
        /* average */
        let m = OccipitalMorph {
            protuberance: 0.6,
            nuchal_width: 0.6,
            squama_curve: 0.6,
        };
        assert!((occ_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_blend() {
        /* t=1 gives b */
        let a = new_occipital_morph();
        let b = OccipitalMorph {
            protuberance: 1.0,
            nuchal_width: 0.0,
            squama_curve: 0.0,
        };
        let c = occ_blend(&a, &b, 1.0);
        assert!((c.protuberance - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        /* JSON key present */
        assert!(occ_to_json(&new_occipital_morph()).contains("protuberance"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = OccipitalMorph {
            protuberance: 0.2,
            nuchal_width: 0.5,
            squama_curve: 0.8,
        };
        let m2 = m.clone();
        assert_eq!(m.nuchal_width, m2.nuchal_width);
    }
}
