// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Represents a contact between a point and a triangular face.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FaceContact {
    pub face_normal: [f32; 3],
    pub contact_point: [f32; 3],
    pub penetration: f32,
    pub face_index: u32,
}

#[allow(dead_code)]
impl FaceContact {
    pub fn new(face_normal: [f32; 3], contact_point: [f32; 3], penetration: f32, face_index: u32) -> Self {
        Self {
            face_normal,
            contact_point,
            penetration,
            face_index,
        }
    }

    pub fn is_penetrating(&self) -> bool {
        self.penetration > 0.0
    }

    pub fn separation_impulse(&self, stiffness: f32) -> [f32; 3] {
        let mag = self.penetration * stiffness;
        [
            self.face_normal[0] * mag,
            self.face_normal[1] * mag,
            self.face_normal[2] * mag,
        ]
    }

    pub fn compute_face_normal(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let cross = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        if len > 1e-9 {
            [cross[0] / len, cross[1] / len, cross[2] / len]
        } else {
            [0.0, 1.0, 0.0]
        }
    }

    pub fn point_to_plane_distance(point: [f32; 3], plane_point: [f32; 3], plane_normal: [f32; 3]) -> f32 {
        let d: f32 = (0..3)
            .map(|i| (point[i] - plane_point[i]) * plane_normal[i])
            .sum();
        d
    }

    pub fn barycentric(point: [f32; 3], v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
        let e0 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e1 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let vp = [point[0] - v0[0], point[1] - v0[1], point[2] - v0[2]];

        let d00: f32 = (0..3).map(|i| e0[i] * e0[i]).sum();
        let d01: f32 = (0..3).map(|i| e0[i] * e1[i]).sum();
        let d11: f32 = (0..3).map(|i| e1[i] * e1[i]).sum();
        let d20: f32 = (0..3).map(|i| vp[i] * e0[i]).sum();
        let d21: f32 = (0..3).map(|i| vp[i] * e1[i]).sum();

        let denom = d00 * d11 - d01 * d01;
        if denom.abs() < 1e-12 {
            return [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
        }
        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = 1.0 - v - w;
        [u, v, w]
    }

    pub fn is_inside_triangle(bary: [f32; 3]) -> bool {
        (0.0..=1.0).contains(&bary[0])
            && (0.0..=1.0).contains(&bary[1])
            && (0.0..=1.0).contains(&bary[2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let fc = FaceContact::new([0.0, 1.0, 0.0], [0.0; 3], 0.1, 0);
        assert!(fc.is_penetrating());
    }

    #[test]
    fn test_not_penetrating() {
        let fc = FaceContact::new([0.0, 1.0, 0.0], [0.0; 3], -0.1, 0);
        assert!(!fc.is_penetrating());
    }

    #[test]
    fn test_separation_impulse() {
        let fc = FaceContact::new([0.0, 1.0, 0.0], [0.0; 3], 2.0, 0);
        let imp = fc.separation_impulse(10.0);
        assert!((imp[1] - 20.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_face_normal() {
        let n = FaceContact::compute_face_normal(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
        );
        // Cross of (1,0,0) x (0,0,1) = (0,-1,0), normalized = (0,-1,0)
        assert!((n[1].abs() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_point_to_plane_distance() {
        let d = FaceContact::point_to_plane_distance(
            [0.0, 5.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!((d - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_barycentric_center() {
        let bary = FaceContact::barycentric(
            [1.0 / 3.0, 0.0, 1.0 / 3.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
        );
        for &b in &bary {
            assert!((b - 1.0 / 3.0).abs() < 1e-4);
        }
    }

    #[test]
    fn test_barycentric_vertex() {
        let bary = FaceContact::barycentric(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
        );
        assert!((bary[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_inside_triangle() {
        assert!(FaceContact::is_inside_triangle([0.5, 0.25, 0.25]));
        assert!(!FaceContact::is_inside_triangle([-0.1, 0.6, 0.5]));
    }

    #[test]
    fn test_face_index() {
        let fc = FaceContact::new([0.0, 1.0, 0.0], [0.0; 3], 0.0, 42);
        assert_eq!(fc.face_index, 42);
    }

    #[test]
    fn test_zero_penetration() {
        let fc = FaceContact::new([0.0, 1.0, 0.0], [0.0; 3], 0.0, 0);
        assert!(!fc.is_penetrating());
    }
}
