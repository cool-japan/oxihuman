// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Pregnancy body shape progression morph.

/// Trimester stage of pregnancy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trimester {
    First,
    Second,
    Third,
    PostPartum,
}

/// Configuration for the pregnancy morph.
#[derive(Debug, Clone)]
pub struct PregnancyMorphConfig {
    pub max_belly_scale: f32,
    pub breast_scale_factor: f32,
    pub hip_expansion: f32,
}

impl Default for PregnancyMorphConfig {
    fn default() -> Self {
        PregnancyMorphConfig {
            max_belly_scale: 1.0,
            breast_scale_factor: 0.4,
            hip_expansion: 0.15,
        }
    }
}

/// State for the pregnancy body morph.
#[derive(Debug, Clone)]
pub struct PregnancyMorph {
    pub weeks: f32,
    pub config: PregnancyMorphConfig,
    pub enabled: bool,
}

/// Create a new pregnancy morph at zero weeks.
pub fn new_pregnancy_morph() -> PregnancyMorph {
    PregnancyMorph {
        weeks: 0.0,
        config: PregnancyMorphConfig::default(),
        enabled: true,
    }
}

/// Set gestational age in weeks (0–42).
pub fn pm_set_weeks(m: &mut PregnancyMorph, weeks: f32) {
    m.weeks = weeks.clamp(0.0, 42.0);
}

/// Return the current trimester based on weeks.
pub fn pm_trimester(m: &PregnancyMorph) -> Trimester {
    if m.weeks < 13.0 {
        Trimester::First
    } else if m.weeks < 27.0 {
        Trimester::Second
    } else if m.weeks <= 42.0 {
        Trimester::Third
    } else {
        Trimester::PostPartum
    }
}

/// Normalised belly weight in [0, 1].
pub fn pm_belly_weight(m: &PregnancyMorph) -> f32 {
    let t = (m.weeks / 40.0).clamp(0.0, 1.0);
    t * t * m.config.max_belly_scale
}

/// Breast volume scale (additive delta on top of base).
pub fn pm_breast_delta(m: &PregnancyMorph) -> f32 {
    let t = (m.weeks / 40.0).clamp(0.0, 1.0);
    t * m.config.breast_scale_factor
}

/// Serialise state to a simple JSON string.
pub fn pm_to_json(m: &PregnancyMorph) -> String {
    format!(
        r#"{{"weeks":{:.1},"enabled":{},"belly_weight":{:.3}}}"#,
        m.weeks,
        m.enabled,
        pm_belly_weight(m)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_zero_weeks() {
        let m = new_pregnancy_morph();
        assert!((m.weeks - 0.0).abs() < 1e-6 /* zero weeks */);
    }

    #[test]
    fn set_weeks_clamps() {
        let mut m = new_pregnancy_morph();
        pm_set_weeks(&mut m, 999.0);
        assert!((m.weeks - 42.0).abs() < 1e-6 /* clamped to max */);
        pm_set_weeks(&mut m, -5.0);
        assert!((m.weeks - 0.0).abs() < 1e-6 /* clamped to min */);
    }

    #[test]
    fn trimester_first() {
        let mut m = new_pregnancy_morph();
        pm_set_weeks(&mut m, 8.0);
        assert_eq!(
            pm_trimester(&m),
            Trimester::First /* 8 weeks = first */
        );
    }

    #[test]
    fn trimester_second() {
        let mut m = new_pregnancy_morph();
        pm_set_weeks(&mut m, 20.0);
        assert_eq!(
            pm_trimester(&m),
            Trimester::Second /* 20 weeks = second */
        );
    }

    #[test]
    fn trimester_third() {
        let mut m = new_pregnancy_morph();
        pm_set_weeks(&mut m, 35.0);
        assert_eq!(
            pm_trimester(&m),
            Trimester::Third /* 35 weeks = third */
        );
    }

    #[test]
    fn belly_weight_increases_with_weeks() {
        let mut m = new_pregnancy_morph();
        pm_set_weeks(&mut m, 10.0);
        let w10 = pm_belly_weight(&m);
        pm_set_weeks(&mut m, 30.0);
        let w30 = pm_belly_weight(&m);
        assert!(w30 > w10 /* belly grows over time */);
    }

    #[test]
    fn breast_delta_increases_with_weeks() {
        let mut m = new_pregnancy_morph();
        pm_set_weeks(&mut m, 5.0);
        let d5 = pm_breast_delta(&m);
        pm_set_weeks(&mut m, 38.0);
        let d38 = pm_breast_delta(&m);
        assert!(d38 > d5 /* breast grows over time */);
    }

    #[test]
    fn to_json_contains_weeks() {
        let mut m = new_pregnancy_morph();
        pm_set_weeks(&mut m, 20.0);
        let j = pm_to_json(&m);
        assert!(j.contains("20.0") /* JSON includes weeks */);
    }

    #[test]
    fn enabled_flag() {
        let mut m = new_pregnancy_morph();
        m.enabled = false;
        assert!(!m.enabled /* disabled morph */);
    }
}
