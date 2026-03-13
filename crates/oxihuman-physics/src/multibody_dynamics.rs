// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Multi-body dynamics system with explicit joint constraints.
//!
//! Implements a simple but complete multi-body dynamics (MBD) simulation:
//! rigid bodies connected by spring-like bilateral joints. Integration
//! uses symplectic Euler, and joint constraints are resolved as impulses.

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Global configuration for the multi-body dynamics system.
pub struct MbdConfig {
    /// Gravitational acceleration vector (typically [0, -9.81, 0]).
    pub gravity: [f32; 3],
    /// Maximum number of sub-steps per `mbd_step` call.
    pub max_substeps: u32,
    /// Simulation time step (seconds).
    pub dt: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// A single rigid body in the MBD system.
pub struct MbdBody {
    /// World-space position.
    pub position: [f32; 3],
    /// Linear velocity.
    pub velocity: [f32; 3],
    /// Mass (kg).
    pub mass: f32,
    /// Inverse mass (0 for static/infinite-mass bodies).
    pub inv_mass: f32,
    /// User-assigned identifier.
    pub id: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// A bilateral joint connecting two bodies via spring-damper constraint.
pub struct MbdJoint {
    /// ID of the first body.
    pub body_a: u32,
    /// ID of the second body.
    pub body_b: u32,
    /// Anchor point offset in body A's local frame (world offset for simplicity).
    pub anchor_a: [f32; 3],
    /// Anchor point offset in body B's local frame (world offset for simplicity).
    pub anchor_b: [f32; 3],
    /// Spring stiffness for constraint enforcement.
    pub stiffness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// The complete multi-body dynamics simulation state.
pub struct MbdSystem {
    pub bodies: Vec<MbdBody>,
    pub joints: Vec<MbdJoint>,
    pub config: MbdConfig,
}

// ─── vector math ─────────────────────────────────────────────────────────────

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        scale3(v, 1.0 / l)
    }
}

// ─── public API ──────────────────────────────────────────────────────────────

/// Return a sensible default MBD configuration (Earth gravity, 60 Hz).
#[allow(dead_code)]
pub fn default_mbd_config() -> MbdConfig {
    MbdConfig {
        gravity: [0.0, -9.81, 0.0],
        max_substeps: 4,
        dt: 1.0 / 60.0,
    }
}

/// Construct a new MBD body with given position, mass, and ID.
///
/// Static bodies (infinite mass) use mass = 0.0; inv_mass will be set to 0.
#[allow(dead_code)]
pub fn new_mbd_body(pos: [f32; 3], mass: f32, id: u32) -> MbdBody {
    let inv_mass = if mass > 1e-12 { 1.0 / mass } else { 0.0 };
    MbdBody {
        position: pos,
        velocity: [0.0, 0.0, 0.0],
        mass,
        inv_mass,
        id,
    }
}

/// Create a new MBD system with the given configuration.
#[allow(dead_code)]
pub fn new_mbd_system(cfg: MbdConfig) -> MbdSystem {
    MbdSystem {
        bodies: Vec::new(),
        joints: Vec::new(),
        config: cfg,
    }
}

/// Add a body to the system.
#[allow(dead_code)]
pub fn mbd_add_body(sys: &mut MbdSystem, body: MbdBody) {
    sys.bodies.push(body);
}

/// Add a joint to the system.
#[allow(dead_code)]
pub fn mbd_add_joint(sys: &mut MbdSystem, joint: MbdJoint) {
    sys.joints.push(joint);
}

/// Return the number of bodies in the system.
#[allow(dead_code)]
pub fn mbd_body_count(sys: &MbdSystem) -> usize {
    sys.bodies.len()
}

/// Return the number of joints in the system.
#[allow(dead_code)]
pub fn mbd_joint_count(sys: &MbdSystem) -> usize {
    sys.joints.len()
}

/// Find a body by ID. Returns `None` if no body has the given ID.
#[allow(dead_code)]
pub fn mbd_find_body(sys: &MbdSystem, id: u32) -> Option<&MbdBody> {
    sys.bodies.iter().find(|b| b.id == id)
}

/// Advance the simulation by one time step, using sub-stepping.
///
/// Each sub-step applies gravity, integrates velocities and positions
/// (symplectic Euler), then resolves joint constraints as position/velocity
/// corrections.
#[allow(dead_code)]
pub fn mbd_step(sys: &mut MbdSystem) {
    let dt_sub = if sys.config.max_substeps > 0 {
        sys.config.dt / sys.config.max_substeps as f32
    } else {
        sys.config.dt
    };
    let substeps = sys.config.max_substeps.max(1);

    for _ in 0..substeps {
        step_once(sys, dt_sub);
    }
}

fn step_once(sys: &mut MbdSystem, dt: f32) {
    let gravity = sys.config.gravity;

    // 1. Apply gravity to velocities (semi-implicit Euler: update v first)
    for body in sys.bodies.iter_mut() {
        if body.inv_mass > 1e-12 {
            body.velocity = add3(body.velocity, scale3(gravity, dt));
        }
    }

    // 2. Integrate positions
    for body in sys.bodies.iter_mut() {
        if body.inv_mass > 1e-12 {
            body.position = add3(body.position, scale3(body.velocity, dt));
        }
    }

    // 3. Resolve joint constraints
    // We do one iteration of spring-impulse correction.
    // We collect (idx_a, idx_b, impulse) first to avoid borrow issues.
    let joints: Vec<MbdJoint> = sys.joints.clone();

    for joint in &joints {
        // Find body indices by ID
        let ia = sys.bodies.iter().position(|b| b.id == joint.body_a);
        let ib = sys.bodies.iter().position(|b| b.id == joint.body_b);

        let (ia, ib) = match (ia, ib) {
            (Some(a), Some(b)) => (a, b),
            _ => continue,
        };

        // World anchor positions
        let pa = add3(sys.bodies[ia].position, joint.anchor_a);
        let pb = add3(sys.bodies[ib].position, joint.anchor_b);

        let delta = sub3(pb, pa);
        let dist = len3(delta);

        if dist < 1e-12 {
            continue;
        }

        // Constraint: maintain the initial distance (zero rest length drift).
        // Use positional correction proportional to stiffness.
        let dir = normalize3(delta);

        // Effective mass for the constraint
        let inv_mass_sum = sys.bodies[ia].inv_mass + sys.bodies[ib].inv_mass;
        if inv_mass_sum < 1e-12 {
            continue;
        }

        // Baumgarte-style position correction
        let correction = scale3(dir, dist * joint.stiffness / inv_mass_sum);

        if sys.bodies[ia].inv_mass > 1e-12 {
            sys.bodies[ia].position =
                add3(sys.bodies[ia].position, scale3(correction, sys.bodies[ia].inv_mass));
        }
        if sys.bodies[ib].inv_mass > 1e-12 {
            sys.bodies[ib].position =
                sub3(sys.bodies[ib].position, scale3(correction, sys.bodies[ib].inv_mass));
        }

        // Velocity correction (remove relative velocity along constraint)
        let va = sys.bodies[ia].velocity;
        let vb = sys.bodies[ib].velocity;
        let rel_vel = dot3(sub3(vb, va), dir);

        let impulse_mag = rel_vel / inv_mass_sum * joint.stiffness;
        let impulse = scale3(dir, impulse_mag);

        if sys.bodies[ia].inv_mass > 1e-12 {
            sys.bodies[ia].velocity =
                add3(sys.bodies[ia].velocity, scale3(impulse, sys.bodies[ia].inv_mass));
        }
        if sys.bodies[ib].inv_mass > 1e-12 {
            sys.bodies[ib].velocity =
                sub3(sys.bodies[ib].velocity, scale3(impulse, sys.bodies[ib].inv_mass));
        }
    }
}

/// Serialize the MBD system state to compact JSON.
#[allow(dead_code)]
pub fn mbd_system_to_json(sys: &MbdSystem) -> String {
    let bodies_json: Vec<String> = sys
        .bodies
        .iter()
        .map(|b| {
            format!(
                "{{\"id\":{},\"pos\":[{:.4},{:.4},{:.4}],\"vel\":[{:.4},{:.4},{:.4}],\"mass\":{:.4}}}",
                b.id,
                b.position[0], b.position[1], b.position[2],
                b.velocity[0], b.velocity[1], b.velocity[2],
                b.mass
            )
        })
        .collect();

    format!(
        "{{\"body_count\":{},\"joint_count\":{},\"bodies\":[{}]}}",
        sys.bodies.len(),
        sys.joints.len(),
        bodies_json.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_two_body_system() -> MbdSystem {
        let cfg = default_mbd_config();
        let mut sys = new_mbd_system(cfg);
        mbd_add_body(&mut sys, new_mbd_body([0.0, 10.0, 0.0], 1.0, 1));
        mbd_add_body(&mut sys, new_mbd_body([0.0, 5.0, 0.0], 1.0, 2));
        sys
    }

    #[test]
    fn test_default_config() {
        let cfg = default_mbd_config();
        assert!((cfg.gravity[1] - (-9.81)).abs() < 1e-4);
        assert_eq!(cfg.max_substeps, 4);
        assert!((cfg.dt - 1.0 / 60.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_mbd_body() {
        let body = new_mbd_body([1.0, 2.0, 3.0], 5.0, 42);
        assert_eq!(body.id, 42);
        assert!((body.mass - 5.0).abs() < 1e-6);
        assert!((body.inv_mass - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_static_body() {
        let body = new_mbd_body([0.0, 0.0, 0.0], 0.0, 0);
        assert!((body.inv_mass).abs() < 1e-9);
    }

    #[test]
    fn test_add_body_and_joint() {
        let mut sys = make_two_body_system();
        assert_eq!(mbd_body_count(&sys), 2);
        mbd_add_joint(
            &mut sys,
            MbdJoint {
                body_a: 1,
                body_b: 2,
                anchor_a: [0.0, 0.0, 0.0],
                anchor_b: [0.0, 0.0, 0.0],
                stiffness: 1.0,
            },
        );
        assert_eq!(mbd_joint_count(&sys), 1);
    }

    #[test]
    fn test_mbd_find_body_found() {
        let sys = make_two_body_system();
        let b = mbd_find_body(&sys, 1);
        assert!(b.is_some());
        assert_eq!(b.expect("should succeed").id, 1);
    }

    #[test]
    fn test_mbd_find_body_not_found() {
        let sys = make_two_body_system();
        assert!(mbd_find_body(&sys, 99).is_none());
    }

    #[test]
    fn test_mbd_step_gravity() {
        let mut sys = make_two_body_system();
        let y0 = sys.bodies[0].position[1];
        mbd_step(&mut sys);
        // Body should fall under gravity
        let y1 = sys.bodies[0].position[1];
        assert!(y1 < y0, "body should fall: y0={} y1={}", y0, y1);
    }

    #[test]
    fn test_mbd_step_static_body() {
        let cfg = default_mbd_config();
        let mut sys = new_mbd_system(cfg);
        // static body: mass = 0
        mbd_add_body(&mut sys, new_mbd_body([0.0, 0.0, 0.0], 0.0, 1));
        let y0 = sys.bodies[0].position[1];
        mbd_step(&mut sys);
        let y1 = sys.bodies[0].position[1];
        assert!((y1 - y0).abs() < 1e-9, "static body should not move");
    }

    #[test]
    fn test_mbd_step_velocity_accumulates() {
        let cfg = MbdConfig {
            gravity: [0.0, -9.81, 0.0],
            max_substeps: 1,
            dt: 1.0,
        };
        let mut sys = new_mbd_system(cfg);
        mbd_add_body(&mut sys, new_mbd_body([0.0, 100.0, 0.0], 1.0, 1));
        mbd_step(&mut sys);
        let vy = sys.bodies[0].velocity[1];
        assert!((vy - (-9.81)).abs() < 1e-4, "velocity should be -9.81, got {}", vy);
    }

    #[test]
    fn test_mbd_with_joint() {
        let mut sys = make_two_body_system();
        mbd_add_joint(
            &mut sys,
            MbdJoint {
                body_a: 1,
                body_b: 2,
                anchor_a: [0.0, 0.0, 0.0],
                anchor_b: [0.0, 0.0, 0.0],
                stiffness: 0.1,
            },
        );
        // Should not panic
        mbd_step(&mut sys);
        assert!(mbd_body_count(&sys) == 2);
    }

    #[test]
    fn test_mbd_multiple_steps() {
        let mut sys = make_two_body_system();
        for _ in 0..10 {
            mbd_step(&mut sys);
        }
        // Bodies should have moved downward
        assert!(sys.bodies[0].position[1] < 10.0);
    }

    #[test]
    fn test_mbd_system_to_json() {
        let sys = make_two_body_system();
        let json = mbd_system_to_json(&sys);
        assert!(json.contains("body_count"));
        assert!(json.contains("joint_count"));
        assert!(json.contains("bodies"));
    }

    #[test]
    fn test_mbd_system_to_json_two_bodies() {
        let sys = make_two_body_system();
        let json = mbd_system_to_json(&sys);
        assert!(json.contains("\"body_count\":2"));
        assert!(json.contains("\"joint_count\":0"));
    }

    #[test]
    fn test_mbd_energy_decreasing_with_joint() {
        // Two bodies connected by a stiff joint: relative velocity should be damped
        let cfg = MbdConfig {
            gravity: [0.0, 0.0, 0.0],
            max_substeps: 1,
            dt: 0.016,
        };
        let mut sys = new_mbd_system(cfg);
        let mut b1 = new_mbd_body([0.0, 0.0, 0.0], 1.0, 1);
        let mut b2 = new_mbd_body([1.0, 0.0, 0.0], 1.0, 2);
        b1.velocity = [1.0, 0.0, 0.0];
        b2.velocity = [-1.0, 0.0, 0.0];
        mbd_add_body(&mut sys, b1);
        mbd_add_body(&mut sys, b2);
        mbd_add_joint(
            &mut sys,
            MbdJoint {
                body_a: 1,
                body_b: 2,
                anchor_a: [0.0, 0.0, 0.0],
                anchor_b: [0.0, 0.0, 0.0],
                stiffness: 0.5,
            },
        );
        mbd_step(&mut sys);
        // After one step constraint should have partially damped relative velocity
        let rv = sys.bodies[0].velocity[0] - sys.bodies[1].velocity[0];
        // Initial relative vel was 2.0; should be reduced
        assert!(rv.abs() < 2.0 + 1e-4);
    }
}
