#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraTarget {
    position: [f32; 3],
    current: [f32; 3],
    speed: f32,
}

#[allow(dead_code)]
pub fn new_camera_target(pos: [f32; 3]) -> CameraTarget {
    CameraTarget { position: pos, current: pos, speed: 5.0 }
}

#[allow(dead_code)]
pub fn set_target_pos(ct: &mut CameraTarget, pos: [f32; 3]) { ct.position = pos; }

#[allow(dead_code)]
pub fn target_position_ct(ct: &CameraTarget) -> [f32; 3] { ct.position }

#[allow(dead_code)]
pub fn smooth_follow(ct: &mut CameraTarget, dt: f32) -> [f32; 3] {
    let t = (ct.speed * dt).min(1.0);
    for i in 0..3 { ct.current[i] += (ct.position[i] - ct.current[i]) * t; }
    ct.current
}

#[allow(dead_code)]
pub fn follow_speed(ct: &CameraTarget) -> f32 { ct.speed }

#[allow(dead_code)]
pub fn target_to_json(ct: &CameraTarget) -> String {
    format!("{{\"target\":[{:.2},{:.2},{:.2}]}}", ct.position[0], ct.position[1], ct.position[2])
}

#[allow(dead_code)]
pub fn target_reset(ct: &mut CameraTarget) { ct.current = ct.position; }

#[allow(dead_code)]
pub fn target_distance_ct(ct: &CameraTarget) -> f32 {
    let dx = ct.position[0] - ct.current[0];
    let dy = ct.position[1] - ct.current[1];
    let dz = ct.position[2] - ct.current[2];
    (dx*dx + dy*dy + dz*dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let t = new_camera_target([0.0; 3]); assert!((target_position_ct(&t)[0]).abs() < 1e-6); }
    #[test] fn test_set() { let mut t = new_camera_target([0.0; 3]); set_target_pos(&mut t, [1.0, 2.0, 3.0]); assert!((target_position_ct(&t)[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_follow() { let mut t = new_camera_target([0.0; 3]); set_target_pos(&mut t, [1.0, 0.0, 0.0]); smooth_follow(&mut t, 0.1); assert!(t.current[0] > 0.0); }
    #[test] fn test_speed() { let t = new_camera_target([0.0; 3]); assert!((follow_speed(&t) - 5.0).abs() < 1e-6); }
    #[test] fn test_json() { let t = new_camera_target([0.0; 3]); assert!(target_to_json(&t).contains("target")); }
    #[test] fn test_reset() { let mut t = new_camera_target([0.0; 3]); set_target_pos(&mut t, [1.0, 0.0, 0.0]); target_reset(&mut t); assert!((target_distance_ct(&t)).abs() < 1e-6); }
    #[test] fn test_distance_zero() { let t = new_camera_target([1.0, 2.0, 3.0]); assert!(target_distance_ct(&t) < 1e-6); }
    #[test] fn test_distance() { let mut t = new_camera_target([0.0; 3]); set_target_pos(&mut t, [3.0, 4.0, 0.0]); assert!((target_distance_ct(&t) - 5.0).abs() < 1e-4); }
    #[test] fn test_follow_converge() { let mut t = new_camera_target([0.0; 3]); set_target_pos(&mut t, [1.0, 0.0, 0.0]); for _ in 0..100 { smooth_follow(&mut t, 0.1); } assert!((t.current[0] - 1.0).abs() < 0.01); }
    #[test] fn test_position() { let t = new_camera_target([5.0, 6.0, 7.0]); assert!((target_position_ct(&t)[2] - 7.0).abs() < 1e-6); }
}
