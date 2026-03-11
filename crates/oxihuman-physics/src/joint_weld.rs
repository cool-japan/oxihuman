#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeldJointDef {
    stiffness: f32,
    break_threshold: f32,
    error: f32,
    force: f32,
    broken: bool,
}

#[allow(dead_code)]
pub fn new_weld_joint_def(stiffness: f32, break_threshold: f32) -> WeldJointDef {
    WeldJointDef {
        stiffness,
        break_threshold,
        error: 0.0,
        force: 0.0,
        broken: false,
    }
}

#[allow(dead_code)]
pub fn weld_error_def(joint: &WeldJointDef) -> f32 {
    joint.error
}

#[allow(dead_code)]
pub fn weld_solve_def(joint: &mut WeldJointDef, distance: f32) {
    joint.error = distance;
    joint.force = -distance * joint.stiffness;
    if joint.break_threshold > 0.0 && distance > joint.break_threshold {
        joint.broken = true;
    }
}

#[allow(dead_code)]
pub fn weld_is_broken_def(joint: &WeldJointDef) -> bool {
    joint.broken
}

#[allow(dead_code)]
pub fn weld_break_threshold_def(joint: &WeldJointDef) -> f32 {
    joint.break_threshold
}

#[allow(dead_code)]
pub fn weld_set_stiffness_def(joint: &mut WeldJointDef, stiffness: f32) {
    joint.stiffness = stiffness;
}

#[allow(dead_code)]
pub fn weld_reset_def(joint: &mut WeldJointDef) {
    joint.error = 0.0;
    joint.force = 0.0;
    joint.broken = false;
}

#[allow(dead_code)]
pub fn weld_force_def(joint: &WeldJointDef) -> f32 {
    joint.force
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let j = new_weld_joint_def(100.0, 5.0);
        assert!(!weld_is_broken_def(&j));
    }

    #[test]
    fn test_solve() {
        let mut j = new_weld_joint_def(10.0, 5.0);
        weld_solve_def(&mut j, 1.0);
        assert!((weld_error_def(&j) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_force() {
        let mut j = new_weld_joint_def(10.0, 5.0);
        weld_solve_def(&mut j, 2.0);
        assert!((weld_force_def(&j) - (-20.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_break() {
        let mut j = new_weld_joint_def(10.0, 3.0);
        weld_solve_def(&mut j, 4.0);
        assert!(weld_is_broken_def(&j));
    }

    #[test]
    fn test_no_break() {
        let mut j = new_weld_joint_def(10.0, 5.0);
        weld_solve_def(&mut j, 3.0);
        assert!(!weld_is_broken_def(&j));
    }

    #[test]
    fn test_break_threshold() {
        let j = new_weld_joint_def(10.0, 7.0);
        assert!((weld_break_threshold_def(&j) - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_stiffness() {
        let mut j = new_weld_joint_def(10.0, 5.0);
        weld_set_stiffness_def(&mut j, 20.0);
        weld_solve_def(&mut j, 1.0);
        assert!((weld_force_def(&j) - (-20.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_reset() {
        let mut j = new_weld_joint_def(10.0, 3.0);
        weld_solve_def(&mut j, 4.0);
        weld_reset_def(&mut j);
        assert!(!weld_is_broken_def(&j));
        assert_eq!(weld_error_def(&j), 0.0);
    }

    #[test]
    fn test_zero_threshold() {
        let mut j = new_weld_joint_def(10.0, 0.0);
        weld_solve_def(&mut j, 100.0);
        assert!(!weld_is_broken_def(&j));
    }

    #[test]
    fn test_zero_distance() {
        let mut j = new_weld_joint_def(10.0, 5.0);
        weld_solve_def(&mut j, 0.0);
        assert_eq!(weld_force_def(&j), 0.0);
    }
}
