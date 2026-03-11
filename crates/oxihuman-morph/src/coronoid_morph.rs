// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Coronoid process of the mandible — shape morph.
#[derive(Debug, Clone)]
pub struct CoronoidMorph {
    /// Height of the coronoid process (0.0 = flat, 1.0 = prominent).
    pub height: f32,
    /// Anterior-posterior taper of the process (0.0 = narrow, 1.0 = broad).
    pub breadth: f32,
    /// Medial curvature of the apex (0.0 = straight, 1.0 = strongly curved).
    pub apex_curve: f32,
}

pub fn new_coronoid_morph() -> CoronoidMorph {
    CoronoidMorph {
        height: 0.0,
        breadth: 0.0,
        apex_curve: 0.0,
    }
}

pub fn cor_set_height(m: &mut CoronoidMorph, v: f32) {
    m.height = v.clamp(0.0, 1.0);
}

pub fn cor_set_breadth(m: &mut CoronoidMorph, v: f32) {
    m.breadth = v.clamp(0.0, 1.0);
}

pub fn cor_set_apex_curve(m: &mut CoronoidMorph, v: f32) {
    m.apex_curve = v.clamp(0.0, 1.0);
}

pub fn cor_overall_weight(m: &CoronoidMorph) -> f32 {
    (m.height + m.breadth + m.apex_curve) / 3.0
}

pub fn cor_blend(a: &CoronoidMorph, b: &CoronoidMorph, t: f32) -> CoronoidMorph {
    let t = t.clamp(0.0, 1.0);
    CoronoidMorph {
        height: a.height + (b.height - a.height) * t,
        breadth: a.breadth + (b.breadth - a.breadth) * t,
        apex_curve: a.apex_curve + (b.apex_curve - a.apex_curve) * t,
    }
}

pub fn cor_is_neutral(m: &CoronoidMorph) -> bool {
    m.height < 1e-5 && m.breadth < 1e-5 && m.apex_curve < 1e-5
}

pub fn cor_to_json(m: &CoronoidMorph) -> String {
    format!(
        r#"{{"height":{:.4},"breadth":{:.4},"apex_curve":{:.4}}}"#,
        m.height, m.breadth, m.apex_curve
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all fields zero */
        let m = new_coronoid_morph();
        assert_eq!(m.height, 0.0);
        assert_eq!(m.breadth, 0.0);
        assert_eq!(m.apex_curve, 0.0);
    }

    #[test]
    fn test_set_height() {
        /* valid height stored */
        let mut m = new_coronoid_morph();
        cor_set_height(&mut m, 0.7);
        assert!((m.height - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_height_clamp_high() {
        /* clamp above 1 */
        let mut m = new_coronoid_morph();
        cor_set_height(&mut m, 5.0);
        assert_eq!(m.height, 1.0);
    }

    #[test]
    fn test_set_breadth_clamp_low() {
        /* clamp below 0 */
        let mut m = new_coronoid_morph();
        cor_set_breadth(&mut m, -1.0);
        assert_eq!(m.breadth, 0.0);
    }

    #[test]
    fn test_is_neutral_true() {
        /* default is neutral */
        let m = new_coronoid_morph();
        assert!(cor_is_neutral(&m));
    }

    #[test]
    fn test_is_neutral_false() {
        /* after setting height no longer neutral */
        let mut m = new_coronoid_morph();
        cor_set_height(&mut m, 0.5);
        assert!(!cor_is_neutral(&m));
    }

    #[test]
    fn test_overall_weight() {
        /* average of three fields */
        let m = CoronoidMorph {
            height: 0.3,
            breadth: 0.6,
            apex_curve: 0.9,
        };
        assert!((cor_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_blend_midpoint() {
        /* mid-blend */
        let a = CoronoidMorph {
            height: 0.0,
            breadth: 0.0,
            apex_curve: 0.0,
        };
        let b = CoronoidMorph {
            height: 1.0,
            breadth: 1.0,
            apex_curve: 1.0,
        };
        let c = cor_blend(&a, &b, 0.5);
        assert!((c.height - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_to_json_contains_height() {
        /* JSON serialization includes height */
        let m = new_coronoid_morph();
        assert!(cor_to_json(&m).contains("height"));
    }

    #[test]
    fn test_clone_independent() {
        /* clone is independent */
        let m = CoronoidMorph {
            height: 0.4,
            breadth: 0.5,
            apex_curve: 0.6,
        };
        let m2 = m.clone();
        assert_eq!(m.height, m2.height);
    }
}
