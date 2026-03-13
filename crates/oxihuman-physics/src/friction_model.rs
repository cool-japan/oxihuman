// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Coulomb friction model for contact physics.
//!
//! Implements static friction, kinetic friction, rolling resistance, the
//! Stribeck smooth-transition model, and impulse-based friction application.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Which friction regime is currently active.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrictionMode {
    Static,
    Kinetic,
    Rolling,
}

/// Global friction configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FrictionConfig {
    /// Static friction coefficient (μs).
    pub static_coefficient: f32,
    /// Kinetic friction coefficient (μk; must be ≤ μs).
    pub kinetic_coefficient: f32,
    /// Rolling friction coefficient (μr; much smaller than μk).
    pub rolling_coefficient: f32,
    /// Velocity below which the object is considered at rest for Stribeck.
    pub stribeck_velocity: f32,
}

/// Contact friction state for a single contact pair.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactFriction {
    /// Normal force magnitude at the contact (N).
    pub normal_force: f32,
    /// Relative tangential velocity magnitude at the contact (m/s).
    pub relative_velocity: f32,
    /// Current friction mode.
    pub mode: FrictionMode,
    /// Static friction coefficient for this contact.
    pub mu_static: f32,
    /// Kinetic friction coefficient for this contact.
    pub mu_kinetic: f32,
    /// Rolling friction coefficient for this contact.
    pub mu_rolling: f32,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// Result of applying a friction impulse: (tangential impulse magnitude, updated velocity).
pub type FrictionImpulseResult = (f32, [f32; 3]);

// ── Public functions ──────────────────────────────────────────────────────────

/// Return a sensible default [`FrictionConfig`].
#[allow(dead_code)]
pub fn default_friction_config() -> FrictionConfig {
    FrictionConfig {
        static_coefficient: 0.6,
        kinetic_coefficient: 0.4,
        rolling_coefficient: 0.01,
        stribeck_velocity: 0.01,
    }
}

/// Construct a new [`ContactFriction`] from a config and initial contact state.
#[allow(dead_code)]
pub fn new_contact_friction(
    cfg: &FrictionConfig,
    normal_force: f32,
    relative_velocity: f32,
) -> ContactFriction {
    let mode = if relative_velocity.abs() < 1e-6 {
        FrictionMode::Static
    } else {
        FrictionMode::Kinetic
    };
    ContactFriction {
        normal_force,
        relative_velocity,
        mode,
        mu_static: cfg.static_coefficient,
        mu_kinetic: cfg.kinetic_coefficient,
        mu_rolling: cfg.rolling_coefficient,
    }
}

/// Compute the maximum static friction force: f_s = μs · N.
#[allow(dead_code)]
pub fn static_friction_force(cf: &ContactFriction) -> f32 {
    cf.mu_static * cf.normal_force.max(0.0)
}

/// Compute the kinetic friction force magnitude: f_k = μk · N.
#[allow(dead_code)]
pub fn kinetic_friction_force(cf: &ContactFriction) -> f32 {
    cf.mu_kinetic * cf.normal_force.max(0.0)
}

/// Compute the rolling friction torque magnitude: τ_r = μr · N · r.
///
/// `radius` is the effective rolling radius of the contact body.
#[allow(dead_code)]
pub fn rolling_friction_torque(cf: &ContactFriction, radius: f32) -> f32 {
    cf.mu_rolling * cf.normal_force.max(0.0) * radius.max(0.0)
}

/// Return the effective friction coefficient for the current mode.
#[allow(dead_code)]
pub fn friction_coefficient(cf: &ContactFriction) -> f32 {
    match cf.mode {
        FrictionMode::Static => cf.mu_static,
        FrictionMode::Kinetic => cf.mu_kinetic,
        FrictionMode::Rolling => cf.mu_rolling,
    }
}

/// Return `true` if the applied tangential force exceeds the static friction limit.
///
/// `applied_tangential` is the magnitude of the applied tangential force.
#[allow(dead_code)]
pub fn is_sliding(cf: &ContactFriction, applied_tangential: f32) -> bool {
    applied_tangential > static_friction_force(cf)
}

/// Apply a friction impulse to a velocity, opposing the direction of motion.
///
/// - `velocity` is the current 3-D velocity of the object.
/// - `normal` is the contact normal (unit vector pointing away from the surface).
/// - `inv_mass` is the inverse mass of the object.
/// - Returns (impulse magnitude, new velocity).
#[allow(dead_code)]
pub fn apply_friction_impulse(
    cf: &ContactFriction,
    velocity: [f32; 3],
    normal: [f32; 3],
    inv_mass: f32,
) -> FrictionImpulseResult {
    if inv_mass == 0.0 {
        return (0.0, velocity);
    }
    // Compute tangential velocity (remove normal component)
    let vdotn = velocity[0] * normal[0] + velocity[1] * normal[1] + velocity[2] * normal[2];
    let vt = [
        velocity[0] - vdotn * normal[0],
        velocity[1] - vdotn * normal[1],
        velocity[2] - vdotn * normal[2],
    ];
    let vt_len = (vt[0] * vt[0] + vt[1] * vt[1] + vt[2] * vt[2]).sqrt();
    if vt_len < 1e-12 {
        return (0.0, velocity);
    }
    let friction_force = kinetic_friction_force(cf);
    // Maximum impulse magnitude to remove tangential velocity
    let max_impulse = vt_len / inv_mass;
    let impulse_mag = friction_force.min(max_impulse / inv_mass).min(vt_len / inv_mass);
    let dir = [vt[0] / vt_len, vt[1] / vt_len, vt[2] / vt_len];
    let new_vel = [
        velocity[0] - dir[0] * impulse_mag * inv_mass,
        velocity[1] - dir[1] * impulse_mag * inv_mass,
        velocity[2] - dir[2] * impulse_mag * inv_mass,
    ];
    (impulse_mag, new_vel)
}

/// Estimate energy dissipated by friction over time step `dt`.
///
/// E = f_k · |v_tangential| · dt
#[allow(dead_code)]
pub fn friction_energy_dissipation(cf: &ContactFriction, dt: f32) -> f32 {
    kinetic_friction_force(cf) * cf.relative_velocity.abs() * dt.max(0.0)
}

/// Set the static friction coefficient on a contact and update its mode.
#[allow(dead_code)]
pub fn set_static_coefficient(cf: &mut ContactFriction, mu_s: f32) {
    cf.mu_static = mu_s.max(0.0);
}

/// Set the kinetic friction coefficient on a contact.
#[allow(dead_code)]
pub fn set_kinetic_coefficient(cf: &mut ContactFriction, mu_k: f32) {
    cf.mu_kinetic = mu_k.max(0.0);
}

/// Return the current friction mode.
#[allow(dead_code)]
pub fn friction_mode(cf: &ContactFriction) -> FrictionMode {
    cf.mode
}

/// Stribeck model: smooth transition between static and kinetic friction.
///
/// Returns the friction coefficient blended by a Gaussian in velocity space:
/// μ(v) = μk + (μs − μk) · exp(−(v / v_s)²)
///
/// where `v_s` is the Stribeck velocity.
#[allow(dead_code)]
pub fn stribeck_friction(cfg: &FrictionConfig, relative_velocity: f32) -> f32 {
    let v = relative_velocity.abs();
    let vs = cfg.stribeck_velocity.max(1e-12);
    let blend = (-(v / vs).powi(2)).exp();
    cfg.kinetic_coefficient + (cfg.static_coefficient - cfg.kinetic_coefficient) * blend
}

/// Return the normal force at the contact.
#[allow(dead_code)]
pub fn friction_normal_force(cf: &ContactFriction) -> f32 {
    cf.normal_force
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    fn default_cf() -> ContactFriction {
        let cfg = default_friction_config();
        new_contact_friction(&cfg, 10.0, 0.5)
    }

    // 1. default_friction_config has static >= kinetic
    #[test]
    fn test_default_static_ge_kinetic() {
        let cfg = default_friction_config();
        assert!(cfg.static_coefficient >= cfg.kinetic_coefficient);
    }

    // 2. default_friction_config rolling << kinetic
    #[test]
    fn test_default_rolling_lt_kinetic() {
        let cfg = default_friction_config();
        assert!(cfg.rolling_coefficient < cfg.kinetic_coefficient);
    }

    // 3. new_contact_friction: zero velocity → Static mode
    #[test]
    fn test_new_contact_static_mode_zero_vel() {
        let cfg = default_friction_config();
        let cf = new_contact_friction(&cfg, 10.0, 0.0);
        assert_eq!(cf.mode, FrictionMode::Static);
    }

    // 4. new_contact_friction: nonzero velocity → Kinetic mode
    #[test]
    fn test_new_contact_kinetic_mode_nonzero_vel() {
        let cfg = default_friction_config();
        let cf = new_contact_friction(&cfg, 10.0, 1.0);
        assert_eq!(cf.mode, FrictionMode::Kinetic);
    }

    // 5. static_friction_force = mu_s * N
    #[test]
    fn test_static_friction_force_formula() {
        let cf = default_cf();
        let expected = cf.mu_static * cf.normal_force;
        assert!((static_friction_force(&cf) - expected).abs() < EPS);
    }

    // 6. kinetic_friction_force = mu_k * N
    #[test]
    fn test_kinetic_friction_force_formula() {
        let cf = default_cf();
        let expected = cf.mu_kinetic * cf.normal_force;
        assert!((kinetic_friction_force(&cf) - expected).abs() < EPS);
    }

    // 7. kinetic < static friction force for same N
    #[test]
    fn test_kinetic_lt_static_force() {
        let cf = default_cf();
        assert!(kinetic_friction_force(&cf) <= static_friction_force(&cf));
    }

    // 8. rolling_friction_torque = mu_r * N * r
    #[test]
    fn test_rolling_friction_torque_formula() {
        let cf = default_cf();
        let r = 0.5;
        let expected = cf.mu_rolling * cf.normal_force * r;
        assert!((rolling_friction_torque(&cf, r) - expected).abs() < EPS);
    }

    // 9. friction_coefficient for Static mode returns mu_s
    #[test]
    fn test_friction_coefficient_static_mode() {
        let cfg = default_friction_config();
        let mut cf = new_contact_friction(&cfg, 10.0, 0.0);
        cf.mode = FrictionMode::Static;
        assert!((friction_coefficient(&cf) - cf.mu_static).abs() < EPS);
    }

    // 10. friction_coefficient for Kinetic mode returns mu_k
    #[test]
    fn test_friction_coefficient_kinetic_mode() {
        let cfg = default_friction_config();
        let mut cf = new_contact_friction(&cfg, 10.0, 1.0);
        cf.mode = FrictionMode::Kinetic;
        assert!((friction_coefficient(&cf) - cf.mu_kinetic).abs() < EPS);
    }

    // 11. is_sliding: small force → not sliding
    #[test]
    fn test_is_sliding_false_small_force() {
        let cf = default_cf(); // static friction limit = 0.6 * 10 = 6 N
        assert!(!is_sliding(&cf, 2.0));
    }

    // 12. is_sliding: large force → sliding
    #[test]
    fn test_is_sliding_true_large_force() {
        let cf = default_cf();
        assert!(is_sliding(&cf, 100.0));
    }

    // 13. apply_friction_impulse on static body returns unchanged velocity
    #[test]
    fn test_apply_friction_impulse_static_body() {
        let cf = default_cf();
        let vel = [1.0f32, 0.0, 0.0];
        let (imp, new_vel) = apply_friction_impulse(&cf, vel, [0.0, 1.0, 0.0], 0.0);
        assert!((imp - 0.0).abs() < EPS);
        assert_eq!(new_vel, vel);
    }

    // 14. apply_friction_impulse reduces tangential velocity
    #[test]
    fn test_apply_friction_impulse_reduces_tangential() {
        let cf = default_cf();
        let vel = [2.0f32, 0.0, 0.0]; // moving in x, normal is y
        let (_, new_vel) = apply_friction_impulse(&cf, vel, [0.0, 1.0, 0.0], 1.0);
        // tangential component (x) should be reduced
        assert!(new_vel[0].abs() < vel[0].abs());
    }

    // 15. friction_energy_dissipation is non-negative
    #[test]
    fn test_friction_energy_dissipation_non_negative() {
        let cf = default_cf();
        let e = friction_energy_dissipation(&cf, 0.016);
        assert!(e >= 0.0);
    }

    // 16. friction_energy_dissipation zero at zero velocity
    #[test]
    fn test_friction_energy_dissipation_zero_velocity() {
        let cfg = default_friction_config();
        let cf = new_contact_friction(&cfg, 10.0, 0.0);
        let e = friction_energy_dissipation(&cf, 0.1);
        assert!((e - 0.0).abs() < EPS);
    }

    // 17. set_static_coefficient updates mu_s
    #[test]
    fn test_set_static_coefficient_updates() {
        let mut cf = default_cf();
        set_static_coefficient(&mut cf, 0.9);
        assert!((cf.mu_static - 0.9).abs() < EPS);
    }

    // 18. set_static_coefficient clamps to zero for negative input
    #[test]
    fn test_set_static_coefficient_clamps_negative() {
        let mut cf = default_cf();
        set_static_coefficient(&mut cf, -0.5);
        assert!(cf.mu_static >= 0.0);
    }

    // 19. set_kinetic_coefficient updates mu_k
    #[test]
    fn test_set_kinetic_coefficient_updates() {
        let mut cf = default_cf();
        set_kinetic_coefficient(&mut cf, 0.3);
        assert!((cf.mu_kinetic - 0.3).abs() < EPS);
    }

    // 20. friction_mode accessor matches stored mode
    #[test]
    fn test_friction_mode_accessor() {
        let cf = default_cf(); // kinetic (vel=0.5)
        assert_eq!(friction_mode(&cf), cf.mode);
    }

    // 21. stribeck_friction at zero velocity ≈ static coefficient
    #[test]
    fn test_stribeck_zero_velocity_approaches_static() {
        let cfg = default_friction_config();
        let mu = stribeck_friction(&cfg, 0.0);
        assert!((mu - cfg.static_coefficient).abs() < EPS);
    }

    // 22. stribeck_friction at high velocity ≈ kinetic coefficient
    #[test]
    fn test_stribeck_high_velocity_approaches_kinetic() {
        let cfg = default_friction_config();
        let mu = stribeck_friction(&cfg, 100.0 * cfg.stribeck_velocity);
        assert!((mu - cfg.kinetic_coefficient).abs() < 1e-3);
    }

    // 23. stribeck_friction is always in [mu_k, mu_s]
    #[test]
    fn test_stribeck_always_in_range() {
        let cfg = default_friction_config();
        for v in [0.0f32, 0.001, 0.01, 0.1, 1.0, 10.0] {
            let mu = stribeck_friction(&cfg, v);
            assert!(
                mu >= cfg.kinetic_coefficient - EPS && mu <= cfg.static_coefficient + EPS,
                "stribeck({v}) = {mu} out of range"
            );
        }
    }

    // 24. friction_normal_force returns stored normal force
    #[test]
    fn test_friction_normal_force_accessor() {
        let cf = default_cf();
        assert!((friction_normal_force(&cf) - 10.0).abs() < EPS);
    }

    // 25. rolling_friction_torque zero for zero radius
    #[test]
    fn test_rolling_friction_torque_zero_radius() {
        let cf = default_cf();
        assert!((rolling_friction_torque(&cf, 0.0) - 0.0).abs() < EPS);
    }

    // 26. kinetic_friction_force zero for zero normal force
    #[test]
    fn test_kinetic_friction_force_zero_normal() {
        let cfg = default_friction_config();
        let cf = new_contact_friction(&cfg, 0.0, 1.0);
        assert!((kinetic_friction_force(&cf) - 0.0).abs() < EPS);
    }

    // 27. apply_friction_impulse zero velocity returns unchanged
    #[test]
    fn test_apply_friction_impulse_zero_velocity() {
        let cf = default_cf();
        let vel = [0.0f32; 3];
        let (imp, new_vel) = apply_friction_impulse(&cf, vel, [0.0, 1.0, 0.0], 1.0);
        assert!((imp - 0.0).abs() < EPS);
        assert_eq!(new_vel, [0.0; 3]);
    }

    // 28. static_friction_force is non-negative for positive N
    #[test]
    fn test_static_friction_force_non_negative() {
        let cf = default_cf();
        assert!(static_friction_force(&cf) >= 0.0);
    }

    // 29. new_contact_friction copies mu values from config
    #[test]
    fn test_new_contact_friction_copies_mu_values() {
        let cfg = default_friction_config();
        let cf = new_contact_friction(&cfg, 5.0, 2.0);
        assert!((cf.mu_static - cfg.static_coefficient).abs() < EPS);
        assert!((cf.mu_kinetic - cfg.kinetic_coefficient).abs() < EPS);
    }

    // 30. friction_energy_dissipation proportional to normal force
    #[test]
    fn test_friction_energy_dissipation_proportional_to_normal() {
        let cfg = default_friction_config();
        let cf1 = new_contact_friction(&cfg, 10.0, 1.0);
        let cf2 = new_contact_friction(&cfg, 20.0, 1.0);
        let e1 = friction_energy_dissipation(&cf1, 1.0);
        let e2 = friction_energy_dissipation(&cf2, 1.0);
        assert!((e2 / e1 - 2.0).abs() < EPS, "e2={e2}, e1={e1}");
    }
}
