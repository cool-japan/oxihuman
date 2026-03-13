// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! GPU fence and synchronization primitives.

use std::f32::consts::PI;

/// Configuration for gpu fence.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GpuFenceConfig {
    pub timeout_ms: f32,
    pub max_in_flight: f32,
    pub signal_value: f32,
    pub wait_value: f32,
    pub enabled: f32,
}

#[allow(dead_code)]
pub fn default_gpu_fence() -> GpuFenceConfig {
    GpuFenceConfig { timeout_ms: 1000.0, max_in_flight: 2.0, signal_value: 0.0, wait_value: 0.0, enabled: 1.0 }
}

#[allow(dead_code)]
pub fn set_gpu_fence_timeout_ms(cfg: &mut GpuFenceConfig, value: f32) {
    cfg.timeout_ms = value.clamp(1.0_f32, 60000.0_f32);
}

#[allow(dead_code)]
pub fn set_gpu_fence_max_in_flight(cfg: &mut GpuFenceConfig, value: f32) {
    cfg.max_in_flight = value.clamp(1.0_f32, 128.0_f32);
}

#[allow(dead_code)]
pub fn set_gpu_fence_signal_value(cfg: &mut GpuFenceConfig, value: f32) {
    cfg.signal_value = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_gpu_fence_wait_value(cfg: &mut GpuFenceConfig, value: f32) {
    cfg.wait_value = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_gpu_fence_enabled(cfg: &mut GpuFenceConfig, value: f32) {
    cfg.enabled = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn gpu_fence_weight(cfg: &GpuFenceConfig) -> f32 {
    (cfg.timeout_ms * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_gpu_fence(a: &GpuFenceConfig, b: &GpuFenceConfig, t: f32) -> GpuFenceConfig {
    let t = t.clamp(0.0, 1.0);
    GpuFenceConfig {
        timeout_ms: a.timeout_ms + (b.timeout_ms - a.timeout_ms) * t,
        max_in_flight: a.max_in_flight + (b.max_in_flight - a.max_in_flight) * t,
        signal_value: a.signal_value + (b.signal_value - a.signal_value) * t,
        wait_value: a.wait_value + (b.wait_value - a.wait_value) * t,
        enabled: a.enabled + (b.enabled - a.enabled) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_gpu_fence();
        assert!((cfg.timeout_ms - 1000.0_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_timeout_ms() {
        let mut cfg = default_gpu_fence();
        set_gpu_fence_timeout_ms(&mut cfg, 0.7);
        assert!((cfg.timeout_ms - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_max_in_flight() {
        let mut cfg = default_gpu_fence();
        set_gpu_fence_max_in_flight(&mut cfg, 0.8);
        assert!((cfg.max_in_flight - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_signal_value() {
        let mut cfg = default_gpu_fence();
        set_gpu_fence_signal_value(&mut cfg, 0.6);
        assert!((cfg.signal_value - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_wait_value() {
        let mut cfg = default_gpu_fence();
        set_gpu_fence_wait_value(&mut cfg, 0.5);
        assert!((cfg.wait_value - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_enabled() {
        let mut cfg = default_gpu_fence();
        set_gpu_fence_enabled(&mut cfg, 0.4);
        assert!((cfg.enabled - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_gpu_fence();
        let w = gpu_fence_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_gpu_fence();
        let mut b = default_gpu_fence();
        b.timeout_ms = 1.0;
        let mid = blend_gpu_fence(&a, &b, 0.5);
        assert!((mid.timeout_ms - 500.5_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_gpu_fence();
        let b = default_gpu_fence();
        let r = blend_gpu_fence(&a, &b, 0.0);
        assert!((r.timeout_ms - a.timeout_ms).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_gpu_fence();
        let b = default_gpu_fence();
        let r = blend_gpu_fence(&a, &b, 1.0);
        assert!((r.timeout_ms - b.timeout_ms).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_gpu_fence();
        let b = default_gpu_fence();
        let r = blend_gpu_fence(&a, &b, 2.0);
        assert!((r.timeout_ms - b.timeout_ms).abs() < 1e-6);
    }
}
