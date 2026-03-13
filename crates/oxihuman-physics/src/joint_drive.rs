// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Joint motor with position/velocity drive.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum JointDriveMode {
    Position,
    Velocity,
    Torque,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointDriveConfig {
    pub stiffness: f32,
    pub damping: f32,
    pub max_force: f32,
    pub mode: JointDriveMode,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointDrive {
    pub target: f32,
    pub current: f32,
    pub lambda: f32,
    pub config: JointDriveConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointDriveResult {
    pub force: f32,
    pub error: f32,
}

#[allow(dead_code)]
pub fn default_joint_drive_config() -> JointDriveConfig {
    JointDriveConfig {
        stiffness: 100.0,
        damping: 10.0,
        max_force: 1000.0,
        mode: JointDriveMode::Position,
    }
}

#[allow(dead_code)]
pub fn new_joint_drive(config: JointDriveConfig) -> JointDrive {
    JointDrive { target: 0.0, current: 0.0, lambda: 0.0, config }
}

#[allow(dead_code)]
pub fn jd_set_target(drive: &mut JointDrive, target: f32) {
    drive.target = target;
}

#[allow(dead_code)]
pub fn jd_solve(drive: &mut JointDrive, dt: f32) -> JointDriveResult {
    let error = drive.target - drive.current;
    let force = match drive.config.mode {
        JointDriveMode::Position => {
            let f = drive.config.stiffness * error - drive.config.damping * (drive.current * dt);
            f.clamp(-drive.config.max_force, drive.config.max_force)
        }
        JointDriveMode::Velocity => {
            let f = drive.config.damping * error;
            f.clamp(-drive.config.max_force, drive.config.max_force)
        }
        JointDriveMode::Torque => {
            drive.target.clamp(-drive.config.max_force, drive.config.max_force)
        }
    };
    drive.lambda += force * dt;
    drive.current += force * dt;
    JointDriveResult { force, error }
}

#[allow(dead_code)]
pub fn jd_reset(drive: &mut JointDrive) {
    drive.target = 0.0;
    drive.current = 0.0;
    drive.lambda = 0.0;
}

#[allow(dead_code)]
pub fn jd_error(drive: &JointDrive) -> f32 {
    drive.target - drive.current
}

#[allow(dead_code)]
pub fn jd_is_satisfied(drive: &JointDrive, tol: f32) -> bool {
    jd_error(drive).abs() < tol
}

#[allow(dead_code)]
pub fn jd_to_json(drive: &JointDrive) -> String {
    format!(
        "{{\"target\":{},\"current\":{},\"lambda\":{},\"error\":{}}}",
        drive.target,
        drive.current,
        drive.lambda,
        jd_error(drive)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_joint_drive_config();
        assert_eq!(cfg.mode, JointDriveMode::Position);
        assert!(cfg.stiffness > 0.0);
    }

    #[test]
    fn test_new_joint_drive() {
        let cfg = default_joint_drive_config();
        let d = new_joint_drive(cfg);
        assert_eq!(d.target, 0.0);
        assert_eq!(d.current, 0.0);
    }

    #[test]
    fn test_jd_set_target() {
        let cfg = default_joint_drive_config();
        let mut d = new_joint_drive(cfg);
        jd_set_target(&mut d, 1.0);
        assert_eq!(d.target, 1.0);
    }

    #[test]
    fn test_jd_solve_produces_force() {
        let cfg = default_joint_drive_config();
        let mut d = new_joint_drive(cfg);
        jd_set_target(&mut d, 1.0);
        let result = jd_solve(&mut d, 0.016);
        assert_ne!(result.force, 0.0);
    }

    #[test]
    fn test_jd_error() {
        let cfg = default_joint_drive_config();
        let mut d = new_joint_drive(cfg);
        jd_set_target(&mut d, 5.0);
        // Before solve, current=0
        assert!((jd_error(&d) - 5.0).abs() < 1e-5);
        jd_solve(&mut d, 0.001);
        // Error should decrease (small dt avoids overshoot)
        assert!(jd_error(&d).abs() < 5.0);
    }

    #[test]
    fn test_jd_reset() {
        let cfg = default_joint_drive_config();
        let mut d = new_joint_drive(cfg);
        jd_set_target(&mut d, 5.0);
        jd_solve(&mut d, 0.1);
        jd_reset(&mut d);
        assert_eq!(d.target, 0.0);
        assert_eq!(d.current, 0.0);
    }

    #[test]
    fn test_jd_to_json() {
        let cfg = default_joint_drive_config();
        let d = new_joint_drive(cfg);
        let j = jd_to_json(&d);
        assert!(j.contains("target"));
    }

    #[test]
    fn test_velocity_mode() {
        let cfg = JointDriveConfig {
            stiffness: 0.0,
            damping: 1.0,
            max_force: 100.0,
            mode: JointDriveMode::Velocity,
        };
        let mut d = new_joint_drive(cfg);
        jd_set_target(&mut d, 2.0);
        let r = jd_solve(&mut d, 0.1);
        assert!(r.force.abs() > 0.0);
    }

    #[test]
    fn test_jd_is_satisfied_at_zero() {
        let cfg = default_joint_drive_config();
        let d = new_joint_drive(cfg);
        // target=0, current=0
        assert!(jd_is_satisfied(&d, 0.01));
    }
}
