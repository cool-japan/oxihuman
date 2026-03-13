// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Transform gizmos (translate/rotate/scale handles) for 3D viewport.

// ── Types ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GizmoMode {
    Translate,
    Rotate,
    Scale,
    Universal,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
    XY,
    YZ,
    XZ,
    All,
}

#[allow(dead_code)]
pub struct GizmoState {
    pub mode: GizmoMode,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub active_axis: Option<GizmoAxis>,
    pub visible: bool,
    pub size: f32,
}

#[allow(dead_code)]
pub struct GizmoDragResult {
    pub delta_position: [f32; 3],
    pub delta_rotation: [f32; 4],
    pub delta_scale: [f32; 3],
    pub axis: GizmoAxis,
}

#[allow(dead_code)]
pub struct GizmoHandle {
    pub axis: GizmoAxis,
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    pub length: f32,
    pub color: [f32; 4],
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn new_gizmo(mode: GizmoMode, position: [f32; 3]) -> GizmoState {
    GizmoState {
        mode,
        position,
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
        active_axis: None,
        visible: true,
        size: 1.0,
    }
}

#[allow(dead_code)]
pub fn gizmo_handles(gizmo: &GizmoState) -> Vec<GizmoHandle> {
    let pos = gizmo.position;
    let len = gizmo.size;
    match gizmo.mode {
        GizmoMode::Translate | GizmoMode::Scale => vec![
            GizmoHandle {
                axis: GizmoAxis::X,
                origin: pos,
                direction: [1.0, 0.0, 0.0],
                length: len,
                color: [1.0, 0.0, 0.0, 1.0],
            },
            GizmoHandle {
                axis: GizmoAxis::Y,
                origin: pos,
                direction: [0.0, 1.0, 0.0],
                length: len,
                color: [0.0, 1.0, 0.0, 1.0],
            },
            GizmoHandle {
                axis: GizmoAxis::Z,
                origin: pos,
                direction: [0.0, 0.0, 1.0],
                length: len,
                color: [0.0, 0.0, 1.0, 1.0],
            },
        ],
        GizmoMode::Rotate => vec![
            GizmoHandle {
                axis: GizmoAxis::X,
                origin: pos,
                direction: [1.0, 0.0, 0.0],
                length: len,
                color: [1.0, 0.0, 0.0, 1.0],
            },
            GizmoHandle {
                axis: GizmoAxis::Y,
                origin: pos,
                direction: [0.0, 1.0, 0.0],
                length: len,
                color: [0.0, 1.0, 0.0, 1.0],
            },
            GizmoHandle {
                axis: GizmoAxis::Z,
                origin: pos,
                direction: [0.0, 0.0, 1.0],
                length: len,
                color: [0.0, 0.0, 1.0, 1.0],
            },
        ],
        GizmoMode::Universal => vec![
            GizmoHandle {
                axis: GizmoAxis::X,
                origin: pos,
                direction: [1.0, 0.0, 0.0],
                length: len,
                color: [1.0, 0.0, 0.0, 1.0],
            },
            GizmoHandle {
                axis: GizmoAxis::Y,
                origin: pos,
                direction: [0.0, 1.0, 0.0],
                length: len,
                color: [0.0, 1.0, 0.0, 1.0],
            },
            GizmoHandle {
                axis: GizmoAxis::Z,
                origin: pos,
                direction: [0.0, 0.0, 1.0],
                length: len,
                color: [0.0, 0.0, 1.0, 1.0],
            },
            GizmoHandle {
                axis: GizmoAxis::All,
                origin: pos,
                direction: [1.0, 1.0, 1.0],
                length: len * 0.3,
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ],
    }
}

#[allow(dead_code)]
pub fn translate_gizmo(gizmo: &mut GizmoState, delta: [f32; 3]) {
    gizmo.position[0] += delta[0];
    gizmo.position[1] += delta[1];
    gizmo.position[2] += delta[2];
}

#[allow(dead_code)]
pub fn rotate_gizmo(gizmo: &mut GizmoState, axis: GizmoAxis, angle_rad: f32) {
    let half = angle_rad * 0.5;
    let s = half.sin();
    let c = half.cos();
    let (ax, ay, az) = match axis {
        GizmoAxis::X => (1.0f32, 0.0, 0.0),
        GizmoAxis::Y => (0.0, 1.0, 0.0),
        GizmoAxis::Z => (0.0, 0.0, 1.0),
        _ => (0.0, 1.0, 0.0),
    };
    let dq = [ax * s, ay * s, az * s, c];
    // Multiply existing rotation by delta quaternion
    let q = gizmo.rotation;
    gizmo.rotation = [
        q[3] * dq[0] + q[0] * dq[3] + q[1] * dq[2] - q[2] * dq[1],
        q[3] * dq[1] - q[0] * dq[2] + q[1] * dq[3] + q[2] * dq[0],
        q[3] * dq[2] + q[0] * dq[1] - q[1] * dq[0] + q[2] * dq[3],
        q[3] * dq[3] - q[0] * dq[0] - q[1] * dq[1] - q[2] * dq[2],
    ];
}

#[allow(dead_code)]
pub fn scale_gizmo(gizmo: &mut GizmoState, axis: GizmoAxis, factor: f32) {
    match axis {
        GizmoAxis::X => gizmo.scale[0] *= factor,
        GizmoAxis::Y => gizmo.scale[1] *= factor,
        GizmoAxis::Z => gizmo.scale[2] *= factor,
        GizmoAxis::All => {
            gizmo.scale[0] *= factor;
            gizmo.scale[1] *= factor;
            gizmo.scale[2] *= factor;
        }
        GizmoAxis::XY => {
            gizmo.scale[0] *= factor;
            gizmo.scale[1] *= factor;
        }
        GizmoAxis::YZ => {
            gizmo.scale[1] *= factor;
            gizmo.scale[2] *= factor;
        }
        GizmoAxis::XZ => {
            gizmo.scale[0] *= factor;
            gizmo.scale[2] *= factor;
        }
    }
}

#[allow(dead_code)]
pub fn pick_gizmo_axis(
    gizmo: &GizmoState,
    ray_origin: [f32; 3],
    ray_dir: [f32; 3],
) -> Option<GizmoAxis> {
    let handles = gizmo_handles(gizmo);
    let threshold = gizmo.size * 0.1;
    for handle in &handles {
        // Closest distance from ray to handle line segment
        let d = point_to_segment_dist(
            ray_origin,
            ray_dir,
            handle.origin,
            handle.direction,
            handle.length,
        );
        if d < threshold {
            return Some(handle.axis);
        }
    }
    None
}

fn point_to_segment_dist(
    ray_o: [f32; 3],
    ray_d: [f32; 3],
    seg_o: [f32; 3],
    seg_d: [f32; 3],
    seg_len: f32,
) -> f32 {
    // Distance from ray to segment (approximate)
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
        (0.0f32, (e / c).clamp(0.0, seg_len))
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
    (dot3(closest, closest)).sqrt()
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn drag_gizmo(
    gizmo: &mut GizmoState,
    axis: GizmoAxis,
    drag_delta: [f32; 2],
    _view_matrix: &[[f32; 4]; 4],
) -> GizmoDragResult {
    let magnitude = (drag_delta[0] * drag_delta[0] + drag_delta[1] * drag_delta[1]).sqrt();
    let sign = if drag_delta[0] + drag_delta[1] >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let amount = magnitude * sign * 0.01;

    let delta_position = match axis {
        GizmoAxis::X => [amount, 0.0, 0.0],
        GizmoAxis::Y => [0.0, amount, 0.0],
        GizmoAxis::Z => [0.0, 0.0, amount],
        GizmoAxis::All => [amount, amount, amount],
        _ => [0.0; 3],
    };
    translate_gizmo(gizmo, delta_position);

    GizmoDragResult {
        delta_position,
        delta_rotation: [0.0, 0.0, 0.0, 1.0],
        delta_scale: [1.0, 1.0, 1.0],
        axis,
    }
}

#[allow(dead_code)]
pub fn snap_translate(delta: [f32; 3], snap: f32) -> [f32; 3] {
    if snap <= 0.0 {
        return delta;
    }
    [
        (delta[0] / snap).round() * snap,
        (delta[1] / snap).round() * snap,
        (delta[2] / snap).round() * snap,
    ]
}

#[allow(dead_code)]
pub fn snap_rotate(angle_rad: f32, snap_deg: f32) -> f32 {
    if snap_deg <= 0.0 {
        return angle_rad;
    }
    let snap_rad = snap_deg.to_radians();
    (angle_rad / snap_rad).round() * snap_rad
}

#[allow(dead_code)]
pub fn snap_scale(factor: f32, snap: f32) -> f32 {
    if snap <= 0.0 {
        return factor;
    }
    (factor / snap).round() * snap
}

#[allow(dead_code)]
pub fn gizmo_world_matrix(gizmo: &GizmoState) -> [[f32; 4]; 4] {
    let [qx, qy, qz, qw] = gizmo.rotation;
    let [sx, sy, sz] = gizmo.scale;
    let [tx, ty, tz] = gizmo.position;

    // Rotation matrix from quaternion
    let r00 = (1.0 - 2.0 * (qy * qy + qz * qz)) * sx;
    let r10 = 2.0 * (qx * qy + qz * qw) * sx;
    let r20 = 2.0 * (qx * qz - qy * qw) * sx;
    let r01 = 2.0 * (qx * qy - qz * qw) * sy;
    let r11 = (1.0 - 2.0 * (qx * qx + qz * qz)) * sy;
    let r21 = 2.0 * (qy * qz + qx * qw) * sy;
    let r02 = 2.0 * (qx * qz + qy * qw) * sz;
    let r12 = 2.0 * (qy * qz - qx * qw) * sz;
    let r22 = (1.0 - 2.0 * (qx * qx + qy * qy)) * sz;

    // Column-major: m[col][row]
    [
        [r00, r10, r20, 0.0],
        [r01, r11, r21, 0.0],
        [r02, r12, r22, 0.0],
        [tx, ty, tz, 1.0],
    ]
}

#[allow(dead_code)]
pub fn set_gizmo_mode(gizmo: &mut GizmoState, mode: GizmoMode) {
    gizmo.mode = mode;
}

#[allow(dead_code)]
pub fn reset_gizmo(gizmo: &mut GizmoState) {
    gizmo.position = [0.0, 0.0, 0.0];
    gizmo.rotation = [0.0, 0.0, 0.0, 1.0];
    gizmo.scale = [1.0, 1.0, 1.0];
    gizmo.active_axis = None;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gizmo_position() {
        let g = new_gizmo(GizmoMode::Translate, [1.0, 2.0, 3.0]);
        assert_eq!(g.position, [1.0, 2.0, 3.0]);
        assert_eq!(g.rotation, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(g.scale, [1.0, 1.0, 1.0]);
        assert!(g.visible);
    }

    #[test]
    fn test_translate_gizmo() {
        let mut g = new_gizmo(GizmoMode::Translate, [0.0, 0.0, 0.0]);
        translate_gizmo(&mut g, [1.0, 2.0, 3.0]);
        assert_eq!(g.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_translate_gizmo_accumulates() {
        let mut g = new_gizmo(GizmoMode::Translate, [0.0, 0.0, 0.0]);
        translate_gizmo(&mut g, [1.0, 0.0, 0.0]);
        translate_gizmo(&mut g, [1.0, 0.0, 0.0]);
        assert!((g.position[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale_gizmo_x() {
        let mut g = new_gizmo(GizmoMode::Scale, [0.0; 3]);
        scale_gizmo(&mut g, GizmoAxis::X, 2.0);
        assert!((g.scale[0] - 2.0).abs() < 1e-5);
        assert!((g.scale[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale_gizmo_all() {
        let mut g = new_gizmo(GizmoMode::Scale, [0.0; 3]);
        scale_gizmo(&mut g, GizmoAxis::All, 3.0);
        assert!((g.scale[0] - 3.0).abs() < 1e-5);
        assert!((g.scale[1] - 3.0).abs() < 1e-5);
        assert!((g.scale[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_handles_nonempty_translate() {
        let g = new_gizmo(GizmoMode::Translate, [0.0; 3]);
        let handles = gizmo_handles(&g);
        assert!(!handles.is_empty());
    }

    #[test]
    fn test_handles_nonempty_rotate() {
        let g = new_gizmo(GizmoMode::Rotate, [0.0; 3]);
        let handles = gizmo_handles(&g);
        assert!(!handles.is_empty());
    }

    #[test]
    fn test_handles_nonempty_scale() {
        let g = new_gizmo(GizmoMode::Scale, [0.0; 3]);
        let handles = gizmo_handles(&g);
        assert!(!handles.is_empty());
    }

    #[test]
    fn test_handles_nonempty_universal() {
        let g = new_gizmo(GizmoMode::Universal, [0.0; 3]);
        let handles = gizmo_handles(&g);
        assert!(handles.len() >= 4);
    }

    #[test]
    fn test_snap_translate() {
        let result = snap_translate([1.3, 2.7, 0.5], 1.0);
        assert!((result[0] - 1.0).abs() < 1e-5);
        assert!((result[1] - 3.0).abs() < 1e-5);
        assert!((result[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_snap_translate_no_snap() {
        let delta = [1.3, 2.7, 0.5];
        let result = snap_translate(delta, 0.0);
        assert_eq!(result, delta);
    }

    #[test]
    fn test_snap_rotate() {
        let angle = 0.35; // ~20 degrees
        let snapped = snap_rotate(angle, 15.0);
        let expected = (0.35f32 / 15.0f32.to_radians()).round() * 15.0f32.to_radians();
        assert!((snapped - expected).abs() < 1e-5);
    }

    #[test]
    fn test_reset_gizmo() {
        let mut g = new_gizmo(GizmoMode::Translate, [5.0, 5.0, 5.0]);
        translate_gizmo(&mut g, [1.0, 1.0, 1.0]);
        scale_gizmo(&mut g, GizmoAxis::All, 2.0);
        reset_gizmo(&mut g);
        assert_eq!(g.position, [0.0, 0.0, 0.0]);
        assert_eq!(g.scale, [1.0, 1.0, 1.0]);
        assert_eq!(g.rotation, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_set_gizmo_mode() {
        let mut g = new_gizmo(GizmoMode::Translate, [0.0; 3]);
        set_gizmo_mode(&mut g, GizmoMode::Rotate);
        assert_eq!(g.mode, GizmoMode::Rotate);
    }

    #[test]
    fn test_gizmo_world_matrix_identity() {
        let g = new_gizmo(GizmoMode::Translate, [0.0; 3]);
        let m = gizmo_world_matrix(&g);
        // Translation column should be [0,0,0,1]
        assert!((m[3][0] - 0.0).abs() < 1e-5);
        assert!((m[3][3] - 1.0).abs() < 1e-5);
    }
}
