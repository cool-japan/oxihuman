// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct BreastShapeMorphNew {
    pub volume: f32,
    pub ptosis: f32,
    pub projection: f32,
    pub width: f32,
}

pub fn new_breast_shape_morph_new() -> BreastShapeMorphNew {
    BreastShapeMorphNew {
        volume: 0.3,
        ptosis: 0.0,
        projection: 0.4,
        width: 0.4,
    }
}

pub fn breast_set_volume_new(m: &mut BreastShapeMorphNew, v: f32) {
    m.volume = v.clamp(0.0, 1.0);
}

pub fn breast_bra_size_category(m: &BreastShapeMorphNew) -> &'static str {
    if m.volume < 0.2 {
        "small"
    } else if m.volume < 0.5 {
        "medium"
    } else if m.volume < 0.8 {
        "large"
    } else {
        "xlarge"
    }
}

pub fn breast_overall_weight_new(m: &BreastShapeMorphNew) -> f32 {
    (m.volume + m.projection + m.width) / 3.0
}

pub fn breast_blend_new(
    a: &BreastShapeMorphNew,
    b: &BreastShapeMorphNew,
    t: f32,
) -> BreastShapeMorphNew {
    let t = t.clamp(0.0, 1.0);
    BreastShapeMorphNew {
        volume: a.volume + (b.volume - a.volume) * t,
        ptosis: a.ptosis + (b.ptosis - a.ptosis) * t,
        projection: a.projection + (b.projection - a.projection) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* default volume set */
        let m = new_breast_shape_morph_new();
        assert!(m.volume > 0.0);
    }

    #[test]
    fn test_set_volume() {
        /* volume clamped */
        let mut m = new_breast_shape_morph_new();
        breast_set_volume_new(&mut m, 0.9);
        assert!((m.volume - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_bra_category() {
        /* category changes with volume */
        let mut m = new_breast_shape_morph_new();
        breast_set_volume_new(&mut m, 0.1);
        assert_eq!(breast_bra_size_category(&m), "small");
        breast_set_volume_new(&mut m, 0.9);
        assert_eq!(breast_bra_size_category(&m), "xlarge");
    }

    #[test]
    fn test_overall_weight() {
        /* weight in range */
        let m = new_breast_shape_morph_new();
        let w = breast_overall_weight_new(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = BreastShapeMorphNew {
            volume: 0.0,
            ptosis: 0.0,
            projection: 0.0,
            width: 0.0,
        };
        let b = BreastShapeMorphNew {
            volume: 1.0,
            ptosis: 1.0,
            projection: 1.0,
            width: 1.0,
        };
        let c = breast_blend_new(&a, &b, 0.5);
        assert!((c.volume - 0.5).abs() < 1e-5);
    }
}
