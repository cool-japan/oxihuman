//! Physics-driven secondary expression dynamics (jiggle/spring for cheeks, jaw, etc.).

#[allow(dead_code)]
pub struct SpringJoint {
    pub name: String,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub rest_position: [f32; 3],
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

#[allow(dead_code)]
pub struct ExpressionPhysics {
    pub joints: Vec<SpringJoint>,
    pub gravity: [f32; 3],
    pub enabled: bool,
}

#[allow(dead_code)]
pub struct PhysicsExpressionResult {
    pub deltas: Vec<[f32; 3]>,
    pub kinetic_energy: f32,
}

#[allow(dead_code)]
pub fn new_expression_physics(gravity: [f32; 3]) -> ExpressionPhysics {
    ExpressionPhysics {
        joints: Vec::new(),
        gravity,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn add_spring_joint(
    ep: &mut ExpressionPhysics,
    name: &str,
    rest_pos: [f32; 3],
    stiffness: f32,
    damping: f32,
    mass: f32,
) -> usize {
    let idx = ep.joints.len();
    ep.joints.push(SpringJoint {
        name: name.to_string(),
        position: rest_pos,
        velocity: [0.0; 3],
        rest_position: rest_pos,
        stiffness,
        damping,
        mass,
    });
    idx
}

#[allow(dead_code)]
pub fn step_expression_physics(ep: &mut ExpressionPhysics, dt: f32) {
    if !ep.enabled {
        return;
    }
    let gravity = ep.gravity;
    for joint in &mut ep.joints {
        let m = if joint.mass > 0.0 { joint.mass } else { 1.0 };
        let force = [
            -joint.stiffness * (joint.position[0] - joint.rest_position[0])
                - joint.damping * joint.velocity[0]
                + gravity[0],
            -joint.stiffness * (joint.position[1] - joint.rest_position[1])
                - joint.damping * joint.velocity[1]
                + gravity[1],
            -joint.stiffness * (joint.position[2] - joint.rest_position[2])
                - joint.damping * joint.velocity[2]
                + gravity[2],
        ];
        joint.velocity[0] += (force[0] / m) * dt;
        joint.velocity[1] += (force[1] / m) * dt;
        joint.velocity[2] += (force[2] / m) * dt;
        joint.position[0] += joint.velocity[0] * dt;
        joint.position[1] += joint.velocity[1] * dt;
        joint.position[2] += joint.velocity[2] * dt;
    }
}

#[allow(dead_code)]
pub fn set_rest_position(ep: &mut ExpressionPhysics, idx: usize, pos: [f32; 3]) {
    if idx < ep.joints.len() {
        ep.joints[idx].rest_position = pos;
    }
}

#[allow(dead_code)]
pub fn apply_impulse_to_joint(ep: &mut ExpressionPhysics, idx: usize, impulse: [f32; 3]) {
    if idx < ep.joints.len() {
        let joint = &mut ep.joints[idx];
        let m = if joint.mass > 0.0 { joint.mass } else { 1.0 };
        joint.velocity[0] += impulse[0] / m;
        joint.velocity[1] += impulse[1] / m;
        joint.velocity[2] += impulse[2] / m;
    }
}

#[allow(dead_code)]
pub fn evaluate_expression_physics(ep: &ExpressionPhysics) -> PhysicsExpressionResult {
    let mut deltas = Vec::with_capacity(ep.joints.len());
    let mut kinetic_energy = 0.0f32;
    for joint in &ep.joints {
        let delta = [
            joint.position[0] - joint.rest_position[0],
            joint.position[1] - joint.rest_position[1],
            joint.position[2] - joint.rest_position[2],
        ];
        deltas.push(delta);
        kinetic_energy += joint_kinetic_energy(joint);
    }
    PhysicsExpressionResult {
        deltas,
        kinetic_energy,
    }
}

#[allow(dead_code)]
pub fn reset_to_rest(ep: &mut ExpressionPhysics) {
    for joint in &mut ep.joints {
        joint.position = joint.rest_position;
        joint.velocity = [0.0; 3];
    }
}

#[allow(dead_code)]
pub fn joint_displacement(joint: &SpringJoint) -> f32 {
    let dx = joint.position[0] - joint.rest_position[0];
    let dy = joint.position[1] - joint.rest_position[1];
    let dz = joint.position[2] - joint.rest_position[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn default_facial_physics() -> ExpressionPhysics {
    let mut ep = new_expression_physics([0.0, -9.81, 0.0]);
    add_spring_joint(&mut ep, "cheek_L", [-0.05, 0.0, 0.03], 80.0, 8.0, 0.01);
    add_spring_joint(&mut ep, "cheek_R", [0.05, 0.0, 0.03], 80.0, 8.0, 0.01);
    add_spring_joint(&mut ep, "jaw", [0.0, -0.03, 0.02], 60.0, 6.0, 0.015);
    add_spring_joint(&mut ep, "nose_tip", [0.0, 0.01, 0.05], 120.0, 12.0, 0.005);
    add_spring_joint(&mut ep, "chin", [0.0, -0.05, 0.02], 70.0, 7.0, 0.012);
    add_spring_joint(&mut ep, "forehead", [0.0, 0.06, 0.01], 100.0, 10.0, 0.008);
    ep
}

#[allow(dead_code)]
pub fn joint_kinetic_energy(joint: &SpringJoint) -> f32 {
    let v_sq = joint.velocity[0] * joint.velocity[0]
        + joint.velocity[1] * joint.velocity[1]
        + joint.velocity[2] * joint.velocity[2];
    0.5 * joint.mass * v_sq
}

#[allow(dead_code)]
pub fn set_enabled(ep: &mut ExpressionPhysics, enabled: bool) {
    ep.enabled = enabled;
}

#[allow(dead_code)]
pub fn joint_count(ep: &ExpressionPhysics) -> usize {
    ep.joints.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_expression_physics() {
        let ep = new_expression_physics([0.0, -9.81, 0.0]);
        assert!(ep.joints.is_empty());
        assert!(ep.enabled);
        assert!((ep.gravity[1] + 9.81).abs() < 1e-5);
    }

    #[test]
    fn test_add_spring_joint() {
        let mut ep = new_expression_physics([0.0; 3]);
        let idx = add_spring_joint(&mut ep, "cheek_L", [0.1, 0.2, 0.3], 100.0, 10.0, 0.01);
        assert_eq!(idx, 0);
        assert_eq!(joint_count(&ep), 1);
        assert_eq!(ep.joints[0].name, "cheek_L");
    }

    #[test]
    fn test_step_changes_position_after_impulse() {
        let mut ep = new_expression_physics([0.0; 3]);
        add_spring_joint(&mut ep, "jaw", [0.0; 3], 10.0, 1.0, 1.0);
        apply_impulse_to_joint(&mut ep, 0, [1.0, 0.0, 0.0]);
        step_expression_physics(&mut ep, 0.016);
        assert!(ep.joints[0].position[0] > 0.0);
    }

    #[test]
    fn test_reset_zeroes_velocity() {
        let mut ep = new_expression_physics([0.0; 3]);
        add_spring_joint(&mut ep, "jaw", [0.0; 3], 10.0, 1.0, 1.0);
        apply_impulse_to_joint(&mut ep, 0, [2.0, 1.0, 0.5]);
        step_expression_physics(&mut ep, 0.1);
        reset_to_rest(&mut ep);
        let j = &ep.joints[0];
        assert_eq!(j.velocity, [0.0; 3]);
        assert_eq!(j.position, j.rest_position);
    }

    #[test]
    fn test_evaluate_returns_correct_count() {
        let mut ep = new_expression_physics([0.0; 3]);
        add_spring_joint(&mut ep, "a", [0.0; 3], 1.0, 0.1, 1.0);
        add_spring_joint(&mut ep, "b", [1.0, 0.0, 0.0], 1.0, 0.1, 1.0);
        let result = evaluate_expression_physics(&ep);
        assert_eq!(result.deltas.len(), 2);
    }

    #[test]
    fn test_kinetic_energy_formula() {
        let joint = SpringJoint {
            name: "test".to_string(),
            position: [0.0; 3],
            velocity: [1.0, 0.0, 0.0],
            rest_position: [0.0; 3],
            stiffness: 10.0,
            damping: 1.0,
            mass: 2.0,
        };
        let ke = joint_kinetic_energy(&joint);
        assert!((ke - 1.0).abs() < 1e-6); // 0.5 * 2.0 * 1.0^2 = 1.0
    }

    #[test]
    fn test_default_facial_physics_count() {
        let ep = default_facial_physics();
        assert_eq!(joint_count(&ep), 6);
    }

    #[test]
    fn test_displacement_formula() {
        let joint = SpringJoint {
            name: "test".to_string(),
            position: [1.0, 0.0, 0.0],
            velocity: [0.0; 3],
            rest_position: [0.0; 3],
            stiffness: 10.0,
            damping: 1.0,
            mass: 1.0,
        };
        let d = joint_displacement(&joint);
        assert!((d - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_rest_position() {
        let mut ep = new_expression_physics([0.0; 3]);
        add_spring_joint(&mut ep, "j", [0.0; 3], 10.0, 1.0, 1.0);
        set_rest_position(&mut ep, 0, [1.0, 2.0, 3.0]);
        assert_eq!(ep.joints[0].rest_position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_apply_impulse_changes_velocity() {
        let mut ep = new_expression_physics([0.0; 3]);
        add_spring_joint(&mut ep, "j", [0.0; 3], 10.0, 1.0, 2.0);
        apply_impulse_to_joint(&mut ep, 0, [2.0, 0.0, 0.0]);
        // impulse / mass = 2.0 / 2.0 = 1.0
        assert!((ep.joints[0].velocity[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_enabled_disables_stepping() {
        let mut ep = new_expression_physics([0.0; 3]);
        add_spring_joint(&mut ep, "j", [0.0; 3], 10.0, 1.0, 1.0);
        apply_impulse_to_joint(&mut ep, 0, [1.0, 0.0, 0.0]);
        set_enabled(&mut ep, false);
        let pos_before = ep.joints[0].position;
        step_expression_physics(&mut ep, 0.1);
        assert_eq!(ep.joints[0].position, pos_before);
    }

    #[test]
    fn test_evaluate_kinetic_energy_at_rest_zero() {
        let ep = new_expression_physics([0.0; 3]);
        let result = evaluate_expression_physics(&ep);
        assert!((result.kinetic_energy).abs() < 1e-9);
    }

    #[test]
    fn test_evaluate_deltas_at_rest_zero() {
        let mut ep = new_expression_physics([0.0; 3]);
        add_spring_joint(&mut ep, "j", [1.0, 2.0, 3.0], 10.0, 1.0, 1.0);
        let result = evaluate_expression_physics(&ep);
        assert_eq!(result.deltas[0], [0.0; 3]);
    }

    #[test]
    fn test_spring_returns_to_rest() {
        let mut ep = new_expression_physics([0.0; 3]);
        // high damping ensures it returns to rest
        add_spring_joint(&mut ep, "j", [0.0; 3], 200.0, 40.0, 1.0);
        apply_impulse_to_joint(&mut ep, 0, [1.0, 0.0, 0.0]);
        for _ in 0..500 {
            step_expression_physics(&mut ep, 0.01);
        }
        let d = joint_displacement(&ep.joints[0]);
        assert!(d < 0.01, "displacement should settle near rest: {}", d);
    }

    #[test]
    fn test_joint_count_empty() {
        let ep = new_expression_physics([0.0; 3]);
        assert_eq!(joint_count(&ep), 0);
    }
}
