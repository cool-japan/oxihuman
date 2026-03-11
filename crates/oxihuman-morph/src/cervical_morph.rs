// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Cervical spine curvature morph (lordosis / kyphosis balance).
#[derive(Debug, Clone)]
pub struct CervicalMorph {
    /// Lordotic curve depth (0.0 = straight, 1.0 = deep lordosis).
    pub lordosis: f32,
    /// Forward head posture offset (0.0 = neutral, 1.0 = maximum forward).
    pub forward_head: f32,
    /// Lateral list (-1.0 = left, 0.0 = neutral, 1.0 = right).
    pub lateral_list: f32,
}

pub fn new_cervical_morph() -> CervicalMorph {
    CervicalMorph {
        lordosis: 0.0,
        forward_head: 0.0,
        lateral_list: 0.0,
    }
}

pub fn cerv_set_lordosis(m: &mut CervicalMorph, v: f32) {
    m.lordosis = v.clamp(0.0, 1.0);
}

pub fn cerv_set_forward_head(m: &mut CervicalMorph, v: f32) {
    m.forward_head = v.clamp(0.0, 1.0);
}

pub fn cerv_set_lateral_list(m: &mut CervicalMorph, v: f32) {
    m.lateral_list = v.clamp(-1.0, 1.0);
}

pub fn cerv_overall_weight(m: &CervicalMorph) -> f32 {
    (m.lordosis + m.forward_head + m.lateral_list.abs()) / 3.0
}

pub fn cerv_blend(a: &CervicalMorph, b: &CervicalMorph, t: f32) -> CervicalMorph {
    let t = t.clamp(0.0, 1.0);
    CervicalMorph {
        lordosis: a.lordosis + (b.lordosis - a.lordosis) * t,
        forward_head: a.forward_head + (b.forward_head - a.forward_head) * t,
        lateral_list: a.lateral_list + (b.lateral_list - a.lateral_list) * t,
    }
}

pub fn cerv_is_neutral(m: &CervicalMorph) -> bool {
    m.lordosis < 1e-5 && m.forward_head < 1e-5 && m.lateral_list.abs() < 1e-5
}

pub fn cerv_to_json(m: &CervicalMorph) -> String {
    format!(
        r#"{{"lordosis":{:.4},"forward_head":{:.4},"lateral_list":{:.4}}}"#,
        m.lordosis, m.forward_head, m.lateral_list
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* all zero */
        let m = new_cervical_morph();
        assert_eq!(m.lordosis, 0.0);
    }

    #[test]
    fn test_set_lordosis() {
        /* valid value */
        let mut m = new_cervical_morph();
        cerv_set_lordosis(&mut m, 0.6);
        assert!((m.lordosis - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_high() {
        /* clamp */
        let mut m = new_cervical_morph();
        cerv_set_lordosis(&mut m, 2.0);
        assert_eq!(m.lordosis, 1.0);
    }

    #[test]
    fn test_lateral_clamp() {
        /* lateral list clamped */
        let mut m = new_cervical_morph();
        cerv_set_lateral_list(&mut m, 5.0);
        assert_eq!(m.lateral_list, 1.0);
    }

    #[test]
    fn test_neutral_true() {
        /* default neutral */
        assert!(cerv_is_neutral(&new_cervical_morph()));
    }

    #[test]
    fn test_neutral_false() {
        /* non-zero breaks neutral */
        let mut m = new_cervical_morph();
        cerv_set_lordosis(&mut m, 0.5);
        assert!(!cerv_is_neutral(&m));
    }

    #[test]
    fn test_weight_formula() {
        /* symmetric formula */
        let m = CervicalMorph {
            lordosis: 0.3,
            forward_head: 0.3,
            lateral_list: 0.3,
        };
        assert!((cerv_overall_weight(&m) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = new_cervical_morph();
        let b = CervicalMorph {
            lordosis: 1.0,
            forward_head: 0.0,
            lateral_list: 0.0,
        };
        let c = cerv_blend(&a, &b, 0.5);
        assert!((c.lordosis - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        /* JSON has lordosis */
        assert!(cerv_to_json(&new_cervical_morph()).contains("lordosis"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = CervicalMorph {
            lordosis: 0.4,
            forward_head: 0.5,
            lateral_list: -0.2,
        };
        let m2 = m.clone();
        assert_eq!(m.forward_head, m2.forward_head);
    }
}
