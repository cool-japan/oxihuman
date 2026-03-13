// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Camera pivot — orbit/tumble point management for viewport cameras.

use std::f32::consts::PI;

/// Camera pivot state — the world-space point cameras orbit around.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraPivot {
    pub position: [f32; 3],
    pub auto_fit: bool,
}

/// Pivot animation target.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PivotTransition {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub duration: f32,
    pub elapsed: f32,
}

#[allow(dead_code)]
pub fn default_camera_pivot() -> CameraPivot {
    CameraPivot {
        position: [0.0, 0.9, 0.0],
        auto_fit: true,
    }
}

#[allow(dead_code)]
pub fn cp_set_position(pivot: &mut CameraPivot, pos: [f32; 3]) {
    pivot.position = pos;
}

#[allow(dead_code)]
pub fn cp_move_by(pivot: &mut CameraPivot, delta: [f32; 3]) {
    pivot.position[0] += delta[0];
    pivot.position[1] += delta[1];
    pivot.position[2] += delta[2];
}

#[allow(dead_code)]
pub fn cp_reset(pivot: &mut CameraPivot) {
    *pivot = default_camera_pivot();
}

#[allow(dead_code)]
pub fn cp_distance_to(pivot: &CameraPivot, eye: [f32; 3]) -> f32 {
    let dx = eye[0] - pivot.position[0];
    let dy = eye[1] - pivot.position[1];
    let dz = eye[2] - pivot.position[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn cp_is_at_origin(pivot: &CameraPivot) -> bool {
    pivot.position.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn cp_orbit_position(
    pivot: &CameraPivot,
    yaw_deg: f32,
    pitch_deg: f32,
    radius: f32,
) -> [f32; 3] {
    let yaw = yaw_deg.to_radians();
    let pitch = pitch_deg.to_radians().clamp(-PI * 0.49, PI * 0.49);
    let x = radius * yaw.sin() * pitch.cos();
    let y = radius * pitch.sin();
    let z = radius * yaw.cos() * pitch.cos();
    [
        pivot.position[0] + x,
        pivot.position[1] + y,
        pivot.position[2] + z,
    ]
}

#[allow(dead_code)]
pub fn cp_start_transition(from: [f32; 3], to: [f32; 3], duration: f32) -> PivotTransition {
    PivotTransition {
        from,
        to,
        duration: duration.max(0.001),
        elapsed: 0.0,
    }
}

#[allow(dead_code)]
pub fn cp_advance_transition(tr: &mut PivotTransition, dt: f32) -> [f32; 3] {
    tr.elapsed = (tr.elapsed + dt).min(tr.duration);
    let t = (tr.elapsed / tr.duration).clamp(0.0, 1.0);
    let smooth = t * t * (3.0 - 2.0 * t);
    [
        tr.from[0] + (tr.to[0] - tr.from[0]) * smooth,
        tr.from[1] + (tr.to[1] - tr.from[1]) * smooth,
        tr.from[2] + (tr.to[2] - tr.from[2]) * smooth,
    ]
}

#[allow(dead_code)]
pub fn cp_transition_done(tr: &PivotTransition) -> bool {
    tr.elapsed >= tr.duration
}

#[allow(dead_code)]
pub fn cp_to_json(pivot: &CameraPivot) -> String {
    format!(
        r#"{{"position":[{:.4},{:.4},{:.4}],"auto_fit":{}}}"#,
        pivot.position[0], pivot.position[1], pivot.position[2], pivot.auto_fit
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_not_at_origin() {
        let p = default_camera_pivot();
        assert!(!cp_is_at_origin(&p));
    }

    #[test]
    fn set_position() {
        let mut p = default_camera_pivot();
        cp_set_position(&mut p, [1.0, 2.0, 3.0]);
        assert!((p.position[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn move_by() {
        let mut p = default_camera_pivot();
        let orig = p.position;
        cp_move_by(&mut p, [1.0, 0.0, 0.0]);
        assert!((p.position[0] - orig[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_to_default() {
        let mut p = default_camera_pivot();
        cp_set_position(&mut p, [5.0, 5.0, 5.0]);
        cp_reset(&mut p);
        assert!((p.position[1] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn distance_to_eye() {
        let p = default_camera_pivot();
        let eye = [p.position[0], p.position[1], p.position[2] + 3.0];
        assert!((cp_distance_to(&p, eye) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn orbit_position_distance() {
        let p = default_camera_pivot();
        let eye = cp_orbit_position(&p, 0.0, 0.0, 2.0);
        let d = cp_distance_to(&p, eye);
        assert!((d - 2.0).abs() < 1e-5);
    }

    #[test]
    fn transition_starts_at_from() {
        let from = [0.0, 0.0, 0.0];
        let to = [1.0, 1.0, 1.0];
        let mut tr = cp_start_transition(from, to, 1.0);
        let pos = cp_advance_transition(&mut tr, 0.0);
        assert!(pos.iter().all(|&v| v.abs() < 1e-6));
    }

    #[test]
    fn transition_ends_at_to() {
        let from = [0.0, 0.0, 0.0];
        let to = [2.0, 2.0, 2.0];
        let mut tr = cp_start_transition(from, to, 0.5);
        cp_advance_transition(&mut tr, 1.0);
        assert!(cp_transition_done(&tr));
    }

    #[test]
    fn to_json_fields() {
        let p = default_camera_pivot();
        let j = cp_to_json(&p);
        assert!(j.contains("position"));
        assert!(j.contains("auto_fit"));
    }
}
