// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! TAA resolve pass — final compositing, tone-curve aware blending,
//! and firefly suppression for temporal anti-aliasing.

/// TAA resolve configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TaaResolveConfig {
    /// Feedback (history blend) factor, 0..=1.
    pub feedback: f32,
    /// Firefly clamp threshold.
    pub firefly_threshold: f32,
    /// Whether to use YCoCg colour space for neighbourhood clamping.
    pub use_ycocg: bool,
    /// Motion-based weight reduction (reduce feedback for fast-moving pixels).
    pub motion_rejection: f32,
}

impl Default for TaaResolveConfig {
    fn default() -> Self {
        Self {
            feedback: 0.9,
            firefly_threshold: 5.0,
            use_ycocg: true,
            motion_rejection: 0.5,
        }
    }
}

/// Convert RGB to YCoCg colour space.
#[allow(dead_code)]
pub fn rgb_to_ycocg(rgb: [f32; 3]) -> [f32; 3] {
    let y = 0.25 * rgb[0] + 0.5 * rgb[1] + 0.25 * rgb[2];
    let co = 0.5 * rgb[0] - 0.5 * rgb[2];
    let cg = -0.25 * rgb[0] + 0.5 * rgb[1] - 0.25 * rgb[2];
    [y, co, cg]
}

/// Convert YCoCg back to RGB.
#[allow(dead_code)]
pub fn ycocg_to_rgb(ycocg: [f32; 3]) -> [f32; 3] {
    let y = ycocg[0];
    let co = ycocg[1];
    let cg = ycocg[2];
    let r = y + co - cg;
    let g = y + cg;
    let b = y - co - cg;
    [r, g, b]
}

/// Tone-curve space conversion for better blending in HDR.
///
/// Uses Reinhard operator: `x / (1 + x)`.
#[allow(dead_code)]
pub fn tonemap_reinhard(x: f32) -> f32 {
    x / (1.0 + x)
}

/// Inverse Reinhard: `x / (1 - x)`.
#[allow(dead_code)]
pub fn inverse_reinhard(x: f32) -> f32 {
    let x = x.clamp(0.0, 0.999);
    x / (1.0 - x)
}

/// Tonemap an RGB value for blending.
#[allow(dead_code)]
pub fn tonemap_rgb(rgb: [f32; 3]) -> [f32; 3] {
    [tonemap_reinhard(rgb[0]), tonemap_reinhard(rgb[1]), tonemap_reinhard(rgb[2])]
}

/// Inverse tonemap RGB.
#[allow(dead_code)]
pub fn inverse_tonemap_rgb(rgb: [f32; 3]) -> [f32; 3] {
    [inverse_reinhard(rgb[0]), inverse_reinhard(rgb[1]), inverse_reinhard(rgb[2])]
}

/// Firefly suppression: clamp maximum luminance change.
#[allow(dead_code)]
pub fn suppress_firefly(current: [f32; 3], threshold: f32) -> [f32; 3] {
    let lum = 0.2126 * current[0] + 0.7152 * current[1] + 0.0722 * current[2];
    if lum > threshold && lum > 0.0 {
        let scale = threshold / lum;
        [current[0] * scale, current[1] * scale, current[2] * scale]
    } else {
        current
    }
}

/// Compute motion-based feedback reduction.
#[allow(dead_code)]
pub fn motion_feedback_adjust(base_feedback: f32, motion_length: f32, rejection: f32) -> f32 {
    let reduction = (motion_length * rejection * 10.0).clamp(0.0, 1.0);
    (base_feedback * (1.0 - reduction)).clamp(0.0, 1.0)
}

/// Full TAA resolve for one pixel.
#[allow(dead_code)]
pub fn resolve_pixel(
    current: [f32; 3],
    history: [f32; 3],
    neighbourhood: &[[f32; 3]],
    motion_length: f32,
    config: &TaaResolveConfig,
) -> [f32; 3] {
    // Firefly suppression
    let current = suppress_firefly(current, config.firefly_threshold);

    // Neighbourhood clamp in YCoCg
    let clamped_history = if config.use_ycocg {
        let h_ycocg = rgb_to_ycocg(history);
        let n_ycocg: Vec<[f32; 3]> = neighbourhood.iter().map(|c| rgb_to_ycocg(*c)).collect();
        if n_ycocg.is_empty() {
            history
        } else {
            let mut min_c = [f32::MAX; 3];
            let mut max_c = [f32::MIN; 3];
            for c in &n_ycocg {
                for i in 0..3 {
                    min_c[i] = min_c[i].min(c[i]);
                    max_c[i] = max_c[i].max(c[i]);
                }
            }
            let clamped = [
                h_ycocg[0].clamp(min_c[0], max_c[0]),
                h_ycocg[1].clamp(min_c[1], max_c[1]),
                h_ycocg[2].clamp(min_c[2], max_c[2]),
            ];
            ycocg_to_rgb(clamped)
        }
    } else {
        history
    };

    // Motion-adjusted feedback
    let feedback = motion_feedback_adjust(config.feedback, motion_length, config.motion_rejection);

    // Blend in tone-mapped space
    let cur_tm = tonemap_rgb(current);
    let hist_tm = tonemap_rgb(clamped_history);
    let blended = [
        cur_tm[0] * (1.0 - feedback) + hist_tm[0] * feedback,
        cur_tm[1] * (1.0 - feedback) + hist_tm[1] * feedback,
        cur_tm[2] * (1.0 - feedback) + hist_tm[2] * feedback,
    ];

    inverse_tonemap_rgb(blended)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = TaaResolveConfig::default();
        assert!((0.0..=1.0).contains(&c.feedback));
    }

    #[test]
    fn test_rgb_ycocg_roundtrip() {
        let rgb = [0.5, 0.3, 0.7];
        let ycocg = rgb_to_ycocg(rgb);
        let back = ycocg_to_rgb(ycocg);
        for i in 0..3 {
            assert!((rgb[i] - back[i]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_tonemap_roundtrip() {
        let values = [0.0, 0.5, 1.0, 2.0, 5.0];
        for &v in &values {
            let tm = tonemap_reinhard(v);
            let back = inverse_reinhard(tm);
            assert!((v - back).abs() < 0.01, "Failed for {v}: got {back}");
        }
    }

    #[test]
    fn test_suppress_firefly_below() {
        let c = [1.0, 1.0, 1.0];
        let r = suppress_firefly(c, 5.0);
        assert!((r[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_suppress_firefly_above() {
        let c = [10.0, 10.0, 10.0];
        let r = suppress_firefly(c, 5.0);
        let lum = 0.2126 * r[0] + 0.7152 * r[1] + 0.0722 * r[2];
        assert!(lum <= 5.01);
    }

    #[test]
    fn test_motion_feedback_zero_motion() {
        let f = motion_feedback_adjust(0.9, 0.0, 0.5);
        assert!((f - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_motion_feedback_high_motion() {
        let f = motion_feedback_adjust(0.9, 1.0, 1.0);
        assert!(f < 0.9);
    }

    #[test]
    fn test_resolve_pixel_static() {
        let config = TaaResolveConfig::default();
        let current = [0.5, 0.5, 0.5];
        let history = [0.5, 0.5, 0.5];
        let neighbourhood = [[0.5, 0.5, 0.5]; 4];
        let r = resolve_pixel(current, history, &neighbourhood, 0.0, &config);
        for ch in &r {
            assert!((ch - 0.5).abs() < 0.05);
        }
    }

    #[test]
    fn test_resolve_pixel_empty_neighbourhood() {
        let config = TaaResolveConfig { use_ycocg: true, ..Default::default() };
        let r = resolve_pixel([0.5; 3], [0.5; 3], &[], 0.0, &config);
        assert!(r[0] > 0.0);
    }

    #[test]
    fn test_tonemap_rgb_positive() {
        let r = tonemap_rgb([1.0, 2.0, 3.0]);
        for ch in &r {
            assert!(*ch > 0.0 && *ch < 1.0);
        }
    }
}
