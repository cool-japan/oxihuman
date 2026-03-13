// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Environment reflection probe.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvProbeConfig {
    pub resolution: u32,
    pub near: f32,
    pub far: f32,
    pub update_rate: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvProbe {
    pub position: [f32; 3],
    pub config: EnvProbeConfig,
    pub dirty: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvProbeResult {
    pub face_count: usize,
    pub resolution: u32,
}

#[allow(dead_code)]
pub fn default_env_probe_config() -> EnvProbeConfig {
    EnvProbeConfig { resolution: 128, near: 0.1, far: 100.0, update_rate: 1 }
}

#[allow(dead_code)]
pub fn new_env_probe(position: [f32; 3]) -> EnvProbe {
    EnvProbe { position, config: default_env_probe_config(), dirty: true }
}

#[allow(dead_code)]
pub fn probe_mark_dirty(probe: &mut EnvProbe) {
    probe.dirty = true;
}

#[allow(dead_code)]
pub fn probe_is_dirty(probe: &EnvProbe) -> bool {
    probe.dirty
}

#[allow(dead_code)]
pub fn probe_update(probe: &mut EnvProbe) -> EnvProbeResult {
    probe.dirty = false;
    EnvProbeResult { face_count: 6, resolution: probe.config.resolution }
}

#[allow(dead_code)]
pub fn probe_result(probe: &EnvProbe) -> EnvProbeResult {
    EnvProbeResult { face_count: 6, resolution: probe.config.resolution }
}

#[allow(dead_code)]
pub fn probe_set_position(probe: &mut EnvProbe, pos: [f32; 3]) {
    probe.position = pos;
    probe.dirty = true;
}

#[allow(dead_code)]
pub fn probe_to_json(probe: &EnvProbe) -> String {
    format!(
        r#"{{"position":[{:.4},{:.4},{:.4}],"resolution":{},"dirty":{}}}"#,
        probe.position[0], probe.position[1], probe.position[2],
        probe.config.resolution, probe.dirty
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_env_probe_config();
        assert_eq!(cfg.resolution, 128);
        assert!((cfg.near - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_new_probe_is_dirty() {
        let p = new_env_probe([0.0, 1.0, 0.0]);
        assert!(probe_is_dirty(&p));
    }

    #[test]
    fn test_probe_mark_dirty() {
        let mut p = new_env_probe([0.0, 0.0, 0.0]);
        p.dirty = false;
        probe_mark_dirty(&mut p);
        assert!(p.dirty);
    }

    #[test]
    fn test_probe_update_clears_dirty() {
        let mut p = new_env_probe([0.0, 0.0, 0.0]);
        let r = probe_update(&mut p);
        assert!(!p.dirty);
        assert_eq!(r.face_count, 6);
    }

    #[test]
    fn test_probe_result_face_count() {
        let p = new_env_probe([0.0, 0.0, 0.0]);
        let r = probe_result(&p);
        assert_eq!(r.face_count, 6);
        assert_eq!(r.resolution, 128);
    }

    #[test]
    fn test_probe_set_position_marks_dirty() {
        let mut p = new_env_probe([0.0, 0.0, 0.0]);
        p.dirty = false;
        probe_set_position(&mut p, [1.0, 2.0, 3.0]);
        assert!(p.dirty);
        assert!((p.position[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_probe_to_json() {
        let p = new_env_probe([1.0, 2.0, 3.0]);
        let j = probe_to_json(&p);
        assert!(j.contains("position"));
        assert!(j.contains("resolution"));
    }

    #[test]
    fn test_probe_resolution_in_result() {
        let mut p = new_env_probe([0.0, 0.0, 0.0]);
        p.config.resolution = 256;
        let r = probe_result(&p);
        assert_eq!(r.resolution, 256);
    }
}
