// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Sharp edge detection by dihedral angle threshold.

use std::f32::consts::PI;

#[allow(dead_code)]
pub struct SharpEdgeDetectorV2 {
    pub threshold_rad: f32,
    pub sharp_edges: Vec<[u32; 2]>,
}

#[allow(dead_code)]
pub fn new_sharp_edge_detector_v2(threshold_deg: f32) -> SharpEdgeDetectorV2 {
    SharpEdgeDetectorV2 {
        threshold_rad: threshold_deg * PI / 180.0,
        sharp_edges: Vec::new(),
    }
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 { return [0.0; 3]; }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn face_normal(positions: &[[f32; 3]], tri: [u32; 3]) -> [f32; 3] {
    let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
    if a >= positions.len() || b >= positions.len() || c >= positions.len() {
        return [0.0, 0.0, 1.0];
    }
    let ab = [positions[b][0] - positions[a][0], positions[b][1] - positions[a][1], positions[b][2] - positions[a][2]];
    let ac = [positions[c][0] - positions[a][0], positions[c][1] - positions[a][1], positions[c][2] - positions[a][2]];
    normalize(cross(ab, ac))
}

#[allow(dead_code)]
pub fn dihedral_angle_between_v2(n1: [f32; 3], n2: [f32; 3]) -> f32 {
    let dot = (n1[0] * n2[0] + n1[1] * n2[1] + n1[2] * n2[2]).clamp(-1.0, 1.0);
    dot.acos()
}

#[allow(dead_code)]
pub fn sed_detect_v2(det: &mut SharpEdgeDetectorV2, positions: &[[f32; 3]], indices: &[[u32; 3]]) {
    det.sharp_edges.clear();
    let n = indices.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let ni = face_normal(positions, indices[i]);
            let nj = face_normal(positions, indices[j]);
            let angle = dihedral_angle_between_v2(ni, nj);
            if angle > det.threshold_rad {
                let shared_verts: Vec<u32> = indices[i].iter().filter(|v| indices[j].contains(v)).copied().collect();
                if shared_verts.len() >= 2 {
                    det.sharp_edges.push([shared_verts[0], shared_verts[1]]);
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn sed_sharp_count_v2(det: &SharpEdgeDetectorV2) -> usize { det.sharp_edges.len() }

#[allow(dead_code)]
pub fn sed_threshold_rad_v2(det: &SharpEdgeDetectorV2) -> f32 { det.threshold_rad }

#[allow(dead_code)]
pub fn sed_clear_v2(det: &mut SharpEdgeDetectorV2) { det.sharp_edges.clear(); }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_rad() {
        let det = new_sharp_edge_detector_v2(90.0);
        assert!((det.threshold_rad - PI / 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_sharp_count_flat_mesh() {
        let mut det = new_sharp_edge_detector_v2(30.0);
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        let indices = vec![[0u32, 1, 2], [1, 3, 2]];
        sed_detect_v2(&mut det, &positions, &indices);
        assert_eq!(sed_sharp_count_v2(&det), 0);
    }

    #[test]
    fn test_clear() {
        let mut det = new_sharp_edge_detector_v2(10.0);
        det.sharp_edges.push([0, 1]);
        sed_clear_v2(&mut det);
        assert_eq!(sed_sharp_count_v2(&det), 0);
    }

    #[test]
    fn test_dihedral_same_normal() {
        let n = [0.0f32, 0.0, 1.0];
        let angle = dihedral_angle_between_v2(n, n);
        assert!(angle.abs() < 1e-4);
    }

    #[test]
    fn test_dihedral_opposite_normals() {
        let n1 = [0.0f32, 0.0, 1.0];
        let n2 = [0.0f32, 0.0, -1.0];
        let angle = dihedral_angle_between_v2(n1, n2);
        assert!((angle - PI).abs() < 1e-4);
    }

    #[test]
    fn test_new_empty() {
        let det = new_sharp_edge_detector_v2(45.0);
        assert_eq!(sed_sharp_count_v2(&det), 0);
    }

    #[test]
    fn test_detect_empty_mesh() {
        let mut det = new_sharp_edge_detector_v2(45.0);
        sed_detect_v2(&mut det, &[], &[]);
        assert_eq!(sed_sharp_count_v2(&det), 0);
    }

    #[test]
    fn test_threshold_180_degrees() {
        let det = new_sharp_edge_detector_v2(180.0);
        assert!((det.threshold_rad - PI).abs() < 1e-4);
    }
}
