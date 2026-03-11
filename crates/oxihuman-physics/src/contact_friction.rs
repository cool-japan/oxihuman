// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Contact friction with static/kinetic threshold.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactFrictionConfig {
    pub static_mu: f32,
    pub kinetic_mu: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactState {
    pub normal: [f32; 3],
    pub tangent_vel: [f32; 3],
    pub normal_force: f32,
    pub is_sliding: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FrictionForceResult {
    pub tangent_force: [f32; 3],
    pub friction_mu: f32,
}

#[allow(dead_code)]
pub fn default_contact_friction_config() -> ContactFrictionConfig {
    ContactFrictionConfig { static_mu: 0.6, kinetic_mu: 0.4 }
}

#[allow(dead_code)]
pub fn new_contact_state(normal: [f32; 3], tangent_vel: [f32; 3], normal_force: f32) -> ContactState {
    let tangent_speed = vec3_len(tangent_vel);
    ContactState { normal, tangent_vel, normal_force, is_sliding: tangent_speed > 1e-6 }
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = vec3_len(v);
    if len < 1e-10 {
        return [0.0; 3];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[allow(dead_code)]
pub fn compute_friction_force(state: &ContactState, config: &ContactFrictionConfig) -> FrictionForceResult {
    let mu = if state.is_sliding { config.kinetic_mu } else { config.static_mu };
    let max_friction = mu * state.normal_force.abs();
    let dir = vec3_normalize(state.tangent_vel);
    // Friction opposes tangent velocity
    let tangent_force = [-dir[0] * max_friction, -dir[1] * max_friction, -dir[2] * max_friction];
    FrictionForceResult { tangent_force, friction_mu: mu }
}

#[allow(dead_code)]
pub fn contact_is_sliding(state: &ContactState, _config: &ContactFrictionConfig) -> bool {
    state.is_sliding
}

#[allow(dead_code)]
pub fn contact_normal_force(state: &ContactState) -> f32 {
    state.normal_force
}

#[allow(dead_code)]
pub fn contact_tangent_speed(state: &ContactState) -> f32 {
    vec3_len(state.tangent_vel)
}

#[allow(dead_code)]
pub fn contact_set_sliding(state: &mut ContactState, sliding: bool) {
    state.is_sliding = sliding;
}

#[allow(dead_code)]
pub fn friction_impulse_magnitude(state: &ContactState, config: &ContactFrictionConfig, dt: f32) -> f32 {
    let mu = if state.is_sliding { config.kinetic_mu } else { config.static_mu };
    mu * state.normal_force.abs() * dt
}

#[allow(dead_code)]
pub struct ContactFriction {
    pub mu_static: f32,
    pub mu_dynamic: f32,
    pub restitution: f32,
}

#[allow(dead_code)]
pub fn new_contact_friction(mu_s: f32, mu_d: f32) -> ContactFriction {
    ContactFriction { mu_static: mu_s, mu_dynamic: mu_d, restitution: 0.5 }
}

#[allow(dead_code)]
pub fn cf_is_sliding(f: &ContactFriction, tangential_force: f32, normal_force: f32) -> bool {
    tangential_force.abs() > f.mu_static * normal_force.abs()
}

#[allow(dead_code)]
pub fn cf_friction_force(f: &ContactFriction, tangential_vel: [f32; 3], normal_force: f32) -> [f32; 3] {
    let len = (tangential_vel[0].powi(2) + tangential_vel[1].powi(2) + tangential_vel[2].powi(2)).sqrt();
    if len < 1e-10 {
        return [0.0; 3];
    }
    let scale = -f.mu_dynamic * normal_force.abs() / len;
    [tangential_vel[0] * scale, tangential_vel[1] * scale, tangential_vel[2] * scale]
}

#[allow(dead_code)]
pub fn cf_impulse_normal(f: &ContactFriction, v_rel_normal: f32, mass_inv: f32) -> f32 {
    if mass_inv.abs() < 1e-15 {
        return 0.0;
    }
    -(1.0 + f.restitution) * v_rel_normal / (2.0 * mass_inv)
}

#[allow(dead_code)]
pub fn cf_mu_static(f: &ContactFriction) -> f32 {
    f.mu_static
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_contact_friction_config();
        assert!((cfg.static_mu - 0.6).abs() < 1e-6);
        assert!((cfg.kinetic_mu - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_new_contact_state_sliding() {
        let state = new_contact_state([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], 10.0);
        assert!(state.is_sliding);
    }

    #[test]
    fn test_new_contact_state_static() {
        let state = new_contact_state([0.0, 1.0, 0.0], [0.0, 0.0, 0.0], 10.0);
        assert!(!state.is_sliding);
    }

    #[test]
    fn test_compute_friction_kinetic() {
        let cfg = default_contact_friction_config();
        let state = new_contact_state([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], 10.0);
        let result = compute_friction_force(&state, &cfg);
        assert!((result.friction_mu - 0.4).abs() < 1e-6);
        assert!((result.tangent_force[0] + 4.0).abs() < 1e-5); // -mu*Fn
    }

    #[test]
    fn test_contact_normal_force() {
        let state = new_contact_state([0.0, 1.0, 0.0], [0.0; 3], 15.0);
        assert!((contact_normal_force(&state) - 15.0).abs() < 1e-6);
    }

    #[test]
    fn test_contact_tangent_speed() {
        let state = new_contact_state([0.0, 1.0, 0.0], [3.0, 4.0, 0.0], 1.0);
        assert!((contact_tangent_speed(&state) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_contact_set_sliding() {
        let mut state = new_contact_state([0.0, 1.0, 0.0], [0.0; 3], 1.0);
        contact_set_sliding(&mut state, true);
        assert!(state.is_sliding);
    }

    #[test]
    fn test_friction_impulse() {
        let cfg = default_contact_friction_config();
        let state = new_contact_state([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], 10.0);
        let imp = friction_impulse_magnitude(&state, &cfg, 0.1);
        assert!((imp - 0.4).abs() < 1e-5); // kinetic: 0.4 * 10 * 0.1
    }

    #[test]
    fn test_is_sliding() {
        let cfg = default_contact_friction_config();
        let s1 = new_contact_state([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], 1.0);
        let s2 = new_contact_state([0.0, 1.0, 0.0], [0.0; 3], 1.0);
        assert!(contact_is_sliding(&s1, &cfg));
        assert!(!contact_is_sliding(&s2, &cfg));
    }
}
