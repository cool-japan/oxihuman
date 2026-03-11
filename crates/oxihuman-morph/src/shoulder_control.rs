//! Shoulder shape and muscle morphology controls for character customization.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShoulderConfig {
    pub width: f32,
    pub slope: f32,
    pub muscle_mass: f32,
    pub acromion_height: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShoulderState {
    pub width_l: f32,
    pub width_r: f32,
    pub slope_l: f32,
    pub slope_r: f32,
    pub muscle_l: f32,
    pub muscle_r: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShoulderMorphWeights {
    pub broad_l: f32,
    pub broad_r: f32,
    pub narrow_l: f32,
    pub narrow_r: f32,
    pub slope_l: f32,
    pub slope_r: f32,
}

#[allow(dead_code)]
pub fn default_shoulder_config() -> ShoulderConfig {
    ShoulderConfig {
        width: 0.5,
        slope: 0.0,
        muscle_mass: 0.5,
        acromion_height: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_shoulder_state() -> ShoulderState {
    ShoulderState {
        width_l: 0.5,
        width_r: 0.5,
        slope_l: 0.0,
        slope_r: 0.0,
        muscle_l: 0.5,
        muscle_r: 0.5,
    }
}

#[allow(dead_code)]
pub fn set_shoulder_width(state: &mut ShoulderState, left: f32, right: f32) {
    state.width_l = left.clamp(0.0, 1.0);
    state.width_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_shoulder_slope(state: &mut ShoulderState, left: f32, right: f32) {
    state.slope_l = left.clamp(-1.0, 1.0);
    state.slope_r = right.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn set_shoulder_muscle(state: &mut ShoulderState, left: f32, right: f32) {
    state.muscle_l = left.clamp(0.0, 1.0);
    state.muscle_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn symmetrize_shoulders(state: &mut ShoulderState) {
    let avg_width = (state.width_l + state.width_r) * 0.5;
    let avg_slope = (state.slope_l + state.slope_r) * 0.5;
    let avg_muscle = (state.muscle_l + state.muscle_r) * 0.5;
    state.width_l = avg_width;
    state.width_r = avg_width;
    state.slope_l = avg_slope;
    state.slope_r = avg_slope;
    state.muscle_l = avg_muscle;
    state.muscle_r = avg_muscle;
}

#[allow(dead_code)]
pub fn compute_shoulder_weights(state: &ShoulderState, cfg: &ShoulderConfig) -> ShoulderMorphWeights {
    let broad_l = (state.width_l * cfg.width).clamp(0.0, 1.0);
    let broad_r = (state.width_r * cfg.width).clamp(0.0, 1.0);
    let narrow_l = ((1.0 - state.width_l) * cfg.width).clamp(0.0, 1.0);
    let narrow_r = ((1.0 - state.width_r) * cfg.width).clamp(0.0, 1.0);
    let slope_l = (state.slope_l * cfg.slope.abs()).clamp(-1.0, 1.0);
    let slope_r = (state.slope_r * cfg.slope.abs()).clamp(-1.0, 1.0);
    ShoulderMorphWeights { broad_l, broad_r, narrow_l, narrow_r, slope_l, slope_r }
}

#[allow(dead_code)]
pub fn blend_shoulders(a: &ShoulderState, b: &ShoulderState, t: f32) -> ShoulderState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    ShoulderState {
        width_l: a.width_l * inv + b.width_l * t,
        width_r: a.width_r * inv + b.width_r * t,
        slope_l: a.slope_l * inv + b.slope_l * t,
        slope_r: a.slope_r * inv + b.slope_r * t,
        muscle_l: a.muscle_l * inv + b.muscle_l * t,
        muscle_r: a.muscle_r * inv + b.muscle_r * t,
    }
}

#[allow(dead_code)]
pub fn reset_shoulders(state: &mut ShoulderState) {
    *state = new_shoulder_state();
}

#[allow(dead_code)]
pub fn shoulder_state_to_json(state: &ShoulderState) -> String {
    format!(
        r#"{{"width_l":{:.4},"width_r":{:.4},"slope_l":{:.4},"slope_r":{:.4},"muscle_l":{:.4},"muscle_r":{:.4}}}"#,
        state.width_l, state.width_r, state.slope_l, state.slope_r, state.muscle_l, state.muscle_r
    )
}

#[allow(dead_code)]
pub fn shoulder_width_avg(state: &ShoulderState) -> f32 {
    (state.width_l + state.width_r) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_shoulder_config() {
        let cfg = default_shoulder_config();
        assert!((cfg.width - 0.5).abs() < 1e-6);
        assert!((cfg.muscle_mass - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_new_shoulder_state() {
        let s = new_shoulder_state();
        assert!((s.width_l - 0.5).abs() < 1e-6);
        assert!((s.width_r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_shoulder_width_clamps() {
        let mut s = new_shoulder_state();
        set_shoulder_width(&mut s, -0.5, 1.5);
        assert!((s.width_l - 0.0).abs() < 1e-6);
        assert!((s.width_r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_symmetrize_shoulders() {
        let mut s = new_shoulder_state();
        s.width_l = 0.2;
        s.width_r = 0.8;
        symmetrize_shoulders(&mut s);
        assert!((s.width_l - 0.5).abs() < 1e-6);
        assert!((s.width_r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_shoulders_at_zero() {
        let a = new_shoulder_state();
        let mut b = new_shoulder_state();
        b.width_l = 1.0;
        let result = blend_shoulders(&a, &b, 0.0);
        assert!((result.width_l - a.width_l).abs() < 1e-6);
    }

    #[test]
    fn test_blend_shoulders_at_one() {
        let a = new_shoulder_state();
        let mut b = new_shoulder_state();
        b.width_l = 1.0;
        let result = blend_shoulders(&a, &b, 1.0);
        assert!((result.width_l - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_shoulder_width_avg() {
        let mut s = new_shoulder_state();
        s.width_l = 0.2;
        s.width_r = 0.8;
        let avg = shoulder_width_avg(&s);
        assert!((avg - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_shoulder_state_to_json_contains_fields() {
        let s = new_shoulder_state();
        let j = shoulder_state_to_json(&s);
        assert!(j.contains("width_l"));
        assert!(j.contains("slope_r"));
        assert!(j.contains("muscle_l"));
    }

    #[test]
    fn test_reset_shoulders() {
        let mut s = new_shoulder_state();
        s.width_l = 0.9;
        s.slope_l = 0.7;
        reset_shoulders(&mut s);
        assert!((s.width_l - 0.5).abs() < 1e-6);
        assert!((s.slope_l - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_shoulder_weights() {
        let s = new_shoulder_state();
        let cfg = default_shoulder_config();
        let w = compute_shoulder_weights(&s, &cfg);
        assert!(w.broad_l >= 0.0 && w.broad_l <= 1.0);
        assert!(w.narrow_l >= 0.0 && w.narrow_l <= 1.0);
    }
}
