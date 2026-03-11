// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Coulomb friction model (static, kinetic, viscous).

#![allow(dead_code)]

/// Configuration for the Coulomb friction model.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CoulombFrictionConfig {
    pub static_coeff: f32,
    pub kinetic_coeff: f32,
    pub viscous_coeff: f32,
}

/// State of a friction contact.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CoulombFrictionState {
    pub slip_velocity: f32,
    pub normal_force: f32,
    pub is_sliding: bool,
}

/// Return the default Coulomb friction config.
#[allow(dead_code)]
pub fn default_coulomb_friction_config() -> CoulombFrictionConfig {
    CoulombFrictionConfig {
        static_coeff: 0.6,
        kinetic_coeff: 0.4,
        viscous_coeff: 0.01,
    }
}

/// Create a new friction state.
#[allow(dead_code)]
pub fn new_coulomb_friction_state(normal_force: f32) -> CoulombFrictionState {
    CoulombFrictionState {
        slip_velocity: 0.0,
        normal_force,
        is_sliding: false,
    }
}

/// Compute the friction force magnitude given state and config.
#[allow(dead_code)]
pub fn coulomb_friction_force(state: &CoulombFrictionState, config: &CoulombFrictionConfig) -> f32 {
    let viscous = config.viscous_coeff * state.slip_velocity.abs();
    if state.is_sliding {
        config.kinetic_coeff * state.normal_force.abs() + viscous
    } else {
        config.static_coeff * state.normal_force.abs() + viscous
    }
}

/// Return whether sliding is occurring.
#[allow(dead_code)]
pub fn coulomb_friction_is_sliding(state: &CoulombFrictionState) -> bool {
    state.is_sliding
}

/// Return the static friction threshold force.
#[allow(dead_code)]
pub fn coulomb_friction_static_threshold(state: &CoulombFrictionState, config: &CoulombFrictionConfig) -> f32 {
    config.static_coeff * state.normal_force.abs()
}

/// Update the friction state given new slip velocity and an applied tangential force.
#[allow(dead_code)]
pub fn coulomb_friction_update_state(
    state: &mut CoulombFrictionState,
    slip_velocity: f32,
    applied_tangential: f32,
    config: &CoulombFrictionConfig,
) {
    state.slip_velocity = slip_velocity;
    let threshold = coulomb_friction_static_threshold(state, config);
    state.is_sliding = applied_tangential.abs() > threshold || slip_velocity.abs() > 1e-6;
}

/// Compute energy dissipated over dt.
#[allow(dead_code)]
pub fn coulomb_friction_energy_dissipated(
    state: &CoulombFrictionState,
    config: &CoulombFrictionConfig,
    dt: f32,
) -> f32 {
    coulomb_friction_force(state, config) * state.slip_velocity.abs() * dt
}

/// Return the effective friction coefficient at the given slip speed.
#[allow(dead_code)]
pub fn coulomb_friction_coefficient_at_slip(
    config: &CoulombFrictionConfig,
    slip_speed: f32,
) -> f32 {
    if slip_speed.abs() < 1e-6 {
        config.static_coeff
    } else {
        config.kinetic_coeff + config.viscous_coeff * slip_speed.abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_coulomb_friction_config();
        assert!(cfg.static_coeff > cfg.kinetic_coeff);
        assert!(cfg.kinetic_coeff > 0.0);
    }

    #[test]
    fn test_new_state() {
        let state = new_coulomb_friction_state(100.0);
        assert!((state.normal_force - 100.0).abs() < 1e-6);
        assert!(!state.is_sliding);
    }

    #[test]
    fn test_static_friction_force() {
        let state = new_coulomb_friction_state(10.0);
        let cfg = default_coulomb_friction_config();
        let f = coulomb_friction_force(&state, &cfg);
        assert!((f - 6.0).abs() < 1e-5); // 0.6 * 10
    }

    #[test]
    fn test_kinetic_friction_force() {
        let mut state = new_coulomb_friction_state(10.0);
        state.is_sliding = true;
        let cfg = default_coulomb_friction_config();
        let f = coulomb_friction_force(&state, &cfg);
        assert!((f - 4.0).abs() < 1e-5); // 0.4 * 10
    }

    #[test]
    fn test_is_sliding() {
        let mut state = new_coulomb_friction_state(10.0);
        assert!(!coulomb_friction_is_sliding(&state));
        state.is_sliding = true;
        assert!(coulomb_friction_is_sliding(&state));
    }

    #[test]
    fn test_static_threshold() {
        let state = new_coulomb_friction_state(20.0);
        let cfg = default_coulomb_friction_config();
        let threshold = coulomb_friction_static_threshold(&state, &cfg);
        assert!((threshold - 12.0).abs() < 1e-5); // 0.6 * 20
    }

    #[test]
    fn test_update_state_sets_sliding() {
        let mut state = new_coulomb_friction_state(10.0);
        let cfg = default_coulomb_friction_config();
        // Apply more than static threshold
        coulomb_friction_update_state(&mut state, 0.0, 100.0, &cfg);
        assert!(state.is_sliding);
    }

    #[test]
    fn test_energy_dissipated() {
        let mut state = new_coulomb_friction_state(10.0);
        state.is_sliding = true;
        state.slip_velocity = 1.0;
        let cfg = default_coulomb_friction_config();
        let e = coulomb_friction_energy_dissipated(&state, &cfg, 1.0);
        assert!(e > 0.0);
    }

    #[test]
    fn test_coefficient_at_zero_slip() {
        let cfg = default_coulomb_friction_config();
        let c = coulomb_friction_coefficient_at_slip(&cfg, 0.0);
        assert!((c - cfg.static_coeff).abs() < 1e-6);
    }

    #[test]
    fn test_coefficient_at_nonzero_slip() {
        let cfg = default_coulomb_friction_config();
        let c = coulomb_friction_coefficient_at_slip(&cfg, 1.0);
        assert!(c >= cfg.kinetic_coeff);
    }
}
