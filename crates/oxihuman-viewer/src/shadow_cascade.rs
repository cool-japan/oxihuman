// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Cascaded shadow map configuration.

#![allow(dead_code)]

/// Configuration for cascaded shadow maps.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShadowCascadeConfig {
    pub num_cascades: usize,
    pub max_distance: f32,
    pub lambda: f32,
}

/// A single shadow cascade.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShadowCascade {
    pub near: f32,
    pub far: f32,
    pub resolution: u32,
    pub bias: f32,
}

/// Runtime state for cascaded shadow maps.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShadowCascadeState {
    pub cascades: Vec<ShadowCascade>,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn default_shadow_cascade_config() -> ShadowCascadeConfig {
    ShadowCascadeConfig {
        num_cascades: 4,
        max_distance: 100.0,
        lambda: 0.75,
    }
}

#[allow(dead_code)]
pub fn new_shadow_cascade_state(config: &ShadowCascadeConfig) -> ShadowCascadeState {
    let splits = sc_compute_splits(config, 0.1, config.max_distance);
    let mut cascades = Vec::with_capacity(config.num_cascades);
    for i in 0..config.num_cascades {
        cascades.push(ShadowCascade {
            near: splits[i],
            far: splits[i + 1],
            resolution: 1024,
            bias: 0.005,
        });
    }
    ShadowCascadeState { cascades, enabled: true }
}

#[allow(dead_code)]
pub fn sc_compute_splits(config: &ShadowCascadeConfig, near: f32, far: f32) -> Vec<f32> {
    let n = config.num_cascades;
    let lambda = config.lambda.clamp(0.0, 1.0);
    let mut splits = Vec::with_capacity(n + 1);
    splits.push(near);
    for i in 1..n {
        let uniform = near + (far - near) * (i as f32 / n as f32);
        let log = near * (far / near).powf(i as f32 / n as f32);
        splits.push(lambda * log + (1.0 - lambda) * uniform);
    }
    splits.push(far);
    splits
}

#[allow(dead_code)]
pub fn sc_cascade_count(state: &ShadowCascadeState) -> usize {
    state.cascades.len()
}

#[allow(dead_code)]
pub fn sc_get_cascade(state: &ShadowCascadeState, index: usize) -> Option<&ShadowCascade> {
    state.cascades.get(index)
}

#[allow(dead_code)]
pub fn sc_set_enabled(state: &mut ShadowCascadeState, enabled: bool) {
    state.enabled = enabled;
}

#[allow(dead_code)]
pub fn sc_to_json(state: &ShadowCascadeState) -> String {
    format!(
        r#"{{"enabled":{},"cascade_count":{}}}"#,
        state.enabled,
        state.cascades.len()
    )
}

#[allow(dead_code)]
pub fn sc_total_resolution(state: &ShadowCascadeState) -> u32 {
    state.cascades.iter().map(|c| c.resolution).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_shadow_cascade_config();
        assert_eq!(cfg.num_cascades, 4);
        assert!((cfg.max_distance - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_cascade_count() {
        let cfg = default_shadow_cascade_config();
        let state = new_shadow_cascade_state(&cfg);
        assert_eq!(state.cascades.len(), 4);
    }

    #[test]
    fn test_new_state_enabled() {
        let cfg = default_shadow_cascade_config();
        let state = new_shadow_cascade_state(&cfg);
        assert!(state.enabled);
    }

    #[test]
    fn test_compute_splits_count() {
        let cfg = default_shadow_cascade_config();
        let splits = sc_compute_splits(&cfg, 0.1, 100.0);
        assert_eq!(splits.len(), cfg.num_cascades + 1);
    }

    #[test]
    fn test_compute_splits_bounds() {
        let cfg = default_shadow_cascade_config();
        let splits = sc_compute_splits(&cfg, 0.1, 100.0);
        assert!((splits[0] - 0.1).abs() < 1e-5);
        assert!((splits[cfg.num_cascades] - 100.0).abs() < 1e-3);
    }

    #[test]
    fn test_set_enabled() {
        let cfg = default_shadow_cascade_config();
        let mut state = new_shadow_cascade_state(&cfg);
        sc_set_enabled(&mut state, false);
        assert!(!state.enabled);
    }

    #[test]
    fn test_total_resolution() {
        let cfg = default_shadow_cascade_config();
        let state = new_shadow_cascade_state(&cfg);
        assert_eq!(sc_total_resolution(&state), 1024 * 4);
    }

    #[test]
    fn test_to_json_contains_enabled() {
        let cfg = default_shadow_cascade_config();
        let state = new_shadow_cascade_state(&cfg);
        let j = sc_to_json(&state);
        assert!(j.contains("enabled"));
        assert!(j.contains("cascade_count"));
    }
}
