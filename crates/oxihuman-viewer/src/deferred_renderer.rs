// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Deferred renderer G-buffer configuration.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum GBufferTarget {
    Albedo,
    Normal,
    Metallic,
    Roughness,
    Emissive,
    Depth,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GBufferConfig {
    pub targets: Vec<GBufferTarget>,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
pub fn new_gbuffer_config(width: u32, height: u32) -> GBufferConfig {
    GBufferConfig { targets: Vec::new(), width, height }
}

#[allow(dead_code)]
pub fn gbuf_add_target(config: &mut GBufferConfig, target: GBufferTarget) {
    config.targets.push(target);
}

#[allow(dead_code)]
pub fn gbuf_target_count(config: &GBufferConfig) -> usize {
    config.targets.len()
}

#[allow(dead_code)]
pub fn gbuf_has_target(config: &GBufferConfig, target: &GBufferTarget) -> bool {
    config.targets.contains(target)
}

#[allow(dead_code)]
pub fn gbuf_memory_estimate_mb(config: &GBufferConfig) -> f32 {
    let count = config.targets.len() as f32;
    count * config.width as f32 * config.height as f32 * 4.0 / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_target() {
        let mut c = new_gbuffer_config(1920, 1080);
        gbuf_add_target(&mut c, GBufferTarget::Albedo);
        assert_eq!(gbuf_target_count(&c), 1);
    }

    #[test]
    fn test_has_target_true() {
        let mut c = new_gbuffer_config(1920, 1080);
        gbuf_add_target(&mut c, GBufferTarget::Normal);
        assert!(gbuf_has_target(&c, &GBufferTarget::Normal));
    }

    #[test]
    fn test_has_target_false() {
        let c = new_gbuffer_config(1920, 1080);
        assert!(!gbuf_has_target(&c, &GBufferTarget::Depth));
    }

    #[test]
    fn test_memory_estimate_positive() {
        let mut c = new_gbuffer_config(1920, 1080);
        gbuf_add_target(&mut c, GBufferTarget::Albedo);
        assert!(gbuf_memory_estimate_mb(&c) > 0.0);
    }

    #[test]
    fn test_standard_gbuffer() {
        let mut c = new_gbuffer_config(1920, 1080);
        gbuf_add_target(&mut c, GBufferTarget::Albedo);
        gbuf_add_target(&mut c, GBufferTarget::Normal);
        gbuf_add_target(&mut c, GBufferTarget::Metallic);
        gbuf_add_target(&mut c, GBufferTarget::Roughness);
        gbuf_add_target(&mut c, GBufferTarget::Depth);
        assert_eq!(gbuf_target_count(&c), 5);
        assert!(gbuf_has_target(&c, &GBufferTarget::Albedo));
        assert!(gbuf_has_target(&c, &GBufferTarget::Depth));
    }

    #[test]
    fn test_memory_estimate_scales_with_targets() {
        let mut c1 = new_gbuffer_config(256, 256);
        gbuf_add_target(&mut c1, GBufferTarget::Albedo);
        let mut c2 = new_gbuffer_config(256, 256);
        gbuf_add_target(&mut c2, GBufferTarget::Albedo);
        gbuf_add_target(&mut c2, GBufferTarget::Normal);
        assert!(gbuf_memory_estimate_mb(&c2) > gbuf_memory_estimate_mb(&c1));
    }

    #[test]
    fn test_empty_target_count() {
        let c = new_gbuffer_config(1920, 1080);
        assert_eq!(gbuf_target_count(&c), 0);
    }

    #[test]
    fn test_emissive_target() {
        let mut c = new_gbuffer_config(1920, 1080);
        gbuf_add_target(&mut c, GBufferTarget::Emissive);
        assert!(gbuf_has_target(&c, &GBufferTarget::Emissive));
    }
}
