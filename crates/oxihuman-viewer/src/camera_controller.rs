#![allow(dead_code)]

/// Camera control mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlMode { Orbit, Fly, Pan }

/// Camera controller managing different interaction modes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraController {
    mode: ControlMode,
    yaw: f32,
    pitch: f32,
    distance: f32,
    pan_offset: [f32; 2],
}

#[allow(dead_code)]
pub fn new_camera_controller() -> CameraController {
    CameraController { mode: ControlMode::Orbit, yaw: 0.0, pitch: 0.0, distance: 5.0, pan_offset: [0.0; 2] }
}

#[allow(dead_code)]
pub fn controller_orbit(ctrl: &mut CameraController, dyaw: f32, dpitch: f32) {
    ctrl.yaw += dyaw;
    ctrl.pitch = (ctrl.pitch + dpitch).clamp(-89.0, 89.0);
}

#[allow(dead_code)]
pub fn controller_fly(ctrl: &mut CameraController, forward: f32, right: f32) {
    ctrl.pan_offset[0] += right;
    ctrl.pan_offset[1] += forward;
}

#[allow(dead_code)]
pub fn controller_pan_cc(ctrl: &mut CameraController, dx: f32, dy: f32) {
    ctrl.pan_offset[0] += dx;
    ctrl.pan_offset[1] += dy;
}

#[allow(dead_code)]
pub fn set_control_mode(ctrl: &mut CameraController, mode: ControlMode) { ctrl.mode = mode; }

#[allow(dead_code)]
pub fn control_mode_name(ctrl: &CameraController) -> &str {
    match ctrl.mode { ControlMode::Orbit => "orbit", ControlMode::Fly => "fly", ControlMode::Pan => "pan" }
}

#[allow(dead_code)]
pub fn controller_to_json(ctrl: &CameraController) -> String {
    format!("{{\"mode\":\"{}\",\"yaw\":{:.4},\"pitch\":{:.4},\"distance\":{:.4}}}", control_mode_name(ctrl), ctrl.yaw, ctrl.pitch, ctrl.distance)
}

#[allow(dead_code)]
pub fn controller_reset(ctrl: &mut CameraController) {
    ctrl.yaw = 0.0; ctrl.pitch = 0.0; ctrl.distance = 5.0; ctrl.pan_offset = [0.0; 2];
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert_eq!(control_mode_name(&new_camera_controller()), "orbit"); }
    #[test] fn test_orbit() {
        let mut c = new_camera_controller();
        controller_orbit(&mut c, 10.0, 5.0);
        assert!((c.yaw - 10.0).abs() < 1e-6);
    }
    #[test] fn test_pitch_clamp() {
        let mut c = new_camera_controller();
        controller_orbit(&mut c, 0.0, 200.0);
        assert!(c.pitch <= 89.0);
    }
    #[test] fn test_fly() {
        let mut c = new_camera_controller();
        controller_fly(&mut c, 1.0, 2.0);
        assert!((c.pan_offset[0] - 2.0).abs() < 1e-6);
    }
    #[test] fn test_pan() {
        let mut c = new_camera_controller();
        controller_pan_cc(&mut c, 1.0, 2.0);
        assert!((c.pan_offset[1] - 2.0).abs() < 1e-6);
    }
    #[test] fn test_set_mode() {
        let mut c = new_camera_controller();
        set_control_mode(&mut c, ControlMode::Fly);
        assert_eq!(control_mode_name(&c), "fly");
    }
    #[test] fn test_to_json() { assert!(controller_to_json(&new_camera_controller()).contains("mode")); }
    #[test] fn test_reset() {
        let mut c = new_camera_controller();
        controller_orbit(&mut c, 45.0, 30.0);
        controller_reset(&mut c);
        assert!((c.yaw).abs() < 1e-6);
    }
    #[test] fn test_pan_mode() {
        let mut c = new_camera_controller();
        set_control_mode(&mut c, ControlMode::Pan);
        assert_eq!(control_mode_name(&c), "pan");
    }
    #[test] fn test_distance() { assert!((new_camera_controller().distance - 5.0).abs() < 1e-6); }
    #[test] fn test_orbit_cumulative() {
        let mut c = new_camera_controller();
        controller_orbit(&mut c, 10.0, 0.0);
        controller_orbit(&mut c, 10.0, 0.0);
        assert!((c.yaw - 20.0).abs() < 1e-6);
    }
}
