//! Eye blink controller with natural variation.

#[allow(dead_code)]
pub struct BlinkParams {
    /// Seconds for one complete blink.
    pub blink_duration: f32,
    /// Average blinks per second.
    pub blink_rate_hz: f32,
    /// Timing randomness factor.
    pub variation: f32,
    /// Eye close speed multiplier.
    pub close_speed: f32,
    /// Eye open speed multiplier.
    pub open_speed: f32,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BlinkPhase {
    Open,
    Closing,
    Closed,
    Opening,
}

#[allow(dead_code)]
pub struct BlinkState {
    pub phase: BlinkPhase,
    pub phase_time: f32,
    pub next_blink_time: f32,
    /// 0 = closed, 1 = open.
    pub left_eye_open: f32,
    pub right_eye_open: f32,
    pub synchronized: bool,
    pub enabled: bool,
}

// ---------------------------------------------------------------------------
// LCG RNG helper (no external deps)
// ---------------------------------------------------------------------------

/// Linear congruential generator. Advances `state` and returns a value in [0, 1).
#[allow(dead_code)]
pub fn lcg_next(state: &mut u64) -> f32 {
    // Parameters from Knuth / Numerical Recipes (64-bit LCG).
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    // Use upper bits for better quality.
    let bits = (*state >> 33) as u32;
    (bits as f32) / (u32::MAX as f32 + 1.0)
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn default_blink_params() -> BlinkParams {
    BlinkParams {
        blink_duration: 0.15,
        blink_rate_hz: 0.25,
        variation: 0.3,
        close_speed: 1.0,
        open_speed: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_blink_state(sync: bool) -> BlinkState {
    BlinkState {
        phase: BlinkPhase::Open,
        phase_time: 0.0,
        next_blink_time: 3.0,
        left_eye_open: 1.0,
        right_eye_open: 1.0,
        synchronized: sync,
        enabled: true,
    }
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn update_blink(state: &mut BlinkState, params: &BlinkParams, dt: f32, rng: &mut u64) {
    if !state.enabled {
        return;
    }

    state.phase_time += dt;

    match state.phase {
        BlinkPhase::Open => {
            state.left_eye_open = 1.0;
            state.right_eye_open = 1.0;
            if state.phase_time >= state.next_blink_time {
                state.phase = BlinkPhase::Closing;
                state.phase_time = 0.0;
            }
        }
        BlinkPhase::Closing => {
            let half = params.blink_duration * 0.5 / params.close_speed.max(0.001);
            let t = (state.phase_time / half).min(1.0);
            let v = 1.0 - t;
            state.left_eye_open = v;
            if state.synchronized {
                state.right_eye_open = v;
            } else {
                state.right_eye_open = (v + lcg_next(rng) * 0.1).min(1.0);
            }
            if state.phase_time >= half {
                state.phase = BlinkPhase::Closed;
                state.phase_time = 0.0;
                state.left_eye_open = 0.0;
                state.right_eye_open = 0.0;
            }
        }
        BlinkPhase::Closed => {
            let hold = params.blink_duration * 0.1;
            if state.phase_time >= hold {
                state.phase = BlinkPhase::Opening;
                state.phase_time = 0.0;
            }
        }
        BlinkPhase::Opening => {
            let half = params.blink_duration * 0.5 / params.open_speed.max(0.001);
            let t = (state.phase_time / half).min(1.0);
            state.left_eye_open = t;
            if state.synchronized {
                state.right_eye_open = t;
            } else {
                state.right_eye_open = (t + lcg_next(rng) * 0.05).min(1.0);
            }
            if state.phase_time >= half {
                state.phase = BlinkPhase::Open;
                state.phase_time = 0.0;
                state.left_eye_open = 1.0;
                state.right_eye_open = 1.0;
                // Schedule next blink with variation.
                let base = 1.0 / params.blink_rate_hz.max(0.001);
                let var = (lcg_next(rng) * 2.0 - 1.0) * params.variation * base;
                state.next_blink_time = (base + var).max(0.1);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Control
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn trigger_manual_blink(state: &mut BlinkState) {
    if state.phase == BlinkPhase::Open {
        state.phase = BlinkPhase::Closing;
        state.phase_time = 0.0;
    }
}

#[allow(dead_code)]
pub fn blink_value(state: &BlinkState) -> f32 {
    (state.left_eye_open + state.right_eye_open) * 0.5
}

#[allow(dead_code)]
pub fn set_blink_synchronized(state: &mut BlinkState, sync: bool) {
    state.synchronized = sync;
}

#[allow(dead_code)]
pub fn enable_blink(state: &mut BlinkState) {
    state.enabled = true;
}

#[allow(dead_code)]
pub fn disable_blink(state: &mut BlinkState) {
    state.enabled = false;
}

#[allow(dead_code)]
pub fn is_blinking(state: &BlinkState) -> bool {
    state.phase != BlinkPhase::Open
}

#[allow(dead_code)]
pub fn force_open_eyes(state: &mut BlinkState) {
    state.phase = BlinkPhase::Open;
    state.phase_time = 0.0;
    state.left_eye_open = 1.0;
    state.right_eye_open = 1.0;
}

#[allow(dead_code)]
pub fn force_close_eyes(state: &mut BlinkState) {
    state.phase = BlinkPhase::Closed;
    state.phase_time = 0.0;
    state.left_eye_open = 0.0;
    state.right_eye_open = 0.0;
}

#[allow(dead_code)]
pub fn blink_speed_for_emotion(emotion: &str) -> f32 {
    match emotion {
        "surprised" => 2.0,
        "tired" => 0.5,
        _ => 1.0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let p = default_blink_params();
        assert!(p.blink_duration > 0.0);
        assert!(p.blink_rate_hz > 0.0);
        assert!(p.variation >= 0.0);
        assert!(p.close_speed > 0.0);
        assert!(p.open_speed > 0.0);
    }

    #[test]
    fn test_new_state_defaults() {
        let state = new_blink_state(true);
        assert_eq!(state.phase, BlinkPhase::Open);
        assert!((state.left_eye_open - 1.0).abs() < 1e-6);
        assert!((state.right_eye_open - 1.0).abs() < 1e-6);
        assert!(state.synchronized);
        assert!(state.enabled);
    }

    #[test]
    fn test_update_advances_phase() {
        let params = default_blink_params();
        let mut state = new_blink_state(true);
        // Move past next_blink_time to trigger a blink.
        let mut rng: u64 = 42;
        let trigger_time = state.next_blink_time + 0.01;
        update_blink(&mut state, &params, trigger_time, &mut rng);
        assert_ne!(state.phase, BlinkPhase::Open);
    }

    #[test]
    fn test_trigger_manual_blink() {
        let mut state = new_blink_state(true);
        assert_eq!(state.phase, BlinkPhase::Open);
        trigger_manual_blink(&mut state);
        assert_eq!(state.phase, BlinkPhase::Closing);
    }

    #[test]
    fn test_blink_value_open() {
        let state = new_blink_state(true);
        assert!((blink_value(&state) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blink_value_closed() {
        let mut state = new_blink_state(true);
        force_close_eyes(&mut state);
        assert!((blink_value(&state)).abs() < 1e-6);
    }

    #[test]
    fn test_enable_disable() {
        let mut state = new_blink_state(false);
        disable_blink(&mut state);
        assert!(!state.enabled);
        enable_blink(&mut state);
        assert!(state.enabled);
    }

    #[test]
    fn test_is_blinking_open() {
        let state = new_blink_state(true);
        assert!(!is_blinking(&state));
    }

    #[test]
    fn test_is_blinking_closing() {
        let mut state = new_blink_state(true);
        trigger_manual_blink(&mut state);
        assert!(is_blinking(&state));
    }

    #[test]
    fn test_force_open_eyes() {
        let mut state = new_blink_state(true);
        force_close_eyes(&mut state);
        force_open_eyes(&mut state);
        assert_eq!(state.phase, BlinkPhase::Open);
        assert!((state.left_eye_open - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_force_close_eyes() {
        let mut state = new_blink_state(true);
        force_close_eyes(&mut state);
        assert_eq!(state.phase, BlinkPhase::Closed);
        assert!((state.left_eye_open).abs() < 1e-6);
    }

    #[test]
    fn test_blink_speed_for_emotion() {
        assert!((blink_speed_for_emotion("surprised") - 2.0).abs() < 1e-6);
        assert!((blink_speed_for_emotion("tired") - 0.5).abs() < 1e-6);
        assert!((blink_speed_for_emotion("neutral") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_lcg_next_range() {
        let mut rng: u64 = 123456;
        for _ in 0..1000 {
            let v = lcg_next(&mut rng);
            assert!((0.0..1.0).contains(&v), "lcg_next out of [0,1): {}", v);
        }
    }

    #[test]
    fn test_set_blink_synchronized() {
        let mut state = new_blink_state(true);
        set_blink_synchronized(&mut state, false);
        assert!(!state.synchronized);
        set_blink_synchronized(&mut state, true);
        assert!(state.synchronized);
    }

    #[test]
    fn test_full_blink_cycle() {
        let params = default_blink_params();
        let mut state = new_blink_state(true);
        let mut rng: u64 = 999;
        trigger_manual_blink(&mut state);
        // Step through closing
        for _ in 0..100 {
            update_blink(&mut state, &params, 0.01, &mut rng);
        }
        // After enough time the cycle should complete and return to Open
        assert_eq!(state.phase, BlinkPhase::Open);
    }
}
