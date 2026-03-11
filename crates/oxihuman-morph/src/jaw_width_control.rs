// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Jaw width and mandibular angle morph controls.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawWidthConfig {
    pub width_range: f32,
    pub angle_range: f32,
    pub ramus_range: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawWidthState {
    pub width: f32,
    pub gonial_angle: f32,
    pub ramus_height: f32,
    pub masseter_bulk: f32,
    pub asymmetry: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawWidthMorphWeights {
    pub wide: f32,
    pub narrow: f32,
    pub angular: f32,
    pub rounded: f32,
    pub masseter: f32,
}

#[allow(dead_code)]
pub fn default_jaw_width_config() -> JawWidthConfig {
    JawWidthConfig {
        width_range: 0.7,
        angle_range: 0.5,
        ramus_range: 0.4,
    }
}

#[allow(dead_code)]
pub fn new_jaw_width_state() -> JawWidthState {
    JawWidthState {
        width: 0.5,
        gonial_angle: 0.5,
        ramus_height: 0.5,
        masseter_bulk: 0.0,
        asymmetry: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_jaw_width(state: &mut JawWidthState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_gonial_angle(state: &mut JawWidthState, value: f32) {
    state.gonial_angle = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_ramus_height(state: &mut JawWidthState, value: f32) {
    state.ramus_height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_masseter_bulk(state: &mut JawWidthState, value: f32) {
    state.masseter_bulk = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_jaw_width_weights(state: &JawWidthState, cfg: &JawWidthConfig) -> JawWidthMorphWeights {
    let w = (state.width - 0.5) * 2.0 * cfg.width_range;
    let wide = w.max(0.0).clamp(0.0, 1.0);
    let narrow = (-w).max(0.0).clamp(0.0, 1.0);
    let a = (state.gonial_angle - 0.5) * 2.0 * cfg.angle_range;
    let angular = a.max(0.0).clamp(0.0, 1.0);
    let rounded = (-a).max(0.0).clamp(0.0, 1.0);
    let masseter = (state.masseter_bulk * cfg.ramus_range).clamp(0.0, 1.0);
    JawWidthMorphWeights {
        wide,
        narrow,
        angular,
        rounded,
        masseter,
    }
}

#[allow(dead_code)]
pub fn jaw_width_to_json(state: &JawWidthState) -> String {
    format!(
        r#"{{"width":{},"gonial_angle":{},"ramus_height":{},"masseter":{},"asymmetry":{}}}"#,
        state.width, state.gonial_angle, state.ramus_height, state.masseter_bulk, state.asymmetry
    )
}

#[allow(dead_code)]
pub fn blend_jaw_width_states(a: &JawWidthState, b: &JawWidthState, t: f32) -> JawWidthState {
    let t = t.clamp(0.0, 1.0);
    JawWidthState {
        width: a.width + (b.width - a.width) * t,
        gonial_angle: a.gonial_angle + (b.gonial_angle - a.gonial_angle) * t,
        ramus_height: a.ramus_height + (b.ramus_height - a.ramus_height) * t,
        masseter_bulk: a.masseter_bulk + (b.masseter_bulk - a.masseter_bulk) * t,
        asymmetry: a.asymmetry + (b.asymmetry - a.asymmetry) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_jaw_width_config();
        assert!((0.0..=1.0).contains(&c.width_range));
    }

    #[test]
    fn test_new_state() {
        let s = new_jaw_width_state();
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_jaw_width_state();
        set_jaw_width(&mut s, 0.8);
        assert!((s.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_angle_clamp() {
        let mut s = new_jaw_width_state();
        set_gonial_angle(&mut s, -1.0);
        assert!(s.gonial_angle.abs() < 1e-6);
    }

    #[test]
    fn test_set_ramus() {
        let mut s = new_jaw_width_state();
        set_ramus_height(&mut s, 0.7);
        assert!((s.ramus_height - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_weights_neutral() {
        let s = new_jaw_width_state();
        let cfg = default_jaw_width_config();
        let w = compute_jaw_width_weights(&s, &cfg);
        assert!(w.wide.abs() < 1e-6);
        assert!(w.narrow.abs() < 1e-6);
    }

    #[test]
    fn test_weights_wide() {
        let mut s = new_jaw_width_state();
        s.width = 1.0;
        let cfg = default_jaw_width_config();
        let w = compute_jaw_width_weights(&s, &cfg);
        assert!(w.wide > 0.0);
    }

    #[test]
    fn test_to_json() {
        let s = new_jaw_width_state();
        let j = jaw_width_to_json(&s);
        assert!(j.contains("gonial_angle"));
    }

    #[test]
    fn test_blend() {
        let a = new_jaw_width_state();
        let mut b = new_jaw_width_state();
        b.width = 1.0;
        let mid = blend_jaw_width_states(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_masseter() {
        let mut s = new_jaw_width_state();
        set_masseter_bulk(&mut s, 0.6);
        assert!((s.masseter_bulk - 0.6).abs() < 1e-6);
    }
}
