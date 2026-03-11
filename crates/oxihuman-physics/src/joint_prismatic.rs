#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PrismaticJoint {
    position: f32,
    velocity: f32,
    force: f32,
    min_limit: f32,
    max_limit: f32,
    limited: bool,
}

#[allow(dead_code)]
pub fn new_prismatic_joint(min_limit: f32, max_limit: f32) -> PrismaticJoint {
    PrismaticJoint {
        position: 0.0,
        velocity: 0.0,
        force: 0.0,
        min_limit,
        max_limit,
        limited: true,
    }
}

#[allow(dead_code)]
pub fn prismatic_position(joint: &PrismaticJoint) -> f32 {
    joint.position
}

#[allow(dead_code)]
pub fn prismatic_velocity(joint: &PrismaticJoint) -> f32 {
    joint.velocity
}

#[allow(dead_code)]
pub fn prismatic_force(joint: &PrismaticJoint) -> f32 {
    joint.force
}

#[allow(dead_code)]
pub fn prismatic_set_limits(joint: &mut PrismaticJoint, min_val: f32, max_val: f32) {
    joint.min_limit = min_val;
    joint.max_limit = max_val;
}

#[allow(dead_code)]
pub fn prismatic_solve(joint: &mut PrismaticJoint, target_pos: f32, stiffness: f32, dt: f32) {
    let clamped = if joint.limited {
        target_pos.clamp(joint.min_limit, joint.max_limit)
    } else {
        target_pos
    };
    let error = clamped - joint.position;
    joint.force = error * stiffness;
    joint.velocity = if dt > 0.0 { error / dt } else { 0.0 };
    joint.position = clamped;
}

#[allow(dead_code)]
pub fn prismatic_is_limited(joint: &PrismaticJoint) -> bool {
    joint.limited
}

#[allow(dead_code)]
pub fn prismatic_reset(joint: &mut PrismaticJoint) {
    joint.position = 0.0;
    joint.velocity = 0.0;
    joint.force = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let j = new_prismatic_joint(-1.0, 1.0);
        assert_eq!(prismatic_position(&j), 0.0);
    }

    #[test]
    fn test_solve() {
        let mut j = new_prismatic_joint(-2.0, 2.0);
        prismatic_solve(&mut j, 1.0, 10.0, 0.016);
        assert!((prismatic_position(&j) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clamped() {
        let mut j = new_prismatic_joint(-1.0, 1.0);
        prismatic_solve(&mut j, 5.0, 10.0, 0.016);
        assert!((prismatic_position(&j) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_force() {
        let mut j = new_prismatic_joint(-10.0, 10.0);
        prismatic_solve(&mut j, 2.0, 5.0, 0.016);
        assert!((prismatic_force(&j) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_velocity() {
        let mut j = new_prismatic_joint(-10.0, 10.0);
        prismatic_solve(&mut j, 1.0, 5.0, 0.5);
        assert!((prismatic_velocity(&j) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_limits() {
        let mut j = new_prismatic_joint(0.0, 0.0);
        prismatic_set_limits(&mut j, -5.0, 5.0);
        prismatic_solve(&mut j, 3.0, 1.0, 0.016);
        assert!((prismatic_position(&j) - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_limited() {
        let j = new_prismatic_joint(-1.0, 1.0);
        assert!(prismatic_is_limited(&j));
    }

    #[test]
    fn test_reset() {
        let mut j = new_prismatic_joint(-1.0, 1.0);
        prismatic_solve(&mut j, 0.5, 1.0, 0.016);
        prismatic_reset(&mut j);
        assert_eq!(prismatic_position(&j), 0.0);
        assert_eq!(prismatic_velocity(&j), 0.0);
    }

    #[test]
    fn test_negative_position() {
        let mut j = new_prismatic_joint(-5.0, 5.0);
        prismatic_solve(&mut j, -3.0, 1.0, 0.016);
        assert!((prismatic_position(&j) - (-3.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_zero_dt() {
        let mut j = new_prismatic_joint(-1.0, 1.0);
        prismatic_solve(&mut j, 0.5, 1.0, 0.0);
        assert_eq!(prismatic_velocity(&j), 0.0);
    }
}
