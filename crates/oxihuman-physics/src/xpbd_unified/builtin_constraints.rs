// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Built-in XPBD constraints: distance, volume, shape-matching, collision.

use super::constraint::XpbdConstraint;

// ─── Helpers ────────────────────────────────────────────────────────────────

#[inline]
fn vec3_sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn vec3_scale(v: [f64; 3], s: f64) -> [f64; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn vec3_dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_length(v: [f64; 3]) -> f64 {
    vec3_dot(v, v).sqrt()
}

#[inline]
fn vec3_cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

// ─── DistanceConstraint ─────────────────────────────────────────────────────

/// Maintains a fixed distance between two particles.
///
/// `C(x) = |x_j - x_i| - rest_length`
pub struct DistanceConstraint {
    /// Indices of the two constrained particles.
    pub indices: [usize; 2],
    /// Rest (target) length.
    pub rest_length: f64,
    /// Compliance α (0 = rigid).
    pub compliance_val: f64,
}

impl DistanceConstraint {
    pub fn new(i: usize, j: usize, rest_length: f64, compliance: f64) -> Self {
        Self {
            indices: [i, j],
            rest_length,
            compliance_val: compliance,
        }
    }
}

impl XpbdConstraint for DistanceConstraint {
    fn project(
        &self,
        positions: &mut [[f64; 3]],
        inv_masses: &[f64],
        dt: f64,
    ) -> f64 {
        let i = self.indices[0];
        let j = self.indices[1];
        let w_i = inv_masses[i];
        let w_j = inv_masses[j];
        let w_sum = w_i + w_j;
        if w_sum < f64::EPSILON {
            return 0.0; // both fixed
        }

        let diff = vec3_sub(positions[j], positions[i]);
        let len = vec3_length(diff);
        if len < f64::EPSILON {
            return 0.0; // degenerate
        }

        let c = len - self.rest_length; // constraint value
        let alpha_tilde = self.compliance_val / (dt * dt);
        let delta_lambda = -c / (w_sum + alpha_tilde);

        let correction = vec3_scale(diff, delta_lambda / len);
        positions[i] = vec3_sub(positions[i], vec3_scale(correction, w_i));
        positions[j] = vec3_add(positions[j], vec3_scale(correction, w_j));

        c.abs()
    }

    fn compliance(&self) -> f64 {
        self.compliance_val
    }

    fn particle_indices(&self) -> &[usize] {
        &self.indices
    }

    fn label(&self) -> &str {
        "distance"
    }
}

// ─── VolumeConstraint ───────────────────────────────────────────────────────

/// Preserves the volume of a tetrahedron formed by four particles.
///
/// `C(x) = V(x) - rest_volume` where `V` is 1/6 of the signed volume
/// of the tet.
pub struct VolumeConstraint {
    /// Indices of four particles forming a tetrahedron.
    pub indices: [usize; 4],
    /// Rest volume (signed).
    pub rest_volume: f64,
    /// Compliance α.
    pub compliance_val: f64,
}

impl VolumeConstraint {
    pub fn new(indices: [usize; 4], rest_volume: f64, compliance: f64) -> Self {
        Self {
            indices,
            rest_volume,
            compliance_val: compliance,
        }
    }

    /// Compute signed tet volume = 1/6 * dot((p1-p0), cross(p2-p0, p3-p0)).
    fn signed_volume(p: [[f64; 3]; 4]) -> f64 {
        let e1 = vec3_sub(p[1], p[0]);
        let e2 = vec3_sub(p[2], p[0]);
        let e3 = vec3_sub(p[3], p[0]);
        vec3_dot(e1, vec3_cross(e2, e3)) / 6.0
    }
}

impl XpbdConstraint for VolumeConstraint {
    fn project(
        &self,
        positions: &mut [[f64; 3]],
        inv_masses: &[f64],
        dt: f64,
    ) -> f64 {
        let idx = self.indices;
        let p = [
            positions[idx[0]],
            positions[idx[1]],
            positions[idx[2]],
            positions[idx[3]],
        ];

        let vol = Self::signed_volume(p);
        let c = vol - self.rest_volume;

        // Gradients: ∂C/∂p_i
        // grad_0 = cross(p2-p1, p3-p1) / 6
        // grad_1 = cross(p2-p0, p3-p0) / 6  (with appropriate signs)
        let e10 = vec3_sub(p[1], p[0]);
        let e20 = vec3_sub(p[2], p[0]);
        let e30 = vec3_sub(p[3], p[0]);

        let grad0 = vec3_scale(vec3_cross(vec3_sub(p[2], p[1]), vec3_sub(p[3], p[1])), 1.0 / 6.0);
        let grad1 = vec3_scale(vec3_cross(e30, e20), 1.0 / 6.0);
        let grad2 = vec3_scale(vec3_cross(e10, e30), 1.0 / 6.0);
        let grad3 = vec3_scale(vec3_cross(e20, e10), 1.0 / 6.0);

        let grads = [grad0, grad1, grad2, grad3];

        let mut denom = 0.0;
        for k in 0..4 {
            denom += inv_masses[idx[k]] * vec3_dot(grads[k], grads[k]);
        }
        let alpha_tilde = self.compliance_val / (dt * dt);
        denom += alpha_tilde;
        if denom.abs() < f64::EPSILON {
            return c.abs();
        }

        let delta_lambda = -c / denom;

        for k in 0..4 {
            let corr = vec3_scale(grads[k], delta_lambda * inv_masses[idx[k]]);
            positions[idx[k]] = vec3_add(positions[idx[k]], corr);
        }

        c.abs()
    }

    fn compliance(&self) -> f64 {
        self.compliance_val
    }

    fn particle_indices(&self) -> &[usize] {
        &self.indices
    }

    fn label(&self) -> &str {
        "volume"
    }
}

// ─── ShapeMatchingConstraint ────────────────────────────────────────────────

/// Shape-matching constraint: drives a set of particles towards a rigid
/// transformation of their rest configuration.
///
/// This implements the Müller et al. shape-matching approach within XPBD.
/// The rest positions define the "goal" shape; each iteration the optimal
/// rotation is extracted via polar decomposition and particles are corrected
/// towards their matched positions.
pub struct ShapeMatchingConstraint {
    /// Particle indices.
    pub indices: Vec<usize>,
    /// Rest positions relative to center of mass.
    pub rest_relative: Vec<[f64; 3]>,
    /// Rest center of mass.
    pub rest_com: [f64; 3],
    /// Compliance α.
    pub compliance_val: f64,
    /// Stiffness blend (0..1) — 1 means full shape matching.
    pub stiffness: f64,
}

impl ShapeMatchingConstraint {
    /// Create from rest positions (absolute).  Computes rest COM and relative
    /// offsets automatically.
    pub fn new(
        indices: Vec<usize>,
        rest_positions: &[[f64; 3]],
        masses: &[f64],
        compliance: f64,
        stiffness: f64,
    ) -> Self {
        let n = indices.len();
        let total_mass: f64 = indices.iter().map(|&i| masses[i]).sum();
        let inv_total = if total_mass.abs() > f64::EPSILON {
            1.0 / total_mass
        } else {
            0.0
        };

        let mut com = [0.0; 3];
        for &idx in &indices {
            let m = masses[idx];
            for d in 0..3 {
                com[d] += rest_positions[idx][d] * m;
            }
        }
        for d in 0..3 {
            com[d] *= inv_total;
        }

        let mut rest_relative = Vec::with_capacity(n);
        for &idx in &indices {
            rest_relative.push(vec3_sub(rest_positions[idx], com));
        }

        Self {
            indices,
            rest_relative,
            rest_com: com,
            compliance_val: compliance,
            stiffness: stiffness.clamp(0.0, 1.0),
        }
    }

    /// Compute current center of mass weighted by inverse of inverse-mass
    /// (i.e. by actual mass).
    fn current_com(positions: &[[f64; 3]], indices: &[usize], inv_masses: &[f64]) -> [f64; 3] {
        let mut com = [0.0; 3];
        let mut total = 0.0;
        for &idx in indices {
            let m = if inv_masses[idx] > f64::EPSILON {
                1.0 / inv_masses[idx]
            } else {
                1e12 // very large mass for fixed particles
            };
            for d in 0..3 {
                com[d] += positions[idx][d] * m;
            }
            total += m;
        }
        if total.abs() > f64::EPSILON {
            let inv = 1.0 / total;
            for d in 0..3 {
                com[d] *= inv;
            }
        }
        com
    }

    /// Extract rotation via iterative polar decomposition of the
    /// deformation matrix A = Σ m_i (q_i ⊗ r_i).
    ///
    /// Returns the 3×3 rotation matrix (row-major).
    fn extract_rotation(
        positions: &[[f64; 3]],
        indices: &[usize],
        inv_masses: &[f64],
        current_com: [f64; 3],
        rest_relative: &[[f64; 3]],
    ) -> [[f64; 3]; 3] {
        // Build A = Σ m_i * q_i * r_i^T
        let mut a = [[0.0f64; 3]; 3];
        for (k, &idx) in indices.iter().enumerate() {
            let m = if inv_masses[idx] > f64::EPSILON {
                1.0 / inv_masses[idx]
            } else {
                1e12
            };
            let q = vec3_sub(positions[idx], current_com);
            let r = rest_relative[k];
            for row in 0..3 {
                for col in 0..3 {
                    a[row][col] += m * q[row] * r[col];
                }
            }
        }

        // Iterative polar decomposition: R ← A * (A^T A)^{-1/2}
        // We do a simple iterative approach: R_{n+1} = 0.5*(R_n + (R_n^{-T}))
        // Start with R = A
        let mut r_mat = a;

        for _ in 0..10 {
            // Compute R^{-T} via adjugate / determinant
            let det = mat3_det(r_mat);
            if det.abs() < 1e-30 {
                return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
            }
            let inv_t = mat3_inv_transpose(r_mat, det);
            // R = 0.5 * (R + R^{-T})
            for row in 0..3 {
                for col in 0..3 {
                    r_mat[row][col] = 0.5 * (r_mat[row][col] + inv_t[row][col]);
                }
            }
        }

        r_mat
    }
}

/// Determinant of a 3×3 matrix (row-major).
fn mat3_det(m: [[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// Inverse-transpose of a 3×3 matrix given precomputed determinant.
fn mat3_inv_transpose(m: [[f64; 3]; 3], det: f64) -> [[f64; 3]; 3] {
    let inv_det = 1.0 / det;
    // cofactor matrix (transposed gives adjugate; adjugate/det = inverse)
    // We want inverse-transpose = cofactor / det
    [
        [
            (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv_det,
            (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv_det,
            (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv_det,
        ],
        [
            (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv_det,
            (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv_det,
            (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv_det,
        ],
        [
            (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv_det,
            (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv_det,
            (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv_det,
        ],
    ]
}

/// Multiply 3×3 matrix (row-major) by 3-vector.
fn mat3_mul_vec(m: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

impl XpbdConstraint for ShapeMatchingConstraint {
    fn project(
        &self,
        positions: &mut [[f64; 3]],
        inv_masses: &[f64],
        _dt: f64,
    ) -> f64 {
        if self.indices.is_empty() {
            return 0.0;
        }

        let com = Self::current_com(positions, &self.indices, inv_masses);
        let rot = Self::extract_rotation(
            positions,
            &self.indices,
            inv_masses,
            com,
            &self.rest_relative,
        );

        let mut total_violation = 0.0;

        for (k, &idx) in self.indices.iter().enumerate() {
            if inv_masses[idx] < f64::EPSILON {
                continue; // fixed particle
            }
            // Goal position: com + R * rest_relative[k]
            let goal = vec3_add(com, mat3_mul_vec(rot, self.rest_relative[k]));
            let diff = vec3_sub(goal, positions[idx]);
            let violation = vec3_length(diff);
            total_violation += violation;

            // Blend towards goal
            for d in 0..3 {
                positions[idx][d] += self.stiffness * diff[d];
            }
        }

        total_violation
    }

    fn compliance(&self) -> f64 {
        self.compliance_val
    }

    fn particle_indices(&self) -> &[usize] {
        &self.indices
    }

    fn label(&self) -> &str {
        "shape_matching"
    }
}

// ─── CollisionConstraint ────────────────────────────────────────────────────

/// Half-plane collision constraint: keeps a particle above a plane.
///
/// `C(x) = dot(x_i - point, normal) ≥ 0`
///
/// When violated, projects the particle to the surface.
pub struct CollisionConstraint {
    /// Index of the particle.
    pub index: usize,
    /// A point on the collision plane.
    pub plane_point: [f64; 3],
    /// Outward normal of the plane (unit length).
    pub plane_normal: [f64; 3],
    /// Compliance α.
    pub compliance_val: f64,
    /// Coefficient of friction (Coulomb model, 0 = frictionless).
    pub friction: f64,
}

impl CollisionConstraint {
    pub fn new(
        index: usize,
        plane_point: [f64; 3],
        plane_normal: [f64; 3],
        compliance: f64,
        friction: f64,
    ) -> Self {
        let len = vec3_length(plane_normal);
        let normal = if len > f64::EPSILON {
            vec3_scale(plane_normal, 1.0 / len)
        } else {
            [0.0, 1.0, 0.0]
        };
        Self {
            index,
            plane_point,
            plane_normal: normal,
            compliance_val: compliance,
            friction: friction.max(0.0),
        }
    }
}

impl XpbdConstraint for CollisionConstraint {
    fn project(
        &self,
        positions: &mut [[f64; 3]],
        inv_masses: &[f64],
        dt: f64,
    ) -> f64 {
        let idx = self.index;
        let w = inv_masses[idx];
        if w < f64::EPSILON {
            return 0.0; // fixed
        }

        let rel = vec3_sub(positions[idx], self.plane_point);
        let depth = vec3_dot(rel, self.plane_normal);

        if depth >= 0.0 {
            return 0.0; // no penetration
        }

        let c = -depth; // positive violation magnitude
        let alpha_tilde = self.compliance_val / (dt * dt);
        let delta_lambda = c / (w + alpha_tilde);

        // Normal correction
        let normal_corr = vec3_scale(self.plane_normal, delta_lambda * w);
        positions[idx] = vec3_add(positions[idx], normal_corr);

        // Friction: project tangential displacement
        if self.friction > f64::EPSILON {
            let tangent = vec3_sub(rel, vec3_scale(self.plane_normal, depth));
            let tang_len = vec3_length(tangent);
            if tang_len > f64::EPSILON {
                let max_tang = self.friction * delta_lambda * w;
                let tang_corr = if tang_len <= max_tang {
                    tangent
                } else {
                    vec3_scale(tangent, max_tang / tang_len)
                };
                positions[idx] = vec3_sub(positions[idx], tang_corr);
            }
        }

        c
    }

    fn compliance(&self) -> f64 {
        self.compliance_val
    }

    fn particle_indices(&self) -> &[usize] {
        std::slice::from_ref(&self.index)
    }

    fn label(&self) -> &str {
        "collision"
    }
}

// ─── Particle-Particle Collision Constraint ─────────────────────────────────

/// Collision constraint between two particles modeled as spheres.
pub struct ParticleCollisionConstraint {
    /// Indices of the two particles.
    pub indices: [usize; 2],
    /// Combined radius (sum of radii).
    pub combined_radius: f64,
    /// Compliance α.
    pub compliance_val: f64,
}

impl ParticleCollisionConstraint {
    pub fn new(i: usize, j: usize, combined_radius: f64, compliance: f64) -> Self {
        Self {
            indices: [i, j],
            combined_radius,
            compliance_val: compliance,
        }
    }
}

impl XpbdConstraint for ParticleCollisionConstraint {
    fn project(
        &self,
        positions: &mut [[f64; 3]],
        inv_masses: &[f64],
        dt: f64,
    ) -> f64 {
        let i = self.indices[0];
        let j = self.indices[1];
        let w_sum = inv_masses[i] + inv_masses[j];
        if w_sum < f64::EPSILON {
            return 0.0;
        }

        let diff = vec3_sub(positions[j], positions[i]);
        let dist = vec3_length(diff);
        if dist >= self.combined_radius || dist < f64::EPSILON {
            return 0.0; // no overlap or degenerate
        }

        let penetration = self.combined_radius - dist;
        let alpha_tilde = self.compliance_val / (dt * dt);
        let delta_lambda = penetration / (w_sum + alpha_tilde);

        let n = vec3_scale(diff, 1.0 / dist);
        positions[i] = vec3_sub(positions[i], vec3_scale(n, delta_lambda * inv_masses[i]));
        positions[j] = vec3_add(positions[j], vec3_scale(n, delta_lambda * inv_masses[j]));

        penetration
    }

    fn compliance(&self) -> f64 {
        self.compliance_val
    }

    fn particle_indices(&self) -> &[usize] {
        &self.indices
    }

    fn label(&self) -> &str {
        "particle_collision"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_constraint_projects_towards_rest() {
        let c = DistanceConstraint::new(0, 1, 1.0, 0.0);
        let mut positions = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let inv_masses = [1.0, 1.0];
        let violation = c.project(&mut positions, &inv_masses, 0.01);
        assert!(violation > 0.5); // was stretched to 2, rest is 1
        // After projection particles should be closer to rest length
        let new_dist = vec3_length(vec3_sub(positions[1], positions[0]));
        assert!(new_dist < 2.0);
    }

    #[test]
    fn test_distance_constraint_fixed_particle() {
        let c = DistanceConstraint::new(0, 1, 1.0, 0.0);
        let mut positions = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let inv_masses = [0.0, 1.0]; // particle 0 is fixed
        c.project(&mut positions, &inv_masses, 0.01);
        // Particle 0 should not move
        assert!((positions[0][0]).abs() < f64::EPSILON);
    }

    #[test]
    fn test_volume_constraint_basic() {
        // Unit tet
        let tet = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let rest_vol = VolumeConstraint::signed_volume(tet);
        assert!((rest_vol - 1.0 / 6.0).abs() < 1e-10);

        let c = VolumeConstraint::new([0, 1, 2, 3], rest_vol, 0.0);
        // Squish the tet
        let mut positions = [
            [0.0, 0.0, 0.0],
            [0.5, 0.0, 0.0],
            [0.0, 0.5, 0.0],
            [0.0, 0.0, 0.5],
        ];
        let inv_masses = [1.0; 4];
        let violation = c.project(&mut positions, &inv_masses, 0.01);
        assert!(violation > 0.0);
    }

    #[test]
    fn test_collision_constraint_no_penetration() {
        let c = CollisionConstraint::new(0, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.0, 0.0);
        let mut positions = [[0.0, 1.0, 0.0]]; // above plane
        let inv_masses = [1.0];
        let v = c.project(&mut positions, &inv_masses, 0.01);
        assert!(v < f64::EPSILON);
    }

    #[test]
    fn test_collision_constraint_pushes_out() {
        let c = CollisionConstraint::new(0, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.0, 0.0);
        let mut positions = [[0.0, -0.5, 0.0]]; // below plane
        let inv_masses = [1.0];
        let v = c.project(&mut positions, &inv_masses, 0.01);
        assert!(v > 0.0);
        assert!(positions[0][1] > -0.5); // pushed up
    }

    #[test]
    fn test_particle_collision_separates() {
        let c = ParticleCollisionConstraint::new(0, 1, 1.0, 0.0);
        let mut positions = [[0.0, 0.0, 0.0], [0.3, 0.0, 0.0]]; // overlapping
        let inv_masses = [1.0, 1.0];
        let v = c.project(&mut positions, &inv_masses, 0.01);
        assert!(v > 0.0);
        let new_dist = vec3_length(vec3_sub(positions[1], positions[0]));
        assert!(new_dist > 0.3);
    }

    #[test]
    fn test_shape_matching_basic() {
        let indices = vec![0, 1, 2];
        let rest = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let masses = vec![1.0, 1.0, 1.0];
        let c = ShapeMatchingConstraint::new(indices, &rest, &masses, 0.0, 1.0);

        // Displace one vertex
        let mut positions = vec![[0.0, 0.0, 0.0], [1.0, 0.5, 0.0], [0.0, 1.0, 0.0]];
        let inv_masses = vec![1.0, 1.0, 1.0];
        let v = c.project(&mut positions, &inv_masses, 0.01);
        assert!(v > 0.0);
    }
}
