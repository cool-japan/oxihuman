// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Rigid body dynamics with inertia tensors, torques, and angular momentum.

// ── Config ────────────────────────────────────────────────────────────────────

/// Configuration for rigid body construction and integration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigidBodyConfig {
    /// Coefficient of restitution (bounciness) in [0, 1].
    pub restitution: f32,
    /// Friction coefficient.
    pub friction: f32,
    /// Linear velocity damping per second.
    pub linear_damping: f32,
    /// Angular velocity damping per second.
    pub angular_damping: f32,
    /// Gravitational acceleration vector.
    pub gravity: [f32; 3],
}

/// Return a sensible default [`RigidBodyConfig`].
#[allow(dead_code)]
pub fn default_rigid_body_config() -> RigidBodyConfig {
    RigidBodyConfig {
        restitution: 0.3,
        friction: 0.5,
        linear_damping: 0.01,
        angular_damping: 0.01,
        gravity: [0.0, -9.81, 0.0],
    }
}

#[allow(dead_code)]
/// A rigid body with full 3-DOF rotation support.
#[derive(Debug, Clone)]
pub struct RigidBody {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    /// Quaternion stored as [x, y, z, w].
    pub orientation: [f32; 4],
    pub angular_velocity: [f32; 3],
    /// Inverse mass; 0.0 means static/infinite mass.
    pub inv_mass: f32,
    /// Inverse inertia tensor in world space (3×3 row-major).
    pub inv_inertia: [[f32; 3]; 3],
    pub restitution: f32,
    pub friction: f32,
}

#[allow(dead_code)]
/// Snapshot of a rigid body's kinematic state.
#[derive(Debug, Clone)]
pub struct RigidBodyState {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub orientation: [f32; 4],
    pub angular_velocity: [f32; 3],
}

// ── Matrix helpers ────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn mat3_inv_diagonal(diag: [f32; 3]) -> [[f32; 3]; 3] {
    [
        [if diag[0] != 0.0 { 1.0 / diag[0] } else { 0.0 }, 0.0, 0.0],
        [0.0, if diag[1] != 0.0 { 1.0 / diag[1] } else { 0.0 }, 0.0],
        [0.0, 0.0, if diag[2] != 0.0 { 1.0 / diag[2] } else { 0.0 }],
    ]
}

/// Build the symmetric inertia tensor for a uniform-density sphere.
///
/// I = (2/5) * m * r²  on each diagonal axis.
#[allow(dead_code)]
pub fn mat3_sym_inertia_sphere(mass: f32, radius: f32) -> [[f32; 3]; 3] {
    let i = 2.0 / 5.0 * mass * radius * radius;
    [[i, 0.0, 0.0], [0.0, i, 0.0], [0.0, 0.0, i]]
}

/// Build the symmetric inertia tensor for a uniform-density box.
///
/// `extents` = full side lengths [lx, ly, lz].
/// I_x = m/12 * (ly² + lz²),  I_y = m/12 * (lx² + lz²),  I_z = m/12 * (lx² + ly²).
#[allow(dead_code)]
pub fn mat3_sym_inertia_box(mass: f32, extents: [f32; 3]) -> [[f32; 3]; 3] {
    let [lx, ly, lz] = extents;
    let factor = mass / 12.0;
    let ix = factor * (ly * ly + lz * lz);
    let iy = factor * (lx * lx + lz * lz);
    let iz = factor * (lx * lx + ly * ly);
    [[ix, 0.0, 0.0], [0.0, iy, 0.0], [0.0, 0.0, iz]]
}

#[allow(dead_code)]
fn mat3_mul_vec3(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

#[allow(dead_code)]
fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[allow(dead_code)]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn vec3_scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

// ── Quaternion helpers ────────────────────────────────────────────────────────

/// Create a quaternion from an axis-angle representation.
///
/// Returns `[x, y, z, w]`.  If the axis has zero length, returns the identity
/// quaternion `[0, 0, 0, 1]`.
#[allow(dead_code)]
pub fn quat_from_axis_angle(axis: [f32; 3], angle: f32) -> [f32; 4] {
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len < 1e-9 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let n = [axis[0] / len, axis[1] / len, axis[2] / len];
    let half = angle * 0.5;
    let s = half.sin();
    [n[0] * s, n[1] * s, n[2] * s, half.cos()]
}

/// Multiply two quaternions: `a ⊗ b`.
///
/// Both stored as `[x, y, z, w]`.
#[allow(dead_code)]
pub fn quat_multiply(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

/// Normalize a quaternion to unit length.
///
/// Returns the identity quaternion `[0, 0, 0, 1]` for near-zero input.
#[allow(dead_code)]
pub fn quat_normalize(q: [f32; 4]) -> [f32; 4] {
    let len_sq = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
    if len_sq < 1e-18 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let inv = 1.0 / len_sq.sqrt();
    [q[0] * inv, q[1] * inv, q[2] * inv, q[3] * inv]
}

/// Integrate a quaternion orientation by an angular velocity vector for time `dt`.
///
/// Uses the first-order approximation: q += 0.5 * dt * [ω, 0] ⊗ q, then normalizes.
#[allow(dead_code)]
pub fn integrate_orientation(q: [f32; 4], omega: [f32; 3], dt: f32) -> [f32; 4] {
    // ω as a pure quaternion [ωx, ωy, ωz, 0]
    let omega_q = [omega[0], omega[1], omega[2], 0.0];
    let dq = quat_multiply(omega_q, q);
    let half_dt = 0.5 * dt;
    let q_new = [
        q[0] + half_dt * dq[0],
        q[1] + half_dt * dq[1],
        q[2] + half_dt * dq[2],
        q[3] + half_dt * dq[3],
    ];
    quat_normalize(q_new)
}

// ── RigidBody impl ────────────────────────────────────────────────────────────

impl RigidBody {
    /// Construct a box-shaped rigid body with the given half-extents and mass.
    ///
    /// If `mass == 0.0`, the body is treated as static (`inv_mass == 0`).
    #[allow(dead_code)]
    pub fn new_box(half_extents: [f32; 3], mass: f32) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        let extents = [
            half_extents[0] * 2.0,
            half_extents[1] * 2.0,
            half_extents[2] * 2.0,
        ];
        let inertia = mat3_sym_inertia_box(mass, extents);
        // Invert the diagonal elements
        let inv_inertia = mat3_inv_diagonal([inertia[0][0], inertia[1][1], inertia[2][2]]);
        RigidBody {
            position: [0.0; 3],
            velocity: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
            angular_velocity: [0.0; 3],
            inv_mass,
            inv_inertia,
            restitution: 0.3,
            friction: 0.5,
        }
    }

    /// Construct a sphere-shaped rigid body.
    #[allow(dead_code)]
    pub fn new_sphere(radius: f32, mass: f32) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        let inertia = mat3_sym_inertia_sphere(mass, radius);
        let inv_inertia = mat3_inv_diagonal([inertia[0][0], inertia[1][1], inertia[2][2]]);
        RigidBody {
            position: [0.0; 3],
            velocity: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
            angular_velocity: [0.0; 3],
            inv_mass,
            inv_inertia,
            restitution: 0.3,
            friction: 0.5,
        }
    }

    /// Construct a capsule-shaped rigid body.
    ///
    /// Uses the cylinder approximation for the inertia tensor.
    #[allow(dead_code)]
    pub fn new_capsule(radius: f32, half_height: f32, mass: f32) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        // Approximate as a cylinder for inertia
        let height = half_height * 2.0;
        let ix = mass / 12.0 * (3.0 * radius * radius + height * height);
        let iy = 0.5 * mass * radius * radius; // around the long axis
        let iz = ix;
        let inv_inertia = mat3_inv_diagonal([ix, iy, iz]);
        RigidBody {
            position: [0.0; 3],
            velocity: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
            angular_velocity: [0.0; 3],
            inv_mass,
            inv_inertia,
            restitution: 0.3,
            friction: 0.5,
        }
    }

    /// Apply a linear force over a time step: v += F * inv_mass * dt.
    #[allow(dead_code)]
    pub fn apply_force(&mut self, force: [f32; 3], dt: f32) {
        if self.inv_mass == 0.0 {
            return;
        }
        let scale = self.inv_mass * dt;
        self.velocity[0] += force[0] * scale;
        self.velocity[1] += force[1] * scale;
        self.velocity[2] += force[2] * scale;
    }

    /// Apply a torque over a time step: ω += I⁻¹ · τ · dt.
    #[allow(dead_code)]
    pub fn apply_torque(&mut self, torque: [f32; 3], dt: f32) {
        if self.inv_mass == 0.0 {
            return;
        }
        let alpha = mat3_mul_vec3(self.inv_inertia, torque);
        self.angular_velocity[0] += alpha[0] * dt;
        self.angular_velocity[1] += alpha[1] * dt;
        self.angular_velocity[2] += alpha[2] * dt;
    }

    /// Apply a linear and angular impulse at a world-space point.
    ///
    /// Linear impulse:  v += J * inv_mass
    /// Angular impulse: ω += I⁻¹ · (r × J)
    /// where r = point − position.
    #[allow(dead_code)]
    pub fn apply_impulse_at_point(&mut self, impulse: [f32; 3], point: [f32; 3]) {
        if self.inv_mass == 0.0 {
            return;
        }
        // Linear
        self.velocity[0] += impulse[0] * self.inv_mass;
        self.velocity[1] += impulse[1] * self.inv_mass;
        self.velocity[2] += impulse[2] * self.inv_mass;
        // Angular
        let r = [
            point[0] - self.position[0],
            point[1] - self.position[1],
            point[2] - self.position[2],
        ];
        let torque = vec3_cross(r, impulse);
        let delta_omega = mat3_mul_vec3(self.inv_inertia, torque);
        self.angular_velocity[0] += delta_omega[0];
        self.angular_velocity[1] += delta_omega[1];
        self.angular_velocity[2] += delta_omega[2];
    }

    /// Semi-implicit Euler integration step.
    ///
    /// 1. v += gravity * dt
    /// 2. x += v * dt
    /// 3. Update orientation from angular velocity.
    #[allow(dead_code)]
    pub fn integrate(&mut self, dt: f32, gravity: [f32; 3]) {
        if self.inv_mass == 0.0 {
            return;
        }
        // Update linear velocity from gravity
        self.velocity[0] += gravity[0] * dt;
        self.velocity[1] += gravity[1] * dt;
        self.velocity[2] += gravity[2] * dt;
        // Update position
        self.position[0] += self.velocity[0] * dt;
        self.position[1] += self.velocity[1] * dt;
        self.position[2] += self.velocity[2] * dt;
        // Update orientation
        self.orientation = integrate_orientation(self.orientation, self.angular_velocity, dt);
    }

    /// Total kinetic energy: ½mv² + ½ωᵀIω.
    #[allow(dead_code)]
    pub fn kinetic_energy(&self) -> f32 {
        let v2 = vec3_dot(self.velocity, self.velocity);
        let linear_ke = if self.inv_mass > 0.0 {
            0.5 / self.inv_mass * v2
        } else {
            0.0
        };
        // Rotational KE: ½ωᵀ · I · ω
        // I = inv_inertia⁻¹ (diagonal case: just reciprocal of inv)
        let omega = self.angular_velocity;
        let rot_ke = 0.5
            * (omega[0]
                * omega[0]
                * (if self.inv_inertia[0][0] > 0.0 {
                    1.0 / self.inv_inertia[0][0]
                } else {
                    0.0
                })
                + omega[1]
                    * omega[1]
                    * (if self.inv_inertia[1][1] > 0.0 {
                        1.0 / self.inv_inertia[1][1]
                    } else {
                        0.0
                    })
                + omega[2]
                    * omega[2]
                    * (if self.inv_inertia[2][2] > 0.0 {
                        1.0 / self.inv_inertia[2][2]
                    } else {
                        0.0
                    }));
        linear_ke + rot_ke
    }

    /// Take a snapshot of the current kinematic state.
    #[allow(dead_code)]
    pub fn state(&self) -> RigidBodyState {
        RigidBodyState {
            position: self.position,
            velocity: self.velocity,
            orientation: self.orientation,
            angular_velocity: self.angular_velocity,
        }
    }
}

// ── Additional standalone functions ──────────────────────────────────────────

/// Construct a new rigid body from config and initial mass / inertia tensor diagonal.
#[allow(dead_code)]
pub fn new_rigid_body(mass: f32, inertia_diag: [f32; 3], cfg: &RigidBodyConfig) -> RigidBody {
    let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
    let inv_inertia = mat3_inv_diagonal(inertia_diag);
    RigidBody {
        position: [0.0; 3],
        velocity: [0.0; 3],
        orientation: [0.0, 0.0, 0.0, 1.0],
        angular_velocity: [0.0; 3],
        inv_mass,
        inv_inertia,
        restitution: cfg.restitution,
        friction: cfg.friction,
    }
}

/// Perform linear + angular integration (semi-implicit Euler) using a config.
#[allow(dead_code)]
pub fn integrate_rigid_body(rb: &mut RigidBody, dt: f32, cfg: &RigidBodyConfig) {
    rb.integrate(dt, cfg.gravity);
    // Apply damping
    let ld = (1.0 - cfg.linear_damping * dt).max(0.0);
    let ad = (1.0 - cfg.angular_damping * dt).max(0.0);
    rb.velocity[0] *= ld;
    rb.velocity[1] *= ld;
    rb.velocity[2] *= ld;
    rb.angular_velocity[0] *= ad;
    rb.angular_velocity[1] *= ad;
    rb.angular_velocity[2] *= ad;
}

/// Apply a central linear impulse (no torque).
#[allow(dead_code)]
pub fn apply_impulse(rb: &mut RigidBody, impulse: [f32; 3]) {
    if rb.inv_mass == 0.0 {
        return;
    }
    rb.velocity[0] += impulse[0] * rb.inv_mass;
    rb.velocity[1] += impulse[1] * rb.inv_mass;
    rb.velocity[2] += impulse[2] * rb.inv_mass;
}

/// Apply a pure angular (torque) impulse.
#[allow(dead_code)]
pub fn apply_torque_impulse(rb: &mut RigidBody, torque_impulse: [f32; 3]) {
    let delta = mat3_mul_vec3(rb.inv_inertia, torque_impulse);
    rb.angular_velocity[0] += delta[0];
    rb.angular_velocity[1] += delta[1];
    rb.angular_velocity[2] += delta[2];
}

/// Return the linear velocity of a rigid body.
#[allow(dead_code)]
pub fn rigid_body_velocity(rb: &RigidBody) -> [f32; 3] {
    rb.velocity
}

/// Return the angular velocity of a rigid body.
#[allow(dead_code)]
pub fn rigid_body_angular_velocity(rb: &RigidBody) -> [f32; 3] {
    rb.angular_velocity
}

/// Compute linear momentum p = m * v.
#[allow(dead_code)]
pub fn linear_momentum(rb: &RigidBody) -> [f32; 3] {
    if rb.inv_mass == 0.0 {
        return [0.0; 3];
    }
    let m = 1.0 / rb.inv_mass;
    [rb.velocity[0] * m, rb.velocity[1] * m, rb.velocity[2] * m]
}

/// Compute angular momentum L = I * ω  (diagonal inertia approximation).
#[allow(dead_code)]
pub fn angular_momentum_rb(rb: &RigidBody) -> [f32; 3] {
    // I = inv(inv_inertia); for diagonal: I_ii = 1 / inv_inertia[i][i]
    let ix = if rb.inv_inertia[0][0] > 0.0 {
        1.0 / rb.inv_inertia[0][0]
    } else {
        0.0
    };
    let iy = if rb.inv_inertia[1][1] > 0.0 {
        1.0 / rb.inv_inertia[1][1]
    } else {
        0.0
    };
    let iz = if rb.inv_inertia[2][2] > 0.0 {
        1.0 / rb.inv_inertia[2][2]
    } else {
        0.0
    };
    [
        ix * rb.angular_velocity[0],
        iy * rb.angular_velocity[1],
        iz * rb.angular_velocity[2],
    ]
}

/// Total kinetic energy (standalone version, delegates to method).
#[allow(dead_code)]
pub fn kinetic_energy_rb(rb: &RigidBody) -> f32 {
    rb.kinetic_energy()
}

/// Set the mass of a rigid body (recomputes `inv_mass`).
#[allow(dead_code)]
pub fn set_mass(rb: &mut RigidBody, mass: f32) {
    rb.inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
}

/// Set the inertia tensor diagonal of a rigid body (recomputes `inv_inertia`).
#[allow(dead_code)]
pub fn set_inertia_tensor(rb: &mut RigidBody, diag: [f32; 3]) {
    rb.inv_inertia = mat3_inv_diagonal(diag);
}

/// Reset a rigid body's kinematic state to rest at the origin.
#[allow(dead_code)]
pub fn reset_rigid_body(rb: &mut RigidBody) {
    rb.position = [0.0; 3];
    rb.velocity = [0.0; 3];
    rb.orientation = [0.0, 0.0, 0.0, 1.0];
    rb.angular_velocity = [0.0; 3];
}

/// Compute the axis-aligned bounding box of a rigid body approximated as a unit sphere.
///
/// Returns `(min, max)` in world space using the body position as the centre.
/// The radius is estimated as the reciprocal of the smallest `inv_inertia` diagonal
/// divided by the mass (very rough proxy for the body's extent).
#[allow(dead_code)]
pub fn rigid_body_aabb(rb: &RigidBody) -> ([f32; 3], [f32; 3]) {
    // Estimate a radius from inertia; fallback to 1.0 for static bodies.
    let r = if rb.inv_mass > 0.0 {
        let mass = 1.0 / rb.inv_mass;
        let min_inv_i = rb.inv_inertia[0][0]
            .min(rb.inv_inertia[1][1])
            .min(rb.inv_inertia[2][2]);
        if min_inv_i > 1e-12 {
            (1.0 / (min_inv_i * mass)).sqrt().max(0.01)
        } else {
            1.0
        }
    } else {
        1.0
    };
    let p = rb.position;
    (
        [p[0] - r, p[1] - r, p[2] - r],
        [p[0] + r, p[1] + r, p[2] + r],
    )
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPS
    }

    // 1. quat_from_axis_angle with zero-length axis → identity
    #[test]
    fn test_quat_from_axis_angle_zero_axis() {
        let q = quat_from_axis_angle([0.0, 0.0, 0.0], 1.0);
        assert!(approx_eq(q[3], 1.0));
        assert!(approx_eq(q[0], 0.0));
    }

    // 2. quat_from_axis_angle half-turn around Y axis
    #[test]
    fn test_quat_from_axis_angle_half_turn() {
        let q = quat_from_axis_angle([0.0, 1.0, 0.0], std::f32::consts::PI);
        // w ≈ cos(π/2) = 0
        assert!((q[3]).abs() < 1e-4);
        // y ≈ sin(π/2) = 1
        assert!((q[1] - 1.0).abs() < 1e-4);
    }

    // 3. quat_multiply: identity ⊗ identity = identity
    #[test]
    fn test_quat_multiply_identity() {
        let id = [0.0f32, 0.0, 0.0, 1.0];
        let res = quat_multiply(id, id);
        assert!(approx_eq(res[3], 1.0));
        assert!(approx_eq(res[0], 0.0));
    }

    // 4. quat_multiply: q ⊗ q⁻¹ ≈ identity (for unit q)
    #[test]
    fn test_quat_multiply_inverse() {
        let q = quat_from_axis_angle([1.0, 0.0, 0.0], 0.8);
        // Conjugate = inverse for unit quaternion
        let q_inv = [-q[0], -q[1], -q[2], q[3]];
        let res = quat_multiply(q, q_inv);
        assert!((res[3] - 1.0).abs() < 1e-5);
        assert!(res[0].abs() < 1e-5);
    }

    // 5. quat_normalize: output has unit length
    #[test]
    fn test_quat_normalize_unit_length() {
        let q = quat_normalize([3.0, 1.0, 2.0, 4.0]);
        let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        assert!((len - 1.0).abs() < EPS);
    }

    // 6. quat_normalize: near-zero → identity
    #[test]
    fn test_quat_normalize_near_zero() {
        let q = quat_normalize([0.0, 0.0, 0.0, 0.0]);
        assert!(approx_eq(q[3], 1.0));
    }

    // 7. mat3_sym_inertia_sphere formula check
    #[test]
    fn test_inertia_sphere_formula() {
        let m = 3.0f32;
        let r = 2.0f32;
        let i_mat = mat3_sym_inertia_sphere(m, r);
        let expected = 2.0 / 5.0 * m * r * r;
        assert!((i_mat[0][0] - expected).abs() < EPS);
        assert!((i_mat[1][1] - expected).abs() < EPS);
        assert!((i_mat[2][2] - expected).abs() < EPS);
        // Off-diagonal should be zero
        assert!(approx_eq(i_mat[0][1], 0.0));
    }

    // 8. mat3_sym_inertia_box diagonal values
    #[test]
    fn test_inertia_box_diagonal() {
        let m = 6.0f32;
        let ext = [2.0f32, 4.0, 6.0];
        let i_mat = mat3_sym_inertia_box(m, ext);
        let ix = m / 12.0 * (ext[1] * ext[1] + ext[2] * ext[2]);
        let iy = m / 12.0 * (ext[0] * ext[0] + ext[2] * ext[2]);
        let iz = m / 12.0 * (ext[0] * ext[0] + ext[1] * ext[1]);
        assert!((i_mat[0][0] - ix).abs() < EPS);
        assert!((i_mat[1][1] - iy).abs() < EPS);
        assert!((i_mat[2][2] - iz).abs() < EPS);
    }

    // 9. new_box inv_mass correctness
    #[test]
    fn test_new_box_inv_mass() {
        let rb = RigidBody::new_box([1.0, 1.0, 1.0], 4.0);
        assert!((rb.inv_mass - 0.25).abs() < EPS);
    }

    // 10. new_sphere inv_mass correctness
    #[test]
    fn test_new_sphere_inv_mass() {
        let rb = RigidBody::new_sphere(1.0, 5.0);
        assert!((rb.inv_mass - 0.2).abs() < EPS);
    }

    // 11. apply_force changes velocity
    #[test]
    fn test_apply_force_changes_velocity() {
        let mut rb = RigidBody::new_box([1.0; 3], 2.0); // inv_mass = 0.5
        rb.apply_force([10.0, 0.0, 0.0], 1.0); // Δv = 10 * 0.5 * 1 = 5
        assert!((rb.velocity[0] - 5.0).abs() < EPS);
    }

    // 12. apply_torque changes angular velocity
    #[test]
    fn test_apply_torque_changes_angular_velocity() {
        let mut rb = RigidBody::new_sphere(1.0, 1.0);
        let initial = rb.angular_velocity;
        rb.apply_torque([0.0, 5.0, 0.0], 1.0);
        assert!(rb.angular_velocity[1].abs() > initial[1].abs() + 1e-6);
    }

    // 13. integrate moves position
    #[test]
    fn test_integrate_position_moves() {
        let mut rb = RigidBody::new_sphere(1.0, 1.0);
        rb.velocity = [1.0, 0.0, 0.0];
        rb.integrate(0.1, [0.0, 0.0, 0.0]);
        assert!((rb.position[0] - 0.1).abs() < EPS);
    }

    // 14. kinetic_energy is positive after applying velocity
    #[test]
    fn test_kinetic_energy_positive_after_fall() {
        let mut rb = RigidBody::new_sphere(1.0, 2.0);
        rb.integrate(1.0, [0.0, -9.81, 0.0]);
        let ke = rb.kinetic_energy();
        assert!(ke > 0.0, "kinetic energy should be positive, got {ke}");
    }

    // 15. static body (inv_mass=0) unaffected by force
    #[test]
    fn test_static_body_unaffected_by_force() {
        let mut rb = RigidBody::new_box([1.0; 3], 0.0);
        rb.apply_force([1000.0, 1000.0, 1000.0], 1.0);
        assert!(approx_eq(rb.velocity[0], 0.0));
        rb.integrate(1.0, [0.0, -9.81, 0.0]);
        assert!(approx_eq(rb.position[1], 0.0));
    }

    // 16. integrate_orientation changes orientation
    #[test]
    fn test_integrate_orientation_changes() {
        let q0 = [0.0f32, 0.0, 0.0, 1.0];
        let omega = [0.0f32, 1.0, 0.0]; // spinning around Y
        let q1 = integrate_orientation(q0, omega, 0.1);
        // Should differ from identity
        let diff = (q1[0] - q0[0]).abs()
            + (q1[1] - q0[1]).abs()
            + (q1[2] - q0[2]).abs()
            + (q1[3] - q0[3]).abs();
        assert!(diff > 1e-4, "orientation should have changed");
    }

    // 17. apply_impulse_at_point: linear velocity changes
    #[test]
    fn test_apply_impulse_at_point_linear() {
        let mut rb = RigidBody::new_sphere(1.0, 2.0); // inv_mass = 0.5
        rb.apply_impulse_at_point([4.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert!((rb.velocity[0] - 2.0).abs() < EPS);
    }

    // 18. new_capsule has positive inv_mass
    #[test]
    fn test_new_capsule_inv_mass() {
        let rb = RigidBody::new_capsule(0.1, 0.5, 10.0);
        assert!((rb.inv_mass - 0.1).abs() < EPS);
    }

    // 19. default_rigid_body_config has correct gravity
    #[test]
    fn test_default_rigid_body_config_gravity() {
        let cfg = default_rigid_body_config();
        assert!((cfg.gravity[1] - (-9.81)).abs() < EPS);
    }

    // 20. new_rigid_body uses config restitution
    #[test]
    fn test_new_rigid_body_restitution() {
        let cfg = default_rigid_body_config();
        let rb = new_rigid_body(1.0, [1.0, 1.0, 1.0], &cfg);
        assert!((rb.restitution - cfg.restitution).abs() < EPS);
    }

    // 21. integrate_rigid_body applies gravity
    #[test]
    fn test_integrate_rigid_body_applies_gravity() {
        let cfg = default_rigid_body_config();
        let mut rb = new_rigid_body(1.0, [1.0, 1.0, 1.0], &cfg);
        integrate_rigid_body(&mut rb, 0.1, &cfg);
        // velocity[1] should be negative after one step with gravity
        assert!(rb.velocity[1] < 0.0);
    }

    // 22. apply_impulse changes velocity
    #[test]
    fn test_apply_impulse_changes_velocity() {
        let cfg = default_rigid_body_config();
        let mut rb = new_rigid_body(2.0, [1.0, 1.0, 1.0], &cfg); // inv_mass = 0.5
        apply_impulse(&mut rb, [4.0, 0.0, 0.0]);
        assert!((rb.velocity[0] - 2.0).abs() < EPS);
    }

    // 23. apply_torque_impulse changes angular velocity
    #[test]
    fn test_apply_torque_impulse_changes_angular_velocity() {
        let cfg = default_rigid_body_config();
        let mut rb = new_rigid_body(1.0, [1.0, 1.0, 1.0], &cfg);
        apply_torque_impulse(&mut rb, [0.0, 5.0, 0.0]);
        assert!(rb.angular_velocity[1].abs() > 1e-6);
    }

    // 24. rigid_body_velocity returns current velocity
    #[test]
    fn test_rigid_body_velocity_accessor() {
        let cfg = default_rigid_body_config();
        let mut rb = new_rigid_body(1.0, [1.0, 1.0, 1.0], &cfg);
        rb.velocity = [1.0, 2.0, 3.0];
        let v = rigid_body_velocity(&rb);
        assert!((v[0] - 1.0).abs() < EPS);
        assert!((v[1] - 2.0).abs() < EPS);
        assert!((v[2] - 3.0).abs() < EPS);
    }

    // 25. rigid_body_angular_velocity returns current angular velocity
    #[test]
    fn test_rigid_body_angular_velocity_accessor() {
        let cfg = default_rigid_body_config();
        let mut rb = new_rigid_body(1.0, [1.0, 1.0, 1.0], &cfg);
        rb.angular_velocity = [0.1, 0.2, 0.3];
        let av = rigid_body_angular_velocity(&rb);
        assert!((av[1] - 0.2).abs() < EPS);
    }

    // 26. linear_momentum is zero for static body
    #[test]
    fn test_linear_momentum_static_body() {
        let cfg = default_rigid_body_config();
        let rb = new_rigid_body(0.0, [1.0, 1.0, 1.0], &cfg);
        let p = linear_momentum(&rb);
        assert_eq!(p, [0.0; 3]);
    }

    // 27. linear_momentum for moving body is m*v
    #[test]
    fn test_linear_momentum_moving_body() {
        let cfg = default_rigid_body_config();
        let mut rb = new_rigid_body(2.0, [1.0, 1.0, 1.0], &cfg);
        rb.velocity = [3.0, 0.0, 0.0];
        let p = linear_momentum(&rb);
        assert!((p[0] - 6.0).abs() < EPS);
    }

    // 28. angular_momentum_rb non-zero after torque impulse
    #[test]
    fn test_angular_momentum_non_zero() {
        let cfg = default_rigid_body_config();
        let mut rb = new_rigid_body(1.0, [2.0, 2.0, 2.0], &cfg);
        apply_torque_impulse(&mut rb, [0.0, 4.0, 0.0]);
        let l = angular_momentum_rb(&rb);
        assert!(l[1].abs() > 1e-6);
    }

    // 29. kinetic_energy_rb is positive after motion
    #[test]
    fn test_kinetic_energy_rb_positive() {
        let cfg = default_rigid_body_config();
        let mut rb = new_rigid_body(1.0, [1.0, 1.0, 1.0], &cfg);
        rb.velocity = [1.0, 0.0, 0.0];
        assert!(kinetic_energy_rb(&rb) > 0.0);
    }

    // 30. set_mass updates inv_mass correctly
    #[test]
    fn test_set_mass_updates_inv_mass() {
        let cfg = default_rigid_body_config();
        let mut rb = new_rigid_body(1.0, [1.0, 1.0, 1.0], &cfg);
        set_mass(&mut rb, 4.0);
        assert!((rb.inv_mass - 0.25).abs() < EPS);
    }

    // 31. set_inertia_tensor updates inv_inertia
    #[test]
    fn test_set_inertia_tensor_updates() {
        let cfg = default_rigid_body_config();
        let mut rb = new_rigid_body(1.0, [1.0, 1.0, 1.0], &cfg);
        set_inertia_tensor(&mut rb, [2.0, 4.0, 8.0]);
        assert!((rb.inv_inertia[0][0] - 0.5).abs() < EPS);
        assert!((rb.inv_inertia[1][1] - 0.25).abs() < EPS);
        assert!((rb.inv_inertia[2][2] - 0.125).abs() < EPS);
    }

    // 32. reset_rigid_body zeros state
    #[test]
    fn test_reset_rigid_body_zeros_state() {
        let cfg = default_rigid_body_config();
        let mut rb = new_rigid_body(1.0, [1.0, 1.0, 1.0], &cfg);
        rb.velocity = [10.0, 10.0, 10.0];
        rb.position = [5.0, 5.0, 5.0];
        reset_rigid_body(&mut rb);
        assert_eq!(rb.position, [0.0; 3]);
        assert_eq!(rb.velocity, [0.0; 3]);
    }

    // 33. rigid_body_aabb min < max
    #[test]
    fn test_rigid_body_aabb_min_lt_max() {
        let cfg = default_rigid_body_config();
        let rb = new_rigid_body(1.0, [1.0, 1.0, 1.0], &cfg);
        let (mn, mx) = rigid_body_aabb(&rb);
        for i in 0..3 {
            assert!(mn[i] < mx[i], "AABB min[{i}] not < max[{i}]");
        }
    }
}
