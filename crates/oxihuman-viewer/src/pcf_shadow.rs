// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Percentage-closer filtering (PCF) for soft shadow mapping.

use std::f32::consts::PI;

/// PCF configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct PcfConfig {
    /// Shadow map resolution.
    pub resolution: u32,
    /// Number of PCF samples.
    pub sample_count: u32,
    /// Filter radius in shadow-map texels.
    pub filter_radius: f32,
    /// Depth bias to prevent shadow acne.
    pub bias: f32,
    /// Normal-based bias scale.
    pub normal_bias: f32,
    /// Whether to use Poisson disk sampling.
    pub poisson_disk: bool,
}

impl Default for PcfConfig {
    fn default() -> Self {
        Self {
            resolution: 1024,
            sample_count: 16,
            filter_radius: 2.0,
            bias: 0.005,
            normal_bias: 0.02,
            poisson_disk: true,
        }
    }
}

/// Generate a Poisson disk sample set (deterministic).
///
/// Returns `(dx, dy)` offsets normalised to the unit disk.
#[allow(dead_code)]
pub fn generate_poisson_disk(count: u32) -> Vec<(f32, f32)> {
    let mut samples = Vec::with_capacity(count as usize);
    for i in 0..count {
        let angle = halton_seq(i, 2) * 2.0 * PI;
        let radius = halton_seq(i, 3).sqrt();
        samples.push((radius * angle.cos(), radius * angle.sin()));
    }
    samples
}

/// Halton sequence for base `b`.
#[allow(dead_code)]
pub fn halton_seq(mut index: u32, base: u32) -> f32 {
    let mut f = 1.0_f32;
    let mut r = 0.0_f32;
    let base_f = base as f32;
    while index > 0 {
        f /= base_f;
        r += f * (index % base) as f32;
        index /= base;
    }
    r
}

/// Generate a regular grid sample set.
#[allow(dead_code)]
pub fn generate_grid_samples(count: u32) -> Vec<(f32, f32)> {
    let side = (count as f32).sqrt().ceil() as u32;
    let mut samples = Vec::new();
    for y in 0..side {
        for x in 0..side {
            if samples.len() >= count as usize {
                break;
            }
            let sx = (x as f32 + 0.5) / side as f32 * 2.0 - 1.0;
            let sy = (y as f32 + 0.5) / side as f32 * 2.0 - 1.0;
            samples.push((sx, sy));
        }
    }
    samples
}

/// Compute depth bias based on surface normal and light direction.
#[allow(dead_code)]
pub fn compute_bias(n_dot_l: f32, config: &PcfConfig) -> f32 {
    let n_dot_l = n_dot_l.clamp(0.0, 1.0);
    let slope = if n_dot_l > 1e-4 {
        (1.0 - n_dot_l * n_dot_l).sqrt() / n_dot_l
    } else {
        10.0
    };
    config.bias + config.normal_bias * slope.min(10.0)
}

/// Single shadow-map depth comparison.
#[allow(dead_code)]
pub fn shadow_compare(current_depth: f32, shadow_depth: f32, bias: f32) -> f32 {
    if current_depth - bias > shadow_depth {
        0.0 // in shadow
    } else {
        1.0 // lit
    }
}

/// PCF shadow lookup with the given sample offsets.
///
/// `shadow_map` is a flat row-major depth buffer.
/// `uv` is the shadow-map coordinate, `depth` is the fragment depth.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn pcf_sample(
    shadow_map: &[f32],
    resolution: u32,
    uv: (f32, f32),
    depth: f32,
    bias: f32,
    filter_radius: f32,
    samples: &[(f32, f32)],
) -> f32 {
    if samples.is_empty() || resolution == 0 {
        return 1.0;
    }
    let texel_size = 1.0 / resolution as f32;
    let mut lit = 0.0_f32;

    for &(dx, dy) in samples {
        let su = uv.0 + dx * filter_radius * texel_size;
        let sv = uv.1 + dy * filter_radius * texel_size;

        let sx = (su * resolution as f32) as i32;
        let sy = (sv * resolution as f32) as i32;

        if sx < 0 || sy < 0 || sx >= resolution as i32 || sy >= resolution as i32 {
            lit += 1.0;
            continue;
        }

        let idx = sy as usize * resolution as usize + sx as usize;
        if idx < shadow_map.len() {
            lit += shadow_compare(depth, shadow_map[idx], bias);
        } else {
            lit += 1.0;
        }
    }

    lit / samples.len() as f32
}

/// Estimate softness from filter radius and map resolution.
#[allow(dead_code)]
pub fn estimate_softness(filter_radius: f32, resolution: u32) -> f32 {
    if resolution == 0 {
        return 0.0;
    }
    filter_radius / resolution as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_config() {
        let c = PcfConfig::default();
        assert_eq!(c.resolution, 1024);
    }

    #[test]
    fn test_halton_seq_base2() {
        let h1 = halton_seq(1, 2);
        assert!((h1 - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_halton_seq_zero() {
        assert!(halton_seq(0, 2).abs() < 1e-5);
    }

    #[test]
    fn test_poisson_disk_count() {
        let samples = generate_poisson_disk(16);
        assert_eq!(samples.len(), 16);
    }

    #[test]
    fn test_poisson_disk_within_unit() {
        let samples = generate_poisson_disk(32);
        for (x, y) in &samples {
            let r = (x * x + y * y).sqrt();
            assert!(r <= 1.001, "Sample outside unit disk: r={r}");
        }
    }

    #[test]
    fn test_grid_samples_count() {
        let samples = generate_grid_samples(9);
        assert_eq!(samples.len(), 9);
    }

    #[test]
    fn test_compute_bias_perpendicular() {
        let c = PcfConfig::default();
        let b = compute_bias(1.0, &c);
        assert!((b - c.bias).abs() < 1e-5);
    }

    #[test]
    fn test_shadow_compare_lit() {
        assert_eq!(shadow_compare(0.5, 0.6, 0.01), 1.0);
    }

    #[test]
    fn test_shadow_compare_shadowed() {
        assert_eq!(shadow_compare(0.7, 0.5, 0.01), 0.0);
    }

    #[test]
    fn test_pcf_sample_empty() {
        assert_eq!(pcf_sample(&[], 0, (0.5, 0.5), 0.5, 0.01, 1.0, &[]), 1.0);
    }

    #[test]
    fn test_estimate_softness() {
        let s = estimate_softness(2.0, 1024);
        assert!((s - 2.0 / 1024.0).abs() < 1e-6);
    }
}
