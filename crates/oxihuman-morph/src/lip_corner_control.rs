//! Lip corner shape controls for smile, frown, and expression morphology.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipCornerConfig {
    pub corner_width: f32,
    pub vertical_range: f32,
    pub horizontal_range: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipCornerState {
    pub pull_l: f32,
    pub pull_r: f32,
    pub depress_l: f32,
    pub depress_r: f32,
    pub stretch_l: f32,
    pub stretch_r: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipCornerWeights {
    pub smile_l: f32,
    pub smile_r: f32,
    pub frown_l: f32,
    pub frown_r: f32,
    pub stretch_l: f32,
    pub stretch_r: f32,
}

#[allow(dead_code)]
pub fn default_lip_corner_config() -> LipCornerConfig {
    LipCornerConfig {
        corner_width: 1.0,
        vertical_range: 1.0,
        horizontal_range: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_lip_corner_state() -> LipCornerState {
    LipCornerState {
        pull_l: 0.0,
        pull_r: 0.0,
        depress_l: 0.0,
        depress_r: 0.0,
        stretch_l: 0.0,
        stretch_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_corner_pull(state: &mut LipCornerState, left: f32, right: f32) {
    state.pull_l = left.clamp(0.0, 1.0);
    state.pull_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_corner_depress(state: &mut LipCornerState, left: f32, right: f32) {
    state.depress_l = left.clamp(0.0, 1.0);
    state.depress_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_corner_stretch(state: &mut LipCornerState, left: f32, right: f32) {
    state.stretch_l = left.clamp(0.0, 1.0);
    state.stretch_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_lip_corner_weights(state: &LipCornerState, cfg: &LipCornerConfig) -> LipCornerWeights {
    let scale = cfg.corner_width * cfg.horizontal_range;
    LipCornerWeights {
        smile_l: (state.pull_l * scale).clamp(0.0, 1.0),
        smile_r: (state.pull_r * scale).clamp(0.0, 1.0),
        frown_l: (state.depress_l * cfg.vertical_range).clamp(0.0, 1.0),
        frown_r: (state.depress_r * cfg.vertical_range).clamp(0.0, 1.0),
        stretch_l: (state.stretch_l * scale).clamp(0.0, 1.0),
        stretch_r: (state.stretch_r * scale).clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn blend_lip_corners(a: &LipCornerState, b: &LipCornerState, t: f32) -> LipCornerState {
    let t = t.clamp(0.0, 1.0);
    let s = 1.0 - t;
    LipCornerState {
        pull_l: a.pull_l * s + b.pull_l * t,
        pull_r: a.pull_r * s + b.pull_r * t,
        depress_l: a.depress_l * s + b.depress_l * t,
        depress_r: a.depress_r * s + b.depress_r * t,
        stretch_l: a.stretch_l * s + b.stretch_l * t,
        stretch_r: a.stretch_r * s + b.stretch_r * t,
    }
}

#[allow(dead_code)]
pub fn reset_lip_corners(state: &mut LipCornerState) {
    state.pull_l = 0.0;
    state.pull_r = 0.0;
    state.depress_l = 0.0;
    state.depress_r = 0.0;
    state.stretch_l = 0.0;
    state.stretch_r = 0.0;
}

#[allow(dead_code)]
pub fn symmetrize_lip_corners(state: &mut LipCornerState) {
    let pull_avg = (state.pull_l + state.pull_r) * 0.5;
    state.pull_l = pull_avg;
    state.pull_r = pull_avg;
    let depress_avg = (state.depress_l + state.depress_r) * 0.5;
    state.depress_l = depress_avg;
    state.depress_r = depress_avg;
    let stretch_avg = (state.stretch_l + state.stretch_r) * 0.5;
    state.stretch_l = stretch_avg;
    state.stretch_r = stretch_avg;
}

#[allow(dead_code)]
pub fn lip_corner_to_json(state: &LipCornerState) -> String {
    format!(
        "{{\"pull_l\":{},\"pull_r\":{},\"depress_l\":{},\"depress_r\":{},\"stretch_l\":{},\"stretch_r\":{}}}",
        state.pull_l, state.pull_r,
        state.depress_l, state.depress_r,
        state.stretch_l, state.stretch_r
    )
}

#[allow(dead_code)]
pub fn smile_intensity(state: &LipCornerState) -> f32 {
    (state.pull_l + state.pull_r) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_lip_corner_config();
        assert!((cfg.corner_width - 1.0).abs() < 1e-6);
        assert!((cfg.vertical_range - 1.0).abs() < 1e-6);
        assert!((cfg.horizontal_range - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_lip_corner_state();
        assert_eq!(s.pull_l, 0.0);
        assert_eq!(s.pull_r, 0.0);
        assert_eq!(s.depress_l, 0.0);
        assert_eq!(s.depress_r, 0.0);
        assert_eq!(s.stretch_l, 0.0);
        assert_eq!(s.stretch_r, 0.0);
    }

    #[test]
    fn test_set_corner_pull_clamped() {
        let mut s = new_lip_corner_state();
        set_corner_pull(&mut s, 2.0, -0.5);
        assert!((s.pull_l - 1.0).abs() < 1e-6);
        assert_eq!(s.pull_r, 0.0);
    }

    #[test]
    fn test_smile_intensity() {
        let mut s = new_lip_corner_state();
        set_corner_pull(&mut s, 0.6, 0.4);
        let intensity = smile_intensity(&s);
        assert!((intensity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_lip_corners() {
        let a = new_lip_corner_state();
        let mut b = new_lip_corner_state();
        set_corner_pull(&mut b, 1.0, 1.0);
        let blended = blend_lip_corners(&a, &b, 0.5);
        assert!((blended.pull_l - 0.5).abs() < 1e-6);
        assert!((blended.pull_r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_symmetrize() {
        let mut s = new_lip_corner_state();
        s.pull_l = 0.8;
        s.pull_r = 0.2;
        symmetrize_lip_corners(&mut s);
        assert!((s.pull_l - 0.5).abs() < 1e-6);
        assert!((s.pull_r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut s = new_lip_corner_state();
        set_corner_pull(&mut s, 1.0, 1.0);
        set_corner_depress(&mut s, 0.5, 0.5);
        reset_lip_corners(&mut s);
        assert_eq!(s.pull_l, 0.0);
        assert_eq!(s.depress_l, 0.0);
    }

    #[test]
    fn test_compute_weights() {
        let mut s = new_lip_corner_state();
        set_corner_pull(&mut s, 1.0, 1.0);
        set_corner_depress(&mut s, 0.5, 0.5);
        let cfg = default_lip_corner_config();
        let w = compute_lip_corner_weights(&s, &cfg);
        assert!((w.smile_l - 1.0).abs() < 1e-6);
        assert!((w.frown_l - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let s = new_lip_corner_state();
        let json = lip_corner_to_json(&s);
        assert!(json.contains("pull_l"));
        assert!(json.contains("stretch_r"));
    }
}
