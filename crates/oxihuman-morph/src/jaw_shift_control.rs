// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Jaw shift morph — controls lateral and anterior/posterior jaw displacement.

/// Configuration for jaw shift control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawShiftConfig {
    pub max_lateral: f32,
    pub max_ap: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawShiftState {
    pub lateral: f32,
    pub anterior: f32,
    pub posterior: f32,
    pub torsion: f32,
}

#[allow(dead_code)]
pub fn default_jaw_shift_config() -> JawShiftConfig {
    JawShiftConfig {
        max_lateral: 1.0,
        max_ap: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_jaw_shift_state() -> JawShiftState {
    JawShiftState {
        lateral: 0.0,
        anterior: 0.0,
        posterior: 0.0,
        torsion: 0.0,
    }
}

#[allow(dead_code)]
pub fn js_set_lateral(state: &mut JawShiftState, cfg: &JawShiftConfig, v: f32) {
    state.lateral = v.clamp(-cfg.max_lateral, cfg.max_lateral);
}

#[allow(dead_code)]
pub fn js_set_anterior(state: &mut JawShiftState, cfg: &JawShiftConfig, v: f32) {
    state.anterior = v.clamp(0.0, cfg.max_ap);
    state.posterior = 0.0;
}

#[allow(dead_code)]
pub fn js_set_posterior(state: &mut JawShiftState, cfg: &JawShiftConfig, v: f32) {
    state.posterior = v.clamp(0.0, cfg.max_ap);
    state.anterior = 0.0;
}

#[allow(dead_code)]
pub fn js_set_torsion(state: &mut JawShiftState, v: f32) {
    state.torsion = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn js_reset(state: &mut JawShiftState) {
    *state = new_jaw_shift_state();
}

#[allow(dead_code)]
pub fn js_is_neutral(state: &JawShiftState) -> bool {
    state.lateral.abs() < 1e-6
        && state.anterior.abs() < 1e-6
        && state.posterior.abs() < 1e-6
        && state.torsion.abs() < 1e-6
}

#[allow(dead_code)]
pub fn js_net_ap(state: &JawShiftState) -> f32 {
    state.anterior - state.posterior
}

#[allow(dead_code)]
pub fn js_displacement_magnitude(state: &JawShiftState) -> f32 {
    let ap = js_net_ap(state);
    (state.lateral * state.lateral + ap * ap).sqrt()
}

#[allow(dead_code)]
pub fn js_blend(a: &JawShiftState, b: &JawShiftState, t: f32) -> JawShiftState {
    let t = t.clamp(0.0, 1.0);
    JawShiftState {
        lateral: a.lateral + (b.lateral - a.lateral) * t,
        anterior: a.anterior + (b.anterior - a.anterior) * t,
        posterior: a.posterior + (b.posterior - a.posterior) * t,
        torsion: a.torsion + (b.torsion - a.torsion) * t,
    }
}

#[allow(dead_code)]
pub fn js_to_weights(state: &JawShiftState) -> Vec<(String, f32)> {
    vec![
        ("jaw_lateral_shift".to_string(), state.lateral),
        ("jaw_anterior".to_string(), state.anterior),
        ("jaw_posterior".to_string(), state.posterior),
        ("jaw_torsion".to_string(), state.torsion),
    ]
}

#[allow(dead_code)]
pub fn js_to_json(state: &JawShiftState) -> String {
    format!(
        r#"{{"lateral":{:.4},"anterior":{:.4},"posterior":{:.4},"torsion":{:.4}}}"#,
        state.lateral, state.anterior, state.posterior, state.torsion
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_jaw_shift_config();
        assert!((cfg.max_lateral - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_jaw_shift_state();
        assert!(js_is_neutral(&s));
    }

    #[test]
    fn set_lateral_signed() {
        let cfg = default_jaw_shift_config();
        let mut s = new_jaw_shift_state();
        js_set_lateral(&mut s, &cfg, -0.5);
        assert!((s.lateral + 0.5).abs() < 1e-6);
    }

    #[test]
    fn set_anterior_clears_posterior() {
        let cfg = default_jaw_shift_config();
        let mut s = new_jaw_shift_state();
        js_set_posterior(&mut s, &cfg, 0.5);
        js_set_anterior(&mut s, &cfg, 0.3);
        assert_eq!(s.posterior, 0.0);
        assert!((s.anterior - 0.3).abs() < 1e-6);
    }

    #[test]
    fn set_posterior_clears_anterior() {
        let cfg = default_jaw_shift_config();
        let mut s = new_jaw_shift_state();
        js_set_anterior(&mut s, &cfg, 0.5);
        js_set_posterior(&mut s, &cfg, 0.4);
        assert_eq!(s.anterior, 0.0);
        assert!((s.posterior - 0.4).abs() < 1e-6);
    }

    #[test]
    fn net_ap_anterior() {
        let cfg = default_jaw_shift_config();
        let mut s = new_jaw_shift_state();
        js_set_anterior(&mut s, &cfg, 0.7);
        assert!((js_net_ap(&s) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_jaw_shift_config();
        let mut s = new_jaw_shift_state();
        js_set_lateral(&mut s, &cfg, 0.5);
        js_reset(&mut s);
        assert!(js_is_neutral(&s));
    }

    #[test]
    fn displacement_magnitude_zero_at_neutral() {
        let s = new_jaw_shift_state();
        assert!(js_displacement_magnitude(&s) < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let a = new_jaw_shift_state();
        let cfg = default_jaw_shift_config();
        let mut b = new_jaw_shift_state();
        js_set_lateral(&mut b, &cfg, 1.0);
        let mid = js_blend(&a, &b, 0.5);
        assert!((mid.lateral - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_jaw_shift_state();
        assert_eq!(js_to_weights(&s).len(), 4);
    }
}
