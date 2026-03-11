// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! BodyScale — uniform and axis-specific body scaling.

#![allow(dead_code)]

/// Which axis to scale.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleAxis {
    X,
    Y,
    Z,
    All,
}

/// Per-axis scale factors for a body mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyScale {
    pub sx: f32,
    pub sy: f32,
    pub sz: f32,
}

impl Default for BodyScale {
    fn default() -> Self {
        BodyScale { sx: 1.0, sy: 1.0, sz: 1.0 }
    }
}

/// Create a default `BodyScale` (all axes = 1.0).
#[allow(dead_code)]
pub fn new_body_scale() -> BodyScale {
    BodyScale::default()
}

/// Apply `scale` to a flat position array `[x,y,z,...]` in-place.
#[allow(dead_code)]
pub fn apply_body_scale(scale: &BodyScale, positions: &mut [f32]) {
    let mut i = 0;
    while i + 2 < positions.len() {
        positions[i] *= scale.sx;
        positions[i + 1] *= scale.sy;
        positions[i + 2] *= scale.sz;
        i += 3;
    }
}

/// Set all axes to the same scale factor.
#[allow(dead_code)]
pub fn uniform_scale(scale: &mut BodyScale, s: f32) {
    scale.sx = s;
    scale.sy = s;
    scale.sz = s;
}

/// Set the X-axis scale.
#[allow(dead_code)]
pub fn scale_x(scale: &mut BodyScale, sx: f32) {
    scale.sx = sx;
}

/// Set the Y-axis scale.
#[allow(dead_code)]
pub fn scale_y(scale: &mut BodyScale, sy: f32) {
    scale.sy = sy;
}

/// Set the Z-axis scale.
#[allow(dead_code)]
pub fn scale_z(scale: &mut BodyScale, sz: f32) {
    scale.sz = sz;
}

/// Apply a separate limb scale factor on X and Z (width/depth of limbs).
#[allow(dead_code)]
pub fn scale_limbs(scale: &mut BodyScale, factor: f32) {
    scale.sx *= factor;
    scale.sz *= factor;
}

/// Apply a torso-specific Y scale (height stretch).
#[allow(dead_code)]
pub fn scale_torso(scale: &mut BodyScale, factor: f32) {
    scale.sy *= factor;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_scale_is_one() {
        let s = new_body_scale();
        assert_eq!(s.sx, 1.0);
        assert_eq!(s.sy, 1.0);
        assert_eq!(s.sz, 1.0);
    }

    #[test]
    fn test_uniform_scale() {
        let mut s = new_body_scale();
        uniform_scale(&mut s, 2.0);
        assert_eq!(s.sx, 2.0);
        assert_eq!(s.sy, 2.0);
        assert_eq!(s.sz, 2.0);
    }

    #[test]
    fn test_scale_x_only() {
        let mut s = new_body_scale();
        scale_x(&mut s, 3.0);
        assert_eq!(s.sx, 3.0);
        assert_eq!(s.sy, 1.0);
    }

    #[test]
    fn test_scale_y_only() {
        let mut s = new_body_scale();
        scale_y(&mut s, 1.5);
        assert_eq!(s.sy, 1.5);
        assert_eq!(s.sx, 1.0);
    }

    #[test]
    fn test_scale_z_only() {
        let mut s = new_body_scale();
        scale_z(&mut s, 0.5);
        assert_eq!(s.sz, 0.5);
    }

    #[test]
    fn test_apply_body_scale() {
        let s = BodyScale { sx: 2.0, sy: 3.0, sz: 4.0 };
        let mut pos = vec![1.0_f32, 1.0, 1.0];
        apply_body_scale(&s, &mut pos);
        assert!((pos[0] - 2.0).abs() < 1e-6);
        assert!((pos[1] - 3.0).abs() < 1e-6);
        assert!((pos[2] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale_limbs_multiplies_xz() {
        let mut s = new_body_scale();
        scale_limbs(&mut s, 2.0);
        assert!((s.sx - 2.0).abs() < 1e-6);
        assert!((s.sy - 1.0).abs() < 1e-6);
        assert!((s.sz - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale_torso_multiplies_y() {
        let mut s = new_body_scale();
        scale_torso(&mut s, 1.5);
        assert!((s.sy - 1.5).abs() < 1e-6);
        assert!((s.sx - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale_axis_enum() {
        assert_eq!(ScaleAxis::X, ScaleAxis::X);
        assert_ne!(ScaleAxis::Y, ScaleAxis::Z);
    }
}
