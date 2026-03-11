#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpringConstraint {
    rest_length: f32,
    stiffness: f32,
    damping: f32,
    current_length: f32,
    force_value: f32,
}

#[allow(dead_code)]
pub fn new_spring_constraint(rest_length: f32, stiffness: f32, damping: f32) -> SpringConstraint {
    SpringConstraint {
        rest_length,
        stiffness,
        damping,
        current_length: rest_length,
        force_value: 0.0,
    }
}

#[allow(dead_code)]
pub fn spring_force_cs(sc: &SpringConstraint) -> f32 {
    sc.force_value
}

#[allow(dead_code)]
pub fn spring_damping_cs(sc: &SpringConstraint) -> f32 {
    sc.damping
}

#[allow(dead_code)]
pub fn spring_rest_length_cs(sc: &SpringConstraint) -> f32 {
    sc.rest_length
}

#[allow(dead_code)]
pub fn spring_current_length_cs(sc: &SpringConstraint) -> f32 {
    sc.current_length
}

#[allow(dead_code)]
pub fn spring_error_cs(sc: &SpringConstraint) -> f32 {
    sc.current_length - sc.rest_length
}

#[allow(dead_code)]
pub fn spring_solve_cs(sc: &mut SpringConstraint, current_length: f32, velocity: f32) {
    sc.current_length = current_length;
    let displacement = current_length - sc.rest_length;
    sc.force_value = -sc.stiffness * displacement - sc.damping * velocity;
}

#[allow(dead_code)]
pub fn spring_reset_cs(sc: &mut SpringConstraint) {
    sc.current_length = sc.rest_length;
    sc.force_value = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = new_spring_constraint(1.0, 10.0, 0.5);
        assert!((spring_rest_length_cs(&s) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_solve_stretched() {
        let mut s = new_spring_constraint(1.0, 10.0, 0.0);
        spring_solve_cs(&mut s, 2.0, 0.0);
        assert!((spring_force_cs(&s) - (-10.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_solve_compressed() {
        let mut s = new_spring_constraint(1.0, 10.0, 0.0);
        spring_solve_cs(&mut s, 0.5, 0.0);
        assert!((spring_force_cs(&s) - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_damping() {
        let mut s = new_spring_constraint(1.0, 10.0, 2.0);
        spring_solve_cs(&mut s, 1.0, 3.0);
        assert!((spring_force_cs(&s) - (-6.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rest_length() {
        let s = new_spring_constraint(2.5, 10.0, 0.5);
        assert!((spring_rest_length_cs(&s) - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_current_length() {
        let mut s = new_spring_constraint(1.0, 10.0, 0.5);
        spring_solve_cs(&mut s, 3.0, 0.0);
        assert!((spring_current_length_cs(&s) - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_error() {
        let mut s = new_spring_constraint(1.0, 10.0, 0.5);
        spring_solve_cs(&mut s, 1.5, 0.0);
        assert!((spring_error_cs(&s) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_reset() {
        let mut s = new_spring_constraint(1.0, 10.0, 0.5);
        spring_solve_cs(&mut s, 2.0, 1.0);
        spring_reset_cs(&mut s);
        assert_eq!(spring_force_cs(&s), 0.0);
        assert!((spring_current_length_cs(&s) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_damping_value() {
        let s = new_spring_constraint(1.0, 10.0, 3.0);
        assert!((spring_damping_cs(&s) - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_at_rest() {
        let mut s = new_spring_constraint(1.0, 10.0, 0.0);
        spring_solve_cs(&mut s, 1.0, 0.0);
        assert_eq!(spring_force_cs(&s), 0.0);
    }
}
