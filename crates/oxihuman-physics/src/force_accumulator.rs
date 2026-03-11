#![allow(dead_code)]

/// Accumulates forces and torques for a rigid body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForceAccumulator {
    pub force: [f32; 3],
    pub torque: [f32; 3],
}

/// Creates a new zeroed force accumulator.
#[allow(dead_code)]
pub fn new_force_accumulator() -> ForceAccumulator {
    ForceAccumulator {
        force: [0.0; 3],
        torque: [0.0; 3],
    }
}

/// Adds a force vector.
#[allow(dead_code)]
pub fn add_force(acc: &mut ForceAccumulator, force: [f32; 3]) {
    acc.force[0] += force[0];
    acc.force[1] += force[1];
    acc.force[2] += force[2];
}

/// Adds a torque vector.
#[allow(dead_code)]
pub fn add_torque(acc: &mut ForceAccumulator, torque: [f32; 3]) {
    acc.torque[0] += torque[0];
    acc.torque[1] += torque[1];
    acc.torque[2] += torque[2];
}

/// Returns the total force.
#[allow(dead_code)]
pub fn total_force(acc: &ForceAccumulator) -> [f32; 3] {
    acc.force
}

/// Returns the total torque.
#[allow(dead_code)]
pub fn total_torque(acc: &ForceAccumulator) -> [f32; 3] {
    acc.torque
}

/// Clears all accumulated forces and torques.
#[allow(dead_code)]
pub fn clear_accumulator(acc: &mut ForceAccumulator) {
    acc.force = [0.0; 3];
    acc.torque = [0.0; 3];
}

/// Applies gravity as a force (mass * g).
#[allow(dead_code)]
pub fn apply_gravity(acc: &mut ForceAccumulator, mass: f32, gravity: [f32; 3]) {
    acc.force[0] += mass * gravity[0];
    acc.force[1] += mass * gravity[1];
    acc.force[2] += mass * gravity[2];
}

/// Returns true if all forces and torques are zero.
#[allow(dead_code)]
pub fn accumulator_is_zero(acc: &ForceAccumulator) -> bool {
    let f_sq = acc.force[0] * acc.force[0] + acc.force[1] * acc.force[1] + acc.force[2] * acc.force[2];
    let t_sq = acc.torque[0] * acc.torque[0] + acc.torque[1] * acc.torque[1] + acc.torque[2] * acc.torque[2];
    f_sq < f32::EPSILON && t_sq < f32::EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let acc = new_force_accumulator();
        assert!(accumulator_is_zero(&acc));
    }

    #[test]
    fn test_add_force() {
        let mut acc = new_force_accumulator();
        add_force(&mut acc, [1.0, 0.0, 0.0]);
        let f = total_force(&acc);
        assert!((f[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_add_torque() {
        let mut acc = new_force_accumulator();
        add_torque(&mut acc, [0.0, 1.0, 0.0]);
        let t = total_torque(&acc);
        assert!((t[1] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_multiple_forces() {
        let mut acc = new_force_accumulator();
        add_force(&mut acc, [1.0, 0.0, 0.0]);
        add_force(&mut acc, [2.0, 0.0, 0.0]);
        let f = total_force(&acc);
        assert!((f[0] - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clear() {
        let mut acc = new_force_accumulator();
        add_force(&mut acc, [10.0; 3]);
        clear_accumulator(&mut acc);
        assert!(accumulator_is_zero(&acc));
    }

    #[test]
    fn test_gravity() {
        let mut acc = new_force_accumulator();
        apply_gravity(&mut acc, 2.0, [0.0, -9.81, 0.0]);
        let f = total_force(&acc);
        assert!((f[1] - (-19.62)).abs() < 0.01);
    }

    #[test]
    fn test_is_zero_after_add() {
        let mut acc = new_force_accumulator();
        add_force(&mut acc, [1.0, 0.0, 0.0]);
        assert!(!accumulator_is_zero(&acc));
    }

    #[test]
    fn test_force_and_torque() {
        let mut acc = new_force_accumulator();
        add_force(&mut acc, [1.0, 2.0, 3.0]);
        add_torque(&mut acc, [4.0, 5.0, 6.0]);
        assert!(!accumulator_is_zero(&acc));
    }

    #[test]
    fn test_total_torque_zero() {
        let acc = new_force_accumulator();
        let t = total_torque(&acc);
        assert!((t[0]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_gravity_zero_mass() {
        let mut acc = new_force_accumulator();
        apply_gravity(&mut acc, 0.0, [0.0, -9.81, 0.0]);
        assert!(accumulator_is_zero(&acc));
    }
}
