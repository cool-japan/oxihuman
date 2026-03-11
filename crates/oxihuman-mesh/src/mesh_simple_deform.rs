#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Simple deform (twist/bend/taper/stretch).

#[allow(dead_code)]
pub enum SimpleDeformMode {
    Twist,
    Bend,
    Taper,
    Stretch,
}

#[allow(dead_code)]
pub struct SimpleDeformModifier {
    pub mode: SimpleDeformMode,
    pub angle: f32,
    pub factor: f32,
    pub axis: u8,
}

#[allow(dead_code)]
pub fn twist_vertex(v: [f32; 3], angle: f32, axis: u8) -> [f32; 3] {
    match axis {
        0 => {
            let t = angle * v[0];
            let (s, c) = t.sin_cos();
            [v[0], v[1] * c - v[2] * s, v[1] * s + v[2] * c]
        }
        1 => {
            let t = angle * v[1];
            let (s, c) = t.sin_cos();
            [v[0] * c + v[2] * s, v[1], -v[0] * s + v[2] * c]
        }
        _ => {
            let t = angle * v[2];
            let (s, c) = t.sin_cos();
            [v[0] * c - v[1] * s, v[0] * s + v[1] * c, v[2]]
        }
    }
}

#[allow(dead_code)]
pub fn taper_vertex(v: [f32; 3], factor: f32, axis: u8) -> [f32; 3] {
    match axis {
        0 => {
            let scale = 1.0 + factor * v[0];
            [v[0], v[1] * scale, v[2] * scale]
        }
        1 => {
            let scale = 1.0 + factor * v[1];
            [v[0] * scale, v[1], v[2] * scale]
        }
        _ => {
            let scale = 1.0 + factor * v[2];
            [v[0] * scale, v[1] * scale, v[2]]
        }
    }
}

#[allow(dead_code)]
pub fn apply_simple_deform(verts: &[[f32; 3]], sd: &SimpleDeformModifier) -> Vec<[f32; 3]> {
    verts.iter().map(|v| match sd.mode {
        SimpleDeformMode::Twist => twist_vertex(*v, sd.angle, sd.axis),
        SimpleDeformMode::Taper => taper_vertex(*v, sd.factor, sd.axis),
        SimpleDeformMode::Bend => {
            // simple bend around axis
            let t = sd.angle;
            let (s, c) = t.sin_cos();
            match sd.axis {
                0 => [v[0], v[1] * c - v[2] * s, v[1] * s + v[2] * c],
                1 => [v[0] * c + v[2] * s, v[1], -v[0] * s + v[2] * c],
                _ => [v[0] * c - v[1] * s, v[0] * s + v[1] * c, v[2]],
            }
        }
        SimpleDeformMode::Stretch => {
            let factor = sd.factor;
            match sd.axis {
                0 => [v[0] * (1.0 + factor), v[1] / (1.0 + factor).sqrt(), v[2] / (1.0 + factor).sqrt()],
                1 => [v[0] / (1.0 + factor).sqrt(), v[1] * (1.0 + factor), v[2] / (1.0 + factor).sqrt()],
                _ => [v[0] / (1.0 + factor).sqrt(), v[1] / (1.0 + factor).sqrt(), v[2] * (1.0 + factor)],
            }
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn twist_zero_angle_unchanged() {
        let v = [1.0, 1.0, 0.0];
        let r = twist_vertex(v, 0.0, 2);
        assert!((r[0] - 1.0).abs() < 1e-6);
        assert!((r[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn taper_zero_factor_unchanged() {
        let v = [1.0, 2.0, 3.0];
        let r = taper_vertex(v, 0.0, 0);
        assert!((r[0] - 1.0).abs() < 1e-6);
        assert!((r[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn twist_axis_x_rotates_yz() {
        let v = [1.0, 1.0, 0.0];
        let r = twist_vertex(v, PI / 2.0, 0);
        // at x=1, rotation = PI/2: y should rotate to z
        assert!(r[2].abs() > 0.5);
    }

    #[test]
    fn taper_axis_z_scales_xy() {
        let v = [1.0, 1.0, 1.0];
        let r = taper_vertex(v, 1.0, 2);
        // scale = 1 + 1*1 = 2
        assert!((r[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn apply_twist_preserves_count() {
        let verts = vec![[0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];
        let sd = SimpleDeformModifier { mode: SimpleDeformMode::Twist, angle: PI / 4.0, factor: 0.0, axis: 2 };
        let out = apply_simple_deform(&verts, &sd);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn apply_taper_preserves_count() {
        let verts = vec![[1.0, 1.0, 1.0]];
        let sd = SimpleDeformModifier { mode: SimpleDeformMode::Taper, angle: 0.0, factor: 0.5, axis: 1 };
        let out = apply_simple_deform(&verts, &sd);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn apply_bend_preserves_count() {
        let verts = vec![[1.0, 0.0, 0.0]];
        let sd = SimpleDeformModifier { mode: SimpleDeformMode::Bend, angle: PI / 6.0, factor: 0.0, axis: 2 };
        let out = apply_simple_deform(&verts, &sd);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn apply_stretch_preserves_count() {
        let verts = vec![[1.0, 1.0, 1.0]];
        let sd = SimpleDeformModifier { mode: SimpleDeformMode::Stretch, angle: 0.0, factor: 1.0, axis: 0 };
        let out = apply_simple_deform(&verts, &sd);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn stretch_axis_x_elongates_x() {
        let verts = vec![[1.0, 0.0, 0.0]];
        let sd = SimpleDeformModifier { mode: SimpleDeformMode::Stretch, angle: 0.0, factor: 1.0, axis: 0 };
        let out = apply_simple_deform(&verts, &sd);
        assert!(out[0][0] > 1.0);
    }

    #[test]
    fn pi_constant_correct() {
        assert!((PI - std::f32::consts::PI).abs() < 1e-6);
    }
}
