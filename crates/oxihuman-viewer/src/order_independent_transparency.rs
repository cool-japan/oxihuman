// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Order-independent transparency (OIT) — weighted blended OIT
//! implementation using McGuire and Bavoil's method.

/// OIT configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct OitConfig {
    /// Weight function exponent (controls depth sensitivity).
    pub weight_exponent: f32,
    /// Near plane distance for depth scaling.
    pub near_plane: f32,
    /// Far plane distance for depth scaling.
    pub far_plane: f32,
    /// Maximum number of transparent layers.
    pub max_layers: u32,
}

impl Default for OitConfig {
    fn default() -> Self {
        Self {
            weight_exponent: 3.0,
            near_plane: 0.1,
            far_plane: 100.0,
            max_layers: 8,
        }
    }
}

/// Accumulation buffer entry (per pixel).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub struct OitAccum {
    /// Weighted colour sum [r, g, b, a].
    pub color_sum: [f32; 4],
    /// Reveal term (product of (1 - alpha * weight)).
    pub reveal: f32,
}

/// Fragment to be composited.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct TransparentFragment {
    pub color: [f32; 3],
    pub alpha: f32,
    pub depth: f32,
}

/// Compute weight for a transparent fragment (McGuire-Bavoil).
///
/// Higher weight for closer fragments.
#[allow(dead_code)]
pub fn compute_weight(alpha: f32, depth: f32, config: &OitConfig) -> f32 {
    let depth = depth.clamp(config.near_plane, config.far_plane);
    let z = (config.far_plane - depth) / (config.far_plane - config.near_plane);
    let z_clamped = z.clamp(1e-5, 1.0);
    alpha * z_clamped.powf(config.weight_exponent).max(1e-4).min(3e4)
}

/// Accumulate a fragment into the OIT buffer.
#[allow(dead_code)]
pub fn accumulate_fragment(
    accum: &mut OitAccum,
    frag: &TransparentFragment,
    config: &OitConfig,
) {
    let w = compute_weight(frag.alpha, frag.depth, config);
    accum.color_sum[0] += frag.color[0] * frag.alpha * w;
    accum.color_sum[1] += frag.color[1] * frag.alpha * w;
    accum.color_sum[2] += frag.color[2] * frag.alpha * w;
    accum.color_sum[3] += frag.alpha * w;
    accum.reveal *= 1.0 - frag.alpha;
}

/// Resolve the final composite colour from the accumulation buffer.
///
/// `bg_color` is the opaque background colour.
#[allow(dead_code)]
pub fn resolve_oit(accum: &OitAccum, bg_color: [f32; 3]) -> [f32; 3] {
    if accum.color_sum[3].abs() < 1e-6 {
        return bg_color;
    }
    let avg_color = [
        accum.color_sum[0] / accum.color_sum[3],
        accum.color_sum[1] / accum.color_sum[3],
        accum.color_sum[2] / accum.color_sum[3],
    ];
    let coverage = 1.0 - accum.reveal;
    [
        avg_color[0] * coverage + bg_color[0] * accum.reveal,
        avg_color[1] * coverage + bg_color[1] * accum.reveal,
        avg_color[2] * coverage + bg_color[2] * accum.reveal,
    ]
}

/// Linearize depth from NDC (-1..1 or 0..1) to view space.
#[allow(dead_code)]
pub fn linearize_depth(ndc_z: f32, near: f32, far: f32) -> f32 {
    if (far - near).abs() < 1e-6 {
        return near;
    }
    (2.0 * near * far) / (far + near - ndc_z * (far - near))
}

/// Depth-sorted compositing (front-to-back alpha blending).
#[allow(dead_code)]
pub fn composite_sorted(fragments: &mut [TransparentFragment], bg: [f32; 3]) -> [f32; 3] {
    fragments.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal));

    let mut result = bg;
    let mut remaining_alpha = 1.0_f32;

    for frag in fragments.iter() {
        if remaining_alpha < 1e-4 {
            break;
        }
        let contrib = frag.alpha * remaining_alpha;
        result[0] = result[0] * (1.0 - contrib) + frag.color[0] * contrib;
        result[1] = result[1] * (1.0 - contrib) + frag.color[1] * contrib;
        result[2] = result[2] * (1.0 - contrib) + frag.color[2] * contrib;
        remaining_alpha *= 1.0 - frag.alpha;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = OitConfig::default();
        assert!(c.far_plane > c.near_plane);
    }

    #[test]
    fn test_compute_weight_positive() {
        let c = OitConfig::default();
        let w = compute_weight(0.5, 1.0, &c);
        assert!(w > 0.0);
    }

    #[test]
    fn test_compute_weight_closer_heavier() {
        let c = OitConfig::default();
        let w_close = compute_weight(0.5, 1.0, &c);
        let w_far = compute_weight(0.5, 50.0, &c);
        assert!(w_close > w_far);
    }

    #[test]
    fn test_accumulate_and_resolve_single() {
        let c = OitConfig::default();
        let mut accum = OitAccum { color_sum: [0.0; 4], reveal: 1.0 };
        let frag = TransparentFragment { color: [1.0, 0.0, 0.0], alpha: 0.5, depth: 5.0 };
        accumulate_fragment(&mut accum, &frag, &c);
        let result = resolve_oit(&accum, [0.0, 0.0, 0.0]);
        assert!(result[0] > 0.0);
    }

    #[test]
    fn test_resolve_empty() {
        let accum = OitAccum { color_sum: [0.0; 4], reveal: 1.0 };
        let bg = [0.5, 0.5, 0.5];
        let result = resolve_oit(&accum, bg);
        assert!((result[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_linearize_depth_near() {
        let d = linearize_depth(-1.0, 0.1, 100.0);
        assert!((d - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_linearize_depth_equal_planes() {
        let d = linearize_depth(0.0, 1.0, 1.0);
        assert!((d - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_composite_sorted_empty() {
        let result = composite_sorted(&mut [], [0.5, 0.5, 0.5]);
        assert!((result[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_composite_sorted_opaque() {
        let mut frags = vec![
            TransparentFragment { color: [1.0, 0.0, 0.0], alpha: 1.0, depth: 1.0 },
        ];
        let result = composite_sorted(&mut frags, [0.0, 1.0, 0.0]);
        assert!(result[0] > 0.9);
    }

    #[test]
    fn test_composite_sorted_order() {
        let mut frags = vec![
            TransparentFragment { color: [1.0, 0.0, 0.0], alpha: 0.5, depth: 2.0 },
            TransparentFragment { color: [0.0, 0.0, 1.0], alpha: 0.5, depth: 1.0 },
        ];
        let result = composite_sorted(&mut frags, [0.0, 0.0, 0.0]);
        // Blue is closer, should dominate
        assert!(result[2] > result[0]);
    }
}
