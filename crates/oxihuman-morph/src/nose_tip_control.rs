// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Nose tip shape controls for character customization.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NoseTipConfig {
    pub tip_width: f32,
    pub tip_height: f32,
    pub tip_projection: f32,
    pub alar_width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NoseTipState {
    pub width: f32,
    pub height: f32,
    pub projection: f32,
    pub alar_flare: f32,
    pub tip_angle: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NoseTipWeights {
    pub tip_wide: f32,
    pub tip_narrow: f32,
    pub tip_up: f32,
    pub tip_down: f32,
    pub alar_l: f32,
    pub alar_r: f32,
}

#[allow(dead_code)]
pub fn default_nose_tip_config() -> NoseTipConfig {
    NoseTipConfig {
        tip_width: 1.0,
        tip_height: 1.0,
        tip_projection: 1.0,
        alar_width: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_nose_tip_state() -> NoseTipState {
    NoseTipState {
        width: 0.5,
        height: 0.5,
        projection: 0.5,
        alar_flare: 0.0,
        tip_angle: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_tip_width(state: &mut NoseTipState, width: f32) {
    state.width = width.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_tip_projection(state: &mut NoseTipState, proj: f32) {
    state.projection = proj.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_alar_flare(state: &mut NoseTipState, flare: f32) {
    state.alar_flare = flare.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_tip_angle(state: &mut NoseTipState, angle: f32) {
    // angle in range [-1, 1]: negative = down, positive = up
    state.tip_angle = angle.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_nose_tip_weights(state: &NoseTipState, cfg: &NoseTipConfig) -> NoseTipWeights {
    let wide = (state.width - 0.5).max(0.0) * 2.0 * cfg.tip_width;
    let narrow = (0.5 - state.width).max(0.0) * 2.0 * cfg.tip_width;
    let up_w = state.tip_angle.max(0.0) * cfg.tip_height;
    let down_w = (-state.tip_angle).max(0.0) * cfg.tip_height;
    let alar = state.alar_flare * cfg.alar_width;
    NoseTipWeights {
        tip_wide: wide.clamp(0.0, 1.0),
        tip_narrow: narrow.clamp(0.0, 1.0),
        tip_up: up_w.clamp(0.0, 1.0),
        tip_down: down_w.clamp(0.0, 1.0),
        alar_l: alar.clamp(0.0, 1.0),
        alar_r: alar.clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn blend_nose_tip(a: &NoseTipState, b: &NoseTipState, t: f32) -> NoseTipState {
    let t = t.clamp(0.0, 1.0);
    let s = 1.0 - t;
    NoseTipState {
        width: a.width * s + b.width * t,
        height: a.height * s + b.height * t,
        projection: a.projection * s + b.projection * t,
        alar_flare: a.alar_flare * s + b.alar_flare * t,
        tip_angle: a.tip_angle * s + b.tip_angle * t,
    }
}

#[allow(dead_code)]
pub fn reset_nose_tip(state: &mut NoseTipState) {
    state.width = 0.5;
    state.height = 0.5;
    state.projection = 0.5;
    state.alar_flare = 0.0;
    state.tip_angle = 0.0;
}

#[allow(dead_code)]
pub fn nose_tip_to_json(state: &NoseTipState) -> String {
    format!(
        r#"{{"width":{:.4},"height":{:.4},"projection":{:.4},"alar_flare":{:.4},"tip_angle":{:.4}}}"#,
        state.width,
        state.height,
        state.projection,
        state.alar_flare,
        state.tip_angle,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_nose_tip_config();
        assert!((cfg.tip_width - 1.0).abs() < 1e-5);
        assert!((cfg.alar_width - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_new_state_defaults() {
        let s = new_nose_tip_state();
        assert!((s.width - 0.5).abs() < 1e-5);
        assert!((s.projection - 0.5).abs() < 1e-5);
        assert_eq!(s.alar_flare, 0.0);
        assert_eq!(s.tip_angle, 0.0);
    }

    #[test]
    fn test_set_tip_width_clamps() {
        let mut s = new_nose_tip_state();
        set_tip_width(&mut s, 2.0);
        assert!((s.width - 1.0).abs() < 1e-5);
        set_tip_width(&mut s, -0.5);
        assert_eq!(s.width, 0.0);
    }

    #[test]
    fn test_set_tip_angle_clamps() {
        let mut s = new_nose_tip_state();
        set_tip_angle(&mut s, 5.0);
        assert!((s.tip_angle - 1.0).abs() < 1e-5);
        set_tip_angle(&mut s, -5.0);
        assert!((s.tip_angle - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn test_compute_weights_wide() {
        let cfg = default_nose_tip_config();
        let mut s = new_nose_tip_state();
        set_tip_width(&mut s, 1.0); // fully wide
        let w = compute_nose_tip_weights(&s, &cfg);
        assert!(w.tip_wide > 0.0);
        assert_eq!(w.tip_narrow, 0.0);
    }

    #[test]
    fn test_compute_weights_tip_up() {
        let cfg = default_nose_tip_config();
        let mut s = new_nose_tip_state();
        set_tip_angle(&mut s, 1.0);
        let w = compute_nose_tip_weights(&s, &cfg);
        assert!(w.tip_up > 0.0);
        assert_eq!(w.tip_down, 0.0);
    }

    #[test]
    fn test_blend_nose_tip_midpoint() {
        let a = new_nose_tip_state();
        let mut b = new_nose_tip_state();
        b.width = 1.0;
        let c = blend_nose_tip(&a, &b, 0.5);
        assert!((c.width - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_reset_nose_tip() {
        let mut s = new_nose_tip_state();
        set_tip_width(&mut s, 1.0);
        set_alar_flare(&mut s, 0.9);
        reset_nose_tip(&mut s);
        assert!((s.width - 0.5).abs() < 1e-5);
        assert_eq!(s.alar_flare, 0.0);
    }

    #[test]
    fn test_nose_tip_to_json() {
        let s = new_nose_tip_state();
        let j = nose_tip_to_json(&s);
        assert!(j.contains("\"width\""));
        assert!(j.contains("\"alar_flare\""));
        assert!(j.contains("\"tip_angle\""));
    }

    #[test]
    fn test_alar_weights_symmetric() {
        let cfg = default_nose_tip_config();
        let mut s = new_nose_tip_state();
        set_alar_flare(&mut s, 0.7);
        let w = compute_nose_tip_weights(&s, &cfg);
        assert!((w.alar_l - w.alar_r).abs() < 1e-5);
    }

    #[test]
    fn test_blend_t1_equals_b() {
        let a = new_nose_tip_state();
        let mut b = new_nose_tip_state();
        b.projection = 0.9;
        let c = blend_nose_tip(&a, &b, 1.0);
        assert!((c.projection - 0.9).abs() < 1e-5);
    }
}
