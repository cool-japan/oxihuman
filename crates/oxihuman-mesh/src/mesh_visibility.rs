// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Per-face and per-vertex visibility classification for a mesh.

use std::f32::consts::PI;

/// Visibility of a face relative to a view direction.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceVisibility {
    FrontFacing,
    BackFacing,
    Grazing,
}

/// Result of a mesh visibility pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VisibilityResult {
    /// Per-face classification.
    pub face_vis: Vec<FaceVisibility>,
    /// Number of front-facing faces.
    pub front_count: usize,
    /// Number of back-facing faces.
    pub back_count: usize,
}

/// Compute a face normal (not normalised).
#[allow(dead_code)]
pub fn face_normal_vis(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ]
}

/// Dot product of two 3-D vectors.
#[allow(dead_code)]
pub fn dot3_vis(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Classify every triangle against a view direction.
/// `grazing_threshold` is the cosine below which a face is considered grazing
/// (typically a small positive value like 0.05).
#[allow(dead_code)]
pub fn classify_visibility(
    positions: &[[f32; 3]],
    indices: &[u32],
    view_dir: [f32; 3],
    grazing_threshold: f32,
) -> VisibilityResult {
    let tri_count = indices.len() / 3;
    let mut face_vis = Vec::with_capacity(tri_count);
    let mut front_count = 0usize;
    let mut back_count = 0usize;

    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            face_vis.push(FaceVisibility::BackFacing);
            back_count += 1;
            continue;
        }
        let n = face_normal_vis(positions[i0], positions[i1], positions[i2]);
        let dot = dot3_vis(n, view_dir);
        let vis = if dot.abs() <= grazing_threshold {
            FaceVisibility::Grazing
        } else if dot > 0.0 {
            FaceVisibility::FrontFacing
        } else {
            FaceVisibility::BackFacing
        };
        match vis {
            FaceVisibility::FrontFacing => front_count += 1,
            FaceVisibility::BackFacing => back_count += 1,
            FaceVisibility::Grazing => {}
        }
        face_vis.push(vis);
    }
    VisibilityResult {
        face_vis,
        front_count,
        back_count,
    }
}

/// Return the fraction of faces that are front-facing.
#[allow(dead_code)]
pub fn front_facing_ratio(result: &VisibilityResult) -> f32 {
    let total = result.face_vis.len();
    if total == 0 {
        return 0.0;
    }
    result.front_count as f32 / total as f32
}

/// Return the angle (degrees) corresponding to the grazing threshold cosine.
#[allow(dead_code)]
pub fn grazing_angle_deg(threshold: f32) -> f32 {
    threshold.clamp(-1.0, 1.0).acos() * 180.0 / PI
}

/// Serialize the result to a brief JSON string.
#[allow(dead_code)]
pub fn visibility_to_json(result: &VisibilityResult) -> String {
    format!(
        "{{\"front\":{},\"back\":{},\"grazing\":{}}}",
        result.front_count,
        result.back_count,
        result.face_vis.len() - result.front_count - result.back_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn test_face_normal_z_up() {
        let n = face_normal_vis([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(n[2] > 0.0);
    }

    #[test]
    fn test_dot3_vis() {
        let d = dot3_vis([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((d - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_classify_front_facing() {
        let pos = quad_positions();
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        let view = [0.0, 0.0, 1.0];
        let r = classify_visibility(&pos, &idx, view, 0.01);
        assert!(r.front_count > 0);
    }

    #[test]
    fn test_classify_back_facing() {
        let pos = quad_positions();
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        let view = [0.0, 0.0, -1.0];
        let r = classify_visibility(&pos, &idx, view, 0.01);
        assert!(r.back_count > 0);
    }

    #[test]
    fn test_front_facing_ratio_full() {
        let r = VisibilityResult {
            face_vis: vec![FaceVisibility::FrontFacing; 4],
            front_count: 4,
            back_count: 0,
        };
        assert!((front_facing_ratio(&r) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_front_facing_ratio_empty() {
        let r = VisibilityResult {
            face_vis: vec![],
            front_count: 0,
            back_count: 0,
        };
        assert!((front_facing_ratio(&r) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_grazing_angle_deg_zero() {
        let ang = grazing_angle_deg(1.0);
        assert!(ang.abs() < 1e-4);
    }

    #[test]
    fn test_grazing_angle_deg_ninety() {
        let ang = grazing_angle_deg(0.0);
        assert!((ang - 90.0).abs() < 1e-3);
    }

    #[test]
    fn test_visibility_to_json() {
        let r = VisibilityResult {
            face_vis: vec![FaceVisibility::FrontFacing],
            front_count: 1,
            back_count: 0,
        };
        let j = visibility_to_json(&r);
        assert!(j.contains("\"front\":1"));
    }

    #[test]
    fn test_empty_mesh() {
        let r = classify_visibility(&[], &[], [0.0, 0.0, 1.0], 0.01);
        assert_eq!(r.face_vis.len(), 0);
    }
}
