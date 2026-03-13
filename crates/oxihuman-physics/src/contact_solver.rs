// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Iterative Gauss-Seidel contact resolution with Coulomb friction.
//!
//! Resolves penetrating contacts between particles using a position-based
//! correction (Baumgarte stabilization) and velocity-level impulses with
//! a Coulomb friction cone clamp.

// ── Contact ───────────────────────────────────────────────────────────────────

/// A contact between two particles (or a particle and a static surface).
///
/// Use `usize::MAX` for `b` to represent a static surface.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Contact {
    /// Index of the first particle.
    pub a: usize,
    /// Index of the second particle (`usize::MAX` = static/world).
    pub b: usize,
    /// Contact normal pointing from `b` to `a` (unit vector).
    pub normal: [f32; 3],
    /// Penetration depth (positive = overlapping).
    pub penetration: f32,
    /// Coefficient of restitution in `[0, 1]`.
    pub restitution: f32,
    /// Coulomb friction coefficient μ.
    pub friction: f32,
}

impl Contact {
    /// Create a contact with default restitution and friction.
    pub fn new(a: usize, b: usize, normal: [f32; 3], penetration: f32) -> Self {
        Self {
            a,
            b,
            normal,
            penetration,
            restitution: 0.0,
            friction: 0.3,
        }
    }
}

// ── ContactSolverConfig ───────────────────────────────────────────────────────

/// Configuration for the iterative contact solver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactSolverConfig {
    /// Maximum Gauss-Seidel iterations per call.
    pub max_iter: u32,
    /// Baumgarte stabilization factor (0 = off, 1 = full correction per step).
    pub baumgarte: f32,
    /// Penetration slop (depth below which no correction is applied).
    pub slop: f32,
}

impl Default for ContactSolverConfig {
    fn default() -> Self {
        Self {
            max_iter: 10,
            baumgarte: 0.2,
            slop: 0.001,
        }
    }
}

// ── resolve_contacts ──────────────────────────────────────────────────────────

/// Resolve all contacts iteratively using Gauss-Seidel.
///
/// Applies:
/// 1. Positional correction (Baumgarte) to separate overlapping particles.
/// 2. Velocity correction with Coulomb friction at the contact point.
#[allow(clippy::too_many_arguments)]
pub fn resolve_contacts(
    positions: &mut [[f32; 3]],
    velocities: &mut [[f32; 3]],
    inv_masses: &[f32],
    contacts: &[Contact],
    dt: f32,
    cfg: &ContactSolverConfig,
) {
    for _ in 0..cfg.max_iter {
        for contact in contacts {
            resolve_single(positions, velocities, inv_masses, contact, dt, cfg);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_single(
    positions: &mut [[f32; 3]],
    velocities: &mut [[f32; 3]],
    inv_masses: &[f32],
    c: &Contact,
    dt: f32,
    cfg: &ContactSolverConfig,
) {
    let depth = c.penetration - cfg.slop;
    if depth <= 0.0 {
        return;
    }

    let w_a = inv_masses[c.a];
    let w_b = if c.b == usize::MAX {
        0.0
    } else {
        inv_masses[c.b]
    };
    let w_sum = w_a + w_b;
    if w_sum < f32::EPSILON {
        return;
    }

    let n = normalise3(c.normal);

    // ── Position correction (Baumgarte) ──────────────────────────────────────
    let correction = cfg.baumgarte * depth / w_sum;
    let pos_delta = scale3(n, correction);
    positions[c.a] = add3(positions[c.a], scale3(pos_delta, w_a));
    if c.b != usize::MAX {
        positions[c.b] = add3(positions[c.b], scale3(pos_delta, -w_b));
    }

    // ── Velocity correction ───────────────────────────────────────────────────
    let va = velocities[c.a];
    let vb = if c.b == usize::MAX {
        [0.0f32; 3]
    } else {
        velocities[c.b]
    };
    let v_rel = sub3(va, vb);

    // Normal velocity component.
    let v_n = dot3(v_rel, n);

    // Only apply impulse if objects are approaching.
    if v_n < 0.0 {
        // Normal impulse with restitution.
        let j_n = -(1.0 + c.restitution) * v_n / w_sum;
        let impulse_n = scale3(n, j_n);

        // Tangential (friction) impulse.
        let v_t = sub3(v_rel, scale3(n, v_n));
        let v_t_len = len3(v_t);
        let friction_impulse = if v_t_len > f32::EPSILON {
            let t_dir = scale3(v_t, 1.0 / v_t_len);
            let j_t_max = c.friction * j_n.abs();
            let j_t = (v_t_len / w_sum).min(j_t_max);
            scale3(t_dir, -j_t)
        } else {
            [0.0; 3]
        };

        let total_impulse = add3(impulse_n, friction_impulse);
        velocities[c.a] = add3(velocities[c.a], scale3(total_impulse, w_a));
        if c.b != usize::MAX {
            velocities[c.b] = add3(velocities[c.b], scale3(total_impulse, -w_b));
        }
    }

    let _ = dt; // dt reserved for future use (e.g. warm-starting)
}

// ── detect_sphere_contacts ────────────────────────────────────────────────────

/// Brute-force O(n²) sphere-sphere contact detection.
///
/// Each sphere is represented as `(particle_index, center, radius)`.
/// Returns all overlapping pairs as `Contact` values.
pub fn detect_sphere_contacts(spheres: &[(usize, [f32; 3], f32)]) -> Vec<Contact> {
    let mut contacts = Vec::new();
    for i in 0..spheres.len() {
        for j in (i + 1)..spheres.len() {
            let (idx_a, ca, ra) = spheres[i];
            let (idx_b, cb, rb) = spheres[j];

            let diff = sub3(ca, cb);
            let dist = len3(diff);
            let sum_r = ra + rb;

            if dist < sum_r {
                let normal = if dist > f32::EPSILON {
                    scale3(diff, 1.0 / dist)
                } else {
                    [0.0, 1.0, 0.0]
                };
                let penetration = sum_r - dist;
                contacts.push(Contact::new(idx_a, idx_b, normal, penetration));
            }
        }
    }
    contacts
}

// ── contact_energy ────────────────────────────────────────────────────────────

/// Compute a scalar energy metric for all contacts.
///
/// Returns `Σ max(0, penetration)²` — useful for convergence monitoring.
pub fn contact_energy(contacts: &[Contact], _positions: &[[f32; 3]]) -> f32 {
    contacts
        .iter()
        .map(|c| {
            let d = c.penetration.max(0.0);
            d * d
        })
        .sum()
}

// ── Vec3 helpers ──────────────────────────────────────────────────────────────

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

#[inline]
fn normalise3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l < f32::EPSILON {
        a
    } else {
        scale3(a, 1.0 / l)
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cfg() -> ContactSolverConfig {
        ContactSolverConfig::default()
    }

    // ── Single contact penetration resolution ─────────────────────────────────

    #[test]
    fn test_single_contact_separates() {
        // Two particles at the same position — should be pushed apart.
        let mut pos = [[0.0f32, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let mut vel = [[0.0f32; 3]; 2];
        let inv_masses = [1.0f32, 1.0];
        let contact = Contact {
            a: 0,
            b: 1,
            normal: [0.0, 1.0, 0.0],
            penetration: 0.5,
            restitution: 0.0,
            friction: 0.0,
        };
        let cfg = default_cfg();
        resolve_contacts(&mut pos, &mut vel, &inv_masses, &[contact], 0.016, &cfg);
        // After resolution particle 0 should be above particle 1.
        assert!(
            pos[0][1] > pos[1][1],
            "pos[0][1]={} pos[1][1]={}",
            pos[0][1],
            pos[1][1]
        );
    }

    #[test]
    fn test_no_correction_below_slop() {
        let mut pos = [[0.0f32, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let orig = pos;
        let mut vel = [[0.0f32; 3]; 2];
        let inv_masses = [1.0f32, 1.0];
        let contact = Contact {
            a: 0,
            b: 1,
            normal: [0.0, 1.0, 0.0],
            penetration: 0.0005, // below default slop of 0.001
            restitution: 0.0,
            friction: 0.0,
        };
        let cfg = default_cfg();
        resolve_contacts(&mut pos, &mut vel, &inv_masses, &[contact], 0.016, &cfg);
        assert_eq!(pos, orig);
    }

    // ── Restitution bounce ────────────────────────────────────────────────────

    #[test]
    fn test_restitution_reverses_velocity() {
        // Particle 0 moving at -1 m/s toward a static surface (b = usize::MAX).
        let mut pos = [[0.0f32, 0.0, 0.0]];
        let mut vel = [[0.0f32, -1.0, 0.0]];
        let inv_masses = [1.0f32];
        let contact = Contact {
            a: 0,
            b: usize::MAX,
            normal: [0.0, 1.0, 0.0],
            penetration: 0.01,
            restitution: 1.0,
            friction: 0.0,
        };
        let cfg = ContactSolverConfig {
            max_iter: 1,
            baumgarte: 0.0,
            slop: 0.0,
        };
        resolve_contacts(&mut pos, &mut vel, &inv_masses, &[contact], 0.016, &cfg);
        // Velocity should have flipped to positive Y.
        assert!(vel[0][1] > 0.0, "vel[0][1]={}", vel[0][1]);
    }

    #[test]
    fn test_zero_restitution_stops_normal_velocity() {
        let mut pos = [[0.0f32, 0.0, 0.0]];
        let mut vel = [[0.0f32, -2.0, 0.0]];
        let inv_masses = [1.0f32];
        let contact = Contact {
            a: 0,
            b: usize::MAX,
            normal: [0.0, 1.0, 0.0],
            penetration: 0.01,
            restitution: 0.0,
            friction: 0.0,
        };
        let cfg = ContactSolverConfig {
            max_iter: 1,
            baumgarte: 0.0,
            slop: 0.0,
        };
        resolve_contacts(&mut pos, &mut vel, &inv_masses, &[contact], 0.016, &cfg);
        // Normal velocity should be zero (absorbed).
        assert!(vel[0][1].abs() < 1e-5, "vel[0][1]={}", vel[0][1]);
    }

    // ── Friction cone clamp ───────────────────────────────────────────────────

    #[test]
    fn test_friction_reduces_tangential_velocity() {
        let mut pos = [[0.0f32, 0.0, 0.0]];
        // Moving fast tangentially and slightly into the surface.
        let mut vel = [[5.0f32, -0.1, 0.0]];
        let inv_masses = [1.0f32];
        let contact = Contact {
            a: 0,
            b: usize::MAX,
            normal: [0.0, 1.0, 0.0],
            penetration: 0.01,
            restitution: 0.0,
            friction: 0.5,
        };
        let cfg = ContactSolverConfig {
            max_iter: 1,
            baumgarte: 0.0,
            slop: 0.0,
        };
        let vx_before = vel[0][0].abs();
        resolve_contacts(&mut pos, &mut vel, &inv_masses, &[contact], 0.016, &cfg);
        let vx_after = vel[0][0].abs();
        assert!(
            vx_after < vx_before,
            "vx_after={vx_after} >= vx_before={vx_before}"
        );
    }

    #[test]
    fn test_friction_cone_clamp_zero_friction() {
        let mut pos = [[0.0f32, 0.0, 0.0]];
        let mut vel = [[5.0f32, -0.1, 0.0]];
        let inv_masses = [1.0f32];
        let contact = Contact {
            a: 0,
            b: usize::MAX,
            normal: [0.0, 1.0, 0.0],
            penetration: 0.01,
            restitution: 0.0,
            friction: 0.0,
        };
        let cfg = ContactSolverConfig {
            max_iter: 1,
            baumgarte: 0.0,
            slop: 0.0,
        };
        let vx_before = vel[0][0];
        resolve_contacts(&mut pos, &mut vel, &inv_masses, &[contact], 0.016, &cfg);
        // Zero friction should not change tangential velocity.
        assert!(
            (vel[0][0] - vx_before).abs() < 1e-5,
            "vx changed unexpectedly"
        );
    }

    // ── Zero-mass particle pinning ────────────────────────────────────────────

    #[test]
    fn test_pinned_particle_does_not_move() {
        let mut pos = [[0.0f32, 0.0, 0.0], [0.5, 0.0, 0.0]];
        let mut vel = [[0.0f32; 3]; 2];
        let inv_masses = [0.0f32, 1.0]; // a is pinned
        let contact = Contact {
            a: 0,
            b: 1,
            normal: [1.0, 0.0, 0.0],
            penetration: 0.2,
            restitution: 0.0,
            friction: 0.0,
        };
        let cfg = default_cfg();
        resolve_contacts(&mut pos, &mut vel, &inv_masses, &[contact], 0.016, &cfg);
        assert_eq!(pos[0], [0.0, 0.0, 0.0], "pinned particle moved");
    }

    // ── detect_sphere_contacts ────────────────────────────────────────────────

    #[test]
    fn test_detect_sphere_contacts_overlap() {
        // Two spheres at (0,0,0) and (1,0,0) with radii 0.8 each → overlap.
        let spheres = [
            (0usize, [0.0f32, 0.0, 0.0], 0.8f32),
            (1, [1.0, 0.0, 0.0], 0.8),
        ];
        let contacts = detect_sphere_contacts(&spheres);
        assert_eq!(contacts.len(), 1);
        assert!((contacts[0].penetration - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_detect_sphere_contacts_no_overlap() {
        let spheres = [
            (0usize, [0.0f32, 0.0, 0.0], 0.4f32),
            (1, [2.0, 0.0, 0.0], 0.4),
        ];
        let contacts = detect_sphere_contacts(&spheres);
        assert_eq!(contacts.len(), 0);
    }

    #[test]
    fn test_detect_sphere_contacts_three() {
        // Three mutually overlapping spheres → 3 pairs.
        let spheres = [
            (0usize, [0.0f32, 0.0, 0.0], 1.0f32),
            (1, [1.0, 0.0, 0.0], 1.0),
            (2, [0.5, 0.866, 0.0], 1.0),
        ];
        let contacts = detect_sphere_contacts(&spheres);
        assert_eq!(
            contacts.len(),
            3,
            "expected 3 contacts, got {}",
            contacts.len()
        );
    }

    // ── contact_energy ────────────────────────────────────────────────────────

    #[test]
    fn test_contact_energy_positive_penetration() {
        let contacts = vec![
            Contact::new(0, 1, [0.0, 1.0, 0.0], 2.0),
            Contact::new(1, 2, [1.0, 0.0, 0.0], 3.0),
        ];
        let pos = [[0.0f32; 3]; 3];
        let e = contact_energy(&contacts, &pos);
        assert!((e - (4.0 + 9.0)).abs() < 1e-5, "e={e}");
    }

    #[test]
    fn test_contact_energy_no_penetration() {
        let mut c = Contact::new(0, 1, [0.0, 1.0, 0.0], 0.0);
        c.penetration = -0.5; // already separated
        let pos = [[0.0f32; 3]; 2];
        let e = contact_energy(&[c], &pos);
        assert_eq!(e, 0.0);
    }

    #[test]
    fn test_contact_energy_empty() {
        let pos: [[f32; 3]; 0] = [];
        let e = contact_energy(&[], &pos);
        assert_eq!(e, 0.0);
    }
}
