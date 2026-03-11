// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Surface adhesion/stiction force.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AdhesionConfig {
    pub strength: f32,
    pub range: f32,
    pub surface_energy: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AdhesionState {
    pub distance: f32,
    pub attached: bool,
    pub pull_off_force: f32,
}

#[allow(dead_code)]
pub fn default_adhesion_config() -> AdhesionConfig {
    AdhesionConfig { strength: 10.0, range: 0.1, surface_energy: 0.5 }
}

#[allow(dead_code)]
pub fn new_adhesion_state(distance: f32, config: &AdhesionConfig) -> AdhesionState {
    let attached = distance <= config.range;
    let pull_off_force = adhesion_pull_off_force(config);
    AdhesionState { distance, attached, pull_off_force }
}

#[allow(dead_code)]
pub fn adhesion_force_at_distance(config: &AdhesionConfig, dist: f32) -> f32 {
    if dist > config.range || dist < 0.0 {
        return 0.0;
    }
    let t = 1.0 - dist / config.range;
    config.strength * t * t
}

#[allow(dead_code)]
pub fn adhesion_is_attached(state: &AdhesionState) -> bool {
    state.attached
}

#[allow(dead_code)]
pub fn adhesion_pull_off_force(config: &AdhesionConfig) -> f32 {
    // Approximate pull-off force from surface energy (Derjaguin-Muller-Toporov model stub)
    config.strength * config.surface_energy
}

#[allow(dead_code)]
pub fn adhesion_update_state(state: &mut AdhesionState, dist: f32, config: &AdhesionConfig) {
    state.distance = dist;
    state.attached = dist <= config.range;
    state.pull_off_force = adhesion_pull_off_force(config);
}

#[allow(dead_code)]
pub fn adhesion_energy(state: &AdhesionState, config: &AdhesionConfig) -> f32 {
    if !state.attached {
        return 0.0;
    }
    let f = adhesion_force_at_distance(config, state.distance);
    0.5 * f * state.distance
}

#[allow(dead_code)]
pub fn adhesion_to_json(state: &AdhesionState) -> String {
    format!(
        r#"{{"distance":{},"attached":{},"pull_off_force":{}}}"#,
        state.distance, state.attached, state.pull_off_force
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state_attached() {
        let cfg = default_adhesion_config();
        let s = new_adhesion_state(0.05, &cfg);
        assert!(adhesion_is_attached(&s));
    }

    #[test]
    fn test_new_state_detached() {
        let cfg = default_adhesion_config();
        let s = new_adhesion_state(0.5, &cfg);
        assert!(!adhesion_is_attached(&s));
    }

    #[test]
    fn test_force_at_zero_distance() {
        let cfg = default_adhesion_config();
        let f = adhesion_force_at_distance(&cfg, 0.0);
        assert!((f - cfg.strength).abs() < 1e-5);
    }

    #[test]
    fn test_force_zero_beyond_range() {
        let cfg = default_adhesion_config();
        let f = adhesion_force_at_distance(&cfg, 1.0);
        assert!((f - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_pull_off_force() {
        let cfg = default_adhesion_config();
        let f = adhesion_pull_off_force(&cfg);
        assert!(f > 0.0);
    }

    #[test]
    fn test_update_state() {
        let cfg = default_adhesion_config();
        let mut s = new_adhesion_state(0.05, &cfg);
        adhesion_update_state(&mut s, 0.5, &cfg);
        assert!(!s.attached);
    }

    #[test]
    fn test_energy_zero_when_detached() {
        let cfg = default_adhesion_config();
        let s = new_adhesion_state(1.0, &cfg);
        assert!((adhesion_energy(&s, &cfg) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_to_json() {
        let cfg = default_adhesion_config();
        let s = new_adhesion_state(0.05, &cfg);
        let json = adhesion_to_json(&s);
        assert!(json.contains("attached"));
        assert!(json.contains("distance"));
    }
}
