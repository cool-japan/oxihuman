// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sequential impulse contact resolver for pairs of dynamic bodies.

/// A simple rigid body for contact resolution.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ResolvableBody {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub inv_mass: f32,
    pub restitution: f32,
}

#[allow(dead_code)]
impl ResolvableBody {
    pub fn new(mass: f32, restitution: f32) -> Self {
        Self {
            position: [0.0; 3],
            velocity: [0.0; 3],
            inv_mass: if mass > 1e-9 { 1.0 / mass } else { 0.0 },
            restitution,
        }
    }

    pub fn is_static(&self) -> bool {
        self.inv_mass < 1e-12
    }
}

/// A contact between two bodies.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ResolveContact {
    pub normal: [f32; 3], // from body_b toward body_a
    pub penetration: f32, // positive = overlap depth
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Resolve a single contact impulse between two bodies.
#[allow(dead_code)]
pub fn resolve_contact(a: &mut ResolvableBody, b: &mut ResolvableBody, contact: &ResolveContact) {
    let rel_vel = [
        a.velocity[0] - b.velocity[0],
        a.velocity[1] - b.velocity[1],
        a.velocity[2] - b.velocity[2],
    ];
    let vel_along_normal = dot3(rel_vel, contact.normal);

    // Only resolve if bodies are approaching
    if vel_along_normal >= 0.0 {
        return;
    }

    let restitution = a.restitution.min(b.restitution);
    let j_num = -(1.0 + restitution) * vel_along_normal;
    let inv_mass_sum = a.inv_mass + b.inv_mass;
    if inv_mass_sum < 1e-12 {
        return;
    }
    let j = j_num / inv_mass_sum;

    let impulse = [
        contact.normal[0] * j,
        contact.normal[1] * j,
        contact.normal[2] * j,
    ];

    a.velocity[0] += impulse[0] * a.inv_mass;
    a.velocity[1] += impulse[1] * a.inv_mass;
    a.velocity[2] += impulse[2] * a.inv_mass;

    b.velocity[0] -= impulse[0] * b.inv_mass;
    b.velocity[1] -= impulse[1] * b.inv_mass;
    b.velocity[2] -= impulse[2] * b.inv_mass;
}

/// Positional correction to prevent sinking.
#[allow(dead_code)]
pub fn positional_correction(
    a: &mut ResolvableBody,
    b: &mut ResolvableBody,
    contact: &ResolveContact,
    slop: f32,
    percent: f32,
) {
    let correction_mag =
        ((contact.penetration - slop).max(0.0) / (a.inv_mass + b.inv_mass)) * percent;
    let correction = [
        contact.normal[0] * correction_mag,
        contact.normal[1] * correction_mag,
        contact.normal[2] * correction_mag,
    ];
    a.position[0] += correction[0] * a.inv_mass;
    a.position[1] += correction[1] * a.inv_mass;
    a.position[2] += correction[2] * a.inv_mass;
    b.position[0] -= correction[0] * b.inv_mass;
    b.position[1] -= correction[1] * b.inv_mass;
    b.position[2] -= correction[2] * b.inv_mass;
}

/// Relative velocity along contact normal (positive = separating).
#[allow(dead_code)]
pub fn relative_normal_velocity(a: &ResolvableBody, b: &ResolvableBody, normal: [f32; 3]) -> f32 {
    let rv = [
        a.velocity[0] - b.velocity[0],
        a.velocity[1] - b.velocity[1],
        a.velocity[2] - b.velocity[2],
    ];
    dot3(rv, normal)
}

/// Combined restitution coefficient.
#[allow(dead_code)]
pub fn combined_restitution(a: f32, b: f32) -> f32 {
    a.min(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_body_has_zero_inv_mass() {
        let b = ResolvableBody::new(0.0, 0.5);
        assert!(b.is_static());
    }

    #[test]
    fn dynamic_body_not_static() {
        let b = ResolvableBody::new(1.0, 0.5);
        assert!(!b.is_static());
    }

    #[test]
    fn resolve_bouncing_contact() {
        let mut a = ResolvableBody::new(1.0, 1.0); // perfectly elastic
        a.velocity = [0.0, -5.0, 0.0];
        let mut b = ResolvableBody::new(0.0, 1.0); // static floor
        let contact = ResolveContact {
            normal: [0.0, 1.0, 0.0],
            penetration: 0.01,
        };
        resolve_contact(&mut a, &mut b, &contact);
        // Ball should bounce upward
        assert!(a.velocity[1] > 0.0);
    }

    #[test]
    fn no_resolution_when_separating() {
        let mut a = ResolvableBody::new(1.0, 0.5);
        a.velocity = [0.0, 5.0, 0.0]; // moving away
        let mut b = ResolvableBody::new(1.0, 0.5);
        let contact = ResolveContact {
            normal: [0.0, 1.0, 0.0],
            penetration: 0.0,
        };
        let vel_before = a.velocity[1];
        resolve_contact(&mut a, &mut b, &contact);
        assert!((a.velocity[1] - vel_before).abs() < 1e-6);
    }

    #[test]
    fn relative_normal_velocity_approaching() {
        let mut a = ResolvableBody::new(1.0, 0.5);
        a.velocity = [0.0, -3.0, 0.0];
        let b = ResolvableBody::new(1.0, 0.5);
        let rv = relative_normal_velocity(&a, &b, [0.0, 1.0, 0.0]);
        assert!(rv < 0.0); // approaching
    }

    #[test]
    fn positional_correction_moves_bodies() {
        let mut a = ResolvableBody::new(1.0, 0.5);
        let mut b = ResolvableBody::new(1.0, 0.5);
        let contact = ResolveContact {
            normal: [0.0, 1.0, 0.0],
            penetration: 0.1,
        };
        let pos_a_before = a.position[1];
        positional_correction(&mut a, &mut b, &contact, 0.01, 0.8);
        assert!(a.position[1] > pos_a_before);
    }

    #[test]
    fn combined_restitution_takes_min() {
        assert!((combined_restitution(0.3, 0.8) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn equal_mass_collision_exchanges_velocity() {
        // a moves +x, b is at rest; normal points from b toward a = -x direction
        // rel_vel dot normal = [2,0,0] . [-1,0,0] = -2 (approaching → resolve)
        let mut a = ResolvableBody::new(1.0, 0.0); // perfectly inelastic
        a.velocity = [2.0, 0.0, 0.0];
        let mut b = ResolvableBody::new(1.0, 0.0);
        let contact = ResolveContact {
            normal: [-1.0, 0.0, 0.0],
            penetration: 0.0,
        };
        resolve_contact(&mut a, &mut b, &contact);
        // For perfectly inelastic equal mass: both end up at 1 m/s
        assert!((a.velocity[0] - 1.0).abs() < 0.1);
        assert!((b.velocity[0] - 1.0).abs() < 0.1);
    }

    #[test]
    fn double_static_no_resolution() {
        let mut a = ResolvableBody::new(0.0, 0.5);
        let mut b = ResolvableBody::new(0.0, 0.5);
        a.velocity = [0.0, -1.0, 0.0];
        let contact = ResolveContact {
            normal: [0.0, 1.0, 0.0],
            penetration: 0.1,
        };
        resolve_contact(&mut a, &mut b, &contact);
        // No change since both are static
        assert!((a.velocity[1] - (-1.0)).abs() < 1e-6);
    }
}
