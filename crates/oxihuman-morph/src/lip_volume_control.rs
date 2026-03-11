// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lip volume morph controls: upper/lower lip fullness, cupid's bow, vermilion border.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipVolumeConfig {
    pub upper_range: f32,
    pub lower_range: f32,
    pub bow_range: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipVolumeState {
    pub upper_volume: f32,
    pub lower_volume: f32,
    pub cupids_bow: f32,
    pub vermilion_width: f32,
    pub philtrum_depth: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipVolumeMorphWeights {
    pub upper_full: f32,
    pub upper_thin: f32,
    pub lower_full: f32,
    pub lower_thin: f32,
    pub bow_defined: f32,
    pub wide_vermilion: f32,
}

#[allow(dead_code)]
pub fn default_lip_volume_config() -> LipVolumeConfig {
    LipVolumeConfig {
        upper_range: 0.7,
        lower_range: 0.7,
        bow_range: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_lip_volume_state() -> LipVolumeState {
    LipVolumeState {
        upper_volume: 0.5,
        lower_volume: 0.5,
        cupids_bow: 0.5,
        vermilion_width: 0.5,
        philtrum_depth: 0.3,
    }
}

#[allow(dead_code)]
pub fn set_upper_volume(state: &mut LipVolumeState, value: f32) {
    state.upper_volume = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_lower_volume(state: &mut LipVolumeState, value: f32) {
    state.lower_volume = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_cupids_bow(state: &mut LipVolumeState, value: f32) {
    state.cupids_bow = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_vermilion_width(state: &mut LipVolumeState, value: f32) {
    state.vermilion_width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_philtrum_depth(state: &mut LipVolumeState, value: f32) {
    state.philtrum_depth = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_lip_volume_weights(state: &LipVolumeState, cfg: &LipVolumeConfig) -> LipVolumeMorphWeights {
    let u = state.upper_volume * cfg.upper_range;
    let upper_full = u.clamp(0.0, 1.0);
    let upper_thin = (1.0 - u).clamp(0.0, 1.0);
    let l = state.lower_volume * cfg.lower_range;
    let lower_full = l.clamp(0.0, 1.0);
    let lower_thin = (1.0 - l).clamp(0.0, 1.0);
    let bow_defined = (state.cupids_bow * cfg.bow_range).clamp(0.0, 1.0);
    let wide_vermilion = (state.vermilion_width * 0.8).clamp(0.0, 1.0);
    LipVolumeMorphWeights {
        upper_full,
        upper_thin,
        lower_full,
        lower_thin,
        bow_defined,
        wide_vermilion,
    }
}

#[allow(dead_code)]
pub fn lip_volume_to_json(state: &LipVolumeState) -> String {
    format!(
        r#"{{"upper":{},"lower":{},"bow":{},"vermilion":{},"philtrum":{}}}"#,
        state.upper_volume, state.lower_volume, state.cupids_bow,
        state.vermilion_width, state.philtrum_depth
    )
}

#[allow(dead_code)]
pub fn blend_lip_volume_states(a: &LipVolumeState, b: &LipVolumeState, t: f32) -> LipVolumeState {
    let t = t.clamp(0.0, 1.0);
    LipVolumeState {
        upper_volume: a.upper_volume + (b.upper_volume - a.upper_volume) * t,
        lower_volume: a.lower_volume + (b.lower_volume - a.lower_volume) * t,
        cupids_bow: a.cupids_bow + (b.cupids_bow - a.cupids_bow) * t,
        vermilion_width: a.vermilion_width + (b.vermilion_width - a.vermilion_width) * t,
        philtrum_depth: a.philtrum_depth + (b.philtrum_depth - a.philtrum_depth) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_lip_volume_config();
        assert!((0.0..=1.0).contains(&c.upper_range));
    }

    #[test]
    fn test_new_state() {
        let s = new_lip_volume_state();
        assert!((s.upper_volume - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_upper() {
        let mut s = new_lip_volume_state();
        set_upper_volume(&mut s, 0.9);
        assert!((s.upper_volume - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_lower_clamp() {
        let mut s = new_lip_volume_state();
        set_lower_volume(&mut s, 5.0);
        assert!((s.lower_volume - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_bow() {
        let mut s = new_lip_volume_state();
        set_cupids_bow(&mut s, 0.8);
        assert!((s.cupids_bow - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_weights_range() {
        let s = new_lip_volume_state();
        let cfg = default_lip_volume_config();
        let w = compute_lip_volume_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.upper_full));
        assert!((0.0..=1.0).contains(&w.lower_full));
    }

    #[test]
    fn test_to_json() {
        let s = new_lip_volume_state();
        let j = lip_volume_to_json(&s);
        assert!(j.contains("upper"));
    }

    #[test]
    fn test_blend() {
        let a = new_lip_volume_state();
        let mut b = new_lip_volume_state();
        b.upper_volume = 1.0;
        let mid = blend_lip_volume_states(&a, &b, 0.5);
        assert!((mid.upper_volume - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_vermilion() {
        let mut s = new_lip_volume_state();
        set_vermilion_width(&mut s, 0.6);
        assert!((s.vermilion_width - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_philtrum() {
        let mut s = new_lip_volume_state();
        set_philtrum_depth(&mut s, 0.7);
        assert!((s.philtrum_depth - 0.7).abs() < 1e-6);
    }
}
