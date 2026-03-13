// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Forearm shape morph controls: muscle mass, pronation, wrist taper.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForearmConfig {
    pub muscle_range: f32,
    pub taper_range: f32,
    pub rotation_range: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForearmState {
    pub muscle_mass: f32,
    pub taper: f32,
    pub pronation: f32,
    pub vein_visibility: f32,
    pub length_ratio: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForearmMorphWeights {
    pub muscular: f32,
    pub thin: f32,
    pub tapered: f32,
    pub pronated: f32,
    pub supinated: f32,
}

#[allow(dead_code)]
pub fn default_forearm_config() -> ForearmConfig {
    ForearmConfig {
        muscle_range: 0.8,
        taper_range: 0.5,
        rotation_range: 0.6,
    }
}

#[allow(dead_code)]
pub fn new_forearm_state() -> ForearmState {
    ForearmState {
        muscle_mass: 0.5,
        taper: 0.5,
        pronation: 0.5,
        vein_visibility: 0.0,
        length_ratio: 0.5,
    }
}

#[allow(dead_code)]
pub fn set_forearm_muscle(state: &mut ForearmState, value: f32) {
    state.muscle_mass = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_forearm_taper(state: &mut ForearmState, value: f32) {
    state.taper = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_forearm_pronation(state: &mut ForearmState, value: f32) {
    state.pronation = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_vein_visibility(state: &mut ForearmState, value: f32) {
    state.vein_visibility = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_forearm_weights(state: &ForearmState, cfg: &ForearmConfig) -> ForearmMorphWeights {
    let m = state.muscle_mass * cfg.muscle_range;
    let muscular = m.clamp(0.0, 1.0);
    let thin = (1.0 - m).clamp(0.0, 1.0);
    let tapered = (state.taper * cfg.taper_range).clamp(0.0, 1.0);
    let rot = (state.pronation - 0.5) * 2.0 * cfg.rotation_range;
    let pronated = rot.max(0.0).clamp(0.0, 1.0);
    let supinated = (-rot).max(0.0).clamp(0.0, 1.0);
    ForearmMorphWeights {
        muscular,
        thin,
        tapered,
        pronated,
        supinated,
    }
}

#[allow(dead_code)]
pub fn forearm_to_json(state: &ForearmState) -> String {
    format!(
        r#"{{"muscle_mass":{},"taper":{},"pronation":{},"veins":{},"length":{}}}"#,
        state.muscle_mass, state.taper, state.pronation, state.vein_visibility, state.length_ratio
    )
}

#[allow(dead_code)]
pub fn blend_forearm_states(a: &ForearmState, b: &ForearmState, t: f32) -> ForearmState {
    let t = t.clamp(0.0, 1.0);
    ForearmState {
        muscle_mass: a.muscle_mass + (b.muscle_mass - a.muscle_mass) * t,
        taper: a.taper + (b.taper - a.taper) * t,
        pronation: a.pronation + (b.pronation - a.pronation) * t,
        vein_visibility: a.vein_visibility + (b.vein_visibility - a.vein_visibility) * t,
        length_ratio: a.length_ratio + (b.length_ratio - a.length_ratio) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_forearm_config();
        assert!((0.0..=1.0).contains(&c.muscle_range));
    }

    #[test]
    fn test_new_state() {
        let s = new_forearm_state();
        assert!((s.muscle_mass - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_muscle() {
        let mut s = new_forearm_state();
        set_forearm_muscle(&mut s, 0.9);
        assert!((s.muscle_mass - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_taper() {
        let mut s = new_forearm_state();
        set_forearm_taper(&mut s, 0.3);
        assert!((s.taper - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_pronation_clamp() {
        let mut s = new_forearm_state();
        set_forearm_pronation(&mut s, 5.0);
        assert!((s.pronation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights_range() {
        let s = new_forearm_state();
        let cfg = default_forearm_config();
        let w = compute_forearm_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.muscular));
        assert!((0.0..=1.0).contains(&w.thin));
    }

    #[test]
    fn test_to_json() {
        let s = new_forearm_state();
        let j = forearm_to_json(&s);
        assert!(j.contains("muscle_mass"));
    }

    #[test]
    fn test_blend() {
        let a = new_forearm_state();
        let mut b = new_forearm_state();
        b.muscle_mass = 1.0;
        let mid = blend_forearm_states(&a, &b, 0.5);
        assert!((mid.muscle_mass - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_veins() {
        let mut s = new_forearm_state();
        set_vein_visibility(&mut s, 0.6);
        assert!((s.vein_visibility - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_forearm_state();
        let r = blend_forearm_states(&a, &a, 0.5);
        assert!((r.taper - a.taper).abs() < 1e-6);
    }
}
