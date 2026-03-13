// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Camera manipulation controls: orbit, pan, zoom, and fly modes.
//!
//! All control operations work on a [`CameraControlState`] value and
//! produce updated state without any mutable global state.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Operational mode of the camera controller.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraControlMode {
    /// Orbit around a fixed target point.
    Orbit,
    /// Pan the camera and target together.
    Pan,
    /// Free-fly first-person movement.
    Fly,
    /// Turntable rotation (yaw only, no pitch).
    Turntable,
}

/// Mutable camera control state (position, target, up, distance).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CameraControlState {
    /// Eye position in world space.
    pub position: [f32; 3],
    /// Point the camera orbits / looks at.
    pub target: [f32; 3],
    /// Camera up vector.
    pub up: [f32; 3],
    /// Distance between position and target (kept in sync).
    pub distance: f32,
    /// Active control mode.
    pub mode: CameraControlMode,
}

/// Tuning parameters for the camera controller.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraControlConfig {
    /// Sensitivity multiplier for orbit mouse movement.
    pub orbit_speed: f32,
    /// Sensitivity multiplier for pan movement.
    pub pan_speed: f32,
    /// Sensitivity multiplier for zoom.
    pub zoom_speed: f32,
    /// Sensitivity multiplier for fly movement.
    pub fly_speed: f32,
    /// Minimum allowed distance between position and target.
    pub min_distance: f32,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// A 4×4 column-major view matrix.
pub type ViewMatrix4 = [[f32; 4]; 4];

// ── Config helpers ────────────────────────────────────────────────────────────

/// Create a sensible default [`CameraControlConfig`].
#[allow(dead_code)]
pub fn default_camera_control_config() -> CameraControlConfig {
    CameraControlConfig {
        orbit_speed: 0.5,
        pan_speed: 0.01,
        zoom_speed: 0.1,
        fly_speed: 0.05,
        min_distance: 0.05,
    }
}

// ── State construction ────────────────────────────────────────────────────────

/// Create a new [`CameraControlState`] looking at `target` from `position`.
#[allow(dead_code)]
pub fn new_camera_control_state(
    position: [f32; 3],
    target: [f32; 3],
) -> CameraControlState {
    let dist = vec3_len(vec3_sub(position, target));
    CameraControlState {
        position,
        target,
        up: [0.0, 1.0, 0.0],
        distance: dist.max(0.05),
        mode: CameraControlMode::Orbit,
    }
}

// ── Mode control ──────────────────────────────────────────────────────────────

/// Change the active control mode.
#[allow(dead_code)]
pub fn set_control_mode(state: &mut CameraControlState, mode: CameraControlMode) {
    state.mode = mode;
}

/// Reset the camera to look at the origin from `[0, 1, -3]`.
#[allow(dead_code)]
pub fn reset_camera_view(state: &mut CameraControlState) {
    state.position = [0.0, 1.0, -3.0];
    state.target = [0.0, 0.0, 0.0];
    state.up = [0.0, 1.0, 0.0];
    state.distance = vec3_len(vec3_sub(state.position, state.target));
}

// ── Camera queries ────────────────────────────────────────────────────────────

/// Return the camera target position.
#[allow(dead_code)]
pub fn camera_target(state: &CameraControlState) -> [f32; 3] {
    state.target
}

/// Return the camera eye position.
#[allow(dead_code)]
pub fn camera_position(state: &CameraControlState) -> [f32; 3] {
    state.position
}

/// Return the camera up vector.
#[allow(dead_code)]
pub fn camera_up(state: &CameraControlState) -> [f32; 3] {
    state.up
}

/// Return the distance between the camera eye and its target.
#[allow(dead_code)]
pub fn camera_distance(state: &CameraControlState) -> f32 {
    state.distance
}

// ── Camera movement ───────────────────────────────────────────────────────────

/// Orbit the camera around its target by `yaw_deg` (horizontal) and
/// `pitch_deg` (vertical) in degrees.
#[allow(dead_code)]
pub fn orbit_camera(
    state: &mut CameraControlState,
    yaw_deg: f32,
    pitch_deg: f32,
    cfg: &CameraControlConfig,
) {
    let yaw = (yaw_deg * cfg.orbit_speed).to_radians();
    let pitch = (pitch_deg * cfg.orbit_speed).to_radians();

    let offset = vec3_sub(state.position, state.target);
    let dist = vec3_len(offset);
    if dist < 1e-6 {
        return;
    }

    let theta = f32::atan2(offset[0], offset[2]);
    let phi = f32::asin((offset[1] / dist).clamp(-1.0, 1.0));

    let new_theta = theta + yaw;
    let new_phi = (phi + pitch).clamp(
        -std::f32::consts::FRAC_PI_2 + 0.01,
        std::f32::consts::FRAC_PI_2 - 0.01,
    );

    state.position = [
        state.target[0] + dist * new_phi.cos() * new_theta.sin(),
        state.target[1] + dist * new_phi.sin(),
        state.target[2] + dist * new_phi.cos() * new_theta.cos(),
    ];
    state.distance = dist;
}

/// Pan the camera and its target together in the view-plane.
///
/// `delta` is `[dx, dy]` in screen-pixel units.
#[allow(dead_code)]
pub fn pan_camera(
    state: &mut CameraControlState,
    delta: [f32; 2],
    cfg: &CameraControlConfig,
) {
    let fwd = vec3_normalize(vec3_sub(state.target, state.position));
    let right = vec3_normalize(vec3_cross(fwd, state.up));
    let up = vec3_cross(right, fwd);

    let dx = delta[0] * cfg.pan_speed;
    let dy = delta[1] * cfg.pan_speed;

    let pan = [
        right[0] * dx + up[0] * dy,
        right[1] * dx + up[1] * dy,
        right[2] * dx + up[2] * dy,
    ];

    state.position = vec3_add(state.position, pan);
    state.target = vec3_add(state.target, pan);
}

/// Zoom by adjusting the distance between the camera eye and its target.
///
/// Positive `delta` zooms in (closer).
#[allow(dead_code)]
pub fn zoom_camera(
    state: &mut CameraControlState,
    delta: f32,
    cfg: &CameraControlConfig,
) {
    let offset = vec3_sub(state.position, state.target);
    let dist = vec3_len(offset).max(cfg.min_distance);
    let new_dist = (dist - delta * cfg.zoom_speed).max(cfg.min_distance);
    let dir = vec3_normalize(offset);
    state.position = vec3_add(state.target, vec3_scale(dir, new_dist));
    state.distance = new_dist;
}

/// Move the camera forward along its look direction (fly mode).
#[allow(dead_code)]
pub fn fly_camera_forward(
    state: &mut CameraControlState,
    amount: f32,
    cfg: &CameraControlConfig,
) {
    let fwd = vec3_normalize(vec3_sub(state.target, state.position));
    let delta = vec3_scale(fwd, amount * cfg.fly_speed);
    state.position = vec3_add(state.position, delta);
    state.target = vec3_add(state.target, delta);
    state.distance = vec3_len(vec3_sub(state.position, state.target));
}

/// Apply a 2-D mouse delta to the camera according to the active control mode.
#[allow(dead_code)]
pub fn apply_mouse_delta(
    state: &mut CameraControlState,
    delta: [f32; 2],
    cfg: &CameraControlConfig,
) {
    match state.mode {
        CameraControlMode::Orbit => orbit_camera(state, delta[0], delta[1], cfg),
        CameraControlMode::Turntable => orbit_camera(state, delta[0], 0.0, cfg),
        CameraControlMode::Pan => pan_camera(state, delta, cfg),
        CameraControlMode::Fly => {
            orbit_camera(state, delta[0], delta[1], cfg);
        }
    }
}

// ── View matrix ───────────────────────────────────────────────────────────────

/// Compute a column-major look-at view matrix from the current state.
#[allow(dead_code)]
pub fn camera_view_matrix(state: &CameraControlState) -> ViewMatrix4 {
    let e = state.position;
    let t = state.target;
    let u = state.up;

    let fwd = vec3_normalize(vec3_sub(t, e));
    let right = vec3_normalize(vec3_cross(fwd, u));
    let up = vec3_cross(right, fwd);

    let tx = -vec3_dot(right, e);
    let ty = -vec3_dot(up, e);
    let tz = vec3_dot(fwd, e);

    [
        [right[0], up[0], -fwd[0], 0.0],
        [right[1], up[1], -fwd[1], 0.0],
        [right[2], up[2], -fwd[2], 0.0],
        [tx, ty, tz, 1.0],
    ]
}

// ── Private vector math ───────────────────────────────────────────────────────

#[inline]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn vec3_scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn vec3_len(a: [f32; 3]) -> f32 {
    vec3_dot(a, a).sqrt()
}

#[inline]
fn vec3_normalize(a: [f32; 3]) -> [f32; 3] {
    let l = vec3_len(a);
    if l < 1e-9 {
        [0.0, 0.0, 0.0]
    } else {
        vec3_scale(a, 1.0 / l)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> CameraControlState {
        new_camera_control_state([0.0, 1.0, -3.0], [0.0, 0.0, 0.0])
    }

    fn make_cfg() -> CameraControlConfig {
        default_camera_control_config()
    }

    #[test]
    fn default_config_orbit_speed_positive() {
        let cfg = make_cfg();
        assert!(cfg.orbit_speed > 0.0);
    }

    #[test]
    fn new_state_position() {
        let s = make_state();
        assert_eq!(s.position, [0.0, 1.0, -3.0]);
        assert_eq!(s.target, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn new_state_distance_positive() {
        let s = make_state();
        assert!(s.distance > 0.0);
    }

    #[test]
    fn set_control_mode_changes_mode() {
        let mut s = make_state();
        set_control_mode(&mut s, CameraControlMode::Pan);
        assert_eq!(s.mode, CameraControlMode::Pan);
    }

    #[test]
    fn reset_camera_view_restores_defaults() {
        let mut s = make_state();
        orbit_camera(&mut s, 90.0, 45.0, &make_cfg());
        reset_camera_view(&mut s);
        assert_eq!(s.position, [0.0, 1.0, -3.0]);
        assert_eq!(s.target, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn camera_target_accessor() {
        let s = make_state();
        assert_eq!(camera_target(&s), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn camera_position_accessor() {
        let s = make_state();
        assert_eq!(camera_position(&s), [0.0, 1.0, -3.0]);
    }

    #[test]
    fn camera_up_accessor() {
        let s = make_state();
        assert_eq!(camera_up(&s), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn camera_distance_matches_euclidean() {
        let s = make_state();
        let expected = vec3_len(vec3_sub(s.position, s.target));
        assert!((camera_distance(&s) - expected).abs() < 1e-4);
    }

    #[test]
    fn orbit_camera_moves_position() {
        let mut s = make_state();
        let before = s.position;
        orbit_camera(&mut s, 45.0, 0.0, &make_cfg());
        assert_ne!(s.position, before);
    }

    #[test]
    fn orbit_camera_preserves_distance() {
        let mut s = make_state();
        let before = camera_distance(&s);
        orbit_camera(&mut s, 90.0, 20.0, &make_cfg());
        let after = vec3_len(vec3_sub(s.position, s.target));
        assert!((before - after).abs() < 1e-2);
    }

    #[test]
    fn pan_camera_moves_both_position_and_target() {
        let mut s = make_state();
        let tgt_before = s.target;
        let pos_before = s.position;
        pan_camera(&mut s, [10.0, 0.0], &make_cfg());
        assert_ne!(s.target, tgt_before);
        assert_ne!(s.position, pos_before);
    }

    #[test]
    fn pan_camera_preserves_relative_offset() {
        let mut s = make_state();
        let offset_before = vec3_sub(s.position, s.target);
        pan_camera(&mut s, [5.0, 3.0], &make_cfg());
        let offset_after = vec3_sub(s.position, s.target);
        for i in 0..3 {
            assert!((offset_before[i] - offset_after[i]).abs() < 1e-4);
        }
    }

    #[test]
    fn zoom_camera_reduces_distance() {
        let mut s = make_state();
        let before = camera_distance(&s);
        zoom_camera(&mut s, 1.0, &make_cfg());
        assert!(camera_distance(&s) < before);
    }

    #[test]
    fn zoom_camera_clamps_min_distance() {
        let mut s = make_state();
        let cfg = make_cfg();
        zoom_camera(&mut s, 10000.0, &cfg);
        assert!(camera_distance(&s) >= cfg.min_distance);
    }

    #[test]
    fn fly_camera_forward_moves_both() {
        let mut s = make_state();
        let pos_before = s.position;
        fly_camera_forward(&mut s, 10.0, &make_cfg());
        assert_ne!(s.position, pos_before);
    }

    #[test]
    fn apply_mouse_delta_orbit_mode() {
        let mut s = make_state();
        set_control_mode(&mut s, CameraControlMode::Orbit);
        let before = s.position;
        apply_mouse_delta(&mut s, [30.0, 0.0], &make_cfg());
        assert_ne!(s.position, before);
    }

    #[test]
    fn camera_view_matrix_homogeneous_element() {
        let s = make_state();
        let m = camera_view_matrix(&s);
        assert!((m[3][3] - 1.0).abs() < 1e-5, "m[3][3] should be 1");
    }

    #[test]
    fn camera_view_matrix_is_4x4() {
        let s = make_state();
        let m = camera_view_matrix(&s);
        assert_eq!(m.len(), 4);
        for col in &m {
            assert_eq!(col.len(), 4);
        }
    }
}
