// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Child body proportion morph (ages 2–12).

/// Configuration for the child morph.
#[derive(Debug, Clone)]
pub struct ChildMorphConfig {
    pub limb_elongation: f32,
    pub torso_narrowness: f32,
    pub head_ratio: f32,
}

impl Default for ChildMorphConfig {
    fn default() -> Self {
        ChildMorphConfig {
            limb_elongation: 0.7,
            torso_narrowness: 0.85,
            head_ratio: 0.85,
        }
    }
}

/// State for the child body morph.
#[derive(Debug, Clone)]
pub struct ChildMorph {
    /// Age in years (2–12).
    pub age_years: f32,
    pub config: ChildMorphConfig,
    pub enabled: bool,
}

/// Create a new child morph at age 2.
pub fn new_child_morph() -> ChildMorph {
    ChildMorph {
        age_years: 2.0,
        config: ChildMorphConfig::default(),
        enabled: true,
    }
}

/// Set age in years (clamped to 2–12).
pub fn cm_set_age(m: &mut ChildMorph, years: f32) {
    m.age_years = years.clamp(2.0, 12.0);
}

/// Normalised progress (0 = age 2, 1 = age 12).
pub fn cm_progress(m: &ChildMorph) -> f32 {
    (m.age_years - 2.0) / 10.0
}

/// Body height scale factor relative to adult norm.
pub fn cm_height_scale(m: &ChildMorph) -> f32 {
    0.45 + 0.35 * cm_progress(m)
}

/// Limb length scale.
pub fn cm_limb_scale(m: &ChildMorph) -> f32 {
    m.config.limb_elongation + 0.15 * cm_progress(m)
}

/// Serialise to JSON.
pub fn cm_to_json(m: &ChildMorph) -> String {
    format!(
        r#"{{"age_years":{:.1},"height_scale":{:.3},"enabled":{}}}"#,
        m.age_years,
        cm_height_scale(m),
        m.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_age_is_two() {
        let m = new_child_morph();
        assert!((m.age_years - 2.0).abs() < 1e-6 /* default age 2 */);
    }

    #[test]
    fn set_age_clamps_bounds() {
        let mut m = new_child_morph();
        cm_set_age(&mut m, 0.0);
        assert!((m.age_years - 2.0).abs() < 1e-6 /* clamped to 2 */);
        cm_set_age(&mut m, 99.0);
        assert!((m.age_years - 12.0).abs() < 1e-6 /* clamped to 12 */);
    }

    #[test]
    fn progress_at_age_12_is_one() {
        let mut m = new_child_morph();
        cm_set_age(&mut m, 12.0);
        assert!((cm_progress(&m) - 1.0).abs() < 1e-6 /* full progress */);
    }

    #[test]
    fn height_increases_with_age() {
        let mut m = new_child_morph();
        cm_set_age(&mut m, 2.0);
        let h2 = cm_height_scale(&m);
        cm_set_age(&mut m, 12.0);
        let h12 = cm_height_scale(&m);
        assert!(h12 > h2 /* taller at 12 */);
    }

    #[test]
    fn limb_scale_increases() {
        let mut m = new_child_morph();
        cm_set_age(&mut m, 2.0);
        let l2 = cm_limb_scale(&m);
        cm_set_age(&mut m, 12.0);
        let l12 = cm_limb_scale(&m);
        assert!(l12 > l2 /* longer limbs at 12 */);
    }

    #[test]
    fn json_contains_age() {
        let mut m = new_child_morph();
        cm_set_age(&mut m, 8.0);
        assert!(cm_to_json(&m).contains("8.0") /* age in json */);
    }

    #[test]
    fn enabled_flag_works() {
        let mut m = new_child_morph();
        m.enabled = false;
        assert!(!m.enabled /* disabled */);
    }

    #[test]
    fn torso_narrowness_positive() {
        let m = new_child_morph();
        assert!(m.config.torso_narrowness > 0.0 /* valid config */);
    }
}
