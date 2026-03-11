// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Facial profile (side-view) morph controls: forehead slope, nasal projection, chin recession.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ProfileType {
    Convex,
    Straight,
    Concave,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FacialProfileConfig {
    pub forehead_range: f32,
    pub nasal_range: f32,
    pub chin_range: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FacialProfileState {
    pub forehead_slope: f32,
    pub nasal_projection: f32,
    pub chin_projection: f32,
    pub lip_projection: f32,
    pub profile_type: ProfileType,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FacialProfileWeights {
    pub forehead_forward: f32,
    pub forehead_receding: f32,
    pub nose_prominent: f32,
    pub nose_flat: f32,
    pub chin_forward: f32,
    pub chin_receding: f32,
}

#[allow(dead_code)]
pub fn default_facial_profile_config() -> FacialProfileConfig {
    FacialProfileConfig {
        forehead_range: 0.5,
        nasal_range: 0.6,
        chin_range: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_facial_profile_state() -> FacialProfileState {
    FacialProfileState {
        forehead_slope: 0.5,
        nasal_projection: 0.5,
        chin_projection: 0.5,
        lip_projection: 0.5,
        profile_type: ProfileType::Straight,
    }
}

#[allow(dead_code)]
pub fn set_forehead_slope(state: &mut FacialProfileState, value: f32) {
    state.forehead_slope = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_nasal_projection(state: &mut FacialProfileState, value: f32) {
    state.nasal_projection = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_chin_projection_fp(state: &mut FacialProfileState, value: f32) {
    state.chin_projection = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn detect_profile_type(state: &FacialProfileState) -> ProfileType {
    let sum = state.forehead_slope + state.nasal_projection + state.chin_projection;
    if sum > 1.8 {
        ProfileType::Convex
    } else if sum < 1.2 {
        ProfileType::Concave
    } else {
        ProfileType::Straight
    }
}

#[allow(dead_code)]
pub fn compute_facial_profile_weights(state: &FacialProfileState, cfg: &FacialProfileConfig) -> FacialProfileWeights {
    let fh = (state.forehead_slope - 0.5) * 2.0 * cfg.forehead_range;
    let ns = (state.nasal_projection - 0.5) * 2.0 * cfg.nasal_range;
    let ch = (state.chin_projection - 0.5) * 2.0 * cfg.chin_range;
    FacialProfileWeights {
        forehead_forward: fh.max(0.0).clamp(0.0, 1.0),
        forehead_receding: (-fh).max(0.0).clamp(0.0, 1.0),
        nose_prominent: ns.max(0.0).clamp(0.0, 1.0),
        nose_flat: (-ns).max(0.0).clamp(0.0, 1.0),
        chin_forward: ch.max(0.0).clamp(0.0, 1.0),
        chin_receding: (-ch).max(0.0).clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn facial_profile_to_json(state: &FacialProfileState) -> String {
    let pt = match &state.profile_type {
        ProfileType::Convex => "convex",
        ProfileType::Straight => "straight",
        ProfileType::Concave => "concave",
    };
    format!(
        r#"{{"forehead":{},"nasal":{},"chin":{},"lip":{},"type":"{}"}}"#,
        state.forehead_slope, state.nasal_projection, state.chin_projection, state.lip_projection, pt
    )
}

#[allow(dead_code)]
pub fn blend_facial_profile_states(a: &FacialProfileState, b: &FacialProfileState, t: f32) -> FacialProfileState {
    let t = t.clamp(0.0, 1.0);
    let mut result = FacialProfileState {
        forehead_slope: a.forehead_slope + (b.forehead_slope - a.forehead_slope) * t,
        nasal_projection: a.nasal_projection + (b.nasal_projection - a.nasal_projection) * t,
        chin_projection: a.chin_projection + (b.chin_projection - a.chin_projection) * t,
        lip_projection: a.lip_projection + (b.lip_projection - a.lip_projection) * t,
        profile_type: ProfileType::Straight,
    };
    result.profile_type = detect_profile_type(&result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_facial_profile_config();
        assert!((0.0..=1.0).contains(&c.forehead_range));
    }

    #[test]
    fn test_new_state() {
        let s = new_facial_profile_state();
        assert_eq!(s.profile_type, ProfileType::Straight);
    }

    #[test]
    fn test_set_forehead() {
        let mut s = new_facial_profile_state();
        set_forehead_slope(&mut s, 0.8);
        assert!((s.forehead_slope - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_detect_convex() {
        let mut s = new_facial_profile_state();
        s.forehead_slope = 0.9;
        s.nasal_projection = 0.9;
        s.chin_projection = 0.9;
        assert_eq!(detect_profile_type(&s), ProfileType::Convex);
    }

    #[test]
    fn test_detect_concave() {
        let mut s = new_facial_profile_state();
        s.forehead_slope = 0.1;
        s.nasal_projection = 0.1;
        s.chin_projection = 0.1;
        assert_eq!(detect_profile_type(&s), ProfileType::Concave);
    }

    #[test]
    fn test_weights_neutral() {
        let s = new_facial_profile_state();
        let cfg = default_facial_profile_config();
        let w = compute_facial_profile_weights(&s, &cfg);
        assert!(w.forehead_forward.abs() < 1e-6);
        assert!(w.forehead_receding.abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let s = new_facial_profile_state();
        let j = facial_profile_to_json(&s);
        assert!(j.contains("straight"));
    }

    #[test]
    fn test_blend() {
        let a = new_facial_profile_state();
        let mut b = new_facial_profile_state();
        b.forehead_slope = 1.0;
        let mid = blend_facial_profile_states(&a, &b, 0.5);
        assert!((mid.forehead_slope - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_nasal() {
        let mut s = new_facial_profile_state();
        set_nasal_projection(&mut s, 0.3);
        assert!((s.nasal_projection - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_chin_clamp() {
        let mut s = new_facial_profile_state();
        set_chin_projection_fp(&mut s, 2.0);
        assert!((s.chin_projection - 1.0).abs() < 1e-6);
    }
}
