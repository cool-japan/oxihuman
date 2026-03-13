// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Forehead morphology controls for expressive facial animation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadConfig {
    pub brow_height: f32,
    pub forehead_width: f32,
    pub hairline_height: f32,
    pub supraorbital_ridge: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadState {
    pub brow_raise_l: f32,
    pub brow_raise_r: f32,
    pub furrow: f32,
    pub wrinkle: f32,
    pub width_scale: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadMorphWeights {
    pub raise_l: f32,
    pub raise_r: f32,
    pub furrow: f32,
    pub wrinkle_inner_l: f32,
    pub wrinkle_inner_r: f32,
    pub supraorbital: f32,
}

#[allow(dead_code)]
pub fn default_forehead_config() -> ForeheadConfig {
    ForeheadConfig {
        brow_height: 1.0,
        forehead_width: 1.0,
        hairline_height: 1.0,
        supraorbital_ridge: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_forehead_state() -> ForeheadState {
    ForeheadState {
        brow_raise_l: 0.0,
        brow_raise_r: 0.0,
        furrow: 0.0,
        wrinkle: 0.0,
        width_scale: 1.0,
    }
}

#[allow(dead_code)]
pub fn raise_brow(state: &mut ForeheadState, left: f32, right: f32) {
    state.brow_raise_l = left.clamp(0.0, 1.0);
    state.brow_raise_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_furrow(state: &mut ForeheadState, amount: f32) {
    state.furrow = amount.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_wrinkle(state: &mut ForeheadState, amount: f32) {
    state.wrinkle = amount.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_forehead_weights(
    state: &ForeheadState,
    cfg: &ForeheadConfig,
) -> ForeheadMorphWeights {
    let ridge_factor = cfg.supraorbital_ridge.clamp(0.0, 1.0);
    ForeheadMorphWeights {
        raise_l: state.brow_raise_l * cfg.brow_height,
        raise_r: state.brow_raise_r * cfg.brow_height,
        furrow: state.furrow,
        wrinkle_inner_l: state.wrinkle * (0.5 + 0.5 * state.furrow),
        wrinkle_inner_r: state.wrinkle * (0.5 + 0.5 * state.furrow),
        supraorbital: ridge_factor * (state.brow_raise_l + state.brow_raise_r) * 0.5,
    }
}

#[allow(dead_code)]
pub fn blend_forehead(a: &ForeheadState, b: &ForeheadState, t: f32) -> ForeheadState {
    let t = t.clamp(0.0, 1.0);
    let s = 1.0 - t;
    ForeheadState {
        brow_raise_l: a.brow_raise_l * s + b.brow_raise_l * t,
        brow_raise_r: a.brow_raise_r * s + b.brow_raise_r * t,
        furrow: a.furrow * s + b.furrow * t,
        wrinkle: a.wrinkle * s + b.wrinkle * t,
        width_scale: a.width_scale * s + b.width_scale * t,
    }
}

#[allow(dead_code)]
pub fn reset_forehead(state: &mut ForeheadState) {
    state.brow_raise_l = 0.0;
    state.brow_raise_r = 0.0;
    state.furrow = 0.0;
    state.wrinkle = 0.0;
    state.width_scale = 1.0;
}

#[allow(dead_code)]
pub fn forehead_state_to_json(state: &ForeheadState) -> String {
    format!(
        r#"{{"brow_raise_l":{:.4},"brow_raise_r":{:.4},"furrow":{:.4},"wrinkle":{:.4},"width_scale":{:.4}}}"#,
        state.brow_raise_l,
        state.brow_raise_r,
        state.furrow,
        state.wrinkle,
        state.width_scale,
    )
}

#[allow(dead_code)]
pub fn symmetrize_forehead(state: &mut ForeheadState) {
    let avg = (state.brow_raise_l + state.brow_raise_r) * 0.5;
    state.brow_raise_l = avg;
    state.brow_raise_r = avg;
}

#[allow(dead_code)]
pub fn forehead_raise_avg(state: &ForeheadState) -> f32 {
    (state.brow_raise_l + state.brow_raise_r) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_forehead_config();
        assert!((cfg.brow_height - 1.0).abs() < 1e-5);
        assert!((cfg.supraorbital_ridge - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_new_state_zero() {
        let s = new_forehead_state();
        assert_eq!(s.brow_raise_l, 0.0);
        assert_eq!(s.brow_raise_r, 0.0);
        assert_eq!(s.furrow, 0.0);
        assert_eq!(s.wrinkle, 0.0);
        assert!((s.width_scale - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_raise_brow_clamps() {
        let mut s = new_forehead_state();
        raise_brow(&mut s, 2.0, -1.0);
        assert!((s.brow_raise_l - 1.0).abs() < 1e-5);
        assert_eq!(s.brow_raise_r, 0.0);
    }

    #[test]
    fn test_set_furrow_clamps() {
        let mut s = new_forehead_state();
        set_furrow(&mut s, 1.5);
        assert!((s.furrow - 1.0).abs() < 1e-5);
        set_furrow(&mut s, -0.3);
        assert_eq!(s.furrow, 0.0);
    }

    #[test]
    fn test_compute_weights() {
        let cfg = default_forehead_config();
        let mut s = new_forehead_state();
        raise_brow(&mut s, 0.8, 0.6);
        set_furrow(&mut s, 0.5);
        set_wrinkle(&mut s, 0.4);
        let w = compute_forehead_weights(&s, &cfg);
        assert!((w.raise_l - 0.8).abs() < 1e-5);
        assert!((w.raise_r - 0.6).abs() < 1e-5);
        assert!((w.furrow - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_blend_forehead() {
        let mut a = new_forehead_state();
        let mut b = new_forehead_state();
        a.brow_raise_l = 0.0;
        b.brow_raise_l = 1.0;
        let c = blend_forehead(&a, &b, 0.5);
        assert!((c.brow_raise_l - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_reset_forehead() {
        let mut s = new_forehead_state();
        raise_brow(&mut s, 1.0, 1.0);
        set_furrow(&mut s, 1.0);
        reset_forehead(&mut s);
        assert_eq!(s.brow_raise_l, 0.0);
        assert_eq!(s.furrow, 0.0);
    }

    #[test]
    fn test_forehead_state_to_json() {
        let s = new_forehead_state();
        let j = forehead_state_to_json(&s);
        assert!(j.contains("brow_raise_l"));
        assert!(j.contains("width_scale"));
    }

    #[test]
    fn test_symmetrize_forehead() {
        let mut s = new_forehead_state();
        s.brow_raise_l = 0.2;
        s.brow_raise_r = 0.8;
        symmetrize_forehead(&mut s);
        assert!((s.brow_raise_l - 0.5).abs() < 1e-5);
        assert!((s.brow_raise_r - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_forehead_raise_avg() {
        let mut s = new_forehead_state();
        s.brow_raise_l = 0.4;
        s.brow_raise_r = 0.6;
        let avg = forehead_raise_avg(&s);
        assert!((avg - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_blend_t0_equals_a() {
        let mut a = new_forehead_state();
        a.brow_raise_l = 0.7;
        a.furrow = 0.3;
        let b = new_forehead_state();
        let c = blend_forehead(&a, &b, 0.0);
        assert!((c.brow_raise_l - 0.7).abs() < 1e-5);
        assert!((c.furrow - 0.3).abs() < 1e-5);
    }
}
