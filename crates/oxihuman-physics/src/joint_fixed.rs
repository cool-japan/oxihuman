#![allow(dead_code)]

/// A fixed joint that rigidly connects two bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FixedJoint {
    pub anchor_a: [f32; 3],
    pub anchor_b: [f32; 3],
    pub break_force: f32,
    pub broken: bool,
}

/// Creates a new fixed joint between two anchors.
#[allow(dead_code)]
pub fn new_fixed_joint(anchor_a: [f32; 3], anchor_b: [f32; 3]) -> FixedJoint {
    FixedJoint {
        anchor_a,
        anchor_b,
        break_force: f32::MAX,
        broken: false,
    }
}

/// Returns the positional error.
#[allow(dead_code)]
pub fn fixed_error(joint: &FixedJoint) -> f32 {
    let dx = joint.anchor_b[0] - joint.anchor_a[0];
    let dy = joint.anchor_b[1] - joint.anchor_a[1];
    let dz = joint.anchor_b[2] - joint.anchor_a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Solves the fixed joint constraint.
#[allow(dead_code)]
pub fn fixed_solve(joint: &mut FixedJoint, stiffness: f32) {
    if joint.broken {
        return;
    }
    let err = fixed_error(joint);
    if err > joint.break_force {
        joint.broken = true;
        return;
    }
    let correction = stiffness * 0.5;
    let dx = joint.anchor_b[0] - joint.anchor_a[0];
    let dy = joint.anchor_b[1] - joint.anchor_a[1];
    let dz = joint.anchor_b[2] - joint.anchor_a[2];
    joint.anchor_a[0] += dx * correction;
    joint.anchor_a[1] += dy * correction;
    joint.anchor_a[2] += dz * correction;
    joint.anchor_b[0] -= dx * correction;
    joint.anchor_b[1] -= dy * correction;
    joint.anchor_b[2] -= dz * correction;
}

/// Returns the constraint force magnitude.
#[allow(dead_code)]
pub fn fixed_force(joint: &FixedJoint) -> f32 {
    fixed_error(joint) * 100.0
}

/// Returns the constraint torque (approximation).
#[allow(dead_code)]
pub fn fixed_torque(joint: &FixedJoint) -> f32 {
    fixed_error(joint) * 50.0
}

/// Returns whether the joint is broken.
#[allow(dead_code)]
pub fn fixed_is_broken(joint: &FixedJoint) -> bool {
    joint.broken
}

/// Sets the break force threshold.
#[allow(dead_code)]
pub fn fixed_break_force(joint: &mut FixedJoint, force: f32) {
    joint.break_force = force;
}

/// Resets the joint to its initial state.
#[allow(dead_code)]
pub fn fixed_reset(joint: &mut FixedJoint) {
    joint.broken = false;
    joint.anchor_a = [0.0; 3];
    joint.anchor_b = [0.0; 3];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fixed() {
        let j = new_fixed_joint([0.0; 3], [1.0, 0.0, 0.0]);
        assert!(!fixed_is_broken(&j));
    }

    #[test]
    fn test_error() {
        let j = new_fixed_joint([0.0; 3], [3.0, 4.0, 0.0]);
        assert!((fixed_error(&j) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_solve() {
        let mut j = new_fixed_joint([0.0; 3], [2.0, 0.0, 0.0]);
        fixed_solve(&mut j, 0.5);
        assert!(fixed_error(&j) < 2.0);
    }

    #[test]
    fn test_force() {
        let j = new_fixed_joint([0.0; 3], [1.0, 0.0, 0.0]);
        assert!(fixed_force(&j) > 0.0);
    }

    #[test]
    fn test_torque() {
        let j = new_fixed_joint([0.0; 3], [1.0, 0.0, 0.0]);
        assert!(fixed_torque(&j) > 0.0);
    }

    #[test]
    fn test_break() {
        let mut j = new_fixed_joint([0.0; 3], [100.0, 0.0, 0.0]);
        fixed_break_force(&mut j, 1.0);
        fixed_solve(&mut j, 1.0);
        assert!(fixed_is_broken(&j));
    }

    #[test]
    fn test_reset() {
        let mut j = new_fixed_joint([1.0; 3], [2.0; 3]);
        j.broken = true;
        fixed_reset(&mut j);
        assert!(!fixed_is_broken(&j));
        assert!(fixed_error(&j).abs() < f32::EPSILON);
    }

    #[test]
    fn test_no_break_default() {
        let mut j = new_fixed_joint([0.0; 3], [0.1, 0.0, 0.0]);
        fixed_solve(&mut j, 0.5);
        assert!(!fixed_is_broken(&j));
    }

    #[test]
    fn test_broken_no_solve() {
        let mut j = new_fixed_joint([0.0; 3], [1.0, 0.0, 0.0]);
        j.broken = true;
        let err_before = fixed_error(&j);
        fixed_solve(&mut j, 1.0);
        assert!((fixed_error(&j) - err_before).abs() < f32::EPSILON);
    }

    #[test]
    fn test_zero_error() {
        let j = new_fixed_joint([5.0; 3], [5.0; 3]);
        assert!(fixed_error(&j).abs() < f32::EPSILON);
    }
}
