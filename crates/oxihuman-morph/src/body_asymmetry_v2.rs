// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Body asymmetry v2 — independent per-region asymmetry offsets.

use std::f32::consts::PI;

/// Number of asymmetry regions.
pub const REGION_COUNT: usize = 8;

/// Per-region asymmetry configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyAsymmetryV2Config {
    pub max_offset: f32,
}

impl Default for BodyAsymmetryV2Config {
    fn default() -> Self {
        Self { max_offset: 0.15 }
    }
}

/// Asymmetry offsets for each body region.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyAsymmetryV2State {
    pub offsets: [f32; REGION_COUNT],
    pub config: BodyAsymmetryV2Config,
}

#[allow(dead_code)]
pub fn default_body_asymmetry_v2_config() -> BodyAsymmetryV2Config {
    BodyAsymmetryV2Config::default()
}

#[allow(dead_code)]
pub fn new_body_asymmetry_v2_state(config: BodyAsymmetryV2Config) -> BodyAsymmetryV2State {
    BodyAsymmetryV2State {
        offsets: [0.0; REGION_COUNT],
        config,
    }
}

#[allow(dead_code)]
pub fn bav2_set_offset(state: &mut BodyAsymmetryV2State, region: usize, offset: f32) {
    if region < REGION_COUNT {
        let max = state.config.max_offset;
        state.offsets[region] = offset.clamp(-max, max);
    }
}

#[allow(dead_code)]
pub fn bav2_reset(state: &mut BodyAsymmetryV2State) {
    state.offsets = [0.0; REGION_COUNT];
}

#[allow(dead_code)]
pub fn bav2_is_neutral(state: &BodyAsymmetryV2State) -> bool {
    state.offsets.iter().all(|&v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn bav2_total_deviation(state: &BodyAsymmetryV2State) -> f32 {
    state.offsets.iter().map(|v| v.abs()).sum()
}

#[allow(dead_code)]
pub fn bav2_average_deviation(state: &BodyAsymmetryV2State) -> f32 {
    bav2_total_deviation(state) / REGION_COUNT as f32
}

#[allow(dead_code)]
pub fn bav2_angular_spread_rad(state: &BodyAsymmetryV2State) -> f32 {
    let dev = bav2_total_deviation(state);
    (dev * PI / state.config.max_offset).min(PI)
}

#[allow(dead_code)]
pub fn bav2_to_weights(state: &BodyAsymmetryV2State) -> [f32; REGION_COUNT] {
    let max = state.config.max_offset;
    let mut w = [0.0f32; REGION_COUNT];
    #[allow(clippy::needless_range_loop)]
    for i in 0..REGION_COUNT {
        w[i] = if max > 1e-9 {
            state.offsets[i] / max
        } else {
            0.0
        };
    }
    w
}

#[allow(dead_code)]
pub fn bav2_blend(
    a: &BodyAsymmetryV2State,
    b: &BodyAsymmetryV2State,
    t: f32,
) -> [f32; REGION_COUNT] {
    let t = t.clamp(0.0, 1.0);
    let mut out = [0.0f32; REGION_COUNT];
    #[allow(clippy::needless_range_loop)]
    for i in 0..REGION_COUNT {
        out[i] = a.offsets[i] * (1.0 - t) + b.offsets[i] * t;
    }
    out
}

#[allow(dead_code)]
pub fn bav2_to_json(state: &BodyAsymmetryV2State) -> String {
    let parts: Vec<String> = state
        .offsets
        .iter()
        .enumerate()
        .map(|(i, v)| format!("\"region_{i}\":{v:.4}"))
        .collect();
    format!("{{{}}}", parts.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_neutral() {
        let s = new_body_asymmetry_v2_state(default_body_asymmetry_v2_config());
        assert!(bav2_is_neutral(&s));
    }

    #[test]
    fn set_offset_clamps() {
        let mut s = new_body_asymmetry_v2_state(default_body_asymmetry_v2_config());
        bav2_set_offset(&mut s, 0, 999.0);
        assert!((0.0..=1.0).contains(&s.offsets[0]));
    }

    #[test]
    fn reset_returns_to_neutral() {
        let mut s = new_body_asymmetry_v2_state(default_body_asymmetry_v2_config());
        bav2_set_offset(&mut s, 0, 0.1);
        bav2_reset(&mut s);
        assert!(bav2_is_neutral(&s));
    }

    #[test]
    fn total_deviation_sums_abs() {
        let mut s = new_body_asymmetry_v2_state(default_body_asymmetry_v2_config());
        bav2_set_offset(&mut s, 0, 0.1);
        bav2_set_offset(&mut s, 1, -0.05);
        assert!((bav2_total_deviation(&s) - 0.15).abs() < 1e-5);
    }

    #[test]
    fn average_deviation_correct() {
        let mut s = new_body_asymmetry_v2_state(default_body_asymmetry_v2_config());
        bav2_set_offset(&mut s, 0, 0.08);
        let avg = bav2_average_deviation(&s);
        assert!((avg - 0.08 / REGION_COUNT as f32).abs() < 1e-5);
    }

    #[test]
    fn to_weights_range() {
        let mut s = new_body_asymmetry_v2_state(default_body_asymmetry_v2_config());
        bav2_set_offset(&mut s, 2, 0.15);
        let w = bav2_to_weights(&s);
        assert!((w[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn blend_midpoint() {
        let mut a = new_body_asymmetry_v2_state(default_body_asymmetry_v2_config());
        let mut b = new_body_asymmetry_v2_state(default_body_asymmetry_v2_config());
        bav2_set_offset(&mut a, 0, 0.0);
        bav2_set_offset(&mut b, 0, 0.1);
        let w = bav2_blend(&a, &b, 0.5);
        assert!((w[0] - 0.05).abs() < 1e-5);
    }

    #[test]
    fn to_json_contains_region_key() {
        let s = new_body_asymmetry_v2_state(default_body_asymmetry_v2_config());
        let j = bav2_to_json(&s);
        assert!(j.contains("region_0"));
    }

    #[test]
    fn angular_spread_is_nonneg() {
        let s = new_body_asymmetry_v2_state(default_body_asymmetry_v2_config());
        assert!(bav2_angular_spread_rad(&s) >= 0.0);
    }

    #[test]
    fn out_of_range_region_ignored() {
        let mut s = new_body_asymmetry_v2_state(default_body_asymmetry_v2_config());
        bav2_set_offset(&mut s, REGION_COUNT + 5, 0.1); // no panic
        assert!(bav2_is_neutral(&s));
    }
}
