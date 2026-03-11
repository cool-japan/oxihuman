// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct BmiMorph {
    pub bmi: f32,
    pub underweight_blend: f32,
    pub overweight_blend: f32,
}

pub fn new_bmi_morph(bmi: f32) -> BmiMorph {
    let clamped = bmi.clamp(10.0, 60.0);
    let underweight_blend = if clamped < 18.5 {
        (18.5 - clamped) / 8.5
    } else {
        0.0
    };
    let overweight_blend = if clamped > 25.0 {
        ((clamped - 25.0) / 35.0).min(1.0)
    } else {
        0.0
    };
    BmiMorph {
        bmi: clamped,
        underweight_blend,
        overweight_blend,
    }
}

pub fn bmi_category(bmi: f32) -> &'static str {
    if bmi < 18.5 {
        "underweight"
    } else if bmi < 25.0 {
        "normal"
    } else if bmi < 30.0 {
        "overweight"
    } else {
        "obese"
    }
}

pub fn bmi_blend_weight(m: &BmiMorph) -> f32 {
    (m.underweight_blend + m.overweight_blend).min(1.0)
}

pub fn bmi_is_healthy(m: &BmiMorph) -> bool {
    m.bmi >= 18.5 && m.bmi <= 25.0
}

pub fn bmi_blend(a: &BmiMorph, b: &BmiMorph, t: f32) -> BmiMorph {
    let t = t.clamp(0.0, 1.0);
    let bmi = a.bmi + (b.bmi - a.bmi) * t;
    new_bmi_morph(bmi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bmi_morph_normal() {
        /* BMI 22 is normal */
        let m = new_bmi_morph(22.0);
        assert!((m.bmi - 22.0).abs() < 1e-5);
        assert!((m.underweight_blend).abs() < 1e-5);
        assert!((m.overweight_blend).abs() < 1e-5);
    }

    #[test]
    fn test_bmi_category() {
        /* category checks */
        assert_eq!(bmi_category(15.0), "underweight");
        assert_eq!(bmi_category(22.0), "normal");
        assert_eq!(bmi_category(27.0), "overweight");
        assert_eq!(bmi_category(35.0), "obese");
    }

    #[test]
    fn test_bmi_blend_weight_normal() {
        /* normal bmi => zero blend */
        let m = new_bmi_morph(22.0);
        assert!((bmi_blend_weight(&m)).abs() < 1e-5);
    }

    #[test]
    fn test_bmi_is_healthy() {
        /* healthy range */
        let m_healthy = new_bmi_morph(22.0);
        let m_under = new_bmi_morph(16.0);
        assert!(bmi_is_healthy(&m_healthy));
        assert!(!bmi_is_healthy(&m_under));
    }

    #[test]
    fn test_bmi_blend() {
        /* blend between two morphs */
        let a = new_bmi_morph(18.0);
        let b = new_bmi_morph(30.0);
        let c = bmi_blend(&a, &b, 0.5);
        assert!((c.bmi - 24.0).abs() < 1e-4);
    }

    #[test]
    fn test_bmi_category_obese() {
        /* obese boundary */
        assert_eq!(bmi_category(30.0), "obese");
    }
}
