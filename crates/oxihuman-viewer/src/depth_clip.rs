// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Depth clip — near/far plane clipping with reverse-Z support.

/// Depth clip configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DepthClipConfig {
    pub near: f32,
    pub far: f32,
    pub reversed_z: bool,
}

#[allow(dead_code)]
pub fn default_depth_clip_config() -> DepthClipConfig {
    DepthClipConfig {
        near: 0.1,
        far: 1000.0,
        reversed_z: false,
    }
}

#[allow(dead_code)]
pub fn dc_set_planes(cfg: &mut DepthClipConfig, near: f32, far: f32) {
    cfg.near = near.max(1e-5);
    cfg.far = far.max(cfg.near + 0.1);
}

#[allow(dead_code)]
pub fn dc_set_reversed(cfg: &mut DepthClipConfig, reversed: bool) {
    cfg.reversed_z = reversed;
}

#[allow(dead_code)]
pub fn dc_depth_range(cfg: &DepthClipConfig) -> f32 {
    cfg.far - cfg.near
}

#[allow(dead_code)]
pub fn dc_linearize(cfg: &DepthClipConfig, ndc_depth: f32) -> f32 {
    if cfg.reversed_z {
        cfg.near / ndc_depth.max(1e-10)
    } else {
        cfg.near * cfg.far / (cfg.far - ndc_depth * (cfg.far - cfg.near))
    }
}

#[allow(dead_code)]
pub fn dc_is_in_range(cfg: &DepthClipConfig, view_depth: f32) -> bool {
    (cfg.near..=cfg.far).contains(&view_depth)
}

#[allow(dead_code)]
pub fn dc_ratio(cfg: &DepthClipConfig) -> f32 {
    cfg.far / cfg.near
}

#[allow(dead_code)]
pub fn dc_precision_bits(cfg: &DepthClipConfig) -> f32 {
    let ratio = dc_ratio(cfg);
    -ratio.log2()
}

#[allow(dead_code)]
pub fn dc_blend(a: &DepthClipConfig, b: &DepthClipConfig, t: f32) -> DepthClipConfig {
    let t = t.clamp(0.0, 1.0);
    DepthClipConfig {
        near: a.near + (b.near - a.near) * t,
        far: a.far + (b.far - a.far) * t,
        reversed_z: a.reversed_z,
    }
}

#[allow(dead_code)]
pub fn dc_to_json(cfg: &DepthClipConfig) -> String {
    format!(
        r#"{{"near":{:.4},"far":{:.4},"reversed_z":{}}}"#,
        cfg.near, cfg.far, cfg.reversed_z
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_near_far() {
        let cfg = default_depth_clip_config();
        assert!((cfg.near - 0.1).abs() < 1e-6);
        assert!((cfg.far - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn set_planes() {
        let mut cfg = default_depth_clip_config();
        dc_set_planes(&mut cfg, 0.5, 500.0);
        assert!((cfg.near - 0.5).abs() < 1e-6);
        assert!((cfg.far - 500.0).abs() < 1e-6);
    }

    #[test]
    fn set_reversed() {
        let mut cfg = default_depth_clip_config();
        dc_set_reversed(&mut cfg, true);
        assert!(cfg.reversed_z);
    }

    #[test]
    fn depth_range() {
        let cfg = default_depth_clip_config();
        assert!((dc_depth_range(&cfg) - 999.9).abs() < 0.01);
    }

    #[test]
    fn in_range_midpoint() {
        let cfg = default_depth_clip_config();
        assert!(dc_is_in_range(&cfg, 500.0));
    }

    #[test]
    fn out_of_range_below_near() {
        let cfg = default_depth_clip_config();
        assert!(!dc_is_in_range(&cfg, 0.01));
    }

    #[test]
    fn ratio_positive() {
        let cfg = default_depth_clip_config();
        assert!(dc_ratio(&cfg) > 1.0);
    }

    #[test]
    fn blend_midpoint() {
        let a = default_depth_clip_config();
        let mut b = default_depth_clip_config();
        dc_set_planes(&mut b, 1.0, 2000.0);
        let m = dc_blend(&a, &b, 0.5);
        assert!(m.near > 0.1 && m.near < 1.0);
    }

    #[test]
    fn to_json_fields() {
        let cfg = default_depth_clip_config();
        let j = dc_to_json(&cfg);
        assert!(j.contains("near"));
        assert!(j.contains("reversed_z"));
    }
}
