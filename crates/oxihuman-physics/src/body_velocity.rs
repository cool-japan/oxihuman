#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BodyVelocity {
    linear: [f32; 3],
    angular: [f32; 3],
}

#[allow(dead_code)]
pub fn new_body_velocity() -> BodyVelocity {
    BodyVelocity {
        linear: [0.0; 3],
        angular: [0.0; 3],
    }
}

#[allow(dead_code)]
pub fn set_linear(bv: &mut BodyVelocity, x: f32, y: f32, z: f32) {
    bv.linear = [x, y, z];
}

#[allow(dead_code)]
pub fn set_angular(bv: &mut BodyVelocity, x: f32, y: f32, z: f32) {
    bv.angular = [x, y, z];
}

#[allow(dead_code)]
pub fn linear_velocity(bv: &BodyVelocity) -> [f32; 3] {
    bv.linear
}

#[allow(dead_code)]
pub fn angular_velocity_bv(bv: &BodyVelocity) -> [f32; 3] {
    bv.angular
}

#[allow(dead_code)]
pub fn kinetic_energy_bv(bv: &BodyVelocity, mass: f32) -> f32 {
    let v2 = bv.linear[0] * bv.linear[0]
        + bv.linear[1] * bv.linear[1]
        + bv.linear[2] * bv.linear[2];
    0.5 * mass * v2
}

#[allow(dead_code)]
pub fn velocity_magnitude_bv(bv: &BodyVelocity) -> f32 {
    let v2 = bv.linear[0] * bv.linear[0]
        + bv.linear[1] * bv.linear[1]
        + bv.linear[2] * bv.linear[2];
    v2.sqrt()
}

#[allow(dead_code)]
pub fn velocity_reset_bv(bv: &mut BodyVelocity) {
    bv.linear = [0.0; 3];
    bv.angular = [0.0; 3];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bv = new_body_velocity();
        assert_eq!(linear_velocity(&bv), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_set_linear() {
        let mut bv = new_body_velocity();
        set_linear(&mut bv, 1.0, 2.0, 3.0);
        assert_eq!(linear_velocity(&bv), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_set_angular() {
        let mut bv = new_body_velocity();
        set_angular(&mut bv, 0.1, 0.2, 0.3);
        assert_eq!(angular_velocity_bv(&bv), [0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_kinetic_energy() {
        let mut bv = new_body_velocity();
        set_linear(&mut bv, 3.0, 4.0, 0.0);
        let ke = kinetic_energy_bv(&bv, 2.0);
        assert!((ke - 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_velocity_magnitude() {
        let mut bv = new_body_velocity();
        set_linear(&mut bv, 3.0, 4.0, 0.0);
        assert!((velocity_magnitude_bv(&bv) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut bv = new_body_velocity();
        set_linear(&mut bv, 1.0, 1.0, 1.0);
        set_angular(&mut bv, 1.0, 1.0, 1.0);
        velocity_reset_bv(&mut bv);
        assert_eq!(linear_velocity(&bv), [0.0, 0.0, 0.0]);
        assert_eq!(angular_velocity_bv(&bv), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_zero_mass_ke() {
        let bv = new_body_velocity();
        assert_eq!(kinetic_energy_bv(&bv, 0.0), 0.0);
    }

    #[test]
    fn test_zero_velocity_magnitude() {
        let bv = new_body_velocity();
        assert_eq!(velocity_magnitude_bv(&bv), 0.0);
    }

    #[test]
    fn test_negative_velocity() {
        let mut bv = new_body_velocity();
        set_linear(&mut bv, -3.0, -4.0, 0.0);
        assert!((velocity_magnitude_bv(&bv) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_angular_independent() {
        let mut bv = new_body_velocity();
        set_angular(&mut bv, 5.0, 0.0, 0.0);
        assert_eq!(velocity_magnitude_bv(&bv), 0.0);
    }
}
