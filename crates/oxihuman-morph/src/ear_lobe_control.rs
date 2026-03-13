// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ear lobe shape and attachment morph controls.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum EarLobeType {
    Free,
    Attached,
    Partial,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarLobeConfig {
    pub max_droop: f32,
    pub size_range: f32,
    pub thickness_range: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarLobeState {
    pub lobe_type: EarLobeType,
    pub droop: f32,
    pub size: f32,
    pub thickness: f32,
    pub left_right_diff: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarLobeMorphWeights {
    pub free_lobe: f32,
    pub attached_lobe: f32,
    pub droopy: f32,
    pub large: f32,
    pub thick: f32,
}

#[allow(dead_code)]
pub fn default_ear_lobe_config() -> EarLobeConfig {
    EarLobeConfig {
        max_droop: 0.8,
        size_range: 0.6,
        thickness_range: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_ear_lobe_state() -> EarLobeState {
    EarLobeState {
        lobe_type: EarLobeType::Free,
        droop: 0.3,
        size: 0.5,
        thickness: 0.5,
        left_right_diff: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_lobe_type(state: &mut EarLobeState, lobe_type: EarLobeType) {
    state.lobe_type = lobe_type;
}

#[allow(dead_code)]
pub fn set_lobe_droop(state: &mut EarLobeState, value: f32) {
    state.droop = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_lobe_size(state: &mut EarLobeState, value: f32) {
    state.size = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_lobe_thickness(state: &mut EarLobeState, value: f32) {
    state.thickness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_ear_lobe_weights(state: &EarLobeState, cfg: &EarLobeConfig) -> EarLobeMorphWeights {
    let (free_lobe, attached_lobe) = match state.lobe_type {
        EarLobeType::Free => (1.0, 0.0),
        EarLobeType::Attached => (0.0, 1.0),
        EarLobeType::Partial => (0.5, 0.5),
    };
    let droopy = (state.droop * cfg.max_droop).clamp(0.0, 1.0);
    let large = (state.size * cfg.size_range).clamp(0.0, 1.0);
    let thick = (state.thickness * cfg.thickness_range).clamp(0.0, 1.0);
    EarLobeMorphWeights {
        free_lobe,
        attached_lobe,
        droopy,
        large,
        thick,
    }
}

#[allow(dead_code)]
pub fn ear_lobe_to_json(state: &EarLobeState) -> String {
    let type_str = match &state.lobe_type {
        EarLobeType::Free => "free",
        EarLobeType::Attached => "attached",
        EarLobeType::Partial => "partial",
    };
    format!(
        r#"{{"type":"{}","droop":{},"size":{},"thickness":{}}}"#,
        type_str, state.droop, state.size, state.thickness
    )
}

#[allow(dead_code)]
pub fn blend_ear_lobe_states(a: &EarLobeState, b: &EarLobeState, t: f32) -> EarLobeState {
    let t = t.clamp(0.0, 1.0);
    EarLobeState {
        lobe_type: if t < 0.5 { a.lobe_type.clone() } else { b.lobe_type.clone() },
        droop: a.droop + (b.droop - a.droop) * t,
        size: a.size + (b.size - a.size) * t,
        thickness: a.thickness + (b.thickness - a.thickness) * t,
        left_right_diff: a.left_right_diff + (b.left_right_diff - a.left_right_diff) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_ear_lobe_config();
        assert!((0.0..=1.0).contains(&c.max_droop));
    }

    #[test]
    fn test_new_state() {
        let s = new_ear_lobe_state();
        assert_eq!(s.lobe_type, EarLobeType::Free);
    }

    #[test]
    fn test_set_type() {
        let mut s = new_ear_lobe_state();
        set_lobe_type(&mut s, EarLobeType::Attached);
        assert_eq!(s.lobe_type, EarLobeType::Attached);
    }

    #[test]
    fn test_set_droop() {
        let mut s = new_ear_lobe_state();
        set_lobe_droop(&mut s, 0.8);
        assert!((s.droop - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_size_clamp() {
        let mut s = new_ear_lobe_state();
        set_lobe_size(&mut s, 2.0);
        assert!((s.size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights_free() {
        let s = new_ear_lobe_state();
        let cfg = default_ear_lobe_config();
        let w = compute_ear_lobe_weights(&s, &cfg);
        assert!((w.free_lobe - 1.0).abs() < 1e-6);
        assert!(w.attached_lobe.abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights_attached() {
        let mut s = new_ear_lobe_state();
        s.lobe_type = EarLobeType::Attached;
        let cfg = default_ear_lobe_config();
        let w = compute_ear_lobe_weights(&s, &cfg);
        assert!(w.free_lobe.abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let s = new_ear_lobe_state();
        let j = ear_lobe_to_json(&s);
        assert!(j.contains("free"));
    }

    #[test]
    fn test_blend() {
        let a = new_ear_lobe_state();
        let mut b = new_ear_lobe_state();
        b.droop = 1.0;
        let mid = blend_ear_lobe_states(&a, &b, 0.5);
        assert!((mid.droop - 0.65).abs() < 1e-6);
    }

    #[test]
    fn test_set_thickness() {
        let mut s = new_ear_lobe_state();
        set_lobe_thickness(&mut s, 0.9);
        assert!((s.thickness - 0.9).abs() < 1e-6);
    }
}
