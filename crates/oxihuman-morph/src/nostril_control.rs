//! Nostril flare and nasal morphology controls.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NostrilConfig {
    pub flare_range: f32,
    pub width_range: f32,
    pub height_range: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NostrilState {
    pub flare_l: f32,
    pub flare_r: f32,
    pub width_l: f32,
    pub width_r: f32,
    pub pinch: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NostrilMorphWeights {
    pub flare_l: f32,
    pub flare_r: f32,
    pub wide_l: f32,
    pub wide_r: f32,
    pub pinch: f32,
}

#[allow(dead_code)]
pub fn default_nostril_config() -> NostrilConfig {
    NostrilConfig {
        flare_range: 1.0,
        width_range: 1.0,
        height_range: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_nostril_state() -> NostrilState {
    NostrilState {
        flare_l: 0.0,
        flare_r: 0.0,
        width_l: 0.0,
        width_r: 0.0,
        pinch: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_nostril_flare(state: &mut NostrilState, left: f32, right: f32) {
    state.flare_l = left.clamp(0.0, 1.0);
    state.flare_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_nostril_width(state: &mut NostrilState, left: f32, right: f32) {
    state.width_l = left.clamp(-1.0, 1.0);
    state.width_r = right.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn set_nostril_pinch(state: &mut NostrilState, pinch: f32) {
    state.pinch = pinch.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_nostril_weights(state: &NostrilState, cfg: &NostrilConfig) -> NostrilMorphWeights {
    let fr = cfg.flare_range.max(0.001);
    let wr = cfg.width_range.max(0.001);
    NostrilMorphWeights {
        flare_l: (state.flare_l * fr).clamp(0.0, 1.0),
        flare_r: (state.flare_r * fr).clamp(0.0, 1.0),
        wide_l: (state.width_l * wr).clamp(0.0, 1.0),
        wide_r: (state.width_r * wr).clamp(0.0, 1.0),
        pinch: state.pinch.clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn blend_nostrils(a: &NostrilState, b: &NostrilState, t: f32) -> NostrilState {
    let t = t.clamp(0.0, 1.0);
    let s = 1.0 - t;
    NostrilState {
        flare_l: a.flare_l * s + b.flare_l * t,
        flare_r: a.flare_r * s + b.flare_r * t,
        width_l: a.width_l * s + b.width_l * t,
        width_r: a.width_r * s + b.width_r * t,
        pinch: a.pinch * s + b.pinch * t,
    }
}

#[allow(dead_code)]
pub fn reset_nostrils(state: &mut NostrilState) {
    *state = new_nostril_state();
}

#[allow(dead_code)]
pub fn symmetrize_nostrils(state: &mut NostrilState) {
    let avg_flare = (state.flare_l + state.flare_r) * 0.5;
    let avg_width = (state.width_l + state.width_r) * 0.5;
    state.flare_l = avg_flare;
    state.flare_r = avg_flare;
    state.width_l = avg_width;
    state.width_r = avg_width;
}

#[allow(dead_code)]
pub fn nostril_state_to_json(state: &NostrilState) -> String {
    format!(
        r#"{{"flare_l":{:.4},"flare_r":{:.4},"width_l":{:.4},"width_r":{:.4},"pinch":{:.4}}}"#,
        state.flare_l, state.flare_r, state.width_l, state.width_r, state.pinch
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_nostril_config();
        assert!((cfg.flare_range - 1.0).abs() < 1e-6);
        assert!((cfg.width_range - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeroed() {
        let s = new_nostril_state();
        assert_eq!(s.flare_l, 0.0);
        assert_eq!(s.flare_r, 0.0);
        assert_eq!(s.pinch, 0.0);
    }

    #[test]
    fn test_set_clamping() {
        let mut s = new_nostril_state();
        set_nostril_flare(&mut s, 2.0, -1.0);
        assert!((s.flare_l - 1.0).abs() < 1e-6);
        assert_eq!(s.flare_r, 0.0);
        set_nostril_pinch(&mut s, 5.0);
        assert!((s.pinch - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let cfg = default_nostril_config();
        let mut s = new_nostril_state();
        set_nostril_flare(&mut s, 0.8, 0.4);
        let w = compute_nostril_weights(&s, &cfg);
        assert!((w.flare_l - 0.8).abs() < 1e-5);
        assert!((w.flare_r - 0.4).abs() < 1e-5);
    }

    #[test]
    fn test_blend() {
        let a = new_nostril_state();
        let mut b = new_nostril_state();
        b.flare_l = 1.0;
        let mid = blend_nostrils(&a, &b, 0.5);
        assert!((mid.flare_l - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_symmetrize() {
        let mut s = new_nostril_state();
        s.flare_l = 0.8;
        s.flare_r = 0.4;
        symmetrize_nostrils(&mut s);
        assert!((s.flare_l - 0.6).abs() < 1e-6);
        assert!((s.flare_r - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let s = new_nostril_state();
        let j = nostril_state_to_json(&s);
        assert!(j.contains("\"flare_l\""));
        assert!(j.contains("\"pinch\""));
    }
}
