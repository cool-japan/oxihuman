// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Kinematic (script-animated) rigid body.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KinematicScriptConfig {
    pub max_speed: f32,
    pub max_angular_speed: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KinematicScriptBody {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub velocity: [f32; 3],
    pub angular_vel: [f32; 3],
    pub config: KinematicScriptConfig,
}

#[allow(dead_code)]
pub fn default_kinematic_script_config() -> KinematicScriptConfig {
    KinematicScriptConfig { max_speed: 100.0, max_angular_speed: 10.0 }
}

#[allow(dead_code)]
pub fn new_kinematic_script_body(config: KinematicScriptConfig) -> KinematicScriptBody {
    KinematicScriptBody {
        position: [0.0; 3],
        rotation: [0.0, 0.0, 0.0, 1.0],
        velocity: [0.0; 3],
        angular_vel: [0.0; 3],
        config,
    }
}

#[allow(dead_code)]
pub fn kb_move_to(body: &mut KinematicScriptBody, pos: [f32; 3]) {
    body.position = pos;
}

#[allow(dead_code)]
pub fn kb_set_velocity(body: &mut KinematicScriptBody, vel: [f32; 3]) {
    body.velocity = vel;
}

#[allow(dead_code)]
pub fn kb_step(body: &mut KinematicScriptBody, dt: f32) {
    body.position[0] += body.velocity[0] * dt;
    body.position[1] += body.velocity[1] * dt;
    body.position[2] += body.velocity[2] * dt;
}

#[allow(dead_code)]
pub fn kb_speed(body: &KinematicScriptBody) -> f32 {
    let v = &body.velocity;
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
pub fn kb_is_moving(body: &KinematicScriptBody, tol: f32) -> bool {
    kb_speed(body) > tol
}

#[allow(dead_code)]
pub fn kb_to_json(body: &KinematicScriptBody) -> String {
    let p = &body.position;
    let v = &body.velocity;
    format!(
        "{{\"position\":[{},{},{}],\"velocity\":[{},{},{}],\"speed\":{}}}",
        p[0], p[1], p[2],
        v[0], v[1], v[2],
        kb_speed(body)
    )
}

#[allow(dead_code)]
pub fn kb_reset(body: &mut KinematicScriptBody) {
    body.position = [0.0; 3];
    body.rotation = [0.0, 0.0, 0.0, 1.0];
    body.velocity = [0.0; 3];
    body.angular_vel = [0.0; 3];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_kinematic_script_config();
        assert!(cfg.max_speed > 0.0);
    }

    #[test]
    fn test_new_body_at_origin() {
        let cfg = default_kinematic_script_config();
        let b = new_kinematic_script_body(cfg);
        assert_eq!(b.position, [0.0; 3]);
    }

    #[test]
    fn test_kb_move_to() {
        let cfg = default_kinematic_script_config();
        let mut b = new_kinematic_script_body(cfg);
        kb_move_to(&mut b, [1.0, 2.0, 3.0]);
        assert_eq!(b.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_kb_step() {
        let cfg = default_kinematic_script_config();
        let mut b = new_kinematic_script_body(cfg);
        kb_set_velocity(&mut b, [2.0, 0.0, 0.0]);
        kb_step(&mut b, 1.0);
        assert!((b.position[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_kb_speed() {
        let cfg = default_kinematic_script_config();
        let mut b = new_kinematic_script_body(cfg);
        kb_set_velocity(&mut b, [3.0, 4.0, 0.0]);
        assert!((kb_speed(&b) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_kb_is_moving() {
        let cfg = default_kinematic_script_config();
        let mut b = new_kinematic_script_body(cfg);
        assert!(!kb_is_moving(&b, 0.01));
        kb_set_velocity(&mut b, [1.0, 0.0, 0.0]);
        assert!(kb_is_moving(&b, 0.01));
    }

    #[test]
    fn test_kb_to_json() {
        let cfg = default_kinematic_script_config();
        let b = new_kinematic_script_body(cfg);
        let j = kb_to_json(&b);
        assert!(j.contains("position"));
    }

    #[test]
    fn test_kb_reset() {
        let cfg = default_kinematic_script_config();
        let mut b = new_kinematic_script_body(cfg);
        kb_move_to(&mut b, [5.0, 5.0, 5.0]);
        kb_reset(&mut b);
        assert_eq!(b.position, [0.0; 3]);
    }

    #[test]
    fn test_kb_step_multiple() {
        let cfg = default_kinematic_script_config();
        let mut b = new_kinematic_script_body(cfg);
        kb_set_velocity(&mut b, [1.0, 0.0, 0.0]);
        kb_step(&mut b, 0.5);
        kb_step(&mut b, 0.5);
        assert!((b.position[0] - 1.0).abs() < 1e-5);
    }
}
