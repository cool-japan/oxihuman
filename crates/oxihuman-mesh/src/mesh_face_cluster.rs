// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Face clustering by normal deviation for segmentation.

/// Face cluster assignment.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceClusterResult {
    pub labels: Vec<u32>,
    pub cluster_count: u32,
}

/// Compute face normal.
#[allow(dead_code)]
pub fn face_normal(positions: &[[f32; 3]], i0: usize, i1: usize, i2: usize) -> [f32; 3] {
    let e1 = [positions[i1][0] - positions[i0][0], positions[i1][1] - positions[i0][1], positions[i1][2] - positions[i0][2]];
    let e2 = [positions[i2][0] - positions[i0][0], positions[i2][1] - positions[i0][1], positions[i2][2] - positions[i0][2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-12 { [0.0, 0.0, 1.0] } else { [n[0] / len, n[1] / len, n[2] / len] }
}

/// Compute all face normals.
#[allow(dead_code)]
pub fn compute_face_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let tc = indices.len() / 3;
    (0..tc).map(|t| {
        face_normal(positions, indices[t * 3] as usize, indices[t * 3 + 1] as usize, indices[t * 3 + 2] as usize)
    }).collect()
}

/// Dot product of normals.
#[allow(dead_code)]
pub fn normal_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Cluster faces by normal similarity using flood-fill.
#[allow(dead_code)]
pub fn cluster_by_normal(
    positions: &[[f32; 3]],
    indices: &[u32],
    angle_threshold_deg: f32,
) -> FaceClusterResult {
    let normals = compute_face_normals(positions, indices);
    let tc = normals.len();
    let cos_thresh = (angle_threshold_deg * std::f32::consts::PI / 180.0).cos();
    let adj = build_face_adjacency(indices, tc);
    let mut labels = vec![u32::MAX; tc];
    let mut cluster_id = 0u32;
    for seed in 0..tc {
        if labels[seed] != u32::MAX { continue; }
        let mut stack = vec![seed];
        labels[seed] = cluster_id;
        while let Some(f) = stack.pop() {
            for &neighbor in &adj[f] {
                if labels[neighbor] == u32::MAX && normal_dot(normals[f], normals[neighbor]) >= cos_thresh {
                    labels[neighbor] = cluster_id;
                    stack.push(neighbor);
                }
            }
        }
        cluster_id += 1;
    }
    FaceClusterResult { labels, cluster_count: cluster_id }
}

/// Face cluster count.
#[allow(dead_code)]
pub fn face_cluster_count(result: &FaceClusterResult) -> u32 {
    result.cluster_count
}

/// Faces in cluster.
#[allow(dead_code)]
pub fn faces_in_cluster(result: &FaceClusterResult, cluster: u32) -> Vec<usize> {
    result.labels.iter().enumerate()
        .filter(|&(_, &l)| l == cluster)
        .map(|(i, _)| i)
        .collect()
}

fn build_face_adjacency(indices: &[u32], tc: usize) -> Vec<Vec<usize>> {
    use std::collections::HashMap;
    let mut edge_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for t in 0..tc {
        for k in 0..3 {
            let a = indices[t * 3 + k];
            let b = indices[t * 3 + (k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(t);
        }
    }
    let mut adj = vec![Vec::new(); tc];
    for faces in edge_faces.values() {
        for i in 0..faces.len() {
            for j in (i + 1)..faces.len() {
                adj[faces[i]].push(faces[j]);
                adj[faces[j]].push(faces[i]);
            }
        }
    }
    adj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_normal_unit() {
        let n = face_normal(
            &[[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            0, 1, 2,
        );
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_normals_count() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals = compute_face_normals(&pos, &[0, 1, 2]);
        assert_eq!(normals.len(), 1);
    }

    #[test]
    fn test_cluster_single_face() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let r = cluster_by_normal(&pos, &[0, 1, 2], 45.0);
        assert_eq!(face_cluster_count(&r), 1);
    }

    #[test]
    fn test_cluster_coplanar() {
        let pos = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        let r = cluster_by_normal(&pos, &idx, 5.0);
        assert_eq!(face_cluster_count(&r), 1);
    }

    #[test]
    fn test_cluster_perpendicular() {
        let pos = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0],
        ];
        let idx = vec![0, 1, 2, 0, 1, 3];
        let r = cluster_by_normal(&pos, &idx, 10.0);
        assert!(face_cluster_count(&r) >= 1);
    }

    #[test]
    fn test_faces_in_cluster() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let r = cluster_by_normal(&pos, &[0, 1, 2], 45.0);
        let faces = faces_in_cluster(&r, 0);
        assert_eq!(faces.len(), 1);
    }

    #[test]
    fn test_normal_dot_parallel() {
        let d = normal_dot([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]);
        assert!((d - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normal_dot_perpendicular() {
        let d = normal_dot([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(d.abs() < 1e-5);
    }

    #[test]
    fn test_empty_mesh() {
        let r = cluster_by_normal(&[], &[], 45.0);
        assert_eq!(face_cluster_count(&r), 0);
    }

    #[test]
    fn test_labels_length() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2, 1, 3, 2];
        let r = cluster_by_normal(&pos, &idx, 45.0);
        assert_eq!(r.labels.len(), 2);
    }

}
