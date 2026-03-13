// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Project a polyline curve onto a mesh surface stub.

/// A 3-D polyline curve (sequence of control points).
#[derive(Debug, Clone, Default)]
pub struct Curve3D {
    pub points: Vec<[f32; 3]>,
}

impl Curve3D {
    pub fn new(points: Vec<[f32; 3]>) -> Self {
        Self { points }
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// Result of projecting a curve onto a mesh.
#[derive(Debug, Clone, Default)]
pub struct ProjectedCurve {
    pub projected_points: Vec<[f32; 3]>,
    pub triangle_indices: Vec<u32>,
}

/// Project each curve point onto the nearest triangle in the mesh.
pub fn project_curve_onto_mesh(
    curve: &Curve3D,
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
) -> ProjectedCurve {
    /* Stub: project each point to the closest triangle centroid */
    let mut projected_points = Vec::with_capacity(curve.points.len());
    let mut triangle_indices = Vec::with_capacity(curve.points.len());

    for &cp in &curve.points {
        let (proj, tri_idx) = closest_point_on_mesh(cp, verts, tris);
        projected_points.push(proj);
        triangle_indices.push(tri_idx);
    }

    ProjectedCurve {
        projected_points,
        triangle_indices,
    }
}

/// Find the closest point on the mesh surface to a query point.
pub fn closest_point_on_mesh(
    query: [f32; 3],
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
) -> ([f32; 3], u32) {
    /* Stub: find triangle with nearest centroid */
    let mut best_dist = f32::MAX;
    let mut best_proj = query;
    let mut best_idx = 0u32;

    for (i, tri) in tris.iter().enumerate() {
        let mut centroid = [0.0f32; 3];
        for &idx in tri.iter() {
            let vi = idx as usize;
            if vi < verts.len() {
                for k in 0..3 {
                    centroid[k] += verts[vi][k];
                }
            }
        }
        centroid.iter_mut().for_each(|x| *x /= 3.0);
        let dist = dist3(query, centroid);
        if dist < best_dist {
            best_dist = dist;
            best_proj = centroid;
            best_idx = i as u32;
        }
    }
    (best_proj, best_idx)
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute the total arc length of a projected curve.
pub fn curve_arc_length(points: &[[f32; 3]]) -> f32 {
    points.windows(2).map(|w| dist3(w[0], w[1])).sum()
}

/// Resample a projected curve to a fixed number of points.
pub fn resample_curve(points: &[[f32; 3]], n: usize) -> Vec<[f32; 3]> {
    /* Stub: linearly interpolate n evenly-spaced points along the curve */
    if points.len() < 2 || n == 0 {
        return points.to_vec();
    }
    let total = curve_arc_length(points);
    if total < 1e-12 {
        return vec![points[0]; n];
    }
    let step = total / (n - 1).max(1) as f32;
    let mut result = Vec::with_capacity(n);
    result.push(points[0]);
    let mut dist_acc = 0.0f32;
    let mut seg = 0usize;
    let mut t_in_seg = 0.0f32;
    for i in 1..n {
        let target = i as f32 * step;
        while seg + 1 < points.len() {
            let seg_len = dist3(points[seg], points[seg + 1]);
            if dist_acc + seg_len >= target {
                t_in_seg = (target - dist_acc) / seg_len.max(1e-12);
                break;
            }
            dist_acc += seg_len;
            seg += 1;
            t_in_seg = 0.0;
        }
        let a = points[seg];
        let b = if seg + 1 < points.len() {
            points[seg + 1]
        } else {
            points[seg]
        };
        result.push([
            a[0] + t_in_seg * (b[0] - a[0]),
            a[1] + t_in_seg * (b[1] - a[1]),
            a[2] + t_in_seg * (b[2] - a[2]),
        ]);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curve3d_len() {
        let c = Curve3D::new(vec![[0.0f32; 3], [1.0, 0.0, 0.0]]);
        assert_eq!(c.len(), 2 /* two points */);
    }

    #[test]
    fn test_curve3d_is_empty() {
        let c = Curve3D::default();
        assert!(c.is_empty() /* default is empty */);
    }

    #[test]
    fn test_project_curve_empty_mesh() {
        let curve = Curve3D::new(vec![[0.0f32, 0.0, 0.0]]);
        let result = project_curve_onto_mesh(&curve, &[], &[]);
        assert!(result.projected_points[0].iter().all(|&x| x == 0.0) /* fallback to query */);
    }

    #[test]
    fn test_project_returns_one_per_point() {
        let curve = Curve3D::new(vec![[0.5f32, 0.5, 0.0], [1.5, 0.5, 0.0]]);
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let result = project_curve_onto_mesh(&curve, &verts, &tris);
        assert_eq!(
            result.projected_points.len(),
            2 /* one per curve point */
        );
    }

    #[test]
    fn test_arc_length_zero_for_single_point() {
        let pts = vec![[0.0f32; 3]];
        assert_eq!(curve_arc_length(&pts), 0.0 /* no segments */);
    }

    #[test]
    fn test_arc_length_straight_line() {
        let pts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let len = curve_arc_length(&pts);
        assert!((len - 2.0).abs() < 1e-5 /* total length = 2 */);
    }

    #[test]
    fn test_resample_preserves_start() {
        let pts = vec![[0.0f32, 0.0, 0.0], [4.0, 0.0, 0.0]];
        let resampled = resample_curve(&pts, 5);
        assert!(resampled[0]
            .iter()
            .zip([0.0f32, 0.0, 0.0].iter())
            .all(|(a, b)| (a - b).abs() < 1e-5) /* start unchanged */);
    }

    #[test]
    fn test_resample_count() {
        let pts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let resampled = resample_curve(&pts, 5);
        assert_eq!(resampled.len(), 5 /* correct count */);
    }

    #[test]
    fn test_dist3() {
        let d = dist3([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((d - 5.0).abs() < 1e-5 /* 3-4-5 triangle */);
    }
}
