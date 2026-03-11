// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Environment cubemap: manages environment map state for IBL lighting.

use std::f32::consts::PI;

/// Environment cubemap descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvCubemap {
    pub face_resolution: u32,
    pub mip_levels: u32,
    pub intensity: f32,
    pub rotation_deg: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn new_env_cubemap(face_resolution: u32) -> EnvCubemap {
    let mip_levels = ((face_resolution as f32).log2().floor() as u32).max(1);
    EnvCubemap {
        face_resolution,
        mip_levels,
        intensity: 1.0,
        rotation_deg: 0.0,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn set_env_intensity(env: &mut EnvCubemap, v: f32) {
    env.intensity = v.max(0.0);
}

#[allow(dead_code)]
pub fn set_env_rotation(env: &mut EnvCubemap, degrees: f32) {
    env.rotation_deg = degrees % 360.0;
}

#[allow(dead_code)]
pub fn set_env_enabled(env: &mut EnvCubemap, enabled: bool) {
    env.enabled = enabled;
}

/// Rotation in radians.
#[allow(dead_code)]
pub fn env_rotation_rad(env: &EnvCubemap) -> f32 {
    env.rotation_deg * PI / 180.0
}

/// Total number of texels across all faces.
#[allow(dead_code)]
pub fn total_texel_count(env: &EnvCubemap) -> u64 {
    6 * (env.face_resolution as u64) * (env.face_resolution as u64)
}

/// Approximate memory in bytes (RGBA16F = 8 bytes per texel).
#[allow(dead_code)]
pub fn estimated_memory_bytes(env: &EnvCubemap) -> u64 {
    let mut total = 0u64;
    let mut size = env.face_resolution;
    for _ in 0..env.mip_levels {
        total += 6 * (size as u64) * (size as u64) * 8;
        size = (size / 2).max(1);
    }
    total
}

#[allow(dead_code)]
pub fn env_cubemap_to_json(env: &EnvCubemap) -> String {
    format!(
        r#"{{"face_resolution":{},"mip_levels":{},"intensity":{:.4},"rotation_deg":{:.1},"enabled":{}}}"#,
        env.face_resolution, env.mip_levels, env.intensity, env.rotation_deg, env.enabled
    )
}

#[allow(dead_code)]
pub fn reset_env_cubemap(env: &mut EnvCubemap) {
    env.intensity = 1.0;
    env.rotation_deg = 0.0;
    env.enabled = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_env_cubemap() {
        let e = new_env_cubemap(256);
        assert_eq!(e.face_resolution, 256);
        assert_eq!(e.mip_levels, 8);
    }

    #[test]
    fn test_set_intensity() {
        let mut e = new_env_cubemap(128);
        set_env_intensity(&mut e, 2.5);
        assert!((e.intensity - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_intensity_clamps() {
        let mut e = new_env_cubemap(128);
        set_env_intensity(&mut e, -1.0);
        assert!(e.intensity.abs() < 1e-6);
    }

    #[test]
    fn test_set_rotation() {
        let mut e = new_env_cubemap(128);
        set_env_rotation(&mut e, 90.0);
        assert!((e.rotation_deg - 90.0).abs() < 1e-6);
    }

    #[test]
    fn test_rotation_rad() {
        let mut e = new_env_cubemap(128);
        set_env_rotation(&mut e, 180.0);
        assert!((env_rotation_rad(&e) - PI).abs() < 1e-4);
    }

    #[test]
    fn test_total_texel_count() {
        let e = new_env_cubemap(64);
        assert_eq!(total_texel_count(&e), 6 * 64 * 64);
    }

    #[test]
    fn test_estimated_memory() {
        let e = new_env_cubemap(128);
        assert!(estimated_memory_bytes(&e) > 0);
    }

    #[test]
    fn test_env_cubemap_to_json() {
        let e = new_env_cubemap(256);
        let j = env_cubemap_to_json(&e);
        assert!(j.contains("face_resolution"));
    }

    #[test]
    fn test_reset() {
        let mut e = new_env_cubemap(128);
        set_env_intensity(&mut e, 5.0);
        reset_env_cubemap(&mut e);
        assert!((e.intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_enabled() {
        let mut e = new_env_cubemap(128);
        set_env_enabled(&mut e, false);
        assert!(!e.enabled);
    }
}
