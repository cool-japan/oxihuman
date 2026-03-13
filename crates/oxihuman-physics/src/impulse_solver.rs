//! Impulse-based rigid body collision response.

#[allow(dead_code)]
pub struct ImpulseBody {
    pub id: u32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub restitution: f32,
    pub inv_mass: f32,
}

#[allow(dead_code)]
pub struct ImpulseContact {
    pub body_a: u32,
    pub body_b: u32,
    pub contact_point: [f32; 3],
    pub normal: [f32; 3],
    pub penetration: f32,
}

#[allow(dead_code)]
pub struct ImpulseSolver {
    pub bodies: Vec<ImpulseBody>,
    pub restitution_threshold: f32,
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l > 1e-10 {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[allow(dead_code)]
pub fn new_impulse_solver() -> ImpulseSolver {
    ImpulseSolver {
        bodies: Vec::new(),
        restitution_threshold: 1e-4,
    }
}

#[allow(dead_code)]
pub fn add_impulse_body(
    solver: &mut ImpulseSolver,
    pos: [f32; 3],
    mass: f32,
    restitution: f32,
) -> u32 {
    let id = solver.bodies.len() as u32;
    let inv_mass = if mass > 1e-10 { 1.0 / mass } else { 0.0 };
    solver.bodies.push(ImpulseBody {
        id,
        position: pos,
        velocity: [0.0, 0.0, 0.0],
        mass,
        restitution,
        inv_mass,
    });
    id
}

#[allow(dead_code)]
pub fn apply_impulse_to_body(body: &mut ImpulseBody, impulse: [f32; 3]) {
    body.velocity = add3(body.velocity, scale3(impulse, body.inv_mass));
}

#[allow(dead_code)]
pub fn relative_velocity_at_contact(a: &ImpulseBody, b: &ImpulseBody, normal: [f32; 3]) -> f32 {
    let rel_vel = sub3(a.velocity, b.velocity);
    dot3(rel_vel, normal)
}

#[allow(dead_code)]
pub fn compute_impulse_magnitude(
    inv_mass_a: f32,
    inv_mass_b: f32,
    rv: f32,
    restitution: f32,
) -> f32 {
    let denom = inv_mass_a + inv_mass_b;
    if denom < 1e-10 {
        return 0.0;
    }
    -(1.0 + restitution) * rv / denom
}

#[allow(dead_code)]
pub fn resolve_impulse_contact(solver: &mut ImpulseSolver, contact: &ImpulseContact) {
    let idx_a = solver.bodies.iter().position(|b| b.id == contact.body_a);
    let idx_b = solver.bodies.iter().position(|b| b.id == contact.body_b);

    let (Some(ia), Some(ib)) = (idx_a, idx_b) else {
        return;
    };

    let rv = {
        let a = &solver.bodies[ia];
        let b = &solver.bodies[ib];
        relative_velocity_at_contact(a, b, contact.normal)
    };

    if rv >= 0.0 {
        return;
    }

    let (inv_a, inv_b, rest) = {
        let a = &solver.bodies[ia];
        let b = &solver.bodies[ib];
        let restitution = a.restitution.min(b.restitution);
        (a.inv_mass, b.inv_mass, restitution)
    };

    let j = compute_impulse_magnitude(inv_a, inv_b, rv, rest);
    let impulse = scale3(contact.normal, j);
    let neg_impulse = scale3(contact.normal, -j);

    apply_impulse_to_body(&mut solver.bodies[ia], impulse);
    apply_impulse_to_body(&mut solver.bodies[ib], neg_impulse);
}

#[allow(dead_code)]
pub fn integrate_impulse_bodies(solver: &mut ImpulseSolver, dt: f32, gravity: [f32; 3]) {
    for body in solver.bodies.iter_mut() {
        if body.inv_mass > 1e-10 {
            body.velocity = add3(body.velocity, scale3(gravity, dt));
            body.position = add3(body.position, scale3(body.velocity, dt));
        }
    }
}

#[allow(dead_code)]
pub fn sphere_sphere_impulse_contact(
    a: &ImpulseBody,
    b: &ImpulseBody,
    radius_a: f32,
    radius_b: f32,
) -> Option<ImpulseContact> {
    let diff = sub3(a.position, b.position);
    let dist = len3(diff);
    let combined = radius_a + radius_b;

    if dist < combined && dist > 1e-10 {
        let normal = normalize3(diff);
        let penetration = combined - dist;
        let contact_point = add3(b.position, scale3(normal, radius_b + penetration * 0.5));
        Some(ImpulseContact {
            body_a: a.id,
            body_b: b.id,
            contact_point,
            normal,
            penetration,
        })
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn separate_impulse_bodies(solver: &mut ImpulseSolver, contact: &ImpulseContact) {
    let idx_a = solver.bodies.iter().position(|b| b.id == contact.body_a);
    let idx_b = solver.bodies.iter().position(|b| b.id == contact.body_b);

    let (Some(ia), Some(ib)) = (idx_a, idx_b) else {
        return;
    };

    let (inv_a, inv_b) = (solver.bodies[ia].inv_mass, solver.bodies[ib].inv_mass);
    let total_inv = inv_a + inv_b;
    if total_inv < 1e-10 {
        return;
    }
    let correction = contact.penetration / total_inv;
    let push = scale3(contact.normal, correction);
    solver.bodies[ia].position = add3(solver.bodies[ia].position, scale3(push, inv_a));
    solver.bodies[ib].position = sub3(solver.bodies[ib].position, scale3(push, inv_b));
}

#[allow(dead_code)]
pub fn impulse_body_by_id(solver: &ImpulseSolver, id: u32) -> Option<&ImpulseBody> {
    solver.bodies.iter().find(|b| b.id == id)
}

#[allow(dead_code)]
pub fn impulse_body_count(solver: &ImpulseSolver) -> usize {
    solver.bodies.len()
}

#[allow(dead_code)]
pub fn total_impulse_kinetic_energy(solver: &ImpulseSolver) -> f32 {
    solver
        .bodies
        .iter()
        .map(|b| {
            let v2 = dot3(b.velocity, b.velocity);
            0.5 * b.mass * v2
        })
        .sum()
}

#[allow(dead_code)]
pub fn remove_impulse_body(solver: &mut ImpulseSolver, id: u32) -> bool {
    if let Some(pos) = solver.bodies.iter().position(|b| b.id == id) {
        solver.bodies.remove(pos);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_impulse_solver() {
        let solver = new_impulse_solver();
        assert_eq!(impulse_body_count(&solver), 0);
    }

    #[test]
    fn test_add_impulse_body() {
        let mut solver = new_impulse_solver();
        let id = add_impulse_body(&mut solver, [0.0, 0.0, 0.0], 1.0, 0.5);
        assert_eq!(id, 0);
        assert_eq!(impulse_body_count(&solver), 1);
    }

    #[test]
    fn test_add_multiple_bodies() {
        let mut solver = new_impulse_solver();
        add_impulse_body(&mut solver, [0.0, 0.0, 0.0], 1.0, 0.5);
        let id = add_impulse_body(&mut solver, [1.0, 0.0, 0.0], 2.0, 0.3);
        assert_eq!(id, 1);
        assert_eq!(impulse_body_count(&solver), 2);
    }

    #[test]
    fn test_body_inv_mass() {
        let mut solver = new_impulse_solver();
        add_impulse_body(&mut solver, [0.0, 0.0, 0.0], 2.0, 0.5);
        assert!((solver.bodies[0].inv_mass - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_impulse_to_body() {
        let mut body = ImpulseBody {
            id: 0,
            position: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            mass: 1.0,
            restitution: 0.5,
            inv_mass: 1.0,
        };
        apply_impulse_to_body(&mut body, [1.0, 0.0, 0.0]);
        assert!((body.velocity[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_relative_velocity_at_contact() {
        let a = ImpulseBody {
            id: 0,
            position: [0.0; 3],
            velocity: [-1.0, 0.0, 0.0],
            mass: 1.0,
            restitution: 0.5,
            inv_mass: 1.0,
        };
        let b = ImpulseBody {
            id: 1,
            position: [1.0, 0.0, 0.0],
            velocity: [1.0, 0.0, 0.0],
            mass: 1.0,
            restitution: 0.5,
            inv_mass: 1.0,
        };
        let rv = relative_velocity_at_contact(&a, &b, [1.0, 0.0, 0.0]);
        assert!((rv - (-2.0)).abs() < 1e-5);
    }

    #[test]
    fn test_compute_impulse_magnitude() {
        let j = compute_impulse_magnitude(1.0, 1.0, -2.0, 1.0);
        assert!((j - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_sphere_sphere_contact_colliding() {
        let a = ImpulseBody {
            id: 0,
            position: [0.0; 3],
            velocity: [0.0; 3],
            mass: 1.0,
            restitution: 0.5,
            inv_mass: 1.0,
        };
        let b = ImpulseBody {
            id: 1,
            position: [1.5, 0.0, 0.0],
            velocity: [0.0; 3],
            mass: 1.0,
            restitution: 0.5,
            inv_mass: 1.0,
        };
        let contact = sphere_sphere_impulse_contact(&a, &b, 1.0, 1.0);
        assert!(contact.is_some());
    }

    #[test]
    fn test_sphere_sphere_contact_separated() {
        let a = ImpulseBody {
            id: 0,
            position: [0.0; 3],
            velocity: [0.0; 3],
            mass: 1.0,
            restitution: 0.5,
            inv_mass: 1.0,
        };
        let b = ImpulseBody {
            id: 1,
            position: [5.0, 0.0, 0.0],
            velocity: [0.0; 3],
            mass: 1.0,
            restitution: 0.5,
            inv_mass: 1.0,
        };
        let contact = sphere_sphere_impulse_contact(&a, &b, 1.0, 1.0);
        assert!(contact.is_none());
    }

    #[test]
    fn test_resolve_impulse_contact_separates_velocities() {
        let mut solver = new_impulse_solver();
        let id_a = add_impulse_body(&mut solver, [0.0, 0.0, 0.0], 1.0, 1.0);
        let id_b = add_impulse_body(&mut solver, [1.5, 0.0, 0.0], 1.0, 1.0);
        // body_a moves in -x (toward b), body_b moves in +x (toward a)
        // normal points from b to a: [-1, 0, 0]
        // rv = dot(a.vel - b.vel, normal) = dot([-1-1, 0, 0], [-1,0,0]) = 2 > 0, would skip
        // So: normal = [-1,0,0], a.vel = [-1,0,0], b.vel = [1,0,0]
        // rv = dot([-1-1,0,0],[-1,0,0]) = dot([-2,0,0],[-1,0,0]) = 2 > 0, still skips
        // Correct setup: a is at +x side, b at -x side, normal = +x (from a to b direction of separation)
        // For resolve to trigger, rv must be < 0: bodies approaching.
        // rv = dot(a.vel - b.vel, normal). With normal=[1,0,0]:
        //   If a.vel=[- 1,0,0], b.vel=[+1,0,0]: rv = dot([-2,0,0],[1,0,0]) = -2 < 0. triggers!
        solver.bodies[0].velocity = [-1.0, 0.0, 0.0];
        solver.bodies[1].velocity = [1.0, 0.0, 0.0];

        let v_a_before = solver.bodies[0].velocity[0];
        let v_b_before = solver.bodies[1].velocity[0];

        let contact = ImpulseContact {
            body_a: id_a,
            body_b: id_b,
            contact_point: [0.75, 0.0, 0.0],
            normal: [1.0, 0.0, 0.0],
            penetration: 0.5,
        };
        resolve_impulse_contact(&mut solver, &contact);
        // After resolution velocities should change (impulse was applied)
        let v_a_after = solver.bodies[0].velocity[0];
        let v_b_after = solver.bodies[1].velocity[0];
        assert!(
            (v_a_after - v_a_before).abs() > 1e-5 || (v_b_after - v_b_before).abs() > 1e-5,
            "Impulse should change velocities"
        );
    }

    #[test]
    fn test_impulse_body_by_id() {
        let mut solver = new_impulse_solver();
        add_impulse_body(&mut solver, [1.0, 2.0, 3.0], 2.0, 0.5);
        let body = impulse_body_by_id(&solver, 0);
        assert!(body.is_some());
        assert!((body.expect("should succeed").position[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_impulse_body_by_id_not_found() {
        let solver = new_impulse_solver();
        assert!(impulse_body_by_id(&solver, 99).is_none());
    }

    #[test]
    fn test_total_kinetic_energy() {
        let mut solver = new_impulse_solver();
        add_impulse_body(&mut solver, [0.0; 3], 2.0, 0.5);
        solver.bodies[0].velocity = [2.0, 0.0, 0.0];
        let ke = total_impulse_kinetic_energy(&solver);
        assert!((ke - 4.0).abs() < 1e-5); // 0.5 * 2 * 4 = 4
    }

    #[test]
    fn test_remove_impulse_body() {
        let mut solver = new_impulse_solver();
        let id = add_impulse_body(&mut solver, [0.0; 3], 1.0, 0.5);
        let removed = remove_impulse_body(&mut solver, id);
        assert!(removed);
        assert_eq!(impulse_body_count(&solver), 0);
    }

    #[test]
    fn test_remove_nonexistent_body() {
        let mut solver = new_impulse_solver();
        let removed = remove_impulse_body(&mut solver, 99);
        assert!(!removed);
    }

    #[test]
    fn test_integrate_moves_body() {
        let mut solver = new_impulse_solver();
        add_impulse_body(&mut solver, [0.0, 10.0, 0.0], 1.0, 0.5);
        solver.bodies[0].velocity = [0.0, 0.0, 0.0];
        integrate_impulse_bodies(&mut solver, 1.0, [0.0, -9.81, 0.0]);
        assert!(
            solver.bodies[0].position[1] < 10.0,
            "Body should fall under gravity"
        );
    }
}
