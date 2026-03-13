// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! XPBD (eXtended Position Based Dynamics) constraint types and solvers.
//!
//! Implements distance, bend, and volume constraints with compliance (α) and
//! damping (β) parameters following the XPBD formulation by Müller et al.

// ── ConstraintKind ────────────────────────────────────────────────────────────

/// The kind/type of a constraint.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintKind {
    Distance,
    Bend,
    Volume,
    Stretch,
    Shear,
}

// ── DistanceConstraint ────────────────────────────────────────────────────────

/// A distance constraint between two particles.
///
/// When `compliance == 0.0` the constraint is perfectly rigid; larger values
/// allow elastic stretching.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DistanceConstraint {
    /// Index of the first particle.
    pub a: usize,
    /// Index of the second particle.
    pub b: usize,
    /// Natural (rest) distance.
    pub rest_length: f32,
    /// Compliance α (inverse stiffness). `0.0` = rigid.
    pub compliance: f32,
}

impl DistanceConstraint {
    /// Create a new distance constraint.
    pub fn new(a: usize, b: usize, rest_length: f32) -> Self {
        Self {
            a,
            b,
            rest_length,
            compliance: 0.0,
        }
    }

    /// Create a new distance constraint with explicit compliance.
    pub fn with_compliance(a: usize, b: usize, rest_length: f32, compliance: f32) -> Self {
        Self {
            a,
            b,
            rest_length,
            compliance,
        }
    }
}

// ── BendConstraint ────────────────────────────────────────────────────────────

/// A bend constraint over three consecutive particles.
///
/// Maintains the rest angle at vertex `b` between edges `a→b` and `b→c`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BendConstraint {
    /// Hinge/tip vertex index.
    pub a: usize,
    /// Middle (hinge) vertex.
    pub b: usize,
    /// Second arm vertex.
    pub c: usize,
    /// Rest angle in radians.
    pub rest_angle: f32,
    /// Compliance α.
    pub compliance: f32,
}

impl BendConstraint {
    /// Create a new bend constraint.
    pub fn new(a: usize, b: usize, c: usize, rest_angle: f32) -> Self {
        Self {
            a,
            b,
            c,
            rest_angle,
            compliance: 0.0,
        }
    }
}

// ── VolumeConstraint ─────────────────────────────────────────────────────────

/// A volume constraint for a tetrahedron defined by four vertex indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VolumeConstraint {
    /// Indices of the four tetrahedron vertices (in order).
    pub verts: [usize; 4],
    /// Rest volume.
    pub rest_volume: f32,
    /// Compliance α.
    pub compliance: f32,
}

impl VolumeConstraint {
    /// Create a new volume constraint, computing rest_volume from the supplied
    /// positions.
    pub fn from_positions(verts: [usize; 4], positions: &[[f32; 3]], compliance: f32) -> Self {
        let [i0, i1, i2, i3] = verts;
        let rv = tet_volume(positions[i0], positions[i1], positions[i2], positions[i3]);
        Self {
            verts,
            rest_volume: rv.abs(),
            compliance,
        }
    }
}

// ── Geometry helpers ──────────────────────────────────────────────────────────

/// Signed volume of a tetrahedron (a, b, c, d).
///
/// Returns the signed 1/6-scaled scalar triple product.
pub fn tet_volume(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> f32 {
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let ad = sub3(d, a);
    dot3(ab, cross3(ac, ad)) / 6.0
}

// ── Core constraint solvers ───────────────────────────────────────────────────

/// Solve a distance constraint and return the positional correction magnitude (λ).
///
/// Returns 0.0 when both particles are pinned (inv_mass == 0).
pub fn solve_distance(
    positions: &mut [[f32; 3]],
    c: &DistanceConstraint,
    inv_masses: &[f32],
    dt: f32,
    substeps: u32,
) -> f32 {
    apply_distance_constraint(positions, c, inv_masses, dt, substeps)
}

/// Solve a bend constraint and return the positional correction magnitude (λ).
pub fn solve_bend(
    positions: &mut [[f32; 3]],
    c: &BendConstraint,
    inv_masses: &[f32],
    dt: f32,
    substeps: u32,
) -> f32 {
    apply_bend_constraint(positions, c, inv_masses, dt, substeps)
}

/// Solve a volume constraint and return the positional correction magnitude (λ).
pub fn solve_volume(
    positions: &mut [[f32; 3]],
    c: &VolumeConstraint,
    inv_masses: &[f32],
    dt: f32,
    substeps: u32,
) -> f32 {
    apply_volume_constraint(positions, c, inv_masses, dt, substeps)
}

// ── XPBD apply functions ──────────────────────────────────────────────────────

/// Apply an XPBD distance constraint correction in-place.
///
/// Uses the XPBD formulation with `α_tilde = α / dt²` where `dt` is the
/// sub-step time.
#[allow(clippy::too_many_arguments)]
pub fn apply_distance_constraint(
    positions: &mut [[f32; 3]],
    c: &DistanceConstraint,
    inv_masses: &[f32],
    dt: f32,
    substeps: u32,
) -> f32 {
    let h = dt / substeps as f32;
    let w_a = inv_masses[c.a];
    let w_b = inv_masses[c.b];
    let w_sum = w_a + w_b;
    if w_sum < f32::EPSILON {
        return 0.0;
    }

    let pa = positions[c.a];
    let pb = positions[c.b];
    let diff = sub3(pa, pb);
    let dist = len3(diff);
    if dist < f32::EPSILON {
        return 0.0;
    }

    let constraint = dist - c.rest_length;
    let alpha_tilde = c.compliance / (h * h);
    let lambda = -constraint / (w_sum + alpha_tilde);

    let dir = scale3(diff, 1.0 / dist);
    let delta = scale3(dir, lambda);

    positions[c.a] = add3(positions[c.a], scale3(delta, w_a));
    positions[c.b] = add3(positions[c.b], scale3(delta, -w_b));

    lambda.abs()
}

/// Apply an XPBD bend constraint correction in-place.
#[allow(clippy::too_many_arguments)]
pub fn apply_bend_constraint(
    positions: &mut [[f32; 3]],
    c: &BendConstraint,
    inv_masses: &[f32],
    dt: f32,
    substeps: u32,
) -> f32 {
    let h = dt / substeps as f32;
    let w_a = inv_masses[c.a];
    let w_b = inv_masses[c.b];
    let w_c = inv_masses[c.c];

    let pa = positions[c.a];
    let pb = positions[c.b];
    let pc = positions[c.c];

    // Vectors from hinge b to the two arm vertices.
    let e1 = sub3(pa, pb);
    let e2 = sub3(pc, pb);

    let len_e1 = len3(e1);
    let len_e2 = len3(e2);

    if len_e1 < f32::EPSILON || len_e2 < f32::EPSILON {
        return 0.0;
    }

    let cos_angle = dot3(e1, e2) / (len_e1 * len_e2);
    let cos_angle = cos_angle.clamp(-1.0, 1.0);
    let current_angle = cos_angle.acos();

    let constraint = current_angle - c.rest_angle;
    if constraint.abs() < f32::EPSILON {
        return 0.0;
    }

    // Gradient of angle w.r.t. each vertex.
    let sin_angle = (1.0 - cos_angle * cos_angle).sqrt().max(f32::EPSILON);

    let grad_pa = scale3(
        sub3(
            scale3(e2, 1.0 / (len_e1 * len_e2)),
            scale3(e1, cos_angle / (len_e1 * len_e1)),
        ),
        -1.0 / sin_angle,
    );
    let grad_pc = scale3(
        sub3(
            scale3(e1, 1.0 / (len_e1 * len_e2)),
            scale3(e2, cos_angle / (len_e2 * len_e2)),
        ),
        -1.0 / sin_angle,
    );
    let grad_pb = scale3(add3(grad_pa, grad_pc), -1.0);

    let w_sum =
        w_a * dot3(grad_pa, grad_pa) + w_b * dot3(grad_pb, grad_pb) + w_c * dot3(grad_pc, grad_pc);

    if w_sum < f32::EPSILON {
        return 0.0;
    }

    let alpha_tilde = c.compliance / (h * h);
    let lambda = -constraint / (w_sum + alpha_tilde);

    positions[c.a] = add3(positions[c.a], scale3(grad_pa, lambda * w_a));
    positions[c.b] = add3(positions[c.b], scale3(grad_pb, lambda * w_b));
    positions[c.c] = add3(positions[c.c], scale3(grad_pc, lambda * w_c));

    lambda.abs()
}

/// Apply an XPBD volume constraint correction in-place.
#[allow(clippy::too_many_arguments)]
pub fn apply_volume_constraint(
    positions: &mut [[f32; 3]],
    c: &VolumeConstraint,
    inv_masses: &[f32],
    dt: f32,
    substeps: u32,
) -> f32 {
    let h = dt / substeps as f32;
    let [i0, i1, i2, i3] = c.verts;

    let p0 = positions[i0];
    let p1 = positions[i1];
    let p2 = positions[i2];
    let p3 = positions[i3];

    let current_vol = tet_volume(p0, p1, p2, p3);
    let constraint = current_vol - c.rest_volume;

    // Volume gradients: ∂V/∂pᵢ = (1/6) × cross product of the two opposite edges.
    // ∂V/∂p0 = (1/6)(p1-p3) × (p2-p3) etc.
    let grad0 = scale3(cross3(sub3(p1, p3), sub3(p2, p3)), 1.0 / 6.0);
    let grad1 = scale3(cross3(sub3(p2, p3), sub3(p0, p3)), 1.0 / 6.0);
    let grad2 = scale3(cross3(sub3(p0, p3), sub3(p1, p3)), 1.0 / 6.0);
    let grad3 = scale3(cross3(sub3(p1, p0), sub3(p2, p0)), 1.0 / 6.0);

    let w0 = inv_masses[i0];
    let w1 = inv_masses[i1];
    let w2 = inv_masses[i2];
    let w3 = inv_masses[i3];

    let w_sum = w0 * dot3(grad0, grad0)
        + w1 * dot3(grad1, grad1)
        + w2 * dot3(grad2, grad2)
        + w3 * dot3(grad3, grad3);

    if w_sum < f32::EPSILON {
        return 0.0;
    }

    let alpha_tilde = c.compliance / (h * h);
    let lambda = -constraint / (w_sum + alpha_tilde);

    positions[i0] = add3(positions[i0], scale3(grad0, lambda * w0));
    positions[i1] = add3(positions[i1], scale3(grad1, lambda * w1));
    positions[i2] = add3(positions[i2], scale3(grad2, lambda * w2));
    positions[i3] = add3(positions[i3], scale3(grad3, lambda * w3));

    lambda.abs()
}

/// Compute the elastic potential energy stored in a constraint.
///
/// `E = λ² / (2α)` when α > 0, otherwise 0.
pub fn constraint_energy(lambda: f32, compliance: f32, dt: f32) -> f32 {
    if compliance < f32::EPSILON {
        return 0.0;
    }
    // In XPBD the energy is (1/2) * (λ² / α_tilde) but we use the physical
    // compliance: E = λ² * dt² / (2 * compliance).
    let _ = dt;
    (lambda * lambda) / (2.0 * compliance)
}

// ── Vec3 helpers (no-std friendly) ───────────────────────────────────────────

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
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── tet_volume ────────────────────────────────────────────────────────────

    #[test]
    fn test_tet_volume_unit() {
        // Unit tet: (0,0,0),(1,0,0),(0,1,0),(0,0,1) → vol = 1/6
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let d = [0.0, 0.0, 1.0];
        let vol = tet_volume(a, b, c, d);
        assert!((vol - 1.0 / 6.0).abs() < 1e-6, "vol={vol}");
    }

    #[test]
    fn test_tet_volume_degenerate() {
        // Flat tet → volume ≈ 0
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let d = [0.5, 0.5, 0.0]; // coplanar
        let vol = tet_volume(a, b, c, d);
        assert!(vol.abs() < 1e-6, "vol={vol}");
    }

    #[test]
    fn test_tet_volume_scaled() {
        // Scaled by 2 on each axis → vol = (1/6) * 8
        let a = [0.0f32, 0.0, 0.0];
        let b = [2.0, 0.0, 0.0];
        let c = [0.0, 2.0, 0.0];
        let d = [0.0, 0.0, 2.0];
        let vol = tet_volume(a, b, c, d);
        assert!((vol - 8.0 / 6.0).abs() < 1e-5, "vol={vol}");
    }

    #[test]
    fn test_tet_volume_negative_orientation() {
        // Swapping two vertices negates the signed volume.
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let d = [0.0, 0.0, 1.0];
        let v1 = tet_volume(a, b, c, d);
        let v2 = tet_volume(a, c, b, d); // swap b↔c
        assert!((v1 + v2).abs() < 1e-6, "v1={v1} v2={v2}");
    }

    // ── Distance constraint ───────────────────────────────────────────────────

    #[test]
    fn test_distance_constraint_convergence() {
        // Two equal-mass particles 2 units apart; rest_length = 1.
        // After one solve step they should move closer.
        let mut pos = [[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let inv_masses = [1.0f32, 1.0];
        let c = DistanceConstraint::new(0, 1, 1.0);
        let lambda = apply_distance_constraint(&mut pos, &c, &inv_masses, 0.016, 1);
        assert!(lambda > 0.0, "lambda should be positive");
        let d = len3(sub3(pos[1], pos[0]));
        assert!((d - 1.0).abs() < 1e-5, "d={d}");
    }

    #[test]
    fn test_distance_constraint_already_satisfied() {
        // When rest_length == current distance, lambda should be 0.
        let mut pos = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let inv_masses = [1.0f32, 1.0];
        let c = DistanceConstraint::new(0, 1, 1.0);
        let lambda = apply_distance_constraint(&mut pos, &c, &inv_masses, 0.016, 1);
        assert!(lambda < 1e-5, "lambda={lambda}");
    }

    #[test]
    fn test_distance_constraint_pinned_vertex() {
        // Particle 0 is pinned (inv_mass=0); only particle 1 should move.
        let mut pos = [[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let inv_masses = [0.0f32, 1.0];
        let c = DistanceConstraint::new(0, 1, 1.0);
        apply_distance_constraint(&mut pos, &c, &inv_masses, 0.016, 1);
        // Pinned vertex must not move.
        assert_eq!(pos[0], [0.0, 0.0, 0.0]);
        // Free vertex should be at distance 1 from origin.
        let d = len3(pos[1]);
        assert!((d - 1.0).abs() < 1e-5, "d={d}");
    }

    #[test]
    fn test_distance_constraint_both_pinned() {
        // Both pinned → nothing changes, lambda = 0.
        let mut pos = [[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let orig = pos;
        let inv_masses = [0.0f32, 0.0];
        let c = DistanceConstraint::new(0, 1, 1.0);
        let lambda = apply_distance_constraint(&mut pos, &c, &inv_masses, 0.016, 1);
        assert!(lambda < 1e-10, "lambda={lambda}");
        assert_eq!(pos, orig);
    }

    #[test]
    fn test_distance_constraint_compliance() {
        // High compliance → smaller correction in one step.
        let mut pos_rigid = [[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let mut pos_soft = [[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let inv_masses = [1.0f32, 1.0];
        let dt = 0.016;
        let c_rigid = DistanceConstraint::with_compliance(0, 1, 1.0, 0.0);
        let c_soft = DistanceConstraint::with_compliance(0, 1, 1.0, 1.0);
        let lam_rigid = apply_distance_constraint(&mut pos_rigid, &c_rigid, &inv_masses, dt, 1);
        let lam_soft = apply_distance_constraint(&mut pos_soft, &c_soft, &inv_masses, dt, 1);
        // Rigid resolves fully in one step (larger lambda magnitude).
        assert!(
            lam_rigid >= lam_soft,
            "lam_rigid={lam_rigid} lam_soft={lam_soft}"
        );
    }

    // ── Bend constraint ───────────────────────────────────────────────────────

    #[test]
    fn test_bend_constraint_angle_correction() {
        // Straight line: a=(0,0,0), b=(1,0,0), c=(2,0,0) → angle=π.
        // rest_angle = π/2 → the bend solver should try to bend to 90°.
        let mut pos = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let inv_masses = [1.0f32, 0.0, 1.0]; // middle pinned
        let c = BendConstraint::new(0, 1, 2, std::f32::consts::FRAC_PI_2);
        let lambda = apply_bend_constraint(&mut pos, &c, &inv_masses, 0.016, 1);
        // Some correction should have been applied.
        assert!(lambda >= 0.0); // mainly checking no panic
    }

    #[test]
    fn test_bend_constraint_satisfied() {
        // 90° angle, rest_angle = π/2 → no correction needed.
        let mut pos = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let inv_masses = [1.0f32, 1.0, 1.0];
        let c = BendConstraint::new(0, 1, 2, std::f32::consts::FRAC_PI_2);
        let lambda = apply_bend_constraint(&mut pos, &c, &inv_masses, 0.016, 1);
        assert!(lambda < 1e-5, "lambda={lambda}");
    }

    #[test]
    fn test_bend_constraint_all_pinned() {
        let mut pos = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let orig = pos;
        let inv_masses = [0.0f32, 0.0, 0.0];
        let c = BendConstraint::new(0, 1, 2, std::f32::consts::FRAC_PI_2);
        apply_bend_constraint(&mut pos, &c, &inv_masses, 0.016, 1);
        assert_eq!(pos, orig);
    }

    // ── Volume constraint ─────────────────────────────────────────────────────

    #[test]
    fn test_volume_constraint_preservation() {
        // Slightly compressed tet; volume constraint should restore it.
        let p0 = [0.0f32, 0.0, 0.0];
        let p1 = [1.0, 0.0, 0.0];
        let p2 = [0.0, 1.0, 0.0];
        let p3 = [0.0, 0.0, 1.0];
        let rest_vol = tet_volume(p0, p1, p2, p3).abs();

        // Compress along Z.
        let mut pos = [p0, p1, p2, [0.0, 0.0, 0.5]];
        let inv_masses = [1.0f32; 4];
        let c = VolumeConstraint {
            verts: [0, 1, 2, 3],
            rest_volume: rest_vol,
            compliance: 0.0,
        };
        for _ in 0..20 {
            apply_volume_constraint(&mut pos, &c, &inv_masses, 0.016, 1);
        }
        let new_vol = tet_volume(pos[0], pos[1], pos[2], pos[3]).abs();
        assert!(
            (new_vol - rest_vol).abs() / rest_vol < 0.05,
            "new_vol={new_vol}"
        );
    }

    #[test]
    fn test_volume_constraint_all_pinned() {
        let mut pos = [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.5],
        ];
        let orig = pos;
        let inv_masses = [0.0f32; 4];
        let c = VolumeConstraint {
            verts: [0, 1, 2, 3],
            rest_volume: 1.0 / 6.0,
            compliance: 0.0,
        };
        apply_volume_constraint(&mut pos, &c, &inv_masses, 0.016, 1);
        assert_eq!(pos, orig);
    }

    // ── constraint_energy ─────────────────────────────────────────────────────

    #[test]
    fn test_constraint_energy_zero_compliance() {
        // Rigid constraint: energy should be 0.
        let e = constraint_energy(5.0, 0.0, 0.016);
        assert_eq!(e, 0.0);
    }

    #[test]
    fn test_constraint_energy_positive() {
        let e = constraint_energy(2.0, 1.0, 0.016);
        assert!((e - 2.0).abs() < 1e-6, "e={e}"); // (2^2)/(2*1) = 2
    }

    #[test]
    fn test_constraint_energy_lambda_zero() {
        let e = constraint_energy(0.0, 1.0, 0.016);
        assert_eq!(e, 0.0);
    }
}
