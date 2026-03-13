// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Orbit / fly camera rig for the interactive viewer.

use std::f32::consts::PI;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Camera control mode.
#[derive(Debug, Clone, PartialEq)]
pub enum CameraMode {
    Orbit,
    Fly,
    Fixed,
}

/// State for an orbit camera (spherical coordinates around a target point).
#[derive(Debug, Clone, PartialEq)]
pub struct OrbitState {
    pub target: [f32; 3],
    pub distance: f32,
    /// Horizontal angle in radians.
    pub azimuth: f32,
    /// Vertical angle in radians (clamped away from poles).
    pub elevation: f32,
}

/// State for a free-fly camera.
#[derive(Debug, Clone, PartialEq)]
pub struct FlyState {
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
}

/// Combined camera rig supporting orbit, fly, and fixed modes.
#[derive(Debug, Clone)]
pub struct CameraRig {
    pub mode: CameraMode,
    pub orbit: OrbitState,
    pub fly: FlyState,
    pub fov_deg: f32,
    pub near: f32,
    pub far: f32,
}

// ── Constructors ──────────────────────────────────────────────────────────────

impl CameraRig {
    /// Create an orbit camera looking at `target` from `distance` away.
    pub fn new_orbit(target: [f32; 3], distance: f32) -> Self {
        CameraRig {
            mode: CameraMode::Orbit,
            orbit: OrbitState {
                target,
                distance: distance.max(0.01),
                azimuth: 0.0,
                elevation: 0.0,
            },
            fly: FlyState {
                position: [0.0, 0.0, distance],
                yaw: 0.0,
                pitch: 0.0,
                speed: 5.0,
            },
            fov_deg: 60.0,
            near: 0.01,
            far: 1000.0,
        }
    }

    /// Create a free-fly camera starting at `position`.
    pub fn new_fly(position: [f32; 3]) -> Self {
        CameraRig {
            mode: CameraMode::Fly,
            orbit: OrbitState {
                target: [0.0; 3],
                distance: 3.0,
                azimuth: 0.0,
                elevation: 0.0,
            },
            fly: FlyState {
                position,
                yaw: 0.0,
                pitch: 0.0,
                speed: 5.0,
            },
            fov_deg: 60.0,
            near: 0.01,
            far: 1000.0,
        }
    }

    // ── Orbit controls ────────────────────────────────────────────────────────

    /// Compute the camera position from the current orbit spherical coordinates.
    pub fn orbit_position(&self) -> [f32; 3] {
        let az = self.orbit.azimuth;
        let el = self.orbit.elevation;
        let d = self.orbit.distance;
        let t = &self.orbit.target;
        [
            t[0] + d * el.cos() * az.sin(),
            t[1] + d * el.sin(),
            t[2] + d * el.cos() * az.cos(),
        ]
    }

    /// Rotate the orbit camera. Delta angles in radians.
    pub fn orbit_rotate(&mut self, delta_az: f32, delta_el: f32) {
        self.orbit.azimuth += delta_az;
        // Keep azimuth in [-π, π]
        while self.orbit.azimuth > PI {
            self.orbit.azimuth -= 2.0 * PI;
        }
        while self.orbit.azimuth < -PI {
            self.orbit.azimuth += 2.0 * PI;
        }
        self.orbit.elevation =
            (self.orbit.elevation + delta_el).clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01);
    }

    /// Zoom the orbit camera. Positive `delta` moves closer.
    pub fn orbit_zoom(&mut self, delta: f32) {
        self.orbit.distance = (self.orbit.distance - delta).max(0.01);
    }

    // ── View matrix ───────────────────────────────────────────────────────────

    /// Compute the 4×4 row-major look-at view matrix for the current camera state.
    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        match self.mode {
            CameraMode::Orbit | CameraMode::Fixed => {
                let eye = self.orbit_position();
                look_at_matrix(eye, self.orbit.target, [0.0, 1.0, 0.0])
            }
            CameraMode::Fly => {
                let forward = fly_forward(self.fly.yaw, self.fly.pitch);
                let target = add3(self.fly.position, forward);
                look_at_matrix(self.fly.position, target, [0.0, 1.0, 0.0])
            }
        }
    }

    // ── Projection matrix ─────────────────────────────────────────────────────

    /// Compute the 4×4 perspective projection matrix.
    pub fn projection_matrix(&self, aspect: f32) -> [[f32; 4]; 4] {
        let fov_rad = self.fov_deg.to_radians();
        perspective_matrix(fov_rad, aspect, self.near, self.far)
    }

    // ── Fly controls ──────────────────────────────────────────────────────────

    /// Move the fly camera. `forward`, `right`, `up` are signed unit amounts;
    /// `dt` is delta time in seconds.
    pub fn fly_move(&mut self, forward: f32, right: f32, up: f32, dt: f32) {
        let fwd = fly_forward(self.fly.yaw, self.fly.pitch);
        let right_vec = normalize3(cross3(fwd, [0.0, 1.0, 0.0]));
        let up_vec = [0.0f32, 1.0, 0.0];
        let speed = self.fly.speed * dt;
        self.fly.position = add3(
            self.fly.position,
            add3(
                add3(
                    scale3(fwd, forward * speed),
                    scale3(right_vec, right * speed),
                ),
                scale3(up_vec, up * speed),
            ),
        );
    }

    /// Adjust the fly camera look direction. Delta values in radians.
    pub fn fly_look(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.fly.yaw += delta_yaw;
        self.fly.pitch = (self.fly.pitch + delta_pitch).clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01);
    }
}

// ── Standalone matrix functions ───────────────────────────────────────────────

/// Compute a row-major look-at matrix (right-handed, OpenGL convention).
pub fn look_at_matrix(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let fwd = normalize3(sub3(target, eye));
    let right = normalize3(cross3(fwd, up));
    let up2 = cross3(right, fwd);

    let tx = -dot3(right, eye);
    let ty = -dot3(up2, eye);
    let tz = dot3(fwd, eye);

    [
        [right[0], up2[0], -fwd[0], 0.0],
        [right[1], up2[1], -fwd[1], 0.0],
        [right[2], up2[2], -fwd[2], 0.0],
        [tx, ty, tz, 1.0],
    ]
}

/// Compute a row-major perspective projection matrix.
///
/// `fov_y_rad` — vertical field of view in radians.
pub fn perspective_matrix(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let f = 1.0 / (fov_y_rad / 2.0).tan();
    let nf = 1.0 / (near - far);
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (far + near) * nf, -1.0],
        [0.0, 0.0, 2.0 * far * near * nf, 0.0],
    ]
}

/// Build a default orbit camera looking at the origin from distance 3.
pub fn default_orbit_camera() -> CameraRig {
    CameraRig::new_orbit([0.0, 1.0, 0.0], 3.0)
}

// ── Private math helpers ──────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

#[inline]
fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l < 1e-9 {
        [0.0, 0.0, 1.0]
    } else {
        scale3(a, 1.0 / l)
    }
}

/// Compute forward vector from yaw/pitch angles (radians).
fn fly_forward(yaw: f32, pitch: f32) -> [f32; 3] {
    normalize3([
        yaw.sin() * pitch.cos(),
        pitch.sin(),
        yaw.cos() * pitch.cos(),
    ])
}

// ── Camera rig hierarchy (new API) ────────────────────────────────────────────

#[allow(dead_code)]
pub struct CameraRigNode {
    pub name: String,
    pub local_pos: [f32; 3],
    pub children: Vec<usize>,
}

#[allow(dead_code)]
pub struct CameraRigHierarchy {
    pub nodes: Vec<CameraRigNode>,
    pub active_camera: usize,
}

#[allow(dead_code)]
pub fn new_camera_rig_hierarchy() -> CameraRigHierarchy {
    CameraRigHierarchy {
        nodes: Vec::new(),
        active_camera: 0,
    }
}

#[allow(dead_code)]
pub fn cr_add_node(rig: &mut CameraRigHierarchy, name: &str, pos: [f32; 3]) -> usize {
    let idx = rig.nodes.len();
    rig.nodes.push(CameraRigNode {
        name: name.to_string(),
        local_pos: pos,
        children: Vec::new(),
    });
    idx
}

#[allow(dead_code)]
pub fn cr_set_parent(rig: &mut CameraRigHierarchy, parent: usize, child: usize) {
    if parent < rig.nodes.len() {
        rig.nodes[parent].children.push(child);
    }
}

#[allow(dead_code)]
pub fn cr_node_count(rig: &CameraRigHierarchy) -> usize {
    rig.nodes.len()
}

#[allow(dead_code)]
pub fn cr_set_active(rig: &mut CameraRigHierarchy, idx: usize) {
    if idx < rig.nodes.len() {
        rig.active_camera = idx;
    }
}

#[allow(dead_code)]
pub fn cr_active_camera(rig: &CameraRigHierarchy) -> usize {
    rig.active_camera
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq3(a: [f32; 3], b: [f32; 3], eps: f32) -> bool {
        (a[0] - b[0]).abs() < eps && (a[1] - b[1]).abs() < eps && (a[2] - b[2]).abs() < eps
    }

    fn no_nan_mat4(m: [[f32; 4]; 4]) -> bool {
        m.iter().all(|row| row.iter().all(|v| !v.is_nan()))
    }

    #[test]
    fn orbit_position_at_zero_angles() {
        // az=0, el=0 → camera should be at (0, 0, dist) relative to target
        let rig = CameraRig::new_orbit([0.0; 3], 5.0);
        let pos = rig.orbit_position();
        // el=0 → y component = 0; az=0 → sin(0)=0 for x, cos(0)=1 for z
        assert!(approx_eq3(pos, [0.0, 0.0, 5.0], 1e-4), "got {:?}", pos);
    }

    #[test]
    fn orbit_position_no_nan() {
        let rig = CameraRig::new_orbit([0.0; 3], 3.0);
        let pos = rig.orbit_position();
        assert!(pos.iter().all(|v| !v.is_nan()));
    }

    #[test]
    fn view_matrix_no_nan() {
        let rig = CameraRig::new_orbit([0.0; 3], 3.0);
        let m = rig.view_matrix();
        assert!(no_nan_mat4(m), "view matrix contains NaN");
    }

    #[test]
    fn projection_matrix_no_nan() {
        let rig = CameraRig::new_orbit([0.0; 3], 3.0);
        let m = rig.projection_matrix(16.0 / 9.0);
        assert!(no_nan_mat4(m), "projection matrix contains NaN");
    }

    #[test]
    fn orbit_zoom_clamps_minimum() {
        let mut rig = CameraRig::new_orbit([0.0; 3], 1.0);
        rig.orbit_zoom(1000.0);
        assert!(
            rig.orbit.distance >= 0.01,
            "distance should be clamped >= 0.01"
        );
    }

    #[test]
    fn orbit_zoom_moves_closer() {
        let mut rig = CameraRig::new_orbit([0.0; 3], 5.0);
        rig.orbit_zoom(1.0);
        assert!((rig.orbit.distance - 4.0).abs() < 1e-5);
    }

    #[test]
    fn orbit_rotate_changes_azimuth() {
        let mut rig = CameraRig::new_orbit([0.0; 3], 3.0);
        rig.orbit_rotate(0.5, 0.0);
        assert!((rig.orbit.azimuth - 0.5).abs() < 1e-5);
    }

    #[test]
    fn orbit_rotate_clamps_elevation() {
        let mut rig = CameraRig::new_orbit([0.0; 3], 3.0);
        rig.orbit_rotate(0.0, PI); // push above pole
        assert!(rig.orbit.elevation < PI / 2.0);
    }

    #[test]
    fn look_at_matrix_deterministic() {
        let m1 = look_at_matrix([0.0, 0.0, 3.0], [0.0; 3], [0.0, 1.0, 0.0]);
        let m2 = look_at_matrix([0.0, 0.0, 3.0], [0.0; 3], [0.0, 1.0, 0.0]);
        assert_eq!(m1, m2, "look_at should be deterministic");
    }

    #[test]
    fn look_at_matrix_no_nan() {
        let m = look_at_matrix([1.0, 2.0, 3.0], [0.0; 3], [0.0, 1.0, 0.0]);
        assert!(no_nan_mat4(m));
    }

    #[test]
    fn perspective_matrix_no_nan() {
        let m = perspective_matrix(PI / 3.0, 16.0 / 9.0, 0.01, 1000.0);
        assert!(no_nan_mat4(m));
    }

    #[test]
    fn fly_move_changes_position() {
        let mut rig = CameraRig::new_fly([0.0; 3]);
        let before = rig.fly.position;
        rig.fly_move(1.0, 0.0, 0.0, 0.1);
        assert_ne!(rig.fly.position, before, "fly_move should change position");
    }

    #[test]
    fn fly_look_changes_yaw() {
        let mut rig = CameraRig::new_fly([0.0; 3]);
        rig.fly_look(0.3, 0.0);
        assert!((rig.fly.yaw - 0.3).abs() < 1e-5);
    }

    #[test]
    fn fly_look_clamps_pitch() {
        let mut rig = CameraRig::new_fly([0.0; 3]);
        rig.fly_look(0.0, PI); // push above ±90°
        assert!(rig.fly.pitch < PI / 2.0);
    }

    #[test]
    fn default_orbit_camera_valid() {
        let rig = default_orbit_camera();
        assert_eq!(rig.mode, CameraMode::Orbit);
        assert!((rig.orbit.distance - 3.0).abs() < 1e-5);
    }

    #[test]
    fn fly_camera_view_matrix_no_nan() {
        let mut rig = CameraRig::new_fly([0.0, 1.0, 3.0]);
        rig.mode = CameraMode::Fly;
        let m = rig.view_matrix();
        assert!(no_nan_mat4(m));
    }

    /* hierarchy API tests */
    #[test]
    fn test_cr_add_node() {
        let mut rig = new_camera_rig_hierarchy();
        let idx = cr_add_node(&mut rig, "main", [0.0, 0.0, 5.0]);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_cr_node_count() {
        let mut rig = new_camera_rig_hierarchy();
        cr_add_node(&mut rig, "a", [0.0; 3]);
        cr_add_node(&mut rig, "b", [1.0; 3]);
        assert_eq!(cr_node_count(&rig), 2);
    }

    #[test]
    fn test_cr_set_parent() {
        let mut rig = new_camera_rig_hierarchy();
        let p = cr_add_node(&mut rig, "parent", [0.0; 3]);
        let c = cr_add_node(&mut rig, "child", [1.0; 3]);
        cr_set_parent(&mut rig, p, c);
        assert!(rig.nodes[p].children.contains(&c));
    }

    #[test]
    fn test_cr_set_active() {
        let mut rig = new_camera_rig_hierarchy();
        cr_add_node(&mut rig, "cam0", [0.0; 3]);
        cr_add_node(&mut rig, "cam1", [1.0; 3]);
        cr_set_active(&mut rig, 1);
        assert_eq!(cr_active_camera(&rig), 1);
    }

    #[test]
    fn test_cr_active_camera_default() {
        let rig = new_camera_rig_hierarchy();
        assert_eq!(cr_active_camera(&rig), 0);
    }

    #[test]
    fn test_cr_set_active_oob_safe() {
        let mut rig = new_camera_rig_hierarchy();
        cr_set_active(&mut rig, 99);
        assert_eq!(cr_active_camera(&rig), 0);
    }

    #[test]
    fn test_cr_node_local_pos() {
        let mut rig = new_camera_rig_hierarchy();
        let idx = cr_add_node(&mut rig, "cam", [1.0, 2.0, 3.0]);
        assert!((rig.nodes[idx].local_pos[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_cr_set_parent_oob_safe() {
        let mut rig = new_camera_rig_hierarchy();
        cr_add_node(&mut rig, "cam", [0.0; 3]);
        cr_set_parent(&mut rig, 99, 0);
    }
}
