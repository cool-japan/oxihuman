//! Eye gaze direction morph control (horizontal/vertical eye rotation).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeMovementConfig {
    pub max_horizontal: f32,
    pub max_vertical: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeMovementState {
    pub config: EyeMovementConfig,
    pub gaze_left: f32,
    pub gaze_right: f32,
    pub gaze_up: f32,
    pub gaze_down: f32,
    pub cross_eyed: f32,
}

#[allow(dead_code)]
pub fn default_eye_movement_config() -> EyeMovementConfig {
    EyeMovementConfig {
        max_horizontal: 1.0,
        max_vertical: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_eye_movement_state(cfg: &EyeMovementConfig) -> EyeMovementState {
    EyeMovementState {
        config: cfg.clone(),
        gaze_left: 0.0,
        gaze_right: 0.0,
        gaze_up: 0.0,
        gaze_down: 0.0,
        cross_eyed: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_gaze_left(state: &mut EyeMovementState, amount: f32) {
    state.gaze_left = amount.clamp(0.0, state.config.max_horizontal);
    state.gaze_right = 0.0;
}

#[allow(dead_code)]
pub fn set_gaze_right(state: &mut EyeMovementState, amount: f32) {
    state.gaze_right = amount.clamp(0.0, state.config.max_horizontal);
    state.gaze_left = 0.0;
}

#[allow(dead_code)]
pub fn set_gaze_up(state: &mut EyeMovementState, amount: f32) {
    state.gaze_up = amount.clamp(0.0, state.config.max_vertical);
    state.gaze_down = 0.0;
}

#[allow(dead_code)]
pub fn set_gaze_down(state: &mut EyeMovementState, amount: f32) {
    state.gaze_down = amount.clamp(0.0, state.config.max_vertical);
    state.gaze_up = 0.0;
}

/// Returns `[gaze_left, gaze_right, gaze_up, gaze_down]`.
#[allow(dead_code)]
pub fn eye_morph_weights(state: &EyeMovementState) -> [f32; 4] {
    [state.gaze_left, state.gaze_right, state.gaze_up, state.gaze_down]
}

#[allow(dead_code)]
pub fn set_cross_eyed(state: &mut EyeMovementState, amount: f32) {
    state.cross_eyed = amount.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn reset_eye_movement(state: &mut EyeMovementState) {
    state.gaze_left = 0.0;
    state.gaze_right = 0.0;
    state.gaze_up = 0.0;
    state.gaze_down = 0.0;
    state.cross_eyed = 0.0;
}

/// Returns the overall gaze angle in degrees, computed from the dominant horizontal/vertical pair.
#[allow(dead_code)]
pub fn gaze_angle_degrees(state: &EyeMovementState) -> f32 {
    let h = state.gaze_right - state.gaze_left;
    let v = state.gaze_up - state.gaze_down;
    v.atan2(h).to_degrees()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_eye_movement_config();
        assert_eq!(cfg.max_horizontal, 1.0);
        assert_eq!(cfg.max_vertical, 1.0);
    }

    #[test]
    fn test_new_state_zero() {
        let cfg = default_eye_movement_config();
        let state = new_eye_movement_state(&cfg);
        let w = eye_morph_weights(&state);
        assert!(w.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_set_gaze_left_clears_right() {
        let cfg = default_eye_movement_config();
        let mut state = new_eye_movement_state(&cfg);
        set_gaze_right(&mut state, 0.5);
        set_gaze_left(&mut state, 0.8);
        assert!((state.gaze_left - 0.8).abs() < 1e-6);
        assert_eq!(state.gaze_right, 0.0);
    }

    #[test]
    fn test_set_gaze_up_clears_down() {
        let cfg = default_eye_movement_config();
        let mut state = new_eye_movement_state(&cfg);
        set_gaze_down(&mut state, 0.3);
        set_gaze_up(&mut state, 0.7);
        assert!((state.gaze_up - 0.7).abs() < 1e-6);
        assert_eq!(state.gaze_down, 0.0);
    }

    #[test]
    fn test_clamp_exceeds_max() {
        let cfg = default_eye_movement_config();
        let mut state = new_eye_movement_state(&cfg);
        set_gaze_right(&mut state, 5.0);
        assert_eq!(state.gaze_right, 1.0);
    }

    #[test]
    fn test_set_cross_eyed_clamps() {
        let cfg = default_eye_movement_config();
        let mut state = new_eye_movement_state(&cfg);
        set_cross_eyed(&mut state, -0.5);
        assert_eq!(state.cross_eyed, 0.0);
        set_cross_eyed(&mut state, 2.0);
        assert_eq!(state.cross_eyed, 1.0);
    }

    #[test]
    fn test_reset_eye_movement() {
        let cfg = default_eye_movement_config();
        let mut state = new_eye_movement_state(&cfg);
        set_gaze_left(&mut state, 0.5);
        set_gaze_down(&mut state, 0.3);
        set_cross_eyed(&mut state, 0.4);
        reset_eye_movement(&mut state);
        let w = eye_morph_weights(&state);
        assert!(w.iter().all(|&v| v == 0.0));
        assert_eq!(state.cross_eyed, 0.0);
    }

    #[test]
    fn test_gaze_angle_degrees_right() {
        let cfg = default_eye_movement_config();
        let mut state = new_eye_movement_state(&cfg);
        set_gaze_right(&mut state, 1.0);
        let angle = gaze_angle_degrees(&state);
        assert!((angle - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_gaze_angle_degrees_up() {
        let cfg = default_eye_movement_config();
        let mut state = new_eye_movement_state(&cfg);
        set_gaze_up(&mut state, 1.0);
        let angle = gaze_angle_degrees(&state);
        assert!((angle - 90.0).abs() < 1e-4);
    }
}
