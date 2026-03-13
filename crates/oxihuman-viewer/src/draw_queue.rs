// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Draw call queue management for batched rendering.

use std::f32::consts::PI;

/// Configuration for draw queue.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawQueueConfig {
    pub max_items: f32,
    pub sort_key_bits: f32,
    pub depth_sort: f32,
    pub alpha_sort: f32,
    pub instancing: f32,
}

#[allow(dead_code)]
pub fn default_draw_queue() -> DrawQueueConfig {
    DrawQueueConfig { max_items: 1024.0, sort_key_bits: 32.0, depth_sort: 1.0, alpha_sort: 1.0, instancing: 0.0 }
}

#[allow(dead_code)]
pub fn set_draw_queue_max_items(cfg: &mut DrawQueueConfig, value: f32) {
    cfg.max_items = value.clamp(1.0_f32, 128.0_f32);
}

#[allow(dead_code)]
pub fn set_draw_queue_sort_key_bits(cfg: &mut DrawQueueConfig, value: f32) {
    cfg.sort_key_bits = value.clamp(1.0_f32, 128.0_f32);
}

#[allow(dead_code)]
pub fn set_draw_queue_depth_sort(cfg: &mut DrawQueueConfig, value: f32) {
    cfg.depth_sort = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_draw_queue_alpha_sort(cfg: &mut DrawQueueConfig, value: f32) {
    cfg.alpha_sort = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_draw_queue_instancing(cfg: &mut DrawQueueConfig, value: f32) {
    cfg.instancing = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn draw_queue_weight(cfg: &DrawQueueConfig) -> f32 {
    (cfg.max_items * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_draw_queue(a: &DrawQueueConfig, b: &DrawQueueConfig, t: f32) -> DrawQueueConfig {
    let t = t.clamp(0.0, 1.0);
    DrawQueueConfig {
        max_items: a.max_items + (b.max_items - a.max_items) * t,
        sort_key_bits: a.sort_key_bits + (b.sort_key_bits - a.sort_key_bits) * t,
        depth_sort: a.depth_sort + (b.depth_sort - a.depth_sort) * t,
        alpha_sort: a.alpha_sort + (b.alpha_sort - a.alpha_sort) * t,
        instancing: a.instancing + (b.instancing - a.instancing) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_draw_queue();
        assert!((cfg.max_items - 1024.0_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_max_items() {
        let mut cfg = default_draw_queue();
        set_draw_queue_max_items(&mut cfg, 0.7);
        assert!((cfg.max_items - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_sort_key_bits() {
        let mut cfg = default_draw_queue();
        set_draw_queue_sort_key_bits(&mut cfg, 0.8);
        assert!((cfg.sort_key_bits - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_sort() {
        let mut cfg = default_draw_queue();
        set_draw_queue_depth_sort(&mut cfg, 0.6);
        assert!((cfg.depth_sort - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_alpha_sort() {
        let mut cfg = default_draw_queue();
        set_draw_queue_alpha_sort(&mut cfg, 0.5);
        assert!((cfg.alpha_sort - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_instancing() {
        let mut cfg = default_draw_queue();
        set_draw_queue_instancing(&mut cfg, 0.4);
        assert!((cfg.instancing - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_draw_queue();
        let w = draw_queue_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_draw_queue();
        let mut b = default_draw_queue();
        b.max_items = 1.0;
        let mid = blend_draw_queue(&a, &b, 0.5);
        assert!((mid.max_items - 512.5_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_draw_queue();
        let b = default_draw_queue();
        let r = blend_draw_queue(&a, &b, 0.0);
        assert!((r.max_items - a.max_items).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_draw_queue();
        let b = default_draw_queue();
        let r = blend_draw_queue(&a, &b, 1.0);
        assert!((r.max_items - b.max_items).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_draw_queue();
        let b = default_draw_queue();
        let r = blend_draw_queue(&a, &b, 2.0);
        assert!((r.max_items - b.max_items).abs() < 1e-6);
    }
}
