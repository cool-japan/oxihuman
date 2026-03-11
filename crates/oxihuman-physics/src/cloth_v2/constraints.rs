// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Constraint types for XPBD cloth simulation.
//!
//! Implements three constraint types:
//! - [`DistanceConstraint`]: Maintains rest-length between connected vertices
//! - [`DihedralBendConstraint`]: Maintains dihedral angle between adjacent triangles
//! - [`AreaConservationConstraint`]: Preserves triangle area

use std::collections::HashSet;

/// A distance (stretch/shear) constraint between two vertices.
///
/// In XPBD, the constraint function is:
///   C(p_i, p_j) = |p_i - p_j| - d
/// where d is the rest length.
#[derive(Debug, Clone)]
pub struct DistanceConstraint {
    /// First vertex index.
    pub i: usize,
    /// Second vertex index.
    pub j: usize,
    /// Rest length (equilibrium distance).
    pub rest_length: f64,
    /// Lagrange multiplier for XPBD (accumulated per substep).
    lambda: f64,
}

impl DistanceConstraint {
    /// Create a new distance constraint.
    pub fn new(i: usize, j: usize, rest_length: f64) -> Self {
        Self {
            i,
            j,
            rest_length,
            lambda: 0.0,
        }
    }

    /// Create a distance constraint from vertex positions, computing rest length automatically.
    pub fn from_positions(i: usize, j: usize, positions: &[[f64; 3]]) -> Option<Self> {
        let pi = positions.get(i)?;
        let pj = positions.get(j)?;
        let d = vec3_dist(pi, pj);
        Some(Self::new(i, j, d))
    }

    /// Reset the Lagrange multiplier (call at the start of each time step).
    pub fn reset_lambda(&mut self) {
        self.lambda = 0.0;
    }

    /// Project this constraint using XPBD with compliance.
    ///
    /// Updates positions in place and returns the constraint error magnitude.
    ///
    /// `compliance` is the inverse stiffness (alpha). A value of 0 means
    /// infinitely stiff. `dt` is the substep time.
    pub fn project(
        &mut self,
        positions: &mut [[f64; 3]],
        inv_masses: &[f64],
        compliance: f64,
        dt: f64,
    ) -> f64 {
        let wi = match inv_masses.get(self.i) {
            Some(&w) => w,
            None => return 0.0,
        };
        let wj = match inv_masses.get(self.j) {
            Some(&w) => w,
            None => return 0.0,
        };
        let w_sum = wi + wj;
        if w_sum < 1e-30 {
            return 0.0;
        }

        let pi = match positions.get(self.i) {
            Some(p) => *p,
            None => return 0.0,
        };
        let pj = match positions.get(self.j) {
            Some(p) => *p,
            None => return 0.0,
        };

        let diff = vec3_sub(&pi, &pj);
        let dist = vec3_len(&diff);
        if dist < 1e-30 {
            return 0.0;
        }

        let c = dist - self.rest_length;

        // XPBD: alpha_tilde = compliance / dt^2
        let alpha_tilde = compliance / (dt * dt);
        let delta_lambda = (-c - alpha_tilde * self.lambda) / (w_sum + alpha_tilde);
        self.lambda += delta_lambda;

        let correction = delta_lambda / dist;
        let dp = vec3_scale(&diff, correction);

        if let Some(p) = positions.get_mut(self.i) {
            p[0] += dp[0] * wi;
            p[1] += dp[1] * wi;
            p[2] += dp[2] * wi;
        }
        if let Some(p) = positions.get_mut(self.j) {
            p[0] -= dp[0] * wj;
            p[1] -= dp[1] * wj;
            p[2] -= dp[2] * wj;
        }

        c.abs()
    }
}

/// A dihedral bend constraint between four vertices forming two adjacent triangles.
///
/// Given triangles (v0, v1, v2) and (v0, v1, v3) sharing edge (v0, v1),
/// this constraint maintains the dihedral angle between the two triangle normals.
#[derive(Debug, Clone)]
pub struct DihedralBendConstraint {
    /// First vertex of shared edge.
    pub v0: usize,
    /// Second vertex of shared edge.
    pub v1: usize,
    /// Opposite vertex in first triangle.
    pub v2: usize,
    /// Opposite vertex in second triangle.
    pub v3: usize,
    /// Rest dihedral angle in radians.
    pub rest_angle: f64,
    /// Lagrange multiplier for XPBD.
    lambda: f64,
}

impl DihedralBendConstraint {
    /// Create a new dihedral bend constraint with a given rest angle.
    pub fn new(v0: usize, v1: usize, v2: usize, v3: usize, rest_angle: f64) -> Self {
        Self {
            v0,
            v1,
            v2,
            v3,
            rest_angle,
            lambda: 0.0,
        }
    }

    /// Create from positions, computing the rest angle from the current configuration.
    pub fn from_positions(
        v0: usize,
        v1: usize,
        v2: usize,
        v3: usize,
        positions: &[[f64; 3]],
    ) -> Option<Self> {
        let angle = compute_dihedral_angle(v0, v1, v2, v3, positions)?;
        Some(Self::new(v0, v1, v2, v3, angle))
    }

    /// Reset Lagrange multiplier.
    pub fn reset_lambda(&mut self) {
        self.lambda = 0.0;
    }

    /// Project this dihedral bend constraint.
    ///
    /// Uses the gradient of the dihedral angle with respect to positions
    /// to compute position corrections.
    pub fn project(
        &mut self,
        positions: &mut [[f64; 3]],
        inv_masses: &[f64],
        compliance: f64,
        dt: f64,
    ) -> f64 {
        let (p0, p1, p2, p3) = match get_four_positions(positions, self.v0, self.v1, self.v2, self.v3) {
            Some(v) => v,
            None => return 0.0,
        };

        let (w0, w1, w2, w3) = match get_four_inv_masses(inv_masses, self.v0, self.v1, self.v2, self.v3) {
            Some(v) => v,
            None => return 0.0,
        };

        // Compute edge vectors
        let e0 = vec3_sub(&p1, &p0); // shared edge
        let e1 = vec3_sub(&p2, &p0); // to opposite vertex in tri 1
        let e2 = vec3_sub(&p3, &p0); // to opposite vertex in tri 2

        // Triangle normals (not normalized)
        let n1 = vec3_cross(&e0, &e1);
        let n2 = vec3_cross(&e0, &e2);

        let n1_len = vec3_len(&n1);
        let n2_len = vec3_len(&n2);
        if n1_len < 1e-30 || n2_len < 1e-30 {
            return 0.0;
        }

        let n1_hat = vec3_scale(&n1, 1.0 / n1_len);
        let n2_hat = vec3_scale(&n2, 1.0 / n2_len);

        // Current dihedral angle
        let cos_theta = vec3_dot(&n1_hat, &n2_hat).clamp(-1.0, 1.0);
        let sin_check = vec3_dot(&vec3_cross(&n1_hat, &n2_hat), &e0);
        let current_angle = cos_theta.acos().copysign(sin_check);

        let c = current_angle - self.rest_angle;
        if c.abs() < 1e-12 {
            return 0.0;
        }

        // Compute gradients of the dihedral angle w.r.t. each vertex
        let e0_len = vec3_len(&e0);
        if e0_len < 1e-30 {
            return 0.0;
        }

        let e0_hat = vec3_scale(&e0, 1.0 / e0_len);

        // Gradient for v2 (opposite vertex in triangle 1)
        let grad_v2 = vec3_scale(&n1_hat, e0_len / (n1_len));
        // Gradient for v3 (opposite vertex in triangle 2)
        let grad_v3 = vec3_scale(&n2_hat, -(e0_len / (n2_len)));

        // Gradient for v0 and v1 from edge contributions
        let d02 = vec3_dot(&e1, &e0_hat);
        let d03 = vec3_dot(&e2, &e0_hat);

        let grad_v0_part1 = vec3_scale(&grad_v2, -(1.0 - d02 / e0_len));
        let grad_v0_part2 = vec3_scale(&grad_v3, -(1.0 - d03 / e0_len));
        let grad_v0 = vec3_add(&grad_v0_part1, &grad_v0_part2);

        let grad_v1_part1 = vec3_scale(&grad_v2, -(d02 / e0_len));
        let grad_v1_part2 = vec3_scale(&grad_v3, -(d03 / e0_len));
        let grad_v1 = vec3_add(&grad_v1_part1, &grad_v1_part2);

        // Weighted gradient magnitude sum
        let w_grad = w0 * vec3_dot(&grad_v0, &grad_v0)
            + w1 * vec3_dot(&grad_v1, &grad_v1)
            + w2 * vec3_dot(&grad_v2, &grad_v2)
            + w3 * vec3_dot(&grad_v3, &grad_v3);

        let alpha_tilde = compliance / (dt * dt);
        let denom = w_grad + alpha_tilde;
        if denom < 1e-30 {
            return 0.0;
        }

        let delta_lambda = (-c - alpha_tilde * self.lambda) / denom;
        self.lambda += delta_lambda;

        // Apply corrections
        apply_correction(positions, self.v0, &grad_v0, delta_lambda * w0);
        apply_correction(positions, self.v1, &grad_v1, delta_lambda * w1);
        apply_correction(positions, self.v2, &grad_v2, delta_lambda * w2);
        apply_correction(positions, self.v3, &grad_v3, delta_lambda * w3);

        c.abs()
    }
}

/// Area conservation constraint for a single triangle.
///
/// Preserves the area of a triangle to prevent excessive compression or stretch.
#[derive(Debug, Clone)]
pub struct AreaConservationConstraint {
    /// First vertex of the triangle.
    pub v0: usize,
    /// Second vertex of the triangle.
    pub v1: usize,
    /// Third vertex of the triangle.
    pub v2: usize,
    /// Rest area of the triangle.
    pub rest_area: f64,
    /// Lagrange multiplier for XPBD.
    lambda: f64,
}

impl AreaConservationConstraint {
    /// Create a new area conservation constraint.
    pub fn new(v0: usize, v1: usize, v2: usize, rest_area: f64) -> Self {
        Self {
            v0,
            v1,
            v2,
            rest_area,
            lambda: 0.0,
        }
    }

    /// Create from positions, computing rest area from current configuration.
    pub fn from_positions(
        v0: usize,
        v1: usize,
        v2: usize,
        positions: &[[f64; 3]],
    ) -> Option<Self> {
        let area = compute_triangle_area(v0, v1, v2, positions)?;
        Some(Self::new(v0, v1, v2, area))
    }

    /// Reset Lagrange multiplier.
    pub fn reset_lambda(&mut self) {
        self.lambda = 0.0;
    }

    /// Project this area conservation constraint.
    ///
    /// The constraint function is C = A - A_rest where A is the current area.
    /// Gradients are computed analytically from the cross product formulation.
    pub fn project(
        &mut self,
        positions: &mut [[f64; 3]],
        inv_masses: &[f64],
        compliance: f64,
        dt: f64,
    ) -> f64 {
        let (p0, p1, p2) = match get_three_positions(positions, self.v0, self.v1, self.v2) {
            Some(v) => v,
            None => return 0.0,
        };
        let (w0, w1, w2) = match get_three_inv_masses(inv_masses, self.v0, self.v1, self.v2) {
            Some(v) => v,
            None => return 0.0,
        };

        let e01 = vec3_sub(&p1, &p0);
        let e02 = vec3_sub(&p2, &p0);
        let cross = vec3_cross(&e01, &e02);
        let twice_area = vec3_len(&cross);

        if twice_area < 1e-30 {
            return 0.0;
        }

        let area = twice_area * 0.5;
        let c = area - self.rest_area;

        // Normal of the triangle
        let n = vec3_scale(&cross, 1.0 / twice_area);

        // Gradients of the area w.r.t. each vertex:
        // grad_v0 = 0.5 * n x (p2 - p1)
        // grad_v1 = 0.5 * n x (p0 - p2)
        // grad_v2 = 0.5 * n x (p1 - p0)
        let e12 = vec3_sub(&p2, &p1);
        let e20 = vec3_sub(&p0, &p2);
        let e01_neg = vec3_sub(&p1, &p0);

        let grad_v0 = vec3_scale(&vec3_cross(&n, &e12), 0.5);
        let grad_v1 = vec3_scale(&vec3_cross(&n, &e20), 0.5);
        let grad_v2 = vec3_scale(&vec3_cross(&n, &e01_neg), 0.5);

        let w_grad = w0 * vec3_dot(&grad_v0, &grad_v0)
            + w1 * vec3_dot(&grad_v1, &grad_v1)
            + w2 * vec3_dot(&grad_v2, &grad_v2);

        let alpha_tilde = compliance / (dt * dt);
        let denom = w_grad + alpha_tilde;
        if denom < 1e-30 {
            return 0.0;
        }

        let delta_lambda = (-c - alpha_tilde * self.lambda) / denom;
        self.lambda += delta_lambda;

        apply_correction(positions, self.v0, &grad_v0, delta_lambda * w0);
        apply_correction(positions, self.v1, &grad_v1, delta_lambda * w1);
        apply_correction(positions, self.v2, &grad_v2, delta_lambda * w2);

        c.abs()
    }
}

/// Build all distance constraints from a triangle mesh.
///
/// Each edge in the mesh gets one distance constraint. Duplicate edges
/// (shared between triangles) are deduplicated.
pub fn build_distance_constraints(
    triangles: &[[usize; 3]],
    positions: &[[f64; 3]],
) -> Vec<DistanceConstraint> {
    let mut edge_set = HashSet::new();
    let mut constraints = Vec::new();

    for tri in triangles {
        let edges = [
            (tri[0], tri[1]),
            (tri[1], tri[2]),
            (tri[2], tri[0]),
        ];
        for (a, b) in edges {
            let key = if a < b { (a, b) } else { (b, a) };
            if edge_set.insert(key) {
                if let Some(c) = DistanceConstraint::from_positions(key.0, key.1, positions) {
                    constraints.push(c);
                }
            }
        }
    }

    constraints
}

/// Build dihedral bend constraints from adjacent triangle pairs.
///
/// For each interior edge shared by two triangles, creates a dihedral
/// constraint between the four involved vertices.
pub fn build_bend_constraints(
    triangles: &[[usize; 3]],
    positions: &[[f64; 3]],
) -> Vec<DihedralBendConstraint> {
    use std::collections::HashMap;

    // Map each edge to the triangles that share it
    let mut edge_to_tris: HashMap<(usize, usize), Vec<usize>> = HashMap::new();

    for (ti, tri) in triangles.iter().enumerate() {
        let edges = [
            (tri[0], tri[1]),
            (tri[1], tri[2]),
            (tri[2], tri[0]),
        ];
        for (a, b) in edges {
            let key = if a < b { (a, b) } else { (b, a) };
            edge_to_tris.entry(key).or_default().push(ti);
        }
    }

    let mut constraints = Vec::new();

    for ((v0, v1), tri_indices) in &edge_to_tris {
        if tri_indices.len() != 2 {
            continue; // boundary edge or non-manifold
        }
        let t0 = tri_indices[0];
        let t1 = tri_indices[1];

        // Find opposite vertices
        let opp0 = find_opposite_vertex(&triangles[t0], *v0, *v1);
        let opp1 = find_opposite_vertex(&triangles[t1], *v0, *v1);

        if let (Some(v2), Some(v3)) = (opp0, opp1) {
            if let Some(c) = DihedralBendConstraint::from_positions(*v0, *v1, v2, v3, positions) {
                constraints.push(c);
            }
        }
    }

    constraints
}

/// Build area conservation constraints for all triangles.
pub fn build_area_constraints(
    triangles: &[[usize; 3]],
    positions: &[[f64; 3]],
) -> Vec<AreaConservationConstraint> {
    let mut constraints = Vec::with_capacity(triangles.len());

    for tri in triangles {
        if let Some(c) = AreaConservationConstraint::from_positions(tri[0], tri[1], tri[2], positions) {
            constraints.push(c);
        }
    }

    constraints
}

// ─── Helpers ───────────────────────────────────────────────────────────────

/// Find the vertex in a triangle that is not `a` or `b`.
fn find_opposite_vertex(tri: &[usize; 3], a: usize, b: usize) -> Option<usize> {
    for &v in tri {
        if v != a && v != b {
            return Some(v);
        }
    }
    None
}

/// Compute the dihedral angle between two triangles sharing edge (v0, v1).
fn compute_dihedral_angle(
    v0: usize,
    v1: usize,
    v2: usize,
    v3: usize,
    positions: &[[f64; 3]],
) -> Option<f64> {
    let p0 = positions.get(v0)?;
    let p1 = positions.get(v1)?;
    let p2 = positions.get(v2)?;
    let p3 = positions.get(v3)?;

    let e0 = vec3_sub(p1, p0);
    let e1 = vec3_sub(p2, p0);
    let e2 = vec3_sub(p3, p0);

    let n1 = vec3_cross(&e0, &e1);
    let n2 = vec3_cross(&e0, &e2);

    let n1_len = vec3_len(&n1);
    let n2_len = vec3_len(&n2);
    if n1_len < 1e-30 || n2_len < 1e-30 {
        return Some(0.0);
    }

    let n1_hat = vec3_scale(&n1, 1.0 / n1_len);
    let n2_hat = vec3_scale(&n2, 1.0 / n2_len);

    let cos_theta = vec3_dot(&n1_hat, &n2_hat).clamp(-1.0, 1.0);
    let sin_sign = vec3_dot(&vec3_cross(&n1_hat, &n2_hat), &e0);

    Some(cos_theta.acos().copysign(sin_sign))
}

/// Compute the area of a triangle.
fn compute_triangle_area(
    v0: usize,
    v1: usize,
    v2: usize,
    positions: &[[f64; 3]],
) -> Option<f64> {
    let p0 = positions.get(v0)?;
    let p1 = positions.get(v1)?;
    let p2 = positions.get(v2)?;

    let e01 = vec3_sub(p1, p0);
    let e02 = vec3_sub(p2, p0);
    let cross = vec3_cross(&e01, &e02);

    Some(vec3_len(&cross) * 0.5)
}

fn get_four_positions(
    positions: &[[f64; 3]],
    i0: usize,
    i1: usize,
    i2: usize,
    i3: usize,
) -> Option<([f64; 3], [f64; 3], [f64; 3], [f64; 3])> {
    Some((*positions.get(i0)?, *positions.get(i1)?, *positions.get(i2)?, *positions.get(i3)?))
}

fn get_four_inv_masses(
    inv_masses: &[f64],
    i0: usize,
    i1: usize,
    i2: usize,
    i3: usize,
) -> Option<(f64, f64, f64, f64)> {
    Some((*inv_masses.get(i0)?, *inv_masses.get(i1)?, *inv_masses.get(i2)?, *inv_masses.get(i3)?))
}

fn get_three_positions(
    positions: &[[f64; 3]],
    i0: usize,
    i1: usize,
    i2: usize,
) -> Option<([f64; 3], [f64; 3], [f64; 3])> {
    Some((*positions.get(i0)?, *positions.get(i1)?, *positions.get(i2)?))
}

fn get_three_inv_masses(
    inv_masses: &[f64],
    i0: usize,
    i1: usize,
    i2: usize,
) -> Option<(f64, f64, f64)> {
    Some((*inv_masses.get(i0)?, *inv_masses.get(i1)?, *inv_masses.get(i2)?))
}

fn apply_correction(positions: &mut [[f64; 3]], idx: usize, grad: &[f64; 3], scale: f64) {
    if let Some(p) = positions.get_mut(idx) {
        p[0] += grad[0] * scale;
        p[1] += grad[1] * scale;
        p[2] += grad[2] * scale;
    }
}

// ─── Vec3 math ─────────────────────────────────────────────────────────────

#[inline]
fn vec3_sub(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_add(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn vec3_scale(a: &[f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn vec3_dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn vec3_len(a: &[f64; 3]) -> f64 {
    vec3_dot(a, a).sqrt()
}

#[inline]
fn vec3_dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let d = vec3_sub(a, b);
    vec3_len(&d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quad() -> (Vec<[f64; 3]>, Vec<[usize; 3]>) {
        // A simple quad: two triangles
        //  2---3
        //  |\ |
        //  | \|
        //  0---1
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
        ];
        let triangles = vec![[0, 1, 2], [1, 3, 2]];
        (positions, triangles)
    }

    #[test]
    fn test_distance_constraint_at_rest() {
        let (mut positions, _) = make_quad();
        let inv_masses = vec![1.0; 4];
        let mut c = DistanceConstraint::from_positions(0, 1, &positions).unwrap();

        let error = c.project(&mut positions, &inv_masses, 0.0, 1.0 / 60.0);
        assert!(error < 1e-10, "At rest, error should be ~0");
    }

    #[test]
    fn test_distance_constraint_stretched() {
        let mut positions = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let inv_masses = vec![1.0, 1.0];
        let mut c = DistanceConstraint::new(0, 1, 1.0);

        let error = c.project(&mut positions, &inv_masses, 0.0, 1.0 / 60.0);
        assert!(error > 0.5, "Should detect stretch");

        // After projection, should be closer to rest length
        let d = vec3_dist(&positions[0], &positions[1]);
        assert!((d - 1.0).abs() < 0.1, "Should converge toward rest length");
    }

    #[test]
    fn test_build_distance_constraints_dedup() {
        let (positions, triangles) = make_quad();
        let constraints = build_distance_constraints(&triangles, &positions);
        // A quad with 2 triangles has 5 unique edges
        assert_eq!(constraints.len(), 5);
    }

    #[test]
    fn test_build_bend_constraints() {
        let (positions, triangles) = make_quad();
        let constraints = build_bend_constraints(&triangles, &positions);
        // One interior edge -> one bend constraint
        assert_eq!(constraints.len(), 1);
    }

    #[test]
    fn test_area_constraint_at_rest() {
        let (mut positions, _) = make_quad();
        let inv_masses = vec![1.0; 4];
        let mut c = AreaConservationConstraint::from_positions(0, 1, 2, &positions).unwrap();

        let error = c.project(&mut positions, &inv_masses, 0.0, 1.0 / 60.0);
        assert!(error < 1e-10, "At rest, area error should be ~0");
    }

    #[test]
    fn test_dihedral_flat_rest_angle() {
        let (positions, triangles) = make_quad();
        let bends = build_bend_constraints(&triangles, &positions);
        assert_eq!(bends.len(), 1);
        // A flat quad should have rest angle near 0 or PI
        // (depends on orientation convention)
        assert!(
            bends[0].rest_angle.abs() < 0.1 || (bends[0].rest_angle.abs() - std::f64::consts::PI).abs() < 0.1,
            "Flat quad rest angle = {}",
            bends[0].rest_angle
        );
    }
}
