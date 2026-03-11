// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Environment probe management for IBL.

use std::f32::consts::PI;

/// Probe type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProbeType {
    Reflection,
    Irradiance,
    Combined,
}

/// Environment probe.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvProbe {
    pub position: [f32; 3],
    pub radius: f32,
    pub probe_type: ProbeType,
    pub intensity: f32,
    pub resolution: u32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn new_env_probe(x: f32, y: f32, z: f32, radius: f32) -> EnvProbe {
    EnvProbe {
        position: [x, y, z],
        radius,
        probe_type: ProbeType::Combined,
        intensity: 1.0,
        resolution: 256,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn set_probe_intensity(probe: &mut EnvProbe, value: f32) {
    probe.intensity = value.clamp(0.0, 10.0);
}

#[allow(dead_code)]
pub fn set_probe_resolution(probe: &mut EnvProbe, res: u32) {
    probe.resolution = res.clamp(16, 2048);
}

#[allow(dead_code)]
pub fn set_probe_type(probe: &mut EnvProbe, pt: ProbeType) {
    probe.probe_type = pt;
}

#[allow(dead_code)]
pub fn probe_volume(probe: &EnvProbe) -> f32 {
    (4.0 / 3.0) * PI * probe.radius * probe.radius * probe.radius
}

#[allow(dead_code)]
pub fn probe_contains_point(probe: &EnvProbe, x: f32, y: f32, z: f32) -> bool {
    let dx = x - probe.position[0];
    let dy = y - probe.position[1];
    let dz = z - probe.position[2];
    (dx * dx + dy * dy + dz * dz).sqrt() <= probe.radius
}

#[allow(dead_code)]
pub fn blend_probe_intensity(a: &EnvProbe, b: &EnvProbe, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    a.intensity + (b.intensity - a.intensity) * t
}

#[allow(dead_code)]
pub fn enable_probe(probe: &mut EnvProbe) {
    probe.enabled = true;
}

#[allow(dead_code)]
pub fn disable_probe(probe: &mut EnvProbe) {
    probe.enabled = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_probe() {
        let p = new_env_probe(0.0, 1.0, 0.0, 5.0);
        assert!((p.position[1] - 1.0).abs() < 1e-6);
        assert!(p.enabled);
    }

    #[test]
    fn test_set_intensity_clamp() {
        let mut p = new_env_probe(0.0, 0.0, 0.0, 1.0);
        set_probe_intensity(&mut p, 20.0);
        assert!((p.intensity - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_resolution_clamp() {
        let mut p = new_env_probe(0.0, 0.0, 0.0, 1.0);
        set_probe_resolution(&mut p, 5);
        assert_eq!(p.resolution, 16);
    }

    #[test]
    fn test_set_type() {
        let mut p = new_env_probe(0.0, 0.0, 0.0, 1.0);
        set_probe_type(&mut p, ProbeType::Reflection);
        assert_eq!(p.probe_type, ProbeType::Reflection);
    }

    #[test]
    fn test_probe_volume() {
        let p = new_env_probe(0.0, 0.0, 0.0, 1.0);
        let v = probe_volume(&p);
        assert!((v - (4.0 / 3.0) * PI).abs() < 1e-4);
    }

    #[test]
    fn test_contains_point_inside() {
        let p = new_env_probe(0.0, 0.0, 0.0, 5.0);
        assert!(probe_contains_point(&p, 1.0, 1.0, 1.0));
    }

    #[test]
    fn test_contains_point_outside() {
        let p = new_env_probe(0.0, 0.0, 0.0, 1.0);
        assert!(!probe_contains_point(&p, 10.0, 10.0, 10.0));
    }

    #[test]
    fn test_blend_intensity() {
        let a = new_env_probe(0.0, 0.0, 0.0, 1.0);
        let mut b = new_env_probe(0.0, 0.0, 0.0, 1.0);
        b.intensity = 3.0;
        let result = blend_probe_intensity(&a, &b, 0.5);
        assert!((result - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_enable_disable() {
        let mut p = new_env_probe(0.0, 0.0, 0.0, 1.0);
        disable_probe(&mut p);
        assert!(!p.enabled);
        enable_probe(&mut p);
        assert!(p.enabled);
    }

    #[test]
    fn test_probe_at_origin() {
        let p = new_env_probe(0.0, 0.0, 0.0, 1.0);
        assert!(probe_contains_point(&p, 0.0, 0.0, 0.0));
    }
}
