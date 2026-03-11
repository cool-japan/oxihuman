// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Adolescent body proportion morph (ages 12–18).

/// Sex used to pick secondary sexual characteristic curves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdolSex {
    Male,
    Female,
}

/// Configuration for the adolescent morph.
#[derive(Debug, Clone)]
pub struct AdolescentMorphConfig {
    pub sex: AdolSex,
    pub growth_spurt_peak_years: f32,
}

impl Default for AdolescentMorphConfig {
    fn default() -> Self {
        AdolescentMorphConfig {
            sex: AdolSex::Female,
            growth_spurt_peak_years: 14.0,
        }
    }
}

/// State for the adolescent body morph.
#[derive(Debug, Clone)]
pub struct AdolescentMorph {
    /// Age in years (12–18).
    pub age_years: f32,
    pub config: AdolescentMorphConfig,
    pub enabled: bool,
}

/// Create a new adolescent morph at age 12.
pub fn new_adolescent_morph() -> AdolescentMorph {
    AdolescentMorph {
        age_years: 12.0,
        config: AdolescentMorphConfig::default(),
        enabled: true,
    }
}

/// Set age in years (clamped 12–18).
pub fn adol_set_age(m: &mut AdolescentMorph, years: f32) {
    m.age_years = years.clamp(12.0, 18.0);
}

/// Normalised progress (0 = 12, 1 = 18).
pub fn adol_progress(m: &AdolescentMorph) -> f32 {
    (m.age_years - 12.0) / 6.0
}

/// Hip-width delta for the given sex.
pub fn adol_hip_delta(m: &AdolescentMorph) -> f32 {
    let t = adol_progress(m);
    match m.config.sex {
        AdolSex::Female => 0.2 * t,
        AdolSex::Male => 0.05 * t,
    }
}

/// Shoulder-width delta.
pub fn adol_shoulder_delta(m: &AdolescentMorph) -> f32 {
    let t = adol_progress(m);
    match m.config.sex {
        AdolSex::Male => 0.2 * t,
        AdolSex::Female => 0.08 * t,
    }
}

/// Serialise to JSON.
pub fn adol_to_json(m: &AdolescentMorph) -> String {
    format!(
        r#"{{"age_years":{:.1},"enabled":{},"hip_delta":{:.3}}}"#,
        m.age_years,
        m.enabled,
        adol_hip_delta(m)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_age_is_twelve() {
        let m = new_adolescent_morph();
        assert!((m.age_years - 12.0).abs() < 1e-6 /* default 12 */);
    }

    #[test]
    fn clamp_upper() {
        let mut m = new_adolescent_morph();
        adol_set_age(&mut m, 99.0);
        assert!((m.age_years - 18.0).abs() < 1e-6 /* clamped to 18 */);
    }

    #[test]
    fn progress_at_18() {
        let mut m = new_adolescent_morph();
        adol_set_age(&mut m, 18.0);
        assert!((adol_progress(&m) - 1.0).abs() < 1e-6 /* full progress */);
    }

    #[test]
    fn female_hip_delta_larger_than_male() {
        let mut mf = new_adolescent_morph();
        adol_set_age(&mut mf, 18.0);
        let mut mm = new_adolescent_morph();
        mm.config.sex = AdolSex::Male;
        adol_set_age(&mut mm, 18.0);
        assert!(adol_hip_delta(&mf) > adol_hip_delta(&mm) /* female hips wider */);
    }

    #[test]
    fn male_shoulder_delta_larger() {
        let mut mm = new_adolescent_morph();
        mm.config.sex = AdolSex::Male;
        adol_set_age(&mut mm, 18.0);
        let mut mf = new_adolescent_morph();
        adol_set_age(&mut mf, 18.0);
        assert!(adol_shoulder_delta(&mm) > adol_shoulder_delta(&mf) /* male shoulders wider */);
    }

    #[test]
    fn json_contains_age() {
        let mut m = new_adolescent_morph();
        adol_set_age(&mut m, 15.0);
        assert!(adol_to_json(&m).contains("15.0") /* age in json */);
    }

    #[test]
    fn enabled_default_true() {
        let m = new_adolescent_morph();
        assert!(m.enabled /* enabled by default */);
    }

    #[test]
    fn hip_delta_zero_at_start() {
        let mut m = new_adolescent_morph();
        adol_set_age(&mut m, 12.0);
        assert!((adol_hip_delta(&m) - 0.0).abs() < 1e-6 /* no delta at 12 */);
    }
}
