// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Screen-space reflections (SSR) — ray marching through the depth buffer
//! for real-time reflection approximation.

use std::f32::consts::PI;

/// SSR configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SsrConfig {
    /// Maximum number of ray march steps.
    pub max_steps: u32,
    /// Step size in view space.
    pub step_size: f32,
    /// Maximum ray distance.
    pub max_distance: f32,
    /// Thickness threshold for depth comparison.
    pub thickness: f32,
    /// Edge fade distance (screen-space fraction).
    pub edge_fade: f32,
    /// Roughness cutoff (no SSR above this).
    pub roughness_cutoff: f32,
}

impl Default for SsrConfig {
    fn default() -> Self {
        Self {
            max_steps: 64,
            step_size: 0.1,
            max_distance: 50.0,
            thickness: 0.1,
            edge_fade: 0.1,
            roughness_cutoff: 0.5,
        }
    }
}

/// SSR hit result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SsrHit {
    /// Screen UV of the reflected point.
    pub uv: (f32, f32),
    /// Confidence / fade factor 0..=1.
    pub confidence: f32,
    /// Number of steps taken.
    pub steps_taken: u32,
}

/// Reflect a direction around a normal.
#[allow(dead_code)]
pub fn reflect(incident: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    let d = 2.0 * dot(incident, normal);
    [
        incident[0] - d * normal[0],
        incident[1] - d * normal[1],
        incident[2] - d * normal[2],
    ]
}

/// Edge fade: reduce confidence near screen borders.
#[allow(dead_code)]
pub fn screen_edge_fade(uv: (f32, f32), edge: f32) -> f32 {
    if edge <= 0.0 {
        return 1.0;
    }
    let fx = if uv.0 < edge { uv.0 / edge }
    else if uv.0 > 1.0 - edge { (1.0 - uv.0) / edge }
    else { 1.0 };
    let fy = if uv.1 < edge { uv.1 / edge }
    else if uv.1 > 1.0 - edge { (1.0 - uv.1) / edge }
    else { 1.0 };
    (fx.min(fy)).clamp(0.0, 1.0)
}

/// Distance fade: reduce confidence for far reflections.
#[allow(dead_code)]
pub fn distance_fade(distance: f32, max_distance: f32) -> f32 {
    if max_distance <= 0.0 {
        return 0.0;
    }
    (1.0 - distance / max_distance).clamp(0.0, 1.0)
}

/// Roughness-based fade.
#[allow(dead_code)]
pub fn roughness_fade(roughness: f32, cutoff: f32) -> f32 {
    if cutoff <= 0.0 {
        return 0.0;
    }
    (1.0 - roughness / cutoff).clamp(0.0, 1.0)
}

/// Simple linear ray march through a depth buffer.
///
/// `view_pos`: fragment position in view space.
/// `view_dir`: reflected direction in view space (normalised).
/// `depth_buffer`: row-major depth buffer (linearised view-space Z).
/// `width`, `height`: buffer dimensions.
/// `projection`: simplified projection `(fx, fy, cx, cy)` for view-to-screen.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn ray_march(
    view_pos: [f32; 3],
    view_dir: [f32; 3],
    depth_buffer: &[f32],
    width: u32,
    height: u32,
    projection: (f32, f32, f32, f32),
    config: &SsrConfig,
) -> Option<SsrHit> {
    let (fx, fy, cx, cy) = projection;
    if width == 0 || height == 0 {
        return None;
    }

    let mut pos = view_pos;
    for step in 0..config.max_steps {
        pos[0] += view_dir[0] * config.step_size;
        pos[1] += view_dir[1] * config.step_size;
        pos[2] += view_dir[2] * config.step_size;

        if pos[2] <= 0.0 {
            return None;
        }

        let screen_x = fx * pos[0] / pos[2] + cx;
        let screen_y = fy * pos[1] / pos[2] + cy;

        let u = screen_x / width as f32;
        let v = screen_y / height as f32;

        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
            return None;
        }

        let px = (screen_x as usize).min(width as usize - 1);
        let py = (screen_y as usize).min(height as usize - 1);
        let idx = py * width as usize + px;

        if idx >= depth_buffer.len() {
            return None;
        }

        let buffer_depth = depth_buffer[idx];
        let depth_diff = pos[2] - buffer_depth;

        if depth_diff > 0.0 && depth_diff < config.thickness {
            let dist = ((pos[0] - view_pos[0]).powi(2) + (pos[1] - view_pos[1]).powi(2) + (pos[2] - view_pos[2]).powi(2)).sqrt();
            let edge = screen_edge_fade((u, v), config.edge_fade);
            let dist_f = distance_fade(dist, config.max_distance);
            let confidence = edge * dist_f;

            return Some(SsrHit {
                uv: (u, v),
                confidence,
                steps_taken: step + 1,
            });
        }
    }
    None
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = SsrConfig::default();
        assert_eq!(c.max_steps, 64);
    }

    #[test]
    fn test_reflect_perpendicular() {
        let r = reflect([0.0, -1.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((r[0]).abs() < 1e-5);
        assert!((r[1] - 1.0).abs() < 1e-5);
        assert!((r[2]).abs() < 1e-5);
    }

    #[test]
    fn test_reflect_parallel() {
        let r = reflect([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((r[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_screen_edge_fade_centre() {
        let f = screen_edge_fade((0.5, 0.5), 0.1);
        assert!((f - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_screen_edge_fade_border() {
        let f = screen_edge_fade((0.0, 0.5), 0.1);
        assert!(f.abs() < 1e-5);
    }

    #[test]
    fn test_distance_fade_zero() {
        let f = distance_fade(0.0, 50.0);
        assert!((f - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_distance_fade_max() {
        let f = distance_fade(50.0, 50.0);
        assert!(f.abs() < 1e-5);
    }

    #[test]
    fn test_roughness_fade() {
        let f = roughness_fade(0.0, 0.5);
        assert!((f - 1.0).abs() < 1e-5);
        let f2 = roughness_fade(0.5, 0.5);
        assert!(f2.abs() < 1e-5);
    }

    #[test]
    fn test_ray_march_empty_buffer() {
        let r = ray_march([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], &[], 0, 0,
            (1.0, 1.0, 0.0, 0.0), &SsrConfig::default());
        assert!(r.is_none());
    }

    #[test]
    fn test_ray_march_behind_camera() {
        let r = ray_march([0.0, 0.0, 0.1], [0.0, 0.0, -1.0], &[1.0; 100], 10, 10,
            (10.0, 10.0, 5.0, 5.0), &SsrConfig { max_steps: 4, step_size: 0.2, ..Default::default() });
        assert!(r.is_none());
    }
}
