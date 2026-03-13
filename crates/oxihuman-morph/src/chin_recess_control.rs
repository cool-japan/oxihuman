// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Chin recess morph — controls how recessed or projected the chin is.

/// Configuration for chin recess control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinRecessConfig {
    pub max_recess: f32,
    pub max_protrusion: f32,
}

/// Runtime state for chin recess morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinRecessState {
    pub recess: f32,
    pub protrusion: f32,
    pub vertical_offset: f32,
}

#[allow(dead_code)]
pub fn default_chin_recess_config() -> ChinRecessConfig {
    ChinRecessConfig {
        max_recess: 1.0,
        max_protrusion: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_chin_recess_state() -> ChinRecessState {
    ChinRecessState {
        recess: 0.0,
        protrusion: 0.0,
        vertical_offset: 0.0,
    }
}

#[allow(dead_code)]
pub fn cr_set_recess(state: &mut ChinRecessState, cfg: &ChinRecessConfig, v: f32) {
    state.recess = v.clamp(0.0, cfg.max_recess);
    state.protrusion = 0.0;
}

#[allow(dead_code)]
pub fn cr_set_protrusion(state: &mut ChinRecessState, cfg: &ChinRecessConfig, v: f32) {
    state.protrusion = v.clamp(0.0, cfg.max_protrusion);
    state.recess = 0.0;
}

#[allow(dead_code)]
pub fn cr_set_vertical(state: &mut ChinRecessState, v: f32) {
    state.vertical_offset = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn cr_reset(state: &mut ChinRecessState) {
    *state = new_chin_recess_state();
}

#[allow(dead_code)]
pub fn cr_is_neutral(state: &ChinRecessState) -> bool {
    state.recess.abs() < 1e-6 && state.protrusion.abs() < 1e-6 && state.vertical_offset.abs() < 1e-6
}

#[allow(dead_code)]
pub fn cr_net_offset(state: &ChinRecessState) -> f32 {
    state.protrusion - state.recess
}

#[allow(dead_code)]
pub fn cr_blend(a: &ChinRecessState, b: &ChinRecessState, t: f32) -> ChinRecessState {
    let t = t.clamp(0.0, 1.0);
    ChinRecessState {
        recess: a.recess + (b.recess - a.recess) * t,
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        vertical_offset: a.vertical_offset + (b.vertical_offset - a.vertical_offset) * t,
    }
}

#[allow(dead_code)]
pub fn cr_to_weights(state: &ChinRecessState) -> Vec<(String, f32)> {
    vec![
        ("chin_recess".to_string(), state.recess),
        ("chin_protrusion".to_string(), state.protrusion),
        ("chin_vertical_offset".to_string(), state.vertical_offset),
    ]
}

#[allow(dead_code)]
pub fn cr_to_json(state: &ChinRecessState) -> String {
    format!(
        r#"{{"recess":{:.4},"protrusion":{:.4},"vertical_offset":{:.4}}}"#,
        state.recess, state.protrusion, state.vertical_offset
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_chin_recess_config();
        assert!((cfg.max_recess - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_chin_recess_state();
        assert!(cr_is_neutral(&s));
    }

    #[test]
    fn set_recess_clears_protrusion() {
        let cfg = default_chin_recess_config();
        let mut s = new_chin_recess_state();
        cr_set_protrusion(&mut s, &cfg, 0.5);
        cr_set_recess(&mut s, &cfg, 0.3);
        assert_eq!(s.protrusion, 0.0);
        assert!((s.recess - 0.3).abs() < 1e-6);
    }

    #[test]
    fn set_protrusion_clears_recess() {
        let cfg = default_chin_recess_config();
        let mut s = new_chin_recess_state();
        cr_set_recess(&mut s, &cfg, 0.3);
        cr_set_protrusion(&mut s, &cfg, 0.7);
        assert_eq!(s.recess, 0.0);
        assert!((s.protrusion - 0.7).abs() < 1e-6);
    }

    #[test]
    fn set_vertical_clamps() {
        let mut s = new_chin_recess_state();
        cr_set_vertical(&mut s, 5.0);
        assert!((s.vertical_offset - 1.0).abs() < 1e-6);
    }

    #[test]
    fn net_offset_positive() {
        let cfg = default_chin_recess_config();
        let mut s = new_chin_recess_state();
        cr_set_protrusion(&mut s, &cfg, 0.8);
        assert!((cr_net_offset(&s) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_chin_recess_config();
        let mut s = new_chin_recess_state();
        cr_set_recess(&mut s, &cfg, 0.5);
        cr_reset(&mut s);
        assert!(cr_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_chin_recess_state();
        let cfg = default_chin_recess_config();
        let mut b = new_chin_recess_state();
        cr_set_recess(&mut b, &cfg, 1.0);
        let mid = cr_blend(&a, &b, 0.5);
        assert!((mid.recess - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_chin_recess_state();
        assert_eq!(cr_to_weights(&s).len(), 3);
    }

    #[test]
    fn to_json_fields() {
        let s = new_chin_recess_state();
        let j = cr_to_json(&s);
        assert!(j.contains("recess"));
        assert!(j.contains("vertical_offset"));
    }
}
