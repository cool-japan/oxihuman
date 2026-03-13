// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Gyroscopic forces, angular momentum, and precession.
//!
//! Models rigid-body rotational dynamics including gyroscopic torques,
//! precession, nutation, and damped angular-velocity evolution.

// ── math helpers ─────────────────────────────────────────────────────────────

#[inline]
fn v3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn v3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn v3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn v3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn v3_len(v: [f32; 3]) -> f32 {
    v3_dot(v, v).sqrt()
}

#[inline]
fn v3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Multiply a 3x3 diagonal inertia tensor (stored as [Ixx, Iyy, Izz])
/// by a vector.
#[inline]
fn diag_mul(inertia: [f32; 3], v: [f32; 3]) -> [f32; 3] {
    [inertia[0] * v[0], inertia[1] * v[1], inertia[2] * v[2]]
}

/// Component-wise division by diagonal inertia.
#[inline]
fn diag_inv_mul(inertia: [f32; 3], v: [f32; 3]) -> [f32; 3] {
    [
        if inertia[0].abs() > 1e-12 {
            v[0] / inertia[0]
        } else {
            0.0
        },
        if inertia[1].abs() > 1e-12 {
            v[1] / inertia[1]
        } else {
            0.0
        },
        if inertia[2].abs() > 1e-12 {
            v[2] / inertia[2]
        } else {
            0.0
        },
    ]
}

// ── public types ─────────────────────────────────────────────────────────────

/// A rigid body with gyroscopic properties.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GyroBody {
    /// Principal moments of inertia \[Ixx, Iyy, Izz\].
    pub inertia: [f32; 3],
    /// Angular velocity in body frame (rad/s).
    pub angular_velocity: [f32; 3],
    /// External torque applied this frame (body frame).
    pub external_torque: [f32; 3],
}

/// Configuration for gyroscope simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GyroConfig {
    /// Integration time step.
    pub dt: f32,
    /// Angular velocity damping factor per step (0 = no damping, 1 = full stop).
    pub damping: f32,
    /// Minimum angular speed below which the body is considered stopped.
    pub min_speed: f32,
}

/// Accumulated state of a gyroscopic body over time.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GyroState {
    /// Current angular velocity.
    pub angular_velocity: [f32; 3],
    /// Current angular momentum (L = I * omega).
    pub angular_momentum: [f32; 3],
    /// Total elapsed time.
    pub time: f32,
    /// Number of integration steps taken.
    pub steps: usize,
}

// ── public functions ─────────────────────────────────────────────────────────

/// Create a default gyroscope configuration.
#[allow(dead_code)]
pub fn default_gyro_config() -> GyroConfig {
    GyroConfig {
        dt: 1.0 / 60.0,
        damping: 0.01,
        min_speed: 1e-6,
    }
}

/// Create a new gyro body with the given principal moments of inertia.
#[allow(dead_code)]
pub fn new_gyro_body(inertia: [f32; 3]) -> GyroBody {
    GyroBody {
        inertia,
        angular_velocity: [0.0; 3],
        external_torque: [0.0; 3],
    }
}

/// Compute angular momentum: L = I * omega (diagonal inertia tensor).
#[allow(dead_code)]
pub fn angular_momentum(body: &GyroBody) -> [f32; 3] {
    diag_mul(body.inertia, body.angular_velocity)
}

/// Compute gyroscopic torque: tau_gyro = omega x L.
///
/// This torque arises from the cross product of the angular velocity with
/// the angular momentum in the body frame.  For a symmetric body it
/// vanishes; for asymmetric bodies it causes precession/nutation.
#[allow(dead_code)]
pub fn gyroscopic_torque(body: &GyroBody) -> [f32; 3] {
    let l = angular_momentum(body);
    v3_cross(body.angular_velocity, l)
}

/// Apply gyroscopic torque to the body, updating angular velocity.
///
/// This accounts for the gyroscopic term plus any external torque:
///   I * d(omega)/dt = tau_ext - omega x (I * omega)
///   => d(omega)/dt = I^{-1} * (tau_ext - omega x L)
#[allow(dead_code)]
pub fn apply_gyro_torque(body: &mut GyroBody, dt: f32) {
    let l = angular_momentum(body);
    let gyro = v3_cross(body.angular_velocity, l);
    let net_torque = v3_sub(body.external_torque, gyro);
    let alpha = diag_inv_mul(body.inertia, net_torque);
    body.angular_velocity = v3_add(body.angular_velocity, v3_scale(alpha, dt));
}

/// Integrate angular velocity for one time step.
///
/// Returns the updated `GyroState`.
#[allow(dead_code)]
pub fn update_gyro_state(body: &mut GyroBody, state: &mut GyroState, config: &GyroConfig) {
    apply_gyro_torque(body, config.dt);

    // Apply damping.
    let damp = 1.0 - config.damping;
    body.angular_velocity = v3_scale(body.angular_velocity, damp);

    // Clamp to zero if below threshold.
    let speed = v3_len(body.angular_velocity);
    if speed < config.min_speed {
        body.angular_velocity = [0.0; 3];
    }

    state.angular_velocity = body.angular_velocity;
    state.angular_momentum = angular_momentum(body);
    state.time += config.dt;
    state.steps += 1;
}

/// Compute precession rate for a symmetric top under gravity.
///
/// omega_p = (m * g * d) / (I_spin * omega_spin)
///
/// where `mgl` = m*g*d (gravitational torque magnitude) and `spin_axis`
/// is the index of the spin axis (0=x, 1=y, 2=z).
#[allow(dead_code)]
pub fn precession_rate(body: &GyroBody, mgl: f32, spin_axis: usize) -> f32 {
    let i_spin = body.inertia[spin_axis.min(2)];
    let omega_spin = body.angular_velocity[spin_axis.min(2)];
    let denom = i_spin * omega_spin;
    if denom.abs() < 1e-12 {
        return 0.0;
    }
    mgl / denom
}

/// Compute nutation angle (the wobble angle between the spin axis and
/// the angular momentum vector).
///
/// theta = acos( (L . spin_axis_dir) / |L| )
#[allow(dead_code)]
pub fn nutation_angle(body: &GyroBody, spin_axis: usize) -> f32 {
    let l = angular_momentum(body);
    let l_mag = v3_len(l);
    if l_mag < 1e-12 {
        return 0.0;
    }
    let mut axis = [0.0f32; 3];
    axis[spin_axis.min(2)] = 1.0;
    let cos_theta = (v3_dot(l, axis) / l_mag).clamp(-1.0, 1.0);
    cos_theta.acos()
}

/// Compute rotational kinetic energy: E = 0.5 * omega . L = 0.5 * I * omega^2.
#[allow(dead_code)]
pub fn spin_energy(body: &GyroBody) -> f32 {
    let l = angular_momentum(body);
    0.5 * v3_dot(body.angular_velocity, l)
}

/// Set the angular velocity of the body.
#[allow(dead_code)]
pub fn set_angular_velocity(body: &mut GyroBody, omega: [f32; 3]) {
    body.angular_velocity = omega;
}

/// Set the inertia tensor (diagonal).
#[allow(dead_code)]
pub fn set_inertia_tensor(body: &mut GyroBody, inertia: [f32; 3]) {
    body.inertia = inertia;
}

/// Compute a stability metric based on the ratio of spin energy to
/// perturbation energy.
///
/// A higher value indicates greater gyroscopic stability.
/// Returns |L| / sqrt(Ixx * Iyy + Iyy * Izz + Izz * Ixx).
#[allow(dead_code)]
pub fn gyro_stability_metric(body: &GyroBody) -> f32 {
    let l = angular_momentum(body);
    let l_mag = v3_len(l);
    let i = body.inertia;
    let cross_sum = i[0] * i[1] + i[1] * i[2] + i[2] * i[0];
    if cross_sum < 1e-12 {
        return 0.0;
    }
    l_mag / cross_sum.sqrt()
}

/// Perform a damped gyroscope update: integrate with damping and reset
/// the external torque to zero.
#[allow(dead_code)]
pub fn damped_gyro_update(body: &mut GyroBody, state: &mut GyroState, config: &GyroConfig) {
    update_gyro_state(body, state, config);
    body.external_torque = [0.0; 3];
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform_body() -> GyroBody {
        let mut b = new_gyro_body([1.0, 1.0, 1.0]);
        b.angular_velocity = [0.0, 0.0, 10.0];
        b
    }

    fn asymmetric_body() -> GyroBody {
        let mut b = new_gyro_body([1.0, 2.0, 3.0]);
        b.angular_velocity = [1.0, 0.5, 10.0];
        b
    }

    fn new_state() -> GyroState {
        GyroState {
            angular_velocity: [0.0; 3],
            angular_momentum: [0.0; 3],
            time: 0.0,
            steps: 0,
        }
    }

    #[test]
    fn test_default_config() {
        let cfg = default_gyro_config();
        assert!((cfg.dt - 1.0 / 60.0).abs() < 1e-6);
        assert!((cfg.damping - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_new_gyro_body() {
        let b = new_gyro_body([1.0, 2.0, 3.0]);
        assert_eq!(b.inertia, [1.0, 2.0, 3.0]);
        assert_eq!(b.angular_velocity, [0.0; 3]);
    }

    #[test]
    fn test_angular_momentum_uniform() {
        let b = uniform_body();
        let l = angular_momentum(&b);
        assert!((l[2] - 10.0).abs() < 1e-6);
        assert!((l[0]).abs() < 1e-6);
    }

    #[test]
    fn test_angular_momentum_asymmetric() {
        let b = asymmetric_body();
        let l = angular_momentum(&b);
        assert!((l[0] - 1.0).abs() < 1e-6); // I_x * omega_x = 1*1
        assert!((l[1] - 1.0).abs() < 1e-6); // I_y * omega_y = 2*0.5
        assert!((l[2] - 30.0).abs() < 1e-6); // I_z * omega_z = 3*10
    }

    #[test]
    fn test_gyroscopic_torque_symmetric() {
        let b = uniform_body();
        let tau = gyroscopic_torque(&b);
        // For symmetric body spinning about principal axis, gyro torque = 0.
        assert!((tau[0]).abs() < 1e-6);
        assert!((tau[1]).abs() < 1e-6);
        assert!((tau[2]).abs() < 1e-6);
    }

    #[test]
    fn test_gyroscopic_torque_asymmetric() {
        let b = asymmetric_body();
        let tau = gyroscopic_torque(&b);
        // Non-zero for asymmetric body with off-axis spin.
        let mag = v3_len(tau);
        assert!(mag > 0.1);
    }

    #[test]
    fn test_apply_gyro_torque() {
        let mut b = asymmetric_body();
        let omega_before = b.angular_velocity;
        apply_gyro_torque(&mut b, 1.0 / 60.0);
        // Angular velocity should have changed.
        let changed = (b.angular_velocity[0] - omega_before[0]).abs() > 1e-9
            || (b.angular_velocity[1] - omega_before[1]).abs() > 1e-9;
        assert!(changed);
    }

    #[test]
    fn test_update_gyro_state() {
        let mut b = uniform_body();
        let mut state = new_state();
        let cfg = default_gyro_config();
        update_gyro_state(&mut b, &mut state, &cfg);
        assert_eq!(state.steps, 1);
        assert!(state.time > 0.0);
        // Damping should reduce angular velocity slightly.
        assert!(v3_len(state.angular_velocity) < 10.0 + 1e-6);
    }

    #[test]
    fn test_precession_rate() {
        let b = uniform_body();
        // mgl = 1.0, spin axis = z (index 2)
        let wp = precession_rate(&b, 1.0, 2);
        // omega_p = mgl / (I_z * omega_z) = 1 / (1 * 10) = 0.1
        assert!((wp - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_precession_rate_zero_spin() {
        let b = new_gyro_body([1.0, 1.0, 1.0]);
        let wp = precession_rate(&b, 1.0, 2);
        assert_eq!(wp, 0.0);
    }

    #[test]
    fn test_nutation_angle_aligned() {
        let b = uniform_body();
        // Spinning purely about z: nutation = 0.
        let theta = nutation_angle(&b, 2);
        assert!(theta.abs() < 1e-5);
    }

    #[test]
    fn test_nutation_angle_tilted() {
        let b = asymmetric_body();
        let theta = nutation_angle(&b, 2);
        // L has components in x and y, so nutation > 0.
        assert!(theta > 0.0);
    }

    #[test]
    fn test_spin_energy() {
        let b = uniform_body();
        let e = spin_energy(&b);
        // E = 0.5 * I * omega^2 = 0.5 * 1 * 100 = 50
        assert!((e - 50.0).abs() < 1e-4);
    }

    #[test]
    fn test_set_angular_velocity() {
        let mut b = new_gyro_body([1.0, 1.0, 1.0]);
        set_angular_velocity(&mut b, [5.0, 0.0, 0.0]);
        assert_eq!(b.angular_velocity, [5.0, 0.0, 0.0]);
    }

    #[test]
    fn test_set_inertia_tensor() {
        let mut b = new_gyro_body([1.0, 1.0, 1.0]);
        set_inertia_tensor(&mut b, [2.0, 3.0, 4.0]);
        assert_eq!(b.inertia, [2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_gyro_stability_metric() {
        let b = uniform_body();
        let s = gyro_stability_metric(&b);
        assert!(s > 0.0);
    }

    #[test]
    fn test_gyro_stability_metric_zero() {
        let b = new_gyro_body([0.0, 0.0, 0.0]);
        let s = gyro_stability_metric(&b);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn test_damped_gyro_update() {
        let mut b = uniform_body();
        b.external_torque = [1.0, 0.0, 0.0];
        let mut state = new_state();
        let cfg = default_gyro_config();
        damped_gyro_update(&mut b, &mut state, &cfg);
        // External torque should be reset.
        assert_eq!(b.external_torque, [0.0; 3]);
        assert_eq!(state.steps, 1);
    }
}
