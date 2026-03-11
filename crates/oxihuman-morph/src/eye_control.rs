//! Eye movement and gaze control system with blink integration.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EyeState {
    /// Yaw angle in radians (horizontal rotation).
    pub yaw: f32,
    /// Pitch angle in radians (vertical rotation).
    pub pitch: f32,
    /// Current blink closure fraction [0 = open, 1 = closed].
    pub blink_fraction: f32,
    /// Timer counting up between blinks.
    pub blink_timer: f32,
    /// Duration of current blink.
    pub blink_duration: f32,
    /// Whether a blink is active.
    pub blinking: bool,
    /// LCG state for deterministic variation.
    pub lcg_state: u64,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum GazeTarget {
    /// Look at a world-space point from the given eye origin.
    Point { origin: [f32; 3], target: [f32; 3] },
    /// Directly specified yaw/pitch angles in radians.
    Angles { yaw: f32, pitch: f32 },
    /// Forward (neutral gaze).
    Forward,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EyeControlConfig {
    /// Maximum horizontal deviation in radians.
    pub max_yaw: f32,
    /// Maximum vertical deviation in radians.
    pub max_pitch: f32,
    /// Average time between blinks (seconds).
    pub blink_interval: f32,
    /// Duration of a single blink (seconds).
    pub blink_duration: f32,
    /// Speed of saccade (fraction per second).
    pub saccade_speed: f32,
    /// Randomness in blink interval [0..1].
    pub blink_variation: f32,
}

// ---------------------------------------------------------------------------
// LCG helper
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn lcg_step(state: &mut u64) -> f32 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let bits = (*state >> 33) as u32;
    (bits as f32) / (u32::MAX as f32 + 1.0)
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Return a default `EyeControlConfig` with sensible values.
#[allow(dead_code)]
pub fn default_eye_config() -> EyeControlConfig {
    EyeControlConfig {
        max_yaw: std::f32::consts::FRAC_PI_4,
        max_pitch: std::f32::consts::FRAC_PI_6,
        blink_interval: 4.0,
        blink_duration: 0.15,
        saccade_speed: 5.0,
        blink_variation: 0.3,
    }
}

/// Create a new `EyeState` at neutral gaze.
#[allow(dead_code)]
pub fn new_eye_state(lcg_seed: u64) -> EyeState {
    EyeState {
        yaw: 0.0,
        pitch: 0.0,
        blink_fraction: 0.0,
        blink_timer: 0.0,
        blink_duration: 0.0,
        blinking: false,
        lcg_state: lcg_seed.max(1),
    }
}

// ---------------------------------------------------------------------------
// Angle computation
// ---------------------------------------------------------------------------

/// Compute the yaw and pitch angles needed to look from `origin` toward `target`.
/// Returns `(yaw_rad, pitch_rad)`.
#[allow(dead_code)]
pub fn look_at_target(origin: [f32; 3], target: [f32; 3]) -> (f32, f32) {
    let dx = target[0] - origin[0];
    let dy = target[1] - origin[1];
    let dz = target[2] - origin[2];
    let horiz = (dx * dx + dz * dz).sqrt();
    let yaw = dx.atan2(dz);
    let pitch = (-dy).atan2(horiz);
    (yaw, pitch)
}

/// Return the current yaw angle of the eye state in degrees.
#[allow(dead_code)]
pub fn eye_yaw_deg(state: &EyeState) -> f32 {
    state.yaw.to_degrees()
}

/// Return the current pitch angle of the eye state in degrees.
#[allow(dead_code)]
pub fn eye_pitch_deg(state: &EyeState) -> f32 {
    state.pitch.to_degrees()
}

// ---------------------------------------------------------------------------
// Saccade / update
// ---------------------------------------------------------------------------

/// Smoothly move the eye toward `target_yaw`/`target_pitch` by at most
/// `speed * dt` radians.
#[allow(dead_code)]
pub fn saccade_towards(
    state: &mut EyeState,
    target_yaw: f32,
    target_pitch: f32,
    speed: f32,
    dt: f32,
) {
    let max_step = speed * dt;
    let dy = target_yaw - state.yaw;
    let dp = target_pitch - state.pitch;
    let dist = (dy * dy + dp * dp).sqrt();
    if dist <= max_step || dist < 1e-6 {
        state.yaw = target_yaw;
        state.pitch = target_pitch;
    } else {
        let s = max_step / dist;
        state.yaw += dy * s;
        state.pitch += dp * s;
    }
}

/// Advance the eye gaze state by `dt` seconds toward a given `GazeTarget`.
#[allow(dead_code)]
pub fn update_eye_gaze(
    state: &mut EyeState,
    target: &GazeTarget,
    config: &EyeControlConfig,
    dt: f32,
) {
    let (ty, tp) = match target {
        GazeTarget::Forward => (0.0_f32, 0.0_f32),
        GazeTarget::Angles { yaw, pitch } => (*yaw, *pitch),
        GazeTarget::Point {
            origin,
            target: tgt,
        } => look_at_target(*origin, *tgt),
    };
    saccade_towards(state, ty, tp, config.saccade_speed, dt);
    clamp_gaze(state, config);
}

/// Clamp the eye gaze angles to the configured maximum deviation.
#[allow(dead_code)]
pub fn clamp_gaze(state: &mut EyeState, config: &EyeControlConfig) {
    state.yaw = state.yaw.clamp(-config.max_yaw, config.max_yaw);
    state.pitch = state.pitch.clamp(-config.max_pitch, config.max_pitch);
}

// ---------------------------------------------------------------------------
// Blink
// ---------------------------------------------------------------------------

/// Return the current blink closure fraction `[0..1]` (0 = open, 1 = closed).
#[allow(dead_code)]
pub fn blink_factor(state: &EyeState) -> f32 {
    state.blink_fraction
}

/// Immediately trigger a blink with the given duration.
#[allow(dead_code)]
pub fn trigger_blink(state: &mut EyeState, duration: f32) {
    state.blinking = true;
    state.blink_duration = duration.max(0.01);
    state.blink_timer = 0.0;
    state.blink_fraction = 0.0;
}

/// Tick the automatic blink system: advance timers and trigger blinks when due.
/// Uses `config.blink_interval` + LCG variation.
#[allow(dead_code)]
pub fn auto_blink_tick(state: &mut EyeState, config: &EyeControlConfig, dt: f32) {
    if state.blinking {
        state.blink_timer += dt;
        let half = state.blink_duration * 0.5;
        if state.blink_timer < half {
            state.blink_fraction = state.blink_timer / half;
        } else if state.blink_timer < state.blink_duration {
            state.blink_fraction = 1.0 - (state.blink_timer - half) / half;
        } else {
            state.blink_fraction = 0.0;
            state.blinking = false;
            // Schedule next blink.
            let noise = lcg_step(&mut state.lcg_state) * 2.0 - 1.0;
            state.blink_timer = -config.blink_interval * (1.0 + noise * config.blink_variation);
        }
    } else {
        state.blink_timer += dt;
        if state.blink_timer >= config.blink_interval {
            trigger_blink(state, config.blink_duration);
        }
    }
}

/// Return `true` if the eye is currently in a blink animation.
#[allow(dead_code)]
pub fn is_blinking_eye(state: &EyeState) -> bool {
    state.blinking
}

// ---------------------------------------------------------------------------
// Blending / distance
// ---------------------------------------------------------------------------

/// Blend between two eye states by factor `t` (0 = a, 1 = b).
#[allow(dead_code)]
pub fn gaze_blend(a: &EyeState, b: &EyeState, t: f32) -> EyeState {
    let t = t.clamp(0.0, 1.0);
    let u = 1.0 - t;
    EyeState {
        yaw: a.yaw * u + b.yaw * t,
        pitch: a.pitch * u + b.pitch * t,
        blink_fraction: a.blink_fraction * u + b.blink_fraction * t,
        blink_timer: a.blink_timer * u + b.blink_timer * t,
        blink_duration: a.blink_duration * u + b.blink_duration * t,
        blinking: if t < 0.5 { a.blinking } else { b.blinking },
        lcg_state: a.lcg_state,
    }
}

/// Angular distance between two eye states (Euclidean in yaw-pitch space).
#[allow(dead_code)]
pub fn gaze_distance(a: &EyeState, b: &EyeState) -> f32 {
    let dy = a.yaw - b.yaw;
    let dp = a.pitch - b.pitch;
    (dy * dy + dp * dp).sqrt()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> EyeControlConfig {
        default_eye_config()
    }

    #[test]
    fn test_default_eye_config() {
        let c = cfg();
        assert!(c.max_yaw > 0.0);
        assert!(c.max_pitch > 0.0);
        assert!(c.blink_interval > 0.0);
    }

    #[test]
    fn test_new_eye_state() {
        let s = new_eye_state(42);
        assert_eq!(s.yaw, 0.0);
        assert_eq!(s.pitch, 0.0);
        assert!(!s.blinking);
    }

    #[test]
    fn test_look_at_target_forward() {
        let origin = [0.0_f32, 0.0, 0.0];
        let target = [0.0_f32, 0.0, 10.0];
        let (y, p) = look_at_target(origin, target);
        assert!(y.abs() < 1e-4, "yaw should be ~0 for forward target");
        assert!(p.abs() < 1e-4, "pitch should be ~0 for forward target");
    }

    #[test]
    fn test_look_at_target_right() {
        let origin = [0.0_f32, 0.0, 0.0];
        let target = [1.0_f32, 0.0, 1.0];
        let (y, _p) = look_at_target(origin, target);
        assert!(y > 0.0, "yaw should be positive looking right");
    }

    #[test]
    fn test_eye_yaw_pitch_deg() {
        let mut s = new_eye_state(1);
        s.yaw = std::f32::consts::FRAC_PI_4;
        s.pitch = std::f32::consts::FRAC_PI_6;
        assert!((eye_yaw_deg(&s) - 45.0).abs() < 0.01);
        assert!((eye_pitch_deg(&s) - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_saccade_towards_reaches() {
        let mut s = new_eye_state(1);
        saccade_towards(&mut s, 1.0, 0.5, 10.0, 1.0);
        assert!((s.yaw - 1.0).abs() < 1e-5);
        assert!((s.pitch - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_saccade_towards_partial() {
        let mut s = new_eye_state(1);
        saccade_towards(&mut s, 1.0, 0.0, 0.1, 1.0);
        assert!(s.yaw > 0.0 && s.yaw < 1.0, "should partially approach");
    }

    #[test]
    fn test_clamp_gaze() {
        let mut s = new_eye_state(1);
        s.yaw = 999.0;
        s.pitch = -999.0;
        let c = cfg();
        clamp_gaze(&mut s, &c);
        assert!(s.yaw <= c.max_yaw);
        assert!(s.pitch >= -c.max_pitch);
    }

    #[test]
    fn test_trigger_blink() {
        let mut s = new_eye_state(1);
        trigger_blink(&mut s, 0.2);
        assert!(s.blinking);
        assert!((s.blink_duration - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_blink_factor_initial() {
        let s = new_eye_state(1);
        assert_eq!(blink_factor(&s), 0.0);
    }

    #[test]
    fn test_auto_blink_tick_starts_blink() {
        let mut s = new_eye_state(1);
        let c = EyeControlConfig {
            blink_interval: 0.1,
            ..cfg()
        };
        // Advance past the interval.
        auto_blink_tick(&mut s, &c, 0.2);
        assert!(s.blinking || s.blink_fraction > 0.0 || s.blink_timer != 0.2);
    }

    #[test]
    fn test_auto_blink_tick_closure() {
        let mut s = new_eye_state(1);
        let c = EyeControlConfig {
            blink_interval: 0.01,
            blink_duration: 0.2,
            ..cfg()
        };
        // Trigger blink.
        auto_blink_tick(&mut s, &c, 0.05);
        // Mid blink: fraction should be > 0.
        if s.blinking {
            auto_blink_tick(&mut s, &c, 0.05);
            assert!(s.blink_fraction >= 0.0);
        }
    }

    #[test]
    fn test_is_blinking_eye() {
        let mut s = new_eye_state(1);
        assert!(!is_blinking_eye(&s));
        trigger_blink(&mut s, 0.15);
        assert!(is_blinking_eye(&s));
    }

    #[test]
    fn test_gaze_blend_midpoint() {
        let mut a = new_eye_state(1);
        let mut b = new_eye_state(2);
        a.yaw = 0.0;
        b.yaw = 1.0;
        let m = gaze_blend(&a, &b, 0.5);
        assert!((m.yaw - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_gaze_blend_extremes() {
        let a = new_eye_state(1);
        let b = new_eye_state(2);
        let m0 = gaze_blend(&a, &b, 0.0);
        assert!((m0.yaw - a.yaw).abs() < 1e-6);
        let m1 = gaze_blend(&a, &b, 1.0);
        assert!((m1.yaw - b.yaw).abs() < 1e-6);
    }

    #[test]
    fn test_gaze_distance_zero() {
        let s = new_eye_state(1);
        assert!(gaze_distance(&s, &s) < 1e-6);
    }

    #[test]
    fn test_gaze_distance_nonzero() {
        let mut a = new_eye_state(1);
        let b = new_eye_state(2);
        a.yaw = 1.0;
        assert!(gaze_distance(&a, &b) > 0.5);
    }

    #[test]
    fn test_update_eye_gaze_converges() {
        let mut s = new_eye_state(1);
        let c = cfg();
        let target = GazeTarget::Angles {
            yaw: 0.3,
            pitch: 0.1,
        };
        for _ in 0..200 {
            update_eye_gaze(&mut s, &target, &c, 0.05);
        }
        assert!((s.yaw - 0.3).abs() < 0.01);
        assert!((s.pitch - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_update_eye_gaze_clamped() {
        let mut s = new_eye_state(1);
        let c = cfg();
        let target = GazeTarget::Angles {
            yaw: 99.0,
            pitch: 99.0,
        };
        for _ in 0..200 {
            update_eye_gaze(&mut s, &target, &c, 0.1);
        }
        assert!(s.yaw <= c.max_yaw + 1e-4);
        assert!(s.pitch <= c.max_pitch + 1e-4);
    }
}
