// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpinConfig {
    pub angle: f32,
    pub steps: usize,
    pub axis: [f32; 3],
    pub center: [f32; 3],
    pub duplicate: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpinResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub step_count: usize,
}

#[allow(dead_code)]
pub fn default_spin_config() -> SpinConfig {
    SpinConfig {
        angle: 2.0 * PI,
        steps: 8,
        axis: [0.0, 0.0, 1.0],
        center: [0.0; 3],
        duplicate: false,
    }
}

fn rotate_around_z(p: [f32; 3], center: [f32; 3], angle: f32) -> [f32; 3] {
    let dx = p[0] - center[0];
    let dy = p[1] - center[1];
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    [
        center[0] + dx * cos_a - dy * sin_a,
        center[1] + dx * sin_a + dy * cos_a,
        p[2],
    ]
}

#[allow(dead_code)]
pub fn spin_step_angle(config: &SpinConfig) -> f32 {
    if config.steps == 0 {
        0.0
    } else {
        config.angle / config.steps as f32
    }
}

#[allow(dead_code)]
pub fn spin_validate_config(config: &SpinConfig) -> bool {
    config.steps > 0 && config.angle.is_finite()
}

#[allow(dead_code)]
pub fn spin_profile(profile: &[[f32; 3]], config: &SpinConfig) -> SpinResult {
    if profile.is_empty() || config.steps == 0 {
        return SpinResult { positions: Vec::new(), indices: Vec::new(), step_count: 0 };
    }
    let n_profile = profile.len();
    let n_steps = config.steps;
    let step_ang = spin_step_angle(config);
    let mut positions: Vec<[f32; 3]> = Vec::new();
    for s in 0..=n_steps {
        let ang = step_ang * s as f32;
        for &p in profile {
            positions.push(rotate_around_z(p, config.center, ang));
        }
    }
    let mut indices: Vec<u32> = Vec::new();
    for s in 0..n_steps {
        for i in 0..n_profile {
            let next_i = (i + 1) % n_profile;
            let base = (s * n_profile) as u32;
            let base_next = ((s + 1) * n_profile) as u32;
            indices.extend_from_slice(&[
                base + i as u32,
                base + next_i as u32,
                base_next + i as u32,
            ]);
            indices.extend_from_slice(&[
                base + next_i as u32,
                base_next + next_i as u32,
                base_next + i as u32,
            ]);
        }
    }
    SpinResult { positions, indices, step_count: n_steps }
}

#[allow(dead_code)]
pub fn spin_vertex_count(result: &SpinResult) -> usize {
    result.positions.len()
}

#[allow(dead_code)]
pub fn spin_face_count(result: &SpinResult) -> usize {
    result.indices.len() / 3
}

#[allow(dead_code)]
pub fn spin_to_json(result: &SpinResult) -> String {
    format!(
        r#"{{"vertex_count":{},"face_count":{},"step_count":{}}}"#,
        result.positions.len(),
        spin_face_count(result),
        result.step_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_profile() -> Vec<[f32; 3]> {
        vec![[1.0, 0.0, 0.0], [1.0, 0.0, 1.0]]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_spin_config();
        assert_eq!(cfg.steps, 8);
        assert!((cfg.angle - 2.0 * PI).abs() < 1e-5);
    }

    #[test]
    fn test_step_angle() {
        let mut cfg = default_spin_config();
        cfg.steps = 4;
        cfg.angle = PI;
        let sa = spin_step_angle(&cfg);
        assert!((sa - PI / 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_validate_ok() {
        let cfg = default_spin_config();
        assert!(spin_validate_config(&cfg));
    }

    #[test]
    fn test_validate_zero_steps() {
        let mut cfg = default_spin_config();
        cfg.steps = 0;
        assert!(!spin_validate_config(&cfg));
    }

    #[test]
    fn test_spin_vertex_count() {
        let profile = line_profile();
        let cfg = default_spin_config();
        let res = spin_profile(&profile, &cfg);
        // (steps+1) * profile_len
        assert_eq!(spin_vertex_count(&res), 9 * 2);
    }

    #[test]
    fn test_spin_face_count() {
        let profile = line_profile();
        let cfg = default_spin_config();
        let res = spin_profile(&profile, &cfg);
        // steps * profile_len * 2 triangles
        assert_eq!(spin_face_count(&res), 8 * 2 * 2);
    }

    #[test]
    fn test_empty_profile() {
        let cfg = default_spin_config();
        let res = spin_profile(&[], &cfg);
        assert_eq!(spin_vertex_count(&res), 0);
        assert_eq!(spin_face_count(&res), 0);
    }

    #[test]
    fn test_to_json() {
        let profile = line_profile();
        let cfg = default_spin_config();
        let res = spin_profile(&profile, &cfg);
        let j = spin_to_json(&res);
        assert!(j.contains("vertex_count"));
        assert!(j.contains("step_count"));
    }
}
