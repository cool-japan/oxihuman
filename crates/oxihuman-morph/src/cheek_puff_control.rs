//! Cheek puff/hollow morph control for facial expressions.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekPuffConfig {
    pub max_puff: f32,
    pub max_hollow: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekPuffState {
    pub config: CheekPuffConfig,
    pub left_puff: f32,
    pub right_puff: f32,
    pub left_hollow: f32,
    pub right_hollow: f32,
}

#[allow(dead_code)]
pub fn default_cheek_puff_config() -> CheekPuffConfig {
    CheekPuffConfig {
        max_puff: 1.0,
        max_hollow: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_cheek_puff_state(cfg: &CheekPuffConfig) -> CheekPuffState {
    CheekPuffState {
        config: cfg.clone(),
        left_puff: 0.0,
        right_puff: 0.0,
        left_hollow: 0.0,
        right_hollow: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_left_puff(state: &mut CheekPuffState, amount: f32) {
    state.left_puff = amount.clamp(0.0, state.config.max_puff);
}

#[allow(dead_code)]
pub fn set_right_puff(state: &mut CheekPuffState, amount: f32) {
    state.right_puff = amount.clamp(0.0, state.config.max_puff);
}

#[allow(dead_code)]
pub fn set_left_hollow(state: &mut CheekPuffState, amount: f32) {
    state.left_hollow = amount.clamp(0.0, state.config.max_hollow);
}

#[allow(dead_code)]
pub fn set_right_hollow(state: &mut CheekPuffState, amount: f32) {
    state.right_hollow = amount.clamp(0.0, state.config.max_hollow);
}

/// Returns `[left_puff, right_puff, left_hollow, right_hollow]`.
#[allow(dead_code)]
pub fn cheek_morph_weights(state: &CheekPuffState) -> [f32; 4] {
    [
        state.left_puff,
        state.right_puff,
        state.left_hollow,
        state.right_hollow,
    ]
}

#[allow(dead_code)]
pub fn reset_cheek_puff(state: &mut CheekPuffState) {
    state.left_puff = 0.0;
    state.right_puff = 0.0;
    state.left_hollow = 0.0;
    state.right_hollow = 0.0;
}

#[allow(dead_code)]
pub fn blend_cheek_states(a: &CheekPuffState, b: &CheekPuffState, t: f32) -> CheekPuffState {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    CheekPuffState {
        config: a.config.clone(),
        left_puff: lerp(a.left_puff, b.left_puff),
        right_puff: lerp(a.right_puff, b.right_puff),
        left_hollow: lerp(a.left_hollow, b.left_hollow),
        right_hollow: lerp(a.right_hollow, b.right_hollow),
    }
}

#[allow(dead_code)]
pub fn cheeks_are_neutral(state: &CheekPuffState) -> bool {
    state.left_puff == 0.0
        && state.right_puff == 0.0
        && state.left_hollow == 0.0
        && state.right_hollow == 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_cheek_puff_config();
        assert_eq!(cfg.max_puff, 1.0);
        assert_eq!(cfg.max_hollow, 1.0);
    }

    #[test]
    fn test_new_state_neutral() {
        let cfg = default_cheek_puff_config();
        let state = new_cheek_puff_state(&cfg);
        assert!(cheeks_are_neutral(&state));
    }

    #[test]
    fn test_set_left_puff_clamps() {
        let cfg = default_cheek_puff_config();
        let mut state = new_cheek_puff_state(&cfg);
        set_left_puff(&mut state, 2.0);
        assert_eq!(state.left_puff, 1.0);
        set_left_puff(&mut state, -0.5);
        assert_eq!(state.left_puff, 0.0);
    }

    #[test]
    fn test_set_right_hollow_clamps() {
        let cfg = default_cheek_puff_config();
        let mut state = new_cheek_puff_state(&cfg);
        set_right_hollow(&mut state, 0.7);
        assert!((state.right_hollow - 0.7).abs() < 1e-6);
        set_right_hollow(&mut state, 5.0);
        assert_eq!(state.right_hollow, 1.0);
    }

    #[test]
    fn test_cheek_morph_weights() {
        let cfg = default_cheek_puff_config();
        let mut state = new_cheek_puff_state(&cfg);
        set_left_puff(&mut state, 0.3);
        set_right_puff(&mut state, 0.6);
        set_left_hollow(&mut state, 0.1);
        set_right_hollow(&mut state, 0.9);
        let w = cheek_morph_weights(&state);
        assert!((w[0] - 0.3).abs() < 1e-6);
        assert!((w[1] - 0.6).abs() < 1e-6);
        assert!((w[2] - 0.1).abs() < 1e-6);
        assert!((w[3] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_reset_cheek_puff() {
        let cfg = default_cheek_puff_config();
        let mut state = new_cheek_puff_state(&cfg);
        set_left_puff(&mut state, 0.5);
        set_right_hollow(&mut state, 0.5);
        reset_cheek_puff(&mut state);
        assert!(cheeks_are_neutral(&state));
    }

    #[test]
    fn test_blend_cheek_states() {
        let cfg = default_cheek_puff_config();
        let mut a = new_cheek_puff_state(&cfg);
        let mut b = new_cheek_puff_state(&cfg);
        set_left_puff(&mut a, 0.0);
        set_left_puff(&mut b, 1.0);
        let mid = blend_cheek_states(&a, &b, 0.5);
        assert!((mid.left_puff - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamps_t() {
        let cfg = default_cheek_puff_config();
        let mut a = new_cheek_puff_state(&cfg);
        let mut b = new_cheek_puff_state(&cfg);
        set_right_puff(&mut a, 0.2);
        set_right_puff(&mut b, 0.8);
        let full = blend_cheek_states(&a, &b, 2.0);
        assert!((full.right_puff - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_not_neutral_after_set() {
        let cfg = default_cheek_puff_config();
        let mut state = new_cheek_puff_state(&cfg);
        set_left_hollow(&mut state, 0.1);
        assert!(!cheeks_are_neutral(&state));
    }
}
