// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Slider (prismatic) constraint: restricts motion to a single axis.

/// A slider constraint restricting linear motion along an axis.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintSliderDef {
    pub body_a: u32,
    pub body_b: u32,
    pub axis: [f32; 3],
    pub min_dist: f32,
    pub max_dist: f32,
    pub stiffness: f32,
    pub damping: f32,
}

#[allow(dead_code)]
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0]*b[0] + a[1]*b[1] + a[2]*b[2]
}

#[allow(dead_code)]
fn norm3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt();
    if l < 1e-10 { [1.0,0.0,0.0] } else { [v[0]/l,v[1]/l,v[2]/l] }
}

#[allow(dead_code)]
impl ConstraintSliderDef {
    pub fn new(body_a: u32, body_b: u32, axis: [f32; 3], min: f32, max: f32) -> Self {
        Self {
            body_a, body_b,
            axis: norm3(axis),
            min_dist: min, max_dist: max,
            stiffness: 1.0, damping: 0.0,
        }
    }

    pub fn with_stiffness(mut self, s: f32) -> Self { self.stiffness = s; self }
    pub fn with_damping(mut self, d: f32) -> Self { self.damping = d; self }

    /// Distance along the slider axis between two positions.
    pub fn slide_distance(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
        let diff = [pos_b[0]-pos_a[0], pos_b[1]-pos_a[1], pos_b[2]-pos_a[2]];
        dot(diff, self.axis)
    }

    /// How far out of limits the slider is. 0 = in range.
    pub fn limit_error(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
        let d = self.slide_distance(pos_a, pos_b);
        if d < self.min_dist {
            d - self.min_dist
        } else if d > self.max_dist {
            d - self.max_dist
        } else {
            0.0
        }
    }

    pub fn in_limits(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> bool {
        let d = self.slide_distance(pos_a, pos_b);
        d >= self.min_dist && d <= self.max_dist
    }

    /// Lateral (off-axis) error: distance from B to the slider line.
    pub fn lateral_error(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
        let diff = [pos_b[0]-pos_a[0], pos_b[1]-pos_a[1], pos_b[2]-pos_a[2]];
        let along = dot(diff, self.axis);
        let proj = [
            diff[0] - along * self.axis[0],
            diff[1] - along * self.axis[1],
            diff[2] - along * self.axis[2],
        ];
        (proj[0]*proj[0] + proj[1]*proj[1] + proj[2]*proj[2]).sqrt()
    }

    /// Correction force along axis for limit violation.
    pub fn correction_force(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
        -self.stiffness * self.limit_error(pos_a, pos_b)
    }

    pub fn damping_force(&self, vel_along_axis: f32) -> f32 {
        -self.damping * vel_along_axis
    }

    pub fn travel_range(&self) -> f32 {
        self.max_dist - self.min_dist
    }

    /// Normalized position within limits [0, 1].
    pub fn normalized_position(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
        let d = self.slide_distance(pos_a, pos_b);
        let range = self.travel_range();
        if range < 1e-10 { return 0.5; }
        ((d - self.min_dist) / range).clamp(0.0, 1.0)
    }

    /// Energy from limit violation.
    pub fn energy(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
        let e = self.limit_error(pos_a, pos_b);
        0.5 * self.stiffness * e * e
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slide_distance() {
        let c = ConstraintSliderDef::new(0, 1, [1.0,0.0,0.0], -1.0, 1.0);
        assert!((c.slide_distance([0.0;3], [3.0,0.0,0.0]) - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_in_limits() {
        let c = ConstraintSliderDef::new(0, 1, [1.0,0.0,0.0], -2.0, 2.0);
        assert!(c.in_limits([0.0;3], [1.0,0.0,0.0]));
        assert!(!c.in_limits([0.0;3], [5.0,0.0,0.0]));
    }

    #[test]
    fn test_limit_error() {
        let c = ConstraintSliderDef::new(0, 1, [1.0,0.0,0.0], -1.0, 1.0);
        assert!((c.limit_error([0.0;3], [0.5,0.0,0.0])).abs() < 0.001);
        assert!((c.limit_error([0.0;3], [2.0,0.0,0.0]) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_lateral_error() {
        let c = ConstraintSliderDef::new(0, 1, [1.0,0.0,0.0], -1.0, 1.0);
        assert!((c.lateral_error([0.0;3], [1.0,3.0,4.0]) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_correction_force() {
        let c = ConstraintSliderDef::new(0, 1, [1.0,0.0,0.0], -1.0, 1.0).with_stiffness(10.0);
        let f = c.correction_force([0.0;3], [2.0,0.0,0.0]);
        assert!(f < 0.0); // pushes back
    }

    #[test]
    fn test_travel_range() {
        let c = ConstraintSliderDef::new(0, 1, [1.0,0.0,0.0], -2.0, 3.0);
        assert!((c.travel_range() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_normalized_position() {
        let c = ConstraintSliderDef::new(0, 1, [1.0,0.0,0.0], 0.0, 10.0);
        let n = c.normalized_position([0.0;3], [5.0,0.0,0.0]);
        assert!((n - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_energy_in_limits() {
        let c = ConstraintSliderDef::new(0, 1, [1.0,0.0,0.0], -1.0, 1.0);
        assert!(c.energy([0.0;3], [0.5,0.0,0.0]) < 0.001);
    }

    #[test]
    fn test_damping_force() {
        let c = ConstraintSliderDef::new(0, 1, [1.0,0.0,0.0], -1.0, 1.0).with_damping(5.0);
        assert!((c.damping_force(2.0) - (-10.0)).abs() < 0.01);
    }

    #[test]
    fn test_energy_exceeded() {
        let c = ConstraintSliderDef::new(0, 1, [1.0,0.0,0.0], -1.0, 1.0).with_stiffness(10.0);
        let e = c.energy([0.0;3], [3.0,0.0,0.0]);
        assert!(e > 0.0);
    }
}
