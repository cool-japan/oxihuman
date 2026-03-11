// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

/// A single corrective shape driven by one or more joint angles
pub struct PoseCorrectiveShape {
    pub name: String,
    pub joint_name: String,
    pub axis: [f32; 3],
    pub angle_min: f32,
    pub angle_max: f32,
    pub deltas: Vec<(u32, [f32; 3])>,
}

impl PoseCorrectiveShape {
    pub fn new(name: impl Into<String>, joint_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            joint_name: joint_name.into(),
            axis: [0.0, 0.0, 1.0],
            angle_min: 0.0,
            angle_max: std::f32::consts::PI,
            deltas: Vec::new(),
        }
    }

    /// Compute weight [0..1] given current joint angle along axis
    pub fn weight(&self, current_angle: f32) -> f32 {
        angle_to_weight(
            current_angle,
            self.angle_min,
            self.angle_max,
            BlendInterpolation::Linear,
        )
    }
}

/// Joint rotation state: axis-angle representation
pub struct JointRotation {
    pub joint_name: String,
    pub axis: [f32; 3],
    pub angle: f32,
}

impl JointRotation {
    pub fn new(joint_name: impl Into<String>, axis: [f32; 3], angle: f32) -> Self {
        Self {
            joint_name: joint_name.into(),
            axis,
            angle,
        }
    }

    /// Angle projected onto a specific axis (dot product of axes * angle)
    pub fn projected_angle(&self, target_axis: [f32; 3]) -> f32 {
        let dot = self.axis[0] * target_axis[0]
            + self.axis[1] * target_axis[1]
            + self.axis[2] * target_axis[2];
        dot * self.angle
    }
}

/// Library of corrective shapes
pub struct PoseBlendLibrary {
    shapes: Vec<PoseCorrectiveShape>,
}

impl PoseBlendLibrary {
    pub fn new() -> Self {
        Self { shapes: Vec::new() }
    }

    pub fn add_shape(&mut self, shape: PoseCorrectiveShape) {
        self.shapes.push(shape);
    }

    pub fn shape_count(&self) -> usize {
        self.shapes.len()
    }

    pub fn shapes_for_joint(&self, joint_name: &str) -> Vec<&PoseCorrectiveShape> {
        self.shapes
            .iter()
            .filter(|s| s.joint_name == joint_name)
            .collect()
    }

    pub fn get_shape(&self, name: &str) -> Option<&PoseCorrectiveShape> {
        self.shapes.iter().find(|s| s.name == name)
    }

    pub fn remove_shape(&mut self, name: &str) -> bool {
        let before = self.shapes.len();
        self.shapes.retain(|s| s.name != name);
        self.shapes.len() < before
    }

    /// Compute active weights for all shapes given current joint rotations
    pub fn compute_weights<'a>(
        &'a self,
        rotations: &[JointRotation],
    ) -> Vec<(&'a PoseCorrectiveShape, f32)> {
        self.shapes
            .iter()
            .map(|shape| {
                let weight = rotations
                    .iter()
                    .find(|r| r.joint_name == shape.joint_name)
                    .map(|r| {
                        let projected = r.projected_angle(shape.axis);
                        shape.weight(projected)
                    })
                    .unwrap_or(0.0);
                (shape, weight)
            })
            .collect()
    }

    /// Apply all active corrective shapes to vertex positions.
    /// Returns new positions with corrections applied.
    pub fn apply_corrections(
        &self,
        positions: &[[f32; 3]],
        rotations: &[JointRotation],
    ) -> Vec<[f32; 3]> {
        let mut result = positions.to_vec();
        let weights = self.compute_weights(rotations);

        for (shape, w) in weights {
            if w <= 0.0 {
                continue;
            }
            for &(vi, [dx, dy, dz]) in &shape.deltas {
                let idx = vi as usize;
                if idx < result.len() {
                    result[idx][0] += dx * w;
                    result[idx][1] += dy * w;
                    result[idx][2] += dz * w;
                }
            }
        }

        result
    }
}

impl Default for PoseBlendLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Interpolation modes for weight mapping
pub enum BlendInterpolation {
    Linear,
    SmoothStep,
    Cubic,
}

/// Map angle to weight using specified interpolation
pub fn angle_to_weight(angle: f32, min: f32, max: f32, mode: BlendInterpolation) -> f32 {
    let range = max - min;
    let t = if range == 0.0 {
        0.0
    } else {
        ((angle - min) / range).clamp(0.0, 1.0)
    };

    match mode {
        BlendInterpolation::Linear => t,
        BlendInterpolation::SmoothStep => t * t * (3.0 - 2.0 * t),
        BlendInterpolation::Cubic => t * t * t * (10.0 - 15.0 * t + 6.0 * t * t),
    }
}

/// Create a common elbow corrective shape (example factory)
pub fn make_elbow_corrective(joint_name: impl Into<String>) -> PoseCorrectiveShape {
    PoseCorrectiveShape {
        name: "elbow_corrective".to_string(),
        joint_name: joint_name.into(),
        axis: [0.0, 0.0, 1.0],
        angle_min: 0.0,
        angle_max: std::f32::consts::PI * 0.8,
        deltas: Vec::new(),
    }
}

/// Create a common shoulder corrective shape
pub fn make_shoulder_corrective(joint_name: impl Into<String>) -> PoseCorrectiveShape {
    PoseCorrectiveShape {
        name: "shoulder_corrective".to_string(),
        joint_name: joint_name.into(),
        axis: [1.0, 0.0, 0.0],
        angle_min: 0.0,
        angle_max: std::f32::consts::FRAC_PI_2,
        deltas: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI};

    #[test]
    fn test_pose_corrective_shape_new() {
        let shape = PoseCorrectiveShape::new("test_shape", "elbow_joint");
        assert_eq!(shape.name, "test_shape");
        assert_eq!(shape.joint_name, "elbow_joint");
        assert_eq!(shape.axis, [0.0, 0.0, 1.0]);
        assert_eq!(shape.angle_min, 0.0);
        assert_eq!(shape.angle_max, PI);
        assert!(shape.deltas.is_empty());
    }

    #[test]
    fn test_weight_at_min_angle() {
        let shape = PoseCorrectiveShape {
            name: "s".to_string(),
            joint_name: "j".to_string(),
            axis: [0.0, 0.0, 1.0],
            angle_min: 0.0,
            angle_max: PI,
            deltas: Vec::new(),
        };
        let w = shape.weight(0.0);
        assert!((w - 0.0).abs() < 1e-6, "weight at min should be 0, got {w}");
    }

    #[test]
    fn test_weight_at_max_angle() {
        let shape = PoseCorrectiveShape {
            name: "s".to_string(),
            joint_name: "j".to_string(),
            axis: [0.0, 0.0, 1.0],
            angle_min: 0.0,
            angle_max: PI,
            deltas: Vec::new(),
        };
        let w = shape.weight(PI);
        assert!((w - 1.0).abs() < 1e-6, "weight at max should be 1, got {w}");
    }

    #[test]
    fn test_weight_midpoint() {
        let shape = PoseCorrectiveShape {
            name: "s".to_string(),
            joint_name: "j".to_string(),
            axis: [0.0, 0.0, 1.0],
            angle_min: 0.0,
            angle_max: PI,
            deltas: Vec::new(),
        };
        let w = shape.weight(PI / 2.0);
        assert!(
            (w - 0.5).abs() < 1e-5,
            "weight at midpoint should be 0.5, got {w}"
        );
    }

    #[test]
    fn test_weight_clamped_below() {
        let shape = PoseCorrectiveShape {
            name: "s".to_string(),
            joint_name: "j".to_string(),
            axis: [0.0, 0.0, 1.0],
            angle_min: 1.0,
            angle_max: 2.0,
            deltas: Vec::new(),
        };
        let w = shape.weight(-1.0);
        assert!(
            (w - 0.0).abs() < 1e-6,
            "weight below min should clamp to 0, got {w}"
        );
    }

    #[test]
    fn test_weight_clamped_above() {
        let shape = PoseCorrectiveShape {
            name: "s".to_string(),
            joint_name: "j".to_string(),
            axis: [0.0, 0.0, 1.0],
            angle_min: 0.0,
            angle_max: 1.0,
            deltas: Vec::new(),
        };
        let w = shape.weight(100.0);
        assert!(
            (w - 1.0).abs() < 1e-6,
            "weight above max should clamp to 1, got {w}"
        );
    }

    #[test]
    fn test_joint_rotation_projected_angle() {
        let rot = JointRotation::new("shoulder", [1.0, 0.0, 0.0], FRAC_PI_2);
        // Projection onto the same axis should give the full angle
        let proj = rot.projected_angle([1.0, 0.0, 0.0]);
        assert!(
            (proj - FRAC_PI_2).abs() < 1e-5,
            "projected angle should equal angle, got {proj}"
        );

        // Projection onto orthogonal axis should be 0
        let proj_orth = rot.projected_angle([0.0, 1.0, 0.0]);
        assert!(
            proj_orth.abs() < 1e-6,
            "orthogonal projection should be 0, got {proj_orth}"
        );
    }

    #[test]
    fn test_library_add_and_count() {
        let mut lib = PoseBlendLibrary::new();
        assert_eq!(lib.shape_count(), 0);
        lib.add_shape(PoseCorrectiveShape::new("s1", "j1"));
        lib.add_shape(PoseCorrectiveShape::new("s2", "j2"));
        assert_eq!(lib.shape_count(), 2);
    }

    #[test]
    fn test_library_shapes_for_joint() {
        let mut lib = PoseBlendLibrary::new();
        lib.add_shape(PoseCorrectiveShape::new("s1", "elbow"));
        lib.add_shape(PoseCorrectiveShape::new("s2", "elbow"));
        lib.add_shape(PoseCorrectiveShape::new("s3", "shoulder"));

        let elbow_shapes = lib.shapes_for_joint("elbow");
        assert_eq!(elbow_shapes.len(), 2);

        let shoulder_shapes = lib.shapes_for_joint("shoulder");
        assert_eq!(shoulder_shapes.len(), 1);

        let missing = lib.shapes_for_joint("knee");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_library_compute_weights() {
        let mut lib = PoseBlendLibrary::new();
        let mut shape = PoseCorrectiveShape::new("elbow_corr", "elbow");
        shape.angle_min = 0.0;
        shape.angle_max = PI;
        shape.axis = [0.0, 0.0, 1.0];
        lib.add_shape(shape);

        let rotations = vec![JointRotation::new("elbow", [0.0, 0.0, 1.0], PI / 2.0)];
        let weights = lib.compute_weights(&rotations);
        assert_eq!(weights.len(), 1);
        let (s, w) = &weights[0];
        assert_eq!(s.name, "elbow_corr");
        assert!((w - 0.5).abs() < 1e-5, "expected weight ~0.5, got {w}");
    }

    #[test]
    fn test_library_apply_corrections() {
        let mut lib = PoseBlendLibrary::new();
        let mut shape = PoseCorrectiveShape::new("corr", "elbow");
        shape.angle_min = 0.0;
        shape.angle_max = PI;
        shape.axis = [0.0, 0.0, 1.0];
        // vertex 0 gets +1 on x at full weight
        shape.deltas = vec![(0, [1.0, 0.0, 0.0])];
        lib.add_shape(shape);

        let positions = vec![[0.0_f32, 0.0, 0.0], [1.0, 1.0, 1.0]];
        // Full angle => weight = 1.0
        let rotations = vec![JointRotation::new("elbow", [0.0, 0.0, 1.0], PI)];
        let result = lib.apply_corrections(&positions, &rotations);
        assert_eq!(result.len(), 2);
        assert!(
            (result[0][0] - 1.0).abs() < 1e-5,
            "vertex 0 x should be 1.0, got {}",
            result[0][0]
        );
        assert!((result[0][1]).abs() < 1e-5);
        assert!(
            (result[1][0] - 1.0).abs() < 1e-5,
            "vertex 1 x unchanged, got {}",
            result[1][0]
        );
    }

    #[test]
    fn test_angle_to_weight_linear() {
        let w0 = angle_to_weight(0.0, 0.0, 1.0, BlendInterpolation::Linear);
        let w1 = angle_to_weight(1.0, 0.0, 1.0, BlendInterpolation::Linear);
        let wh = angle_to_weight(0.5, 0.0, 1.0, BlendInterpolation::Linear);
        assert!((w0 - 0.0).abs() < 1e-6);
        assert!((w1 - 1.0).abs() < 1e-6);
        assert!((wh - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_angle_to_weight_smoothstep() {
        let w0 = angle_to_weight(0.0, 0.0, 1.0, BlendInterpolation::SmoothStep);
        let w1 = angle_to_weight(1.0, 0.0, 1.0, BlendInterpolation::SmoothStep);
        let wh = angle_to_weight(0.5, 0.0, 1.0, BlendInterpolation::SmoothStep);
        assert!((w0 - 0.0).abs() < 1e-6, "smoothstep at 0 should be 0");
        assert!((w1 - 1.0).abs() < 1e-6, "smoothstep at 1 should be 1");
        // smoothstep(0.5) = 0.5*0.5*(3 - 2*0.5) = 0.25 * 2 = 0.5
        assert!(
            (wh - 0.5).abs() < 1e-6,
            "smoothstep at 0.5 should be 0.5, got {wh}"
        );
    }

    #[test]
    fn test_make_elbow_corrective() {
        let shape = make_elbow_corrective("elbow_L");
        assert_eq!(shape.joint_name, "elbow_L");
        assert_eq!(shape.axis, [0.0, 0.0, 1.0]);
        assert!((shape.angle_min - 0.0).abs() < 1e-6);
        assert!((shape.angle_max - PI * 0.8).abs() < 1e-5);
        assert!(shape.deltas.is_empty());
    }

    #[test]
    fn test_make_shoulder_corrective() {
        let shape = make_shoulder_corrective("shoulder_R");
        assert_eq!(shape.joint_name, "shoulder_R");
        assert_eq!(shape.axis, [1.0, 0.0, 0.0]);
        assert!((shape.angle_min - 0.0).abs() < 1e-6);
        assert!((shape.angle_max - FRAC_PI_2).abs() < 1e-5);
        assert!(shape.deltas.is_empty());
    }
}
