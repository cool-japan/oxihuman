// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sub-pixel jitter sample patterns for temporal anti-aliasing.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum JitterPattern {
    Halton,
    Uniform,
    BlueNoise,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JitterConfig {
    pub pattern: JitterPattern,
    pub sample_count: u32,
    pub jitter_scale: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct JitterOffset {
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
pub fn default_jitter_config() -> JitterConfig {
    JitterConfig {
        pattern: JitterPattern::Halton,
        sample_count: 16,
        jitter_scale: 1.0,
    }
}

#[allow(dead_code)]
pub fn halton_sequence(index: u32, base: u32) -> f32 {
    let mut result = 0.0f32;
    let mut f = 1.0 / base as f32;
    let mut i = index;
    while i > 0 {
        result += f * (i % base) as f32;
        i /= base;
        f /= base as f32;
    }
    result
}

#[allow(dead_code)]
pub fn generate_halton_jitter(count: u32) -> Vec<JitterOffset> {
    (1..=count)
        .map(|i| JitterOffset {
            x: halton_sequence(i, 2) - 0.5,
            y: halton_sequence(i, 3) - 0.5,
        })
        .collect()
}

#[allow(dead_code)]
pub fn generate_uniform_jitter(count: u32) -> Vec<JitterOffset> {
    let side = (count as f32).sqrt().ceil() as u32;
    let step = 1.0 / side as f32;
    let mut offsets = Vec::with_capacity(count as usize);
    for j in 0..side {
        for i in 0..side {
            if offsets.len() >= count as usize {
                break;
            }
            offsets.push(JitterOffset {
                x: (i as f32 + 0.5) * step - 0.5,
                y: (j as f32 + 0.5) * step - 0.5,
            });
        }
    }
    offsets
}

#[allow(dead_code)]
pub fn generate_jitter(cfg: &JitterConfig) -> Vec<JitterOffset> {
    let base = match cfg.pattern {
        JitterPattern::Halton => generate_halton_jitter(cfg.sample_count),
        JitterPattern::Uniform | JitterPattern::BlueNoise => generate_uniform_jitter(cfg.sample_count),
    };
    base.into_iter()
        .map(|j| JitterOffset {
            x: j.x * cfg.jitter_scale,
            y: j.y * cfg.jitter_scale,
        })
        .collect()
}

#[allow(dead_code)]
pub fn jitter_for_frame(offsets: &[JitterOffset], frame: u32) -> JitterOffset {
    if offsets.is_empty() {
        return JitterOffset { x: 0.0, y: 0.0 };
    }
    offsets[(frame as usize) % offsets.len()]
}

#[allow(dead_code)]
pub fn apply_jitter_to_projection(proj: &mut [[f32; 4]; 4], jitter: JitterOffset, width: f32, height: f32) {
    let _ = PI;
    if width > 0.0 && height > 0.0 {
        proj[2][0] += jitter.x * 2.0 / width;
        proj[2][1] += jitter.y * 2.0 / height;
    }
}

#[allow(dead_code)]
pub fn jitter_to_json(cfg: &JitterConfig) -> String {
    let p = match &cfg.pattern {
        JitterPattern::Halton => "halton",
        JitterPattern::Uniform => "uniform",
        JitterPattern::BlueNoise => "blue_noise",
    };
    format!(
        r#"{{"pattern":"{}","samples":{},"scale":{}}}"#,
        p, cfg.sample_count, cfg.jitter_scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_jitter_config();
        assert_eq!(c.pattern, JitterPattern::Halton);
        assert_eq!(c.sample_count, 16);
    }

    #[test]
    fn test_halton_base2() {
        let v = halton_sequence(1, 2);
        assert!((v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_halton_base3() {
        let v = halton_sequence(1, 3);
        assert!((v - 1.0 / 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_halton_range() {
        for i in 1..100 {
            let v = halton_sequence(i, 2);
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn test_generate_halton() {
        let offsets = generate_halton_jitter(8);
        assert_eq!(offsets.len(), 8);
    }

    #[test]
    fn test_generate_uniform() {
        let offsets = generate_uniform_jitter(4);
        assert_eq!(offsets.len(), 4);
    }

    #[test]
    fn test_jitter_for_frame() {
        let offsets = generate_halton_jitter(4);
        let j0 = jitter_for_frame(&offsets, 0);
        let j4 = jitter_for_frame(&offsets, 4);
        assert!((j0.x - j4.x).abs() < 1e-6);
    }

    #[test]
    fn test_jitter_empty() {
        let j = jitter_for_frame(&[], 0);
        assert!(j.x.abs() < 1e-6);
    }

    #[test]
    fn test_generate_with_scale() {
        let mut cfg = default_jitter_config();
        cfg.jitter_scale = 0.5;
        cfg.sample_count = 4;
        let offsets = generate_jitter(&cfg);
        assert!(offsets.iter().all(|o| o.x.abs() <= 0.5));
    }

    #[test]
    fn test_to_json() {
        let c = default_jitter_config();
        let j = jitter_to_json(&c);
        assert!(j.contains("halton"));
    }
}
