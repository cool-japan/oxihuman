// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Gaze tracking from screen/world coordinates with smooth pursuit.
//!
//! Provides gaze state management, smooth pursuit with delta-time updates,
//! and conversion of gaze angles to eye morph weights.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GazePoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub is_world_space: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GazeTrackConfig {
    /// Smooth pursuit speed in degrees per second.
    pub pursuit_speed: f32,
    /// Fixation threshold: velocity (deg/s) below which gaze is "fixating".
    pub fixation_threshold: f32,
    /// Maximum horizontal range in degrees (±).
    pub max_yaw_deg: f32,
    /// Maximum vertical range in degrees (±).
    pub max_pitch_deg: f32,
    /// Eye spacing in world units (for screen-space projection).
    pub eye_distance: f32,
    /// Saccade interpolation speed multiplier.
    pub saccade_speed: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GazeTrackState {
    /// Current gaze yaw in degrees.
    pub yaw_deg: f32,
    /// Current gaze pitch in degrees.
    pub pitch_deg: f32,
    /// Target gaze yaw in degrees.
    pub target_yaw_deg: f32,
    /// Target gaze pitch in degrees.
    pub target_pitch_deg: f32,
    /// Gaze velocity in degrees per second.
    pub velocity_deg_s: f32,
    /// Distance to gaze target.
    pub distance: f32,
    /// True if currently performing a saccade.
    pub in_saccade: bool,
    pub config: GazeTrackConfig,
}

/// Returns a default gaze tracking configuration.
#[allow(dead_code)]
pub fn default_gaze_track_config() -> GazeTrackConfig {
    GazeTrackConfig {
        pursuit_speed: 180.0,
        fixation_threshold: 2.0,
        max_yaw_deg: 45.0,
        max_pitch_deg: 30.0,
        eye_distance: 0.065,
        saccade_speed: 600.0,
    }
}

/// Creates a new gaze tracking state at rest (looking forward).
#[allow(dead_code)]
pub fn new_gaze_track_state(config: GazeTrackConfig) -> GazeTrackState {
    GazeTrackState {
        yaw_deg: 0.0,
        pitch_deg: 0.0,
        target_yaw_deg: 0.0,
        target_pitch_deg: 0.0,
        velocity_deg_s: 0.0,
        distance: 1.0,
        in_saccade: false,
        config,
    }
}

/// Sets the gaze target using a world-space 3D point.
/// The head is assumed to be at the origin looking along -Z.
#[allow(dead_code)]
pub fn set_gaze_target_world(state: &mut GazeTrackState, target: &GazePoint) {
    let dx = target.x;
    let dy = target.y;
    let dz = target.z;
    let horiz = (dx * dx + dz * dz).sqrt().max(1e-6);
    let yaw = dx.atan2(-dz).to_degrees();
    let pitch = (dy / horiz).atan().to_degrees();
    state.target_yaw_deg = yaw.clamp(-state.config.max_yaw_deg, state.config.max_yaw_deg);
    state.target_pitch_deg = pitch.clamp(-state.config.max_pitch_deg, state.config.max_pitch_deg);
    state.distance = (dx * dx + dy * dy + dz * dz).sqrt().max(0.01);
}

/// Sets the gaze target using normalized screen-space coordinates in [-1,1].
#[allow(dead_code)]
pub fn set_gaze_target_screen(state: &mut GazeTrackState, screen_x: f32, screen_y: f32) {
    let yaw = screen_x * state.config.max_yaw_deg;
    let pitch = -screen_y * state.config.max_pitch_deg;
    state.target_yaw_deg = yaw.clamp(-state.config.max_yaw_deg, state.config.max_yaw_deg);
    state.target_pitch_deg = pitch.clamp(-state.config.max_pitch_deg, state.config.max_pitch_deg);
    state.distance = 1.0;
}

/// Updates gaze tracking with smooth pursuit over a time delta.
#[allow(dead_code)]
pub fn update_gaze_tracking(state: &mut GazeTrackState, dt: f32) {
    let speed = if state.in_saccade {
        state.config.saccade_speed
    } else {
        state.config.pursuit_speed
    };
    let max_step = speed * dt;

    let dyaw = state.target_yaw_deg - state.yaw_deg;
    let dpitch = state.target_pitch_deg - state.pitch_deg;

    let dist_sq = dyaw * dyaw + dpitch * dpitch;
    let dist = dist_sq.sqrt();

    if dist < 1e-4 {
        state.velocity_deg_s = 0.0;
        state.in_saccade = false;
        return;
    }

    let step = max_step.min(dist);
    let t = step / dist;
    let prev_yaw = state.yaw_deg;
    let prev_pitch = state.pitch_deg;

    state.yaw_deg += dyaw * t;
    state.pitch_deg += dpitch * t;

    let actual_dyaw = state.yaw_deg - prev_yaw;
    let actual_dpitch = state.pitch_deg - prev_pitch;
    let actual_dist = (actual_dyaw * actual_dyaw + actual_dpitch * actual_dpitch).sqrt();
    state.velocity_deg_s = if dt > 1e-9 { actual_dist / dt } else { 0.0 };

    if dist <= step {
        state.in_saccade = false;
    }
}

/// Returns the current gaze yaw in degrees.
#[allow(dead_code)]
pub fn gaze_yaw_deg(state: &GazeTrackState) -> f32 {
    state.yaw_deg
}

/// Returns the current gaze pitch in degrees.
#[allow(dead_code)]
pub fn gaze_pitch_deg(state: &GazeTrackState) -> f32 {
    state.pitch_deg
}

/// Returns the gaze distance to the current target.
#[allow(dead_code)]
pub fn gaze_track_distance(state: &GazeTrackState) -> f32 {
    state.distance
}

/// Returns the current gaze velocity in degrees per second.
#[allow(dead_code)]
pub fn gaze_velocity(state: &GazeTrackState) -> f32 {
    state.velocity_deg_s
}

/// Returns true if gaze velocity is below the fixation threshold.
#[allow(dead_code)]
pub fn is_fixating(state: &GazeTrackState) -> bool {
    state.velocity_deg_s < state.config.fixation_threshold
}

/// Converts gaze angles to eye morph weights for the left and right eye.
/// Returns `(left_h, left_v, right_h, right_v)` in [−1, 1].
#[allow(dead_code)]
pub fn gaze_to_eye_morph_weights(state: &GazeTrackState) -> (f32, f32, f32, f32) {
    let h = state.yaw_deg / state.config.max_yaw_deg.max(1.0);
    let v = state.pitch_deg / state.config.max_pitch_deg.max(1.0);
    let h = h.clamp(-1.0, 1.0);
    let v = v.clamp(-1.0, 1.0);
    (h, v, h, v) // symmetric for both eyes
}

/// Resets gaze tracking state to looking straight forward.
#[allow(dead_code)]
pub fn reset_gaze_tracking(state: &mut GazeTrackState) {
    state.yaw_deg = 0.0;
    state.pitch_deg = 0.0;
    state.target_yaw_deg = 0.0;
    state.target_pitch_deg = 0.0;
    state.velocity_deg_s = 0.0;
    state.distance = 1.0;
    state.in_saccade = false;
}

/// Initiates a saccade (fast jump) to the given target angles.
#[allow(dead_code)]
pub fn saccade_to_target(state: &mut GazeTrackState, target_yaw: f32, target_pitch: f32) {
    state.target_yaw_deg = target_yaw.clamp(-state.config.max_yaw_deg, state.config.max_yaw_deg);
    state.target_pitch_deg =
        target_pitch.clamp(-state.config.max_pitch_deg, state.config.max_pitch_deg);
    state.in_saccade = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> GazeTrackState {
        new_gaze_track_state(default_gaze_track_config())
    }

    #[test]
    fn test_default_config() {
        let cfg = default_gaze_track_config();
        assert!(cfg.pursuit_speed > 0.0);
        assert!(cfg.max_yaw_deg > 0.0);
        assert!(cfg.max_pitch_deg > 0.0);
    }

    #[test]
    fn test_new_state_rest() {
        let state = make_state();
        assert_eq!(state.yaw_deg, 0.0);
        assert_eq!(state.pitch_deg, 0.0);
        assert!(!state.in_saccade);
    }

    #[test]
    fn test_set_gaze_target_screen_center() {
        let mut state = make_state();
        set_gaze_target_screen(&mut state, 0.0, 0.0);
        assert!((state.target_yaw_deg).abs() < 1e-5);
        assert!((state.target_pitch_deg).abs() < 1e-5);
    }

    #[test]
    fn test_set_gaze_target_screen_right() {
        let mut state = make_state();
        set_gaze_target_screen(&mut state, 1.0, 0.0);
        assert!(state.target_yaw_deg > 0.0);
    }

    #[test]
    fn test_set_gaze_target_screen_clamped() {
        let mut state = make_state();
        set_gaze_target_screen(&mut state, 5.0, 0.0);
        assert!(state.target_yaw_deg <= state.config.max_yaw_deg);
    }

    #[test]
    fn test_set_gaze_target_world() {
        let mut state = make_state();
        let target = GazePoint { x: 0.5, y: 0.0, z: -1.0, is_world_space: true };
        set_gaze_target_world(&mut state, &target);
        assert!(state.target_yaw_deg > 0.0); // looking right
        assert!(state.distance > 0.0);
    }

    #[test]
    fn test_update_gaze_tracking_approaches_target() {
        let mut state = make_state();
        state.target_yaw_deg = 20.0;
        update_gaze_tracking(&mut state, 0.1);
        assert!(state.yaw_deg > 0.0);
        assert!(state.yaw_deg <= 20.0);
    }

    #[test]
    fn test_update_gaze_reaches_target() {
        let mut state = make_state();
        state.target_yaw_deg = 5.0;
        // Run many steps - should converge
        for _ in 0..100 {
            update_gaze_tracking(&mut state, 0.05);
        }
        assert!((state.yaw_deg - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_gaze_yaw_deg() {
        let mut state = make_state();
        state.yaw_deg = 15.0;
        assert_eq!(gaze_yaw_deg(&state), 15.0);
    }

    #[test]
    fn test_gaze_pitch_deg() {
        let mut state = make_state();
        state.pitch_deg = -10.0;
        assert_eq!(gaze_pitch_deg(&state), -10.0);
    }

    #[test]
    fn test_gaze_track_distance() {
        let mut state = make_state();
        state.distance = 2.5;
        assert_eq!(gaze_track_distance(&state), 2.5);
    }

    #[test]
    fn test_gaze_velocity() {
        let mut state = make_state();
        state.velocity_deg_s = 50.0;
        assert_eq!(gaze_velocity(&state), 50.0);
    }

    #[test]
    fn test_is_fixating_true() {
        let state = make_state(); // velocity = 0
        assert!(is_fixating(&state));
    }

    #[test]
    fn test_is_fixating_false() {
        let mut state = make_state();
        state.velocity_deg_s = 100.0;
        assert!(!is_fixating(&state));
    }

    #[test]
    fn test_gaze_to_eye_morph_weights_center() {
        let state = make_state();
        let (lh, lv, rh, rv) = gaze_to_eye_morph_weights(&state);
        assert_eq!(lh, 0.0);
        assert_eq!(lv, 0.0);
        assert_eq!(rh, 0.0);
        assert_eq!(rv, 0.0);
    }

    #[test]
    fn test_gaze_to_eye_morph_weights_range() {
        let mut state = make_state();
        state.yaw_deg = state.config.max_yaw_deg;
        let (lh, _lv, _rh, _rv) = gaze_to_eye_morph_weights(&state);
        assert!((lh - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_reset_gaze_tracking() {
        let mut state = make_state();
        state.yaw_deg = 20.0;
        state.pitch_deg = -10.0;
        state.in_saccade = true;
        reset_gaze_tracking(&mut state);
        assert_eq!(state.yaw_deg, 0.0);
        assert_eq!(state.pitch_deg, 0.0);
        assert!(!state.in_saccade);
    }

    #[test]
    fn test_saccade_to_target() {
        let mut state = make_state();
        saccade_to_target(&mut state, 30.0, -10.0);
        assert!(state.in_saccade);
        assert!((state.target_yaw_deg - 30.0).abs() < 1e-5);
    }

    #[test]
    fn test_saccade_speed_faster() {
        let mut state = make_state();
        saccade_to_target(&mut state, 30.0, 0.0);
        update_gaze_tracking(&mut state, 0.05);
        let yaw_saccade = state.yaw_deg;

        let mut state2 = make_state();
        state2.target_yaw_deg = 30.0;
        update_gaze_tracking(&mut state2, 0.05);
        let yaw_pursuit = state2.yaw_deg;

        assert!(yaw_saccade >= yaw_pursuit);
    }
}
