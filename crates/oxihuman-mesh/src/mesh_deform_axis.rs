// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Axis-aligned deformation utilities: twist, taper, stretch, shear.

use std::f32::consts::PI;

/// Which axis to deform along.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeformAxis {
    X,
    Y,
    Z,
}

/// Config for axis deformation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AxisDeformConfig {
    pub axis: DeformAxis,
    pub amount: f32,
    pub min_extent: f32,
    pub max_extent: f32,
}

#[allow(dead_code)]
pub fn default_axis_deform_config() -> AxisDeformConfig {
    AxisDeformConfig {
        axis: DeformAxis::Y,
        amount: 0.5,
        min_extent: -1.0,
        max_extent: 1.0,
    }
}

fn axis_value(p: [f32; 3], axis: DeformAxis) -> f32 {
    match axis {
        DeformAxis::X => p[0],
        DeformAxis::Y => p[1],
        DeformAxis::Z => p[2],
    }
}

fn axis_t(p: [f32; 3], cfg: &AxisDeformConfig) -> f32 {
    let span = cfg.max_extent - cfg.min_extent;
    if span.abs() < 1e-10 {
        return 0.0;
    }
    ((axis_value(p, cfg.axis) - cfg.min_extent) / span).clamp(0.0, 1.0)
}

/// Twist deformation: rotate vertices around the axis by amount * t radians.
#[allow(dead_code)]
pub fn twist_deform(positions: &[[f32; 3]], cfg: &AxisDeformConfig) -> Vec<[f32; 3]> {
    positions
        .iter()
        .map(|&p| {
            let t = axis_t(p, cfg);
            let angle = cfg.amount * t * PI;
            let (sin_a, cos_a) = (angle.sin(), angle.cos());
            match cfg.axis {
                DeformAxis::Y => [
                    p[0] * cos_a - p[2] * sin_a,
                    p[1],
                    p[0] * sin_a + p[2] * cos_a,
                ],
                DeformAxis::X => [
                    p[0],
                    p[1] * cos_a - p[2] * sin_a,
                    p[1] * sin_a + p[2] * cos_a,
                ],
                DeformAxis::Z => [
                    p[0] * cos_a - p[1] * sin_a,
                    p[0] * sin_a + p[1] * cos_a,
                    p[2],
                ],
            }
        })
        .collect()
}

/// Taper deformation: scale perpendicular axes by (1 - amount * t).
#[allow(dead_code)]
pub fn taper_deform(positions: &[[f32; 3]], cfg: &AxisDeformConfig) -> Vec<[f32; 3]> {
    positions
        .iter()
        .map(|&p| {
            let t = axis_t(p, cfg);
            let scale = (1.0 - cfg.amount * t).max(0.0);
            match cfg.axis {
                DeformAxis::Y => [p[0] * scale, p[1], p[2] * scale],
                DeformAxis::X => [p[0], p[1] * scale, p[2] * scale],
                DeformAxis::Z => [p[0] * scale, p[1] * scale, p[2]],
            }
        })
        .collect()
}

/// Stretch: elongate along axis and squash perpendicular by sqrt-reciprocal.
#[allow(dead_code)]
pub fn stretch_deform(positions: &[[f32; 3]], cfg: &AxisDeformConfig) -> Vec<[f32; 3]> {
    let s = 1.0 + cfg.amount;
    let inv = if s > 1e-6 { 1.0 / s.sqrt() } else { 1.0 };
    positions
        .iter()
        .map(|&p| match cfg.axis {
            DeformAxis::Y => [p[0] * inv, p[1] * s, p[2] * inv],
            DeformAxis::X => [p[0] * s, p[1] * inv, p[2] * inv],
            DeformAxis::Z => [p[0] * inv, p[1] * inv, p[2] * s],
        })
        .collect()
}

/// Vertex count passthrough.
#[allow(dead_code)]
pub fn deform_vertex_count(pos: &[[f32; 3]]) -> usize {
    pos.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twist_preserves_count() {
        let pos = vec![[0.0f32; 3]; 5];
        let cfg = default_axis_deform_config();
        assert_eq!(twist_deform(&pos, &cfg).len(), 5);
    }

    #[test]
    fn taper_preserves_count() {
        let pos = vec![[1.0f32, 0.5, 0.0]];
        let cfg = default_axis_deform_config();
        assert_eq!(taper_deform(&pos, &cfg).len(), 1);
    }

    #[test]
    fn stretch_preserves_count() {
        let pos = vec![[0.0f32; 3]; 3];
        let cfg = default_axis_deform_config();
        assert_eq!(stretch_deform(&pos, &cfg).len(), 3);
    }

    #[test]
    fn twist_no_rotation_at_zero_amount() {
        let pos = vec![[1.0f32, 0.0, 0.0]];
        let cfg = AxisDeformConfig {
            axis: DeformAxis::Y,
            amount: 0.0,
            min_extent: -1.0,
            max_extent: 1.0,
        };
        let out = twist_deform(&pos, &cfg);
        assert!((out[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn taper_no_scale_at_zero_amount() {
        let pos = vec![[1.0f32, 0.0, 0.0]];
        let cfg = AxisDeformConfig {
            axis: DeformAxis::Y,
            amount: 0.0,
            min_extent: -1.0,
            max_extent: 1.0,
        };
        let out = taper_deform(&pos, &cfg);
        assert!((out[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn stretch_elongates_axis() {
        let pos = vec![[0.0f32, 1.0, 0.0]];
        let cfg = AxisDeformConfig {
            axis: DeformAxis::Y,
            amount: 1.0,
            min_extent: -2.0,
            max_extent: 2.0,
        };
        let out = stretch_deform(&pos, &cfg);
        assert!(out[0][1] > 1.0);
    }

    #[test]
    fn default_config_axis_is_y() {
        let cfg = default_axis_deform_config();
        assert_eq!(cfg.axis, DeformAxis::Y);
    }

    #[test]
    fn deform_vertex_count_correct() {
        let pos = vec![[0.0f32; 3]; 7];
        assert_eq!(deform_vertex_count(&pos), 7);
    }

    #[test]
    fn pi_from_consts() {
        let v = PI * 1.0_f32;
        assert!(v > 3.0);
    }

    #[test]
    fn contains_range() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
