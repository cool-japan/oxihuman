// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Face centroid computation and analysis.

/// Per-face centroid data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceCentroidData {
    pub centroids: Vec<[f32; 3]>,
}

/// Compute the centroid of a single triangle.
#[allow(dead_code)]
pub fn triangle_centroid(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    [
        (v0[0] + v1[0] + v2[0]) / 3.0,
        (v0[1] + v1[1] + v2[1]) / 3.0,
        (v0[2] + v1[2] + v2[2]) / 3.0,
    ]
}

/// Compute centroids for all faces.
#[allow(dead_code)]
pub fn compute_all_face_centroids(positions: &[[f32; 3]], indices: &[u32]) -> FaceCentroidData {
    let tri_count = indices.len() / 3;
    let centroids = (0..tri_count)
        .map(|t| {
            triangle_centroid(
                positions[indices[t * 3] as usize],
                positions[indices[t * 3 + 1] as usize],
                positions[indices[t * 3 + 2] as usize],
            )
        })
        .collect();
    FaceCentroidData { centroids }
}

/// Number of face centroids.
#[allow(dead_code)]
pub fn face_centroid_count(data: &FaceCentroidData) -> usize {
    data.centroids.len()
}

/// Get centroid of a specific face.
#[allow(dead_code)]
pub fn get_face_centroid(data: &FaceCentroidData, face: usize) -> Option<[f32; 3]> {
    data.centroids.get(face).copied()
}

/// Distance between two centroids.
#[allow(dead_code)]
pub fn centroid_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Mean of all centroids (mesh center of faces).
#[allow(dead_code)]
pub fn mean_centroid(data: &FaceCentroidData) -> [f32; 3] {
    let n = data.centroids.len();
    if n == 0 {
        return [0.0; 3];
    }
    let s = data
        .centroids
        .iter()
        .fold([0.0f32; 3], |a, c| [a[0] + c[0], a[1] + c[1], a[2] + c[2]]);
    [s[0] / n as f32, s[1] / n as f32, s[2] / n as f32]
}

/// Find the face whose centroid is closest to a given point.
#[allow(dead_code)]
pub fn nearest_face_centroid(data: &FaceCentroidData, point: [f32; 3]) -> Option<usize> {
    if data.centroids.is_empty() {
        return None;
    }
    let mut best = 0;
    let mut best_d = f32::MAX;
    for (i, c) in data.centroids.iter().enumerate() {
        let d = centroid_distance(*c, point);
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    Some(best)
}

/// Bounding box of all centroids.
#[allow(dead_code)]
pub fn centroid_bounds(data: &FaceCentroidData) -> ([f32; 3], [f32; 3]) {
    if data.centroids.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = data.centroids[0];
    let mut mx = data.centroids[0];
    for c in &data.centroids {
        for k in 0..3 {
            mn[k] = mn[k].min(c[k]);
            mx[k] = mx[k].max(c[k]);
        }
    }
    (mn, mx)
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn face_centroid_to_json(data: &FaceCentroidData) -> String {
    format!("{{\"face_centroid_count\":{}}}", data.centroids.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_centroid() {
        let c = triangle_centroid([0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_all() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let data = compute_all_face_centroids(&pos, &idx);
        assert_eq!(face_centroid_count(&data), 1);
    }

    #[test]
    fn test_get_face_centroid() {
        let pos = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
        let data = compute_all_face_centroids(&pos, &[0, 1, 2]);
        let c = get_face_centroid(&data, 0).unwrap();
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_get_face_centroid_oob() {
        let data = FaceCentroidData { centroids: vec![] };
        assert!(get_face_centroid(&data, 0).is_none());
    }

    #[test]
    fn test_centroid_distance() {
        let d = centroid_distance([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((d - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_mean_centroid() {
        let data = FaceCentroidData {
            centroids: vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
        };
        let m = mean_centroid(&data);
        assert!((m[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_mean_centroid_empty() {
        let data = FaceCentroidData { centroids: vec![] };
        let m = mean_centroid(&data);
        assert!((m[0]).abs() < 1e-6);
    }

    #[test]
    fn test_nearest_face_centroid() {
        let data = FaceCentroidData {
            centroids: vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]],
        };
        assert_eq!(nearest_face_centroid(&data, [1.0, 0.0, 0.0]), Some(0));
    }

    #[test]
    fn test_centroid_bounds() {
        let data = FaceCentroidData {
            centroids: vec![[1.0, 2.0, 3.0], [-1.0, 0.0, 5.0]],
        };
        let (mn, mx) = centroid_bounds(&data);
        assert!((mn[0] - (-1.0)).abs() < 1e-6);
        assert!((mx[2] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_face_centroid_to_json() {
        let data = FaceCentroidData {
            centroids: vec![[0.0; 3]],
        };
        let json = face_centroid_to_json(&data);
        assert!(json.contains("\"face_centroid_count\":1"));
    }
}
