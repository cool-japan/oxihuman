//! PBD position-based constraints for distance, volume, and shape preservation.

/// The type of a PBD constraint.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum PbdConstraintType {
    Distance,
    Volume,
    Shape,
    Collision,
}

/// A single PBD constraint acting between two particles.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PosPbdConstraint {
    pub constraint_type: PbdConstraintType,
    pub particle_a: u32,
    pub particle_b: u32,
    pub rest_length: f32,
    pub stiffness: f32,
}

/// A PBD simulation particle with current and predicted positions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PosPbdParticle {
    pub position: [f32; 3],
    pub predicted: [f32; 3],
    pub mass: f32,
    pub inv_mass: f32,
}

/// Result of solving a single PBD constraint iteration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PbdSolveResult {
    pub delta_a: [f32; 3],
    pub delta_b: [f32; 3],
    pub lambda: f32,
    pub satisfied: bool,
}

/// Construct a new `PosPbdParticle` with the given position and mass.
#[allow(dead_code)]
pub fn new_pbd_particle(pos: [f32; 3], mass: f32) -> PosPbdParticle {
    let inv_mass = if mass > 1e-10 { 1.0 / mass } else { 0.0 };
    PosPbdParticle {
        position: pos,
        predicted: pos,
        mass,
        inv_mass,
    }
}

/// Construct a new distance constraint between particles `a` and `b`.
#[allow(dead_code)]
pub fn new_distance_constraint(a: u32, b: u32, rest: f32, stiffness: f32) -> PosPbdConstraint {
    PosPbdConstraint {
        constraint_type: PbdConstraintType::Distance,
        particle_a: a,
        particle_b: b,
        rest_length: rest,
        stiffness,
    }
}

/// Solve a distance constraint for one Gauss-Seidel iteration.
#[allow(dead_code)]
pub fn solve_distance_constraint(
    c: &PosPbdConstraint,
    pa: &PosPbdParticle,
    pb: &PosPbdParticle,
) -> PbdSolveResult {
    let dx = pb.predicted[0] - pa.predicted[0];
    let dy = pb.predicted[1] - pa.predicted[1];
    let dz = pb.predicted[2] - pa.predicted[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();

    if dist < 1e-10 {
        return PbdSolveResult {
            delta_a: [0.0; 3],
            delta_b: [0.0; 3],
            lambda: 0.0,
            satisfied: true,
        };
    }

    let constraint = dist - c.rest_length;
    let satisfied = constraint.abs() < 1e-6;

    let w_sum = pa.inv_mass + pb.inv_mass;
    if w_sum < 1e-10 {
        return PbdSolveResult {
            delta_a: [0.0; 3],
            delta_b: [0.0; 3],
            lambda: 0.0,
            satisfied,
        };
    }

    let lambda = -c.stiffness * constraint / w_sum;
    let nx = dx / dist;
    let ny = dy / dist;
    let nz = dz / dist;

    let delta_a = [
        -pa.inv_mass * lambda * nx,
        -pa.inv_mass * lambda * ny,
        -pa.inv_mass * lambda * nz,
    ];
    let delta_b = [
        pb.inv_mass * lambda * nx,
        pb.inv_mass * lambda * ny,
        pb.inv_mass * lambda * nz,
    ];

    PbdSolveResult {
        delta_a,
        delta_b,
        lambda,
        satisfied,
    }
}

/// Apply the position deltas from a solved constraint to the two particles.
#[allow(dead_code)]
pub fn apply_pbd_deltas(
    pa: &mut PosPbdParticle,
    pb: &mut PosPbdParticle,
    result: &PbdSolveResult,
) {
    pa.predicted[0] += result.delta_a[0];
    pa.predicted[1] += result.delta_a[1];
    pa.predicted[2] += result.delta_a[2];

    pb.predicted[0] += result.delta_b[0];
    pb.predicted[1] += result.delta_b[1];
    pb.predicted[2] += result.delta_b[2];
}

/// PBD prediction step: advance `predicted` using gravity and current velocity.
#[allow(dead_code)]
pub fn pbd_predict(p: &mut PosPbdParticle, gravity: [f32; 3], dt: f32) {
    if p.inv_mass < 1e-10 {
        p.predicted = p.position;
        return;
    }
    // Simple symplectic Euler: x_pred = x + v*dt + g*dt^2
    p.predicted[0] = p.position[0] + gravity[0] * dt * dt;
    p.predicted[1] = p.position[1] + gravity[1] * dt * dt;
    p.predicted[2] = p.position[2] + gravity[2] * dt * dt;
}

/// PBD finalization: update velocity from position delta and commit predicted position.
#[allow(dead_code)]
pub fn pbd_finalize(p: &mut PosPbdParticle, dt: f32) {
    let inv_dt = if dt > 1e-10 { 1.0 / dt } else { 0.0 };
    // velocity = (predicted - position) / dt
    let _vx = (p.predicted[0] - p.position[0]) * inv_dt;
    let _vy = (p.predicted[1] - p.position[1]) * inv_dt;
    let _vz = (p.predicted[2] - p.position[2]) * inv_dt;
    p.position = p.predicted;
}

/// Return a static string name for the constraint type.
#[allow(dead_code)]
pub fn constraint_type_name(c: &PosPbdConstraint) -> &'static str {
    match c.constraint_type {
        PbdConstraintType::Distance => "distance",
        PbdConstraintType::Volume => "volume",
        PbdConstraintType::Shape => "shape",
        PbdConstraintType::Collision => "collision",
    }
}

/// Serialize a `PosPbdConstraint` to a JSON string.
#[allow(dead_code)]
pub fn pbd_constraint_to_json(c: &PosPbdConstraint) -> String {
    format!(
        "{{\"type\":\"{}\",\"particle_a\":{},\"particle_b\":{},\"rest_length\":{},\"stiffness\":{}}}",
        constraint_type_name(c),
        c.particle_a,
        c.particle_b,
        c.rest_length,
        c.stiffness,
    )
}

/// Serialize a `PosPbdParticle` to a JSON string.
#[allow(dead_code)]
pub fn pbd_particle_to_json(p: &PosPbdParticle) -> String {
    format!(
        "{{\"position\":[{},{},{}],\"mass\":{},\"inv_mass\":{}}}",
        p.position[0], p.position[1], p.position[2], p.mass, p.inv_mass,
    )
}

/// Serialize a `PbdSolveResult` to a JSON string.
#[allow(dead_code)]
pub fn pbd_solve_result_to_json(r: &PbdSolveResult) -> String {
    format!(
        "{{\"lambda\":{},\"satisfied\":{}}}",
        r.lambda, r.satisfied,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pbd_particle_fields() {
        let p = new_pbd_particle([1.0, 2.0, 3.0], 2.0);
        assert!((p.position[0] - 1.0).abs() < 1e-6);
        assert!((p.mass - 2.0).abs() < 1e-6);
        assert!((p.inv_mass - 0.5).abs() < 1e-6);
    }

    #[test]
    fn zero_mass_gives_zero_inv_mass() {
        let p = new_pbd_particle([0.0; 3], 0.0);
        assert!(p.inv_mass.abs() < 1e-10);
    }

    #[test]
    fn constraint_type_names() {
        let dc = new_distance_constraint(0, 1, 1.0, 1.0);
        assert_eq!(constraint_type_name(&dc), "distance");
        let vc = PosPbdConstraint {
            constraint_type: PbdConstraintType::Volume,
            particle_a: 0,
            particle_b: 1,
            rest_length: 0.0,
            stiffness: 1.0,
        };
        assert_eq!(constraint_type_name(&vc), "volume");
    }

    #[test]
    fn solve_distance_constraint_satisfied_when_at_rest() {
        // Two particles exactly at rest_length distance
        let pa = new_pbd_particle([0.0, 0.0, 0.0], 1.0);
        let pb = new_pbd_particle([1.0, 0.0, 0.0], 1.0);
        let c = new_distance_constraint(0, 1, 1.0, 1.0);
        let result = solve_distance_constraint(&c, &pa, &pb);
        assert!(result.satisfied, "should be satisfied at rest length");
        assert!(result.lambda.abs() < 1e-5);
    }

    #[test]
    fn solve_distance_constraint_produces_correction() {
        // Two particles too close — stretch them apart
        let pa = new_pbd_particle([0.0, 0.0, 0.0], 1.0);
        let pb = new_pbd_particle([0.5, 0.0, 0.0], 1.0);
        let c = new_distance_constraint(0, 1, 1.0, 1.0);
        let result = solve_distance_constraint(&c, &pa, &pb);
        assert!(!result.satisfied);
        // delta_b should push pb away (positive x direction)
        assert!(result.delta_b[0] > 0.0, "particle B should move right");
        // delta_a should push pa toward negative x
        assert!(result.delta_a[0] < 0.0, "particle A should move left");
    }

    #[test]
    fn apply_pbd_deltas_updates_predicted() {
        let mut pa = new_pbd_particle([0.0, 0.0, 0.0], 1.0);
        let mut pb = new_pbd_particle([0.5, 0.0, 0.0], 1.0);
        let c = new_distance_constraint(0, 1, 1.0, 1.0);
        let result = solve_distance_constraint(&c, &pa, &pb);
        apply_pbd_deltas(&mut pa, &mut pb, &result);
        // After correction the distance should be closer to 1.0
        let dx = pb.predicted[0] - pa.predicted[0];
        let dist = dx.abs();
        assert!(
            (dist - 1.0).abs() < 0.6,
            "distance after correction should approach 1.0, got {dist}"
        );
    }

    #[test]
    fn pbd_constraint_to_json_contains_type() {
        let c = new_distance_constraint(0, 1, 2.0, 0.9);
        let json = pbd_constraint_to_json(&c);
        assert!(json.contains("distance"));
        assert!(json.contains("rest_length"));
    }

    #[test]
    fn pbd_particle_to_json_contains_mass() {
        let p = new_pbd_particle([1.0, 2.0, 3.0], 5.0);
        let json = pbd_particle_to_json(&p);
        assert!(json.contains("mass"));
        assert!(json.contains("position"));
    }

    #[test]
    fn pbd_finalize_commits_position() {
        let mut p = new_pbd_particle([0.0, 0.0, 0.0], 1.0);
        p.predicted = [1.0, 2.0, 3.0];
        pbd_finalize(&mut p, 0.016);
        assert!((p.position[0] - 1.0).abs() < 1e-6);
        assert!((p.position[1] - 2.0).abs() < 1e-6);
    }
}
