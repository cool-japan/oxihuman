// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Impulse-based collision response for rigid body contacts.

// ── Structs ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionResponseConfig {
    pub restitution: f32,
    pub friction: f32,
    pub slop: f32,
    pub max_impulse: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionBody {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub mass: f32,
    pub inv_mass: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionContact {
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub penetration: f32,
    pub body_a: u32,
    pub body_b: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionImpulse {
    pub linear_a: [f32; 3],
    pub linear_b: [f32; 3],
    pub angular_a: [f32; 3],
    pub angular_b: [f32; 3],
    pub magnitude: f32,
}

// ── Helper math ───────────────────────────────────────────────────────────────

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_collision_response_config() -> CollisionResponseConfig {
    CollisionResponseConfig {
        restitution: 0.3,
        friction: 0.5,
        slop: 0.001,
        max_impulse: 1000.0,
    }
}

#[allow(dead_code)]
pub fn new_collision_body(pos: [f32; 3], vel: [f32; 3], mass: f32) -> CollisionBody {
    let inv_mass = if mass > 1e-10 { 1.0 / mass } else { 0.0 };
    CollisionBody {
        position: pos,
        velocity: vel,
        angular_velocity: [0.0; 3],
        mass,
        inv_mass,
    }
}

#[allow(dead_code)]
pub fn new_collision_contact(
    point: [f32; 3],
    normal: [f32; 3],
    depth: f32,
    a: u32,
    b: u32,
) -> CollisionContact {
    CollisionContact {
        point,
        normal,
        penetration: depth,
        body_a: a,
        body_b: b,
    }
}

#[allow(dead_code)]
pub fn relative_velocity_at_contact_cr(
    a: &CollisionBody,
    b: &CollisionBody,
    _contact: &CollisionContact,
) -> [f32; 3] {
    sub3(a.velocity, b.velocity)
}

#[allow(dead_code)]
pub fn compute_collision_impulse(
    a: &CollisionBody,
    b: &CollisionBody,
    contact: &CollisionContact,
    cfg: &CollisionResponseConfig,
) -> CollisionImpulse {
    let n = contact.normal;
    let rel_vel = relative_velocity_at_contact_cr(a, b, contact);
    let vn = dot3(rel_vel, n);

    // Impulse along normal
    let denom = a.inv_mass + b.inv_mass;
    // vn = dot(v_a - v_b, n); positive when bodies approach (n points A→B).
    // Impulse magnitude j must be non-negative; only apply when approaching.
    let j = if denom < 1e-10 || vn <= 0.0 {
        0.0
    } else {
        ((1.0 + cfg.restitution) * vn / denom)
            .clamp(0.0, cfg.max_impulse)
    };

    // Apply impulse: push A in -n direction, B in +n direction.
    let impulse_b = scale3(n, j);
    let impulse_a = scale3(n, -j);

    CollisionImpulse {
        linear_a: impulse_a,
        linear_b: impulse_b,
        angular_a: [0.0; 3],
        angular_b: [0.0; 3],
        magnitude: j,
    }
}

#[allow(dead_code)]
pub fn apply_impulse_cr(body: &mut CollisionBody, impulse: [f32; 3]) {
    if body.inv_mass < 1e-10 {
        return;
    }
    body.velocity = add3(body.velocity, scale3(impulse, body.inv_mass));
}

#[allow(dead_code)]
pub fn resolve_collision(
    a: &mut CollisionBody,
    b: &mut CollisionBody,
    contact: &CollisionContact,
    cfg: &CollisionResponseConfig,
) {
    let imp = compute_collision_impulse(a, b, contact, cfg);
    apply_impulse_cr(a, imp.linear_a);
    apply_impulse_cr(b, imp.linear_b);

    // Positional correction (slop)
    let correction = ((contact.penetration - cfg.slop).max(0.0)
        / (a.inv_mass + b.inv_mass).max(1e-10))
        * 0.2;
    let corr = scale3(contact.normal, correction);
    a.position = add3(a.position, scale3(corr, a.inv_mass));
    b.position = sub3(b.position, scale3(corr, b.inv_mass));
}

#[allow(dead_code)]
pub fn relative_velocity_at_contact(
    a: &CollisionBody,
    b: &CollisionBody,
    contact: &CollisionContact,
) -> [f32; 3] {
    relative_velocity_at_contact_cr(a, b, contact)
}

#[allow(dead_code)]
pub fn collision_impulse_to_json(imp: &CollisionImpulse) -> String {
    format!(
        "{{\"magnitude\":{},\"linear_a\":[{},{},{}],\"linear_b\":[{},{},{}]}}",
        imp.magnitude,
        imp.linear_a[0],
        imp.linear_a[1],
        imp.linear_a[2],
        imp.linear_b[0],
        imp.linear_b[1],
        imp.linear_b[2],
    )
}

#[allow(dead_code)]
pub fn contact_to_json(c: &CollisionContact) -> String {
    format!(
        "{{\"penetration\":{},\"normal\":[{},{},{}],\"point\":[{},{},{}],\
         \"body_a\":{},\"body_b\":{}}}",
        c.penetration,
        c.normal[0],
        c.normal[1],
        c.normal[2],
        c.point[0],
        c.point[1],
        c.point[2],
        c.body_a,
        c.body_b,
    )
}

#[allow(dead_code)]
pub fn is_separating(
    a: &CollisionBody,
    b: &CollisionBody,
    contact: &CollisionContact,
) -> bool {
    // The contact normal points from A toward B.
    // rel_vel = v_a - v_b; positive dot product means A moves away from B → separating.
    // Approaching bodies: rel_vel along normal is negative (closing speed).
    let rel_vel = relative_velocity_at_contact_cr(a, b, contact);
    dot3(rel_vel, contact.normal) < 0.0
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cfg() -> CollisionResponseConfig {
        default_collision_response_config()
    }

    fn approaching_bodies() -> (CollisionBody, CollisionBody, CollisionContact) {
        let a = new_collision_body([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0);
        let b = new_collision_body([1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], 1.0);
        let contact = new_collision_contact(
            [0.5, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            0.01,
            0,
            1,
        );
        (a, b, contact)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_cfg();
        assert!(cfg.restitution >= 0.0 && cfg.restitution <= 1.0);
        assert!(cfg.friction >= 0.0);
    }

    #[test]
    fn test_new_collision_body_inv_mass() {
        let body = new_collision_body([0.0; 3], [0.0; 3], 2.0);
        assert!((body.inv_mass - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_compute_impulse_positive_magnitude() {
        let (a, b, contact) = approaching_bodies();
        let cfg = default_cfg();
        let imp = compute_collision_impulse(&a, &b, &contact, &cfg);
        assert!(imp.magnitude > 0.0, "approaching bodies must produce positive impulse");
    }

    #[test]
    fn test_resolve_collision_separates_bodies() {
        let (mut a, mut b, contact) = approaching_bodies();
        let cfg = default_cfg();
        let vax_before = a.velocity[0];
        let vbx_before = b.velocity[0];
        resolve_collision(&mut a, &mut b, &contact, &cfg);
        // After resolution body A should slow in X, body B should slow in -X
        assert!(a.velocity[0] < vax_before);
        assert!(b.velocity[0] > vbx_before);
    }

    #[test]
    fn test_is_separating_approaching() {
        let (a, b, contact) = approaching_bodies();
        assert!(!is_separating(&a, &b, &contact));
    }

    #[test]
    fn test_is_separating_diverging() {
        let a = new_collision_body([0.0; 3], [-1.0, 0.0, 0.0], 1.0);
        let b = new_collision_body([1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0);
        let contact = new_collision_contact([0.5, 0.0, 0.0], [1.0, 0.0, 0.0], 0.01, 0, 1);
        assert!(is_separating(&a, &b, &contact));
    }

    #[test]
    fn test_collision_impulse_to_json() {
        let (a, b, contact) = approaching_bodies();
        let cfg = default_cfg();
        let imp = compute_collision_impulse(&a, &b, &contact, &cfg);
        let json = collision_impulse_to_json(&imp);
        assert!(json.contains("\"magnitude\":"));
    }

    #[test]
    fn test_contact_to_json() {
        let contact = new_collision_contact([0.5, 0.0, 0.0], [1.0, 0.0, 0.0], 0.01, 0, 1);
        let json = contact_to_json(&contact);
        assert!(json.contains("\"penetration\":0.01"));
        assert!(json.contains("\"body_a\":0"));
    }

    #[test]
    fn test_zero_mass_body_no_change() {
        let mut static_body = new_collision_body([0.0; 3], [0.0; 3], 0.0);
        let original_vel = static_body.velocity;
        apply_impulse_cr(&mut static_body, [100.0, 0.0, 0.0]);
        assert_eq!(static_body.velocity, original_vel, "static body must not move");
    }
}
