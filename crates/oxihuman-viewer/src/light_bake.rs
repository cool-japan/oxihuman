// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lightmap baking utilities: texel assignment, atlas layout, and bake-job queuing.

use std::f32::consts::PI;

/// Quality preset for the bake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BakeQuality {
    Preview,
    Medium,
    High,
    Production,
}

/// Configuration for a single lightmap bake job.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LightBakeConfig {
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub samples_per_texel: u32,
    pub quality: BakeQuality,
    /// Whether to bake indirect illumination.
    pub bake_indirect: bool,
    /// Maximum ray bounces for indirect.
    pub max_bounces: u32,
}

impl Default for LightBakeConfig {
    fn default() -> Self {
        Self {
            atlas_width: 512,
            atlas_height: 512,
            samples_per_texel: 16,
            quality: BakeQuality::Medium,
            bake_indirect: true,
            max_bounces: 2,
        }
    }
}

/// A single baked texel with irradiance value.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct BakedTexel {
    pub irradiance: [f32; 3],
    pub valid: bool,
}

/// An in-progress or completed bake job.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LightBakeJob {
    pub config: LightBakeConfig,
    pub texels: Vec<BakedTexel>,
    pub progress: f32,
    pub complete: bool,
}

/// Create a new bake job from a config.
#[allow(dead_code)]
pub fn new_bake_job(config: LightBakeConfig) -> LightBakeJob {
    let count = (config.atlas_width * config.atlas_height) as usize;
    LightBakeJob {
        config,
        texels: vec![BakedTexel::default(); count],
        progress: 0.0,
        complete: false,
    }
}

/// Compute the Lambertian irradiance for a surface hit with normal `n` and light direction `l`.
#[allow(dead_code)]
pub fn lambertian_irradiance(normal: [f32; 3], light_dir: [f32; 3], light_intensity: f32) -> f32 {
    let dot = normal[0] * light_dir[0] + normal[1] * light_dir[1] + normal[2] * light_dir[2];
    dot.max(0.0) * light_intensity / PI
}

/// Mark a texel as baked with the given irradiance.
#[allow(dead_code)]
pub fn set_texel(job: &mut LightBakeJob, x: u32, y: u32, irradiance: [f32; 3]) {
    let idx = (y * job.config.atlas_width + x) as usize;
    if idx < job.texels.len() {
        job.texels[idx].irradiance = irradiance;
        job.texels[idx].valid = true;
    }
}

/// Compute fraction of valid texels.
#[allow(dead_code)]
pub fn bake_coverage(job: &LightBakeJob) -> f32 {
    if job.texels.is_empty() {
        return 0.0;
    }
    let valid = job.texels.iter().filter(|t| t.valid).count();
    valid as f32 / job.texels.len() as f32
}

/// Average irradiance over all valid texels.
#[allow(dead_code)]
pub fn average_irradiance(job: &LightBakeJob) -> [f32; 3] {
    let valid: Vec<&BakedTexel> = job.texels.iter().filter(|t| t.valid).collect();
    if valid.is_empty() {
        return [0.0; 3];
    }
    let n = valid.len() as f32;
    let mut sum = [0.0f32; 3];
    for t in &valid {
        sum[0] += t.irradiance[0];
        sum[1] += t.irradiance[1];
        sum[2] += t.irradiance[2];
    }
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bake_job_texel_count() {
        let cfg = LightBakeConfig {
            atlas_width: 4,
            atlas_height: 4,
            ..Default::default()
        };
        let job = new_bake_job(cfg);
        assert_eq!(job.texels.len(), 16);
    }

    #[test]
    fn lambertian_perpendicular() {
        let n = [0.0_f32, 1.0, 0.0];
        let l = [0.0_f32, 1.0, 0.0];
        let irr = lambertian_irradiance(n, l, PI);
        assert!((irr - 1.0).abs() < 1e-5);
    }

    #[test]
    fn lambertian_backface_zero() {
        let n = [0.0_f32, 1.0, 0.0];
        let l = [0.0_f32, -1.0, 0.0];
        let irr = lambertian_irradiance(n, l, 1.0);
        assert_eq!(irr, 0.0);
    }

    #[test]
    fn set_texel_marks_valid() {
        let cfg = LightBakeConfig {
            atlas_width: 2,
            atlas_height: 2,
            ..Default::default()
        };
        let mut job = new_bake_job(cfg);
        set_texel(&mut job, 0, 0, [1.0, 0.0, 0.0]);
        assert!(job.texels[0].valid);
    }

    #[test]
    fn bake_coverage_zero_initially() {
        let cfg = LightBakeConfig {
            atlas_width: 2,
            atlas_height: 2,
            ..Default::default()
        };
        let job = new_bake_job(cfg);
        assert_eq!(bake_coverage(&job), 0.0);
    }

    #[test]
    fn bake_coverage_full_after_all_set() {
        let cfg = LightBakeConfig {
            atlas_width: 2,
            atlas_height: 2,
            ..Default::default()
        };
        let mut job = new_bake_job(cfg);
        for y in 0..2 {
            for x in 0..2 {
                set_texel(&mut job, x, y, [0.5, 0.5, 0.5]);
            }
        }
        assert!((bake_coverage(&job) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn average_irradiance_empty_valid() {
        let cfg = LightBakeConfig {
            atlas_width: 1,
            atlas_height: 1,
            ..Default::default()
        };
        let job = new_bake_job(cfg);
        assert_eq!(average_irradiance(&job), [0.0; 3]);
    }

    #[test]
    fn average_irradiance_single() {
        let cfg = LightBakeConfig {
            atlas_width: 1,
            atlas_height: 1,
            ..Default::default()
        };
        let mut job = new_bake_job(cfg);
        set_texel(&mut job, 0, 0, [0.3, 0.5, 0.7]);
        let avg = average_irradiance(&job);
        assert!((avg[0] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn default_config_not_preview() {
        let cfg = LightBakeConfig::default();
        assert_ne!(cfg.quality, BakeQuality::Preview);
    }
}
