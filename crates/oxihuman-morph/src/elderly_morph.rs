// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Elderly body shape aging morph (60–100+).

/// Configuration for the elderly morph.
#[derive(Debug, Clone)]
pub struct ElderlyMorphConfig {
    pub kyphosis_max: f32,
    pub height_loss_max: f32,
    pub skin_sag_factor: f32,
}

impl Default for ElderlyMorphConfig {
    fn default() -> Self {
        ElderlyMorphConfig {
            kyphosis_max: 0.4,
            height_loss_max: 0.08,
            skin_sag_factor: 0.6,
        }
    }
}

/// State for the elderly body morph.
#[derive(Debug, Clone)]
pub struct ElderlyMorph {
    /// Age in years (60–100).
    pub age_years: f32,
    pub config: ElderlyMorphConfig,
    pub enabled: bool,
}

/// Create a new elderly morph at age 60.
pub fn new_elderly_morph() -> ElderlyMorph {
    ElderlyMorph {
        age_years: 60.0,
        config: ElderlyMorphConfig::default(),
        enabled: true,
    }
}

/// Set age in years (clamped 60–100).
pub fn em_set_age(m: &mut ElderlyMorph, years: f32) {
    m.age_years = years.clamp(60.0, 100.0);
}

/// Normalised progress (0 = 60, 1 = 100).
pub fn em_progress(m: &ElderlyMorph) -> f32 {
    (m.age_years - 60.0) / 40.0
}

/// Spinal curvature (kyphosis) weight.
pub fn em_kyphosis(m: &ElderlyMorph) -> f32 {
    let t = em_progress(m);
    t * t * m.config.kyphosis_max
}

/// Height loss as a fraction of original height.
pub fn em_height_loss(m: &ElderlyMorph) -> f32 {
    em_progress(m) * m.config.height_loss_max
}

/// Skin sag weight [0, 1].
pub fn em_skin_sag(m: &ElderlyMorph) -> f32 {
    em_progress(m) * m.config.skin_sag_factor
}

/// Serialise to JSON.
pub fn em_to_json(m: &ElderlyMorph) -> String {
    format!(
        r#"{{"age_years":{:.1},"kyphosis":{:.3},"height_loss":{:.4},"enabled":{}}}"#,
        m.age_years,
        em_kyphosis(m),
        em_height_loss(m),
        m.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_age_is_60() {
        let m = new_elderly_morph();
        assert!((m.age_years - 60.0).abs() < 1e-6 /* default 60 */);
    }

    #[test]
    fn clamp_to_range() {
        let mut m = new_elderly_morph();
        em_set_age(&mut m, 30.0);
        assert!((m.age_years - 60.0).abs() < 1e-6 /* clamped to 60 */);
        em_set_age(&mut m, 999.0);
        assert!((m.age_years - 100.0).abs() < 1e-6 /* clamped to 100 */);
    }

    #[test]
    fn kyphosis_zero_at_60() {
        let mut m = new_elderly_morph();
        em_set_age(&mut m, 60.0);
        assert!((em_kyphosis(&m) - 0.0).abs() < 1e-6 /* no kyphosis at 60 */);
    }

    #[test]
    fn kyphosis_increases_with_age() {
        let mut m = new_elderly_morph();
        em_set_age(&mut m, 70.0);
        let k70 = em_kyphosis(&m);
        em_set_age(&mut m, 90.0);
        let k90 = em_kyphosis(&m);
        assert!(k90 > k70 /* more kyphosis with age */);
    }

    #[test]
    fn height_loss_at_100() {
        let mut m = new_elderly_morph();
        em_set_age(&mut m, 100.0);
        let loss = em_height_loss(&m);
        assert!(loss > 0.0 /* non-zero height loss */);
    }

    #[test]
    fn skin_sag_in_range() {
        let mut m = new_elderly_morph();
        em_set_age(&mut m, 80.0);
        let s = em_skin_sag(&m);
        assert!((0.0..=1.0).contains(&s) /* valid range */);
    }

    #[test]
    fn json_contains_age() {
        let mut m = new_elderly_morph();
        em_set_age(&mut m, 75.0);
        assert!(em_to_json(&m).contains("75.0") /* age in json */);
    }

    #[test]
    fn enabled_default() {
        let m = new_elderly_morph();
        assert!(m.enabled /* enabled by default */);
    }
}
