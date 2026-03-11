// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Orbital rim / eye socket rim shape morph controls.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OrbitalRimConfig {
    pub max_depth: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OrbitalRimState {
    pub depth_l: f32,
    pub depth_r: f32,
    pub tilt_l: f32,
    pub tilt_r: f32,
}

#[allow(dead_code)]
pub fn default_orbital_rim_config() -> OrbitalRimConfig {
    OrbitalRimConfig { max_depth: 1.0 }
}

#[allow(dead_code)]
pub fn new_orbital_rim_state() -> OrbitalRimState {
    OrbitalRimState {
        depth_l: 0.0,
        depth_r: 0.0,
        tilt_l: 0.0,
        tilt_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn orb_set_depth(state: &mut OrbitalRimState, cfg: &OrbitalRimConfig, left: f32, right: f32) {
    state.depth_l = left.clamp(0.0, cfg.max_depth);
    state.depth_r = right.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn orb_set_tilt(state: &mut OrbitalRimState, left: f32, right: f32) {
    state.tilt_l = left.clamp(-1.0, 1.0);
    state.tilt_r = right.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn orb_mirror(state: &mut OrbitalRimState) {
    let avg_depth = (state.depth_l + state.depth_r) * 0.5;
    let avg_tilt = (state.tilt_l + state.tilt_r) * 0.5;
    state.depth_l = avg_depth;
    state.depth_r = avg_depth;
    state.tilt_l = avg_tilt;
    state.tilt_r = avg_tilt;
}

#[allow(dead_code)]
pub fn orb_reset(state: &mut OrbitalRimState) {
    *state = new_orbital_rim_state();
}

#[allow(dead_code)]
pub fn orb_to_weights(state: &OrbitalRimState) -> [f32; 4] {
    [state.depth_l, state.depth_r, state.tilt_l, state.tilt_r]
}

#[allow(dead_code)]
pub fn orb_to_json(state: &OrbitalRimState) -> String {
    format!(
        r#"{{"depth_l":{:.4},"depth_r":{:.4},"tilt_l":{:.4},"tilt_r":{:.4}}}"#,
        state.depth_l, state.depth_r, state.tilt_l, state.tilt_r
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_orbital_rim_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_orbital_rim_state();
        assert!((s.depth_l - 0.0).abs() < 1e-6);
        assert!((s.tilt_l - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_orbital_rim_config();
        let mut s = new_orbital_rim_state();
        orb_set_depth(&mut s, &cfg, 5.0, -1.0);
        assert!((s.depth_l - 1.0).abs() < 1e-6);
        assert!((s.depth_r - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_tilt_clamps() {
        let mut s = new_orbital_rim_state();
        orb_set_tilt(&mut s, 5.0, -5.0);
        assert!((s.tilt_l - 1.0).abs() < 1e-6);
        assert!((s.tilt_r - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_mirror() {
        let cfg = default_orbital_rim_config();
        let mut s = new_orbital_rim_state();
        orb_set_depth(&mut s, &cfg, 0.2, 0.8);
        orb_mirror(&mut s);
        assert!((s.depth_l - 0.5).abs() < 1e-6);
        assert!((s.depth_r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_orbital_rim_config();
        let mut s = new_orbital_rim_state();
        orb_set_depth(&mut s, &cfg, 0.9, 0.9);
        orb_reset(&mut s);
        assert!((s.depth_l - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights_length() {
        let s = new_orbital_rim_state();
        let w = orb_to_weights(&s);
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn test_to_json() {
        let s = new_orbital_rim_state();
        let j = orb_to_json(&s);
        assert!(j.contains("depth_l"));
        assert!(j.contains("tilt_r"));
    }

    #[test]
    fn test_weights_range_after_set() {
        let cfg = default_orbital_rim_config();
        let mut s = new_orbital_rim_state();
        orb_set_depth(&mut s, &cfg, 0.6, 0.4);
        let w = orb_to_weights(&s);
        assert!(w[0] >= 0.0 && w[0] <= 1.0);
        assert!(w[1] >= 0.0 && w[1] <= 1.0);
    }
}

// ── OrbitalRim (simple blend API) ─────────────────────────────────────────────

/// Simple orbital rim morph parameters (blend API).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct OrbitalRim {
    /// Socket depth, normalised 0..1.
    pub depth: f32,
    /// Socket width, normalised 0..1.
    pub width: f32,
    /// Socket tilt angle, normalised -1..1.
    pub tilt: f32,
    /// Supraorbital ridge prominence, normalised 0..1.
    pub supraorbital_ridge: f32,
}

/// Return a default orbital rim.
#[allow(dead_code)]
pub fn default_orbital_rim() -> OrbitalRim {
    OrbitalRim {
        depth: 0.5,
        width: 0.5,
        tilt: 0.0,
        supraorbital_ridge: 0.3,
    }
}

/// Apply orbital rim to a morph-weight slice.
#[allow(dead_code)]
pub fn apply_orbital_rim(weights: &mut [f32], or_: &OrbitalRim) {
    if !weights.is_empty() {
        weights[0] = or_.depth;
    }
    if weights.len() > 1 {
        weights[1] = or_.width;
    }
    if weights.len() > 2 {
        weights[2] = or_.tilt;
    }
    if weights.len() > 3 {
        weights[3] = or_.supraorbital_ridge;
    }
}

/// Linear blend between two orbital rims.
#[allow(dead_code)]
pub fn orbital_rim_blend(a: &OrbitalRim, b: &OrbitalRim, t: f32) -> OrbitalRim {
    let t = t.clamp(0.0, 1.0);
    OrbitalRim {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        tilt: a.tilt + (b.tilt - a.tilt) * t,
        supraorbital_ridge: a.supraorbital_ridge
            + (b.supraorbital_ridge - a.supraorbital_ridge) * t,
    }
}
