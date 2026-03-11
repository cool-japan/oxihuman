// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Camera state and math helpers for the viewer.

// ── CameraState ───────────────────────────────────────────────────────────────

/// Represents the 3-D camera viewpoint used by the viewer.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraState {
    /// Eye position in world space.
    pub position: [f32; 3],
    /// Point the camera looks at.
    pub target: [f32; 3],
    /// Up vector (usually [0, 1, 0]).
    pub up: [f32; 3],
    /// Vertical field-of-view in degrees.
    pub fov_deg: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        CameraState {
            position: [0.0, 1.0, -3.0],
            target: [0.0, 0.9, 0.0],
            up: [0.0, 1.0, 0.0],
            fov_deg: 60.0,
        }
    }
}

impl CameraState {
    /// Compute a column-major look-at view matrix.
    ///
    /// Returns a 4x4 matrix encoded as `[[f32; 4]; 4]` where `m[col][row]`.
    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        let e = self.position;
        let t = self.target;
        let u = self.up;

        // forward = normalize(target - position)
        let fwd = normalize3(sub3(t, e));
        // right   = normalize(forward x up)
        let right = normalize3(cross3(fwd, u));
        // recomputed up = right x forward
        let up = cross3(right, fwd);

        // Translation part
        let tx = -dot3(right, e);
        let ty = -dot3(up, e);
        let tz = dot3(fwd, e); // note: we look along -fwd in OpenGL convention

        // Column-major: m[col][row]
        [
            [right[0], up[0], -fwd[0], 0.0],
            [right[1], up[1], -fwd[1], 0.0],
            [right[2], up[2], -fwd[2], 0.0],
            [tx, ty, tz, 1.0],
        ]
    }

    /// Orbit the camera around the target by `yaw_deg` and `pitch_deg`.
    pub fn orbit(&mut self, yaw_deg: f32, pitch_deg: f32) {
        let yaw = yaw_deg.to_radians();
        let pitch = pitch_deg.to_radians();

        // Offset from target
        let offset = sub3(self.position, self.target);
        let dist = len3(offset);
        if dist < 1e-6 {
            return;
        }

        // Convert to spherical coordinates
        let theta = f32::atan2(offset[0], offset[2]); // horizontal angle
        let phi = f32::asin((offset[1] / dist).clamp(-1.0, 1.0)); // vertical angle

        let new_theta = theta + yaw;
        let new_phi = (phi + pitch).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );

        self.position = [
            self.target[0] + dist * new_phi.cos() * new_theta.sin(),
            self.target[1] + dist * new_phi.sin(),
            self.target[2] + dist * new_phi.cos() * new_theta.cos(),
        ];
    }

    /// Zoom in or out by adjusting the distance between camera and target.
    ///
    /// Positive `delta` zooms in (moves closer), negative zooms out.
    pub fn zoom(&mut self, delta: f32) {
        const MIN_DIST: f32 = 0.05;
        let offset = sub3(self.position, self.target);
        let dist = len3(offset);
        let new_dist = (dist - delta).max(MIN_DIST);
        let dir = normalize3(offset);
        self.position = add3(self.target, scale3(dir, new_dist));
    }
}

// ── Private math helpers ──────────────────────────────────────────────────────

#[inline]
pub(crate) fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
pub(crate) fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
pub(crate) fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
pub(crate) fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
pub(crate) fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
pub(crate) fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

#[inline]
pub(crate) fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l < 1e-9 {
        [0.0, 0.0, 0.0]
    } else {
        scale3(a, 1.0 / l)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_default_values() {
        let cam = CameraState::default();
        assert_eq!(cam.position, [0.0, 1.0, -3.0]);
        assert_eq!(cam.target, [0.0, 0.9, 0.0]);
        assert_eq!(cam.up, [0.0, 1.0, 0.0]);
        assert!((cam.fov_deg - 60.0).abs() < 1e-6);
    }

    #[test]
    fn camera_view_matrix_is_4x4() {
        let cam = CameraState::default();
        let m = cam.view_matrix();
        assert_eq!(m.len(), 4);
        for col in &m {
            assert_eq!(col.len(), 4);
        }
    }

    #[test]
    fn camera_view_matrix_last_column_homogeneous() {
        let cam = CameraState::default();
        let m = cam.view_matrix();
        assert!((m[3][3] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn camera_orbit_changes_position() {
        let mut cam = CameraState::default();
        let original = cam.position;
        cam.orbit(45.0, 0.0);
        assert_ne!(cam.position, original, "orbit should move the camera");
    }

    #[test]
    fn camera_orbit_preserves_distance() {
        let mut cam = CameraState::default();
        let before = len3(sub3(cam.position, cam.target));
        cam.orbit(90.0, 10.0);
        let after = len3(sub3(cam.position, cam.target));
        assert!(
            (before - after).abs() < 1e-3,
            "orbit should preserve distance"
        );
    }

    #[test]
    fn camera_zoom_changes_distance() {
        let mut cam = CameraState::default();
        let before = len3(sub3(cam.position, cam.target));
        cam.zoom(0.5);
        let after = len3(sub3(cam.position, cam.target));
        assert!(after < before, "positive zoom should move camera closer");
    }

    #[test]
    fn camera_zoom_clamps_minimum_distance() {
        let mut cam = CameraState::default();
        cam.zoom(1000.0);
        let dist = len3(sub3(cam.position, cam.target));
        assert!(dist >= 0.04, "distance should never go below minimum");
    }
}
