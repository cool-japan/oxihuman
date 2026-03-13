// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Shape matching constraint for soft bodies (Mueller et al. 2005).
//!
//! Computes a best-fit rigid transformation from deformed particle positions
//! to rest positions, then blends towards the matched shape for soft-body behaviour.

// ── Type aliases ─────────────────────────────────────────────────────────────

/// 3x3 matrix stored row-major as `[[f32; 3]; 3]`.
#[allow(dead_code)]
pub type Mat3 = [[f32; 3]; 3];

// ── Structs ──────────────────────────────────────────────────────────────────

/// A soft body whose shape is maintained via shape matching.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeMatchingBody {
    /// Current particle positions.
    pub positions: Vec<[f32; 3]>,
    /// Rest-pose positions (the target shape).
    pub rest_positions: Vec<[f32; 3]>,
    /// Per-particle masses.
    pub masses: Vec<f32>,
    /// Per-particle velocities.
    pub velocities: Vec<[f32; 3]>,
    /// Total mass (cached).
    pub total_mass: f32,
    /// Rest center of mass (cached).
    pub rest_com: [f32; 3],
}

/// Configuration for the shape matching solver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeMatchingConfig {
    /// Stiffness parameter in [0, 1]. Higher = more rigid.
    pub stiffness: f32,
    /// Time step for integration.
    pub dt: f32,
    /// Number of solver iterations per step.
    pub iterations: usize,
    /// Damping factor for velocity [0, 1].
    pub damping: f32,
}

// ── Default config ───────────────────────────────────────────────────────────

/// Return a sensible default [`ShapeMatchingConfig`].
#[allow(dead_code)]
pub fn default_shape_matching_config() -> ShapeMatchingConfig {
    ShapeMatchingConfig {
        stiffness: 0.8,
        dt: 1.0 / 60.0,
        iterations: 1,
        damping: 0.01,
    }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

/// 3x3 matrix multiply.
#[allow(dead_code)]
#[inline]
fn mat3_mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut r = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    r
}

/// Transpose a 3x3 matrix.
#[inline]
fn mat3_transpose(m: &Mat3) -> Mat3 {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

/// Multiply a 3x3 matrix by a 3-vector.
#[inline]
fn mat3_mul_vec(m: &Mat3, v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

/// Identity 3x3 matrix.
#[inline]
fn mat3_identity() -> Mat3 {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Create a new shape matching body from positions and masses.
#[allow(dead_code)]
pub fn new_shape_matching_body(positions: Vec<[f32; 3]>, masses: Vec<f32>) -> ShapeMatchingBody {
    let n = positions.len();
    let total_mass: f32 = masses.iter().sum();
    let rest_com = compute_com(&positions, &masses, total_mass);

    ShapeMatchingBody {
        rest_positions: positions.clone(),
        positions,
        masses,
        velocities: vec![[0.0; 3]; n],
        total_mass,
        rest_com,
    }
}

/// Compute the center of mass of a set of particles.
#[allow(dead_code)]
pub fn compute_com(positions: &[[f32; 3]], masses: &[f32], total_mass: f32) -> [f32; 3] {
    if total_mass < 1e-12 {
        return [0.0; 3];
    }
    let n = positions.len().min(masses.len());
    let mut com = [0.0f32; 3];
    for i in 0..n {
        com = add3(com, scale3(positions[i], masses[i]));
    }
    scale3(com, 1.0 / total_mass)
}

/// Compute the Apq matrix (cross-covariance of deformed and rest positions).
///
/// `Apq = sum_i( m_i * (p_i - com) * (q_i - rest_com)^T )`
#[allow(dead_code)]
pub fn compute_apq(body: &ShapeMatchingBody) -> Mat3 {
    let n = body.positions.len();
    let com = compute_com(&body.positions, &body.masses, body.total_mass);
    let mut apq = [[0.0f32; 3]; 3];

    for i in 0..n {
        let p = sub3(body.positions[i], com);
        let q = sub3(body.rest_positions[i], body.rest_com);
        let m = body.masses[i];
        for r in 0..3 {
            for c in 0..3 {
                apq[r][c] += m * p[r] * q[c];
            }
        }
    }

    apq
}

/// Extract the rotation matrix from Apq using polar decomposition
/// (iterative approach via Denman-Beavers or Higham iteration).
///
/// Uses a simple iterative method: R_{k+1} = 0.5 * (R_k + R_k^{-T}).
/// Falls back to identity for singular matrices.
#[allow(dead_code)]
pub fn polar_extract_rotation(apq: &Mat3) -> Mat3 {
    let mut r = *apq;

    // 10 iterations of the polar decomposition iteration
    for _ in 0..10 {
        let rt = mat3_transpose(&r);
        // Compute R^{-T} approximately via adjugate / det
        let det = r[0][0] * (r[1][1] * r[2][2] - r[1][2] * r[2][1])
            - r[0][1] * (r[1][0] * r[2][2] - r[1][2] * r[2][0])
            + r[0][2] * (r[1][0] * r[2][1] - r[1][1] * r[2][0]);

        if det.abs() < 1e-10 {
            return mat3_identity();
        }

        // Adjugate of transpose
        let inv_t = [
            [
                rt[1][1] * rt[2][2] - rt[1][2] * rt[2][1],
                rt[0][2] * rt[2][1] - rt[0][1] * rt[2][2],
                rt[0][1] * rt[1][2] - rt[0][2] * rt[1][1],
            ],
            [
                rt[1][2] * rt[2][0] - rt[1][0] * rt[2][2],
                rt[0][0] * rt[2][2] - rt[0][2] * rt[2][0],
                rt[0][2] * rt[1][0] - rt[0][0] * rt[1][2],
            ],
            [
                rt[1][0] * rt[2][1] - rt[1][1] * rt[2][0],
                rt[0][1] * rt[2][0] - rt[0][0] * rt[2][1],
                rt[0][0] * rt[1][1] - rt[0][1] * rt[1][0],
            ],
        ];

        let inv_det = 1.0 / det;

        let mut new_r = [[0.0f32; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                new_r[i][j] = 0.5 * (r[i][j] + inv_t[i][j] * inv_det);
            }
        }
        r = new_r;
    }

    r
}

/// Apply shape matching: move deformed particles towards their matched positions.
///
/// For each particle: `goal_i = R * (rest_i - rest_com) + com`
/// Then blend: `new_i = lerp(current_i, goal_i, stiffness)`
#[allow(dead_code)]
pub fn apply_shape_matching(body: &mut ShapeMatchingBody, cfg: &ShapeMatchingConfig) {
    let n = body.positions.len();
    if n == 0 {
        return;
    }

    for _ in 0..cfg.iterations {
        let com = compute_com(&body.positions, &body.masses, body.total_mass);
        let apq = compute_apq(body);
        let rot = polar_extract_rotation(&apq);

        for i in 0..n {
            let q = sub3(body.rest_positions[i], body.rest_com);
            let goal = add3(mat3_mul_vec(&rot, q), com);
            let diff = sub3(goal, body.positions[i]);
            body.positions[i] = add3(body.positions[i], scale3(diff, cfg.stiffness));
        }
    }

    // Apply velocity damping
    for v in &mut body.velocities {
        *v = scale3(*v, 1.0 - cfg.damping);
    }
}

/// Return the stiffness of a shape matching config.
#[allow(dead_code)]
pub fn shape_matching_stiffness(cfg: &ShapeMatchingConfig) -> f32 {
    cfg.stiffness
}

/// Set the mass of a specific particle.
#[allow(dead_code)]
pub fn set_particle_mass(body: &mut ShapeMatchingBody, index: usize, mass: f32) {
    if index < body.masses.len() {
        let old_mass = body.masses[index];
        body.masses[index] = mass;
        body.total_mass += mass - old_mass;
    }
}

/// Return the number of particles in the body.
#[allow(dead_code)]
pub fn shape_matching_particle_count(body: &ShapeMatchingBody) -> usize {
    body.positions.len()
}

/// Reset all particles to their rest positions and zero velocities.
#[allow(dead_code)]
pub fn reset_to_rest(body: &mut ShapeMatchingBody) {
    let n = body.positions.len();
    for i in 0..n {
        body.positions[i] = body.rest_positions[i];
        body.velocities[i] = [0.0; 3];
    }
}

/// Estimate the volume of the body using the bounding box of current positions.
#[allow(dead_code)]
pub fn body_volume_estimate(body: &ShapeMatchingBody) -> f32 {
    if body.positions.is_empty() {
        return 0.0;
    }
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for p in &body.positions {
        for k in 0..3 {
            min[k] = min[k].min(p[k]);
            max[k] = max[k].max(p[k]);
        }
    }
    (max[0] - min[0]) * (max[1] - min[1]) * (max[2] - min[2])
}

/// Compute the deformation energy: sum of squared displacements from matched shape.
///
/// Energy = sum_i( m_i * |p_i - goal_i|^2 )
#[allow(dead_code)]
pub fn deformation_energy(body: &ShapeMatchingBody) -> f32 {
    let n = body.positions.len();
    if n == 0 {
        return 0.0;
    }
    let com = compute_com(&body.positions, &body.masses, body.total_mass);
    let apq = compute_apq(body);
    let rot = polar_extract_rotation(&apq);

    let mut energy = 0.0f32;
    for i in 0..n {
        let q = sub3(body.rest_positions[i], body.rest_com);
        let goal = add3(mat3_mul_vec(&rot, q), com);
        let diff = sub3(body.positions[i], goal);
        energy += body.masses[i] * dot3(diff, diff);
    }
    energy
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_body() -> ShapeMatchingBody {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let masses = vec![1.0; 8];
        new_shape_matching_body(positions, masses)
    }

    fn cfg() -> ShapeMatchingConfig {
        default_shape_matching_config()
    }

    #[test]
    fn test_default_shape_matching_config() {
        let c = default_shape_matching_config();
        assert!(c.stiffness > 0.0);
        assert!(c.stiffness <= 1.0);
        assert!(c.dt > 0.0);
    }

    #[test]
    fn test_new_shape_matching_body() {
        let body = cube_body();
        assert_eq!(body.positions.len(), 8);
        assert_eq!(body.rest_positions.len(), 8);
        assert!((body.total_mass - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_com_uniform() {
        let body = cube_body();
        let com = compute_com(&body.positions, &body.masses, body.total_mass);
        assert!((com[0] - 0.5).abs() < 1e-5);
        assert!((com[1] - 0.5).abs() < 1e-5);
        assert!((com[2] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_compute_com_empty() {
        let com = compute_com(&[], &[], 0.0);
        assert_eq!(com, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_compute_apq_at_rest() {
        let body = cube_body();
        let apq = compute_apq(&body);
        // At rest, Apq should be a symmetric positive matrix
        for row in &apq {
            for &val in row {
                assert!(val.is_finite());
            }
        }
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_polar_extract_rotation_identity() {
        let id = mat3_identity();
        let rot = polar_extract_rotation(&id);
        #[allow(clippy::needless_range_loop)]
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (rot[i][j] - expected).abs() < 1e-4,
                    "rot[{i}][{j}] mismatch"
                );
            }
        }
    }

    #[test]
    fn test_polar_extract_rotation_singular() {
        let zero = [[0.0f32; 3]; 3];
        let rot = polar_extract_rotation(&zero);
        // Should fall back to identity
        assert!((rot[0][0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_apply_shape_matching_at_rest() {
        let mut body = cube_body();
        let c = cfg();
        let original = body.positions.clone();
        apply_shape_matching(&mut body, &c);
        // Should remain at rest
        for (i, p) in body.positions.iter().enumerate() {
            for k in 0..3 {
                assert!(
                    (p[k] - original[i][k]).abs() < 0.1,
                    "vertex {i} should stay near rest"
                );
            }
        }
    }

    #[test]
    fn test_apply_shape_matching_deformed() {
        let mut body = cube_body();
        // Deform: shift vertex 0 far away
        body.positions[0] = [5.0, 5.0, 5.0];
        let c = cfg();
        let dist_before = len3(sub3(body.positions[0], body.rest_positions[0]));
        apply_shape_matching(&mut body, &c);
        let dist_after = len3(sub3(body.positions[0], body.rest_positions[0]));
        // After shape matching, vertex should be pulled closer to rest
        assert!(
            dist_after < dist_before,
            "shape matching should reduce deformation"
        );
    }

    #[test]
    fn test_shape_matching_stiffness() {
        let c = cfg();
        assert!((shape_matching_stiffness(&c) - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_set_particle_mass() {
        let mut body = cube_body();
        let old_total = body.total_mass;
        set_particle_mass(&mut body, 0, 5.0);
        assert!((body.masses[0] - 5.0).abs() < 1e-5);
        assert!((body.total_mass - (old_total + 4.0)).abs() < 1e-5);
    }

    #[test]
    fn test_set_particle_mass_out_of_range() {
        let mut body = cube_body();
        let old_total = body.total_mass;
        set_particle_mass(&mut body, 100, 5.0);
        assert!((body.total_mass - old_total).abs() < 1e-5);
    }

    #[test]
    fn test_shape_matching_particle_count() {
        let body = cube_body();
        assert_eq!(shape_matching_particle_count(&body), 8);
    }

    #[test]
    fn test_reset_to_rest() {
        let mut body = cube_body();
        body.positions[0] = [99.0, 99.0, 99.0];
        body.velocities[0] = [10.0, 10.0, 10.0];
        reset_to_rest(&mut body);
        assert_eq!(body.positions[0], body.rest_positions[0]);
        assert_eq!(body.velocities[0], [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_body_volume_estimate() {
        let body = cube_body();
        let vol = body_volume_estimate(&body);
        assert!((vol - 1.0).abs() < 1e-5, "unit cube volume should be 1");
    }

    #[test]
    fn test_body_volume_estimate_empty() {
        let body = new_shape_matching_body(vec![], vec![]);
        assert_eq!(body_volume_estimate(&body), 0.0);
    }

    #[test]
    fn test_deformation_energy_at_rest() {
        let body = cube_body();
        let e = deformation_energy(&body);
        assert!(e < 0.01, "energy at rest should be ~0, got {e}");
    }

    #[test]
    fn test_deformation_energy_deformed() {
        let mut body = cube_body();
        body.positions[0] = [10.0, 10.0, 10.0];
        let e = deformation_energy(&body);
        assert!(e > 1.0, "deformed body should have non-zero energy");
    }

    #[test]
    fn test_mat3_mul_identity() {
        let a = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        let id = mat3_identity();
        let r = mat3_mul(&a, &id);
        for i in 0..3 {
            for j in 0..3 {
                assert!((r[i][j] - a[i][j]).abs() < 1e-6);
            }
        }
    }
}
