//! Rigid body with full 3×3 inertia tensor and angular momentum integration.
//!
//! Unlike the simpler `angular_velocity` module (which uses a scalar moment of
//! inertia), this module tracks the full 3×3 inertia tensor I and integrates
//! angular momentum L directly:
//!
//!   dL/dt = τ  (torque)
//!   ω = I⁻¹ L
//!
//! The rotation is stored as a 4-component unit quaternion [x, y, z, w].

#![allow(dead_code)]

/// Configuration for the angular-momentum rigid body.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AngMomConfig {
    /// Linear damping applied to angular momentum each step.
    pub angular_damping: f32,
}

/// A symmetric 3×3 inertia tensor stored as its 6 independent components.
///
/// Layout: `[Ixx, Iyy, Izz, Ixy, Ixz, Iyz]`.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct InertiaTensor {
    /// Diagonal and off-diagonal components.
    pub components: [f32; 6],
}

/// A rigid body with angular momentum.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AngMomBody {
    /// Inertia tensor in body space.
    pub inertia: InertiaTensor,
    /// Angular momentum vector L (world space).
    pub angular_momentum: [f32; 3],
    /// Orientation quaternion [x, y, z, w].
    pub orientation: [f32; 4],
    /// Accumulated torque for the current step.
    pub torque: [f32; 3],
    /// Angular damping factor.
    pub damping: f32,
}

/// Return sensible defaults for [`AngMomConfig`].
#[allow(dead_code)]
pub fn default_ang_mom_config() -> AngMomConfig {
    AngMomConfig { angular_damping: 0.01 }
}

/// Create a new body with a uniform sphere inertia tensor (Ixx = Iyy = Izz = 1).
#[allow(dead_code)]
pub fn new_ang_mom_body(config: &AngMomConfig) -> AngMomBody {
    AngMomBody {
        inertia: InertiaTensor { components: [1.0, 1.0, 1.0, 0.0, 0.0, 0.0] },
        angular_momentum: [0.0; 3],
        orientation: [0.0, 0.0, 0.0, 1.0], // identity quaternion
        torque: [0.0; 3],
        damping: config.angular_damping,
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Invert a 3×3 symmetric matrix given as [a, e, i, b, c, f] (Ixx,Iyy,Izz,Ixy,Ixz,Iyz).
fn invert_inertia(t: &InertiaTensor) -> [f32; 9] {
    let [ixx, iyy, izz, ixy, ixz, iyz] = t.components;
    // Build full matrix
    let m = [
        ixx, ixy, ixz,
        ixy, iyy, iyz,
        ixz, iyz, izz,
    ];
    // Cofactor matrix
    let c = [
        m[4] * m[8] - m[5] * m[7],
        -(m[3] * m[8] - m[5] * m[6]),
        m[3] * m[7] - m[4] * m[6],
        -(m[1] * m[8] - m[2] * m[7]),
        m[0] * m[8] - m[2] * m[6],
        -(m[0] * m[7] - m[1] * m[6]),
        m[1] * m[5] - m[2] * m[4],
        -(m[0] * m[5] - m[2] * m[3]),
        m[0] * m[4] - m[1] * m[3],
    ];
    let det = m[0] * c[0] + m[1] * c[1] + m[2] * c[2];
    if det.abs() < 1e-12 {
        // Singular: return identity
        return [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    }
    let inv_det = 1.0 / det;
    // Transpose of cofactor / det
    [
        c[0] * inv_det, c[3] * inv_det, c[6] * inv_det,
        c[1] * inv_det, c[4] * inv_det, c[7] * inv_det,
        c[2] * inv_det, c[5] * inv_det, c[8] * inv_det,
    ]
}

fn mat3_mul_vec3(m: &[f32; 9], v: [f32; 3]) -> [f32; 3] {
    [
        m[0] * v[0] + m[1] * v[1] + m[2] * v[2],
        m[3] * v[0] + m[4] * v[1] + m[5] * v[2],
        m[6] * v[0] + m[7] * v[1] + m[8] * v[2],
    ]
}

fn quat_mul(q: [f32; 4], p: [f32; 4]) -> [f32; 4] {
    let [qx, qy, qz, qw] = q;
    let [px, py, pz, pw] = p;
    [
        qw * px + qx * pw + qy * pz - qz * py,
        qw * py - qx * pz + qy * pw + qz * px,
        qw * pz + qx * py - qy * px + qz * pw,
        qw * pw - qx * px - qy * py - qz * pz,
    ]
}

fn quat_normalize(q: [f32; 4]) -> [f32; 4] {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len < 1e-9 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [q[0] / len, q[1] / len, q[2] / len, q[3] / len]
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Advance the body by one time step `dt` (seconds).
///
/// Applies accumulated torque, integrates angular momentum, updates the
/// orientation quaternion, then clears the torque.
#[allow(dead_code)]
pub fn ang_mom_step(body: &mut AngMomBody, dt: f32) {
    // L += τ·dt
    body.angular_momentum[0] += body.torque[0] * dt;
    body.angular_momentum[1] += body.torque[1] * dt;
    body.angular_momentum[2] += body.torque[2] * dt;
    // Apply damping
    let damp = (1.0 - body.damping * dt).max(0.0);
    body.angular_momentum[0] *= damp;
    body.angular_momentum[1] *= damp;
    body.angular_momentum[2] *= damp;
    // ω = I⁻¹ L
    let omega = ang_mom_angular_velocity(body);

    // Integrate orientation: dq/dt = 0.5 * [ω, 0] * q
    let omega_quat = [omega[0] * 0.5, omega[1] * 0.5, omega[2] * 0.5, 0.0];
    let dq = quat_mul(omega_quat, body.orientation);
    body.orientation[0] += dq[0] * dt;
    body.orientation[1] += dq[1] * dt;
    body.orientation[2] += dq[2] * dt;
    body.orientation[3] += dq[3] * dt;
    body.orientation = quat_normalize(body.orientation);

    // Clear torque
    body.torque = [0.0; 3];
}

/// Apply an angular impulse (changes angular momentum immediately).
#[allow(dead_code)]
pub fn ang_mom_apply_impulse(body: &mut AngMomBody, impulse: [f32; 3]) {
    body.angular_momentum[0] += impulse[0];
    body.angular_momentum[1] += impulse[1];
    body.angular_momentum[2] += impulse[2];
}

/// Compute the current angular velocity ω = I⁻¹ · L.
#[allow(dead_code)]
pub fn ang_mom_angular_velocity(body: &AngMomBody) -> [f32; 3] {
    let inv = invert_inertia(&body.inertia);
    mat3_mul_vec3(&inv, body.angular_momentum)
}

/// Compute the rotational kinetic energy ½ ω·(I·ω) = ½ L·ω.
#[allow(dead_code)]
pub fn ang_mom_kinetic_energy(body: &AngMomBody) -> f32 {
    let omega = ang_mom_angular_velocity(body);
    0.5 * (body.angular_momentum[0] * omega[0]
        + body.angular_momentum[1] * omega[1]
        + body.angular_momentum[2] * omega[2])
}

/// Serialise the body state to compact JSON.
#[allow(dead_code)]
pub fn ang_mom_to_json(body: &AngMomBody) -> String {
    let l = body.angular_momentum;
    let q = body.orientation;
    let omega = ang_mom_angular_velocity(body);
    format!(
        r#"{{"L":[{:.4},{:.4},{:.4}],"omega":[{:.4},{:.4},{:.4}],"q":[{:.4},{:.4},{:.4},{:.4}]}}"#,
        l[0], l[1], l[2],
        omega[0], omega[1], omega[2],
        q[0], q[1], q[2], q[3],
    )
}

/// Reset angular momentum to zero and orientation to identity.
#[allow(dead_code)]
pub fn ang_mom_reset(body: &mut AngMomBody) {
    body.angular_momentum = [0.0; 3];
    body.orientation = [0.0, 0.0, 0.0, 1.0];
    body.torque = [0.0; 3];
}

/// Replace the inertia tensor.
#[allow(dead_code)]
pub fn ang_mom_set_inertia(body: &mut AngMomBody, inertia: InertiaTensor) {
    body.inertia = inertia;
}

/// Accumulate a torque that will be applied on the next `ang_mom_step`.
#[allow(dead_code)]
pub fn ang_mom_torque(body: &mut AngMomBody, torque: [f32; 3]) {
    body.torque[0] += torque[0];
    body.torque[1] += torque[1];
    body.torque[2] += torque[2];
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_body() -> AngMomBody {
        let cfg = default_ang_mom_config();
        new_ang_mom_body(&cfg)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_ang_mom_config();
        assert!(cfg.angular_damping > 0.0);
    }

    #[test]
    fn test_new_body_identity_orientation() {
        let body = unit_body();
        assert_eq!(body.orientation, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_apply_impulse() {
        let mut body = unit_body();
        ang_mom_apply_impulse(&mut body, [1.0, 0.0, 0.0]);
        assert_eq!(body.angular_momentum[0], 1.0);
    }

    #[test]
    fn test_angular_velocity_from_identity_inertia() {
        let mut body = unit_body();
        ang_mom_apply_impulse(&mut body, [2.0, 0.0, 0.0]);
        let omega = ang_mom_angular_velocity(&body);
        assert!((omega[0] - 2.0).abs() < 1e-5, "omega_x={}", omega[0]);
    }

    #[test]
    fn test_kinetic_energy_positive() {
        let mut body = unit_body();
        ang_mom_apply_impulse(&mut body, [0.0, 1.0, 0.0]);
        let ke = ang_mom_kinetic_energy(&body);
        assert!(ke > 0.0);
    }

    #[test]
    fn test_step_changes_orientation() {
        let mut body = unit_body();
        ang_mom_apply_impulse(&mut body, [1.0, 0.0, 0.0]);
        let q0 = body.orientation;
        ang_mom_step(&mut body, 0.01);
        let q1 = body.orientation;
        let changed = (0..4).any(|i| (q0[i] - q1[i]).abs() > 1e-6);
        assert!(changed, "orientation should change after step");
    }

    #[test]
    fn test_orientation_stays_unit_length() {
        let mut body = unit_body();
        ang_mom_apply_impulse(&mut body, [5.0, 3.0, 1.0]);
        for _ in 0..100 {
            ang_mom_step(&mut body, 0.01);
        }
        let q = body.orientation;
        let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        assert!((len - 1.0).abs() < 1e-4, "quat length={}", len);
    }

    #[test]
    fn test_to_json_contains_l() {
        let body = unit_body();
        let json = ang_mom_to_json(&body);
        assert!(json.contains('"'));
        assert!(json.contains('L'));
    }

    #[test]
    fn test_reset_clears_state() {
        let mut body = unit_body();
        ang_mom_apply_impulse(&mut body, [1.0, 2.0, 3.0]);
        ang_mom_reset(&mut body);
        assert_eq!(body.angular_momentum, [0.0; 3]);
        assert_eq!(body.orientation, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_set_inertia() {
        let mut body = unit_body();
        let new_i = InertiaTensor { components: [2.0, 2.0, 2.0, 0.0, 0.0, 0.0] };
        ang_mom_set_inertia(&mut body, new_i);
        ang_mom_apply_impulse(&mut body, [2.0, 0.0, 0.0]);
        let omega = ang_mom_angular_velocity(&body);
        assert!((omega[0] - 1.0).abs() < 1e-5, "omega_x={}", omega[0]);
    }

    #[test]
    fn test_torque_accumulates() {
        let mut body = unit_body();
        ang_mom_torque(&mut body, [1.0, 0.0, 0.0]);
        ang_mom_torque(&mut body, [0.0, 1.0, 0.0]);
        assert_eq!(body.torque, [1.0, 1.0, 0.0]);
    }
}
