//! Neck morphology controls for character customization.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckConfig {
    pub neck_length: f32,
    pub neck_width: f32,
    pub neck_tilt: f32,
    pub adam_apple: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckState {
    pub length: f32,
    pub width: f32,
    pub tilt: f32,
    pub adam_apple: f32,
    pub muscle_tone: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckMorphWeights {
    pub long: f32,
    pub short: f32,
    pub wide: f32,
    pub narrow: f32,
    pub tilt: f32,
    pub adam_apple: f32,
}

#[allow(dead_code)]
pub fn default_neck_config() -> NeckConfig {
    NeckConfig {
        neck_length: 0.5,
        neck_width: 0.5,
        neck_tilt: 0.0,
        adam_apple: 0.0,
    }
}

#[allow(dead_code)]
pub fn new_neck_state() -> NeckState {
    NeckState {
        length: 0.5,
        width: 0.5,
        tilt: 0.0,
        adam_apple: 0.0,
        muscle_tone: 0.3,
    }
}

#[allow(dead_code)]
pub fn set_neck_length(state: &mut NeckState, length: f32) {
    state.length = length.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_neck_width(state: &mut NeckState, width: f32) {
    state.width = width.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_neck_tilt(state: &mut NeckState, tilt: f32) {
    state.tilt = tilt.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn set_adam_apple(state: &mut NeckState, amount: f32) {
    state.adam_apple = amount.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_neck_weights(state: &NeckState, cfg: &NeckConfig) -> NeckMorphWeights {
    let len_bias = cfg.neck_length;
    let wid_bias = cfg.neck_width;
    let long = (state.length * len_bias).clamp(0.0, 1.0);
    let short = ((1.0 - state.length) * len_bias).clamp(0.0, 1.0);
    let wide = (state.width * wid_bias).clamp(0.0, 1.0);
    let narrow = ((1.0 - state.width) * wid_bias).clamp(0.0, 1.0);
    let tilt = (state.tilt.abs() * cfg.neck_tilt.abs().max(state.tilt.abs())).clamp(0.0, 1.0);
    let adam_apple = (state.adam_apple * (cfg.adam_apple + 0.001)).clamp(0.0, 1.0);
    NeckMorphWeights { long, short, wide, narrow, tilt, adam_apple }
}

#[allow(dead_code)]
pub fn blend_neck(a: &NeckState, b: &NeckState, t: f32) -> NeckState {
    let t = t.clamp(0.0, 1.0);
    let u = 1.0 - t;
    NeckState {
        length: a.length * u + b.length * t,
        width: a.width * u + b.width * t,
        tilt: a.tilt * u + b.tilt * t,
        adam_apple: a.adam_apple * u + b.adam_apple * t,
        muscle_tone: a.muscle_tone * u + b.muscle_tone * t,
    }
}

#[allow(dead_code)]
pub fn reset_neck(state: &mut NeckState) {
    *state = new_neck_state();
}

#[allow(dead_code)]
pub fn neck_state_to_json(state: &NeckState) -> String {
    format!(
        r#"{{"length":{:.4},"width":{:.4},"tilt":{:.4},"adam_apple":{:.4},"muscle_tone":{:.4}}}"#,
        state.length, state.width, state.tilt, state.adam_apple, state.muscle_tone
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_neck_config() {
        let cfg = default_neck_config();
        assert!((cfg.neck_length - 0.5).abs() < 1e-6);
        assert!(cfg.adam_apple.abs() < 1e-6);
    }

    #[test]
    fn test_new_neck_state() {
        let s = new_neck_state();
        assert!((s.length - 0.5).abs() < 1e-6);
        assert!(s.tilt.abs() < 1e-6);
    }

    #[test]
    fn test_set_neck_length_clamp() {
        let mut s = new_neck_state();
        set_neck_length(&mut s, 2.5);
        assert!((s.length - 1.0).abs() < 1e-6);
        set_neck_length(&mut s, -0.5);
        assert!(s.length.abs() < 1e-6);
    }

    #[test]
    fn test_set_neck_tilt_clamp() {
        let mut s = new_neck_state();
        set_neck_tilt(&mut s, -5.0);
        assert!((s.tilt + 1.0).abs() < 1e-6);
        set_neck_tilt(&mut s, 5.0);
        assert!((s.tilt - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_adam_apple() {
        let mut s = new_neck_state();
        set_adam_apple(&mut s, 0.6);
        assert!((s.adam_apple - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_blend_neck_midpoint() {
        let a = new_neck_state();
        let mut b = new_neck_state();
        b.length = 1.0;
        let mid = blend_neck(&a, &b, 0.5);
        assert!((mid.length - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_reset_neck() {
        let mut s = new_neck_state();
        s.adam_apple = 1.0;
        s.width = 0.9;
        reset_neck(&mut s);
        assert!(s.adam_apple.abs() < 1e-6);
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_neck_state_to_json() {
        let s = new_neck_state();
        let j = neck_state_to_json(&s);
        assert!(j.contains("length"));
        assert!(j.contains("muscle_tone"));
    }

    #[test]
    fn test_compute_neck_weights() {
        let s = new_neck_state();
        let cfg = default_neck_config();
        let w = compute_neck_weights(&s, &cfg);
        assert!(w.long >= 0.0 && w.long <= 1.0);
        assert!(w.wide >= 0.0 && w.wide <= 1.0);
    }
}
