#![allow(dead_code)]

/// A slider (prismatic) joint constraining motion along an axis.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SliderJoint {
    pub axis: [f32; 3],
    pub position: f32,
    pub velocity: f32,
    pub min_pos: f32,
    pub max_pos: f32,
    pub limited: bool,
}

/// Creates a new slider joint along the given axis.
#[allow(dead_code)]
pub fn new_slider_joint(axis: [f32; 3]) -> SliderJoint {
    SliderJoint {
        axis,
        position: 0.0,
        velocity: 0.0,
        min_pos: -1e6,
        max_pos: 1e6,
        limited: false,
    }
}

/// Returns the current slider position.
#[allow(dead_code)]
pub fn slider_position(joint: &SliderJoint) -> f32 {
    joint.position
}

/// Returns the current velocity.
#[allow(dead_code)]
pub fn slider_velocity(joint: &SliderJoint) -> f32 {
    joint.velocity
}

/// Returns the constraint force needed.
#[allow(dead_code)]
pub fn slider_force(joint: &SliderJoint) -> f32 {
    if joint.limited {
        if joint.position < joint.min_pos {
            (joint.min_pos - joint.position) * 100.0
        } else if joint.position > joint.max_pos {
            (joint.max_pos - joint.position) * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    }
}

/// Sets position limits.
#[allow(dead_code)]
pub fn slider_set_limits(joint: &mut SliderJoint, min_pos: f32, max_pos: f32) {
    joint.min_pos = min_pos;
    joint.max_pos = max_pos;
    joint.limited = true;
}

/// Solves the slider constraint for one step.
#[allow(dead_code)]
pub fn slider_solve(joint: &mut SliderJoint, dt: f32) {
    joint.position += joint.velocity * dt;
    if joint.limited {
        joint.position = joint.position.clamp(joint.min_pos, joint.max_pos);
    }
}

/// Returns whether the joint is limited.
#[allow(dead_code)]
pub fn slider_is_limited(joint: &SliderJoint) -> bool {
    joint.limited
}

/// Resets the joint state.
#[allow(dead_code)]
pub fn slider_reset(joint: &mut SliderJoint) {
    joint.position = 0.0;
    joint.velocity = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_slider() {
        let j = new_slider_joint([1.0, 0.0, 0.0]);
        assert!((slider_position(&j)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_velocity() {
        let j = new_slider_joint([1.0, 0.0, 0.0]);
        assert!((slider_velocity(&j)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_solve() {
        let mut j = new_slider_joint([1.0, 0.0, 0.0]);
        j.velocity = 2.0;
        slider_solve(&mut j, 0.5);
        assert!((slider_position(&j) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_limits() {
        let mut j = new_slider_joint([1.0, 0.0, 0.0]);
        slider_set_limits(&mut j, -1.0, 1.0);
        assert!(slider_is_limited(&j));
    }

    #[test]
    fn test_solve_clamped() {
        let mut j = new_slider_joint([1.0, 0.0, 0.0]);
        slider_set_limits(&mut j, -1.0, 1.0);
        j.velocity = 100.0;
        slider_solve(&mut j, 1.0);
        assert!((slider_position(&j) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_force_no_limit() {
        let j = new_slider_joint([1.0, 0.0, 0.0]);
        assert!((slider_force(&j)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_force_at_limit() {
        let mut j = new_slider_joint([1.0, 0.0, 0.0]);
        slider_set_limits(&mut j, -1.0, 1.0);
        j.position = 2.0;
        assert!(slider_force(&j) < 0.0);
    }

    #[test]
    fn test_reset() {
        let mut j = new_slider_joint([1.0, 0.0, 0.0]);
        j.position = 5.0;
        j.velocity = 3.0;
        slider_reset(&mut j);
        assert!((slider_position(&j)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_not_limited_default() {
        let j = new_slider_joint([1.0, 0.0, 0.0]);
        assert!(!slider_is_limited(&j));
    }

    #[test]
    fn test_axis() {
        let j = new_slider_joint([0.0, 1.0, 0.0]);
        assert!((j.axis[1] - 1.0).abs() < f32::EPSILON);
    }
}
