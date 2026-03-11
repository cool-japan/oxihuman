// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Skin fold control — crease/fold morph driven by joint proximity.

/// Named joint site where skin folds form.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldSite {
    ElbowInner,
    ElbowOuter,
    KneeInner,
    KneeOuter,
    ArmPit,
    Groin,
    NeckBase,
    WristInner,
}

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SkinFoldConfig {
    /// Depth scale: multiplied by fold weight to get displacement.
    pub depth_scale: f32,
    /// Width scale for the crease region.
    pub width_scale: f32,
}

impl Default for SkinFoldConfig {
    fn default() -> Self {
        Self {
            depth_scale: 0.005,
            width_scale: 0.012,
        }
    }
}

/// State for all skin fold sites.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct SkinFoldState {
    folds: Vec<(FoldSite, f32)>,
}

#[allow(dead_code)]
pub fn new_skin_fold_state() -> SkinFoldState {
    SkinFoldState::default()
}

#[allow(dead_code)]
pub fn default_skin_fold_config() -> SkinFoldConfig {
    SkinFoldConfig::default()
}

#[allow(dead_code)]
pub fn sf_set(state: &mut SkinFoldState, site: FoldSite, weight: f32) {
    let weight = weight.clamp(0.0, 1.0);
    if let Some(entry) = state.folds.iter_mut().find(|(s, _)| *s == site) {
        entry.1 = weight;
    } else {
        state.folds.push((site, weight));
    }
}

#[allow(dead_code)]
pub fn sf_get(state: &SkinFoldState, site: FoldSite) -> f32 {
    state
        .folds
        .iter()
        .find(|(s, _)| *s == site)
        .map_or(0.0, |(_, w)| *w)
}

#[allow(dead_code)]
pub fn sf_reset(state: &mut SkinFoldState) {
    state.folds.clear();
}

#[allow(dead_code)]
pub fn sf_is_neutral(state: &SkinFoldState) -> bool {
    state.folds.iter().all(|(_, w)| *w < 1e-4)
}

/// Active fold count (weight > threshold).
#[allow(dead_code)]
pub fn sf_active_count(state: &SkinFoldState) -> usize {
    state.folds.iter().filter(|(_, w)| *w > 1e-4).count()
}

/// Depth displacement in metres for a site.
#[allow(dead_code)]
pub fn sf_depth_m(state: &SkinFoldState, site: FoldSite, cfg: &SkinFoldConfig) -> f32 {
    sf_get(state, site) * cfg.depth_scale
}

/// Width of the crease in metres for a site.
#[allow(dead_code)]
pub fn sf_width_m(state: &SkinFoldState, site: FoldSite, cfg: &SkinFoldConfig) -> f32 {
    sf_get(state, site) * cfg.width_scale
}

#[allow(dead_code)]
pub fn sf_blend(a: &SkinFoldState, b: &SkinFoldState, t: f32) -> SkinFoldState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    let mut result = SkinFoldState::default();
    for &(site, wa) in &a.folds {
        let wb = sf_get(b, site);
        result.folds.push((site, wa * inv + wb * t));
    }
    result
}

#[allow(dead_code)]
pub fn sf_to_json(state: &SkinFoldState) -> String {
    format!(
        "{{\"site_count\":{},\"active\":{}}}",
        state.folds.len(),
        sf_active_count(state)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(sf_is_neutral(&new_skin_fold_state()));
    }

    #[test]
    fn set_and_get() {
        let mut s = new_skin_fold_state();
        sf_set(&mut s, FoldSite::ElbowInner, 0.7);
        assert!((sf_get(&s, FoldSite::ElbowInner) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn clamps_high() {
        let mut s = new_skin_fold_state();
        sf_set(&mut s, FoldSite::KneeInner, 5.0);
        assert!((sf_get(&s, FoldSite::KneeInner) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn clamps_low() {
        let mut s = new_skin_fold_state();
        sf_set(&mut s, FoldSite::KneeOuter, -2.0);
        assert!(sf_get(&s, FoldSite::KneeOuter) < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_skin_fold_state();
        sf_set(&mut s, FoldSite::ArmPit, 1.0);
        sf_reset(&mut s);
        assert!(sf_is_neutral(&s));
    }

    #[test]
    fn active_count() {
        let mut s = new_skin_fold_state();
        sf_set(&mut s, FoldSite::Groin, 0.5);
        sf_set(&mut s, FoldSite::NeckBase, 0.0);
        assert_eq!(sf_active_count(&s), 1);
    }

    #[test]
    fn depth_nonzero_when_active() {
        let cfg = default_skin_fold_config();
        let mut s = new_skin_fold_state();
        sf_set(&mut s, FoldSite::WristInner, 1.0);
        assert!(sf_depth_m(&s, FoldSite::WristInner, &cfg) > 0.0);
    }

    #[test]
    fn blend_midpoint() {
        let mut a = new_skin_fold_state();
        sf_set(&mut a, FoldSite::ElbowOuter, 1.0);
        let b = new_skin_fold_state();
        let r = sf_blend(&a, &b, 0.5);
        assert!((sf_get(&r, FoldSite::ElbowOuter) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn update_existing_entry() {
        let mut s = new_skin_fold_state();
        sf_set(&mut s, FoldSite::ElbowInner, 0.3);
        sf_set(&mut s, FoldSite::ElbowInner, 0.9);
        assert_eq!(s.folds.len(), 1);
    }

    #[test]
    fn json_has_keys() {
        let j = sf_to_json(&new_skin_fold_state());
        assert!(j.contains("site_count") && j.contains("active"));
    }
}
