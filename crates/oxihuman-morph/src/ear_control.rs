//! Ear morphology control for character customization.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum EarSide {
    Left,
    Right,
    Both,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EarConfig {
    pub lobe_size: f32,
    pub helix_curl: f32,
    pub ear_protrusion: f32,
    pub ear_size: f32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EarState {
    pub left: EarConfig,
    pub right: EarConfig,
    pub symmetrical: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EarMorphWeights {
    pub lobe_l: f32,
    pub lobe_r: f32,
    pub helix_l: f32,
    pub helix_r: f32,
    pub protrusion_l: f32,
    pub protrusion_r: f32,
}

#[allow(dead_code)]
pub fn default_ear_config() -> EarConfig {
    EarConfig {
        lobe_size: 0.5,
        helix_curl: 0.0,
        ear_protrusion: 0.2,
        ear_size: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_ear_state() -> EarState {
    EarState {
        left: default_ear_config(),
        right: default_ear_config(),
        symmetrical: true,
    }
}

#[allow(dead_code)]
pub fn set_ear_size(state: &mut EarState, side: EarSide, size: f32) {
    let v = size.clamp(0.0, 1.0);
    match side {
        EarSide::Left => state.left.ear_size = v,
        EarSide::Right => state.right.ear_size = v,
        EarSide::Both => {
            state.left.ear_size = v;
            state.right.ear_size = v;
        }
    }
}

#[allow(dead_code)]
pub fn set_lobe_size(state: &mut EarState, side: EarSide, size: f32) {
    let v = size.clamp(0.0, 1.0);
    match side {
        EarSide::Left => state.left.lobe_size = v,
        EarSide::Right => state.right.lobe_size = v,
        EarSide::Both => {
            state.left.lobe_size = v;
            state.right.lobe_size = v;
        }
    }
}

#[allow(dead_code)]
pub fn set_helix_curl(state: &mut EarState, side: EarSide, curl: f32) {
    let v = curl.clamp(0.0, 1.0);
    match side {
        EarSide::Left => state.left.helix_curl = v,
        EarSide::Right => state.right.helix_curl = v,
        EarSide::Both => {
            state.left.helix_curl = v;
            state.right.helix_curl = v;
        }
    }
}

#[allow(dead_code)]
pub fn set_ear_protrusion(state: &mut EarState, side: EarSide, amount: f32) {
    let v = amount.clamp(0.0, 1.0);
    match side {
        EarSide::Left => state.left.ear_protrusion = v,
        EarSide::Right => state.right.ear_protrusion = v,
        EarSide::Both => {
            state.left.ear_protrusion = v;
            state.right.ear_protrusion = v;
        }
    }
}

#[allow(dead_code)]
pub fn symmetrize_ears(state: &mut EarState) {
    state.right = state.left.clone();
    state.symmetrical = true;
}

#[allow(dead_code)]
pub fn ear_state_to_morph_weights(state: &EarState) -> EarMorphWeights {
    EarMorphWeights {
        lobe_l: state.left.lobe_size,
        lobe_r: state.right.lobe_size,
        helix_l: state.left.helix_curl,
        helix_r: state.right.helix_curl,
        protrusion_l: state.left.ear_protrusion,
        protrusion_r: state.right.ear_protrusion,
    }
}

#[allow(dead_code)]
pub fn ear_state_to_json(state: &EarState) -> String {
    format!(
        "{{\"symmetrical\":{},\"left\":{{\"lobe_size\":{},\"helix_curl\":{},\"ear_protrusion\":{},\"ear_size\":{}}},\"right\":{{\"lobe_size\":{},\"helix_curl\":{},\"ear_protrusion\":{},\"ear_size\":{}}}}}",
        state.symmetrical,
        state.left.lobe_size,
        state.left.helix_curl,
        state.left.ear_protrusion,
        state.left.ear_size,
        state.right.lobe_size,
        state.right.helix_curl,
        state.right.ear_protrusion,
        state.right.ear_size,
    )
}

#[allow(dead_code)]
pub fn reset_ear_state(state: &mut EarState) {
    state.left = default_ear_config();
    state.right = default_ear_config();
    state.symmetrical = true;
}

#[allow(dead_code)]
pub fn blend_ear_states(a: &EarState, b: &EarState, t: f32) -> EarState {
    let t = t.clamp(0.0, 1.0);
    let s = 1.0 - t;
    EarState {
        left: EarConfig {
            lobe_size: a.left.lobe_size * s + b.left.lobe_size * t,
            helix_curl: a.left.helix_curl * s + b.left.helix_curl * t,
            ear_protrusion: a.left.ear_protrusion * s + b.left.ear_protrusion * t,
            ear_size: a.left.ear_size * s + b.left.ear_size * t,
        },
        right: EarConfig {
            lobe_size: a.right.lobe_size * s + b.right.lobe_size * t,
            helix_curl: a.right.helix_curl * s + b.right.helix_curl * t,
            ear_protrusion: a.right.ear_protrusion * s + b.right.ear_protrusion * t,
            ear_size: a.right.ear_size * s + b.right.ear_size * t,
        },
        symmetrical: a.symmetrical && b.symmetrical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ear_config() {
        let cfg = default_ear_config();
        assert!((cfg.lobe_size - 0.5).abs() < 1e-6);
        assert!((cfg.helix_curl).abs() < 1e-6);
        assert!((cfg.ear_protrusion - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_new_ear_state_symmetrical() {
        let state = new_ear_state();
        assert!(state.symmetrical);
        assert!((state.left.ear_size - state.right.ear_size).abs() < 1e-6);
    }

    #[test]
    fn test_set_ear_size_both() {
        let mut state = new_ear_state();
        set_ear_size(&mut state, EarSide::Both, 0.8);
        assert!((state.left.ear_size - 0.8).abs() < 1e-6);
        assert!((state.right.ear_size - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_lobe_size_left_only() {
        let mut state = new_ear_state();
        set_lobe_size(&mut state, EarSide::Left, 0.9);
        assert!((state.left.lobe_size - 0.9).abs() < 1e-6);
        assert!((state.right.lobe_size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_helix_curl_clamps() {
        let mut state = new_ear_state();
        set_helix_curl(&mut state, EarSide::Right, 2.0);
        assert!((state.right.helix_curl - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_ear_protrusion() {
        let mut state = new_ear_state();
        set_ear_protrusion(&mut state, EarSide::Left, 0.7);
        assert!((state.left.ear_protrusion - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_symmetrize_ears() {
        let mut state = new_ear_state();
        set_lobe_size(&mut state, EarSide::Left, 0.9);
        set_lobe_size(&mut state, EarSide::Right, 0.1);
        symmetrize_ears(&mut state);
        assert!((state.left.lobe_size - state.right.lobe_size).abs() < 1e-6);
        assert!(state.symmetrical);
    }

    #[test]
    fn test_ear_state_to_morph_weights() {
        let mut state = new_ear_state();
        set_lobe_size(&mut state, EarSide::Both, 0.6);
        let w = ear_state_to_morph_weights(&state);
        assert!((w.lobe_l - 0.6).abs() < 1e-6);
        assert!((w.lobe_r - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_ear_state_to_json() {
        let state = new_ear_state();
        let json = ear_state_to_json(&state);
        assert!(json.contains("lobe_size"));
        assert!(json.contains("symmetrical"));
    }

    #[test]
    fn test_reset_ear_state() {
        let mut state = new_ear_state();
        set_ear_size(&mut state, EarSide::Both, 1.0);
        reset_ear_state(&mut state);
        assert!((state.left.ear_size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_ear_states_midpoint() {
        let mut a = new_ear_state();
        let mut b = new_ear_state();
        set_ear_size(&mut a, EarSide::Both, 0.0);
        set_ear_size(&mut b, EarSide::Both, 1.0);
        let mid = blend_ear_states(&a, &b, 0.5);
        assert!((mid.left.ear_size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_ear_states_t0() {
        let a = new_ear_state();
        let b = new_ear_state();
        let result = blend_ear_states(&a, &b, 0.0);
        assert!((result.left.lobe_size - a.left.lobe_size).abs() < 1e-6);
    }
}
