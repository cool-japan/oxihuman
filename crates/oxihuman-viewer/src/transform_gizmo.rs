// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Transform gizmo widget: translate / rotate / scale handles.
//!
//! Provides ray-vs-handle hit testing, drag delta computation, and
//! JSON serialisation with no external dependencies.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Operation mode of the transform gizmo.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TgGizmoMode {
    Translate,
    Rotate,
    Scale,
    Universal,
}

/// Axis (or axis-plane) of a transform gizmo handle.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TgGizmoAxis {
    X,
    Y,
    Z,
    XY,
    XZ,
    YZ,
    All,
}

/// Runtime state of a transform gizmo instance.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TgGizmoState {
    pub mode: TgGizmoMode,
    /// World-space origin of the gizmo.
    pub position: [f32; 3],
    /// Orientation quaternion `[x, y, z, w]`.
    pub rotation: [f32; 4],
    /// Scale per axis.
    pub scale: [f32; 3],
    /// Which axis / plane is currently being dragged (`None` = idle).
    pub active_axis: Option<TgGizmoAxis>,
    /// Whether the gizmo is rendered.
    pub visible: bool,
    /// Screen-space size multiplier.
    pub size: f32,
    /// Snap increment for translation (0.0 = disabled).
    pub snap_translate: f32,
    /// Snap increment for rotation in degrees (0.0 = disabled).
    pub snap_rotate_deg: f32,
    /// Snap increment for scale (0.0 = disabled).
    pub snap_scale: f32,
}

/// Configuration / tuning for a transform gizmo.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TgGizmoConfig {
    /// Screen-space size of handles.
    pub size: f32,
    /// Hit-test radius as a fraction of `size`.
    pub hit_radius_fraction: f32,
    /// Default snap increment for translation.
    pub default_snap_translate: f32,
    /// Default snap increment for rotation in degrees.
    pub default_snap_rotate_deg: f32,
    /// Default snap increment for scale.
    pub default_snap_scale: f32,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// A handle (line segment) of the gizmo used for rendering and hit-testing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TgGizmoHandle {
    pub axis: TgGizmoAxis,
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    pub length: f32,
    pub color: [f32; 4],
}

/// The delta transform produced by a single drag operation.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct TgDragDelta {
    pub delta_position: [f32; 3],
    /// Delta rotation quaternion `[x, y, z, w]`.
    pub delta_rotation: [f32; 4],
    pub delta_scale: [f32; 3],
    pub axis: TgGizmoAxis,
}

// ── Config helpers ────────────────────────────────────────────────────────────

/// Create a sensible default [`TgGizmoConfig`].
#[allow(dead_code)]
pub fn default_gizmo_config() -> TgGizmoConfig {
    TgGizmoConfig {
        size: 1.0,
        hit_radius_fraction: 0.1,
        default_snap_translate: 0.0,
        default_snap_rotate_deg: 0.0,
        default_snap_scale: 0.0,
    }
}

// ── State construction ────────────────────────────────────────────────────────

/// Create a new [`TgGizmoState`] at `position` in [`TgGizmoMode::Translate`].
#[allow(dead_code)]
pub fn new_gizmo_state(position: [f32; 3], cfg: &TgGizmoConfig) -> TgGizmoState {
    TgGizmoState {
        mode: TgGizmoMode::Translate,
        position,
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
        active_axis: None,
        visible: true,
        size: cfg.size,
        snap_translate: cfg.default_snap_translate,
        snap_rotate_deg: cfg.default_snap_rotate_deg,
        snap_scale: cfg.default_snap_scale,
    }
}

// ── State manipulation ────────────────────────────────────────────────────────

/// Change the gizmo operation mode.
#[allow(dead_code)]
pub fn set_gizmo_mode(state: &mut TgGizmoState, mode: TgGizmoMode) {
    state.mode = mode;
    state.active_axis = None;
}

/// Move the gizmo to a new world-space position.
#[allow(dead_code)]
pub fn set_gizmo_position(state: &mut TgGizmoState, position: [f32; 3]) {
    state.position = position;
}

/// Reset the gizmo transform to identity (zero translation, unit quaternion, unit scale).
#[allow(dead_code)]
pub fn reset_gizmo(state: &mut TgGizmoState) {
    state.position = [0.0, 0.0, 0.0];
    state.rotation = [0.0, 0.0, 0.0, 1.0];
    state.scale = [1.0, 1.0, 1.0];
    state.active_axis = None;
}

// ── Queries ───────────────────────────────────────────────────────────────────

/// Return the currently active axis (if any drag is in progress).
#[allow(dead_code)]
pub fn active_axis(state: &TgGizmoState) -> Option<TgGizmoAxis> {
    state.active_axis
}

/// Return the screen-space size of the gizmo.
#[allow(dead_code)]
pub fn gizmo_size(state: &TgGizmoState) -> f32 {
    state.size
}

/// Return `true` if the gizmo has an active drag in progress.
#[allow(dead_code)]
pub fn is_gizmo_active(state: &TgGizmoState) -> bool {
    state.active_axis.is_some()
}

// ── Handle geometry ───────────────────────────────────────────────────────────

/// Return the world-space positions of the gizmo's axis-arrow tips.
///
/// Returns `[(axis, tip_position)]` for each visible handle.
#[allow(dead_code)]
pub fn gizmo_handle_positions(state: &TgGizmoState) -> Vec<(TgGizmoAxis, [f32; 3])> {
    let pos = state.position;
    let len = state.size;
    let mut out = vec![
        (TgGizmoAxis::X, [pos[0] + len, pos[1], pos[2]]),
        (TgGizmoAxis::Y, [pos[0], pos[1] + len, pos[2]]),
        (TgGizmoAxis::Z, [pos[0], pos[1], pos[2] + len]),
    ];
    if state.mode == TgGizmoMode::Universal {
        let p = len * 0.4;
        out.push((TgGizmoAxis::XY, [pos[0] + p, pos[1] + p, pos[2]]));
        out.push((TgGizmoAxis::XZ, [pos[0] + p, pos[1], pos[2] + p]));
        out.push((TgGizmoAxis::YZ, [pos[0], pos[1] + p, pos[2] + p]));
        out.push((TgGizmoAxis::All, [pos[0] + p, pos[1] + p, pos[2] + p]));
    }
    out
}

/// Build the set of [`TgGizmoHandle`]s for the current gizmo state.
#[allow(dead_code)]
pub fn gizmo_handles(state: &TgGizmoState) -> Vec<TgGizmoHandle> {
    let pos = state.position;
    let len = state.size;
    let mut handles = vec![
        TgGizmoHandle {
            axis: TgGizmoAxis::X,
            origin: pos,
            direction: [1.0, 0.0, 0.0],
            length: len,
            color: [1.0, 0.0, 0.0, 1.0],
        },
        TgGizmoHandle {
            axis: TgGizmoAxis::Y,
            origin: pos,
            direction: [0.0, 1.0, 0.0],
            length: len,
            color: [0.0, 1.0, 0.0, 1.0],
        },
        TgGizmoHandle {
            axis: TgGizmoAxis::Z,
            origin: pos,
            direction: [0.0, 0.0, 1.0],
            length: len,
            color: [0.0, 0.0, 1.0, 1.0],
        },
    ];
    if state.mode == TgGizmoMode::Universal {
        let p = len * 0.4;
        handles.push(TgGizmoHandle {
            axis: TgGizmoAxis::All,
            origin: pos,
            direction: [1.0, 1.0, 1.0],
            length: p,
            color: [1.0, 1.0, 1.0, 1.0],
        });
    }
    handles
}

// ── Hit testing ───────────────────────────────────────────────────────────────

/// Test a world-space ray against the gizmo handles.
///
/// Returns the [`TgGizmoAxis`] of the closest handle within `cfg.hit_radius_fraction * size`,
/// or `None` if no handle is hit.
#[allow(dead_code)]
pub fn hit_test_gizmo(
    state: &TgGizmoState,
    ray_origin: [f32; 3],
    ray_dir: [f32; 3],
    cfg: &TgGizmoConfig,
) -> Option<TgGizmoAxis> {
    let handles = gizmo_handles(state);
    let threshold = state.size * cfg.hit_radius_fraction;
    handles
        .iter()
        .filter_map(|h| {
            let d = ray_to_segment_dist(ray_origin, ray_dir, h.origin, h.direction, h.length);
            if d < threshold { Some((h.axis, d)) } else { None }
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(axis, _)| axis)
}

// ── Drag application ──────────────────────────────────────────────────────────

/// Compute the [`TgDragDelta`] for a 2-D mouse drag and apply it to the gizmo.
///
/// `screen_delta` is `[dx, dy]` in pixels.
#[allow(dead_code)]
pub fn apply_gizmo_drag(
    state: &mut TgGizmoState,
    axis: TgGizmoAxis,
    screen_delta: [f32; 2],
) -> TgDragDelta {
    state.active_axis = Some(axis);

    let magnitude = (screen_delta[0] * screen_delta[0] + screen_delta[1] * screen_delta[1]).sqrt();
    let sign = if screen_delta[0] + screen_delta[1] >= 0.0 { 1.0_f32 } else { -1.0_f32 };
    let amount = magnitude * sign * 0.01;

    match state.mode {
        TgGizmoMode::Translate => {
            let dp = translate_delta(axis, amount);
            let snapped = if state.snap_translate > 0.0 {
                apply_snap3(dp, state.snap_translate)
            } else {
                dp
            };
            state.position[0] += snapped[0];
            state.position[1] += snapped[1];
            state.position[2] += snapped[2];
            TgDragDelta {
                delta_position: snapped,
                delta_rotation: [0.0, 0.0, 0.0, 1.0],
                delta_scale: [1.0, 1.0, 1.0],
                axis,
            }
        }
        TgGizmoMode::Rotate => {
            let angle = if state.snap_rotate_deg > 0.0 {
                let snap_rad = state.snap_rotate_deg.to_radians();
                (amount / snap_rad).round() * snap_rad
            } else {
                amount
            };
            let dq = axis_angle_quat(axis, angle);
            let q = state.rotation;
            state.rotation = quat_mul(q, dq);
            TgDragDelta {
                delta_position: [0.0; 3],
                delta_rotation: dq,
                delta_scale: [1.0, 1.0, 1.0],
                axis,
            }
        }
        TgGizmoMode::Scale | TgGizmoMode::Universal => {
            let raw = 1.0 + amount;
            let factor = if state.snap_scale > 0.0 {
                ((raw / state.snap_scale).round() * state.snap_scale).max(0.001)
            } else {
                raw.max(0.001)
            };
            let ds = scale_delta(axis, factor);
            state.scale[0] *= ds[0];
            state.scale[1] *= ds[1];
            state.scale[2] *= ds[2];
            TgDragDelta {
                delta_position: [0.0; 3],
                delta_rotation: [0.0, 0.0, 0.0, 1.0],
                delta_scale: ds,
                axis,
            }
        }
    }
}

// ── Snap ──────────────────────────────────────────────────────────────────────

/// Snap each component of a translation delta to the nearest multiple of `grid`.
///
/// Disabled (no-op) when `grid <= 0.0`.
#[allow(dead_code)]
pub fn gizmo_snap_to_grid(delta: [f32; 3], grid: f32) -> [f32; 3] {
    if grid <= 0.0 {
        return delta;
    }
    apply_snap3(delta, grid)
}

// ── JSON serialisation ────────────────────────────────────────────────────────

/// Serialise the gizmo state to a compact JSON string.
#[allow(dead_code)]
pub fn gizmo_to_json(state: &TgGizmoState) -> String {
    let mode = match state.mode {
        TgGizmoMode::Translate => "translate",
        TgGizmoMode::Rotate => "rotate",
        TgGizmoMode::Scale => "scale",
        TgGizmoMode::Universal => "universal",
    };
    let p = state.position;
    let r = state.rotation;
    let s = state.scale;
    format!(
        r#"{{"mode":"{mode}","position":[{:.6},{:.6},{:.6}],"rotation":[{:.6},{:.6},{:.6},{:.6}],"scale":[{:.6},{:.6},{:.6}],"visible":{},"size":{:.6}}}"#,
        p[0], p[1], p[2],
        r[0], r[1], r[2], r[3],
        s[0], s[1], s[2],
        state.visible,
        state.size,
    )
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn translate_delta(axis: TgGizmoAxis, amount: f32) -> [f32; 3] {
    match axis {
        TgGizmoAxis::X => [amount, 0.0, 0.0],
        TgGizmoAxis::Y => [0.0, amount, 0.0],
        TgGizmoAxis::Z => [0.0, 0.0, amount],
        TgGizmoAxis::XY => [amount, amount, 0.0],
        TgGizmoAxis::XZ => [amount, 0.0, amount],
        TgGizmoAxis::YZ => [0.0, amount, amount],
        TgGizmoAxis::All => [amount, amount, amount],
    }
}

fn scale_delta(axis: TgGizmoAxis, factor: f32) -> [f32; 3] {
    match axis {
        TgGizmoAxis::X => [factor, 1.0, 1.0],
        TgGizmoAxis::Y => [1.0, factor, 1.0],
        TgGizmoAxis::Z => [1.0, 1.0, factor],
        TgGizmoAxis::XY => [factor, factor, 1.0],
        TgGizmoAxis::XZ => [factor, 1.0, factor],
        TgGizmoAxis::YZ => [1.0, factor, factor],
        TgGizmoAxis::All => [factor, factor, factor],
    }
}

fn axis_angle_quat(axis: TgGizmoAxis, angle: f32) -> [f32; 4] {
    let half = angle * 0.5;
    let s = half.sin();
    let c = half.cos();
    match axis {
        TgGizmoAxis::X => [s, 0.0, 0.0, c],
        TgGizmoAxis::Y => [0.0, s, 0.0, c],
        TgGizmoAxis::Z => [0.0, 0.0, s, c],
        _ => [0.0, 0.0, 0.0, 1.0],
    }
}

fn quat_mul(q: [f32; 4], r: [f32; 4]) -> [f32; 4] {
    [
        q[3] * r[0] + q[0] * r[3] + q[1] * r[2] - q[2] * r[1],
        q[3] * r[1] - q[0] * r[2] + q[1] * r[3] + q[2] * r[0],
        q[3] * r[2] + q[0] * r[1] - q[1] * r[0] + q[2] * r[3],
        q[3] * r[3] - q[0] * r[0] - q[1] * r[1] - q[2] * r[2],
    ]
}

fn apply_snap3(v: [f32; 3], snap: f32) -> [f32; 3] {
    [
        (v[0] / snap).round() * snap,
        (v[1] / snap).round() * snap,
        (v[2] / snap).round() * snap,
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn ray_to_segment_dist(
    ray_o: [f32; 3],
    ray_d: [f32; 3],
    seg_o: [f32; 3],
    seg_d: [f32; 3],
    seg_len: f32,
) -> f32 {
    let w = [
        ray_o[0] - seg_o[0],
        ray_o[1] - seg_o[1],
        ray_o[2] - seg_o[2],
    ];
    let a = dot3(ray_d, ray_d);
    let b = dot3(ray_d, seg_d);
    let c = dot3(seg_d, seg_d);
    let d = dot3(ray_d, w);
    let e = dot3(seg_d, w);
    let denom = a * c - b * b;
    let (sc, tc) = if denom.abs() < 1e-7 {
        (0.0f32, (e / c.max(1e-9)).clamp(0.0, seg_len))
    } else {
        let sn = b * e - c * d;
        let tn = a * e - b * d;
        (sn / denom, (tn / denom).clamp(0.0, seg_len))
    };
    let closest = [
        w[0] + sc * ray_d[0] - tc * seg_d[0],
        w[1] + sc * ray_d[1] - tc * seg_d[1],
        w[2] + sc * ray_d[2] - tc * seg_d[2],
    ];
    dot3(closest, closest).sqrt()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cfg() -> TgGizmoConfig {
        default_gizmo_config()
    }

    fn make_state() -> TgGizmoState {
        new_gizmo_state([0.0; 3], &make_cfg())
    }

    #[test]
    fn default_config_size_one() {
        let cfg = make_cfg();
        assert!((cfg.size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_gizmo_state_translate_mode() {
        let s = make_state();
        assert_eq!(s.mode, TgGizmoMode::Translate);
    }

    #[test]
    fn new_gizmo_state_position() {
        let s = new_gizmo_state([1.0, 2.0, 3.0], &make_cfg());
        assert_eq!(s.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn new_gizmo_state_unit_rotation() {
        let s = make_state();
        assert_eq!(s.rotation, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn set_gizmo_mode_changes_mode() {
        let mut s = make_state();
        set_gizmo_mode(&mut s, TgGizmoMode::Rotate);
        assert_eq!(s.mode, TgGizmoMode::Rotate);
    }

    #[test]
    fn set_gizmo_mode_clears_active_axis() {
        let mut s = make_state();
        s.active_axis = Some(TgGizmoAxis::X);
        set_gizmo_mode(&mut s, TgGizmoMode::Scale);
        assert!(s.active_axis.is_none());
    }

    #[test]
    fn set_gizmo_position_updates() {
        let mut s = make_state();
        set_gizmo_position(&mut s, [5.0, 6.0, 7.0]);
        assert_eq!(s.position, [5.0, 6.0, 7.0]);
    }

    #[test]
    fn reset_gizmo_clears_transform() {
        let mut s = new_gizmo_state([3.0, 3.0, 3.0], &make_cfg());
        apply_gizmo_drag(&mut s, TgGizmoAxis::X, [100.0, 0.0]);
        reset_gizmo(&mut s);
        assert_eq!(s.position, [0.0; 3]);
        assert_eq!(s.rotation, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(s.scale, [1.0; 3]);
    }

    #[test]
    fn active_axis_none_initially() {
        let s = make_state();
        assert!(active_axis(&s).is_none());
    }

    #[test]
    fn is_gizmo_active_false_initially() {
        assert!(!is_gizmo_active(&make_state()));
    }

    #[test]
    fn is_gizmo_active_true_after_drag() {
        let mut s = make_state();
        apply_gizmo_drag(&mut s, TgGizmoAxis::Y, [10.0, 0.0]);
        assert!(is_gizmo_active(&s));
    }

    #[test]
    fn gizmo_size_accessor() {
        let s = make_state();
        assert!((gizmo_size(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn gizmo_handle_positions_nonempty() {
        let s = make_state();
        assert!(!gizmo_handle_positions(&s).is_empty());
    }

    #[test]
    fn gizmo_handle_positions_universal_has_more() {
        let mut s = make_state();
        set_gizmo_mode(&mut s, TgGizmoMode::Universal);
        assert!(gizmo_handle_positions(&s).len() > 3);
    }

    #[test]
    fn hit_test_gizmo_on_x_axis() {
        let s = make_state();
        let cfg = make_cfg();
        // Ray aimed at the tip of the X handle [1.0, 0.0, 0.0] from far on Z.
        // Direction toward (0.5, 0, 0) from (0.5, 0, -10): pure Z.
        let result = hit_test_gizmo(&s, [0.5, 0.0, -10.0], [0.0, 0.0, 1.0], &cfg);
        assert_eq!(result, Some(TgGizmoAxis::X));
    }

    #[test]
    fn hit_test_gizmo_no_hit() {
        let s = make_state();
        let cfg = make_cfg();
        // Ray far from all handles
        let result = hit_test_gizmo(&s, [100.0, 100.0, 100.0], [0.0, 0.0, 1.0], &cfg);
        assert!(result.is_none());
    }

    #[test]
    fn apply_gizmo_drag_translate_moves_position() {
        let mut s = make_state();
        apply_gizmo_drag(&mut s, TgGizmoAxis::X, [100.0, 0.0]);
        assert!(s.position[0].abs() > 0.0);
    }

    #[test]
    fn apply_gizmo_drag_rotate_changes_rotation() {
        let mut s = make_state();
        set_gizmo_mode(&mut s, TgGizmoMode::Rotate);
        let before = s.rotation;
        apply_gizmo_drag(&mut s, TgGizmoAxis::Y, [50.0, 0.0]);
        assert_ne!(s.rotation, before);
    }

    #[test]
    fn apply_gizmo_drag_scale_changes_scale() {
        let mut s = make_state();
        set_gizmo_mode(&mut s, TgGizmoMode::Scale);
        apply_gizmo_drag(&mut s, TgGizmoAxis::All, [50.0, 0.0]);
        assert!(s.scale[0] != 1.0 || s.scale[1] != 1.0 || s.scale[2] != 1.0);
    }

    #[test]
    fn gizmo_snap_to_grid_snaps() {
        let result = gizmo_snap_to_grid([1.3, 2.7, 0.1], 1.0);
        assert!((result[0] - 1.0).abs() < 1e-5);
        assert!((result[1] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn gizmo_snap_to_grid_disabled() {
        let v = [1.3f32, 2.7, 0.1];
        assert_eq!(gizmo_snap_to_grid(v, 0.0), v);
    }

    #[test]
    fn gizmo_to_json_contains_mode() {
        let s = make_state();
        let json = gizmo_to_json(&s);
        assert!(json.contains("translate"));
        assert!(json.contains("position"));
    }
}
