#![allow(dead_code)]

/// A Jacobian matrix row for constraint solving.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Jacobian {
    pub linear: [f32; 3],
    pub angular: [f32; 3],
}

/// Creates a new Jacobian with linear and angular parts.
#[allow(dead_code)]
pub fn new_jacobian(linear: [f32; 3], angular: [f32; 3]) -> Jacobian {
    Jacobian { linear, angular }
}

/// Dot product of the Jacobian with a 6-vector (linear + angular velocity).
#[allow(dead_code)]
pub fn jacobian_dot(j: &Jacobian, velocity: &[f32; 6]) -> f32 {
    j.linear[0] * velocity[0]
        + j.linear[1] * velocity[1]
        + j.linear[2] * velocity[2]
        + j.angular[0] * velocity[3]
        + j.angular[1] * velocity[4]
        + j.angular[2] * velocity[5]
}

/// Applies the Jacobian transpose scaled by lambda to produce impulse.
#[allow(dead_code)]
pub fn jacobian_apply(j: &Jacobian, lambda: f32) -> [f32; 6] {
    [
        j.linear[0] * lambda,
        j.linear[1] * lambda,
        j.linear[2] * lambda,
        j.angular[0] * lambda,
        j.angular[1] * lambda,
        j.angular[2] * lambda,
    ]
}

/// Returns the transpose (same structure for row Jacobian).
#[allow(dead_code)]
pub fn jacobian_transpose(j: &Jacobian) -> Jacobian {
    j.clone()
}

/// Multiplies the Jacobian by inverse mass to get effective mass contribution.
#[allow(dead_code)]
pub fn jacobian_multiply_inv_mass(j: &Jacobian, inv_mass: f32, inv_inertia: f32) -> f32 {
    let lin = j.linear[0] * j.linear[0] + j.linear[1] * j.linear[1] + j.linear[2] * j.linear[2];
    let ang = j.angular[0] * j.angular[0] + j.angular[1] * j.angular[1] + j.angular[2] * j.angular[2];
    lin * inv_mass + ang * inv_inertia
}

/// Returns the effective mass for this Jacobian row.
#[allow(dead_code)]
pub fn jacobian_effective_mass(j: &Jacobian, inv_mass: f32, inv_inertia: f32) -> f32 {
    let em = jacobian_multiply_inv_mass(j, inv_mass, inv_inertia);
    if em > f32::EPSILON { 1.0 / em } else { 0.0 }
}

/// Converts the Jacobian to a 6-element array.
#[allow(dead_code)]
pub fn jacobian_to_array(j: &Jacobian) -> [f32; 6] {
    [
        j.linear[0], j.linear[1], j.linear[2],
        j.angular[0], j.angular[1], j.angular[2],
    ]
}

/// Returns true if all components are zero.
#[allow(dead_code)]
pub fn jacobian_is_zero(j: &Jacobian) -> bool {
    let sq = j.linear[0] * j.linear[0]
        + j.linear[1] * j.linear[1]
        + j.linear[2] * j.linear[2]
        + j.angular[0] * j.angular[0]
        + j.angular[1] * j.angular[1]
        + j.angular[2] * j.angular[2];
    sq < f32::EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let j = new_jacobian([1.0, 0.0, 0.0], [0.0; 3]);
        assert!((j.linear[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_dot() {
        let j = new_jacobian([1.0, 0.0, 0.0], [0.0; 3]);
        let v = [2.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!((jacobian_dot(&j, &v) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply() {
        let j = new_jacobian([1.0, 2.0, 0.0], [0.0; 3]);
        let imp = jacobian_apply(&j, 3.0);
        assert!((imp[0] - 3.0).abs() < f32::EPSILON);
        assert!((imp[1] - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transpose() {
        let j = new_jacobian([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        let jt = jacobian_transpose(&j);
        assert!((jt.linear[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_multiply_inv_mass() {
        let j = new_jacobian([1.0, 0.0, 0.0], [0.0; 3]);
        let em = jacobian_multiply_inv_mass(&j, 1.0, 1.0);
        assert!((em - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_mass() {
        let j = new_jacobian([1.0, 0.0, 0.0], [0.0; 3]);
        let em = jacobian_effective_mass(&j, 1.0, 1.0);
        assert!((em - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_to_array() {
        let j = new_jacobian([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        let arr = jacobian_to_array(&j);
        assert!((arr[3] - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_zero() {
        let j = new_jacobian([0.0; 3], [0.0; 3]);
        assert!(jacobian_is_zero(&j));
    }

    #[test]
    fn test_not_zero() {
        let j = new_jacobian([1.0, 0.0, 0.0], [0.0; 3]);
        assert!(!jacobian_is_zero(&j));
    }

    #[test]
    fn test_dot_angular() {
        let j = new_jacobian([0.0; 3], [0.0, 1.0, 0.0]);
        let v = [0.0, 0.0, 0.0, 0.0, 5.0, 0.0];
        assert!((jacobian_dot(&j, &v) - 5.0).abs() < f32::EPSILON);
    }
}
