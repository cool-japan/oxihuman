// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cone angular joint limit enforcement.

use std::f32::consts::PI;

/// A cone limit constrains a direction vector to lie within `half_angle` of a reference axis.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ConeLimit {
    pub axis: [f32; 3],  // reference axis (must be unit length)
    pub half_angle: f32, // radians [0, PI]
    pub stiffness: f32,
    pub damping: f32,
}

#[allow(dead_code)]
impl ConeLimit {
    pub fn new(axis: [f32; 3], half_angle_deg: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            axis: normalize3(axis),
            half_angle: half_angle_deg.to_radians().clamp(0.0, PI),
            stiffness,
            damping,
        }
    }

    /// True if `dir` lies within the cone (angle to axis <= half_angle).
    pub fn is_within(&self, dir: [f32; 3]) -> bool {
        let d = normalize3(dir);
        let cos_angle = dot3(self.axis, d).clamp(-1.0, 1.0);
        cos_angle >= self.half_angle.cos()
    }

    /// Return the angle (radians) between `dir` and the cone axis.
    pub fn angle_to_axis(&self, dir: [f32; 3]) -> f32 {
        let d = normalize3(dir);
        let cos_angle = dot3(self.axis, d).clamp(-1.0, 1.0);
        cos_angle.acos()
    }

    /// Project `dir` onto cone surface if outside. Returns clamped direction.
    pub fn clamp_to_cone(&self, dir: [f32; 3]) -> [f32; 3] {
        if self.is_within(dir) {
            return dir;
        }
        // Slerp dir toward axis by excess angle
        let angle = self.angle_to_axis(dir);
        let excess = angle - self.half_angle;
        let t = excess / angle;
        let d = normalize3(dir);
        let interpolated = [
            d[0] + t * (self.axis[0] - d[0]),
            d[1] + t * (self.axis[1] - d[1]),
            d[2] + t * (self.axis[2] - d[2]),
        ];
        normalize3(interpolated)
    }

    /// Corrective torque when `dir` violates the cone limit.
    pub fn corrective_torque(&self, dir: [f32; 3]) -> [f32; 3] {
        if self.is_within(dir) {
            return [0.0; 3];
        }
        let angle = self.angle_to_axis(dir);
        let violation = angle - self.half_angle;
        let rot_axis = cross3(dir, self.axis);
        let rot_len = len3(rot_axis);
        if rot_len < 1e-9 {
            return [0.0; 3];
        }
        let n = [
            rot_axis[0] / rot_len,
            rot_axis[1] / rot_len,
            rot_axis[2] / rot_len,
        ];
        [
            n[0] * violation * self.stiffness,
            n[1] * violation * self.stiffness,
            n[2] * violation * self.stiffness,
        ]
    }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-9 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

/// Solid angle of a cone with given half-angle (in steradians).
#[allow(dead_code)]
pub fn cone_solid_angle(half_angle_rad: f32) -> f32 {
    2.0 * PI * (1.0 - half_angle_rad.cos())
}

/// True if angle is within limit (degrees helper).
#[allow(dead_code)]
pub fn angle_within_deg(angle_deg: f32, limit_deg: f32) -> bool {
    angle_deg.abs() <= limit_deg.abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_direction_within_cone() {
        let cl = ConeLimit::new([0.0, 1.0, 0.0], 45.0, 1.0, 0.0);
        assert!(cl.is_within([0.0, 1.0, 0.0]));
    }

    #[test]
    fn perpendicular_outside_45deg_cone() {
        let cl = ConeLimit::new([0.0, 1.0, 0.0], 45.0, 1.0, 0.0);
        assert!(!cl.is_within([1.0, 0.0, 0.0]));
    }

    #[test]
    fn angle_to_axis_zero_for_same_direction() {
        let cl = ConeLimit::new([0.0, 1.0, 0.0], 30.0, 1.0, 0.0);
        assert!(cl.angle_to_axis([0.0, 1.0, 0.0]) < 1e-5);
    }

    #[test]
    fn angle_to_axis_90_for_perpendicular() {
        let cl = ConeLimit::new([0.0, 1.0, 0.0], 30.0, 1.0, 0.0);
        let angle = cl.angle_to_axis([1.0, 0.0, 0.0]);
        assert!((angle - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn clamp_to_cone_no_change_inside() {
        let cl = ConeLimit::new([0.0, 1.0, 0.0], 60.0, 1.0, 0.0);
        let d = normalize3([0.3, 0.9, 0.0]);
        let clamped = cl.clamp_to_cone(d);
        // Should be within cone after clamping
        assert!(cl.is_within(clamped));
    }

    #[test]
    fn clamp_to_cone_moves_outside_in() {
        let cl = ConeLimit::new([0.0, 1.0, 0.0], 10.0, 1.0, 0.0);
        let clamped = cl.clamp_to_cone([1.0, 0.0, 0.0]);
        assert!(cl.is_within(clamped));
    }

    #[test]
    fn corrective_torque_zero_inside_cone() {
        let cl = ConeLimit::new([0.0, 1.0, 0.0], 90.0, 10.0, 0.0);
        let t = cl.corrective_torque([0.0, 1.0, 0.0]);
        assert_eq!(t, [0.0; 3]);
    }

    #[test]
    fn corrective_torque_nonzero_outside() {
        let cl = ConeLimit::new([0.0, 1.0, 0.0], 10.0, 10.0, 0.0);
        let t = cl.corrective_torque([1.0, 0.0, 0.0]);
        let mag = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
        assert!(mag > 0.0);
    }

    #[test]
    fn cone_solid_angle_half_sphere() {
        // Half-angle = PI/2 should give 2*PI steradians
        let sa = cone_solid_angle(PI / 2.0);
        assert!((sa - 2.0 * PI).abs() < 1e-4);
    }

    #[test]
    fn angle_within_deg_basic() {
        assert!(angle_within_deg(30.0, 45.0));
        assert!(!angle_within_deg(50.0, 45.0));
    }
}
