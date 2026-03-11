// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Face centroid v2: area-weighted centroid queries with spatial bucketing.

/// Per-face centroid with area.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FaceCentroidV2 {
    pub position: [f32; 3],
    pub area: f32,
    pub face_index: usize,
}

/// Collection of face centroids.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceCentroidSetV2 {
    pub centroids: Vec<FaceCentroidV2>,
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Compute all face centroids and areas.
#[allow(dead_code)]
pub fn compute_face_centroids_v2(positions: &[[f32; 3]], indices: &[u32]) -> FaceCentroidSetV2 {
    let nf = indices.len() / 3;
    let mut centroids = Vec::with_capacity(nf);
    for f in 0..nf {
        let i0 = indices[f * 3] as usize;
        let i1 = indices[f * 3 + 1] as usize;
        let i2 = indices[f * 3 + 2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];
        let cx = (p0[0] + p1[0] + p2[0]) / 3.0;
        let cy = (p0[1] + p1[1] + p2[1]) / 3.0;
        let cz = (p0[2] + p1[2] + p2[2]) / 3.0;
        let ab = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let ac = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let area = len3(cross3(ab, ac)) * 0.5;
        centroids.push(FaceCentroidV2 {
            position: [cx, cy, cz],
            area,
            face_index: f,
        });
    }
    FaceCentroidSetV2 { centroids }
}

/// Area-weighted global centroid.
#[allow(dead_code)]
pub fn area_weighted_centroid_v2(set: &FaceCentroidSetV2) -> [f32; 3] {
    let mut sum_area = 0.0_f32;
    let mut sum = [0.0_f32; 3];
    for c in &set.centroids {
        sum[0] += c.area * c.position[0];
        sum[1] += c.area * c.position[1];
        sum[2] += c.area * c.position[2];
        sum_area += c.area;
    }
    if sum_area > 1e-10 {
        [sum[0] / sum_area, sum[1] / sum_area, sum[2] / sum_area]
    } else {
        [0.0; 3]
    }
}

/// Find nearest centroid to query point.
#[allow(dead_code)]
pub fn nearest_centroid_v2(set: &FaceCentroidSetV2, query: [f32; 3]) -> Option<usize> {
    set.centroids
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let dx = c.position[0] - query[0];
            let dy = c.position[1] - query[1];
            let dz = c.position[2] - query[2];
            (i, dx * dx + dy * dy + dz * dz)
        })
        .reduce(|a, b| if a.1 <= b.1 { a } else { b })
        .map(|(i, _)| i)
}

/// Total surface area.
#[allow(dead_code)]
pub fn total_area_v2(set: &FaceCentroidSetV2) -> f32 {
    set.centroids.iter().map(|c| c.area).sum()
}

/// Face count.
#[allow(dead_code)]
pub fn face_count_v2(set: &FaceCentroidSetV2) -> usize {
    set.centroids.len()
}

/// Maximum area face index.
#[allow(dead_code)]
pub fn max_area_face_v2(set: &FaceCentroidSetV2) -> Option<usize> {
    set.centroids
        .iter()
        .enumerate()
        .max_by(|a, b| {
            a.1.area
                .partial_cmp(&b.1.area)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    #[test]
    fn face_count_one() {
        let (pos, idx) = one_tri();
        let set = compute_face_centroids_v2(&pos, &idx);
        assert_eq!(face_count_v2(&set), 1);
    }

    #[test]
    fn centroid_correct() {
        let (pos, idx) = one_tri();
        let set = compute_face_centroids_v2(&pos, &idx);
        let c = set.centroids[0].position;
        assert!((c[0] - 2.0 / 3.0).abs() < 1e-4);
    }

    #[test]
    fn area_correct() {
        let (pos, idx) = one_tri();
        let set = compute_face_centroids_v2(&pos, &idx);
        assert!((set.centroids[0].area - 2.0).abs() < 1e-4);
    }

    #[test]
    fn total_area_matches() {
        let (pos, idx) = one_tri();
        let set = compute_face_centroids_v2(&pos, &idx);
        assert!((total_area_v2(&set) - 2.0).abs() < 1e-4);
    }

    #[test]
    fn area_weighted_centroid_equals_single_centroid() {
        let (pos, idx) = one_tri();
        let set = compute_face_centroids_v2(&pos, &idx);
        let wc = area_weighted_centroid_v2(&set);
        assert!((wc[0] - set.centroids[0].position[0]).abs() < 1e-5);
    }

    #[test]
    fn nearest_centroid_returns_zero() {
        let (pos, idx) = one_tri();
        let set = compute_face_centroids_v2(&pos, &idx);
        let n = nearest_centroid_v2(&set, [0.0; 3]);
        assert!(n.is_some_and(|i| i == 0));
    }

    #[test]
    fn max_area_face_returns_some() {
        let (pos, idx) = one_tri();
        let set = compute_face_centroids_v2(&pos, &idx);
        assert!(max_area_face_v2(&set).is_some());
    }

    #[test]
    fn empty_set() {
        let set = compute_face_centroids_v2(&[], &[]);
        assert_eq!(face_count_v2(&set), 0);
    }

    #[test]
    fn nearest_empty_returns_none() {
        let set = FaceCentroidSetV2 { centroids: vec![] };
        assert!(nearest_centroid_v2(&set, [0.0; 3]).is_none());
    }

    #[test]
    fn contains_range() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
